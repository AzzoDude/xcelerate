//! Plugin manager and UniFFI bridge for the engine.
//!
//! The plugin *API* - the [`Plugin`] trait, [`Manifest`], audit log, and the
//! [`PageHost`] interface - lives in `xcelerate-plugin`. **No plugins are built
//! into the core**: every plugin is external, either a crate the embedder
//! installs in-process ([`PluginManager::install`]) or a sandboxed component
//! loaded with [`crate::Browser::load_plugin`]. This module wires them into
//! [`crate::Browser`], implements [`PageHost`] on top of [`Page`], and exposes
//! [`PluginHandle`] to every language binding.

use std::sync::Arc;

use crate::error::{XcelerateError, XcelerateResult};
use crate::page::Page;
use xcelerate_plugin::{ArcPageHost, BoxFut, PageHost, PluginError, PluginResult};

pub use xcelerate_plugin::{
    AuditEvent, Capability, Catalog, Manifest, OpSchema, Plugin, PluginManager, audit_entries,
    audit_verify,
};

#[cfg(feature = "wasm")]
pub(crate) mod wasm;

/// Resolve a `load_plugin` path (a plugin directory or a `plugin.json`) to the
/// manifest path.
pub(crate) fn resolve_manifest_path(path: &str) -> XcelerateResult<std::path::PathBuf> {
    let candidate = std::path::PathBuf::from(path);
    if candidate.is_dir() {
        Ok(candidate.join("plugin.json"))
    } else {
        Ok(candidate)
    }
}

/// The host's plugin catalog. Xcelerate ships **no** plugins built into the
/// core, so this always resolves to nothing; plugins arrive externally through
/// [`PluginManager::install`] (in-process crates) or
/// [`crate::Browser::load_plugin`] (sandboxed components).
pub(crate) fn catalog() -> Catalog {
    Arc::new(|_: &str| None)
}

/// The language's **standard-library plugins**, shipped with the project and
/// **trusted by default**: their declared capabilities are granted without the
/// `XCELERATE_PLUGIN_ALLOW` opt-in. This is the single source of truth - the CLI
/// auto-loads exactly these names. Any other plugin stays default-deny.
pub const STANDARD_PLUGINS: &[&str] = &["core", "browser", "desktop"];

/// Whether `name` is a standard-library plugin (trusted by default).
pub fn is_standard_plugin(name: &str) -> bool {
    STANDARD_PLUGINS.contains(&name)
}

/// A [`PageHost`] backed by the engine's [`Page`], exposed to plugins so they
/// never touch the raw page or transport.
struct PageHostImpl {
    page: Arc<Page>,
}

impl PageHost for PageHostImpl {
    fn add_init_script(&self, script: String) -> BoxFut<PluginResult<()>> {
        let page = Arc::clone(&self.page);
        Box::pin(async move {
            page.add_script_to_evaluate_on_new_document(script)
                .await
                .map(|_| ())
                .map_err(PluginError::from)
        })
    }

    fn dispatch_cdp(&self, method: String, params_json: String) -> BoxFut<PluginResult<String>> {
        let page = Arc::clone(&self.page);
        Box::pin(async move {
            page.execute_cdp_cmd(method, params_json)
                .await
                .map_err(PluginError::from)
        })
    }

    fn move_mouse(&self, x: f64, y: f64) -> BoxFut<PluginResult<()>> {
        let page = Arc::clone(&self.page);
        Box::pin(async move {
            page.move_mouse(x, y)
                .await
                .map(|_| ())
                .map_err(PluginError::from)
        })
    }

    fn click_mouse(&self, x: f64, y: f64) -> BoxFut<PluginResult<()>> {
        let page = Arc::clone(&self.page);
        Box::pin(async move {
            page.click_mouse(x, y)
                .await
                .map(|_| ())
                .map_err(PluginError::from)
        })
    }

    fn keyboard_type(&self, text: String) -> BoxFut<PluginResult<()>> {
        let page = Arc::clone(&self.page);
        Box::pin(async move { page.keyboard_type(text).await.map_err(PluginError::from) })
    }

    fn mouse_position(&self) -> (f64, f64) {
        (
            *self.page.mouse_x.lock().unwrap(),
            *self.page.mouse_y.lock().unwrap(),
        )
    }

