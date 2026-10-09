# {{name}}

A xcelerate mod, compiled to a single portable WebAssembly component.

It implements the typed `plugin` interface (`wit/plugin.wit`) and ships one op,
`echo`. Args and results are **MessagePack**, so the host and the mod stay typed
through `serde` - no JSON on the wire.

## Build

The CLI owns the build - there is no local build script. Run it in this directory
and it writes `wit/plugin.wit` (the canonical host ABI, so you never hand-write
WIT) and stages the `.wasm` next to `plugin.json`.

```bash
xcelerate build --wasm-only
```

Building by hand is possible once the WIT is present (run the CLI first):

```bash
rustup target add wasm32-wasip2
cargo build --release --target wasm32-wasip2
```

## Run

Point the engine at this directory; it reads `plugin.json`, loads
`{{entrypoint}}`, and exposes the mod's ops:

```rust
browser.load_plugin("{{title}}".to_string())?;
let handle = browser.plugin("{{name}}".to_string())?;
let out = handle.invoke("echo".into(), r#"{"message": "hi"}"#.into()).await?;
```

`{{name}}` is sandboxed by the runtime: no ambient filesystem or network, and a
host callback like `get-cookies` is refused unless the user grants the matching
capability. `log` is always allowed and written to the audit log.

## Layout

```
{{title}}/
  Cargo.toml      # cdylib + wit-bindgen
  plugin.json     # manifest: name, ops, capabilities, limits
  src/lib.rs      # your ops
  src/support.rs  # MessagePack helpers + the mod_ops! macro
  wit/plugin.wit  # the interface contract (written by `xcelerate build`)
```

See [`docs/plugins/README.md`](../../README.md) for the manifest, capability, and
packaging reference.
