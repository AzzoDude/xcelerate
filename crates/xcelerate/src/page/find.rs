//! In-page text search (capability #9).
//!
//! Implemented as inherent `impl Page` methods in this module so they stay out
//! of the `#[uniffi::export]` block and binding checksums remain stable.
//!
//! Matching is delegated to [`Page`]'s shared text matcher
//! (see `page::text_match`), so `find` and `click` agree on the same element:
//! NFC-normalized, `aria-label`-aware, visibility-checked, and piercing open
//! shadow roots and same-origin frames.

use crate::error::XcelerateResult;
use crate::page::Page;
use crate::page::text_match::{count_text_matches, find_text_scroll_highlight};

impl Page {
    /// Finds, scrolls to, and highlights the first element whose visible text
    /// contains `text` (case-insensitive). Returns `1` on success, else `0`.
    pub async fn find_text(&self, text: String) -> XcelerateResult<u32> {
        find_text_scroll_highlight(self, text).await
    }

    /// Counts the distinct elements whose text contains `text`
    /// (case-insensitive), using the same matcher as [`Page::find_text`] and the
    /// text-target resolution used by `click`.
    pub async fn count_text(&self, text: String) -> XcelerateResult<u32> {
        count_text_matches(self, text).await
    }
}
