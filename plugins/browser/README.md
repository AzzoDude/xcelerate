# browser

The **browser control surface as a plugin**. It owns the verb *logic* for the
browser - target classification (selector vs. visible text vs. snapshot index),
JavaScript construction, multi-step sequences, and the result strings - while
the host owns the *protocol*. The host holds the page and speaks CDP/BiDi,
exposing only low-level **primitives** through `host.browser` (`goto`,
`click-selector`, `evaluate`, `cdp`, …); this plugin composes them. The guest
never sees a protocol message or a socket.

| | |
| --- | --- |
| Name | `browser` |
| Capability | `browser` (standard library: trusted by default) |
| Bridge | `host.browser(op, args)` |
| Artifact | `browser.wasm` |

## Input contract

The interpreter relays a verb by name with its **resolved positional
arguments** and the interpreter's `--allow-unsafe` decision:

```json
{ "args": ["#submit"], "allow_eval": false }
```

The plugin answers with a single string: the verb's result (a value for reading
verbs, a one-line message for acting verbs).

## Verbs

| verb | positional args | notes |
| --- | --- | --- |
| `open` / `goto` | `<url>` | navigate (the interpreter normalizes the URL) |
| `title` `url` `text` `markdown`/`md` `content`/`html` | – | read the page |
| `hover` | `<selector\|text>` | |
| `mouse` | `[click\|move] <x> <y> \| <index> \| <selector> \| <text>` | |
| `scroll` | `[<pixels>\|up\|down\|top\|bottom]` | builds the scroll script |
| `find` | `<text>` | reports the match count |
| `challenge` / `detect` | – | bot-challenge report (JSON) |
| `media` | – | media list (JSON) |
| `cookie` / `cookies` | `[get [name]\|set …\|add <json>\|delete <name>\|clear]` | via CDP |
| `geolocation` | `<lat> <lon> [accuracy] \| clear` | via CDP `Emulation.*` |
| `storage` | `<local\|session> [get [key]\|set <k> <v>\|clear]` | |
| `click` / `tap` | `<index\|selector\|text>` | the classification rule lives here |
| `fill` | `<index\|selector> <text>` | |
| `select` | `<selector> <value>` | native `<select>` |
| `type` | `<text>` | into the focused element |
| `press` / `submit` / `send` | `[<key>]` | `submit`/`send` press Enter |
| `wait` | `<selector>` | durations are handled by the interpreter |
| `wait-idle`/`idle` `wait-stable`/`stable` | – | network idle / DOM stable |
| `back` `reload` | – | history |
| `eval` / `js` | `<js>` | gated by `allow_eval` |
| `dialog` | `[dismiss\|accept]` | |
| `drag` | `<from> <to>` | |
| `auth` | `<username> <password>` | HTTP auth |

The `wait-ms`/`wait-sec`/`wait-min`/`wait-hr`/`wait-random`/`await` sleeps, the
file verbs (`shot`, `download`, `capture`, `upload`, `storage save|restore`,
`route har`), and the session verbs (`tabs`, `new-tab`, `switch`, `close-tab`,
`permissions`) stay in the interpreter: they are time, filesystem, or
transport/session concerns, not page verbs.

## Build

The CLI owns the build. There is **no local build script**: run it in this
directory and it writes `wit/plugin.wit` (the canonical host ABI, so you never
hand-write WIT) and stages `browser.wasm` beside `plugin.json`.

```bash
xcelerate build --wasm-only
```

## Install and use

Copy this directory into the shared plugin home so any project or script can
load it by name:

```bash
mkdir -p ~/.xcl/plugins
cp -r . ~/.xcl/plugins/browser        # -> ~/.xcl/plugins/browser/{plugin.json,browser.wasm}
```

The standard plugins are auto-loaded and **trusted by default**, so a script
uses the plain verbs directly with no ceremony:

```bash
xcelerate run job.xcl
```

```xcl
# job.xcl
import browser
open "https://example.com"
find "Example"
```

A verb can also be invoked directly with the positional contract:

```xcl
run browser open {"args":["https://example.com"],"allow_eval":false}
```

## Layout

```
browser/
  Cargo.toml      # cdylib + wit-bindgen + xcelerate-plugin + serde_json
  plugin.json     # name, ops, capabilities (browser), limits
  src/lib.rs      # the hand-written Guest dispatcher (the verb logic)
  wit/plugin.wit  # the interface contract (written by `xcelerate build`)
```
