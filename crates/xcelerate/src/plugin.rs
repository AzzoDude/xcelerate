//! Plugin manager and UniFFI bridge for the engine.
//!
//! The plugin *API* - the [`Plugin`] trait, [`Manifest`], audit log, and the
//! [`PageHost`] interface - lives in `xcelerate-plugin-api`. The first-party
//! implementations live in `xcelerate-plugins`. This module wires them into
//! [`crate::Browser`], implements [`PageHost`] on top of [`Page`], and exposes
//! [`PluginHandle`] to every language binding.

use std::sync::Arc;

use crate::error::{XcelerateError, XcelerateResult};
use crate::page::Page;
use xcelerate_plugin_api::{ArcPageHost, BoxFut, Catalog, PageHost, PluginError, PluginResult};

pub use xcelerate_plugin_api::{
    AuditEvent, Capability, Manifest, Plugin, PluginManager, Tier, audit_entries, audit_verify,
};
pub use xcelerate_plugins::{builtin_names, is_builtin};

/// The host's first-party plugin catalog.
pub(crate) fn catalog() -> Catalog {
    Arc::new(xcelerate_plugins::builtin)
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
        xcelerate_plugin_api::audit(&self.name, &op, "invoke");
        self.manager
            .invoke(&self.name, &op, args_json, None)
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

    fn manager(name: &str) -> PluginManager {
        PluginManager::new(&[name.to_string()], catalog()).unwrap()
    }

    #[test]
    fn reserved_names_cover_the_catalog() {
        assert!(is_builtin("stealth"));
        assert!(is_builtin("human"));
        assert_eq!(builtin_names(), &["stealth", "human"]);
        assert!(!is_builtin("other"));
    }

    #[test]
    fn human_plugin_is_available_with_its_ops() {
        let manager = manager("human");
        assert!(manager.has("human"));
        assert_eq!(manager.names(), vec!["human".to_string()]);
        let ops = manager.ops("human");
        for op in ["info", "move", "click", "type", "scroll", "delay"] {
            assert!(ops.contains(&op.to_string()), "missing op {op}");
        }
    }

    #[test]
    fn refuses_unknown_plugin() {
        match PluginManager::new(&["totally-not-real".to_string()], catalog()) {
            Err(PluginError::Unsupported(_)) => {}
            Err(other) => panic!("unexpected error: {other:?}"),
            Ok(_) => panic!("expected an unknown plugin to be refused"),
        }
    }

    #[test]
    fn enable_is_idempotent() {
        let manager = manager("stealth");
        manager.enable("stealth").unwrap();
        assert_eq!(manager.names(), vec!["stealth".to_string()]);
        assert!(matches!(
            manager.enable("third-party-thing").unwrap_err(),
            PluginError::Unsupported(_)
        ));
    }

    #[tokio::test]
    async fn invoke_runs_registered_op() {
        let manager = manager("stealth");
        let out = manager
            .invoke("stealth", "info", "{}".to_string(), None)
            .await
            .unwrap();
        assert!(out.contains("\"name\":\"stealth\""));

        let err = manager
            .invoke("stealth", "missing", "{}".to_string(), None)
            .await
            .unwrap_err();
        assert!(matches!(err, PluginError::NotFound(_)));
    }
}
