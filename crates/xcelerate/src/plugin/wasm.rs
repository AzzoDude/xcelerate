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

use xcelerate_plugin::{
    ArcPageHost, Capability, Manifest, OpCall, Plugin, PluginError, PluginResult, Registry,
};

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
    /// Synchronous bridge to another plugin's op, used to implement a plugin's
    /// *dependency* on another plugin (`host.invoke-plugin`). Runs the target op
    /// on a dedicated thread so an async cross-plugin call never deadlocks the
    /// tokio worker that is driving this guest.
    cross_plugin: Option<CrossPlugin>,
    /// The page this invocation is bound to, so `host.browser` can act on it.
    /// Set from `OpCall.page` before each guest call; `None` otherwise.
    page: Option<ArcPageHost>,
}

/// A synchronous, thread-safe call into another enabled plugin: `(plugin, op,
/// args_json) -> result_json`. Implemented by the browser host; the wasm
/// transport invokes it when a guest asks to call a dependency.
pub(crate) type CrossPlugin =
    Arc<dyn Fn(String, String, String) -> Result<String, String> + Send + Sync>;

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

    /// Run a native-window primitive through UI Automation (Windows only). The
    /// primitives mirror the XCL native verbs and the `desktop` plugin, so the two
    /// surfaces cannot drift apart. The `Uia` handle is created per call (it is
    /// not `Send`, so it may not live in the guest store).
    #[cfg(windows)]
    fn desktop_action(&mut self, op: &str, args_json: &str) -> Result<String, String> {
        use serde_json::{Value, json};

        let args: Value = if args_json.trim().is_empty() {
            Value::Null
        } else {
            serde_json::from_str(args_json).map_err(|error| error.to_string())?
        };
        let text = |key: &str| args.get(key).and_then(Value::as_str).map(str::to_string);
        let number = |key: &str| args.get(key).and_then(Value::as_i64);

        // Launching spawns a process and waits for its window; it needs no UIA
        // handle of its own (`crate::desktop::launch` makes one).
        if op == "launch" {
            let target = text("target").ok_or_else(|| "launch needs a 'target'".to_string())?;
            let title = text("title");
            let wait = number("wait_ms").unwrap_or(15_000).max(0) as u64;
            let outcome = crate::desktop::launch(&target, title.as_deref(), wait)
                .map_err(|error| error.to_string())?;
            let formatted = crate::desktop::format_window(&outcome.window);
            let message = if outcome.attached {
                format!("already running: {formatted}")
            } else {
                format!("started {target} -> {formatted}")
            };
            return Ok(json!({
                "window": outcome.window.name,
                "attached": outcome.attached,
                "message": message,
            })
            .to_string());
        }

        let mut uia = crate::desktop::Uia::new().map_err(|error| error.to_string())?;

        if op == "windows" {
            let windows: Vec<Value> = uia
                .windows()
                .map_err(|error| error.to_string())?
                .iter()
                .map(|window| {
                    json!({ "name": window.name, "class": window.class, "pid": window.pid })
                })
                .collect();
            return Ok(json!({ "windows": windows }).to_string());
        }

        let window = text("window").ok_or_else(|| "a 'window' title is required".to_string())?;
        let limit = number("limit").unwrap_or(400).max(1) as usize;

        match op {
            "snapshot" => {
                let infos = uia
                    .snapshot(&window, limit)
                    .map_err(|error| error.to_string())?;
                let elements: Vec<Value> = infos
                    .iter()
                    .map(|element| {
                        json!({
                            "index": element.index,
                            "role": element.role,
                            "name": element.name,
                            "value": element.value,
                            "rect": [element.rect.0, element.rect.1, element.rect.2, element.rect.3],
                        })
                    })
                    .collect();
                Ok(json!({ "elements": elements }).to_string())
            }
            "click-index" => {
                uia.snapshot(&window, limit)
                    .map_err(|error| error.to_string())?;
                let index = number("index").unwrap_or(0).max(0) as usize;
                let method = uia.click(index).map_err(|error| error.to_string())?;
                Ok(json!({ "method": method }).to_string())
            }
            "click-name" => {
                uia.snapshot(&window, limit)
                    .map_err(|error| error.to_string())?;
                let name = text("name").ok_or_else(|| "click-name needs a 'name'".to_string())?;
                let (index, method) = uia
                    .click_name(&window, &name, limit)
                    .map_err(|error| error.to_string())?;
                Ok(json!({ "index": index, "method": method }).to_string())
            }
            "set-value" => {
                let index = number("index")
                    .ok_or_else(|| "set-value needs an 'index'".to_string())?
                    .max(0) as usize;
                let value = text("text").unwrap_or_default();
                uia.snapshot(&window, limit)
                    .map_err(|error| error.to_string())?;
                uia.set_value(index, &value)
                    .map_err(|error| error.to_string())?;
                Ok(json!({ "set": index }).to_string())
            }
            "close" => {
                uia.close(&window).map_err(|error| error.to_string())?;
                Ok(json!({ "closed": window }).to_string())
            }
            "key" => {
                let key = text("key").ok_or_else(|| "key needs a 'key'".to_string())?;
                if is_dangerous_key(&key) {
                    return Err(format!("`{key}` escapes the app; refused"));
                }
                uia.key_name(&window, &key)
                    .map_err(|error| error.to_string())?;
                Ok(json!({ "key": key }).to_string())
            }
            "wheel" => {
                let notches = number("notches").unwrap_or(1) as i32;
                uia.wheel(&window, notches)
                    .map_err(|error| error.to_string())?;
                Ok(json!({ "wheeled": notches }).to_string())
            }
            "scroll" => {
                let notches = number("notches").unwrap_or(-1) as i32;
                let method = uia
                    .scroll(&window, notches)
                    .map_err(|error| error.to_string())?;
                Ok(json!({ "scrolled": notches, "method": method }).to_string())
            }
            other => Err(format!("unknown app primitive '{other}'")),
        }
    }

    /// Native window control is Windows only.
    #[cfg(not(windows))]
    fn desktop_action(&mut self, _op: &str, _args_json: &str) -> Result<String, String> {
        Err("native desktop control is Windows only".to_string())
    }

    /// Host primitives for the `core` plugin (the language's **stdlib**): stdout,
    /// time, and environment. These are the general-purpose acts that belong to
    /// neither driver, so they live here rather than in the browser or app
    /// bridge. The `core` plugin composes the user-facing verbs (`print`, …).
    fn core_action(&mut self, op: &str, args_json: &str) -> Result<String, String> {
        use serde_json::{Value, json};

        let args: Value = if args_json.trim().is_empty() {
            Value::Null
        } else {
            serde_json::from_str(args_json).map_err(|error| error.to_string())?
        };
        let text = |key: &str| args.get(key).and_then(Value::as_str).map(str::to_string);

        match op {
            "stdout" => {
                let message = text("message").or_else(|| text("text")).unwrap_or_default();
                println!("{message}");
                Ok(json!({ "printed": message }).to_string())
            }
            "now" => {
                let seconds = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|duration| duration.as_secs())
                    .unwrap_or(0);
                Ok(json!({ "seconds": seconds }).to_string())
            }
            "env" => {
                let name = text("name")
                    .or_else(|| text("var"))
                    .ok_or_else(|| "env needs a 'name'".to_string())?;
                Ok(json!({ "name": name, "value": std::env::var(&name).ok() }).to_string())
            }
            // Blocks the guest thread; the `core` plugin's std time verbs
            // (`sleep`, `await`, `wait-*`) compose it.
            "sleep" => {
                let ms = args
                    .get("ms")
                    .and_then(Value::as_u64)
                    .unwrap_or(0)
                    .min(3_600_000);
                std::thread::sleep(std::time::Duration::from_millis(ms));
                Ok(json!({ "slept": ms }).to_string())
            }
            // A bounded pseudo-random millisecond delay for `wait-random`.
            "random" => {
                let min = args.get("min").and_then(Value::as_i64).unwrap_or(0);
                let max = args.get("max").and_then(Value::as_i64).unwrap_or(min);
                Ok(json!({ "ms": pseudo_random(min, max) }).to_string())
            }
            other => Err(format!("unknown core action '{other}'")),
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

    fn invoke_plugin(
        &mut self,
        plugin: String,
        op: String,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, String> {
        self.require(Capability::InvokePlugin)?;
        let invoker = self
            .cross_plugin
            .clone()
            .ok_or_else(|| "cross-plugin calls are not available on this host".to_string())?;
        // The cross-plugin bridge speaks JSON (the language-agnostic args form).
        let args_json = msgpack_to_json(&args).map_err(|error| error.to_string())?;
        let result_json = invoker(plugin, op, args_json)?;
        // The result comes back as JSON; re-encode to MessagePack for the guest.
        let value: serde_json::Value = serde_json::from_str(&result_json)
            .map_err(|error| format!("cross-plugin result is not JSON: {error}"))?;
        rmp_serde::to_vec_named(&value).map_err(|error| error.to_string())
    }

    fn browser(&mut self, op: String, args: Vec<u8>) -> Result<Vec<u8>, String> {
        self.require(Capability::Browser)?;
        let page = self
            .page
            .clone()
            .ok_or_else(|| "no browser page is bound to this plugin invocation".to_string())?;
        let args_json = msgpack_to_json(&args).map_err(|error| error.to_string())?;
        // The guest is synchronous, so run the page future to completion on the
        // tokio runtime that is already driving it (see `run_blocking`).
        let result_json = run_blocking(page.browser_op(&op, &args_json))?;
        let value: serde_json::Value = serde_json::from_str(&result_json)
            .map_err(|error| format!("browser action '{op}' did not return JSON: {error}"))?;
        rmp_serde::to_vec_named(&value).map_err(|error| error.to_string())
    }

    fn desktop(&mut self, op: String, args: Vec<u8>) -> Result<Vec<u8>, String> {
        self.require(Capability::Desktop)?;
        let args_json = msgpack_to_json(&args).map_err(|error| error.to_string())?;
        let result_json = self.desktop_action(&op, &args_json)?;
        let value: serde_json::Value = serde_json::from_str(&result_json)
            .map_err(|error| format!("desktop action '{op}' did not return JSON: {error}"))?;
        rmp_serde::to_vec_named(&value).map_err(|error| error.to_string())
    }

    fn core(&mut self, op: String, args: Vec<u8>) -> Result<Vec<u8>, String> {
        self.require(Capability::Core)?;
        let args_json = msgpack_to_json(&args).map_err(|error| error.to_string())?;
        let result_json = self.core_action(&op, &args_json)?;
        let value: serde_json::Value = serde_json::from_str(&result_json)
            .map_err(|error| format!("core action '{op}' did not return JSON: {error}"))?;
        rmp_serde::to_vec_named(&value).map_err(|error| error.to_string())
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
    /// Non-zero when the component also carries per-op config (schema + defaults)
    /// in `schemas`. Optional and additive: legacy components omit it (0).
    #[serde(default)]
    #[allow(dead_code)]
    schema_version: u32,
    /// `op -> { schema, defaults }`, both JSON strings. Only trusted when
    /// `schema_version >= 1`.
    #[serde(default)]
    schemas: std::collections::BTreeMap<String, DescribeSchema>,
}

/// Per-op config carried in `describe` when `schema_version >= 1`.
#[derive(serde::Deserialize)]
struct DescribeSchema {
    #[serde(default)]
    schema: String,
    #[serde(default)]
    defaults: String,
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
    fn invoke(&self, op: &str, args_json: &str, page: Option<ArcPageHost>) -> PluginResult<String> {
        let args = json_to_msgpack(args_json)?;
        let mut store = self
            .store
            .lock()
            .map_err(|_| PluginError::Message("wasm plugin store poisoned".to_string()))?;
        // Re-arm the deadline: it is relative to the engine's current epoch,
        // which has been advancing on the ticker thread since the last call.
        store.set_epoch_deadline(self.epoch_deadline_ticks);
        // Bind this invocation's page so `host.browser` can act on it.
        store.data_mut().page = page;
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
                Box::pin(async move { instance.invoke(&call.op, &call.args_json, call.page) })
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
    cross_plugin: Option<CrossPlugin>,
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
        cross_plugin,
        page: None,
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

    // Fold any schema-advertising config into the manifest so callers (MCP/
    // bindings) can discover an op's typed input shape and defaults.
    let mut manifest = manifest.clone();
    if describe.schema_version >= 1 {
        for (op, schema) in describe.schemas {
            manifest.config.insert(
                op,
                xcelerate_plugin::OpSchema {
                    schema: schema.schema,
                    defaults: schema.defaults,
                },
            );
        }
    }

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

/// Drive an async page future from a synchronous guest call without deadlocking
/// the worker already running it: `block_in_place` hands that worker's other
/// tasks to a sibling thread, then we run the future to completion on the same
/// runtime (so the page's sockets stay on their original reactor).
fn run_blocking(future: xcelerate_plugin::BoxFut<PluginResult<String>>) -> Result<String, String> {
    let handle = tokio::runtime::Handle::try_current()
        .map_err(|_| "browser actions need a running tokio runtime".to_string())?;
    if handle.runtime_flavor() != tokio::runtime::RuntimeFlavor::MultiThread {
        return Err("browser actions need a multi-thread tokio runtime".to_string());
    }
    tokio::task::block_in_place(|| handle.block_on(future)).map_err(|error| error.to_string())
}

/// A bounded pseudo-random millisecond delay for the `core` `random` primitive.
/// Not cryptographic - `wait-random` only needs to look jittery.
fn pseudo_random(min: i64, max: i64) -> i64 {
    if max <= min {
        return min.max(0);
    }
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.subsec_nanos() as i64)
        .unwrap_or(0)
        ^ (std::process::id() as i64).rotate_left(17);
    let mut x = seed.wrapping_add(0x9E37_79B9_7F4A_7C15u64 as i64);
    x ^= x << 13;
    x ^= (x as u64 >> 7) as i64;
    x ^= x << 17;
    let span = (max - min).saturating_add(1);
    min + (x.abs() % span)
}

/// Key chords whose effect escapes the app into the desktop/session (open a
/// shell, lock the screen, force-quit). Mirrors the interpreter's list so a
/// plugin cannot smuggle one through the bridge.
#[cfg(windows)]
fn is_dangerous_key(name: &str) -> bool {
    let normalized: String = name.to_ascii_lowercase().replace(' ', "");
    matches!(
        normalized.as_str(),
        "win+r"
            | "meta+r"
            | "win+x"
            | "meta+x"
            | "win+l"
            | "meta+l"
            | "win+e"
            | "meta+e"
            | "alt+f4"
            | "ctrl+shift+esc"
            | "ctrl+alt+delete"
    )
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
/// dangerous ones require an explicit opt-in via `XCELERATE_PLUGIN_ALLOW` -
/// except for the standard-library plugins (`core`/`browser`/`app`), which are
/// shipped with the project and trusted by default.
fn granted_capabilities(manifest: &Manifest, allow: &HashSet<String>) -> HashSet<Capability> {
    let trusted = crate::plugin::is_standard_plugin(&manifest.name);
    manifest
        .capabilities
        .iter()
        .copied()
        .filter(|capability| !capability.is_builtin_only())
        .filter(|capability| {
            trusted
                || !capability.is_dangerous()
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
        let plugin = load(&manifest, dir, None, None).unwrap();
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
        let plugin = load(&manifest, &dir, None, None).unwrap();

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
