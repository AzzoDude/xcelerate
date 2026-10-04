# example: `acme.kv` (provider)

A shared, sandboxed **key/value store** plugin. It is the *provider* half of the
[dependency example](../notes/README.md): a second plugin, `acme.notes`, imports
the typed `store` interface this plugin exports and depends on it.

## Why it exists

A sandboxed plugin has no filesystem, so two plugins that need to share state
cannot do it through a file. Instead one plugin *provides an interface* and the
other *imports* it; the host wires the two components together. This example
shows exactly that, with no JSON and no capability inheritance.

## The two interfaces it exports

```wit
world kv-plugin {
    import xcelerate:plugin/host@1.0.0;   // standard host callbacks (log, cookies)
    export xcelerate:plugin/plugin@1.0.0; // standard surface: describe(), invoke()
    export store;                         // typed API other plugins depend on
}
```

- `xcelerate:plugin/plugin` is how the **host** loads and drives any plugin.
- `acme:kv/store` is the **typed** surface (`set`/`get`/`delete`/`keys`) that
  other plugins import. Because it is declared in WIT, a consumer gets real
  types and a compile-time-checked call - not a string op name plus a blob.

## Layout

```
kv-store/
  plugin.json            # name acme.kv, exports store
  wit/
    kv.wit               # package acme:kv  (interface store + world kv-plugin)
    deps/
      plugin.wit         # the shared xcelerate:plugin package (host + plugin)
  src/lib.rs             # implements both exported interfaces over a BTreeMap
```

`wit/deps/` holds the build-time type dependencies. `wit-bindgen` resolves them,
so the compiled component embeds the full types - the dependency is *not* looked
up at runtime.

## Build

```bash
rustup target add wasm32-wasip2
cargo build --release --target wasm32-wasip2
cp target/wasm32-wasip2/release/xcelerate_plugin_kv_store.wasm kv.wasm
```

Inspect what it exposes:

```bash
wasm-tools component wit kv.wasm
# exports: xcelerate:plugin/plugin, acme:kv/store
```

## Driving it

Through the standard surface (MessagePack payloads):

| op | args | result |
| --- | --- | --- |
| `set` | `[key, value]` | `()` |
| `get` | `key` | `option<string>` |
| `del` | `key` | `bool` |
| `keys` | `()` | `list<string>` |

Other plugins skip those string op names and call the typed `acme:kv/store`
functions directly - see [`../notes`](../notes/README.md).
