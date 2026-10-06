//! Chromium backend over the Chrome DevTools Protocol (CDP).
//!
//! Discovers and launches a Chromium-family browser (Chrome, Edge, Chromium)
//! with `--remote-debugging-port` and speaks CDP through the shared
//! [`xcelerate_core::CdpClient`].

use crate::CdpClient;
use crate::error::{XcelerateError, XcelerateResult};
use crate::page::Page;
use crate::plugin::{Plugin, PluginHandle, PluginManager};
use crate::process::{ProcessGuard, spawn_detached};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use super::known::Engine;
use xcelerate_plugin::LaunchPlan;

/// Configuration for the Browser instance.
#[derive(uniffi::Record)]
pub struct BrowserConfig {
    /// Whether to run the browser in headless mode.
    pub headless: bool,
    /// Whether to run the browser as a detached process.
    pub detached: bool,
    /// Optional path to the browser executable.
    pub executable_path: Option<String>,
    /// External plugins to load at launch. Each entry is a path to a plugin
    /// directory or a `plugin.json`.
    ///
    /// Xcelerate ships **no** plugins built into the core. Default-deny: no
    /// plugin does anything unless it is listed here (or installed afterwards
    /// with `Browser::use_plugin` / `Browser::load_plugin`).
    pub plugins: Option<Vec<String>>,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            headless: true,
            detached: true,
            executable_path: None,
            plugins: None,
        }
    }
}

/// Where the browser profile (cookies, logins, cache) lives.
enum Profile {
    /// A throwaway directory removed when the browser closes.
    Ephemeral(tempfile::TempDir),
    /// A user-supplied directory kept between runs.
    Persistent(PathBuf),
    /// No profile: used by [`Browser::connect`], which attaches to a browser it
    /// did not spawn and therefore has no profile directory of its own.
    None,
}

impl Profile {
    fn path(&self) -> &Path {
        match self {
            Profile::Ephemeral(dir) => dir.path(),
            Profile::Persistent(path) => path,
            Profile::None => Path::new(""),
        }
    }
}

/// A process-wide persistent profile directory (Rust-only; the other languages
/// set `XCELERATE_USER_DATA_DIR`).
static USER_DATA_DIR: std::sync::Mutex<Option<PathBuf>> = std::sync::Mutex::new(None);

/// Point the browser at a persistent profile directory so logins and cookies
/// survive restarts. `None` (or an empty string) restores the default throwaway
/// profile.
///
/// Rust-only: it is intentionally not exposed through the language bindings,
/// which set `XCELERATE_USER_DATA_DIR` instead.
pub fn configure_user_data_dir(path: Option<String>) -> XcelerateResult<()> {
    match path
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        Some(dir) => {
            let path = PathBuf::from(dir);
            std::fs::create_dir_all(&path).map_err(|error| {
                XcelerateError::NotFound(format!("cannot create user data dir: {error}"))
            })?;
            // Canonicalize: Chrome resolves a relative `--user-data-dir` against
            // its own cwd, which can silently hang or land somewhere else.
            *USER_DATA_DIR.lock().unwrap() = Some(path.canonicalize().unwrap_or(path));
        }
        None => *USER_DATA_DIR.lock().unwrap() = None,
    }
    Ok(())
}