    fn browser_op(&self, op: &str, args_json: &str) -> BoxFut<PluginResult<String>> {
        let page = Arc::clone(&self.page);
        let op = op.to_string();
        let args_json = args_json.to_string();
        Box::pin(async move { browser_action(page, &op, &args_json).await })
    }
}

/// The **browser primitives** the host bridge exposes to sandboxed plugins.
///
/// These are deliberately low-level *acts* (`goto`, `click-selector`,
/// `evaluate`, `route`, …). The host owns the CDP/BiDi protocol and the page;
/// all verb *logic* - target classification, JavaScript construction, output
/// formatting, multi-step sequences - lives in the `browser` plugin. A plugin
/// asks for a primitive; it never speaks the protocol itself.
async fn browser_action(page: Arc<Page>, op: &str, args_json: &str) -> PluginResult<String> {
    use serde_json::{Value, json};

    let args: Value = if args_json.trim().is_empty() {
        Value::Null
    } else {
        serde_json::from_str(args_json)
            .map_err(|error| PluginError::Message(format!("args are not JSON: {error}")))?
    };
    let text = |key: &str| args.get(key).and_then(Value::as_str).map(str::to_string);
    let flag = |key: &str| args.get(key).and_then(Value::as_bool).unwrap_or(false);
    let number = |key: &str| args.get(key).and_then(Value::as_u64);
    let need = |key: &str| {
        text(key).ok_or_else(|| PluginError::Unsupported(format!("{op} needs '{key}'")))
    };
    let missing = |key: &str| PluginError::Unsupported(format!("{op} needs '{key}'"));

    // Fetch an element by selector (waiting for it), or by its visible text.
    let by_selector = |selector: String| {
        let page = Arc::clone(&page);
        async move {
            page.wait_for_selector(selector)
                .await
                .map_err(PluginError::from)
        }
    };
    let by_text = |needle: String| {
        let page = Arc::clone(&page);
        async move {
            page.find_clickable_by_text(needle.clone())
                .await
                .map_err(PluginError::from)?
                .ok_or_else(|| {
                    PluginError::NotFound(format!("no visible element contains {needle:?}"))
                })
        }
    };

    match op {
        // Navigation and reading.
        "goto" | "navigate" => {
            let url = need("url")?;
            page.navigate(url.clone())
                .await
                .map_err(PluginError::from)?;
            let _ = page.wait_for_navigation().await;
            Ok(json!({ "url": page.url().await.unwrap_or(url) }).to_string())
        }
        "url" => Ok(json!({ "value": page.url().await.map_err(PluginError::from)? }).to_string()),
        "title" => {
            Ok(json!({ "value": page.title().await.map_err(PluginError::from)? }).to_string())
        }
        "html" => {
            Ok(json!({ "value": page.content().await.map_err(PluginError::from)? }).to_string())
        }
        "markdown" => {
            Ok(json!({ "value": page.markdown().await.map_err(PluginError::from)? }).to_string())
        }
        "text" => {
            let value = page
                .evaluate_string("document.body ? document.body.innerText : ''".to_string())
                .await
                .map_err(PluginError::from)?;
            Ok(json!({ "value": value }).to_string())
        }
        "snapshot" => Ok(
            json!({ "value": page.agent_snapshot().await.map_err(PluginError::from)? }).to_string(),
        ),
        "snapshot-json" => Ok(
            json!({ "value": page.snapshot_json().await.map_err(PluginError::from)? }).to_string(),
        ),

        // Evaluate arbitrary JavaScript. `as = "json"` decodes the result as JSON
        // (`evaluate_json`); otherwise the result is a string.
        "evaluate" => {
            let js = need("js")?;
            let value = if text("as").as_deref() == Some("json") {
                page.evaluate_json(js).await.map_err(PluginError::from)?
            } else {
                page.evaluate_string(js).await.map_err(PluginError::from)?
            };
            Ok(json!({ "value": value }).to_string())
        }

        // Waiting.
        "wait-selector" => {
            let selector = need("selector")?;
            Arc::clone(&page)
                .wait_for_selector(selector)
                .await
                .map_err(PluginError::from)?;
            Ok("{}".to_string())
        }
        "wait-navigation" => {
            let _ = page.wait_for_navigation().await;
            Ok("{}".to_string())
        }
        "wait-network-idle" => {
            let idle = number("idle_ms").unwrap_or(500);
            let timeout = number("timeout_ms").unwrap_or(30_000);
            page.wait_for_network_idle(idle, timeout)
                .await
                .map_err(PluginError::from)?;
            Ok("{}".to_string())
        }
        "wait-dom-stable" => {
            let quiet = number("quiet_ms").unwrap_or(500);
            let timeout = number("timeout_ms").unwrap_or(30_000);
            page.wait_for_dom_stable(quiet, timeout)
                .await
                .map_err(PluginError::from)?;
            Ok("{}".to_string())
        }

        // Queries.
        "find-text" => {
            let needle = text("text").unwrap_or_default();
            let count = page
                .find_text(needle.clone())
                .await
                .map_err(PluginError::from)?;
            Ok(json!({ "count": count }).to_string())
        }

        // History.
        "back" => {
            page.go_back().await.map_err(PluginError::from)?;
            Ok("{}".to_string())
        }
        "reload" => {
            page.reload().await.map_err(PluginError::from)?;
            Ok("{}".to_string())
        }

        // Pointer acts.
        "click-index" => {
            let index = number("index").ok_or_else(|| missing("index"))? as u32;
            Arc::clone(&page)
                .click_index(index)
                .await
                .map_err(PluginError::from)?;
            if let Some(ms) = number("settle_ms") {
                tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
            }
            Ok(json!({ "clicked": index }).to_string())
        }
        "move-index" => {
            let index = number("index").ok_or_else(|| missing("index"))? as u32;
            Arc::clone(&page)
                .move_to_index(index)
                .await
                .map_err(PluginError::from)?;
            Ok(json!({ "moved": index }).to_string())
        }
        "click-mouse" => {
            let x = args
                .get("x")
                .and_then(Value::as_f64)
                .ok_or_else(|| missing("x"))?;
            let y = args
                .get("y")
                .and_then(Value::as_f64)
                .ok_or_else(|| missing("y"))?;
            Arc::clone(&page)
                .click_mouse(x, y)
                .await
                .map_err(PluginError::from)?;
            Ok(json!({ "clicked": [x, y] }).to_string())
        }
        "move-mouse" => {
            let x = args
                .get("x")
                .and_then(Value::as_f64)
                .ok_or_else(|| missing("x"))?;
            let y = args
                .get("y")
                .and_then(Value::as_f64)
                .ok_or_else(|| missing("y"))?;
            Arc::clone(&page)
                .move_mouse(x, y)
                .await
                .map_err(PluginError::from)?;
            Ok(json!({ "moved": [x, y] }).to_string())
        }
        "click-selector" => {
            let element = by_selector(need("selector")?).await?;
            element.click_mouse().await.map_err(PluginError::from)?;
            Ok("{}".to_string())
        }
        "hover-selector" => {
            let element = by_selector(need("selector")?).await?;
            element.hover_mouse().await.map_err(PluginError::from)?;
            Ok("{}".to_string())
        }
        "click-text" => {
            let element = by_text(need("text")?).await?;
            element.click_mouse().await.map_err(PluginError::from)?;
            Ok("{}".to_string())
        }
        "hover-text" => {
            let element = by_text(need("text")?).await?;
            element.hover_mouse().await.map_err(PluginError::from)?;
            Ok("{}".to_string())
        }

        // Typing / keys.
        "type-selector" => {
            let element = by_selector(need("selector")?).await?;
            element
                .type_text(text("text").unwrap_or_default())
                .await
                .map_err(PluginError::from)?;
            Ok("{}".to_string())
        }
        "type-index" => {
            let index = number("index").ok_or_else(|| missing("index"))? as u32;
            if Arc::clone(&page).click_index(index).await.is_err() {
                let _ = page.agent_snapshot().await;
                Arc::clone(&page)
                    .click_index(index)
                    .await
                    .map_err(PluginError::from)?;
            }
            let element = page
                .evaluate_handle("document.activeElement".to_string())
                .await
                .map_err(PluginError::from)?;
            element
                .type_text(text("text").unwrap_or_default())
                .await
                .map_err(PluginError::from)?;
            Ok("{}".to_string())
        }
        "type-active" => {
            let element = page
                .evaluate_handle("document.activeElement".to_string())
                .await
                .map_err(PluginError::from)?;
            element
                .type_text(text("text").unwrap_or_default())
                .await
                .map_err(PluginError::from)?;
            Ok("{}".to_string())
        }
        "press-active" => {
            let key = need("key")?;
            let element = page
                .evaluate_handle("document.activeElement".to_string())
                .await
                .map_err(PluginError::from)?;
            element.press(key).await.map_err(PluginError::from)?;
            Ok("{}".to_string())
        }
        "press-selector" => {
            let selector = need("selector")?;
            let key = need("key")?;
            let element = by_selector(selector).await?;
            element.press(key).await.map_err(PluginError::from)?;
            Ok("{}".to_string())
        }
        "press-key" => {
            let key = need("key")?;
            page.keyboard_press(key).await.map_err(PluginError::from)?;
            Ok("{}".to_string())
        }

        // Form controls.
        "select-option" => {
            let selector = need("selector")?;
            let value = need("value")?;
            let values = json!([value]).to_string();
            Arc::clone(&page)
                .select_option(selector, values)
                .await
                .map_err(PluginError::from)?;
            Ok("{}".to_string())
        }
        "set-input-files" => {
            let selector = need("selector")?;
            let files = args.get("files").cloned().ok_or_else(|| missing("files"))?;
            Arc::clone(&page)
                .set_input_files(selector, files.to_string())
                .await
                .map_err(PluginError::from)?;
            Ok("{}".to_string())
        }

        // Scrolling (the plugin builds the script) and screenshots.
        "scroll" => {
            let js = need("js")?;
            let _ = page.evaluate_string(js).await;
            Ok("{}".to_string())
        }
        "screenshot" => {
            let png = if flag("full") {
                page.screenshot_full().await.map_err(PluginError::from)?
            } else {
                page.screenshot().await.map_err(PluginError::from)?
            };
            use base64::Engine as _;
            let encoded = base64::engine::general_purpose::STANDARD.encode(&png);
            Ok(json!({ "png_base64": encoded, "bytes": png.len() }).to_string())
        }

        // Misc page acts.
        "drag" => {
            let from = need("from")?;
            let to = need("to")?;
            Arc::clone(&page)
                .drag(&from, &to)
                .await
                .map_err(PluginError::from)?;
            Ok("{}".to_string())
        }
        "dialog-policy" => {
            let policy = match text("policy").unwrap_or_default().as_str() {
                "accept" => crate::page::DialogPolicy::Accept,
                _ => crate::page::DialogPolicy::Dismiss,
            };
            page.set_dialog_policy(policy)
                .await
                .map_err(PluginError::from)?;
            Ok("{}".to_string())
        }
        "detect-challenge" => {
            let report = page.detect_challenge().await.map_err(PluginError::from)?;
            Ok(json!({ "report": report.to_json() }).to_string())
        }
        "media-json" => {
            Ok(json!({ "value": page.media_json().await.map_err(PluginError::from)? }).to_string())
        }

        // Cookies and arbitrary CDP. Cookie writes/deletes/clears and the
        // geolocation override are CDP calls, so they ride `cdp`.
        "cookies" => {
            Ok(json!({ "value": page.cookies().await.map_err(PluginError::from)? }).to_string())
        }
        "cookie" => {
            let name = need("name")?;
            Ok(json!({ "value": page.cookie(name).await.map_err(PluginError::from)? }).to_string())
        }
        "cdp" => {
            let method = need("method")?;
            let params = args
                .get("params")
                .map(|value| value.to_string())
                .unwrap_or_else(|| "{}".to_string());
            Ok(json!({ "value": page.execute_cdp_cmd(method, params).await.map_err(PluginError::from)? })
                .to_string())
        }

        // Network interception and auth.
        "route" => {
            let pattern = need("pattern")?;
            let action = text("action").unwrap_or_else(|| "continue".to_string());
            let body = text("body");
            let content_type = text("content_type");
            page.route(pattern, action, body, content_type)
                .await
                .map_err(PluginError::from)?;
            Ok("{}".to_string())
        }
        "unroute" => {
            match text("pattern").filter(|p| !p.is_empty()) {
                Some(pattern) => page.unroute(pattern).await.map_err(PluginError::from)?,
                None => page.unroute_all().await.map_err(PluginError::from)?,
            }
            Ok("{}".to_string())
        }
        "route-har" => {
            let path = need("path")?;
            page.route_from_har(path).await.map_err(PluginError::from)?;
            Ok("{}".to_string())
        }
        "auth" => {
            let username = need("username")?;
            let password = text("password").unwrap_or_default();
            page.authenticate(username, password)
                .await
                .map_err(PluginError::from)?;
            Ok("{}".to_string())
        }

        // Storage state and media capture (file paths are resolved by the host).
        "storage-state" => Ok(
            json!({ "value": page.storage_state().await.map_err(PluginError::from)? }).to_string(),
        ),
        "set-storage-state" => {
            let json = need("json")?;
            page.set_storage_state(json)
                .await
                .map_err(PluginError::from)?;
            Ok("{}".to_string())
        }
        "grab" => {
            let url = need("url")?;
            let path = need("path")?;
            let message = page.grab(url, path).await.map_err(PluginError::from)?;
            Ok(json!({ "message": message }).to_string())
        }
        "capture-start" => {
            page.start_media_capture()
                .await
                .map_err(PluginError::from)?;
            Ok("{}".to_string())
        }
        "capture-save" => {
            let path = need("path")?;
            let message = page.save_capture(path).await.map_err(PluginError::from)?;
            Ok(json!({ "message": message }).to_string())
        }

        other => Err(PluginError::Unsupported(format!(
            "unknown browser primitive '{other}'"
        ))),
    }
}

