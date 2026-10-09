# desktop

The **native-window control surface as a plugin**. It owns the verb *logic*; the
host owns the *protocol*: UI Automation stays in the host and the plugin only
asks for a snapshot or an act through `host.desktop`. The guest never touches the
OS.

| | |
| --- | --- |
| Name | `desktop` |
| Capability | `desktop` (+ `core`) - standard library: trusted by default |
| Bridge | `host.desktop(op, args)` |
| Platform | Windows only (the bridge is UI Automation) |
| Artifact | `desktop.wasm` |

## Ops

Forwarded 1:1 to a host primitive:

| op | host primitive |
| --- | --- |
| `windows` | `windows` |
| `launch` | `launch` |
| `set_value` | `set-value` |
| `close` | `close` |
| `key` | `key` |
| `wheel` | `wheel` |
| `scroll` | `scroll` |

Composite (logic in this crate, over `snapshot`/`click-index`/`sleep`):

| op | args | meaning |
| --- | --- | --- |
| `tree` | `{ "window": "Notepad" }` | Render every element as a line. |
| `find` | `{ "window": "Notepad", "text": "File" }` | The matching element lines. |
| `wait` | `{ "window": "Notepad", "text": "Saved", "timeout_ms": 10000 }` | Poll until the text appears. |
| `click` | `{ "window": "Notepad", "index": 3 }` or `{ "name": "Save" }` | Click by index or by name. |

The sleep used by `wait` is the std `core` timer (`host.core("sleep")`), which is
why the manifest also requests the `core` capability.

## Build

The CLI owns the build. There is **no local build script**: run it in this
directory and it writes `wit/plugin.wit` (the canonical host ABI, so you never
hand-write WIT) and stages `desktop.wasm` beside `plugin.json`.

```bash
xcelerate build --wasm-only
```

## Install and use

```bash
mkdir -p ~/.xcl/plugins
cp -r . ~/.xcl/plugins/desktop      # -> ~/.xcl/plugins/desktop/{plugin.json,desktop.wasm}
```

The standard plugins are auto-loaded and **trusted by default**. `--allow-app`
still gates which windows a run may drive:

```bash
xcelerate run --allow-app "Notepad*" job.xcl
```

```xcl
# job.xcl - the driver is the desktop; the plugin is the library behind the verbs
launch "notepad" "Notepad"
find "Text editor"
click "Text editor"
```

## Layout

```
desktop/
  Cargo.toml      # cdylib + wit-bindgen + xcelerate-plugin
  plugin.json     # name, ops, capabilities (desktop, core), limits
  src/lib.rs      # the forwarded + composite op table
  wit/plugin.wit  # the interface contract (written by `xcelerate build`)
```
