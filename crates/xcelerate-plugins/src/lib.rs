//! Built-in plugin catalog for `xcelerate`.
//!
//! The plugins are **independent crates**, each in its own directory under
//! `plugins/`:
//!
//! * [`xcelerate_plugin_stealth`] - binary patching at launch + the
//!   anti-fingerprint payload.
//! * [`xcelerate_plugin_human`] - human-like mouse, typing, and scrolling.
//!
//! This crate is just the single lookup the engine's plugin manager uses.
//! Built-in plugins are trusted and may use privileged primitives (launch
//! control, binary patching, detached spawn, init scripts); loaded plugins never
//! can.

use std::sync::Arc;

use xcelerate_plugin_api::Plugin;

/// Names of all compiled-in built-in plugins.
pub fn builtin_names() -> &'static [&'static str] {
    &["stealth", "human"]
}

/// Whether `name` is a reserved built-in plugin name.
pub fn is_builtin(name: &str) -> bool {
    builtin_names().contains(&name)
}

/// Construct a built-in plugin by name, or `None` if unknown.
pub fn builtin(name: &str) -> Option<Arc<dyn Plugin>> {
    match name {
        "stealth" => Some(Arc::new(xcelerate_plugin_stealth::StealthPlugin)),
        "human" => Some(Arc::new(xcelerate_plugin_human::HumanPlugin::default())),
        _ => None,
    }
}
