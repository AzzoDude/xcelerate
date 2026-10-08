//! Model Context Protocol (MCP) server for the xcelerate CDP engine.
//!
//! Transport: newline-delimited JSON-RPC 2.0 over stdio (the MCP "stdio"
//! transport). The server keeps a single browser session alive across calls and
//! launches it lazily on the first tool that needs it. Nothing is written to
//! stdout except protocol messages; diagnostics go to stderr.
//!
//! Environment configuration:
//!
//! * `XCELERATE_CHROME`   - path to the Chrome/Edge executable.
//! * `XCELERATE_HEADLESS` - `1`/`true` to run headless (default `true`).
//! * `XCELERATE_DETACHED` - `1`/`true` to detach the browser process (default `false`).
//! * `XCELERATE_PLUGINS`  - comma-separated external plugin paths to load.

// The `tool_definitions` array is one large `json!` literal; give the macro room
// to expand as the tool surface grows.
#![recursion_limit = "256"]

use std::error::Error;

use base64::Engine as _;
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use xcelerate::{Browser, BrowserConfig, Page};

/// Protocol revision this server implements.
const PROTOCOL_VERSION: &str = "2025-06-18";
const SERVER_NAME: &str = "xcelerate-mcp";
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Runs the MCP server until stdin is closed, then shuts the browser down.
pub async fn run_stdio() -> Result<(), Box<dyn Error>> {
    let mut lines = BufReader::new(tokio::io::stdin()).lines();
    let mut stdout = tokio::io::stdout();
    let mut server = Server::new();

    while let Some(line) = lines.next_line().await? {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let request: Value = match serde_json::from_str(line) {
            Ok(value) => value,
            Err(err) => {
                write_message(
                    &mut stdout,
                    &json!({
                        "jsonrpc": "2.0",
                        "id": Value::Null,
                        "error": { "code": -32700, "message": format!("Parse error: {err}") }
                    }),
                )
                .await?;
                continue;
            }
        };

        // Notifications have no `id` and must not receive a response.
        let Some(id) = request.get("id").cloned() else {
            continue;
        };

        let method = request
            .get("method")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let params = request.get("params").cloned().unwrap_or(Value::Null);

        let payload = match server.handle(method, &params).await {
            Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
            Err(message) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32603, "message": message }
            }),
        };
        write_message(&mut stdout, &payload).await?;
    }

    server.shutdown().await;
    Ok(())
}

async fn write_message<W: AsyncWriteExt + Unpin>(
    out: &mut W,
    value: &Value,
) -> std::io::Result<()> {
    let mut bytes = serde_json::to_vec(value)?;
    bytes.push(b'\n');
    out.write_all(&bytes).await?;
    out.flush().await
}

/// Result of a tool call: either plain text, or an explicit `content` array
/// (used by tools that return images).
enum Outcome {
    Text(String),
    Content(Value),
}

/// Holds the lazily-launched browser session.
struct Server {
    browser: Option<std::sync::Arc<Browser>>,
    page: Option<std::sync::Arc<Page>>,
    #[cfg(windows)]
    uia: Option<xcelerate_uia::Uia>,
}

impl Server {
    fn new() -> Self {
        Self {
            browser: None,
            page: None,
            #[cfg(windows)]
            uia: None,
        }
    }

    /// The lazily-created UIA automation handle for the native-app tools.
    #[cfg(windows)]
    fn uia(&mut self) -> Result<&mut xcelerate_uia::Uia, String> {
        if self.uia.is_none() {
            self.uia = Some(xcelerate_uia::Uia::new().map_err(|e| e.to_string())?);
        }
        self.uia
            .as_mut()
            .ok_or_else(|| "uia init failed".to_string())
    }

    async fn handle(&mut self, method: &str, params: &Value) -> Result<Value, String> {
        match method {
            "initialize" => {
                // Echo the client's revision when it asks for one; some clients
                // disconnect if the server replies with an unknown version.
                let version = params
                    .get("protocolVersion")
                    .and_then(Value::as_str)
                    .unwrap_or(PROTOCOL_VERSION);
                Ok(json!({
                    "protocolVersion": version,
                    "capabilities": { "tools": { "listChanged": false } },
                    "serverInfo": { "name": SERVER_NAME, "version": SERVER_VERSION },
                    "instructions": "Browser automation through the xcelerate CDP engine. \
                        The browser launches on first use; call browser_navigate to open a page."
                }))
            }
            "ping" => Ok(json!({})),
            "tools/list" => Ok(json!({ "tools": tool_definitions() })),
            "tools/call" => self.call_tool(params).await,
            other => Err(format!("Method not found: {other}")),
        }
    }

    async fn call_tool(&mut self, params: &Value) -> Result<Value, String> {
        let name = params
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| "tools/call requires a 'name'".to_string())?;
        let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

