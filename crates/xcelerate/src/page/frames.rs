//! Cross-frame element queries.
//!
//! A page script cannot reach a cross-origin frame, but a same-origin `<iframe>`
//! document is reachable, and a surprising amount of content (embedded widgets,
//! editors, payment fields, consent frames) lives there. These methods search
//! across the whole document, its open shadow roots and every same-origin frame.
//!
//! Implemented as inherent `impl Page` methods in this module so they stay out of
//! the `#[uniffi::export]` block and binding checksums remain stable.

use std::sync::Arc;

use serde_json::Value;

use crate::element::Element;
use crate::error::{XcelerateError, XcelerateResult};
use crate::page::Page;

impl Page {
    /// Returns every element matching `selector` across the document, its open
    /// shadow roots, and every same-origin iframe document.
    ///
    /// Cross-origin frames are unreachable from page script and are skipped, so
    /// the result is a best-effort view of everything the page can see.
    pub async fn query_selector_all_frames(
        self: Arc<Self>,
        selector: String,
    ) -> XcelerateResult<Vec<Arc<Element>>> {
        let res = self
            .client
            .execute_raw_with_session(
                Some(&self.session_id),
                "Runtime.evaluate",
                serde_json::json!({
                    "expression": "document.documentElement",
                    "returnByValue": false,
                }),
            )
            .await?;
        let object_id = res
            .pointer("/result/objectId")
            .and_then(Value::as_str)
            .ok_or_else(|| XcelerateError::NotFound("document has no root element".to_string()))?;

        let root = Arc::new(Element {
            page: self,
            object_id: object_id.to_string(),
        });
        root.query_selector_all_frames(selector).await
    }
}
