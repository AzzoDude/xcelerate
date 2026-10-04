//! WebAssembly (Component Model) plugin transport.
//!
//! A plugin is a single sandboxed `*.wasm` component that imports the `host`
//! interface and exports the standard `plugin` interface (see
//! `crates/xcelerate/wit/plugin.wit`). The host runs it with `wasmtime`, does a
//! `describe` handshake (MessagePack), and drives `invoke`.
//!
//! Each plugin gets its own [`Store`] - its own sandbox, capabilities, and grant
//! set - so there is no ambient filesystem or network: a component reaches the
//! outside world only through the granted, audited `host` callbacks.

use std::collections::HashSet;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};

use wasmtime::component::{Component, HasSelf, Linker, ResourceTable};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

use xcelerate_plugin_api::{
    Capability, Manifest, OpCall, Plugin, PluginError, PluginResult, Registry,
};

use crate::{CdpClient, XcelerateError, XcelerateResult};

wasmtime::component::bindgen!({
    path: "wit/plugin.wit",
    world: "plugin-world",
});

/// Cap on a returned payload, mirroring the manifest's `max_response_bytes` ceiling.
const MAX_RESULT_BYTES: usize = 1024 * 1024;

/// Whether an `abi` string selects the WebAssembly transport.
pub(crate) fn is_wasm_abi(abi: &str) -> bool {
    abi.starts_with("wasm")
}

// ---------------------------------------------------------------------------
// Host state: WASI context + capability grants
// ---------------------------------------------------------------------------

struct HostState {
    wasi: WasiCtx,
    table: ResourceTable,
    plugin: String,
    granted: HashSet<Capability>,
    #[allow(dead_code)]
    client: Option<Arc<CdpClient>>,
}

impl WasiView for HostState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

impl HostState {
    /// Enforce a capability, auditing both the grant and the denial.
    fn require(&self, capability: Capability) -> Result<(), String> {
        if self.granted.contains(&capability) {
            xcelerate_plugin_api::audit(&self.plugin, capability.as_str(), "grant");
            Ok(())
        } else {
            xcelerate_plugin_api::audit(&self.plugin, capability.as_str(), "deny");
            Err(format!(
                "capability '{}' was not granted to plugin '{}'",
                capability.as_str(),
                self.plugin
            ))
        }
    }
}

impl self::xcelerate::plugin::types::Host for HostState {}

impl self::xcelerate::plugin::host::Host for HostState {
    fn log(&mut self, message: String) {
        // Never let a plugin grow the log without bound.
        let message: String = message.chars().take(1024).collect();
        xcelerate_plugin_api::audit(&self.plugin, "log", &message);
    }

    fn get_cookies(&mut self) -> Result<Vec<self::xcelerate::plugin::types::Cookie>, String> {
        self.require(Capability::ReadCookies)?;
        Err("wasm transport: cookie access is not implemented yet".to_string())
    }

    fn set_cookie(
        &mut self,
        _cookie: self::xcelerate::plugin::types::Cookie,
    ) -> Result<(), String> {
        self.require(Capability::WriteCookies)?;
        Err("wasm transport: cookie access is not implemented yet".to_string())
    }
}

// ---------------------------------------------------------------------------
// Loaded component
// ---------------------------------------------------------------------------

/// The handshake payload a plugin's `describe` returns.
#[derive(serde::Deserialize)]
struct Describe {
    name: String,
    #[serde(default)]
    #[allow(dead_code)]
    version: String,
    #[serde(default)]
    ops: Vec<String>,
}

/// Shared, interior-mutable handle to a running component instance.
struct WasmInstance {
    store: Mutex<Store<HostState>>,
    bindings: PluginWorld,
}

impl WasmInstance {
    fn invoke(&self, op: &str, args_json: &str) -> PluginResult<String> {
        let args = json_to_msgpack(args_json)?;
        let mut store = self
            .store
            .lock()
            .map_err(|_| PluginError::Message("wasm plugin store poisoned".to_string()))?;
        let guest = self.bindings.xcelerate_plugin_plugin();
        let outcome = guest
            .call_invoke(&mut *store, op, &args)
            .map_err(|error| PluginError::Message(format!("wasm: {error}")))?;
        match outcome {
            Ok(payload) => msgpack_to_json(&payload),
            Err(message) => Err(PluginError::Message(message)),
        }
    }
}

/// A loaded WebAssembly plugin.
pub(crate) struct WasmPlugin {
    manifest: Manifest,
    ops: Vec<String>,
    instance: Arc<WasmInstance>,
}

impl Plugin for WasmPlugin {
    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn manifest(&self) -> Manifest {
        self.manifest.clone()
    }

