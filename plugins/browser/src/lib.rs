//! `xcelerate.browser` - the browser control surface as a WebAssembly plugin.
//!
//! Every op forwards to the host's **semantic** browser bridge
//! (`host.browser`): the plugin names an act (`open`, `click`, `fill`, …) and
//! the host maps it onto the engine. There is no CDP/BiDi here - the plugin
//! never sees a protocol message, and the host stays in control of what a
//! sandboxed guest may do.
//!
//! Build: `./build.sh` (Windows: `.\build.ps1`), which stages `browser.wasm`
//! next to this manifest. Then drop the directory in `~/.xcl/plugins/` and load
//! it by name (`browser`).

wit_bindgen::generate!({
    path: "wit/plugin.wit",
    world: "plugin-world",
});

use serde::Serialize;

const NAME: &str = "xcelerate.browser";

/// The op table: the verb this plugin exposes -> the host bridge verb it maps
/// to. Keeping the mapping here (not in the engine) is what lets the browser
/// surface evolve as a plugin.
const OPS: &[(&str, &str)] = &[
    ("open", "goto"),
    ("title", "title"),
    ("url", "url"),
    ("text", "text"),
    ("html", "html"),
    ("markdown", "markdown"),
    ("snapshot", "snapshot"),
    ("click", "click"),
    ("hover", "hover"),
    ("fill", "fill"),
    ("press", "press"),
    ("scroll", "scroll"),
    ("wait", "wait"),
    ("find", "find"),
    ("evaluate", "evaluate"),
    ("screenshot", "screenshot"),
];

struct Browser;

impl exports::xcelerate::plugin::plugin::Guest for Browser {
    fn describe() -> Vec<u8> {
        #[derive(Serialize)]
        struct Describe<'a> {
            name: &'a str,
            version: &'a str,
            ops: Vec<&'a str>,
        }
        let describe = Describe {
            name: NAME,
            version: env!("CARGO_PKG_VERSION"),
            ops: OPS.iter().map(|(op, _)| *op).collect(),
        };
        rmp_serde::to_vec_named(&describe).unwrap_or_default()
    }

    fn invoke(op: String, args: Vec<u8>) -> Result<Vec<u8>, String> {
        let host_op = OPS
            .iter()
            .find(|(verb, _)| *verb == op)
            .map(|(_, host)| *host)
            .ok_or_else(|| format!("{NAME}: unknown op '{op}'"))?;
        xcelerate::plugin::host::browser(host_op, &args)
    }
}

export!(Browser);
