//! `browser` - the browser control surface as a WebAssembly plugin.
//!
//! Every op forwards to the host's **semantic** browser bridge
//! (`host.browser`): the plugin names an act (`open`, `click`, `fill`, …) and
//! the host maps it onto the engine. There is no CDP/BiDi here - the plugin
//! never sees a protocol message, and the host stays in control of what a
//! sandboxed guest may do.
//!
//! The op table is a single `plugin!` declaration: each op forwards its arguments
//! to the same-named host verb, unless `=> "verb"` renames it. So `open` forwards
//! to the host verb `goto`, and everything else maps 1:1.
//!
//! Build: `xcelerate build --wasm-only` (run in this directory) stages
//! `browser.wasm` next to this manifest. Then drop the directory in
//! `~/.xcl/plugins/` and load it by name (`browser`).

wit_bindgen::generate!({
    path: "wit/plugin.wit",
    world: "plugin-world",
});

use xcelerate_plugin::plugin;

struct Browser;

plugin! {
    guest = Browser,
    name = "browser",
    bridge = browser,
    ops = {
        // `open` forwards to the host verb `goto`.
        open => "goto",
        title,
        url,
        text,
        html,
        markdown,
        snapshot,
        click,
        hover,
        fill,
        press,
        scroll,
        wait,
        find,
        evaluate,
        screenshot,
    },
}

export!(Browser);
