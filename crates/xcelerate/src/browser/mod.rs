//! Browser engines.
//!
//! Chromium is driven over the Chrome DevTools Protocol (CDP); Firefox over
//! [WebDriver BiDi](https://w3c.github.io/webdriver-bidi/). Both engines live
//! here side by side, each in its own module:
//!
//! * [`chromium`] — [`Browser`], [`BrowserConfig`] and CDP launch plumbing.
//! * [`firefox`] — [`firefox::FirefoxBrowser`] and [`firefox::FirefoxPage`].

mod chromium;
pub mod firefox;

pub use chromium::*;
