//! # xcelerate
//!
//! `xcelerate` is a high-performance, lightweight Chrome DevTools Protocol (CDP) client.
//! It provides a fluent, chained API for browser automation, designed for speed and reliability.
//!
//! The workspace is split into focused crates:
//!
//! * [`xcelerate_core`] — the WebSocket transport and typed command layer.
//! * [`xcelerate_plugin_api`] — the plugin trait, manifest, audit log, and the
//!   `PageHost` interface plugins use to reach a page.
//! * [`xcelerate_plugins`] — the first-party plugins (`stealth`, `human`) and the
//!   low-level OS helpers they and the engine rely on.
//!
//! This crate is the thin facade: it composes those pieces into the high-level
//! [`Browser`], [`Page`], and [`Element`] API and exposes it to other languages
//! through `uniffi`.

pub mod browser;
pub mod element;
pub mod error;
pub mod page;
pub mod plugin;

mod proxy;

pub mod adapters;

pub use browser::{Browser, BrowserConfig, configure_user_data_dir};
pub use element::Element;
pub use error::{XcelerateError, XcelerateResult};
pub use page::Page;
pub use page::recording::VideoOptions;
pub use plugin::{Capability, Manifest, Plugin, PluginHandle, Tier};

// Proxy configuration (Rust-only; other languages set `XCELERATE_PROXY[_POOL]`).
pub use proxy::configure as configure_proxy;

// Re-export the transport layer and the generated protocol crates so downstream
// code can reach everything through `xcelerate`.
pub use xcelerate_core::{CdpClient, CdpCommand, CdpHandler, connect};
pub use xcelerate_core::{browser_protocol, js_protocol};

uniffi::setup_scaffolding!("xcelerate");