/// Wrap a page so plugins can act on it through the capability-scoped host.
pub fn page_host(page: Arc<Page>) -> ArcPageHost {
    Arc::new(PageHostImpl { page })
}

/// Load a sandboxed WebAssembly plugin from `path` (a directory or a
/// `plugin.json`) into a handle that does **not** belong to any browser.
///
/// This is how a host that is not a `Browser` - a native-only run, say - can
/// still load and drive plugins: it owns a [`PluginManager`] and installs the
/// result. Cross-plugin calls are unavailable on such a host, and the plugin's
/// `browser` bridge is closed (no page is bound), but `app` and pure ops work.
pub fn load_plugin(path: &str) -> XcelerateResult<Arc<dyn Plugin>> {
    let manifest_path = resolve_manifest_path(path)?;
    let manifest = Manifest::load(&manifest_path.to_string_lossy())?;
    manifest
        .validate_reserved(&[])
        .map_err(|error| XcelerateError::Unsupported(error.to_string()))?;
    #[cfg(feature = "wasm")]
    {
        let plugin_dir = manifest_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));
        let plugin = wasm::load(&manifest, plugin_dir, None, None)?;
        Ok(Arc::new(plugin) as Arc<dyn Plugin>)
    }
    #[cfg(not(feature = "wasm"))]
    {
        let _ = manifest;
        Err(XcelerateError::Unsupported(
            "plugin loading requires the `wasm` feature".to_string(),
        ))
    }
}

