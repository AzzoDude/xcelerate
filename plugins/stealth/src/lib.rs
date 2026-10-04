//! The `stealth` plugin.
//!
//! Patches the browser binary at launch and injects the anti-fingerprint payload
//! into every document. The primitives it builds on - the [`BinaryPatcher`] and
//! the [`CDC_PAYLOAD`] - live here.

use xcelerate_plugin_api::{
    ArcPageHost, BoxFut, Budgets, Capability, LaunchPlan, Manifest, Plugin, PluginError,
    PluginResult, Registry,
};

pub mod error;
pub mod patcher;

pub use error::{Error, Result};
pub use patcher::BinaryPatcher;

/// JavaScript injected into every new document to mask common automation signals
/// (`navigator.webdriver`, `cdc_` leaks, `window.chrome`, WebGL, ...).
pub const CDC_PAYLOAD: &str = include_str!("cdc_payload.js");

impl From<Error> for PluginError {
    fn from(error: Error) -> Self {
        PluginError::Message(error.to_string())
    }
}

/// Patches the browser binary at launch and injects the anti-fingerprint payload
/// into every new document.
pub struct StealthPlugin;

impl Plugin for StealthPlugin {
    fn name(&self) -> &str {
        "stealth"
    }

    fn requires_launch(&self) -> bool {
        true
    }

    fn manifest(&self) -> Manifest {
        Manifest {
            name: "stealth".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            host_api: "1.x".to_string(),
            entrypoint: None,
            abi: Some("native/1".to_string()),
            ops: vec!["info".to_string()],
            capabilities: vec![
                Capability::LaunchControl,
                Capability::BinaryPatch,
                Capability::DetachedSpawn,
                Capability::InitScript,
            ],
            dependencies: Default::default(),
            overrides: Default::default(),
            limits: Budgets::default(),
        }
    }

    fn configure_launch(&self, plan: &mut LaunchPlan) -> PluginResult<()> {
        if !plan.patched {
            plan.executable = BinaryPatcher::patch_to_temp(&plan.executable)?;
            plan.patched = true;
        }
        Ok(())
    }

    fn build(&self, reg: &mut Registry) {
        reg.op("info", |_call| {
            Box::pin(async {
                Ok(serde_json::json!({
                    "name": "stealth",
                    "enabled": true,
                    "patched": true,
                })
                .to_string())
            })
        });
    }

    fn on_page_created(&self, page: ArcPageHost) -> BoxFut<PluginResult<()>> {
        Box::pin(async move {
            page.add_init_script(CDC_PAYLOAD.to_string()).await?;
            // Enable the Page domain so navigation/frame events fire.
            page.dispatch_cdp("Page.enable".to_string(), "{}".to_string())
                .await?;
            Ok(())
        })
    }
}
