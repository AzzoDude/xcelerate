use crate::page::Page;
use crate::error::XcelerateResult;
use std::sync::Arc;

/// Represents an HTML element in the DOM.
#[derive(uniffi::Object)]
pub struct Element {
    pub(crate) page: Arc<Page>,
    pub(crate) object_id: String,
}

#[uniffi::export(async_runtime = "tokio")]
impl Element {
    /// Clicks the element.
    pub async fn click(self: Arc<Self>) -> XcelerateResult<Arc<Self>> {
        self.call_js("function() { this.click(); }".to_string()).await?;
        Ok(self)
    }

    pub async fn type_text(self: Arc<Self>, text: String) -> XcelerateResult<Arc<Self>> {
        // 1. Focus the element first
        self.clone().focus().await?;

        // 2. Dispatch key events for each character
        for c in text.chars() {
            let mut params = browser_protocol::input::DispatchKeyEventParams {
                type_: "char".into(),
                ..Default::default()
            };
            params.text = Some(c.to_string());
            params.unmodifiedText = Some(c.to_string());

            self.page.client.execute_with_session(
                Some(&self.page.session_id),
                params
            ).await?;
            
            // Subtle delay to mimic human typing
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
        
        Ok(self)
    }

    /// Hovers over the element.
    pub async fn hover(self: Arc<Self>) -> XcelerateResult<Arc<Self>> {
        self.call_js("function() { this.dispatchEvent(new MouseEvent('mouseover', { bubbles: true })); }".to_string()).await?;
        Ok(self)
    }

    /// Focuses the element.
    pub async fn focus(self: Arc<Self>) -> XcelerateResult<Arc<Self>> {
        self.call_js("function() { this.focus(); }".to_string()).await?;
        Ok(self)
    }

    /// Returns the visible text of the element.
    pub async fn text(&self) -> XcelerateResult<String> {
        let res = self.call_js("function() { return this.innerText; }".to_string()).await?;
        Ok(res.result.value.and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_default())
    }

    /// Returns the value of a specific attribute.
    pub async fn attribute(&self, name: String) -> XcelerateResult<Option<String>> {
        let js = format!("function() {{ return this.getAttribute('{}'); }}", name);
        let res = self.call_js(js).await?;
        Ok(res.result.value.and_then(|v| v.as_str().map(|s| s.to_string())))
    }

    /// Returns the inner HTML of the element.
    pub async fn inner_html(&self) -> XcelerateResult<String> {
        let res = self.call_js("function() { return this.innerHTML; }".to_string()).await?;
        Ok(res.result.value.and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_default())
    }
}

impl Element {
    /// Helper to call JS on this element.
    async fn call_js(&self, js: String) -> XcelerateResult<js_protocol::runtime::CallFunctionOnReturns> {
        self.page.client.execute_with_session(
            Some(&self.page.session_id),
            js_protocol::runtime::CallFunctionOnParams {
                functionDeclaration: js,
                objectId: Some(self.object_id.clone()),
                ..Default::default()
            }
        ).await
    }
}