// ---------------------------------------------------------------------------
// UniFFI bridge
// ---------------------------------------------------------------------------

/// A handle to an enabled plugin, exposed to every language.
#[derive(uniffi::Object)]
pub struct PluginHandle {
    manager: Arc<PluginManager>,
    name: String,
}

impl PluginHandle {
    pub fn new(manager: Arc<PluginManager>, name: String) -> Arc<Self> {
        Arc::new(Self { manager, name })
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl PluginHandle {
    /// The plugin's name.
    pub fn plugin_name(&self) -> String {
        self.name.clone()
    }

    /// The ops this plugin exposes.
    pub fn ops(&self) -> Vec<String> {
        self.manager.ops(&self.name)
    }

    /// Invoke an op with a JSON-encoded argument object; returns JSON.
    pub async fn invoke(&self, op: String, args_json: String) -> XcelerateResult<String> {
        xcelerate_plugin::audit(&self.name, &op, "invoke");
        self.manager
            .invoke(&self.name, &op, args_json, None)
            .await
            .map_err(XcelerateError::from)
    }
}

impl xcelerate_plugin::OpInvoker for PluginHandle {
    fn invoke_op<'a>(
        &'a self,
        op: &'a str,
        args_json: String,
    ) -> xcelerate_plugin::BoxFutLt<'a, xcelerate_plugin::PluginResult<String>> {
        Box::pin(async move {
            xcelerate_plugin::audit(&self.name, op, "invoke");
            self.manager.invoke(&self.name, op, args_json, None).await
        })
    }
}

