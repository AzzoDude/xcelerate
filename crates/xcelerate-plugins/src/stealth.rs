//! First-party `stealth` plugin.
//!
//! Patches the browser binary at launch and injects the anti-fingerprint payload
//! into every new document. The OS-level implementation lives in this crate
//! ([`crate::patcher`]); this module adapts it onto the plugin API.

use xcelerate_plugin_api::{
    ArcPageHost, BoxFut, Budgets, Capability, LaunchPlan, Manifest, Plugin, PluginResult, Registry,
    Tier,
};

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
            tier: Tier::FirstParty,
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
            limits: Budgets::default(),
        }
    }

    fn configure_launch(&self, plan: &mut LaunchPlan) -> PluginResult<()> {
        if !plan.patched {
            plan.executable = crate::BinaryPatcher::patch_to_temp(&plan.executable)?;
            plan.patched = true;
        }
        Ok(())
    }

    fn build(&self, reg: &mut Registry) {
        reg.op("info", |_call| {
            Box::pin(async {
                Ok(serde_json::json!({
                    "name": "stealth",
                    "tier": "first-party",
                    "enabled": true,
                    "patched": true,
                })
                .to_string())
            })
        });
    }

    fn on_page_created(&self, page: ArcPageHost) -> BoxFut<PluginResult<()>> {
        Box::pin(async move {
            page.add_init_script(crate::CDC_PAYLOAD.to_string()).await?;
            // Enable the Page domain so navigation/frame events fire.
            page.dispatch_cdp("Page.enable".to_string(), "{}".to_string())
                .await?;
            Ok(())
        })
    }
}
