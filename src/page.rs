use crate::connection::CdpClient;
use crate::element::Element;
use crate::error::XcelerateResult;
use std::sync::Arc;

#[derive(Clone)]
pub struct Page {
    pub(crate) client: Arc<CdpClient>,
    pub(crate) session_id: String,
}

impl Page {
    pub async fn find_element(&self, selector: &str) -> XcelerateResult<Element> {
        let js = format!("document.querySelector('{}')", selector);
        
        let result = self.evaluate(js).await?;

        if let Some(obj_id) = result.result.objectId {
            Ok(Element {
                page: self.clone(),
                object_id: obj_id,
            })
        } else {
            Err(crate::error::XcelerateError::NotFound(selector.to_string()))
        }
    }

    /// Waits for an element matching the selector to appear in the DOM.
    pub async fn wait_for_selector(&self, selector: &str) -> XcelerateResult<Element> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(30);

        while start.elapsed() < timeout {
            if let Ok(element) = self.find_element(selector).await {
                return Ok(element);
            }
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        }

        Err(crate::error::XcelerateError::NotFound(format!("Timeout waiting for selector: {}", selector)))
    }

    /// Waits for the page to finish loading.
    pub async fn wait_for_navigation(&self) -> XcelerateResult<()> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(30);

        while start.elapsed() < timeout {
            let res = self.evaluate("document.readyState").await?;
            if res.result.value.map_or(false, |v| v.as_str() == Some("complete")) {
                return Ok(());
            }
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        }

        Err(crate::error::XcelerateError::NotFound("Navigation timeout".into()))
    }

    /// Evaluates a JavaScript expression in the context of the page.
    pub async fn evaluate(&self, expression: impl Into<String>) -> XcelerateResult<js_protocol::runtime::EvaluateReturns> {
        self.client.execute_with_session(
            Some(&self.session_id),
            js_protocol::runtime::EvaluateParams {
                expression: expression.into(),
                ..Default::default()
            }
        ).await
    }

    /// Reloads the page.
    pub async fn reload(&self) -> XcelerateResult<()> {
        self.client.execute_with_session(
            Some(&self.session_id),
            browser_protocol::page::ReloadParams { ..Default::default() }
        ).await
    }

    /// Navigates to a URL.
    pub async fn navigate(&self, url: impl Into<String>) -> XcelerateResult<browser_protocol::page::NavigateReturns> {
        self.client.execute_with_session(
            Some(&self.session_id),
            browser_protocol::page::NavigateParams { 
                url: url.into(), 
                ..Default::default() 
            }
        ).await
    }

    /// Returns the page title.
    pub async fn title(&self) -> XcelerateResult<String> {
        let res = self.evaluate("document.title").await?;
        Ok(res.result.value.and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_default())
    }

    /// Returns the full HTML content of the page.
    pub async fn content(&self) -> XcelerateResult<String> {
        let res = self.evaluate("document.documentElement.outerHTML").await?;
        Ok(res.result.value.and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_default())
    }

    /// Captures a screenshot of the page as a PNG.
    pub async fn screenshot(&self) -> XcelerateResult<Vec<u8>> {
        use base64::{Engine as _, engine::general_purpose};
        let res = self.client.execute_with_session(
            Some(&self.session_id),
            browser_protocol::page::CaptureScreenshotParams { ..Default::default() }
        ).await?;
        Ok(general_purpose::STANDARD.decode(res.data).map_err(|_| crate::error::XcelerateError::InternalError)?)
    }

    /// Captures a PDF of the page.
    pub async fn pdf(&self) -> XcelerateResult<Vec<u8>> {
        use base64::{Engine as _, engine::general_purpose};
        let res = self.client.execute_with_session(
            Some(&self.session_id),
            browser_protocol::page::PrintToPDFParams { ..Default::default() }
        ).await?;
        Ok(general_purpose::STANDARD.decode(res.data).map_err(|_| crate::error::XcelerateError::InternalError)?)
    }

    /// Evaluates a script on every new document (before scripts on the page run).
    pub async fn add_script_to_evaluate_on_new_document(&self, source: impl Into<String>) -> XcelerateResult<browser_protocol::page::ScriptIdentifier> {
        let res = self.client.execute_with_session(
            Some(&self.session_id),
            browser_protocol::page::AddScriptToEvaluateOnNewDocumentParams {
                source: source.into(),
                ..Default::default()
            }
        ).await?;
        Ok(res.identifier)
    }

    /// Navigates back in history.
    pub async fn go_back(&self) -> XcelerateResult<()> {
        let history = self.client.execute_with_session(
            Some(&self.session_id),
            browser_protocol::page::GetNavigationHistoryParams {}
        ).await?;
        
        if history.currentIndex > 0 {
            let entry = &history.entries[history.currentIndex as usize - 1];
            self.client.execute_with_session(
                Some(&self.session_id),
                browser_protocol::page::NavigateToHistoryEntryParams { entryId: entry.id }
            ).await?;
        }
        Ok(())
    }
}
