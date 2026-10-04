use crate::CdpClient;
use crate::error::{XcelerateError, XcelerateResult};
use crate::page::Page;
use crate::plugin::{Plugin, PluginHandle, PluginManager};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use xcelerate_plugin_api::LaunchPlan;
use xcelerate_plugins::{ProcessGuard, spawn_detached};

/// Configuration for the Browser instance.
#[derive(uniffi::Record)]
pub struct BrowserConfig {
    /// Whether to run the browser in headless mode.
    pub headless: bool,
    /// Whether to run the browser as a detached process.
    pub detached: bool,
    /// Optional path to the browser executable.
    pub executable_path: Option<String>,
    /// Built-in plugins to enable for this browser (for example
    /// `["stealth", "human"]`). Default-deny: no plugin does anything unless
    /// listed here (or enabled afterwards with `Browser::use_plugin`).
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
}

impl Profile {
    fn path(&self) -> &Path {
        match self {
            Profile::Ephemeral(dir) => dir.path(),
            Profile::Persistent(path) => path,
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

/// Represents a browser instance (e.g., Chrome or Edge).
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
}

#[uniffi::export(async_runtime = "tokio")]
impl Browser {
    #[uniffi::constructor]
    pub async fn launch(config: BrowserConfig) -> XcelerateResult<Arc<Self>> {
        let exe = match config.executable_path {
            Some(p) => Some(PathBuf::from(p)),
            None => find_chrome_executable(),
        }
        .ok_or_else(|| {
            XcelerateError::NotFound(
                "Chrome executable not found. Please specify executable_path.".into(),
            )
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
        let port = get_free_port().ok_or(XcelerateError::InternalError)?;

        // Resolve the enabled plugins (default-deny).
        let names = config.plugins.clone().unwrap_or_default();
        let manager = PluginManager::new(&names, crate::plugin::catalog())?;

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

        let (child, guard) = if plan.detached {
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
                auto_kill: true,
            };
            (Some(child), Some(guard))
        };

        // 3. Connect to debugger
        let ws_url = wait_for_ws_url(port).await?;
        let client = Arc::new(xcelerate_core::connect(&ws_url).await?);

        Ok(Arc::new(Self {
            client,
            _process: tokio::sync::Mutex::new(child),
            _process_guard: guard,
            _profile: profile,
            _proxy: proxy_gateway,
            plugins: Arc::new(manager),
            ws_url: ws_url.clone(),
            events: tokio::sync::Mutex::new(Vec::new()),
        }))
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

        let page = Arc::new(Page {
            client: Arc::clone(&self.client),
            session_id: session.session_id.into_owned(),
            target_id: target_id.into_owned(),
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
        });

        // 3. Run plugin page-created hooks (e.g. stealth payload injection).
        self.plugins
            .on_page_created(crate::plugin::page_host(Arc::clone(&page)))
            .await?;

        // 4. Finally navigate to the actual URL
        page.navigate(url).await?;

        Ok(page)
    }

    /// Names of all compiled-in built-in plugins (the catalog).
    pub fn available_plugins(&self) -> Vec<String> {
        xcelerate_plugins::builtin_names()
            .iter()
            .map(|name| (*name).to_string())
            .collect()
    }

    /// Names of the plugins currently enabled on this browser.
    pub fn plugin_names(&self) -> Vec<String> {
        self.plugins.names()
    }

    /// Enables a built-in plugin at runtime.
    ///
    /// Launch-time contributions (such as binary patching) only take effect if
    /// the plugin was enabled before the browser launched; enabling a plugin
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
    /// `plugin.json` file. The manifest is validated, the entrypoint is spawned
    /// **out-of-process**, and a `describe` handshake wires up its ops. Dangerous
    /// capabilities stay denied unless opted into via `XCELERATE_PLUGIN_ALLOW`.
    ///
    /// Once loaded, the plugin's ops are reachable through
    /// `plugin(name).invoke(op, args_json)` in every language, exactly like a
    /// built-in plugin.
    pub fn load_plugin(&self, path: String) -> XcelerateResult<String> {
        let manifest_path = crate::plugin::process::resolve_manifest_path(&path)?;
        let manifest = xcelerate_plugin_api::Manifest::load(&manifest_path.to_string_lossy())?;

        manifest
            .validate_reserved(xcelerate_plugins::builtin_names())
            .map_err(|error| XcelerateError::Unsupported(error.to_string()))?;
        if self.plugins.has(&manifest.name) {
            return Err(XcelerateError::Unsupported(format!(
                "plugin '{}' is already loaded",
                manifest.name
            )));
        }

        let plugin = crate::plugin::process::spawn(
            &manifest,
            &manifest_path,
            Some(Arc::clone(&self.client)),
        )?;
        self.plugins.install(plugin)?;
        Ok(format!("loaded plugin '{}'", manifest.name))
    }

    /// Verifies the integrity of the append-only plugin audit log.
    pub fn audit_verify(&self) -> bool {
        xcelerate_plugin_api::audit_verify()
    }

    /// Returns the plugin audit log as a JSON array (no secrets are recorded).
    pub fn audit_log(&self) -> String {
        let entries: Vec<serde_json::Value> = xcelerate_plugin_api::audit_entries()
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
    pub async fn close(&self) -> XcelerateResult<()> {
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

/// Compile-time extension point, kept out of the `#[uniffi::export]` block so it
/// stays Rust-only (it takes executable [`Plugin`] values).
impl Browser {
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
    /// Installed plugins run in-process and therefore have the same trust as the
    /// built-in `stealth`/`human` plugins. Install them before creating pages so
    /// their `on_page_created` hook sees them. Names already owned by the host
    /// catalog (for example `stealth`) are refused.
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
}

fn find_chrome_executable() -> Option<PathBuf> {
    let paths = if cfg!(windows) {
        vec![
            r"C:\Program Files\Google\Chrome\Application\chrome.exe",
            r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
            r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
        ]
    } else if cfg!(target_os = "macos") {
        vec![
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
        ]
    } else {
        // Linux and others
        vec![
            "/usr/bin/google-chrome",
            "/usr/bin/chromium-browser",
            "/usr/bin/chromium",
            "/usr/bin/microsoft-edge-stable",
        ]
    };

    for path in paths {
        let pb = PathBuf::from(path);
        if pb.exists() {
            return Some(pb);
        }
    }
    None
}

fn get_free_port() -> Option<u16> {
    use std::net::TcpListener;
    TcpListener::bind("127.0.0.1:0")
        .and_then(|listener| listener.local_addr())
        .map(|addr| addr.port())
        .ok()
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
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    if headless {
        cmd.arg("--headless=new");
        cmd.arg("--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36");
    }
}

async fn wait_for_ws_url(port: u16) -> XcelerateResult<String> {
    let version_url = format!("http://127.0.0.1:{}/json/version", port);

    let mut attempts = 0;
    loop {
        match reqwest::get(&version_url).await {
            Ok(resp) => {
                let json: serde_json::Value = resp
                    .json()
                    .await
                    .map_err(|_| XcelerateError::InternalError)?;
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