/// The configured persistent profile: the Rust setter, then `XCELERATE_USER_DATA_DIR`.
fn configured_profile_dir() -> Option<PathBuf> {
    if let Some(path) = USER_DATA_DIR.lock().unwrap().clone() {
        return Some(path);
    }
    std::env::var("XCELERATE_USER_DATA_DIR")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

/// A running Chromium-family browser (Chrome, Chromium, Edge, Brave, …).
#[derive(uniffi::Object)]
pub struct Browser {
    pub(crate) client: Arc<CdpClient>,
    _process: tokio::sync::Mutex<Option<tokio::process::Child>>,
    _process_guard: Option<ProcessGuard>,
    _profile: Profile,
    pub(crate) plugins: Arc<PluginManager>,
    _proxy: Option<crate::proxy::ProxyGateway>,
    ws_url: String,
    events: tokio::sync::Mutex<Vec<String>>,
    /// Whether this handle owns the browser process. `true` for
    /// [`Browser::launch`], `false` for [`Browser::connect`]; an attached
    /// browser is never closed by [`Browser::close`].
    owned: bool,
}

#[uniffi::export(async_runtime = "tokio")]
impl Browser {
    #[uniffi::constructor]
    pub async fn launch(config: BrowserConfig) -> XcelerateResult<Arc<Self>> {
        let exe = super::known::resolve(config.executable_path.as_deref(), Engine::Chromium)
            .ok_or_else(|| {
                XcelerateError::NotFound(format!(
                    "no Chromium-based browser found; set executable_path or XCELERATE_BROWSER\
                     (try one of: {})",
                    super::known::ids().collect::<Vec<_>>().join(", ")
                ))
            })?;

        // 1. Setup environment
        let profile = match configured_profile_dir() {
            Some(path) => {
                std::fs::create_dir_all(&path).map_err(|error| {
                    XcelerateError::NotFound(format!("cannot use user data dir: {error}"))
                })?;
                Profile::Persistent(path.canonicalize().unwrap_or(path))
            }
            None => {
                Profile::Ephemeral(tempfile::tempdir().map_err(|_| XcelerateError::InternalError)?)
            }
        };
        let port = super::engine::free_port().ok_or(XcelerateError::InternalError)?;

        // Plugins are external and default-deny; nothing is enabled yet.
        let manager = PluginManager::new(&[], crate::plugin::catalog())?;

        // Let built-in plugins contribute to the launch (e.g. binary patching)
        // before the process is spawned.
        let mut plan = LaunchPlan::new(exe, config.headless, config.detached);
        manager.configure_launch(&mut plan)?;
        manager.mark_launched();

        // 2. Spawn process
        let mut cmd = std::process::Command::new(&plan.executable);
        setup_browser_args(&mut cmd, profile.path(), port, plan.headless);
        // A configured proxy pool is served by a local gateway that Chrome points
        // at; the gateway adds upstream credentials Chrome cannot carry.
        let proxy_gateway = match crate::proxy::start_if_configured().await? {
            Some(gateway) => {
                cmd.arg(format!("--proxy-server={}", gateway.url()));
                cmd.arg("--proxy-bypass-list=localhost;127.0.0.1");
                Some(gateway)
            }
            None => None,
        };
        for arg in &plan.extra_args {
            cmd.arg(arg);
        }

        // Capability F: apply process-wide launch options just before spawn.
        let launch_options = crate::options::configured();
        if let Some(options) = &launch_options {
            for arg in &options.extra_args {
                cmd.arg(arg);
            }
            if options.deterministic_rendering {
                cmd.args(deterministic_flags());
            }
            if options.disable_security {
                cmd.args(disable_security_flags());
            }
        }
        // `keep_alive` disables the auto-kill guard so the browser outlives the
        // handle. Detached processes are already never auto-killed.
        let keep_alive = launch_options
            .as_ref()
            .is_some_and(|options| options.keep_alive);

        let (mut child, guard) = if plan.detached {
            let pid = spawn_detached(cmd)?;
            let guard = ProcessGuard {
                pid,
                auto_kill: false,
            };
            (None, Some(guard))
        } else {
            let mut t_cmd = tokio::process::Command::from(cmd);
            let child = t_cmd
                .spawn()
                .map_err(|e| XcelerateError::NotFound(format!("Failed to start Chrome: {}", e)))?;
            let pid = child.id().ok_or(XcelerateError::InternalError)?;
            let guard = ProcessGuard {
                pid,
                auto_kill: !keep_alive,
            };
            (Some(child), Some(guard))
        };

        // 3. Connect to debugger
        let ws_url = wait_for_ws_url(port, child.as_mut()).await?;
        let client = Arc::new(xcelerate_core::connect(&ws_url).await?);

        // Capability F: downloads are opt-in; deny explicitly otherwise. Best
        // effort: a browser that rejects this must not fail the launch.
        if let Some(options) = &launch_options
            && let Some(accept) = options.accept_downloads
        {
            let behavior = if accept { "allow" } else { "deny" };
            let _ = client
                .execute_raw(
                    "Browser.setDownloadBehavior",
                    serde_json::json!({ "behavior": behavior }),
                )
                .await;
        }

        let browser = Arc::new(Self {
            client,
            _process: tokio::sync::Mutex::new(child),
            _process_guard: guard,
            _profile: profile,
            _proxy: proxy_gateway,
            plugins: Arc::new(manager),
            ws_url: ws_url.clone(),
            events: tokio::sync::Mutex::new(Vec::new()),
            owned: true,
        });

        // Load external plugins before any page is created, so their
        // `on_page_created` hook sees the first page.
        for path in config.plugins.clone().unwrap_or_default() {
            browser.load_plugin(path)?;
        }

        Ok(browser)
    }

    pub async fn new_page(self: Arc<Self>, url: String) -> XcelerateResult<Arc<Page>> {
        // 1. Create target with about:blank so we can inject scripts before loading the real URL
        let target = self
            .client
            .execute(browser_protocol::target::CreateTargetParams {
                url: "about:blank".into(),
                ..Default::default()
            })
            .await?;

        // 2. Attach to target
        let target_id = target.target_id.clone();
        let session = self
            .client
            .execute(browser_protocol::target::AttachToTargetParams {
                target_id: target.target_id,
                flatten: Some(true),
            })
            .await?;

        let page = self.build_page(session.session_id.into_owned(), target_id.into_owned());

        // 3. Run plugin page-created hooks (e.g. stealth payload injection).
        self.plugins
            .on_page_created(crate::plugin::page_host(Arc::clone(&page)))
            .await?;

        // 4. Finally navigate to the actual URL
        page.navigate(url).await?;

        Ok(page)
    }

    /// Names of the plugins currently available on this browser.
    ///
    /// Xcelerate ships **no** built-in plugins, so this lists the plugins that
    /// have been installed or loaded on this instance.
    pub fn available_plugins(&self) -> Vec<String> {
        self.plugins.names()
    }

    /// Names of the plugins currently enabled on this browser.
    pub fn plugin_names(&self) -> Vec<String> {
        self.plugins.names()
    }

    /// Enables an installed plugin at runtime.
    ///
    /// Launch-time contributions (such as binary patching) only take effect if
    /// the plugin was installed before the browser launched; enabling a plugin
    /// afterwards applies its runtime hooks to pages created from now on. This
    /// is audited as a runtime enable. Unknown names are refused.
    pub async fn use_plugin(&self, name: String) -> XcelerateResult<()> {
        self.plugins.enable(&name).map_err(XcelerateError::from)
    }

    /// Returns a handle to an enabled plugin so its ops can be invoked.
    pub fn plugin(&self, name: String) -> XcelerateResult<Arc<PluginHandle>> {
        if self.plugins.has(&name) {
            Ok(PluginHandle::new(Arc::clone(&self.plugins), name))
        } else {
            Err(XcelerateError::NotFound(format!(
                "plugin '{name}' is not enabled"
            )))
        }
    }

    /// Loads a plugin from disk.
    ///
    /// `path` may be a plugin directory (containing `plugin.json`) or a
    /// `plugin.json` file. The manifest is validated, the `entrypoint` is
    /// instantiated as a sandboxed WebAssembly component, and a `describe`
    /// handshake wires up its ops. Dangerous capabilities stay denied unless
    /// opted into via `XCELERATE_PLUGIN_ALLOW`.
    ///
    /// Once loaded, the plugin's ops are reachable through
    /// `plugin(name).invoke(op, args_json)` in every language.
    pub fn load_plugin(&self, path: String) -> XcelerateResult<String> {
        let manifest_path = crate::plugin::resolve_manifest_path(&path)?;
        let manifest = xcelerate_plugin::Manifest::load(&manifest_path.to_string_lossy())?;

        // The core ships no built-in plugins, so no name is reserved; the
        // manifest's own rules still apply.
        manifest
            .validate_reserved(&[])
            .map_err(|error| XcelerateError::Unsupported(error.to_string()))?;
        if self.plugins.has(&manifest.name) {
            return Err(XcelerateError::Unsupported(format!(
                "plugin '{}' is already loaded",
                manifest.name
            )));
        }

        #[cfg(feature = "wasm")]
        {
            let plugin_dir = manifest_path
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."));
            let plugin =
                crate::plugin::wasm::load(&manifest, plugin_dir, Some(Arc::clone(&self.client)))?;
            self.plugins.install(Arc::new(plugin))?;
            Ok(format!("loaded plugin '{}'", manifest.name))
        }
        #[cfg(not(feature = "wasm"))]
        {
            let _ = manifest;
            Err(XcelerateError::Unsupported(
                "plugin loading requires the `wasm` feature".to_string(),
            ))
        }
    }

    /// Verifies the integrity of the append-only plugin audit log.
    pub fn audit_verify(&self) -> bool {
        xcelerate_plugin::audit_verify()
    }

    /// Returns the plugin audit log as a JSON array (no secrets are recorded).
    pub fn audit_log(&self) -> String {
        let entries: Vec<serde_json::Value> = xcelerate_plugin::audit_entries()
            .into_iter()
            .map(|event| {
                serde_json::json!({
                    "seq": event.seq,
                    "plugin": event.plugin,
                    "action": event.action,
                    "detail": event.detail,
                    "hash": event.hash,
                })
            })
            .collect();
        serde_json::Value::Array(entries).to_string()
    }

    /// Returns the browser version information.
    pub async fn version(&self) -> XcelerateResult<String> {
        let res = self
            .client
            .execute(browser_protocol::browser::GetVersionParams {})
            .await?;
        Ok(format!(
            "{} (Protocol {})",
            res.product, res.protocol_version
        ))
    }

    /// Closes the browser, letting it flush the profile, then kills it if needed.
    ///
    /// An attached browser (from [`Browser::connect`]) is not owned by this
    /// handle, so this is a no-op: it never sends `Browser.close` and never
    /// kills the process.
    pub async fn close(&self) -> XcelerateResult<()> {
        if !self.owned {
            return Ok(());
        }
        // Ask the browser to close gracefully so it flushes cookies/storage to
        // the profile directory (critical for persistent profiles).
        let _ = self
            .client
            .execute(browser_protocol::browser::CloseParams {})
            .await;

        let mut lock = self._process.lock().await;
        if let Some(mut child) = lock.take() {
            // Wait briefly for the graceful exit; force-kill only if it overruns.
            if tokio::time::timeout(std::time::Duration::from_secs(5), child.wait())
                .await
                .is_err()
            {
                let _ = child.kill().await;
            }
        }
        Ok(())
    }

    /// Returns the current targets as a JSON array (`Target.getTargets`).
    pub async fn targets(&self) -> XcelerateResult<String> {
        let res = self
            .client
            .execute_raw("Target.getTargets", serde_json::json!({}))
            .await?;
        Ok(res.to_string())
    }

    /// Returns the browser context ids as a JSON array.
    pub async fn browser_contexts(&self) -> XcelerateResult<String> {
        let res = self
            .client
            .execute_raw("Target.getBrowserContexts", serde_json::json!({}))
            .await?;
        Ok(res.to_string())
    }

    /// Creates a new (incognito) browser context and returns its id.
    pub async fn new_context(&self) -> XcelerateResult<String> {
        let res = self
            .client
            .execute_raw("Target.createBrowserContext", serde_json::json!({}))
            .await?;
        Ok(res
            .get("browserContextId")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string())
    }

    /// Returns the browser's user agent.
    pub async fn user_agent(&self) -> XcelerateResult<String> {
        let res = self
            .client
            .execute_raw("Browser.getVersion", serde_json::json!({}))
            .await?;
        Ok(res
            .get("userAgent")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string())
    }

    /// Whether the underlying connection is alive.
    pub async fn is_connected(&self) -> bool {
        self.client.is_connected()
    }

    /// The WebSocket endpoint Chrome was launched with.
    pub fn ws_endpoint(&self) -> String {
        self.ws_url.clone()
    }

    /// Starts CDP tracing.
    pub async fn start_tracing(&self) -> XcelerateResult<()> {
        self.client
            .execute_raw(
                "Tracing.start",
                serde_json::json!({ "categories": "*", "transferMode": "ReportEvents" }),
            )
            .await?;
        Ok(())
    }

    /// Stops CDP tracing.
    pub async fn stop_tracing(&self) -> XcelerateResult<()> {
        self.client
            .execute_raw("Tracing.end", serde_json::json!({}))
            .await?;
        Ok(())
    }

    /// Grants permissions (JSON array) to an origin.
    pub async fn grant_permissions(
        &self,
        origin: String,
        permissions_json: String,
    ) -> XcelerateResult<()> {
        let permissions: Vec<serde_json::Value> =
            serde_json::from_str(&permissions_json).unwrap_or_default();
        self.client
            .execute_raw(
                "Browser.grantPermissions",
                serde_json::json!({ "origin": origin, "permissions": permissions }),
            )
            .await
            .map(|_| ())
            .map_err(XcelerateError::from)
    }

    /// Registers interest in a root-session CDP event.
    pub async fn on(&self, event_name: String) {
        let mut events = self.events.lock().await;
        if !events.contains(&event_name) {
            events.push(event_name);
        }
    }

    /// Alias for [`Browser::on`].
    pub async fn once(&self, event_name: String) {
        self.on(event_name).await;
    }

    /// Removes a single registered listener.
    pub async fn remove_listener(&self, event_name: String) {
        self.events.lock().await.retain(|name| name != &event_name);
    }

    /// Removes every registered listener.
    pub async fn remove_all_listeners(&self) {
        self.events.lock().await.clear();
    }

    /// Returns the registered event names.
    pub async fn event_names(&self) -> Vec<String> {
        self.events.lock().await.clone()
    }

    /// Whether an event name is registered.
    pub async fn listens_to(&self, event_name: String) -> bool {
        self.events.lock().await.contains(&event_name)
    }

    /// Waits for the next root-session CDP event named `event_name`.
    pub async fn wait_for_event(
        &self,
        event_name: String,
        timeout_ms: u64,
    ) -> XcelerateResult<String> {
        let domain = event_name.split('.').next().unwrap_or("");
        let enable: Option<(&str, serde_json::Value)> = match domain {
            "Target" => Some((
                "Target.setDiscoverTargets",
                serde_json::json!({ "discover": true }),
            )),
            "Browser" => Some(("Browser.setDownloadBehavior", serde_json::json!({}))),
            _ => None,
        };
        if let Some((method, params)) = enable {
            let _ = self.client.execute_raw(method, params).await;
        }

        let mut receiver = self.client.subscribe();
        let timeout = std::time::Duration::from_millis(timeout_ms.max(1));
        let start = std::time::Instant::now();
        loop {
            let remaining = timeout.saturating_sub(start.elapsed());
            if remaining.is_zero() {
                return Err(XcelerateError::NotFound(format!(
                    "Timeout waiting for event: {event_name}"
                )));
            }
            match tokio::time::timeout(remaining, receiver.recv()).await {
                Ok(Ok(value)) => {
                    if value.get("method").and_then(|m| m.as_str()) == Some(event_name.as_str()) {
                        return Ok(value
                            .get("params")
                            .cloned()
                            .unwrap_or(serde_json::Value::Null)
                            .to_string());
                    }
                }
                Ok(Err(_)) => continue,
                Err(_) => {
                    return Err(XcelerateError::NotFound(format!(
                        "Timeout waiting for event: {event_name}"
                    )));
                }
            }
        }
    }

    /// [`Browser::wait_for_event`] with the default 30s timeout.
    pub async fn wait_for_event_default(&self, event_name: String) -> XcelerateResult<String> {
        self.wait_for_event(event_name, 30_000).await
    }

    /// Returns all browser cookies as a JSON array.
    pub async fn cookies(&self) -> XcelerateResult<String> {
        let res = self
            .client
            .execute_raw("Network.getCookies", serde_json::json!({}))
            .await?;
        Ok(res
            .get("cookies")
            .cloned()
            .unwrap_or(serde_json::json!([]))
            .to_string())
    }

    /// Sets a cookie from a JSON object.
    pub async fn set_cookie(&self, cookie_json: String) -> XcelerateResult<()> {
        let cookie: serde_json::Value =
            serde_json::from_str(&cookie_json).unwrap_or(serde_json::Value::Null);
        self.client.execute_raw("Network.setCookie", cookie).await?;
        Ok(())
    }

    /// Deletes cookies with the given name.
    pub async fn delete_cookie(&self, name: String) -> XcelerateResult<()> {
        self.client
            .execute_raw("Network.deleteCookies", serde_json::json!({ "name": name }))
            .await?;
        Ok(())
    }

    /// Returns the browser version info as JSON.
    pub async fn capabilities(&self) -> XcelerateResult<String> {
        let res = self
            .client
            .execute_raw("Browser.getVersion", serde_json::json!({}))
            .await?;
        Ok(res.to_string())
    }

    /// Resets all permission overrides.
    pub async fn reset_permissions(&self) -> XcelerateResult<()> {
        self.client
            .execute_raw("Browser.resetPermissions", serde_json::json!({}))
            .await?;
        Ok(())
    }

    /// Sets the download directory for the browser.
    pub async fn set_download_behavior(&self, path: String) -> XcelerateResult<()> {
        self.client
            .execute_raw(
                "Browser.setDownloadBehavior",
                serde_json::json!({ "behavior": "allow", "downloadPath": path }),
            )
            .await?;
        Ok(())
    }
}

/// Rust-only extension points, kept out of the `#[uniffi::export]` block so
/// their signatures never affect the language bindings.
impl Browser {
    /// Attaches to a browser that is already running and exposes a CDP
    /// WebSocket endpoint (capability A).
    ///
    /// Unlike [`Browser::launch`], this spawns no process and owns no lifecycle:
    /// the returned handle has no profile, no proxy gateway, and no plugins
    /// enabled. [`Browser::close`] on an attached handle is a no-op, so the
    /// running browser is never shut down.
    ///
    /// Rust-only: it is intentionally not exposed through the language
    /// bindings.
    pub async fn connect(ws_url: String) -> XcelerateResult<Arc<Self>> {
        let client = Arc::new(xcelerate_core::connect(&ws_url).await?);
        let plugins = Arc::new(PluginManager::new(&[], crate::plugin::catalog())?);
        Ok(Arc::new(Self {
            client,
            _process: tokio::sync::Mutex::new(None),
            _process_guard: None,
            _profile: Profile::None,
            _proxy: None,
            plugins,
            ws_url,
            events: tokio::sync::Mutex::new(Vec::new()),
            owned: false,
        }))
    }

    /// Installs trusted, in-process plugins, taken by value.
    ///
    /// This is how a plugin shipped as a Cargo library is added: `cargo add` the
    /// crate, then hand the plugin(s) here. The argument is anything iterable, so
    /// a single plugin and a batch are both one call:
    ///
    /// ```ignore
    /// browser.install_plugins([MyPlugin])?;
    /// browser.install_plugins([PluginA, PluginB])?;
    /// ```
    ///
    /// Installed plugins run in-process and are fully trusted: they receive the
    /// whole `PageHost` interface. Install them before creating pages so their
    /// `on_page_created` hook sees them.
    ///
    /// Rust-only: it is intentionally not exposed through the language bindings,
    /// because it hands the host an executable Rust value.
    pub fn install_plugins<I>(&self, plugins: I) -> XcelerateResult<()>
    where
        I: IntoIterator,
        I::Item: Plugin + 'static,
    {
        for plugin in plugins {
            let plugin: Arc<dyn Plugin> = Arc::new(plugin);
            self.plugins.install(plugin)?;
        }
        Ok(())
    }

    /// Constructs a `Page` handle for an already-attached flattened session.
    ///
    /// Shared by [`Browser::new_page`] and [`Browser::attach_page`]. It only
    /// wires up the handle; neither navigating nor the plugin hooks are done
    /// here, so each caller decides those.
    fn build_page(&self, session_id: String, target_id: String) -> Arc<Page> {
        Arc::new(Page {
            client: Arc::clone(&self.client),
            session_id,
            target_id,
            mouse_x: std::sync::Mutex::new(100.0),
            mouse_y: std::sync::Mutex::new(100.0),
            events: tokio::sync::Mutex::new(Vec::new()),
            routes: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            requests: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            interception_task: Arc::new(tokio::sync::Mutex::new(None)),
            credentials: Arc::new(tokio::sync::Mutex::new(None)),
            drag_interception: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            default_timeout_ms: std::sync::atomic::AtomicU64::new(30_000),
            recording: tokio::sync::Mutex::new(None),
            snapshot_index: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
            downloads_path: Arc::new(tokio::sync::Mutex::new(None)),
            har_entries: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            har_task: Arc::new(tokio::sync::Mutex::new(None)),
            har_body_mode: Arc::new(tokio::sync::Mutex::new("omit".to_string())),
            last_snapshot: Arc::new(tokio::sync::Mutex::new(None)),
        })
    }

    /// Attaches to an existing target and returns a driven page for it.
    ///
    /// [`Browser::new_page`] always opens a fresh tab; this instead takes over a
    /// target that already exists - the way a popup discovered with
    /// [`Page::wait_for_popup`] becomes drivable. Nothing is navigated, so the
    /// tab keeps whatever it is showing; plugin `on_page_created` hooks still
    /// run, so stealth payloads are injected as usual.
    ///
    /// Rust-only: it is intentionally not exposed through the language bindings,
    /// alongside [`Browser::connect`].
    pub async fn attach_page(self: Arc<Self>, target_id: String) -> XcelerateResult<Arc<Page>> {
        let session = self
            .client
            .execute_raw(
                "Target.attachToTarget",
                serde_json::json!({ "targetId": target_id, "flatten": true }),
            )
            .await?;
        let session_id = session
            .get("sessionId")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                XcelerateError::NotFound(format!("target {target_id} could not be attached"))
            })?
            .to_string();

        let page = self.build_page(session_id, target_id);
        self.plugins
            .on_page_created(crate::plugin::page_host(Arc::clone(&page)))
            .await?;
        Ok(page)
    }
}

