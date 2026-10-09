//! `app` - native application control as a WebAssembly plugin.
//!
//! Every op forwards to the host's **native-window** bridge (`host.app`), which
//! drives a window through Windows UI Automation. The plugin names an act
//! (`launch`, `tree`, `click`, `set_value`, …) and the host maps it onto the
//! desktop backend; the plugin never touches the OS itself.
//!
//! The op table is a single `plugin!` declaration: each op forwards its arguments
//! to the same-named host verb, unless `=> "verb"` renames it. So `set_value`
//! forwards to the host verb `set-value`.
//!
//! Build: `xcelerate build --wasm-only` (run in this directory) stages `app.wasm`
//! next to this manifest. Then drop the directory in `~/.xcl/plugins/` and load
//! it by name (`app`). The bridge is Windows only.

wit_bindgen::generate!({
    path: "wit/plugin.wit",
    world: "plugin-world",
});

use xcelerate_plugin::plugin;

struct App;

plugin! {
    guest = App,
    name = "app",
    bridge = app,
    ops = {
        windows,
        launch,
        tree,
        find,
        wait,
        click,
        // `set_value` forwards to the host verb `set-value`.
        set_value => "set-value",
        key,
        wheel,
        scroll,
    },
}

export!(App);
