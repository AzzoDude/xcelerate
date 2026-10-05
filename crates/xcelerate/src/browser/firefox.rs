//! Firefox backend over WebDriver BiDi.
//!
//! Firefox does **not** speak CDP — Mozilla removed it (WebDriver BiDi is the
//! only remote protocol since Firefox 141). This module launches Firefox with
//! `--remote-debugging-port`, completes the `session.new` handshake, and drives
//! pages using the published `webdriver-bidi` types, so the same generated
//! protocol surface that backs any BiDi client is what runs here.

use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use serde_json::{Value, json};
use xcelerate_core::bidi::{self, BidiClient};
use xcelerate_core::webdriver_bidi::{browsing_context, session};

use crate::error::{XcelerateError, XcelerateResult};

/// Configuration for a Firefox session.
#[derive(Clone, Debug)]
pub struct FirefoxConfig {
    /// Run without a visible window.
    pub headless: bool,
    /// Path to the Firefox executable. Discovered from the default install
    /// locations when `None`.
    pub executable_path: Option<String>,
}

impl Default for FirefoxConfig {
    fn default() -> Self {
        Self {
            headless: true,
            executable_path: None,
        }
    }
}

/// A running Firefox instance driven over WebDriver BiDi.
pub struct FirefoxBrowser {
    client: Arc<BidiClient>,
    process: tokio::sync::Mutex<Option<tokio::process::Child>>,
    version: String,
    _profile: tempfile::TempDir,
}

impl FirefoxBrowser {
    /// Launches Firefox and establishes a BiDi session.
    ///
    /// The browser is always started with a throwaway profile and `--no-remote`
    /// so it never touches the user's own session.
    pub async fn launch(config: FirefoxConfig) -> XcelerateResult<Arc<Self>> {
        let executable = config
            .executable_path
            .map(PathBuf::from)
            .or_else(find_firefox)
            .ok_or_else(|| {
                XcelerateError::NotFound(
                    "Firefox executable not found. Set executable_path.".into(),
                )
            })?;

        let port = free_port().ok_or(XcelerateError::InternalError)?;
        let profile = tempfile::tempdir().map_err(|_| XcelerateError::InternalError)?;

        let mut command = tokio::process::Command::new(&executable);
        command
            .arg("--no-remote")
            .arg("--new-instance")
            .arg("-profile")
            .arg(profile.path())
            .arg(format!("--remote-debugging-port={port}"))
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        if config.headless {
            command.arg("-headless");
        }

        let child = command.spawn().map_err(|error| {
            XcelerateError::NotFound(format!("Failed to start Firefox: {error}"))
        })?;

        // The Remote Agent only accepts loopback connections, and BiDi's
        // `session.new` must be the first frame on the connection.
        let ws_url = format!("ws://127.0.0.1:{port}/session");
        let client = connect_with_retry(&ws_url).await?;
        let handshake = client.new_session().await?;
        let created: session::NewResult = serde_json::from_value(handshake)?;
        let version = format!(
            "{} {}",
            created.capabilities.browser_name, created.capabilities.browser_version
        );

        Ok(Arc::new(Self {
            client: Arc::new(client),
            process: tokio::sync::Mutex::new(Some(child)),
            version,
            _profile: profile,
        }))
    }

    /// Opens a new tab and navigates it to `url`.
    pub async fn new_page(self: Arc<Self>, url: String) -> XcelerateResult<Arc<FirefoxPage>> {
        let params =
            browsing_context::CreateParameters::builder(browsing_context::CreateType::Tab).build();
        let result = self
            .client
            .command(
                browsing_context::CreateParameters::METHOD,
                serde_json::to_value(&params)?,
            )
            .await?;
        let created: browsing_context::CreateResult = serde_json::from_value(result)?;

        let page = Arc::new(FirefoxPage {
            client: Arc::clone(&self.client),
            context: created.context,
        });
        page.goto(url).await?;
        Ok(page)
    }

    /// The browser identification, e.g. `firefox 157.0`.
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Whether the BiDi connection is still live.
    pub fn is_connected(&self) -> bool {
        self.client.is_connected()
    }

    /// Kills Firefox and releases the throwaway profile.
    pub async fn close(&self) -> XcelerateResult<()> {
        let mut guard = self.process.lock().await;
        if let Some(mut child) = guard.take() {
            let _ = child.kill().await;
        }
        Ok(())
    }
}

/// A page (browsing context) inside a Firefox session.
pub struct FirefoxPage {
    client: Arc<BidiClient>,
    context: String,
}

impl FirefoxPage {
    /// Navigates the page and waits until the load completes.
    pub async fn goto(&self, url: String) -> XcelerateResult<()> {
        let params =
            browsing_context::NavigateParameters::builder(self.context.as_str(), url.as_str())
                .wait(browsing_context::ReadinessState::Complete)
                .build();
        self.client
            .command(
                browsing_context::NavigateParameters::METHOD,
                serde_json::to_value(&params)?,
            )
            .await?;
        Ok(())
    }

    /// The page's current URL.
    pub async fn url(&self) -> XcelerateResult<String> {
        Ok(self.string("location.href".to_string()).await?)
    }

    /// The page's document title.
    pub async fn title(&self) -> XcelerateResult<String> {
        Ok(self.string("document.title".to_string()).await?)
    }

    /// The browsing context id.
    pub fn context_id(&self) -> &str {
        &self.context
    }

    /// Evaluates a JavaScript expression and returns its JSON value.
    pub async fn evaluate(&self, expression: String) -> XcelerateResult<Value> {
        let params = json!({
            "expression": expression,
            "target": { "context": self.context },
            "awaitPromise": false,
            "resultOwnership": "none",
        });
        Ok(self.client.command("script.evaluate", params).await?)
    }

    /// Convenience: evaluate and read the resulting string value.
    async fn string(&self, expression: String) -> XcelerateResult<String> {
        let result = self.evaluate(expression).await?;
        Ok(result["result"]["value"]
            .as_str()
            .unwrap_or_default()
            .to_string())
    }
}

fn find_firefox() -> Option<PathBuf> {
    let candidates = if cfg!(windows) {
        vec![
            r"C:\Program Files\Mozilla Firefox\firefox.exe",
            r"C:\Program Files (x86)\Mozilla Firefox\firefox.exe",
        ]
    } else if cfg!(target_os = "macos") {
        vec!["/Applications/Firefox.app/Contents/MacOS/firefox"]
    } else {
        vec![
            "/usr/bin/firefox",
            "/usr/bin/firefox-esr",
            "/snap/bin/firefox",
        ]
    };
    candidates
        .into_iter()
        .map(PathBuf::from)
        .find(|path| path.exists())
}

fn free_port() -> Option<u16> {
    std::net::TcpListener::bind("127.0.0.1:0")
        .and_then(|listener| listener.local_addr())
        .map(|addr| addr.port())
        .ok()
}

async fn connect_with_retry(ws_url: &str) -> XcelerateResult<BidiClient> {
    let mut last = String::from("no attempt");
    for _ in 0..120 {
        match bidi::connect(ws_url).await {
            Ok(client) => return Ok(client),
            Err(error) => {
                last = error.to_string();
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
        }
    }
    Err(XcelerateError::WsError(format!(
        "could not connect to {ws_url}: {last}"
    )))
}
