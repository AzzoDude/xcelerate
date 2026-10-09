# core

The **host primitives as a plugin** - the actions that belong to neither the
browser nor a native app. It implements the `plugin` interface (`wit/plugin.wit`)
and forwards every op to the host's semantic bridge, `host.core` - never raw
stdout/socket/file access.

| | |
| --- | --- |
| Name | `core` |
| Capability | `core` (dangerous: off by default, grant-audited) |
| Bridge | `host.core(op, args)` |
| Artifact | `core.wasm` |

## Ops

| op | args | host verb | meaning |
| --- | --- | --- | --- |
| `print` | `{ "message": "hello" }` | `print` | Write a value to the run's output. |
| `now` | `{}` | `now` | The current unix time, in seconds. |
| `env` | `{ "name": "HOME" }` | `env` | Read an environment variable. |

HTTP (`request`) and the filesystem (`read`/`write`) are the next ops; they need a
host HTTP/filesystem client behind the same `host.core` bridge.

## Writing the plugin

The whole op table is one `plugin!` declaration from
[`xcelerate-plugin`](../../crates/xcelerate-plugin). Each op forwards its
arguments to the same-named host verb:

```rust
use xcelerate_plugin::plugin;

plugin! {
    guest = Core,
    name = "core",
    bridge = core,
    ops = { print, now, env },
}
```

`op => "verb"` renames the host verb when it differs from the op name (as the
browser plugin does for `open => "goto"`).

## Build

The CLI owns the build. There is **no local build script**: run it in this
directory and it writes `wit/plugin.wit` (the canonical host ABI, so you never
hand-write WIT) and stages `core.wasm` beside `plugin.json`.

```bash
xcelerate build --wasm-only
```

## Install and use

```bash
mkdir -p ~/.xcl/plugins
cp -r . ~/.xcl/plugins/core        # -> ~/.xcl/plugins/core/{plugin.json,core.wasm}
```

The `core` capability is **dangerous**, so it must be granted explicitly with
`XCELERATE_PLUGIN_ALLOW`; the XCL `run` verb is *also* gated by `--allow-plugin`:

```bash
XCELERATE_PLUGIN_ALLOW=core xcelerate --plugins core run --allow-plugin core job.xcl
```

```xcl
# job.xcl
run core print {"message":"starting"}
run core now
run core env {"name":"HOME"}
```

## Layout

```
core/
  Cargo.toml      # cdylib + wit-bindgen + xcelerate-plugin
  plugin.json     # name, ops, capabilities (core), limits
  src/lib.rs      # the `plugin!` op table
  wit/plugin.wit  # the interface contract (written by `xcelerate build`)
```
