//! Stealth helpers for `xcelerate`: binary patching, detached process
//! management, and the runtime payload injected into every document.
//!
//! This crate is intentionally OS-level and synchronous — it carries no async
//! runtime, WebSocket, or FFI dependencies so it stays small and fast to build.

pub mod error;
pub mod patcher;
pub mod process;

pub use error::{Error, Result};
pub use patcher::BinaryPatcher;
pub use process::{ProcessGuard, ProcessRegistry, spawn_detached};

/// JavaScript injected into every new document to mask common automation
/// signals (`navigator.webdriver`, `cdc_` leaks, `window.chrome`, WebGL, ...).
pub const CDC_PAYLOAD: &str = include_str!("cdc_payload.js");