impl super::engine::Browser for Browser {
    fn engine(&self) -> Engine {
        Engine::Chromium
    }

    fn connected(&self) -> bool {
        self.client.is_connected()
    }

    fn close(&self) -> super::engine::BoxFuture<'_, XcelerateResult<()>> {
        Box::pin(Browser::close(self))
    }
}

fn setup_browser_args(
    cmd: &mut std::process::Command,
    profile_dir: &Path,
    port: u16,
    headless: bool,
) {
    cmd.arg(format!("--remote-debugging-port={}", port))
        .arg("--remote-debugging-address=127.0.0.1")
        .arg(format!("--user-data-dir={}", profile_dir.display()))
        .arg("--no-first-run")
        .arg("--no-default-browser-check")
        .arg("--remote-allow-origins=*")
        .arg("--no-startup-window")
        .args(background_hardening_flags())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    // Chrome's OS sandbox needs user namespaces, which most CI runners and
    // containers lack and root cannot use at all; there the browser refuses to
    // start. Every mainstream driver drops it in those environments, so do the
    // same while leaving it enabled for ordinary desktop launches.
    if needs_no_sandbox() {
        cmd.arg("--no-sandbox");
    }

    if headless {
        cmd.arg("--headless=new");
        cmd.arg("--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36");
    }
}

