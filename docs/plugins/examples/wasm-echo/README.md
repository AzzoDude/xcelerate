# example.wasm-echo

A minimal xcelerate plugin compiled to **WebAssembly**, implementing the typed
`plugin` interface in [`../../../crates/xcelerate/wit/plugin.wit`](../../../crates/xcelerate/wit/plugin.wit).

Unlike the `rpc/1` subprocess plugins, a wasm plugin is a single portable
artifact, runs sandboxed (no ambient filesystem/network), and passes **MessagePack**
payloads instead of JSON.

## Build

Rust's `wasm32-wasip2` target emits a Component Model component directly, so no
adapter is needed:

```bash
rustup target add wasm32-wasip2
cargo build --release --target wasm32-wasip2
cp target/wasm32-wasip2/release/xcelerate_plugin_wasm_echo.wasm wasm-echo.wasm
```

`plugin.json` points `entrypoint` at the resulting `wasm-echo.wasm`. (If you build
for `wasm32-wasip1` instead, you must componentize it yourself and supply the WASI
preview1 adapter: `wasm-tools component new module.wasm --adapt
wasi_snapshot_preview1=<adapter>.wasm -o wasm-echo.wasm`.)

## Run

Once the wasm transport is wired (see [`../../WASM.md`](../../WASM.md)):

```rust
// abi: "wasm32-wasip2/1" selects the wasmtime transport.
browser.load_plugin("docs/plugins/examples/wasm-echo".to_string())?;
let handle = browser.plugin("example.wasm-echo".to_string())?;
// args/result are MessagePack; the bridge converts to/from the language's types.
let out = handle.invoke("echo".into(), rmp_bytes).await?;
```

## What it demonstrates

- A **typed** WIT interface (`describe`, `invoke`) with **no JSON** on the wire.
- A **host callback** (`host.log`) - the always-allowed, audited one.
- The **capability model** carried over unchanged: a `get-cookies` call from the
  plugin would be refused unless the user granted `read_cookies`.

## Status

The guest source and the WIT are the interface contract. The `wasmtime` host
transport is opt-in (`--features wasm`) and requires the toolchain above; it is
not enabled by default.
