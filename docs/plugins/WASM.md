# WebAssembly plugins (typed, not JSON)

`rpc/1` (out-of-process, JSON) is the quick-start transport: any language, zero
dependencies. The **wasm** transport is the production one: one portable
artifact, a real sandbox, and a **typed binary** interface instead of JSON.

| | `rpc/1` subprocess | `wasm32-wasip2/1` component |
| --- | --- | --- |
| Artifact | interpreter/script or native binary per OS | one `.wasm`, runs everywhere |
| Isolation | separate process (OS sandbox pending) | sandboxed by construction |
| Wire format | JSON | MessagePack (binary) |
| Interface | string ops, `{...}` args | typed WIT interface |
| Runtime | none | `wasmtime` |

The ABI is selected by the manifest's `abi`:

```json
{ "abi": "rpc/1" }               // subprocess transport (built)
{ "abi": "wasm32-wasip2/1" }     // wasmtime transport (opt-in)
```

## The interface ([`crates/xcelerate/wit/plugin.wit`](../../crates/xcelerate/wit/plugin.wit))

```wit
world plugin-world {
    import host;   // capability-gated: log, get-cookies, set-cookie
    export plugin; // describe() -> payload, invoke(op, args) -> result<payload, string>
}
```

- `describe` returns `{ name, version, ops }`; the host checks it against
  `plugin.json`.
- `invoke(op, args)` takes and returns a `payload` (a `list<u8>`), encoded with
  **MessagePack** - compact and cheap, and the structs stay typed through
  `serde` on both sides. No JSON parsing on the hot path.
- Host callbacks keep the same **default-deny capability gate**: a `get-cookies`
  call is refused unless `read_cookies` was granted, and is audited.

## Why this is the fast + low-boilerplate option

- **Low boilerplate**: `wit-bindgen` generates the guest stubs (Rust, C, Go, …)
  and `wasmtime::component::bindgen!` generates the host bindings from the same
  WIT - authors write only the op bodies.
- **Fast**: typed calls over the component boundary (no serialize→pipe→parse),
  ~ms instantiation, AOT-compilable with Cranelift.
- **Safe**: no ambient filesystem/network; the sandbox is enforced by the
  runtime, which is what the `rpc/1` path can't provide.

## Host transport sketch (planned)

Not yet wired - shown so the shape is concrete. It would live behind a `wasm`
cargo feature in `crates/xcelerate/src/plugin/wasm.rs` and reuse the same
capability/audit layer.

```rust
wasmtime::component::bindgen!({ path: "wit/plugin.wit", world: "plugin" });

struct HostState { granted: HashSet<Capability>, client: Option<Arc<CdpClient>> }

impl exports::xcelerate::plugin::host::Host for HostState {
    fn log(&mut self, message: String) { audit("log", &message); }
    fn get_cookies(&mut self) -> Result<Vec<Cookie>, String> {
        self.require(Capability::ReadCookies)?;   // default-deny, audited
        /* ... Storage.getCookies via the client ... */
    }
    fn set_cookie(&mut self, cookie: Cookie) -> Result<(), String> {
        self.require(Capability::WriteCookies)?; /* ... */
    }
}

// load
let engine = Engine::new(&config)?;                 // component model enabled
let component = Component::from_file(&engine, path)?;
let mut store = Store::new(&engine, HostState::new(grants, client));
let mut linker = Linker::new(&engine);
Plugin::add_to_linker(&mut linker, |state| state)?;
let bindings = Plugin::instantiate(&mut store, &component, &linker)?;

let manifest: Manifest = rmp_serde::from_slice(&bindings.call_describe(&mut store)?)?;
let result = bindings.call_invoke(&mut store, op, &args)?;   // Result<Vec<u8>, String>
```

Everything downstream - the `Plugin` trait, capability grants, audit, timeouts,
`plugin(name).invoke(...)` in every language - is unchanged; only the transport
differs.

## Status

- **Delivered here:** the **WIT contract**
  ([`wit/plugin.wit`](../../crates/xcelerate/wit/plugin.wit)) and the **guest
  example** ([`examples/wasm-echo`](examples/wasm-echo)).
- **Not yet wired:** the `wasmtime` host transport. It is a deliberate opt-in - a
  `wasm` cargo feature - because `wasmtime` is a large dependency (it should not
  bloat the shipped cdylib for the `rpc/1` majority). Wiring it needs a machine
  with the toolchain below and network access for the crates:

  ```bash
  rustup target add wasm32-wasip2
  ```

  `wasm32-wasip2` emits a Component Model component directly, so no separate
  componentizer or WASI adapter is required. (`wasm-tools` is handy for
  inspecting a component: `wasm-tools component wit plugin.wasm`.)

The subprocess (`rpc/1`) path remains the default and keeps working for any
language; wasm is the upgrade for sandboxed, typed, high-performance plugins.
