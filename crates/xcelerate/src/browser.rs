use crate::CdpClient;
use crate::error::{XcelerateError, XcelerateResult};
use crate::page::Page;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use xcelerate_stealth::{BinaryPatcher, CDC_PAYLOAD, ProcessGuard, spawn_detached};

/// Configuration for the Browser instance.
#[derive(uniffi::Record)]
pub struct BrowserConfig {
    /// Whether to run the browser in headless mode.
    pub headless: bool,
    /// Whether to apply stealth patches to the binary.
    pub stealth: bool,
    /// Whether to run the browser as a detached process.
    pub detached: bool,
    /// Optional path to the browser executable.
    pub executable_path: Option<String>,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            headless: true,
            stealth: true,
            detached: true,
            executable_path: None,
        }
    }
}

/// Represents a browser instance (e.g., Chrome or Edge).
#[derive(uniffi::Object)]
pub struct Browser {
    pub(crate) client: Arc<CdpClient>,
    _process: tokio::sync::Mutex<Option<tokio::process::Child>>,
    _process_guard: Option<ProcessGuard>,
    _user_data_dir: Option<tempfile::TempDir>,
    _stealth: bool,
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
        let user_data_dir = tempfile::tempdir().map_err(|_| XcelerateError::InternalError)?;
        let port = get_free_port().ok_or(XcelerateError::InternalError)?;

        let exe = if config.stealth {
            BinaryPatcher::patch_to_temp(&exe)?
        } else {
            exe
        };

        // 2. Spawn process
        let mut cmd = std::process::Command::new(&exe);
        setup_browser_args(&mut cmd, &user_data_dir, port, config.headless);

        let (child, guard) = if config.detached {
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
            _user_data_dir: Some(user_data_dir),
            _stealth: config.stealth,
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
        });

        // 3. Inject stealth payload if enabled
        if self._stealth {
            page.add_script_to_evaluate_on_new_document(CDC_PAYLOAD.to_string())
                .await?;
            // We also need to enable the Page domain for some events to fire correctly
            self.client
                .execute_with_session(
                    Some(&page.session_id),
                    browser_protocol::page::EnableParams {
                        ..Default::default()
                    },
                )
                .await?;
        }

        // 4. Finally navigate to the actual URL
        page.navigate(url).await?;

        Ok(page)
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

    /// Closes the browser and kills the process.
    pub async fn close(&self) -> XcelerateResult<()> {
        // Try to close gracefully via CDP first
        let _ = self
            .client
            .execute(browser_protocol::browser::CloseParams {})
            .await;

        // Kill the process if it's still running
        let mut lock = self._process.lock().await;
        if let Some(mut child) = lock.take() {
            let _ = child.kill().await;
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
                Ok(Err(_)) => return Err(XcelerateError::InternalError),
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
    user_data_dir: &tempfile::TempDir,
    port: u16,
    headless: bool,
) {
    cmd.arg(format!("--remote-debugging-port={}", port))
        .arg("--remote-debugging-address=127.0.0.1")
        .arg(format!(
            "--user-data-dir={}",
            user_data_dir.path().display()
        ))
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
