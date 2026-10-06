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
use xcelerate_plugin::{ArcPageHost, BoxFut, Catalog, PageHost, PluginError, PluginResult};

pub use xcelerate_plugin::{
    AuditEvent, Capability, Manifest, Plugin, PluginManager, audit_entries, audit_verify,
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
}

/// Wrap a page so plugins can act on it through the capability-scoped host.
pub(crate) fn page_host(page: Arc<Page>) -> ArcPageHost {
    Arc::new(PageHostImpl { page })
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
