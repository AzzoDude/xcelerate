//! `xcelerate.app` - native application control as a WebAssembly plugin.
//!
//! Every op forwards to the host's **native-window** bridge (`host.app`), which
//! drives a window through Windows UI Automation. The plugin names an act
//! (`launch`, `tree`, `click`, `set_value`, …) and the host maps it onto the
//! desktop backend; the plugin never touches the OS itself.
//!
//! Build: `./build.sh` (Windows: `.\build.ps1`), which stages `app.wasm` next
//! to this manifest. Then drop the directory in `~/.xcl/plugins/` and load it by
//! name (`app`). The bridge is Windows only.

wit_bindgen::generate!({
    path: "wit/plugin.wit",
    world: "plugin-world",
});

use serde::Serialize;

const NAME: &str = "xcelerate.app";

/// The op table: the verb this plugin exposes -> the host bridge verb it maps
/// to. Keeping the mapping here (not in the CLI) is what lets the native-app
/// surface evolve as a plugin.
const OPS: &[(&str, &str)] = &[
    ("windows", "windows"),
    ("launch", "launch"),
    ("tree", "tree"),
    ("find", "find"),
    ("wait", "wait"),
    ("click", "click"),
    ("set_value", "set-value"),
    ("key", "key"),
    ("wheel", "wheel"),
    ("scroll", "scroll"),
];

struct App;

impl exports::xcelerate::plugin::plugin::Guest for App {
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
        xcelerate::plugin::host::app(host_op, &args)
    }
}

export!(App);
