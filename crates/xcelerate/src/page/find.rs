//! In-page text search (capability #9).
//!
//! Implemented as inherent `impl Page` methods in this module so they stay out
//! of the `#[uniffi::export]` block and binding checksums remain stable.

use crate::element::JS_QUERY_ALL;
use crate::error::{XcelerateError, XcelerateResult};
use crate::page::Page;
use crate::page::highlight::apply_highlight_fn;

/// Parses a CDP JSON number into a count, defaulting to `0`.
fn parse_u32(raw: &str) -> u32 {
    raw.trim().parse::<u32>().unwrap_or(0)
}

impl Page {
    /// Finds, scrolls to, and highlights the first element whose visible text
    /// contains `text` (case-insensitive). Returns `1` on success, else `0`.
    ///
    /// The text walk runs per composed root (the document plus every open shadow
    /// root), because a `TreeWalker` starting at `document.body` cannot see into a
    /// shadow tree.
    pub async fn find_text(&self, text: String) -> XcelerateResult<u32> {
        if text.is_empty() {
            return Ok(0);
        }

        let needle_lit =
            serde_json::to_string(&text).map_err(|e| XcelerateError::SerdeError(e.to_string()))?;
        let apply_fn = apply_highlight_fn();
        let script = format!(
            "(function(){{const needle={needle_lit}.toLowerCase();const start=document.body||document.documentElement;if(!start)return 0;const roots=[];(function collect(scope){{if(!scope||!scope.querySelectorAll)return;roots.push(scope);const els=scope.querySelectorAll('*');for(let i=0;i<els.length;i++){{if(els[i].shadowRoot)collect(els[i].shadowRoot);}}}})(start);for(let r=0;r<roots.length;r++){{const walker=document.createTreeWalker(roots[r],NodeFilter.SHOW_TEXT,{{acceptNode:function(n){{const p=n.parentElement;if(!p)return NodeFilter.FILTER_REJECT;const t=p.tagName?p.tagName.toLowerCase():'';if(t==='script'||t==='style'||t==='noscript'||t==='template')return NodeFilter.FILTER_REJECT;if(!n.nodeValue||n.nodeValue.toLowerCase().indexOf(needle)===-1)return NodeFilter.FILTER_SKIP;return NodeFilter.FILTER_ACCEPT;}}}});const hit=walker.nextNode();if(hit&&hit.parentElement){{const el=hit.parentElement;el.scrollIntoView({{block:'center',inline:'center'}});({apply_fn}).call(el);return 1;}}}}return 0;}})()"
        );
        let raw = self.evaluate_json(script).await?;
        Ok(parse_u32(&raw))
    }

    /// Counts the elements whose text contains `text` (case-insensitive).
    ///
    /// The element walk pierces open shadow roots (via [`JS_QUERY_ALL`]). An
    /// element nested inside another matching element is skipped, so nested
    /// matches are not double counted.
    pub async fn count_text(&self, text: String) -> XcelerateResult<u32> {
        if text.is_empty() {
            return Ok(0);
        }

        let needle_lit =
            serde_json::to_string(&text).map_err(|e| XcelerateError::SerdeError(e.to_string()))?;
        let script = format!(
            "(function(){{const needle={needle_lit}.toLowerCase();const skip={{script:1,style:1,noscript:1,template:1,head:1}};const all=({JS_QUERY_ALL}).call(document,'*');const matches=function(el){{const t=el.tagName?el.tagName.toLowerCase():'';if(skip[t])return false;return (el.textContent||'').toLowerCase().indexOf(needle)!==-1;}};let count=0;for(let i=0;i<all.length;i++){{const el=all[i];if(!matches(el))continue;const p=el.parentElement;if(p&&matches(p))continue;count++;}}return count;}})()"
        );
        let raw = self.evaluate_json(script).await?;
        Ok(parse_u32(&raw))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_u32_handles_numbers_and_garbage() {
        assert_eq!(parse_u32("42"), 42);
        assert_eq!(parse_u32(" 7 "), 7);
        assert_eq!(parse_u32("null"), 0);
    }
}