        match self.dispatch(name, &arguments).await {
            Ok(Outcome::Text(text)) => Ok(tool_text(&text, false)),
            Ok(Outcome::Content(content)) => Ok(json!({ "content": content, "isError": false })),
            Err(message) => Ok(tool_text(&message, true)),
        }
    }

    async fn dispatch(&mut self, name: &str, args: &Value) -> Result<Outcome, String> {
        match name {
            "browser_navigate" => {
                let url = str_arg(args, "url")?;
                let page = self.ensure_page().await?;
                page.navigate(url.to_string()).await.map_err(to_message)?;
                // `new_page`/`navigate` can return before a single-page app has
                // hydrated, so give the load event a chance before reading the DOM.
                let _ = page.wait_for_navigation().await;
                let title = page.title().await.unwrap_or_default();
                let current = page.url().await.unwrap_or_else(|_| url.to_string());
                Ok(Outcome::Text(format!(
                    "Navigated to {current}\nTitle: {title}"
                )))
            }
            "browser_title" => {
                let page = self.ensure_page().await?;
                Ok(Outcome::Text(page.title().await.map_err(to_message)?))
            }
            "browser_url" => {
                let page = self.ensure_page().await?;
                Ok(Outcome::Text(page.url().await.map_err(to_message)?))
            }
            "browser_get_content" => {
                let page = self.ensure_page().await?;
                Ok(Outcome::Text(page.content().await.map_err(to_message)?))
            }
            "browser_get_text" => {
                let page = self.ensure_page().await?;
                let text = page
                    .evaluate_string("document.body ? document.body.innerText : ''".to_string())
                    .await
                    .map_err(to_message)?;
                Ok(Outcome::Text(text))
            }
            "browser_screenshot" => {
                let full = bool_arg(args, "full").unwrap_or(false);
                let page = self.ensure_page().await?;
                let png = if full {
                    page.screenshot_full().await
                } else {
                    page.screenshot().await
                }
                .map_err(to_message)?;
                let data = base64::engine::general_purpose::STANDARD.encode(&png);
                Ok(Outcome::Content(json!([
                    { "type": "image", "data": data, "mimeType": "image/png" },
                    { "type": "text", "text": format!("Screenshot captured ({} bytes, {}).", png.len(), if full { "full page" } else { "viewport" }) }
                ])))
            }
            "browser_pdf" => {
                let page = self.ensure_page().await?;
                let pdf = page.pdf().await.map_err(to_message)?;
                let data = base64::engine::general_purpose::STANDARD.encode(&pdf);
                Ok(Outcome::Content(json!([
                    { "type": "image", "data": data, "mimeType": "application/pdf" },
                    { "type": "text", "text": format!("PDF generated ({} bytes).", pdf.len()) }
                ])))
            }
            "browser_click" => {
                let selector = str_arg(args, "selector")?;
                let page = self.ensure_page().await?;
                let element = page
                    .clone()
                    .wait_for_selector(selector.to_string())
                    .await
                    .map_err(to_message)?;
                element.click().await.map_err(to_message)?;
                Ok(Outcome::Text(format!("Clicked {selector}")))
            }
            "browser_type" => {
                let selector = str_arg(args, "selector")?;
                let text = str_arg(args, "text")?;
                let submit = bool_arg(args, "submit").unwrap_or(false);
                let page = self.ensure_page().await?;
                let element = page
                    .clone()
                    .wait_for_selector(selector.to_string())
                    .await
                    .map_err(to_message)?;
                let element = element
                    .type_text(text.to_string())
                    .await
                    .map_err(to_message)?;
                if submit {
                    element
                        .press("Enter".to_string())
                        .await
                        .map_err(to_message)?;
                }
                Ok(Outcome::Text(format!(
                    "Typed into {selector}{}",
                    if submit { " and pressed Enter" } else { "" }
                )))
            }
            "browser_fill" => {
                let selector = str_arg(args, "selector")?;
                let value = str_arg(args, "value")?;
                let page = self.ensure_page().await?;
                let element = page
                    .clone()
                    .wait_for_selector(selector.to_string())
                    .await
                    .map_err(to_message)?;
                let arguments = json!([value]).to_string();
                element
                    .call_json(
                        "function(value) { this.value = value; \
                         this.dispatchEvent(new Event('input', { bubbles: true })); \
                         this.dispatchEvent(new Event('change', { bubbles: true })); }"
                            .to_string(),
                        arguments,
                    )
                    .await
                    .map_err(to_message)?;
                Ok(Outcome::Text(format!("Set {selector}")))
            }
            "browser_hover" => {
                let selector = str_arg(args, "selector")?;
                let page = self.ensure_page().await?;
                let element = page
                    .clone()
                    .wait_for_selector(selector.to_string())
                    .await
                    .map_err(to_message)?;
                element.hover().await.map_err(to_message)?;
                Ok(Outcome::Text(format!("Hovered {selector}")))
            }
            "browser_press" => {
                let selector = str_arg(args, "selector")?;
                let key = str_arg(args, "key")?;
                let page = self.ensure_page().await?;
                let element = page
                    .clone()
                    .wait_for_selector(selector.to_string())
                    .await
                    .map_err(to_message)?;
                element.press(key.to_string()).await.map_err(to_message)?;
                Ok(Outcome::Text(format!("Pressed {key} on {selector}")))
            }
            "browser_query" => {
                let selector = str_arg(args, "selector")?;
                let page = self.ensure_page().await?;
                let element = page
                    .clone()
                    .wait_for_selector(selector.to_string())
                    .await
                    .map_err(to_message)?;
                if let Some(attribute) = opt_str_arg(args, "attribute") {
                    let value = element
                        .attribute(attribute.to_string())
                        .await
                        .map_err(to_message)?;
                    Ok(Outcome::Text(value.unwrap_or_default()))
                } else if bool_arg(args, "html").unwrap_or(false) {
                    Ok(Outcome::Text(
                        element.inner_html().await.map_err(to_message)?,
                    ))
                } else {
                    Ok(Outcome::Text(element.text().await.map_err(to_message)?))
                }
            }
            "browser_query_all" => {
                let selector = str_arg(args, "selector")?;
                let page = self.ensure_page().await?;
                let elements = page
                    .clone()
                    .query_selector_all(selector.to_string())
                    .await
                    .map_err(to_message)?;
                let mut lines = Vec::with_capacity(elements.len());
                for element in elements {
                    lines.push(element.text().await.unwrap_or_default());
                }
                Ok(Outcome::Text(format!(
                    "{} match(es):\n{}",
                    lines.len(),
                    lines.join("\n")
                )))
            }
            "browser_evaluate" => {
                let expression = str_arg(args, "expression")?;
                let page = self.ensure_page().await?;
                let result = page
                    .evaluate_json(expression.to_string())
                    .await
                    .map_err(to_message)?;
                Ok(Outcome::Text(result))
            }
            "browser_wait_for_selector" => {
                let selector = str_arg(args, "selector")?;
                let page = self.ensure_page().await?;
                page.clone()
                    .wait_for_selector(selector.to_string())
                    .await
                    .map_err(to_message)?;
                Ok(Outcome::Text(format!("{selector} is present")))
            }
            "browser_go_back" => {
                let page = self.ensure_page().await?;
                page.go_back().await.map_err(to_message)?;
                Ok(Outcome::Text("Went back".to_string()))
            }
            "browser_reload" => {
                let page = self.ensure_page().await?;
                page.reload().await.map_err(to_message)?;
                Ok(Outcome::Text("Reloaded".to_string()))
            }
            "browser_plugins" => {
                let browser = self.ensure_browser().await?;
                Ok(Outcome::Text(format!(
                    "loaded: {}",
                    browser.plugin_names().join(", ")
                )))
            }
            "browser_plugin_invoke" => {
                let plugin = str_arg(args, "name")?;
                let op = str_arg(args, "op")?;
                let arguments = match args.get("args") {
                    Some(Value::String(text)) => text.clone(),
                    Some(value) => value.to_string(),
                    None => "{}".to_string(),
                };
                let browser = self.ensure_browser().await?;
                let handle = browser.plugin(plugin.to_string()).map_err(to_message)?;
                let result = handle
                    .invoke(op.to_string(), arguments)
                    .await
                    .map_err(to_message)?;
                Ok(Outcome::Text(result))
            }
            "browser_plugin_config" => {
                let plugin = str_arg(args, "name")?;
                let browser = self.ensure_browser().await?;
                let handle = browser.plugin(plugin.to_string()).map_err(to_message)?;
                let config = handle.config().map_err(to_message)?;
                Ok(Outcome::Text(config))
            }
            "browser_wait" => {
                let milliseconds = u64_arg(args, "milliseconds").ok_or_else(|| {
                    "missing required integer argument 'milliseconds'".to_string()
                })?;
                tokio::time::sleep(std::time::Duration::from_millis(milliseconds.min(60_000)))
                    .await;
                Ok(Outcome::Text(format!("Waited {milliseconds} ms.")))
            }
            "browser_start_recording" => {
                let path = str_arg(args, "path")?;
                let options = xcelerate::VideoOptions {
                    quality: u32_arg(args, "quality").unwrap_or(80),
                    max_width: u32_arg(args, "max_width").unwrap_or(0),
                    max_height: u32_arg(args, "max_height").unwrap_or(0),
                    fps: u32_arg(args, "fps").unwrap_or(12),
                    ffmpeg: !bool_arg(args, "no_ffmpeg").unwrap_or(false),
                };
                let page = self.ensure_page().await?;
                page.start_video_with_options(path.to_string(), options)
                    .await
                    .map_err(to_message)?;
                Ok(Outcome::Text(format!(
                    "Recording started; frames are being captured for {path}."
                )))
            }
            "browser_stop_recording" => {
                let page = self.ensure_page().await?;
                match page.stop_video().await.map_err(to_message)? {
                    Some(path) => Ok(Outcome::Text(format!("Recording saved to {path}"))),
                    None => Ok(Outcome::Text("No recording was in progress.".to_string())),
                }
            }
            "browser_accessibility" => {
                let page = self.ensure_page().await?;
                let snapshot = page.accessibility_snapshot().await.map_err(to_message)?;
                Ok(Outcome::Text(snapshot))
            }
            "browser_snapshot" => {
                let page = self.ensure_page().await?;
                let snapshot = page.agent_snapshot().await.map_err(to_message)?;
                Ok(Outcome::Text(snapshot))
            }
            "browser_form_fact" => {
                let page = self.ensure_page().await?;
                let fact = page.form_fact().await.map_err(to_message)?;
                Ok(Outcome::Text(fact))
            }
            "browser_click_index" => {
                let index = u32_arg(args, "index")
                    .ok_or_else(|| "missing required integer argument 'index'".to_string())?;
                let page = self.ensure_page().await?;
                std::sync::Arc::clone(&page)
                    .click_index(index)
                    .await
                    .map_err(to_message)?;
                Ok(Outcome::Text(format!("Clicked snapshot index {index}.")))
            }
            "browser_find" => {
                let pattern = str_arg(args, "pattern")?;
                let context = u32_arg(args, "context").unwrap_or(3) as usize;
                let page = self.ensure_page().await?;
                let excerpt = page
                    .find_in_snapshot(pattern, context)
                    .await
                    .map_err(to_message)?;
                Ok(Outcome::Text(excerpt))
            }
            "browser_dismiss_overlays" => {
                let page = self.ensure_page().await?;
                let raw = page.snapshot_json().await.map_err(to_message)?;
                // Malformed output degrades to "nothing matched" rather than panicking.
                let elements = serde_json::from_str::<Value>(&raw)
                    .ok()
                    .and_then(|value| value.as_array().cloned())
                    .unwrap_or_default();
                let mut dismissed: Vec<String> = Vec::new();
                let mut attempted = 0;
                for element in &elements {
                    if attempted >= OVERLAY_CLICK_CAP {
                        break;
                    }
                    let role = element
                        .get("role")
                        .and_then(Value::as_str)
                        .unwrap_or_default();
                    if role != "button" && role != "link" {
                        continue;
                    }
                    let name = element
                        .get("name")
                        .and_then(Value::as_str)
                        .unwrap_or_default();
                    if !is_overlay_control(name) {
                        continue;
                    }
                    let Some(index) = element
                        .get("index")
                        .and_then(Value::as_u64)
                        .map(|v| v as u32)
                    else {
                        continue;
                    };
                    attempted += 1;
                    if std::sync::Arc::clone(&page)
                        .click_index(index)
                        .await
                        .is_ok()
                    {
                        dismissed.push(name.to_string());
                    }
                }
                if dismissed.is_empty() {
                    Ok(Outcome::Text("no overlay controls matched".to_string()))
                } else {
                    Ok(Outcome::Text(format!(
                        "dismissed {} overlays: {}",
                        dismissed.len(),
                        dismissed.join(", ")
                    )))
                }
            }
            "browser_markdown" => {
                let page = self.ensure_page().await?;
                Ok(Outcome::Text(page.markdown().await.map_err(to_message)?))
            }
            "browser_detect_challenge" => {
                let page = self.ensure_page().await?;
                let report = page.detect_challenge().await.map_err(to_message)?;
                Ok(Outcome::Text(report.to_json().to_string()))
            }
            "browser_find_text" => {
                let text = str_arg(args, "text")?;
                let page = self.ensure_page().await?;
                let count = page.find_text(text.to_string()).await.map_err(to_message)?;
                Ok(Outcome::Text(format!("{count} match(es) for {text:?}.")))
            }
            "browser_wait_for_network_idle" => {
                let idle = u64_arg(args, "idle_ms").unwrap_or(500);
                let timeout = u64_arg(args, "timeout_ms").unwrap_or(30_000);
                let page = self.ensure_page().await?;
                page.wait_for_network_idle(idle, timeout)
                    .await
                    .map_err(to_message)?;
                Ok(Outcome::Text("Network is idle.".to_string()))
            }
            "browser_wait_for_dom_stable" => {
                let quiet = u64_arg(args, "quiet_ms").unwrap_or(500);
                let timeout = u64_arg(args, "timeout_ms").unwrap_or(30_000);
                let page = self.ensure_page().await?;
                page.wait_for_dom_stable(quiet, timeout)
                    .await
                    .map_err(to_message)?;
                Ok(Outcome::Text("DOM is stable.".to_string()))
            }
            "browser_highlight" => {
                let page = self.ensure_page().await?;
                let count = page.highlight_all().await.map_err(to_message)?;
                Ok(Outcome::Text(format!("Highlighted {count} element(s).")))
            }
            "browser_clear_highlights" => {
                let page = self.ensure_page().await?;
                page.clear_highlights().await.map_err(to_message)?;
                Ok(Outcome::Text("Cleared highlights.".to_string()))
            }
            "browser_wait_for_download" => {
                let timeout = u64_arg(args, "timeout_ms").unwrap_or(30_000);
                let page = self.ensure_page().await?;
                let path = page.wait_for_download(timeout).await.map_err(to_message)?;
                Ok(Outcome::Text(format!("Downloaded {path}")))
            }
            "browser_wait_for_popup" => {
                let timeout = u64_arg(args, "timeout_ms").unwrap_or(30_000);
                let page = self.ensure_page().await?;
                let target = page.wait_for_popup(timeout).await.map_err(to_message)?;
                Ok(Outcome::Text(format!("New page target: {target}")))
            }
            "browser_save_har" => {
                let path = str_arg(args, "path")?;
                let page = self.ensure_page().await?;
                let saved = page.save_har(path.to_string()).await.map_err(to_message)?;
                Ok(Outcome::Text(format!("HAR written to {saved}")))
            }
            "browser_response_body" => {
                let request_id = str_arg(args, "request_id")?;
                let page = self.ensure_page().await?;
                let body = page
                    .response_body(request_id.to_string())
                    .await
                    .map_err(to_message)?;
                Ok(Outcome::Text(body))
            }
            "browser_health" => {
                let page = self.ensure_page().await?;
                Ok(Outcome::Text(page.health().await.map_err(to_message)?))
            }
            "browser_load_plugin" => {
                let path = str_arg(args, "path")?;
                let browser = self.ensure_browser().await?;
                let message = browser.load_plugin(path.to_string()).map_err(to_message)?;
                Ok(Outcome::Text(message))
            }
            "browser_close" => {
                if let Some(page) = self.page.take() {
                    let _ = page.close().await;
                }
                if let Some(browser) = self.browser.take() {
                    let _ = browser.close().await;
                }
                Ok(Outcome::Text("Browser closed.".to_string()))
            }
            #[cfg(windows)]
            "app_windows" => {
                let uia = self.uia()?;
                let mut out = String::new();
                for window in uia.windows().map_err(|e| e.to_string())? {
                    out.push_str(&xcelerate_uia::format_window(&window));
                    out.push('\n');
                }
                Ok(Outcome::Text(out))
            }
            #[cfg(windows)]
            "app_find" => {
                let window = str_arg(args, "window")?;
                let needle = args
                    .get("text")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_ascii_lowercase();
                let uia = self.uia()?;
                let infos = uia.snapshot(window, 400).map_err(|e| e.to_string())?;
                let mut out = String::new();
                for el in infos
                    .iter()
                    .filter(|e| e.name.to_ascii_lowercase().contains(&needle))
                {
                    out.push_str(&xcelerate_uia::format_element(el));
                    out.push('\n');
                }
                if out.is_empty() {
                    out.push_str("(no match)");
                }
                Ok(Outcome::Text(out))
            }
            #[cfg(windows)]
            "app_click" => {
                let window = str_arg(args, "window")?;
                let uia = self.uia()?;
                let (index, method) = match (
                    args.get("name").and_then(Value::as_str),
                    args.get("index").and_then(Value::as_u64),
                ) {
                    (Some(name), _) => uia
                        .click_name(window, name, 400)
                        .map_err(|e| e.to_string())?,
                    (None, Some(index)) => {
                        uia.snapshot(window, 400).map_err(|e| e.to_string())?;
                        let method = uia.click(index as usize).map_err(|e| e.to_string())?;
                        (index as usize, method)
                    }
                    (None, None) => return Err("app_click needs `name` or `index`".to_string()),
                };
                Ok(Outcome::Text(format!("clicked [{index}] via {method}")))
            }
            #[cfg(windows)]
            "app_set_value" => {
                let window = str_arg(args, "window")?;
                let index = args
                    .get("index")
                    .and_then(Value::as_u64)
                    .ok_or("app_set_value needs `index`")? as usize;
                let text = str_arg(args, "text")?;
                let uia = self.uia()?;
                uia.snapshot(window, 400).map_err(|e| e.to_string())?;
                uia.set_value(index, text).map_err(|e| e.to_string())?;
                Ok(Outcome::Text(format!("set [{index}]")))
            }
            #[cfg(windows)]
            "app_wait" => {
                let window = str_arg(args, "window")?;
                let text = str_arg(args, "text")?;
                let ms = args
                    .get("timeout_ms")
                    .and_then(Value::as_u64)
                    .unwrap_or(10_000);
                let uia = self.uia()?;
                let hit = uia.wait_for(window, text, ms).map_err(|e| e.to_string())?;
                Ok(Outcome::Text(format!("found {hit:?}")))
            }
            #[cfg(windows)]
            "app_key" => {
                let window = str_arg(args, "window")?;
                let key = str_arg(args, "key")?;
                let uia = self.uia()?;
                uia.key_name(window, key).map_err(|e| e.to_string())?;
                Ok(Outcome::Text(format!("key {key:?}")))
            }
            other => Err(format!("Unknown tool: {other}")),
        }
    }

    async fn ensure_browser(&mut self) -> Result<std::sync::Arc<Browser>, String> {
        if let Some(browser) = &self.browser {
            return Ok(std::sync::Arc::clone(browser));
        }
        let browser = Browser::launch(launch_config()).await.map_err(to_message)?;
        self.browser = Some(std::sync::Arc::clone(&browser));
        Ok(browser)
    }

    async fn ensure_page(&mut self) -> Result<std::sync::Arc<Page>, String> {
        if let Some(page) = &self.page {
            return Ok(std::sync::Arc::clone(page));
        }
        let browser = self.ensure_browser().await?;
        let page = browser
            .new_page("about:blank".to_string())
            .await
            .map_err(to_message)?;
        self.page = Some(std::sync::Arc::clone(&page));
        Ok(page)
    }

    async fn shutdown(&mut self) {
        if let Some(page) = self.page.take() {
            let _ = page.close().await;
        }
        if let Some(browser) = self.browser.take() {
            let _ = browser.close().await;
        }
    }
}

