# Writing a plugin

This guide describes how a plugin is authored, declared, and executed by
xcelerate. It is the counterpart to the
[built-in plugin system](../../README.md#plugins): built-in plugins are compiled
into the core in Rust, while a loaded plugin is an external **WebAssembly**
component that runs sandboxed behind a default-deny capability gate.

> **Status.** The manifest format, its validation, and the sandboxed
> **WebAssembly (Component Model) runner** are implemented today.
> `Browser::load_plugin(path)` validates a `plugin.json`, instantiates the
> `entrypoint` as a wasm component with `wasmtime`, runs a `describe` handshake,
> and forwards `invoke` calls. Dangerous capabilities stay **denied by default**
> and require the host to opt in (see below). A component has no ambient
> filesystem or network; it reaches the outside world only through granted,
> audited host callbacks.

## Two ways to get a plugin

| Model | How you add one | Runs today? | Trust |
| --- | --- | --- | --- |
| **Library (compile-time)** | `cargo add` a plugin crate, then install it with `Browser::install_plugins`. | ✅ | Trusted, in-process |
| **Install (runtime)** | A `plugin.json` + `entrypoint`, loaded with `Browser::load_plugin`. | ✅ out-of-process | Capability-gated |

### Adding a plugin as a library (works today)

A plugin is just a crate that implements `Plugin`. Add it as a dependency and
install it **before** creating pages, so its `on_page_created` hook sees them:

```rust
use xcelerate::{Browser, BrowserConfig};

let browser = Browser::launch(BrowserConfig::default()).await?;
browser.install_plugins([my_plugin::MyPlugin])?;   // pass by value, no Arc

let page = browser.new_page("https://example.com".to_string()).await?;
let handle = browser.plugin("my.plugin".to_string())?;
handle.invoke("ping".into(), "{}".into()).await?;
```

An installed plugin is in-process and therefore fully trusted: it may use the
whole `PageHost` interface. `install_plugins`
refuses any name the host catalog already owns, so a library plugin can never
shadow a built-in one. It is a **Rust-only** API - the language bindings
cannot pass an executable Rust value, so a plugin that must reach Python, .NET,
or the other bindings has to be compiled into the shipped `xcelerate` core and
enabled by name.

### Loading a plugin from disk (runtime)

`Browser::load_plugin(path)` takes a plugin **directory** (containing
`plugin.json`) or a `plugin.json` file. It:

1. reads and validates the manifest (reserved names, host-only
   capabilities, and budgets are rejected here);
2. instantiates the `entrypoint` - a WebAssembly **component** - in its own
   sandboxed `wasmtime` store, and runs the `describe` handshake;
3. registers the plugin's ops, which are then callable through
   `plugin(name).invoke(op, args_json)` in **every language** - no binding code
   and no rebuild of the core.

```rust
let browser = Browser::launch(BrowserConfig::default()).await?;
browser.load_plugin("docs/plugins/examples/wasm-echo".to_string())?;
let handle = browser.plugin("example.wasm-echo".to_string())?;
let result = handle.invoke("echo".into(), r#"{"hello":"world"}"#.into()).await?;
```

#### The component contract

A plugin is a `.wasm` component built against
[`crates/xcelerate/wit/plugin.wit`](../../crates/xcelerate/wit/plugin.wit). It
**imports** the capability-gated `host` interface and **exports** the `plugin`
interface (`describe`, `invoke`). Op arguments and results are MessagePack
payloads, so calls are binary and typed rather than JSON strings. See
[`WASM.md`](WASM.md) for the contract and how to build a guest.

The host exposes only what a plugin is granted:

| Host callback | Capability |
| --- | --- |
| `log(message)` | always (audited, capped) |
| `get-cookies()` | `read_cookies` |
| `set-cookie(cookie)` | `write_cookies` |

#### Granting dangerous capabilities

Dangerous callbacks are **denied by default**. The host opts in with the
`XCELERATE_PLUGIN_ALLOW` environment variable (comma-separated). Prefer the
per-plugin form so a grant cannot leak to another plugin:

```bash
# only example.wasm-echo may read cookies
XCELERATE_PLUGIN_ALLOW=example.wasm-echo:read_cookies <host command>
# a bare capability name applies to every loaded plugin (use with care)
XCELERATE_PLUGIN_ALLOW=read_cookies <host command>
# the browser/app plugins grant their bridge the same way
XCELERATE_PLUGIN_ALLOW=browser <host command>
```

Granting a callback is audited; refusing one is audited too.

#### Hardening and residual risk

Each component gets its **own store** with a **default WASI context** (no
preopened files, no environment, no sockets), so it has **no ambient filesystem
or network** - the host callbacks are the only way out, and each is
capability-gated and audited. The `entrypoint` must resolve **inside the plugin
directory**, and its self-described name must match the manifest.

Because the sandbox is the WebAssembly runtime itself, a plugin can only touch
what its granted host imports allow. (Cookie host callbacks are declared but not
implemented yet; calling them returns an error.)

## Installing per language

Every binding can use both kinds of plugin: a **compiled-in** plugin
(built-in, or one baked into the core) enabled by name, and a **loaded**
plugin run sandboxed (WebAssembly) from disk with `load_plugin(path)`.

### Rust - install a plugin library

```toml
# Cargo.toml
[dependencies]
xcelerate-plugin-foo = "0.1"
```

```rust
let browser = Browser::launch(BrowserConfig::default()).await?;
browser.install_plugins([xcelerate_plugin_foo::FooPlugin])?;
```

### Any binding - load a plugin from disk

`load_plugin` is exported to every language. Point it at a plugin directory (or a
`plugin.json`); the plugin's ops are then callable through the same JSON bridge:

| Language | Load | Invoke |
| --- | --- | --- |
| Python | `browser.load_plugin("path/to/plugin")` | `await browser.plugin("example.wasm-echo").invoke("echo", "{}")` |
| JavaScript | `browser.loadPlugin("path/to/plugin")` | `await browser.plugin("example.wasm-echo").invoke("echo", "{}")` |
| .NET / C# | `browser.LoadPlugin("path/to/plugin")` | `await browser.Plugin("example.wasm-echo").Invoke("echo", "{}")` |
| Kotlin | `browser.loadPlugin("path/to/plugin")` | `browser.plugin("example.wasm-echo").invoke("echo", "{}")` |
| Java | `browser.loadPlugin("path/to/plugin")` | `browser.plugin("example.wasm-echo").invoke("echo", "{}").get()` |
| Swift | `try browser.loadPlugin(path: "path/to/plugin")` | `try await browser.plugin(name: "example.wasm-echo").invoke(op: "echo", argsJson: "{}")` |
| Ruby | `browser.load_plugin("path/to/plugin")` | `browser.plugin("example.wasm-echo").invoke("echo", "{}")` |
| Dart | `browser.loadPlugin("path/to/plugin")` | `await browser.plugin("example.wasm-echo").invoke("echo", "{}")` |
| Go | `browser.LoadPlugin("path/to/plugin")` | `browser.Plugin("example.wasm-echo").Invoke("echo", "{}")` |
| PowerShell | `$browser.LoadPlugin('path/to/plugin')` | `(Get-XceleratePlugin -Browser $browser -Name example.wasm-echo).Invoke('echo','{}')` |
| MCP | tool `browser_load_plugin {path}` | tool `browser_plugin_invoke {name, op, args}` |

Dangerous capabilities the plugin requests still need the host to opt in
(`XCELERATE_PLUGIN_ALLOW`), in every language.

### Any binding - drive a loaded plugin by name

Plugins are **not** built into the core. Once a plugin has been loaded from disk,
you can reach it by name through the same bridge (`use_plugin` is idempotent for
an already-loaded plugin):

| Language | Handle | Invoke (JSON in, JSON out) |
| --- | --- | --- |
| Python | `browser.plugin("example.wasm-echo")` | `await handle.invoke("echo", "{}")` |
| JavaScript | `browser.plugin("example.wasm-echo")` | `await handle.invoke("echo", "{}")` |
| .NET / C# | `browser.Plugin("example.wasm-echo")` | `await handle.Invoke("echo", "{}")` |
| Kotlin | `browser.plugin("example.wasm-echo")` | `handle.invoke("echo", "{}")` |
| Java | `browser.plugin("example.wasm-echo")` | `handle.invoke("echo", "{}").get()` |
| Swift | `try browser.plugin(name: "example.wasm-echo")` | `try await handle.invoke(op: "echo", argsJson: "{}")` |
| Ruby | `browser.plugin("example.wasm-echo")` | `handle.invoke("echo", "{}")` |
| Dart | `browser.plugin("example.wasm-echo")` | `await handle.invoke("echo", "{}")` |
| Go | `browser.Plugin("example.wasm-echo")` | `handle.Invoke("echo", "{}")` |
| PowerShell | `Get-XceleratePlugin -Browser $browser -Name example.wasm-echo` | `$handle.Invoke('echo', '{}')` |
| CLI | `xcelerate --plugins path/to/plugin <cmd>` | - |
| MCP | env `XCELERATE_PLUGINS=path/to/plugin` | `browser_plugin_invoke` |

At launch you can instead list plugin paths on the browser config - for example
`BrowserConfig(plugins=["path/to/plugin"])` in Python or
`{ plugins: ["path/to/plugin"] }` in JavaScript (see each binding's README). The
list is the same default-deny allow-list in every language.

### Adding a plugin to your program

Build the plugin to a `.wasm` component and `load_plugin` it (or list its path in
`plugins`). From Rust you can alternatively add the plugin crate and
`install_plugins([MyPlugin])` to run it in-process.

## What a plugin looks like

A plugin is a self-contained directory:

```
example.wasm-echo/
  plugin.json          # the manifest (validated by the host)
  wasm-echo.wasm       # the component (built from Rust/C/Go/... via WIT)
  README.md
```

The host only ever reads `plugin.json` first. If the manifest does not validate -
a reserved name, a host-only capability, limits over the host maxima - the plugin
is rejected before a single byte of plugin code is loaded.

## The manifest (`plugin.json`)

| Field | Required | Meaning |
| --- | --- | --- |
| `name` | yes | Unique id, `[A-Za-z0-9._-]`. May **not** shadow a reserved name. |
| `version` | yes | Plugin version (semver recommended). |
| `host_api` | yes | Host interface range the plugin targets, e.g. `">=1.0 <2.0"`. |
| `entrypoint` | yes | Path to the program, relative to the manifest. |
| `abi` | no | Sandbox ABI the plugin speaks, e.g. `"wasm32-wasip2/1"`. |
| `ops` | yes (≥1) | Operations exposed through `PluginHandle::invoke`. |
| `capabilities` | no | Capabilities requested. Default-deny; audited when granted. |
| `limits` | no | `{ "max_invoke_millis", "max_response_bytes" }`, clamped to host maxima. |

Unknown fields are **rejected** (`deny_unknown_fields`), so a typo fails closed
instead of being silently ignored.

A complete example lives in
[`examples/wasm-echo/plugin.json`](examples/wasm-echo/plugin.json):

```json
{
  "name": "example.wasm-echo",
  "version": "0.1.0",
  "host_api": ">=1.0 <2.0",
  "entrypoint": "wasm-echo.wasm",
  "abi": "wasm32-wasip2/1",
  "ops": ["echo", "log"],
  "capabilities": [],
  "limits": { "max_invoke_millis": 5000, "max_response_bytes": 65536 }
}
```

## Dependencies

A plugin may depend on other plugins. Declare it in the manifest:

```jsonc
"dependencies": { "acme.kv": "^1.0" }   // plugin name -> version range
```

The loader resolves dependencies first (one version per name, no cycles) and
wires the dependent plugin's imports to the provider's exports at instantiation.
Each plugin stays **independently sandboxed and granted** - there is no
capability inheritance.

In WIT, the dependency is a normal cross-package import. The provider exports an
interface; the consumer imports it; the host backs the import with the provider:

```wit
// provider: export the typed surface
world kv-plugin { export store; }

// consumer: import it (no implementation here)
world notes-plugin { import acme:kv/store@1.0.0; }
```

Because `wit-bindgen` resolves the provider's package at **build time**, the
call is fully typed - the consumer depends on it before it ever runs.

See the complete, runnable pair:
[`examples/kv-store`](examples/kv-store/README.md) (provider) and
[`examples/notes`](examples/notes/README.md) (consumer).

## Trust model

| Guarantee | How |
| --- | --- |
| No unknown code in-process | Each plugin runs in its own sandboxed WebAssembly store. |
| No ambient authority | Default-deny: a capability does nothing unless granted. |
| No privileged primitives | `LaunchControl`, `BinaryPatch`, `DetachedSpawn` are **built-in only** and are rejected at manifest validation. |
| No silent data exfiltration | The sandbox has no direct filesystem/network access; the only way out is a granted capability call, which is audited. |
| Bounded resources | Per-invocation time and response-size budgets, clamped to host maxima. |
| Reproducible forensics | Every capability grant, `on_page_created`, and `invoke` is appended to a hash-chained audit log. Cookies/credentials are never logged. |

## Capabilities

Capabilities are grouped by risk. The host grants only what the user consents to.

| Group | Capabilities |
| --- | --- |
| Safe (allow-list per origin/context) | `navigate`, `query`, `click`, `fill`, `type_keys`, `wait_for`, `wait_for_navigation`, `get_text`, `get_attribute` |
| Dangerous (explicit consent + audit) | `evaluate`, `cdp_proxy`, `read_cookies`, `write_cookies`, `init_script`, `screenshot`, `network_capture`, `browser`, `app`, `core`, `invoke_plugin` |
| Built-in only (never grantable) | `launch_control`, `binary_patch`, `detached_spawn` |

`evaluate` and `cdp_proxy` give a plugin the same power as running arbitrary
script in the page, so they are the highest-scrutiny grants. If a plugin does not
need them, it should not request them - least privilege is enforced at authoring
time by reviewers and at runtime by the proxy.

## The component contract

The host is the only caller. It instantiates the component and drives the
exported `plugin` interface:

| Call | When |
| --- | --- |
| `describe() -> payload` | Handshake; returns `{name, version, ops}` (MessagePack). |
| `invoke(op, args) -> result<payload, string>` | `PluginHandle::invoke`. |

The component calls back through the imported `host` interface (each checks a
granted capability first):

| Callback | Capability required |
| --- | --- |
| `log(message)` | always (redacted, never logs secrets) |
| `get-cookies()` | `read_cookies` (dangerous) |
| `set-cookie(cookie)` | `write_cookies` (dangerous) |
| `browser(op, args)` | `browser` (dangerous): a semantic browser action on the run's active page |
| `app(op, args)` | `app` (dangerous, Windows only): a native-window action |
| `core(op, args)` | `core` (dangerous): a host primitive - stdout, time, environment (HTTP/filesystem to follow) |
| `invoke-plugin(plugin, op, args)` | `invoke_plugin` (dangerous) |

`browser`, `app`, and `core` are the **host action bridges**: a plugin names a
*verb* (`goto`, `click`, `fill`, `snapshot`, `tree`, `print`, …) and the host maps
it onto the engine, the desktop backend, or the host itself. They are deliberately
not raw CDP/BiDi, a raw OS handle, or raw socket/file access, so the host keeps
control of what a sandboxed guest may do. The host binds a page to the invocation
first - `PluginHandle::invoke_on(op, args, page)` from Rust, or the `run` verb in
XCL - and refuses `browser` when no page is bound.

`invoke-plugin` is how a plugin declares a *dependency* on another plugin's
behaviour: instead of importing the dependency's wasm interface directly, it
calls the target op through the host, which routes the call (honouring overrides
and budgets) and records it in the audit log. A plugin must still list the
dependency in its manifest `dependencies` for the host to resolve and enable it
first.

Payloads are MessagePack `list<u8>`, matching
`PluginHandle::invoke(op, args_json) -> result_json` in every language binding
(the bridge converts JSON args to MessagePack and back).

## Lifecycle

1. The host reads and validates `plugin.json`. A manifest that requests a
   host-only capability, uses a reserved name, or exceeds the budgets is
   rejected here.
2. The host spins up a fresh OS sandbox for the plugin and runs `entrypoint`.
3. ABI handshake (`describe`); a mismatched `abi` aborts the load.
4. `on_page_created` is delivered for each page the plugin may act on.
5. `invoke(op, args_json)` runs an op under its budget; over-budget ops are
   killed and audited.
6. On `Browser::close` the sandbox is torn down and its grants are revoked.

Plugins loaded from disk can never influence the launch: `configure_launch` and
all launch flags remain available only to built-in plugins.

## Validating a manifest

From Rust, the same validation the host uses is available directly:

```rust
use xcelerate::plugin::Manifest;

let manifest = Manifest::from_json(include_str!("plugin.json"))?;
println!("{} {}", manifest.name, manifest.version);
```

`Manifest::load("path/to/plugin.json")?` reads and validates from disk. A
machine-readable schema is provided at
[`plugin.schema.json`](plugin.schema.json) so editors and CI can validate without
the Rust toolchain.

## Packaging checklist

- [ ] `plugin.json` validates (`Manifest::from_json`) and names do not shadow a
      built-in plugin.
- [ ] The plugin requests the **smallest** capability set it needs; no
      host-only capability is requested.
- [ ] The op set is documented and each op is covered by the `limits` budget.
- [ ] The `entrypoint` is a wasm component built from the shared WIT
      (`xcelerate build` writes `wit/plugin.wit` for you) and its `abi` is
      `wasm32-wasip2/1`.
- [ ] The plugin never assumes it can read files or reach the network directly -
      all access goes through granted host callbacks.

## See also

- [Built-in plugins](../../README.md#plugins) - the audit log and the
  cross-language bridge.
- [`plugins/browser`](../../plugins/browser/README.md) and
  [`plugins/app`](../../plugins/app/README.md) - the ready-made browser and
  native-app plugins built on the host action bridge.
- [`plugin.schema.json`](plugin.schema.json) - manifest JSON Schema.
- [`examples/wasm-echo`](examples/wasm-echo/README.md) - a minimal end-to-end
  example (Rust guest).
- [`examples/kv-store`](examples/kv-store/README.md) and
  [`examples/notes`](examples/notes/README.md) - a provider/consumer pair showing
  a plugin that depends on another plugin.
