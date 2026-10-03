# Third-party plugins

This guide describes how an **untrusted** third-party plugin is authored,
declared, and (eventually) executed by xcelerate. It is the counterpart to the
[first-party plugin system](../../README.md#plugins): first-party plugins are
compiled into the core in Rust, while third-party plugins are authored by anyone
and run **out-of-process under an OS sandbox behind a default-deny capability
proxy**.

> **Status.** The manifest format, the ABI contract, and the host-side validation
> below are **implemented and enforced today** (`Manifest::from_json`,
> `Manifest::load`, `Manifest::validate`). The sandboxed **runner** that would
> execute a validated plugin is **not implemented yet**, so `Browser::load_plugin`
> deliberately refuses every third-party plugin. A plugin that validates will not
> run until the runner ships. This is intentional: xcelerate never executes
> untrusted code just because a manifest looks valid.

## What a plugin looks like

A third-party plugin is a self-contained directory:

```
example.echo/
  plugin.json          # the manifest (validated by the host)
  example-echo.wasm    # the sandboxed program (the `entrypoint`)
  README.md
```

The host only ever reads `plugin.json` first. If the manifest does not validate -
wrong tier, a reserved name, a first-party-only capability, limits over the host
maxima - the plugin is rejected before a single byte of plugin code is loaded.

## The manifest (`plugin.json`)

| Field | Required | Meaning |
| --- | --- | --- |
| `name` | yes | Unique id, `[A-Za-z0-9._-]`. May **not** shadow a first-party name (e.g. `stealth`). |
| `version` | yes | Plugin version (semver recommended). |
| `tier` | yes | `"third-party"` for plugins authored outside xcelerate. |
| `host_api` | yes | Host interface range the plugin targets, e.g. `">=1.0 <2.0"`. |
| `entrypoint` | yes (third-party) | Path to the program, relative to the manifest. |
| `abi` | no | Sandbox ABI the plugin speaks, e.g. `"wasm32-wasi+rpc/1"`. |
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
  "tier": "third-party",
  "host_api": ">=1.0 <2.0",
  "entrypoint": "example-echo.wasm",
  "abi": "wasm32-wasi+rpc/1",
  "ops": ["echo"],
  "capabilities": ["query", "get_text"],
  "limits": { "max_invoke_millis": 5000, "max_response_bytes": 65536 }
}
```

## Trust model

| Guarantee | How |
| --- | --- |
| No untrusted code in-process | Plugins run out-of-process in a fresh OS sandbox. |
| No ambient authority | Default-deny: a capability does nothing unless granted. |
| No privileged primitives | `LaunchControl`, `BinaryPatch`, `DetachedSpawn` are **first-party only** and are rejected at manifest validation. |
| No silent data exfiltration | The sandbox has no direct filesystem/network access; the only way out is a granted capability call, which is audited. |
| Bounded resources | Per-invocation time and response-size budgets, clamped to host maxima. |
| Reproducible forensics | Every capability grant, `on_page_created`, and `invoke` is appended to a hash-chained audit log. Cookies/credentials are never logged. |

## Capabilities

Capabilities are grouped by risk. The host grants only what the user consents to.

| Group | Capabilities |
| --- | --- |
| Safe (allow-list per origin/context) | `navigate`, `query`, `click`, `fill`, `type_keys`, `wait_for`, `wait_for_navigation`, `get_text`, `get_attribute` |
| Dangerous (explicit consent + audit) | `evaluate`, `cdp_proxy`, `read_cookies`, `write_cookies`, `init_script`, `screenshot`, `network_capture` |
| First-party only (never grantable) | `launch_control`, `binary_patch`, `detached_spawn` |

`evaluate` and `cdp_proxy` give a plugin the same power as running arbitrary
script in the page, so they are the highest-scrutiny grants. If a plugin does not
need them, it should not request them - least privilege is enforced at authoring
time by reviewers and at runtime by the proxy.

## Host API (ABI `wasm32-wasi+rpc/1`)

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
   first-party-only capability, uses a reserved name, or exceeds the budgets is
   rejected here.
2. The host spins up a fresh OS sandbox for the plugin and runs `entrypoint`.
3. ABI handshake (`describe`); a mismatched `abi` aborts the load.
4. `on_page_created` is delivered for each page the plugin may act on.
5. `invoke(op, args_json)` runs an op under its budget; over-budget ops are
   killed and audited.
6. On `Browser::close` the sandbox is torn down and its grants are revoked.

Third-party plugins can never influence the launch: `configure_launch` and all
launch flags remain first-party-only.

## Validating a manifest

From Rust, the same validation the host uses is available directly:

```rust
use xcelerate::plugin::Manifest;

let manifest = Manifest::from_json(include_str!("plugin.json"))?;
println!("{} {} ({:?})", manifest.name, manifest.version, manifest.tier);
```

`Manifest::load("path/to/plugin.json")?` reads and validates from disk. A
machine-readable schema is provided at
[`plugin.schema.json`](plugin.schema.json) so editors and CI can validate without
the Rust toolchain.

## Packaging checklist

- [ ] `plugin.json` validates (`Manifest::from_json`) and names do not shadow a
      first-party plugin.
- [ ] The plugin requests the **smallest** capability set it needs; no
      first-party-only capability is requested.
- [ ] The op set is documented and each op is covered by the `limits` budget.
- [ ] The `entrypoint` is deterministic and its `abi` matches a supported host ABI.
- [ ] The plugin never assumes it can read files or reach the network directly -
      all access goes through granted host callbacks.

## See also

- [First-party plugins](../../README.md#plugins) - trust tiers, audit log, the
  cross-language bridge.
- [`plugin.schema.json`](plugin.schema.json) - manifest JSON Schema.
- [`examples/echo`](examples/echo/README.md) - a minimal end-to-end example.
