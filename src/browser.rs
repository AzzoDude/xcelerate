use crate::connection::CdpClient;
use crate::error::{XcelerateResult, XcelerateError};
use crate::page::Page;
use std::sync::Arc;
use tokio_tungstenite::connect_async;
use tokio::sync::mpsc;
use std::process::Stdio;
use tokio::process::Command;
use std::path::PathBuf;
use std::time::Duration;

/// Configuration for the Browser instance.
pub struct BrowserConfig {
    /// Whether to run the browser in headless mode.
    pub headless: bool,
    /// Optional path to the browser executable.
    pub executable_path: Option<PathBuf>,
}

impl BrowserConfig {
    /// Creates a new builder for BrowserConfig.
    pub fn builder() -> BrowserConfigBuilder {
        BrowserConfigBuilder::default()
    }
}

/// A builder for BrowserConfig.
pub struct BrowserConfigBuilder {
    headless: bool,
    executable_path: Option<PathBuf>,
}

impl Default for BrowserConfigBuilder {
    fn default() -> Self {
        Self {
            headless: true,
            executable_path: None,
        }
    }
}

impl BrowserConfigBuilder {
    /// Sets the headless mode.
    pub fn headless(mut self, headless: bool) -> Self {
        self.headless = headless;
        self
    }
    /// Sets the path to the browser executable.
    pub fn executable_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.executable_path = Some(path.into());
        self
    }
    /// Builds the BrowserConfig.
    pub fn build(self) -> XcelerateResult<BrowserConfig> {
        Ok(BrowserConfig {
            headless: self.headless,
            executable_path: self.executable_path,
        })
    }
}

/// Represents a browser instance (e.g., Chrome or Edge).
pub struct Browser {
    pub(crate) client: Arc<CdpClient>,
    _process: Option<tokio::process::Child>, 
    _user_data_dir: Option<tempfile::TempDir>, 
}

impl Browser {
    pub async fn launch(config: BrowserConfig) -> XcelerateResult<(Self, crate::connection::CdpHandler)> {
        let exe = config.executable_path.or_else(find_chrome_executable).ok_or_else(|| {
            XcelerateError::NotFound("Chrome executable not found. Please specify executable_path.".into())
        })?;

        // Create a temporary user data directory and KEEP IT
        let user_data_dir = tempfile::tempdir().map_err(|_| XcelerateError::InternalError)?;
        let port = 9222;

        eprintln!("[LAUNCHER] Found executable: {:?}", exe);

        let mut cmd = Command::new(exe);
        cmd.arg(format!("--remote-debugging-port={}", port))
           .arg("--remote-debugging-address=127.0.0.1") // Force 127.0.0.1
           .arg(format!("--user-data-dir={}", user_data_dir.path().display()))
           .arg("--no-first-run")
           .arg("--no-default-browser-check")
           .arg("--remote-allow-origins=*") 
           .stdout(Stdio::null())
           .stderr(Stdio::null());

        if config.headless {
            cmd.arg("--headless");
        }

        eprintln!("[LAUNCHER] Spawning process...");
        let child = cmd.spawn().map_err(|e| XcelerateError::NotFound(format!("Failed to start Chrome: {}", e)))?;

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
        
        Ok((Self { 
            client, 
            _process: Some(child),
            _user_data_dir: Some(user_data_dir),
        }, handler))
    }

    pub async fn new_page(&self, url: impl Into<String>) -> XcelerateResult<Page> {
        let target = self.client.execute(browser_protocol::target::CreateTargetParams {
            url: url.into(),
            ..Default::default()
        }).await?;

        let session = self.client.execute(browser_protocol::target::AttachToTargetParams {
            targetId: target.targetId,
            flatten: Some(true),
            ..Default::default()
        }).await?;

        Ok(Page {
            client: Arc::clone(&self.client),
            session_id: session.sessionId,
        })
    }

    /// Returns the browser version information.
    pub async fn version(&self) -> XcelerateResult<browser_protocol::browser::GetVersionReturns> {
        self.client.execute(browser_protocol::browser::GetVersionParams { ..Default::default() }).await
    }

    /// Closes the browser and kills the process.
    pub async fn close(&mut self) -> XcelerateResult<()> {
        // Try to close gracefully via CDP first
        let _ = self.client.execute(browser_protocol::browser::CloseParams { ..Default::default() }).await;
        
        // Kill the process if it's still running
        if let Some(mut child) = self._process.take() {
            let _ = child.kill().await;
        }
        Ok(())
    }

    /// Returns all available targets (tabs, workers, etc).
    pub async fn targets(&self) -> XcelerateResult<Vec<browser_protocol::target::TargetInfo>> {
        let res = self.client.execute(browser_protocol::target::GetTargetsParams { ..Default::default() }).await?;
        Ok(res.targetInfos)
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
