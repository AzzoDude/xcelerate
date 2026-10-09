//! Shared selector resolution (capability #9).
//!
//! One resolver is used everywhere a selector is accepted, so the session and
//! XCL verbs gain resilient selectors for free:
//!
//! * `xpath=<expr>` — XPath (pierces shadow roots and same-origin frames)
//! * `role=<name>`   — first element with that ARIA role
//! * `text=<text>`   — first element whose text matches (shared text matcher)
//! * `label=<text>`  — form control associated with that `<label>`
//!
//! Anything else is a shadow-piercing CSS query.
//!
//! Implemented as inherent `impl Page` methods in this module so they stay out of
//! the `#[uniffi::export]` block and binding checksums remain stable.

use std::sync::Arc;

use crate::element::Element;
use crate::error::{XcelerateError, XcelerateResult};
use crate::page::Page;

impl Page {
    /// Resolves a selector that may carry a `role=`/`text=`/`label=`/`xpath=`
    /// prefix, falling back to a shadow-piercing CSS query.
    ///
    /// This is the single entry point behind [`Page::find_element`] and
    /// [`Page::wait_for_selector`], so every verb that accepts a selector accepts
    /// these prefixes too.
    pub async fn resolve_selector(
        self: Arc<Self>,
        selector: String,
    ) -> XcelerateResult<Arc<Element>> {
        let selector = selector.trim().to_string();
        if let Some(rest) = selector.strip_prefix("xpath=") {
            return self.query_selector_xpath(rest.trim().to_string()).await;
        }
        if let Some(rest) = selector.strip_prefix("role=") {
            return self.get_by_role(rest.trim().to_string()).await;
        }
        if let Some(rest) = selector.strip_prefix("label=") {
            return self.get_by_label(rest.trim().to_string()).await;
        }
        if let Some(rest) = selector.strip_prefix("text=") {
            let needle = rest.trim().to_string();
            return self
                .clone()
                .find_text_element(needle.clone())
                .await?
                .ok_or_else(|| {
                    XcelerateError::NotFound(format!("text= selector matched nothing: {needle:?}"))
                });
        }
        self.document_element()
            .await?
            .query_selector(selector)
            .await
    }
}
