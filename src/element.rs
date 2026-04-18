use crate::page::Page;
use crate::error::XcelerateResult;

/// Represents an HTML element in the DOM.
pub struct Element {
    pub(crate) page: Page,
    pub(crate) object_id: String,
}

impl Element {
    /// Clicks the element.
    pub async fn click(&self) -> XcelerateResult<&Self> {
        self.page.client.execute_with_session(
            Some(&self.page.session_id),
            js_protocol::runtime::CallFunctionOnParams {
                functionDeclaration: "function() { this.click(); }".into(),
                objectId: Some(self.object_id.clone()),
                ..Default::default()
            }
        ).await?;
        
        Ok(self)
    }

    pub async fn type_text(&self, text: &str) -> XcelerateResult<&Self> {
        let js = "function(t) { this.value = t; this.dispatchEvent(new Event('input', { bubbles: true })); }";
        self.page.client.execute_with_session(
            Some(&self.page.session_id),
            js_protocol::runtime::CallFunctionOnParams {
                functionDeclaration: js.into(),
                arguments: Some(vec![js_protocol::runtime::CallArgument {
                    value: Some(serde_json::json!(text)),
                    ..Default::default()
                }]),
                objectId: Some(self.object_id.clone()),
                ..Default::default()
            }
        ).await?;
        
        Ok(self)
    }

    /// Hovers over the element.
    pub async fn hover(&self) -> XcelerateResult<&Self> {
        self.call_js("function() { this.dispatchEvent(new MouseEvent('mouseover', { bubbles: true })); }").await?;
        Ok(self)
    }

    /// Focuses the element.
    pub async fn focus(&self) -> XcelerateResult<&Self> {
        self.call_js("function() { this.focus(); }").await?;
        Ok(self)
    }

    /// Returns the visible text of the element.
    pub async fn text(&self) -> XcelerateResult<String> {
        let res = self.call_js("function() { return this.innerText; }").await?;
        Ok(res.result.value.and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_default())
    }

    /// Returns the value of a specific attribute.
    pub async fn attribute(&self, name: &str) -> XcelerateResult<Option<String>> {
        let js = format!("function() {{ return this.getAttribute('{}'); }}", name);
        let res = self.call_js(&js).await?;
        Ok(res.result.value.and_then(|v| v.as_str().map(|s| s.to_string())))
    }

    /// Returns the inner HTML of the element.
    pub async fn inner_html(&self) -> XcelerateResult<String> {
        let res = self.call_js("function() { return this.innerHTML; }").await?;
        Ok(res.result.value.and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_default())
    }

    /// Helper to call JS on this element.
    async fn call_js(&self, js: &str) -> XcelerateResult<js_protocol::runtime::CallFunctionOnReturns> {
        self.page.client.execute_with_session(
            Some(&self.page.session_id),
            js_protocol::runtime::CallFunctionOnParams {
                functionDeclaration: js.into(),
                objectId: Some(self.object_id.clone()),
                ..Default::default()
            }
        ).await
    }
}
