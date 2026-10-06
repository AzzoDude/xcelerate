//! Browser engines.
//!
//! Chromium is driven over the Chrome DevTools Protocol (CDP); Firefox over
//! [WebDriver BiDi](https://w3c.github.io/webdriver-bidi/). Both engines live
//! here side by side, each in its own module, behind the shared
//! [`engine::Browser`] trait:
//!
//! * [`chromium`] — [`ChromiumBrowser`] (also exported as [`Browser`]) and
//!   [`BrowserConfig`].
//! * [`firefox`] — [`firefox::FirefoxBrowser`] and [`firefox::FirefoxPage`].
//! * [`known`] — the well-known browser catalog and executable discovery.
//! * [`engine`] — the [`engine::Browser`] trait and shared helpers.

mod chromium;
pub mod engine;
pub mod firefox;
pub mod known;

/// The Chromium engine under its descriptive name.
pub use chromium::Browser as ChromiumBrowser;
pub use chromium::*;
