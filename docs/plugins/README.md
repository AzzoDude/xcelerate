# Writing a plugin

This guide describes how a plugin is authored, declared, and executed by
xcelerate. It is the counterpart to the
[built-in plugin system](../../README.md#plugins): built-in plugins are compiled
into the core in Rust, while a loaded plugin is authored by anyone and runs
**out-of-process under a sandbox behind a default-deny capability proxy**.

> **Status.** The manifest format, its validation, and the **out-of-process
> runner** are implemented today. `Browser::load_plugin(path)` validates a
> `plugin.json`, spawns its `entrypoint` as a child process, runs a `describe`
> handshake, and forwards `invoke` calls. The plugin speaks line-delimited
> JSON-RPC (ABI `rpc/1`) on stdin/stdout, so it can be written in **any
> language**. Dangerous capabilities stay **denied by default** and require the
> host to opt in (see below). Loading from disk is always sandboxed: a plugin is
> isolated by being a separate process and by capability gating.

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

An installed plugin is in-process and therefore trusted exactly like `stealth`
and `human`: it may use the whole `PageHost` interface. `install_plugins`
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
2. spawns the `entrypoint` as a child process and runs a `describe` handshake;
3. registers the plugin's ops, which are then callable through
   `plugin(name).invoke(op, args_json)` in **every language** - no binding code
   and no rebuild of the core.

The entrypoint is any program that speaks the `rpc/1` protocol. The host infers
an interpreter from the extension (`.py` -> `python`, `.js` -> `node`, `.rb` ->
`ruby`, `.sh` -> `sh`; anything else is run directly), so a plugin can be a
script or a compiled binary.

```rust
let browser = Browser::launch(BrowserConfig::default()).await?;
browser.load_plugin("docs/plugins/examples/echo".to_string())?;
let handle = browser.plugin("example.echo".to_string())?;
let result = handle.invoke("echo".into(), r#"{"hello":"world"}"#.into()).await?;
```

#### The `rpc/1` protocol

Newline-delimited JSON, one object per line. Host to plugin:

| Call | Meaning |
| --- | --- |
| `describe` | handshake; the plugin replies `{name, abi, ops}`. |
| `invoke {op, args}` | run an op; reply with `result` or `error`. |
| `shutdown` | the host is done; exit. |

Plugin to host (each is **capability-gated** and audited):

| Callback | Capability |
| --- | --- |
| `host.log {message}` | always (redacted) |
| `host.get_cookies` | `read_cookies` |
| `host.set_cookie {cookie}` | `write_cookies` |
| `host.cdp {method, params}` | `cdp_proxy` |

#### Granting dangerous capabilities

Dangerous callbacks are **denied by default**. The host opts in with the
`XCELERATE_PLUGIN_ALLOW` environment variable (comma-separated). Prefer the
per-plugin form so a grant cannot leak to another plugin:

```bash
# only example.echo may read cookies
XCELERATE_PLUGIN_ALLOW=example.echo:read_cookies <host command>
# a bare capability name applies to every loaded plugin (use with care)
XCELERATE_PLUGIN_ALLOW=read_cookies <host command>
```

Granting a callback is audited; refusing one is audited too. `host.cdp`
requires the **narrower** capability for cookie/`evaluate` methods as well, so
`cdp_proxy` cannot be used to bypass `read_cookies` / `write_cookies` /
`evaluate`. The `docs/plugins/examples/echo` plugin demonstrates `echo` (always
allowed) and `cookies` (`read_cookies`).

#### Hardening and residual risk

The child runs with a **cleared environment** (no host tokens or keys, no
`XCELERATE_PLUGIN_ALLOW`), an **absolute interpreter path** (no `PATH` or
current-directory exec planting), a **size-capped** protocol, and per-invoke
**timeouts**. The `entrypoint` must resolve **inside the plugin directory**, and
its self-described name and ops must match the manifest.

**There is no OS sandbox yet.** A plugin still runs as the host user and can, in
principle, read the browser profile, reach the network, or connect to the
DevTools port directly - bypassing the host-mediated capabilities entirely.
Treat a loaded plugin as code you have chosen to run: do not load
plugins you do not trust, and do not run them alongside secrets. OS-level
isolation (Job Objects / seccomp / AppContainer) is the next milestone.

## Installing per language

Every binding can use both kinds of plugin: a **compiled-in** plugin
(built-in, or one baked into the core) enabled by name, and a **loaded**
plugin run out-of-process from disk with `load_plugin(path)`.

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
| Python | `browser.load_plugin("path/to/plugin")` | `await browser.plugin("example.echo").invoke("echo", "{}")` |
| JavaScript | `browser.loadPlugin("path/to/plugin")` | `await browser.plugin("example.echo").invoke("echo", "{}")` |
| .NET / C# | `browser.LoadPlugin("path/to/plugin")` | `await browser.Plugin("example.echo").Invoke("echo", "{}")` |
| Kotlin | `browser.loadPlugin("path/to/plugin")` | `browser.plugin("example.echo").invoke("echo", "{}")` |
| Java | `browser.loadPlugin("path/to/plugin")` | `browser.plugin("example.echo").invoke("echo", "{}").get()` |
| Swift | `try browser.loadPlugin(path: "path/to/plugin")` | `try await browser.plugin(name: "example.echo").invoke(op: "echo", argsJson: "{}")` |
| Ruby | `browser.load_plugin("path/to/plugin")` | `browser.plugin("example.echo").invoke("echo", "{}")` |
| Dart | `browser.loadPlugin("path/to/plugin")` | `await browser.plugin("example.echo").invoke("echo", "{}")` |
| Go | `browser.LoadPlugin("path/to/plugin")` | `browser.Plugin("example.echo").Invoke("echo", "{}")` |
| PowerShell | `$browser.LoadPlugin('path/to/plugin')` | `(Get-XceleratePlugin -Browser $browser -Name example.echo).Invoke('echo','{}')` |
| MCP | tool `browser_load_plugin {path}` | tool `browser_plugin_invoke {name, op, args}` |

Dangerous capabilities the plugin requests still need the host to opt in
(`XCELERATE_PLUGIN_ALLOW`), in every language.

### Any binding - enable a compiled-in plugin by name

For plugins compiled into the core, enable by name and drive through the same
bridge:

| Language | Enable by name | Handle | Invoke (JSON in, JSON out) |
| --- | --- | --- | --- |
| Python | `await browser.use_plugin("stealth")` | `browser.plugin("stealth")` | `await handle.invoke("info", "{}")` |
| JavaScript | `await browser.usePlugin("stealth")` | `browser.plugin("stealth")` | `await handle.invoke("info", "{}")` |
| .NET / C# | `await browser.UsePlugin("stealth")` | `browser.Plugin("stealth")` | `await handle.Invoke("info", "{}")` |
| Kotlin | `browser.usePlugin("stealth")` | `browser.plugin("stealth")` | `handle.invoke("info", "{}")` |
| Java | `browser.usePlugin("stealth")` | `browser.plugin("stealth")` | `handle.invoke("info", "{}").get()` |
| Swift | `try await browser.usePlugin(name: "stealth")` | `try browser.plugin(name: "stealth")` | `try await handle.invoke(op: "info", argsJson: "{}")` |
| Ruby | `browser.use_plugin("stealth")` | `browser.plugin("stealth")` | `handle.invoke("info", "{}")` |
| Dart | `await browser.usePlugin("stealth")` | `browser.plugin("stealth")` | `await handle.invoke("info", "{}")` |
| Go | `browser.UsePlugin("stealth")` | `browser.Plugin("stealth")` | `handle.Invoke("info", "{}")` |
| PowerShell | `$browser.UsePlugin('stealth')` | `Get-XceleratePlugin -Browser $browser -Name stealth` | `$handle.Invoke('info', '{}')` |
| CLI | `xcelerate-cli --plugins stealth <cmd>` | - | - |
| MCP | env `XCELERATE_PLUGINS=stealth` | - | `browser_plugin_invoke` |

At launch you can instead list plugins on the browser config - for example
`BrowserConfig(plugins=["stealth"])` in Python or `{ plugins: ["stealth"] }` in
JavaScript (see each binding's README). The list is the same default-deny
allow-list in every language.

### Adding a plugin to the shipped core

To make a custom plugin reachable from the non-Rust bindings, compile it into
the engine: add its crate to `crates/xcelerate/Cargo.toml`, register it in the
catalog in `crates/xcelerate/src/plugin.rs`, and rebuild the `xcelerate` cdylib.
It then appears in `available_plugins()` in every language and can be enabled by
name.

## What a plugin looks like

A plugin is a self-contained directory:

```
example.echo/
  plugin.json          # the manifest (validated by the host)
  echo.py              # the entrypoint (any program speaking `rpc/1`)
  README.md
```

The host only ever reads `plugin.json` first. If the manifest does not validate -
a reserved name, a host-only capability, limits over the host maxima - the plugin
is rejected before a single byte of plugin code is loaded.

## The manifest (`plugin.json`)

| Field | Required | Meaning |
| --- | --- | --- |
| `name` | yes | Unique id, `[A-Za-z0-9._-]`. May **not** shadow a built-in name (e.g. `stealth`). |
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
[`examples/echo/plugin.json`](examples/echo/plugin.json):

```json
{
  "name": "example.echo",
  "version": "0.1.0",
  "host_api": ">=1.0 <2.0",
  "entrypoint": "example-echo.wasm",
  "abi": "wasm32-wasip2/1",
  "ops": ["echo"],
  "capabilities": ["query", "get_text"],
  "limits": { "max_invoke_millis": 5000, "max_response_bytes": 65536 }
}
```

## Trust model

| Guarantee | How |
| --- | --- |
| No unknown code in-process | Plugins run out-of-process in a fresh sandbox. |
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
| Dangerous (explicit consent + audit) | `evaluate`, `cdp_proxy`, `read_cookies`, `write_cookies`, `init_script`, `screenshot`, `network_capture` |
| Built-in only (never grantable) | `launch_control`, `binary_patch`, `detached_spawn` |

`evaluate` and `cdp_proxy` give a plugin the same power as running arbitrary
script in the page, so they are the highest-scrutiny grants. If a plugin does not
need them, it should not request them - least privilege is enforced at authoring
time by reviewers and at runtime by the proxy.

## Host API (ABI `rpc/1`)

The host is the only caller. It drives the plugin over a versioned,
length-prefixed JSON RPC: the plugin exports the ops named in its manifest and
calls back to the host through capability-scoped functions.

Host → plugin:

| Call | When |
| --- | --- |
| `describe()` | Handshake; must match the manifest's `abi`. |
| `on_page_created(page_id)` | A new page is available. |
| `invoke(op, args_json) -> result_json` | `PluginHandle::invoke`. |

Plugin → host (each checks a granted capability first):

| Callback | Capability required |
| --- | --- |
| `host.navigate(url)` | `navigate` |
| `host.query(selector)` / `host.query_all(selector)` | `query` |
| `host.get_text(selector)` / `host.get_attribute(selector, name)` | `get_text` / `get_attribute` |
| `host.click(selector)` / `host.fill(selector, value)` / `host.type(selector, text)` | `click` / `fill` / `type_keys` |
| `host.evaluate(js)` | `evaluate` (dangerous) |
| `host.cdp(method, params_json)` | `cdp_proxy` (dangerous) |
| `host.get_cookies()` / `host.set_cookie(json)` | `read_cookies` / `write_cookies` (dangerous) |
| `host.screenshot(full_page)` | `screenshot` (dangerous) |
| `host.log(message)` | always (redacted, never logs secrets) |

`invoke` arguments and results are JSON strings, matching
`PluginHandle::invoke(op, args_json) -> result_json` in every language binding.

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
- [ ] The `entrypoint` is deterministic and its `abi` matches a supported host ABI.
- [ ] The plugin never assumes it can read files or reach the network directly -
      all access goes through granted host callbacks.

## See also

- [Built-in plugins](../../README.md#plugins) - the audit log and the
  cross-language bridge.
- [`plugin.schema.json`](plugin.schema.json) - manifest JSON Schema.
- [`examples/echo`](examples/echo/README.md) - a minimal end-to-end example.
