//! The shared browser-engine trait and small helpers common to every engine.
//!
//! Chromium (`browser::Browser`) and Firefox ([`super::firefox::FirefoxBrowser`])
//! speak different protocols, but they share a lifecycle: they identify
//! themselves, report whether the transport is alive, and shut down. [`Browser`]
//! is that shared surface, so the rest of the crate can hold either engine
//! through one trait object.

use std::future::Future;
use std::pin::Pin;

use crate::error::XcelerateResult;

use super::known::Engine;

/// A boxed future, the return shape a trait object needs for an `async` method.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// The surface every browser engine exposes.
pub trait Browser: Send + Sync + 'static {
    /// Which protocol/engine this is.
    fn engine(&self) -> Engine;

    /// Whether the transport is still live. This must not perform a round-trip.
    fn connected(&self) -> bool;

    /// Shut the browser down and release its resources.
    fn close(&self) -> BoxFuture<'_, XcelerateResult<()>>;
}

/// Bind a loopback socket and return the OS-assigned port.
///
/// Shared by every engine: both ask the OS for a free port and then pass it to
/// the browser as `--remote-debugging-port`.
pub(crate) fn free_port() -> Option<u16> {
    std::net::TcpListener::bind("127.0.0.1:0")
        .and_then(|listener| listener.local_addr())
        .map(|addr| addr.port())
        .ok()
}
