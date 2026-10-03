//! First-party plugins for `xcelerate`, compiled into the engine and running
//! in-process.
//!
//! Every plugin lives in its own module and is reachable through [`builtin`],
//! the single lookup the engine's plugin manager uses. First-party plugins are
//! trusted and may use privileged primitives (launch control, binary patching,
//! detached spawn, init scripts); third-party plugins never can.
//!
//! * [`stealth`] - binary patching at launch + the anti-fingerprint payload.
//! * [`human`] - human-like mouse, typing, and scrolling.
//!
//! The low-level OS helpers the plugins (and the engine) rely on -
//! [`BinaryPatcher`], [`spawn_detached`], [`ProcessGuard`] - also live here.

pub mod error;
pub mod human;
pub mod patcher;
pub mod process;
pub mod stealth;

pub use error::{Error, Result};
pub use patcher::BinaryPatcher;
pub use process::{ProcessGuard, ProcessRegistry, spawn_detached};

use std::sync::Arc;

use xcelerate_plugin_api::{Plugin, PluginError};

/// JavaScript injected into every new document by the `stealth` plugin to mask
/// common automation signals (`navigator.webdriver`, `cdc_` leaks, `window.chrome`,
/// WebGL, ...).
pub const CDC_PAYLOAD: &str = include_str!("cdc_payload.js");

/// Names of all compiled-in first-party plugins.
pub fn builtin_names() -> &'static [&'static str] {
    &["stealth", "human"]
}

/// Whether `name` is a reserved first-party plugin name.
pub fn is_builtin(name: &str) -> bool {
    builtin_names().contains(&name)
}

/// Construct a first-party plugin by name, or `None` if unknown.
pub fn builtin(name: &str) -> Option<Arc<dyn Plugin>> {
    match name {
        "stealth" => Some(Arc::new(stealth::StealthPlugin)),
        "human" => Some(Arc::new(human::HumanPlugin::default())),
        _ => None,
    }
}

impl From<Error> for PluginError {
    fn from(error: Error) -> Self {
        PluginError::Message(error.to_string())
    }
}