    fn build(&self, reg: &mut Registry) {
        for op in &self.ops {
            let instance = Arc::clone(&self.instance);
            let op = op.clone();
            reg.op(&op, move |call: OpCall| {
                let instance = Arc::clone(&instance);
                Box::pin(async move { instance.invoke(&call.op, &call.args_json) })
            });
        }
    }
}

/// Load and instantiate a wasm component for `manifest`, returning it ready to
/// install.
pub(crate) fn load(
    manifest: &Manifest,
    plugin_dir: &Path,
    client: Option<Arc<CdpClient>>,
) -> XcelerateResult<WasmPlugin> {
    if let Some(abi) = &manifest.abi
        && !is_wasm_abi(abi)
    {
        return Err(XcelerateError::Unsupported(format!(
            "plugin '{}' declares abi '{abi}'; expected a wasm component",
            manifest.name
        )));
    }

    let entrypoint = resolve_entrypoint(plugin_dir, manifest)?;
    let engine = engine();
    let component = Component::from_file(&engine, &entrypoint).map_err(|error| {
        XcelerateError::Unsupported(format!(
            "plugin '{}': not a valid wasm component: {error}",
            manifest.name
        ))
    })?;

    let mut linker = Linker::new(&engine);
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker).map_err(|error| {
        XcelerateError::Unsupported(format!(
            "plugin '{}': wasi link failed: {error}",
            manifest.name
        ))
    })?;
    PluginWorld::add_to_linker::<HostState, HasSelf<HostState>>(&mut linker, |state| state)
        .map_err(|error| {
            XcelerateError::Unsupported(format!(
                "plugin '{}': host link failed: {error}",
                manifest.name
            ))
        })?;

    let state = HostState {
        wasi: WasiCtxBuilder::new().build(),
        table: ResourceTable::new(),
        plugin: manifest.name.clone(),
        granted: granted_capabilities(manifest, &allow_list()),
        client,
    };
    let mut store = Store::new(&engine, state);
    let bindings = PluginWorld::instantiate(&mut store, &component, &linker).map_err(|error| {
        XcelerateError::Unsupported(format!(
            "plugin '{}': instantiate failed: {error}",
            manifest.name
        ))
    })?;

    // `describe` handshake: the component must agree with its manifest.
    let payload = bindings
        .xcelerate_plugin_plugin()
        .call_describe(&mut store)
        .map_err(|error| {
            XcelerateError::Unsupported(format!(
                "plugin '{}': describe failed: {error}",
                manifest.name
            ))
        })?;
    let describe: Describe = rmp_serde::from_slice(&payload).map_err(|error| {
        XcelerateError::Unsupported(format!(
            "plugin '{}': describe is not a MessagePack manifest: {error}",
            manifest.name
        ))
    })?;
    if describe.name != manifest.name {
        return Err(XcelerateError::Unsupported(format!(
            "plugin '{}' reported name '{}'",
            manifest.name, describe.name
        )));
    }
    let ops = if describe.ops.is_empty() {
        manifest.ops.clone()
    } else {
        describe.ops
    };

    Ok(WasmPlugin {
        manifest: manifest.clone(),
        ops,
        instance: Arc::new(WasmInstance {
            store: Mutex::new(store),
            bindings,
        }),
    })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A process-wide engine. Compilation config is identical for every plugin, so
/// one engine (and its compiled-code cache) is shared.
fn engine() -> Engine {
    static ENGINE: OnceLock<Engine> = OnceLock::new();
    ENGINE
        .get_or_init(|| {
            let mut config = Config::new();
            config.wasm_component_model(true);
            Engine::new(&config).expect("failed to create the wasm engine")
        })
        .clone()
}

/// Resolve the entrypoint, requiring it to stay inside the plugin directory.
fn resolve_entrypoint(
    plugin_dir: &Path,
    manifest: &Manifest,
) -> XcelerateResult<std::path::PathBuf> {
    let entrypoint = manifest.entrypoint.as_deref().ok_or_else(|| {
        XcelerateError::Unsupported(format!("plugin '{}' has no entrypoint", manifest.name))
    })?;
    let base = plugin_dir.canonicalize().map_err(|error| {
        XcelerateError::NotFound(format!(
            "plugin '{}': cannot resolve plugin directory: {error}",
            manifest.name
        ))
    })?;
    let resolved = base.join(entrypoint).canonicalize().map_err(|error| {
        XcelerateError::NotFound(format!(
            "plugin '{}' entrypoint not found: {error}",
            manifest.name
        ))
    })?;
    if !resolved.starts_with(&base) {
        return Err(XcelerateError::Unsupported(format!(
            "plugin '{}' entrypoint escapes the plugin directory",
            manifest.name
        )));
    }
    if !resolved.is_file() {
        return Err(XcelerateError::Unsupported(format!(
            "plugin '{}' entrypoint is not a regular file",
            manifest.name
        )));
    }
    Ok(resolved)
}

