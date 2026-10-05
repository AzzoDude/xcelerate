# bidi-protocol

Generated Rust types for the [W3C WebDriver BiDi](https://w3c.github.io/webdriver-bidi/)
protocol — the backend that lets Xcelerate drive **Firefox** (and every other
browser that implements BiDi) the way `browser-protocol` / `js-protocol` let it
drive Chromium over CDP.

## Why this crate exists

Xcelerate's core speaks CDP. Firefox does **not** speak CDP (Mozilla removed it:
`remote.active-protocols` was dropped in Firefox 141 and WebDriver BiDi is the
only remaining remote protocol). BiDi's command envelope is `{id, method, params}`
— the same shape as CDP — so a sibling typed protocol crate keeps the Firefox
backend as idiomatic as the Chromium one.

## How it is harvested (the important part)

`browser-protocol` and `js-protocol` are generated from two machine-readable JSON
files that Google publishes:

* `https://raw.githubusercontent.com/ChromeDevTools/devtools-protocol/master/json/browser_protocol.json`
* `.../js_protocol.json`

WebDriver BiDi has **no equivalent JSON**. Its definitions live as **CDDL**
embedded in the Bikeshed specification source. Both the W3C and Google consume it
by extracting the CDDL first:

* W3C: `w3c/webdriver-bidi` → `scripts/cddl/generate.js` extracts every
  `<pre class="cddl">` block from `index.bs` into `local.cddl` / `remote.cddl` /
  `all.cddl`.
* Google: `GoogleChromeLabs/webdriver-bidi-protocol` pins the spec as a submodule
  and runs `cddlconv all.cddl > src/gen/main.ts`.

This crate mirrors that pipeline in one script:

```
index.bs  ──extract <pre class="cddl">──▶  all.cddl  ──parse CDDL──▶  Rust
```

```bash
python scripts/generate_rust_code.py            # download the spec + regenerate
python scripts/generate_rust_code.py --spec path/to/index.bs
python scripts/generate_rust_code.py --check    # exit 1 when the spec changed
```

`webdriver_bidi.bs` and `all.cddl` are cached in the crate root; `src/` is
generated and should not be edited by hand.

## Layout

| BiDi module (CDDL) | Rust module |
|---|---|
| `session` | `session` |
| `browsingContext` | `browsing_context` |
| `script` | `script` |
| `input` | `input` |
| `network` | `network` |
| `browser` | `browser` |
| `emulation` | `emulation` |
| `storage` | `storage` |
| `log` | `log` |
| `webExtension` | `web_extension` |

The wire envelope (`Command`, `SuccessResponse`, `ErrorResponse`, `EmptyResult`,
`EmptyParams`, and the `BidiCommand` / `BidiEvent` traits) is hand-written in
`src/lib.rs`; everything else is generated.

## Usage

```rust
use bidi_protocol::{BidiCommand, browsing_context};

let params = browsing_context::NavigateParameters {
    context: "abc".into(),
    url: "https://example.com".into(),
    wait: None,
};
let command = bidi_protocol::Command::new(1, &params);
// browser_context::Navigate::METHOD == "browsingContext.navigate"
```

## Known gaps (first cut)

* Inline anonymous maps/unions inside a field are emitted as `serde_json::Value`
  rather than synthesised structs.
* Group unions with a string discriminant are emitted as `#[serde(untagged)]`
  enums; tagged dispatch is a later refinement.
* `js-uint` maps to `u64` and `text` to `String` (owned, not `Cow`).
