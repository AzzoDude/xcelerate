//! In-page text search (capability #9).
//!
//! Implemented as inherent `impl Page` methods in this module so they stay out
//! of the `#[uniffi::export]` block and binding checksums remain stable.

use crate::error::{XcelerateError, XcelerateResult};
use crate::page::Page;
use crate::page::highlight::apply_highlight_fn;

/// Returns `true` when `haystack` contains `needle`, ignoring ASCII casing.
///
/// An empty `needle` never matches, so callers can treat it as "no search".
pub(crate) fn looks_like_match(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return false;
    }
    haystack.to_lowercase().contains(&needle.to_lowercase())
}

/// Parses a CDP JSON number into a count, defaulting to `0`.
fn parse_u32(raw: &str) -> u32 {
    raw.trim().parse::<u32>().unwrap_or(0)
}

impl Page {
    /// Finds, scrolls to, and highlights the first element whose visible text
    /// contains `text` (case-insensitive). Returns `1` on success, else `0`.
    pub async fn find_text(&self, text: String) -> XcelerateResult<u32> {
        if text.is_empty() {
            return Ok(0);
        }
        // Cheap pre-check against the rendered body before walking the DOM.
        let body = self
            .evaluate_string("document.body ? document.body.innerText : ''".to_string())
            .await?;
        if !looks_like_match(&body, &text) {
            return Ok(0);
        }

        let needle_lit =
            serde_json::to_string(&text).map_err(|e| XcelerateError::SerdeError(e.to_string()))?;
        let apply_fn = apply_highlight_fn();
        let script = format!(
            "(function(){{const needle={needle_lit}.toLowerCase();const root=document.body||document.documentElement;if(!root)return 0;const walker=document.createTreeWalker(root,NodeFilter.SHOW_TEXT,{{acceptNode:function(n){{const p=n.parentElement;if(!p)return NodeFilter.FILTER_REJECT;const t=p.tagName?p.tagName.toLowerCase():'';if(t==='script'||t==='style'||t==='noscript'||t==='template')return NodeFilter.FILTER_REJECT;if(!n.nodeValue||n.nodeValue.toLowerCase().indexOf(needle)===-1)return NodeFilter.FILTER_SKIP;return NodeFilter.FILTER_ACCEPT;}}}});const hit=walker.nextNode();if(!hit||!hit.parentElement)return 0;const el=hit.parentElement;el.scrollIntoView({{block:'center',inline:'center'}});({apply_fn}).call(el);return 1;}})()"
        );
        let raw = self.evaluate_json(script).await?;
        Ok(parse_u32(&raw))
    }

    /// Counts the top-level elements whose text contains `text`
    /// (case-insensitive).
    ///
    /// An element nested inside another matching element is skipped, so nested
    /// matches are not double counted.
    pub async fn count_text(&self, text: String) -> XcelerateResult<u32> {
        if text.is_empty() {
            return Ok(0);
        }
        let body = self
            .evaluate_string("document.body ? document.body.innerText : ''".to_string())
            .await?;
        if !looks_like_match(&body, &text) {
            return Ok(0);
        }

        let needle_lit =
            serde_json::to_string(&text).map_err(|e| XcelerateError::SerdeError(e.to_string()))?;
        let script = format!(
            "(function(){{const needle={needle_lit}.toLowerCase();const skip={{script:1,style:1,noscript:1,template:1,head:1}};const all=document.querySelectorAll('*');const matches=function(el){{const t=el.tagName?el.tagName.toLowerCase():'';if(skip[t])return false;return (el.textContent||'').toLowerCase().indexOf(needle)!==-1;}};let count=0;for(let i=0;i<all.length;i++){{const el=all[i];if(!matches(el))continue;const p=el.parentElement;if(p&&matches(p))continue;count++;}}return count;}})()"
        );
        let raw = self.evaluate_json(script).await?;
        Ok(parse_u32(&raw))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn looks_like_match_is_case_insensitive() {
        assert!(looks_like_match("Hello World", "world"));
        assert!(looks_like_match("HELLO", "hello"));
        assert!(looks_like_match("hello world", "lo wo"));
        assert!(!looks_like_match("hello", "world"));
    }

    #[test]
    fn looks_like_match_rejects_empty_needle() {
        assert!(!looks_like_match("anything", ""));
    }

    #[test]
    fn parse_u32_handles_numbers_and_garbage() {
        assert_eq!(parse_u32("42"), 42);
        assert_eq!(parse_u32(" 7 "), 7);
        assert_eq!(parse_u32("null"), 0);
    }
}