async fn wait_for_ws_url(
    port: u16,
    mut child: Option<&mut tokio::process::Child>,
) -> XcelerateResult<String> {
    let mut attempts = 0;
    loop {
        // A browser that dies during startup (locked profile, missing GL, a
        // rejected flag) should surface its exit status instead of a bare
        // timeout that hides the cause.
        if let Some(child) = child.as_deref_mut()
            && let Ok(Some(status)) = child.try_wait()
        {
            return Err(XcelerateError::NotFound(format!(
                "Chrome exited before the DevTools endpoint was ready ({status})"
            )));
        }
        match fetch_devtools_version(port).await {
            Ok(json) => {
                if let Some(ws_url) = json["webSocketDebuggerUrl"].as_str() {
                    return Ok(ws_url.to_string());
                }
            }
            Err(_) => {
                attempts += 1;
                if attempts > 100 {
                    return Err(XcelerateError::NotFound(
                        "Timed out waiting for Chrome HTTP server".into(),
                    ));
                }
                tokio::time::sleep(Duration::from_millis(150)).await;
            }
        }
    }
}

/// `GET /json/version` over a raw loopback socket.
///
/// The DevTools HTTP endpoint is plaintext HTTP/1.1 bound to `127.0.0.1`, and
/// this is the only HTTP the crate ever performs, so it is spoken directly on a
/// `tokio::net::TcpStream` instead of through a full client stack. Chrome answers
/// with a `Content-Length` (never chunked) and `Connection: close` lets the body
/// be read to EOF.
async fn fetch_devtools_version(port: u16) -> XcelerateResult<serde_json::Value> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
        .await
        .map_err(|error| XcelerateError::HttpError(format!("connect 127.0.0.1:{port}: {error}")))?;

    let request = format!(
        "GET /json/version HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nAccept: application/json\r\nConnection: close\r\n\r\n"
    );
    stream
        .write_all(request.as_bytes())
        .await
        .map_err(|error| XcelerateError::HttpError(error.to_string()))?;

    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .await
        .map_err(|error| XcelerateError::HttpError(error.to_string()))?;

    let Some(split) = response.windows(4).position(|window| window == b"\r\n\r\n") else {
        return Err(XcelerateError::HttpError(
            "malformed response from the DevTools endpoint".into(),
        ));
    };
    serde_json::from_slice(&response[split + 4..]).map_err(XcelerateError::from)
}

