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
use std::time::Duration;

use wasmtime::component::{Component, HasSelf, Linker, ResourceTable};
use wasmtime::{Config, Engine, Store, StoreLimits, StoreLimitsBuilder, Trap};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

use xcelerate_plugin::{Capability, Manifest, OpCall, Plugin, PluginError, PluginResult, Registry};

use crate::{CdpClient, XcelerateError, XcelerateResult};

wasmtime::component::bindgen!({
    path: "wit/plugin.wit",
    world: "plugin-world",
});

/// Cap on a returned payload, mirroring the manifest's `max_response_bytes` ceiling.
const MAX_RESULT_BYTES: usize = 1024 * 1024;

/// Hard ceiling on a guest's linear memory. A plugin can never grow past this,
/// so a runaway guest cannot exhaust host memory. Kept as an internal constant
/// (never a manifest field) so it cannot be raised by a plugin on disk.
const MAX_GUEST_MEMORY_BYTES: usize = 256 * 1024 * 1024;

/// Wall-clock granularity of epoch-based interruption, in milliseconds. The
/// engine's epoch advances once per tick, so a per-invocation budget of `n` ms
/// becomes an epoch deadline of `ceil(n / EPOCH_TICK_MILLIS)` ticks.
const EPOCH_TICK_MILLIS: u64 = 50;

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
    /// The guest's memory/table limits, installed on the store via
    /// `Store::limiter` so growth is refused rather than unbounded.
    limits: StoreLimits,
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
            xcelerate_plugin::audit(&self.plugin, capability.as_str(), "grant");
            Ok(())
        } else {
            xcelerate_plugin::audit(&self.plugin, capability.as_str(), "deny");
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
        xcelerate_plugin::audit(&self.plugin, "log", &message);
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
    /// Epoch ticks a single guest call may consume before it is interrupted,
    /// derived from the manifest's per-invocation time budget.
    epoch_deadline_ticks: u64,
}

impl WasmInstance {
    fn invoke(&self, op: &str, args_json: &str) -> PluginResult<String> {
        let args = json_to_msgpack(args_json)?;
        let mut store = self
            .store
            .lock()
            .map_err(|_| PluginError::Message("wasm plugin store poisoned".to_string()))?;
        // Re-arm the deadline: it is relative to the engine's current epoch,
        // which has been advancing on the ticker thread since the last call.
        store.set_epoch_deadline(self.epoch_deadline_ticks);
        let guest = self.bindings.xcelerate_plugin_plugin();
        let outcome = guest
            .call_invoke(&mut *store, op, &args)
            .map_err(|error| guest_error(error, self.epoch_deadline_ticks))?;
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
        limits: StoreLimitsBuilder::new()
            .memory_size(MAX_GUEST_MEMORY_BYTES)
            .build(),
        client,
    };
    let mut store = Store::new(&engine, state);
    // Cap the guest's memory, and arm the epoch deadline so instantiation and
    // the `describe` handshake below are bounded too; `invoke` re-arms it.
    store.limiter(|state| &mut state.limits);
    let epoch_deadline_ticks = budget_ticks(manifest.limits.max_invoke_millis);
    store.set_epoch_deadline(epoch_deadline_ticks);
    store.epoch_deadline_trap();
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
            epoch_deadline_ticks,
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
            // Wall-clock interruption: guests are instrumented to check the
            // engine's epoch, which the ticker started below advances.
            config.epoch_interruption(true);
            let engine = Engine::new(&config).expect("failed to create the wasm engine");
            start_epoch_ticker(engine.clone());
            engine
        })
        .clone()
}

/// Advance the engine's epoch on a fixed cadence so a guest that overruns its
/// deadline traps instead of pinning the host. One process-wide thread serves
/// every plugin, since the epoch (and the engine) are shared.
fn start_epoch_ticker(engine: Engine) {
    std::thread::Builder::new()
        .name("xcelerate-wasm-epoch".to_string())
        .spawn(move || {
            let tick = Duration::from_millis(EPOCH_TICK_MILLIS);
            loop {
                std::thread::sleep(tick);
                engine.increment_epoch();
            }
        })
        .expect("failed to spawn the wasm epoch ticker");
}

/// Translate a per-invocation wall-clock budget into an epoch deadline (ticks).
fn budget_ticks(max_invoke_millis: u64) -> u64 {
    max_invoke_millis.div_ceil(EPOCH_TICK_MILLIS).max(1)
}

/// Map a guest error, naming the invocation budget when the epoch deadline was
/// what halted execution rather than the guest itself.
fn guest_error(error: wasmtime::Error, deadline_ticks: u64) -> PluginError {
    if error.downcast_ref::<Trap>() == Some(&Trap::Interrupt) {
        let millis = deadline_ticks.saturating_mul(EPOCH_TICK_MILLIS);
        return PluginError::Unsupported(format!(
            "wasm plugin exceeded its {millis} ms invocation budget and was interrupted"
        ));
    }
    PluginError::Message(format!("wasm: {error}"))
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
    use xcelerate_plugin::PluginManager;

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

        let catalog: xcelerate_plugin::Catalog =
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

    /// The bytes of `(module (func (export "spin") (loop br 0)))`: a core
    /// module exporting a function that spins forever. Hand-encoded because the
    /// `wat` feature is off for this build.
    fn runaway_module_bytes() -> Vec<u8> {
        vec![
            0x00, 0x61, 0x73, 0x6d, // "\0asm"
            0x01, 0x00, 0x00, 0x00, // version 1
            // type section: one `() -> ()` function type
            0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
            // function section: function 0 has type 0
            0x03, 0x02, 0x01, 0x00, // export section: "spin" -> function 0
            0x07, 0x08, 0x01, 0x04, b's', b'p', b'i', b'n', 0x00, 0x00,
            // code section: `loop br 0 end end`
            0x0a, 0x09, 0x01, 0x07, 0x00, 0x03, 0x40, 0x0c, 0x00, 0x0b, 0x0b,
        ]
    }

    /// A guest that loops forever must be interrupted by the epoch deadline
    /// rather than pinning the host, and the resulting error must name the
    /// budget.
    #[test]
    fn a_runaway_guest_is_interrupted() {
        let engine = engine();
        let module = wasmtime::Module::new(&engine, runaway_module_bytes()).unwrap();
        let mut store = Store::new(&engine, ());
        store.set_epoch_deadline(1);
        store.epoch_deadline_trap();
        let instance = wasmtime::Instance::new(&mut store, &module, &[]).unwrap();
        let spin = instance
            .get_typed_func::<(), ()>(&mut store, "spin")
            .unwrap();

        let error = spin.call(&mut store, ()).unwrap_err();
        let mapped = guest_error(error, 1);
        assert!(
            matches!(&mapped, PluginError::Unsupported(message) if message.contains("budget")),
            "expected a budget error naming the invocation budget, got {mapped:?}"
        );
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

        let catalog: xcelerate_plugin::Catalog =
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
