# core

The language's **standard library** (`std`), as a plugin. The interpreter
understands only the language; the std verbs (`print`, `now`, `env`, the time
verbs, and later `request` + filesystem) are ordinary functions that live here
and compose the host's raw primitives through `host.core` (`stdout`, `now`,
`env`, `sleep`, `random`). The host keeps the primitive; the plugin keeps the
verb logic - the same split as `browser` and `app`.

| | |
| --- | --- |
| Name | `core` |
| Capability | `core` (standard library: trusted by default) |
| Bridge | `host.core(op, args)` |
| Artifact | `core.wasm` |

## Input contract

The interpreter relays a verb by name with its resolved positional arguments:

```json
{ "args": ["hello", "world"] }
```

The plugin answers with a single string: the verb's result.

## Ops

| op | positional args | host primitive | meaning |
| --- | --- | --- | --- |
| `print` | `<arg>...` | `stdout` | Join the args and write one line to stdout. |
| `now` | – | `now` | The current Unix time, in seconds. |
| `env` | `<name>` | `env` | Read an environment variable (empty when unset). |
| `sleep` | `<ms>` | `sleep` | Sleep for a number of milliseconds. |
| `await` | `<seconds>` | `sleep` | Sleep for a (decimal) number of seconds. |
| `wait-ms` / `wait-sec` / `wait-min` / `wait-hr` | `<n>` | `sleep` | Sleep `n` of the named unit. |
| `wait-random` | `<min> <max>` | `random` + `sleep` | Sleep a random number of milliseconds in `[min, max]`. |

`wait <ms|selector>` stays in the interpreter: a duration sleeps (here), a
selector waits on the page (the `browser` plugin). `request` (HTTP) and the
filesystem (`read`/`write`) are the next ops.

## Writing the plugin

The std verbs need composition (joining args, formatting, calling more than one
primitive), so the plugin hand-writes its `Guest` dispatch - the `plugin!` macro
covers the 1:1/`local` cases, but `await` is a Rust keyword and the time verbs
share a helper, so `core` uses an explicit `match`:

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

The standard plugins are auto-loaded and **trusted by default**, so a script uses
the plain verbs with no ceremony - `print` just works:

```bash
xcelerate run job.xcl
```

```xcl
# job.xcl
print "starting"
let user $env-user
```

A verb can also be invoked directly with the positional contract:

```xcl
run core print {"args":["starting"]}
```

## Layout

```
core/
  Cargo.toml      # cdylib + wit-bindgen + xcelerate-plugin
  plugin.json     # name, ops, capabilities (core), limits
  src/lib.rs      # the std functions + the hand-written `Guest` dispatch
  wit/plugin.wit  # the interface contract (written by `xcelerate build`)
```