/// Chrome flags applied to every launch that stop the browser from throttling
/// or backgrounding the work a driver depends on.
///
/// Without these, timers, renderers and network activity are throttled whenever
/// a window is occluded or unfocused, which turns fixed-duration waits,
/// screenshots and media playback into slow, flaky operations. The cost is a
/// handful of argv entries at spawn and no runtime overhead.
fn background_hardening_flags() -> &'static [&'static str] {
    &[
        "--disable-background-timer-throttling",
        "--disable-backgrounding-occluded-windows",
        "--disable-renderer-backgrounding",
        "--disable-ipc-flooding-protection",
        "--disable-background-networking",
        "--disable-component-update",
        "--disable-domain-reliability",
        "--disable-hang-monitor",
        "--disable-sync",
        "--mute-audio",
        "--password-store=basic",
        "--use-mock-keychain",
        "--disable-features=Translate,OptimizationHints,MediaRouter,PrivacySandboxSettings4",
    ]
}

/// Whether Chrome's OS sandbox must be dropped because the host cannot set it
/// up: CI runners and containers generally lack the required user namespaces,
/// and root can never use it. Ordinary desktop launches keep the sandbox.
fn needs_no_sandbox() -> bool {
    if std::env::var_os("CI").is_some() {
        return true;
    }
    #[cfg(unix)]
    {
        // SAFETY: `getuid` is a side-effect-free libc getter.
        unsafe { libc::getuid() == 0 }
    }
    #[cfg(not(unix))]
    {
        false
    }
}