/// Typed, Rust-only calling. Kept out of the `#[uniffi::export]` block so it does
/// not change the binding checksum.
impl PluginHandle {
    /// Serialize `args`, run `op`, deserialize the result - no JSON in your code.
    ///
    /// ```ignore
    /// let report: FillReport = handle.call("fill_register", &profile).await?;
    /// ```
    pub async fn call<A, R>(&self, op: impl Into<String>, args: &A) -> XcelerateResult<R>
    where
        A: serde::Serialize,
        R: serde::de::DeserializeOwned,
    {
        let args_json = serde_json::to_string(args)
            .map_err(|error| XcelerateError::SerdeError(error.to_string()))?;
        let result = self.invoke(op.into(), args_json).await?;
        serde_json::from_str(&result).map_err(|error| XcelerateError::SerdeError(error.to_string()))
    }

    /// The plugin's per-op config (input schema + defaults) the component
    /// advertised in its `describe` handshake, as a JSON object keyed by op:
    /// `{"op": {"schema": "...json-schema...", "defaults": "{...}"}}`.
    /// Empty `{}` for plugins that do not carry config (schema_version 0).
    pub fn config(&self) -> XcelerateResult<String> {
        let config = self
            .manager
            .manifest(&self.name)
            .map(|manifest| manifest.config)
            .unwrap_or_default();
        serde_json::to_string(&config)
            .map_err(|error| XcelerateError::SerdeError(error.to_string()))
    }

