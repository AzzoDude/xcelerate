# app

The **native application control surface as a plugin**. It implements the
`plugin` interface (`wit/plugin.wit`) and forwards every op to the host's
native-window bridge, `host.app`, which drives a window through Windows UI
Automation. The plugin never touches the OS itself.

| | |
| --- | --- |
| Name | `app` |
| Capability | `app` (dangerous: off by default, grant-audited) |
| Bridge | `host.app(op, args)` |
| Platform | Windows only (the bridge is UI Automation) |
| Artifact | `app.wasm` |

## Ops

| op | args | host verb |
| --- | --- | --- |
| `windows` | `{}` | `windows` |
| `launch` | `{ "target": "notepad", "title": "Notepad", "wait_ms": 15000 }` | `launch` |
| `tree` | `{ "window": "Notepad" }` | `tree` |
| `find` | `{ "window": "Notepad", "text": "File" }` | `find` |
| `wait` | `{ "window": "Notepad", "text": "Saved", "timeout_ms": 10000 }` | `wait` |
| `click` | `{ "window": "Notepad", "index": 3 }` or `{ "name": "Save" }` | `click` |
| `set_value` | `{ "window": "Notepad", "index": 1, "text": "hello" }` | `set-value` |
| `key` | `{ "window": "Notepad", "key": "enter" }` | `key` |
| `wheel` | `{ "window": "Notepad", "notches": 3 }` | `wheel` |
| `scroll` | `{ "window": "Notepad", "notches": -1 }` | `scroll` |

## Build

The CLI owns the build. There is **no local build script**: run it in this
directory and it writes `wit/plugin.wit` (the canonical host ABI, so you never
hand-write WIT) and stages `app.wasm` beside `plugin.json`.

```bash
xcelerate build --wasm-only
```

## Install and use

Copy this directory into the shared plugin home so any project or script can
import it:

```bash
mkdir -p ~/.xcl/plugins
cp -r . ~/.xcl/plugins/app            # -> ~/.xcl/plugins/app/{plugin.json,app.wasm}
```

The `app` capability is **dangerous**, so it must be granted explicitly with
`XCELERATE_PLUGIN_ALLOW`. A plugin run is hosted by the engine (a browser is
present), even though the `app` ops drive native windows; the bridge is the gate,
not `--allow-app`:

```bash
XCELERATE_PLUGIN_ALLOW=app xcelerate --plugins app run --allow-plugin app job.xcl
```

```xcl
# job.xcl
run app launch {"target":"notepad","title":"Notepad"}
run app tree {"window":"Notepad"}
```

## Layout

```
app/
  Cargo.toml      # cdylib + wit-bindgen + xcelerate-plugin
  plugin.json     # name, ops, capabilities (app), limits
  src/lib.rs      # the `plugin!` op table
  wit/plugin.wit  # the interface contract (written by `xcelerate build`)
```
