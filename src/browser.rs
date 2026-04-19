use crate::connection::CdpClient;
use crate::error::{XcelerateResult, XcelerateError};
use crate::page::Page;
use std::sync::Arc;
use tokio_tungstenite::connect_async;
use tokio::sync::mpsc;
use std::process::Stdio;
use std::path::PathBuf;
use std::time::Duration;

const CDC_PAYLOAD: &str = include_str!("cdc_payload.js");

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
    _process_guard: Option<crate::stealth::process::ProcessGuard>,
    _user_data_dir: Option<tempfile::TempDir>, 
    _stealth: bool,
}

#[uniffi::export(async_runtime = "tokio")]
impl Browser {
    #[uniffi::constructor]
    pub async fn launch(config: BrowserConfig) -> XcelerateResult<Arc<Self>> {
        let exe = match config.executable_path {
            Some(p) => Some(PathBuf::from(p)),
            None => find_chrome_executable(),
        }.ok_or_else(|| {
            XcelerateError::NotFound("Chrome executable not found. Please specify executable_path.".into())
        })?;

        // Create a temporary user data directory and KEEP IT
        let user_data_dir = tempfile::tempdir().map_err(|_| XcelerateError::InternalError)?;
        let port = 9222;

        if config.stealth {
            eprintln!("[LAUNCHER] Applying stealth patches to binary...");
            crate::stealth::patcher::BinaryPatcher::patch_binary(&exe)?;
        }

        eprintln!("[LAUNCHER] Found executable: {:?}", exe);

        // Convert path to OsString for Command
        let mut cmd = std::process::Command::new(&exe);
        cmd.arg(format!("--remote-debugging-port={}", port))
           .arg("--remote-debugging-address=127.0.0.1")
           .arg(format!("--user-data-dir={}", user_data_dir.path().display()))
           .arg("--no-first-run")
           .arg("--no-default-browser-check")
           .arg("--remote-allow-origins=*") 
           .stdout(Stdio::null())
           .stderr(Stdio::null());

        if config.headless {
            cmd.arg("--headless=new");
            // Also set a standard user agent to avoid "Headless" in the string if it persists
            cmd.arg("--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36");
        }

        let (child, guard) = if config.detached {
            eprintln!("[LAUNCHER] Spawning detached process...");
            let pid = crate::stealth::process::spawn_detached(cmd)?;
            let guard = crate::stealth::process::ProcessGuard { pid, auto_kill: false };
            (None, Some(guard))
        } else {
            eprintln!("[LAUNCHER] Spawning managed process...");
            // We still use tokio::process::Command for non-detached to get the Child
            let mut t_cmd = tokio::process::Command::from(cmd);
            let child = t_cmd.spawn().map_err(|e| XcelerateError::NotFound(format!("Failed to start Chrome: {}", e)))?;
            let pid = child.id().ok_or(XcelerateError::InternalError)?;
            let guard = crate::stealth::process::ProcessGuard { pid, auto_kill: true };
            (Some(child), Some(guard))
        };

        // 1. Wait for the HTTP server to respond and give us the URL
        let version_url = format!("http://127.0.0.1:{}/json/version", port);
        eprintln!("[LAUNCHER] Waiting for browser to respond at {}...", version_url);
        
        let mut attempts = 0;
        let ws_url = loop {
            match reqwest::get(&version_url).await {
                Ok(resp) => {
                    let json: serde_json::Value = resp.json().await.map_err(|_| XcelerateError::InternalError)?;
                    if let Some(ws_url) = json["webSocketDebuggerUrl"].as_str() {
                        break ws_url.to_string();
                    }
                }
                Err(_) => {
                    attempts += 1;
                    if attempts % 10 == 0 {
                        eprintln!("[LAUNCHER] Attempt {}: Still waiting for browser to start...", attempts);
                    }
                    if attempts > 100 { 
                        return Err(XcelerateError::NotFound("Timed out waiting for Chrome HTTP server".into())); 
                    }
                    tokio::time::sleep(Duration::from_millis(150)).await;
                }
            }
        };

        // 2. Now connect to the WebSocket URL we found
        eprintln!("[LAUNCHER] Connecting to WebSocket: {}...", ws_url);
        let (ws, _) = connect_async(&ws_url).await?;
        eprintln!("[LAUNCHER] Debugger connected successfully!");
        
        let (tx, rx) = mpsc::unbounded_channel();
        let (handler, _event_rx) = crate::connection::CdpHandler::new(ws, rx);
        let client = Arc::new(CdpClient::new(tx, handler.event_tx.clone()));
        
        // Spawn the handler task internally in Rust
        tokio::spawn(handler.run());

        Ok(Arc::new(Self { 
            client, 
            _process: tokio::sync::Mutex::new(child),
            _process_guard: guard,
            _user_data_dir: Some(user_data_dir),
            _stealth: config.stealth,
        }))
    }

    pub async fn new_page(self: Arc<Self>, url: String) -> XcelerateResult<Arc<Page>> {
        // 1. Create target with about:blank so we can inject scripts before loading the real URL
        let target = self.client.execute(browser_protocol::target::CreateTargetParams {
            url: "about:blank".into(),
            ..Default::default()
        }).await?;

        // 2. Attach to target
        let session = self.client.execute(browser_protocol::target::AttachToTargetParams {
            targetId: target.targetId,
            flatten: Some(true),
            ..Default::default()
        }).await?;

        let page = Arc::new(Page {
            client: Arc::clone(&self.client),
            session_id: session.sessionId,
        });

        // 3. Inject stealth payload if enabled
        if self._stealth {
            page.add_script_to_evaluate_on_new_document(CDC_PAYLOAD.to_string()).await?;
            // We also need to enable the Page domain for some events to fire correctly
            self.client.execute_with_session(
                Some(&page.session_id),
                browser_protocol::page::EnableParams { ..Default::default() }
            ).await?;
        }

        // 4. Finally navigate to the actual URL
        page.navigate(url).await?;

        Ok(page)
    }

    /// Returns the browser version information.
    pub async fn version(&self) -> XcelerateResult<String> {
        let res = self.client.execute(browser_protocol::browser::GetVersionParams { ..Default::default() }).await?;
        Ok(format!("{} (Protocol {})", res.product, res.protocolVersion))
    }

    /// Closes the browser and kills the process.
    pub async fn close(&self) -> XcelerateResult<()> {
        // Try to close gracefully via CDP first
        let _ = self.client.execute(browser_protocol::browser::CloseParams { ..Default::default() }).await;
        
        // Kill the process if it's still running
        let mut lock = self._process.lock().await;
        if let Some(mut child) = lock.take() {
            let _ = child.kill().await;
        }
        Ok(())
    }
}

fn find_chrome_executable() -> Option<PathBuf> {
    // Common Windows installation paths
    let paths = [
        r"C:\Program Files\Google\Chrome\Application\chrome.exe",
        r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
        r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe", // Fallback to Edge
    ];

    for path in paths {
        let pb = PathBuf::from(path);
        if pb.exists() {
            return Some(pb);
        }
    }
    None
}