fn tool_text(text: &str, is_error: bool) -> Value {
    json!({
        "content": [{ "type": "text", "text": text }],
        "isError": is_error
    })
}

fn to_message<E: std::fmt::Display>(error: E) -> String {
    error.to_string()
}

fn str_arg<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    args.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing required string argument '{key}'"))
}

fn opt_str_arg<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
}

fn bool_arg(args: &Value, key: &str) -> Option<bool> {
    args.get(key).and_then(Value::as_bool)
}

fn u32_arg(args: &Value, key: &str) -> Option<u32> {
    args.get(key)
        .and_then(Value::as_u64)
        .map(|value| value as u32)
}

fn u64_arg(args: &Value, key: &str) -> Option<u64> {
    args.get(key).and_then(Value::as_u64)
}

/// Browser launch configuration, taken from the environment with safe defaults.
fn launch_config() -> BrowserConfig {
    BrowserConfig {
        headless: env_bool("XCELERATE_HEADLESS", true),
        detached: env_bool("XCELERATE_DETACHED", false),
        executable_path: std::env::var("XCELERATE_CHROME")
            .ok()
            .filter(|value| !value.trim().is_empty()),
        plugins: env_plugins(),
    }
}

fn env_bool(name: &str, default: bool) -> bool {
    match std::env::var(name) {
        Ok(value) => matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        Err(_) => default,
    }
}

