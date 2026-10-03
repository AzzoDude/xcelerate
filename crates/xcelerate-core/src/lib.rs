//! `xcelerate-core` is the transport layer shared by the `xcelerate` client.
//!
//! It owns the WebSocket connection, multiplexes CDP commands by id, broadcasts
//! events, and exposes a typed [`CdpCommand`] abstraction so callers can send
//! generated protocol structs without hand-writing any wire glue.

pub mod client;
pub mod command;
pub mod error;
pub mod handler;

pub use client::{CdpClient, connect};
pub use command::CdpCommand;
pub use error::{Error, Result};
pub use handler::CdpHandler;

/// Re-exported so downstream crates can build protocol structs without taking a
/// direct dependency on (and version-pinning) the generated protocol crates.
pub use browser_protocol;
pub use js_protocol;