/// Chrome flags that make rendering deterministic (stable screenshots).
fn deterministic_flags() -> &'static [&'static str] {
    &[
        "--deterministic-mode",
        "--run-all-compositor-stages-before-draw",
        "--disable-new-content-rendering-timeout",
        "--disable-threaded-animation",
        "--disable-threaded-scrolling",
        "--disable-checker-imaging",
        "--disable-image-animation-resync",
    ]
}

/// Chrome flags that disable web security / site isolation (testing only).
fn disable_security_flags() -> &'static [&'static str] {
    &[
        "--disable-web-security",
        "--disable-site-isolation-trials",
        "--allow-running-insecure-content",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_flags_are_populated() {
        let flags = deterministic_flags();
        assert!(!flags.is_empty());
        for expected in [
            "--deterministic-mode",
            "--run-all-compositor-stages-before-draw",
            "--disable-new-content-rendering-timeout",
            "--disable-threaded-animation",
            "--disable-threaded-scrolling",
            "--disable-checker-imaging",
            "--disable-image-animation-resync",
        ] {
            assert!(flags.contains(&expected), "missing flag: {expected}");
        }
    }

    #[test]
    fn hardening_flags_cover_throttling_and_backgrounding() {
        let flags = background_hardening_flags();
        assert!(!flags.is_empty());
        for expected in [
            "--disable-background-timer-throttling",
            "--disable-backgrounding-occluded-windows",
            "--disable-renderer-backgrounding",
            "--disable-ipc-flooding-protection",
        ] {
            assert!(flags.contains(&expected), "missing flag: {expected}");
        }
    }

    /// The readiness loop must report a browser that died during startup instead
    /// of waiting out the full timeout.
    #[tokio::test]
    async fn reports_a_crash_before_the_debugger_is_ready() {
        #[cfg(windows)]
        let mut cmd = {
            let mut cmd = tokio::process::Command::new("cmd");
            cmd.args(["/C", "exit 3"]);
            cmd
        };
        #[cfg(unix)]
        let mut cmd = {
            let mut cmd = tokio::process::Command::new("sh");
            cmd.args(["-c", "exit 3"]);
            cmd
        };

        let mut child = cmd.spawn().expect("spawn a process that exits immediately");

        // Give the stub process a moment to exit, then point the waiter at a
        // port nothing is listening on.
        tokio::time::sleep(Duration::from_millis(50)).await;
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
            .await
            .unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let error = wait_for_ws_url(port, Some(&mut child)).await.unwrap_err();
        assert!(
            error.to_string().contains("exited before"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn disable_security_flags_are_populated() {
        let flags = disable_security_flags();
        assert!(!flags.is_empty());
        for expected in [
            "--disable-web-security",
            "--disable-site-isolation-trials",
            "--allow-running-insecure-content",
        ] {
            assert!(flags.contains(&expected), "missing flag: {expected}");
        }
    }

    #[test]
    fn profile_none_has_an_empty_path() {
        assert!(Profile::None.path().as_os_str().is_empty());
    }

    /// Proves the dependency-free loopback GET (which replaced `reqwest`) speaks
    /// HTTP/1.1 correctly against a real socket.
    #[tokio::test]
    async fn parses_the_devtools_websocket_url_over_a_raw_socket() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
            .await
            .unwrap();
        let port = listener.local_addr().unwrap().port();

        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut scratch = [0u8; 256];
            let _ = socket.read(&mut scratch).await;
            let body = r#"{"webSocketDebuggerUrl":"ws://127.0.0.1:9222/devtools/browser/abc"}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = socket.write_all(response.as_bytes()).await;
        });

        let json = fetch_devtools_version(port).await.unwrap();
        assert_eq!(
            json["webSocketDebuggerUrl"].as_str(),
            Some("ws://127.0.0.1:9222/devtools/browser/abc")
        );
    }

    #[tokio::test]
    async fn rejects_a_response_without_a_body_separator() {
        use tokio::io::AsyncWriteExt;

        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
            .await
            .unwrap();
        let port = listener.local_addr().unwrap().port();

        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let _ = socket.write_all(b"HTTP/1.1 200 OK\r\nno separator").await;
        });

        assert!(fetch_devtools_version(port).await.is_err());
    }
}