fn env_plugins() -> Option<Vec<String>> {
    let raw = std::env::var("XCELERATE_PLUGINS").ok()?;
    let plugins: Vec<String> = raw
        .split(',')
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
        .collect();
    if plugins.is_empty() {
        None
    } else {
        Some(plugins)
    }
}

/// Maximum number of overlay controls `browser_dismiss_overlays` will try to
/// click in one call.
const OVERLAY_CLICK_CAP: usize = 5;

/// Substrings that mark an element's accessible name as an overlay/ad control.
/// Matched case-insensitively against `button`/`link` names.
const OVERLAY_KEYWORDS: &[&str] = &[
    "close",
    "dismiss",
    "skip",
    "accept",
    "allow",
    "agree",
    "got it",
    "no thanks",
    "continue",
    "x",
];

fn is_overlay_control(name: &str) -> bool {
    let lower = name.to_lowercase();
    OVERLAY_KEYWORDS
        .iter()
        .any(|keyword| lower.contains(keyword))
}

fn tool_definitions() -> Value {
    json!([
        {
            "name": "browser_navigate",
            "description": "Open a URL in the browser, launching it if needed.",
            "inputSchema": {
                "type": "object",
                "properties": { "url": { "type": "string", "description": "Absolute URL to open." } },
                "required": ["url"]
            }
        },
        {
            "name": "app_windows",
            "description": "List open native application windows (Windows): name, class, pid. Discover a target for the other app_* tools.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "app_find",
            "description": "List elements of a native window whose name contains `text` (one line each). Cheaper and more precise than dumping the whole tree.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "window": { "type": "string", "description": "Window title substring." },
                    "text": { "type": "string", "description": "Element-name substring (empty = all)." }
                },
                "required": ["window"]
            }
        },
        {
            "name": "app_click",
            "description": "Click an element of a native window by `name` (preferred) or `index`. Pattern-first: no cursor movement when a UI Automation pattern exists.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "window": { "type": "string" },
                    "name": { "type": "string", "description": "Element-name substring." },
                    "index": { "type": "integer", "description": "Element index from app_find." }
                },
                "required": ["window"]
            }
        },
        {
            "name": "app_set_value",
            "description": "Set a native window element's value via ValuePattern (types into a field with no cursor).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "window": { "type": "string" },
                    "index": { "type": "integer" },
                    "text": { "type": "string" }
                },
                "required": ["window", "index", "text"]
            }
        },
        {
            "name": "app_wait",
            "description": "Wait until a native window has an element whose name contains `text` (polls internally, up to timeout_ms).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "window": { "type": "string" },
                    "text": { "type": "string" },
                    "timeout_ms": { "type": "integer" }
                },
                "required": ["window", "text"]
            }
        },
        {
            "name": "app_key",
            "description": "Send a key to a native window: next, prior, down, up, space, enter.",
            "inputSchema": {
                "type": "object",
                "properties": { "window": { "type": "string" }, "key": { "type": "string" } },
                "required": ["window", "key"]
            }
        },
        {
            "name": "browser_title",
            "description": "Return the current page title.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "browser_url",
            "description": "Return the current document URL.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "browser_get_content",
            "description": "Return the full HTML of the current page.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "browser_get_text",
            "description": "Return the visible text of the current page.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "browser_screenshot",
            "description": "Capture a PNG screenshot of the page.",
            "inputSchema": {
                "type": "object",
                "properties": { "full": { "type": "boolean", "description": "Capture the full page instead of the viewport." } }
            }
        },
        {
            "name": "browser_pdf",
            "description": "Render the page to a PDF.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "browser_click",
            "description": "Click the first element matching a CSS selector.",
            "inputSchema": {
                "type": "object",
                "properties": { "selector": { "type": "string" } },
                "required": ["selector"]
            }
        },
        {
            "name": "browser_type",
            "description": "Type text into the element matching a selector (optionally press Enter).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "selector": { "type": "string" },
                    "text": { "type": "string" },
                    "submit": { "type": "boolean" }
                },
                "required": ["selector", "text"]
            }
        },
        {
            "name": "browser_fill",
            "description": "Set an input's value directly and fire input/change events.",
            "inputSchema": {
                "type": "object",
                "properties": { "selector": { "type": "string" }, "value": { "type": "string" } },
                "required": ["selector", "value"]
            }
        },
        {
            "name": "browser_hover",
            "description": "Hover over the element matching a selector.",
            "inputSchema": {
                "type": "object",
                "properties": { "selector": { "type": "string" } },
                "required": ["selector"]
            }
        },
        {
            "name": "browser_press",
            "description": "Press a key while the matching element is focused.",
            "inputSchema": {
                "type": "object",
                "properties": { "selector": { "type": "string" }, "key": { "type": "string" } },
                "required": ["selector", "key"]
            }
        },
        {
            "name": "browser_query",
            "description": "Return an element's text, an attribute, or its inner HTML.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "selector": { "type": "string" },
                    "attribute": { "type": "string", "description": "Return this attribute instead of the text." },
                    "html": { "type": "boolean", "description": "Return inner HTML instead of the text." }
                },
                "required": ["selector"]
            }
        },
        {
            "name": "browser_query_all",
            "description": "Return the text of every element matching a selector.",
            "inputSchema": {
                "type": "object",
                "properties": { "selector": { "type": "string" } },
                "required": ["selector"]
            }
        },
        {
            "name": "browser_evaluate",
            "description": "Evaluate a JavaScript expression and return its JSON result.",
            "inputSchema": {
                "type": "object",
                "properties": { "expression": { "type": "string" } },
                "required": ["expression"]
            }
        },
        {
            "name": "browser_wait_for_selector",
            "description": "Wait until an element matching a selector exists.",
            "inputSchema": {
                "type": "object",
                "properties": { "selector": { "type": "string" } },
                "required": ["selector"]
            }
        },
        {
            "name": "browser_go_back",
            "description": "Navigate back in history.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "browser_reload",
            "description": "Reload the current page.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "browser_plugins",
            "description": "List the plugins loaded on the browser.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "browser_plugin_invoke",
            "description": "Invoke a plugin op with JSON arguments.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": { "type": "string" },
                    "op": { "type": "string" },
                    "args": { "description": "JSON object (or string) of arguments." }
                },
                "required": ["name", "op"]
            }
        },
        {
            "name": "browser_plugin_config",
            "description": "Return a plugin's per-op typed input schema and defaults (JSON, keyed by op), as advertised by the component. Use it to learn an op's expected arguments and default values without reading the DOM.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Plugin name." }
                },
                "required": ["name"]
            }
        },
        {
            "name": "browser_wait",
            "description": "Pause for a number of milliseconds (max 60000).",
            "inputSchema": {
                "type": "object",
                "properties": { "milliseconds": { "type": "integer" } },
                "required": ["milliseconds"]
            }
        },
        {
            "name": "browser_start_recording",
            "description": "Start recording the page to a video file. Uses ffmpeg for .mp4/.webm when available, otherwise writes a Motion-JPEG .avi next to the requested path.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Output video path (e.g. out.mp4 or out.avi)." },
                    "quality": { "type": "integer", "description": "JPEG quality 1-100 (default 80)." },
                    "fps": { "type": "integer", "description": "Frame rate for the native AVI back end (default 12)." },
                    "max_width": { "type": "integer", "description": "Downscale width (0 = native)." },
                    "max_height": { "type": "integer", "description": "Downscale height (0 = native)." },
                    "no_ffmpeg": { "type": "boolean", "description": "Always use the native AVI writer." }
                },
                "required": ["path"]
            }
        },
        {
            "name": "browser_stop_recording",
            "description": "Stop the current recording and write the video file. Returns the saved path.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "browser_accessibility",
            "description": "Return the page's accessibility snapshot as JSON (semantic role/name/value nodes). More robust than CSS selectors for understanding and driving a page.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "browser_snapshot",
            "description": "Return an agent-friendly, indented snapshot of the page. Interactive elements are tagged with a stable [index] usable with browser_click_index; call it before every action instead of guessing CSS selectors.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "browser_form_fact",
            "description": "Return a typed shape for a recognized form (login/register/checkout/search) as {kind, fields:[{name,type,required,placeholder}], submit:{text}}. Cheaper in tokens than a full snapshot when the task is just to understand and fill a form.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "browser_click_index",
            "description": "Click the element at an [index] from the most recent browser_snapshot.",
            "inputSchema": {
                "type": "object",
                "properties": { "index": { "type": "integer", "description": "Index from a preceding browser_snapshot." } },
                "required": ["index"]
            }
        },
        {
            "name": "browser_find",
            "description": "Grep the existing indexed page snapshot for a literal, case-insensitive string and return only the matching lines (plus a little context). Cheaper in tokens than a full browser_snapshot; the [index] markers in the result are usable with browser_click_index.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "pattern": { "type": "string", "description": "Literal text to find (case-insensitive)." },
                    "context": { "type": "integer", "description": "Lines of context before and after each match (default 3)." }
                },
                "required": ["pattern"]
            }
        },
        {
            "name": "browser_dismiss_overlays",
            "description": "Best-effort: click up to five visible overlay controls (close/dismiss/skip/accept/agree/etc.) to move ad and cookie popups out of the way. Returns which controls were dismissed.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "browser_markdown",
            "description": "Return the page's main content as clean Markdown (scripts, nav, and boilerplate removed). Ideal for reading or summarizing a page.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "browser_detect_challenge",
            "description": "Detect anti-bot / CAPTCHA challenges on the page and return JSON {detected, vendors, signals}.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "browser_find_text",
            "description": "Count elements containing the given text, scrolling the first match into view and highlighting it.",
            "inputSchema": {
                "type": "object",
                "properties": { "text": { "type": "string" } },
                "required": ["text"]
            }
        },
        {
            "name": "browser_wait_for_network_idle",
            "description": "Wait until the page has finished loading resources and the network is idle.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "idle_ms": { "type": "integer", "description": "How long the network must stay quiet (default 500)." },
                    "timeout_ms": { "type": "integer", "description": "Overall timeout (default 30000)." }
                }
            }
        },
        {
            "name": "browser_wait_for_dom_stable",
            "description": "Wait until the DOM stops mutating for a quiet period.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "quiet_ms": { "type": "integer", "description": "Required quiet period (default 500)." },
                    "timeout_ms": { "type": "integer", "description": "Overall timeout (default 30000)." }
                }
            }
        },
        {
            "name": "browser_highlight",
            "description": "Draw red outlines over interactive elements so they can be seen in a screenshot or by a human.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "browser_clear_highlights",
            "description": "Remove any highlights drawn by browser_highlight.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "browser_wait_for_download",
            "description": "Wait for the page to trigger a file download and return the saved path.",
            "inputSchema": {
                "type": "object",
                "properties": { "timeout_ms": { "type": "integer", "description": "Overall timeout (default 30000)." } }
            }
        },
        {
            "name": "browser_wait_for_popup",
            "description": "Wait for a popup / new tab to open and return its target id.",
            "inputSchema": {
                "type": "object",
                "properties": { "timeout_ms": { "type": "integer", "description": "Overall timeout (default 30000)." } }
            }
        },
        {
            "name": "browser_save_har",
            "description": "Write the network activity recorded so far to a HAR 1.2 file and return the path.",
            "inputSchema": {
                "type": "object",
                "properties": { "path": { "type": "string" } },
                "required": ["path"]
            }
        },
        {
            "name": "browser_response_body",
            "description": "Fetch a network response body by CDP requestId (base64 payloads are decoded).",
            "inputSchema": {
                "type": "object",
                "properties": { "request_id": { "type": "string" } },
                "required": ["request_id"]
            }
        },
        {
            "name": "browser_health",
            "description": "Return a JSON health report for the page (responsiveness, url, target).",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "browser_load_plugin",
            "description": "Load a plugin from a directory or plugin.json. Its ops become callable through browser_plugin_invoke. Dangerous capabilities stay denied unless the host opted in.",
            "inputSchema": {
                "type": "object",
                "properties": { "path": { "type": "string", "description": "Plugin directory or path to plugin.json." } },
                "required": ["path"]
            }
        },
        {
            "name": "browser_close",
            "description": "Close the browser and end the session.",
            "inputSchema": { "type": "object", "properties": {} }
        }
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_list_is_complete_and_unique() {
        let tools = tool_definitions();
        let tools = tools.as_array().expect("tools is an array");
        assert!(tools.len() >= 20, "expected a broad tool surface");
        let mut names: Vec<&str> = tools
            .iter()
            .filter_map(|tool| tool.get("name").and_then(Value::as_str))
            .collect();
        let count = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), count, "tool names must be unique");
        assert!(names.contains(&"browser_navigate"));
        assert!(names.contains(&"browser_screenshot"));
        assert!(names.contains(&"browser_start_recording"));
        assert!(names.contains(&"browser_stop_recording"));
        assert!(names.contains(&"browser_wait"));
        assert!(names.contains(&"browser_load_plugin"));
        assert!(names.contains(&"browser_accessibility"));
        assert!(names.contains(&"browser_snapshot"));
        assert!(names.contains(&"browser_click_index"));
        assert!(names.contains(&"browser_find"));
        assert!(names.contains(&"browser_dismiss_overlays"));
        assert!(names.contains(&"browser_markdown"));
        assert!(names.contains(&"browser_detect_challenge"));
        assert!(names.contains(&"browser_find_text"));
        assert!(names.contains(&"browser_wait_for_network_idle"));
        assert!(names.contains(&"browser_save_har"));
        assert!(names.contains(&"browser_response_body"));
        assert!(names.contains(&"browser_health"));
    }

    #[test]
    fn missing_argument_is_an_error() {
        assert!(str_arg(&json!({}), "url").is_err());
        assert_eq!(str_arg(&json!({ "url": "x" }), "url").unwrap(), "x");
    }
}