    /// Invoke an op **bound to a live page**, so a plugin granted the `browser`
    /// capability can act on it through `host.browser`.
    ///
    /// Rust-only: the language bindings call [`PluginHandle::invoke`], which
    /// carries no page and therefore leaves the browser bridge closed.
    pub async fn invoke_on(
        &self,
        op: String,
        args_json: String,
        page: Arc<Page>,
    ) -> XcelerateResult<String> {
        xcelerate_plugin::audit(&self.name, &op, "invoke");
        self.manager
            .invoke(&self.name, &op, args_json, Some(page_host(page)))
            .await
            .map_err(XcelerateError::from)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use xcelerate_plugin::{Budgets, Registry};

    /// A minimal *external* plugin. The engine ships none of its own, so the
    /// tests define one and install it in-process - exactly what an embedder
    /// does with a plugin crate.
    struct Echo;

    impl Plugin for Echo {
        fn name(&self) -> &str {
            "echo"
        }

        fn manifest(&self) -> Manifest {
            // Built directly (not via `from_json`): an in-process plugin has no
            // sandbox `entrypoint`.
            Manifest {
                name: "echo".to_string(),
                version: "0.1.0".to_string(),
                host_api: "1.x".to_string(),
                entrypoint: None,
                abi: None,
                ops: vec!["info".to_string()],
                capabilities: Vec::new(),
                dependencies: Default::default(),
                overrides: Default::default(),
                limits: Budgets::default(),
                config: Default::default(),
            }
        }

        fn build(&self, reg: &mut Registry) {
            reg.op("info", |_call| {
                Box::pin(async { Ok(r#"{"name":"echo"}"#.to_string()) })
            });
        }
    }

    #[test]
    fn core_ships_no_built_in_plugins() {
        // The catalog resolves nothing: every plugin is external.
        let manager = PluginManager::new(&[], catalog()).unwrap();
        assert!(manager.names().is_empty());
        assert!(!manager.has("stealth"));
        assert!(!manager.has("human"));
    }

    #[test]
    fn refuses_unknown_plugin() {
        // With no catalog, a name can never be resolved to code.
        match PluginManager::new(&["stealth".to_string()], catalog()) {
            Err(PluginError::Unsupported(_)) => {}
            Err(other) => panic!("unexpected error: {other:?}"),
            Ok(_) => panic!("expected an unknown plugin to be refused"),
        }
    }

    #[tokio::test]
    async fn installed_plugin_is_available_with_its_ops() {
        let manager = PluginManager::new(&[], catalog()).unwrap();
        manager.install(Arc::new(Echo)).unwrap();
        assert!(manager.has("echo"));
        assert_eq!(manager.names(), vec!["echo".to_string()]);
        assert_eq!(manager.ops("echo"), vec!["info".to_string()]);

        let out = manager
            .invoke("echo", "info", "{}".to_string(), None)
            .await
            .unwrap();
        assert!(out.contains("\"name\":\"echo\""));

        let err = manager
            .invoke("echo", "missing", "{}".to_string(), None)
            .await
            .unwrap_err();
        assert!(matches!(err, PluginError::NotFound(_)));
    }
}
