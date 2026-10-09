//! `core` - the host primitives that are neither browser nor app: stdout,
//! time, and environment. Every op forwards to the host's `host.core` bridge, so
//! the guest touches nothing directly and the host stays in control.
//!
//! Build: `xcelerate build --wasm-only` (run in this directory) stages `core.wasm`
//! next to this manifest. Then drop the directory in `~/.xcl/plugins/` and load it
//! by name (`core`).

wit_bindgen::generate!({
    path: "wit/plugin.wit",
    world: "plugin-world",
});

use xcelerate_plugin::plugin;

/// The plugin. Stateless: each op forwards to `host.core`.
struct Core;

plugin! {
    guest = Core,
    name = "core",
    bridge = core,
    ops = {
        print,
        now,
        env,
    },
}

export!(Core);
