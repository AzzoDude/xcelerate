# example: `acme.notes` (consumer / dependent)

A **notes** plugin whose storage is another plugin. It is the *consumer* half of
the dependency example: its manifest declares `"dependencies": { "acme.kv":
"^1.0" }`, its WIT **imports** `acme:kv/store`, and every `save`/`load`/`titles`
call crosses into the [`acme.kv`](../kv-store/README.md) component through the
host.

## The dependency, in three places

```jsonc
// plugin.json  - what the loader must resolve before this plugin
"dependencies": { "acme.kv": "^1.0" }
```

```wit
// wit/notes.wit - the type/behavior dependency
world notes-plugin {
    import xcelerate:plugin/host@1.0.0;
    import acme:kv/store@1.0.0;          // implemented by acme.kv, not here
    export xcelerate:plugin/plugin@1.0.0;
    export notes;
}
```

```rust
// src/lib.rs - the actual cross-plugin call
acme::kv::store::set(&key, &body)?;      // goes through the host linker
```

Note what is **not** here: no store implementation, no file, no socket, no
capability inheritance. `acme.notes` is still its own sandbox; it just has one
more import the host must satisfy.

## The provider's interface is baked in at build time

```
notes/
  wit/
    notes.wit            # package acme:notes
    deps/
      plugin.wit         # xcelerate:plugin (host + plugin)
      kv.wit             # acme:kv  <-- the provider's package, as a build dep
```

`wit-bindgen` resolves `deps/kv.wit`, so `acme::kv::store` is a **typed** Rust
call - a wrong argument type is a compile error, not a runtime "unknown op". The
dependency is resolved when you build, not when the plugin runs.

## Layout

```
notes/
  plugin.json            # name acme.notes, dependencies: { acme.kv: ^1.0 }
  wit/notes.wit
  wit/deps/{plugin,kv}.wit
  src/lib.rs
```

## Build

```bash
rustup target add wasm32-wasip2
cargo build --release --target wasm32-wasip2
cp target/wasm32-wasip2/release/xcelerate_plugin_notes.wasm notes.wasm
```

Inspect the difference from the provider:

```bash
wasm-tools component wit notes.wasm
# imports: xcelerate:plugin/types, acme:kv/store, wasi:*   <-- the dependency
# exports: xcelerate:plugin/plugin, acme:notes/notes
```

`import acme:kv/store` with no local implementation is the dependency, made
explicit and checkable before the plugin ever runs.

## How the host resolves it

1. Read `plugin.json`, see `dependencies`.
2. Load the dependencies first (`acme.kv`), each independently sandboxed and
   capability-granted.
3. Instantiate `acme.notes` with a linker whose `acme:kv/store` import points at
   the **provider's exported** `acme:kv/store`.

Two components, two sandboxes, one typed call between them.

## Driving it

Through the standard surface (MessagePack payloads):

| op | args | result |
| --- | --- | --- |
| `save` | `[title, body]` | key (string) |
| `load` | `title` | `option<string>` |
| `titles` | `()` | `list<string>` |

The `notes` interface is also exported typed, for other plugins to depend on.
