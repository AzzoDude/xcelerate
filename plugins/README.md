# Plugins

Three ready-made xcelerate plugins: **core** (the language's standard library),
**browser**, and **desktop** (native-window control). They are the decoupled home
for std/browser/desktop behavior: the interpreter understands only the language,
and each plugin owns its verbs while the host keeps the protocol.

| Plugin | Name | Ops | Capabilities |
| --- | --- | --- | --- |
| [`core/`](core) | `core` | `print`, `now`, `env`, `sleep`, `await`, `wait-ms`/`wait-sec`/`wait-min`/`wait-hr`, `wait-random` | `core` |
| [`browser/`](browser) | `browser` | `open`, `title`, `url`, `text`, `html`, `markdown`, `snapshot`, `click`, `hover`, `fill`, `press`, `scroll`, `wait`, `find`, `evaluate`, `screenshot`, … | `browser` |
| [`desktop/`](desktop) | `desktop` | `windows`, `launch`, `tree`, `find`, `wait`, `click`, `set_value`, `close`, `key`, `wheel`, `scroll` | `desktop`, `core` |

`core` is the language's `std`; `browser` and `desktop` are the drivers. None
speaks CDP, BiDi, or the OS directly: each calls a **capability-gated host
bridge** (`host.browser` / `host.desktop` / `host.core`) that exposes only raw
primitives (`goto`, `click-selector`, `snapshot`, `stdout`, `sleep`, …). The
plugin composes those into user-facing verbs - the host owns the protocol. See
`crates/xcelerate/wit/plugin.wit`.

## Build

The CLI owns the build. There is **no local build script**: run it in a plugin
directory and `xcelerate build --wasm-only` writes `wit/plugin.wit` (the canonical
host ABI, so you never hand-write WIT) and stages the `.wasm` beside
`plugin.json`.

```bash
(cd core    && xcelerate build --wasm-only)   # -> core.wasm
(cd browser && xcelerate build --wasm-only)   # -> browser.wasm
(cd desktop && xcelerate build --wasm-only)   # -> desktop.wasm
```

## Install into the shared plugin home

Plugin *names* are resolved from the user-global home (`$XCELERATE_HOME/plugins`,
else `~/.xcl/plugins`), then `./plugins`, then `.`. Install once and use from
anywhere:

```bash
mkdir -p ~/.xcl/plugins
cp -r core    ~/.xcl/plugins/core
cp -r browser ~/.xcl/plugins/browser
cp -r desktop ~/.xcl/plugins/desktop
```

## Use

The `core`, `browser`, and `app` plugins are the language's **standard library**
and are **trusted by default**, so a script uses the plain verbs with no ceremony
(`XCELERATE_PLUGIN_ALLOW` is not needed for them). A **third-party** plugin is
default-deny: its dangerous capabilities need `XCELERATE_PLUGIN_ALLOW`, and the
XCL `run` verb is *also* gated by `--allow-plugin`. Run `plugins` in a script to
see the trust surface.

```bash
xcelerate run job.xcl
```

```xcl
# job.xcl
print "starting"
run browser open {"url":"https://example.com"}
run browser find {"text":"Example"}
run desktop launch {"target":"notepad","title":"Notepad"}
```

From Rust, bind the browser plugin to a live page with `PluginHandle::invoke_on`:

```rust
// `load_plugin` takes a directory (or a `plugin.json`); the bare-name search is
// a CLI convenience, so a Rust caller passes the staged path.
let dir = format!("{}/.xcl/plugins/browser", std::env::var("HOME").unwrap());
browser.load_plugin(dir)?;
let page = browser.new_page().await?;
browser
    .plugin("browser".to_string())?
    .invoke_on("open".into(), r#"{"url":"https://example.com"}"#.into(), page)
    .await?;
```

See [`docs/plugins/`](../docs/plugins/README.md) for the manifest, capability,
and packaging reference, and `crates/xcelerate/wit/plugin.wit` for the ABI.