/// The capabilities granted to a plugin: host-only ones are never granted, and
/// dangerous ones require an explicit opt-in via `XCELERATE_PLUGIN_ALLOW`.
fn granted_capabilities(manifest: &Manifest, allow: &HashSet<String>) -> HashSet<Capability> {
    manifest
        .capabilities
        .iter()
        .copied()
        .filter(|capability| !capability.is_builtin_only())
        .filter(|capability| {
            !capability.is_dangerous()
                || allow.contains(capability.as_str())
                || allow.contains(&format!("{}:{}", manifest.name, capability.as_str()))
        })
        .collect()
}

/// Parse the `XCELERATE_PLUGIN_ALLOW` opt-in list (comma separated).
fn allow_list() -> HashSet<String> {
    std::env::var("XCELERATE_PLUGIN_ALLOW")
        .unwrap_or_default()
        .split(',')
        .map(|entry| entry.trim().to_string())
        .filter(|entry| !entry.is_empty())
        .collect()
}

/// JSON (the cross-language `invoke` argument form) to a MessagePack payload.
fn json_to_msgpack(args_json: &str) -> PluginResult<Vec<u8>> {
    let value: serde_json::Value = if args_json.trim().is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::from_str(args_json)
            .map_err(|error| PluginError::Message(format!("args are not JSON: {error}")))?
    };
    rmp_serde::to_vec_named(&value).map_err(|error| PluginError::Message(error.to_string()))
}

/// A MessagePack result payload back to JSON.
fn msgpack_to_json(payload: &[u8]) -> PluginResult<String> {
    if payload.len() > MAX_RESULT_BYTES {
        return Err(PluginError::Unsupported(format!(
            "wasm result of {} bytes exceeds the {MAX_RESULT_BYTES} byte cap",
            payload.len()
        )));
    }
    let value: serde_json::Value =
        rmp_serde::from_slice(payload).map_err(|error| PluginError::Message(error.to_string()))?;
    Ok(value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use xcelerate_plugin_api::PluginManager;

    /// Loads the prebuilt `wasm-echo` component and invokes `echo`.
    ///
    /// Skipped unless the example has been built:
    ///
    /// ```text
    /// cd docs/plugins/examples/wasm-echo && cargo build --release --target wasm32-wasip2
    /// ```
    #[tokio::test]
    async fn loads_and_invokes_a_wasm_component() {
        let dir = Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../docs/plugins/examples/wasm-echo"
        ));
        if !dir.join("wasm-echo.wasm").exists() {
            eprintln!("skipping: build docs/plugins/examples/wasm-echo for wasm32-wasip2 first");
            return;
        }

        let manifest_text = std::fs::read_to_string(dir.join("plugin.json")).unwrap();
        let manifest = Manifest::from_json(&manifest_text).unwrap();
        let plugin = load(&manifest, dir, None).unwrap();
        assert_eq!(plugin.name(), "example.wasm-echo");

        let catalog: xcelerate_plugin_api::Catalog =
            Arc::new(|_: &str| -> Option<Arc<dyn Plugin>> { None });
        let manager = PluginManager::new(&[], catalog).unwrap();
        manager.install(Arc::new(plugin)).unwrap();

        let out = manager
            .invoke(
                "example.wasm-echo",
                "echo",
                r#"{"value":42}"#.to_string(),
                None,
            )
            .await
            .unwrap();
        assert!(out.contains("42"), "unexpected echo result: {out}");
    }

    /// Loads a mod from a directory given by `XCELERATE_TEST_MOD` and invokes
    /// its `hello` op. Skipped when the variable is unset. Useful for manually
    /// checking a scaffolded mod:
    ///
    /// ```text
    /// set XCELERATE_TEST_MOD=C:\path\to\my-mod
    /// cargo test -p xcelerate --lib plugin::wasm
    /// ```
    #[tokio::test]
    async fn loads_a_mod_pointed_at_by_env() {
        let Ok(dir) = std::env::var("XCELERATE_TEST_MOD") else {
            return;
        };
        let dir = PathBuf::from(dir);
        let manifest = Manifest::load(&dir.join("plugin.json").to_string_lossy()).unwrap();
        let plugin = load(&manifest, &dir, None).unwrap();

        let catalog: xcelerate_plugin_api::Catalog =
            Arc::new(|_: &str| -> Option<Arc<dyn Plugin>> { None });
        let manager = PluginManager::new(&[], catalog).unwrap();
        manager.install(Arc::new(plugin)).unwrap();

        let out = manager
            .invoke(
                &manifest.name,
                "hello",
                r#"{"name":"Ada"}"#.to_string(),
                None,
            )
            .await
            .unwrap();
        assert!(out.contains("Ada"), "unexpected hello result: {out}");
    }
}
