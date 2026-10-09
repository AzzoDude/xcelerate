# WebAssembly plugins

Loaded plugins are **WebAssembly components**. There is one transport: a single
portable `.wasm` artifact, a real sandbox, and a **typed binary** interface
(MessagePack over the Component Model) instead of JSON on stdin/stdout.

| | plugin component |
| --- | --- |
| Artifact | one `.wasm`, runs on every OS/arch |
| Isolation | sandboxed by the runtime (no ambient filesystem/network) |
| Wire format | MessagePack (binary) |
| Interface | typed WIT interface |
| Runtime | `wasmtime` (Component Model) |

The manifest's `abi` selects the transport; the only accepted value is a wasm
component ABI:

```json
{ "abi": "wasm32-wasip2/1" }
```

## The interface ([`crates/xcelerate/wit/plugin.wit`](../../crates/xcelerate/wit/plugin.wit))

```wit
world plugin-world {
    // capability-gated: log, get-cookies, set-cookie,
    // invoke-plugin, browser, app
    import host;
    export plugin; // describe() -> payload, invoke(op, args) -> result<payload, string>
}
```

- `describe` returns `{ name, version, ops }`; the host checks the name against
  `plugin.json`.
- `invoke(op, args)` takes and returns a `payload` (a `list<u8>`), encoded with
  **MessagePack** - compact and cheap, and the structs stay typed through
  `serde` on both sides. No JSON parsing on the hot path. (The cross-language
  `PluginHandle::invoke(op, args_json)` bridge converts JSON args to MessagePack
  and back.)
- Host callbacks keep the same **default-deny capability gate**: a `get-cookies`
  call is refused unless `read_cookies` was granted, and is audited.
- The `browser` and `app` callbacks are the **host action bridge**: a plugin asks
  for a semantic verb (`goto`, `click`, `fill`, `snapshot`, `tree`, …) and the
  host performs it against the run's active page or a native window - never a raw
  protocol message. See [`plugins/browser`](../../plugins/browser/README.md) and
  [`plugins/app`](../../plugins/app/README.md).

## Building a guest

`wit-bindgen` generates the guest stubs (Rust, C, Go, …) from the same WIT;
authors write only the op bodies. Rust's `wasm32-wasip2` target emits a
Component Model component directly - no adapter or componentizer needed:

```bash
rustup target add wasm32-wasip2
cargo build --release --target wasm32-wasip2
```

Inside a plugin directory, prefer `xcelerate build --wasm-only`: the CLI writes
`wit/plugin.wit` (the canonical host ABI) and stages the `.wasm` next to
`plugin.json`, so a plugin never has to hand-copy or maintain the interface.

See the runnable [Rust guest example](examples/wasm-echo/README.md), and the
[provider/consumer pair](examples/kv-store/README.md) for a plugin that depends
on another plugin.

## How it is wired

The host lives in `crates/xcelerate/src/plugin/wasm.rs` (behind the `wasm`
cargo feature, on by default). It uses `wasmtime::component::bindgen!` on the
same WIT and a per-plugin `Store`:

```rust
wasmtime::component::bindgen!({ path: "wit/plugin.wit", world: "plugin-world" });

// load (one Store per plugin = one sandbox and one grant set)
let component = Component::from_file(&engine, entrypoint)?;
let mut linker = Linker::new(&engine);
wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;                       // sandboxed WASI
PluginWorld::add_to_linker::<HostState, HasSelf<HostState>>(&mut linker, |s| s)?;
let mut store = Store::new(&engine, HostState { granted, .. });
let bindings = PluginWorld::instantiate(&mut store, &component, &linker)?;

let describe = bindings.xcelerate_plugin_plugin().call_describe(&mut store)?;  // MessagePack
let result = bindings.xcelerate_plugin_plugin().call_invoke(&mut store, op, &args)?;
```

Everything downstream - the `Plugin` trait, capability grants, audit, and
`plugin(name).invoke(...)` in every language - is unchanged.

## Dependencies between plugins

A manifest may declare `dependencies` (name → version range) and a guest may
`use` another plugin's WIT package (see the [provider/consumer
example](examples/notes/README.md)). Because **each plugin has its own store**,
a provider's functions are store-bound: a consumer cannot import them directly.
Cross-plugin calls therefore go through the **host** rather than a direct
component link. Wiring that host-mediated bridge is the next step; until then a
plugin with `import <other>/...` will fail to instantiate.
