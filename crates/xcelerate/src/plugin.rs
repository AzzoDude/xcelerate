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

use crate::element::Element;
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

/// Whether `value` reads as a selector rather than visible text (the same rule
/// the interactive session and the XCL interpreter use).
fn looks_like_selector(value: &str) -> bool {
    let value = value.trim();
    value.starts_with(['#', '.', '['])
        || value.starts_with("//")
        || value.starts_with("xpath=")
        || value.starts_with("role=")
        || value.starts_with("label=")
        || value.starts_with("text=")
        || (value.contains('[') && value.contains(']'))
}

/// Resolve a click/fill target: a CSS selector, or visible text.
async fn resolve_target(page: &Arc<Page>, target: &str) -> PluginResult<Arc<Element>> {
    if looks_like_selector(target) {
        Arc::clone(page)
            .wait_for_selector(target.to_string())
            .await
            .map_err(PluginError::from)
    } else {
        Arc::clone(page)
            .find_clickable_by_text(target.to_string())
            .await
            .map_err(PluginError::from)?
            .ok_or_else(|| PluginError::NotFound(format!("no visible element contains {target:?}")))
    }
}

/// The semantic browser verbs the host bridge exposes to sandboxed plugins.
///
/// This is the decoupled browser API: a plugin names an act (`goto`, `click`,
/// `fill`, `snapshot`, …) and the host maps it onto the engine. A plugin never
/// speaks CDP/BiDi itself.
async fn browser_action(page: Arc<Page>, op: &str, args_json: &str) -> PluginResult<String> {
    use serde_json::{Value, json};

    let args: Value = if args_json.trim().is_empty() {
        Value::Null
    } else {
        serde_json::from_str(args_json)
            .map_err(|error| PluginError::Message(format!("args are not JSON: {error}")))?
    };
    let text = |key: &str| args.get(key).and_then(Value::as_str).map(str::to_string);

    match op {
        "goto" | "open" | "navigate" => {
            let url = text("url")
                .ok_or_else(|| PluginError::Unsupported("goto needs a 'url'".to_string()))?;
            page.navigate(url.clone())
                .await
                .map_err(PluginError::from)?;
            let _ = page.wait_for_navigation().await;
            Ok(json!({ "url": page.url().await.unwrap_or(url) }).to_string())
        }
        "url" => Ok(json!(page.url().await.map_err(PluginError::from)?).to_string()),
        "title" => Ok(json!(page.title().await.map_err(PluginError::from)?).to_string()),
        "html" | "content" => {
            Ok(json!(page.content().await.map_err(PluginError::from)?).to_string())
        }
        "markdown" | "md" => {
            Ok(json!(page.markdown().await.map_err(PluginError::from)?).to_string())
        }
        "text" => Ok(json!(
            page.evaluate_string("document.body ? document.body.innerText : ''".to_string())
                .await
                .map_err(PluginError::from)?
        )
        .to_string()),
        "snapshot" => {
            Ok(json!(page.agent_snapshot().await.map_err(PluginError::from)?).to_string())
        }
        "snapshot-json" => {
            Ok(json!(page.snapshot_json().await.map_err(PluginError::from)?).to_string())
        }
        "click" => {
            let target = text("target")
                .ok_or_else(|| PluginError::Unsupported("click needs a 'target'".to_string()))?;
            let element = resolve_target(&page, &target).await?;
            element.click_mouse().await.map_err(PluginError::from)?;
            Ok(json!({ "clicked": target }).to_string())
        }
        "hover" => {
            let target = text("target")
                .ok_or_else(|| PluginError::Unsupported("hover needs a 'target'".to_string()))?;
            let element = resolve_target(&page, &target).await?;
            element.hover_mouse().await.map_err(PluginError::from)?;
            Ok(json!({ "hovered": target }).to_string())
        }
        "fill" => {
            let target = text("target")
                .ok_or_else(|| PluginError::Unsupported("fill needs a 'target'".to_string()))?;
            let value = text("text").unwrap_or_default();
            let element = resolve_target(&page, &target).await?;
            element.type_text(value).await.map_err(PluginError::from)?;
            Ok(json!({ "filled": target }).to_string())
        }
        "press" => {
            let key = text("key")
                .ok_or_else(|| PluginError::Unsupported("press needs a 'key'".to_string()))?;
            page.keyboard_press(key.clone())
                .await
                .map_err(PluginError::from)?;
            Ok(json!({ "pressed": key }).to_string())
        }
        "scroll" => {
            let to = text("to").unwrap_or_else(|| "down".to_string());
            let js = match to.as_str() {
                "" | "down" => "window.scrollBy(0, window.innerHeight * 0.9)".to_string(),
                "up" => "window.scrollBy(0, -window.innerHeight * 0.9)".to_string(),
                "top" => "window.scrollTo(0, 0)".to_string(),
                "bottom" => "window.scrollTo(0, document.body.scrollHeight)".to_string(),
                other => match other.parse::<i64>() {
                    Ok(pixels) => format!("window.scrollBy(0, {pixels})"),
                    Err(_) => {
                        return Err(PluginError::Unsupported(
                            "scroll 'to' must be a pixel count or up|down|top|bottom".to_string(),
                        ));
                    }
                },
            };
            let _ = page.evaluate_string(js).await;
            Ok(json!({ "scrolled": to }).to_string())
        }
        "evaluate" => {
            let js = text("js")
                .ok_or_else(|| PluginError::Unsupported("evaluate needs 'js'".to_string()))?;
            Ok(page.evaluate_string(js).await.map_err(PluginError::from)?)
        }
        "wait" => {
            let selector = text("selector")
                .ok_or_else(|| PluginError::Unsupported("wait needs a 'selector'".to_string()))?;
            Arc::clone(&page)
                .wait_for_selector(selector.clone())
                .await
                .map_err(PluginError::from)?;
            Ok(json!({ "waited": selector }).to_string())
        }
        "find" => {
            let needle = text("text").unwrap_or_default();
            let count = page
                .find_text(needle.clone())
                .await
                .map_err(PluginError::from)?;
            Ok(json!({ "count": count, "text": needle }).to_string())
        }
        "screenshot" => {
            let full = args.get("full").and_then(Value::as_bool).unwrap_or(false);
            let png = if full {
                page.screenshot_full().await.map_err(PluginError::from)?
            } else {
                page.screenshot().await.map_err(PluginError::from)?
            };
            use base64::Engine as _;
            let encoded = base64::engine::general_purpose::STANDARD.encode(&png);
            Ok(json!({ "png_base64": encoded, "bytes": png.len() }).to_string())
        }
        other => Err(PluginError::Unsupported(format!(
            "unknown browser action '{other}'"
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
