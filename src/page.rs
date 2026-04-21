use crate::connection::CdpClient;
use crate::element::Element;
use crate::error::{XcelerateResult, XcelerateError};
use std::sync::Arc;
use browser_protocol::page::{GetLayoutMetricsParams, CaptureScreenshotParams, ReloadParams, NavigateParams, EnableParams};
use browser_protocol::emulation::{SetDeviceMetricsOverrideParams, ClearDeviceMetricsOverrideParams};

#[derive(uniffi::Object)]
pub struct Page {
    pub(crate) client: Arc<CdpClient>,
    pub(crate) session_id: String,
}

#[uniffi::export(async_runtime = "tokio")]
impl Page {
    /// Finds an element matching the CSS selector.
    pub async fn find_element(self: Arc<Self>, selector: String) -> XcelerateResult<Arc<Element>> {
        let js = format!("document.querySelector('{}')", selector);
        
        // Evaluate returns complex JSON, we handle it internally
        self.client.execute_with_session(
            Some(&self.session_id),
            js_protocol::runtime::EvaluateParams {
                expression: js,
                ..Default::default()
            }
        ).await.and_then(|result| {
            if let Some(obj_id) = result.result.objectId {
                Ok(Arc::new(Element {
                    page: self.clone(),
                    object_id: obj_id,
                }))
            } else {
                Err(XcelerateError::NotFound(selector))
            }
        })
    }

    /// Waits for an element matching the selector to appear in the DOM.
    pub async fn wait_for_selector(self: Arc<Self>, selector: String) -> XcelerateResult<Arc<Element>> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(30);

        while start.elapsed() < timeout {
            if let Ok(element) = self.clone().find_element(selector.clone()).await {
                return Ok(element);
            }
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        }

        Err(XcelerateError::NotFound(format!("Timeout waiting for selector: {}", selector)))
    }

    /// Waits for the page to finish loading.
    pub async fn wait_for_navigation(&self) -> XcelerateResult<()> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(30);

        while start.elapsed() < timeout {
            // Internal call to evaluate
            let res = self.client.execute_with_session(
                Some(&self.session_id),
                js_protocol::runtime::EvaluateParams {
                    expression: "document.readyState".into(),
                    ..Default::default()
                }
            ).await?;
            
            if res.result.value.map_or(false, |v| v.as_str() == Some("complete")) {
                return Ok(());
            }
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        }

        Err(XcelerateError::NotFound("Navigation timeout".into()))
    }

    /// Reloads the page.
    pub async fn reload(&self) -> XcelerateResult<()> {
        self.client.execute_with_session(
            Some(&self.session_id),
            ReloadParams { ..Default::default() }
        ).await.map(|_| ())
    }

    /// Navigates to a URL.
    pub async fn navigate(&self, url: String) -> XcelerateResult<()> {
        self.client.execute_with_session(
            Some(&self.session_id),
            NavigateParams { 
                url, 
                ..Default::default() 
            }
        ).await.map(|_| ())
    }

    /// Returns the page title.
    pub async fn title(&self) -> XcelerateResult<String> {
        let res = self.client.execute_with_session(
            Some(&self.session_id),
            js_protocol::runtime::EvaluateParams {
                expression: "document.title".into(),
                ..Default::default()
            }
        ).await?;
        Ok(res.result.value.and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_default())
    }

    /// Returns the full HTML content of the page.
    pub async fn content(&self) -> XcelerateResult<String> {
        let res = self.client.execute_with_session(
            Some(&self.session_id),
            js_protocol::runtime::EvaluateParams {
                expression: "document.documentElement.outerHTML".into(),
                ..Default::default()
            }
        ).await?;
        Ok(res.result.value.and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_default())
    }

    pub async fn screenshot(&self) -> XcelerateResult<Vec<u8>> {
        let res = self.client.execute_with_session(
            Some(&self.session_id),
            CaptureScreenshotParams { ..Default::default() }
        ).await?;
        self.decode_base64(res.data)
    }

    pub async fn screenshot_full(&self) -> XcelerateResult<Vec<u8>> {
        let _ = self.client.execute_with_session(
            Some(&self.session_id),
            EnableParams { ..Default::default() }
        ).await?;

        let metrics = self.client.execute_with_session(
            Some(&self.session_id),
            GetLayoutMetricsParams {}
        ).await?;

        let width = metrics.contentSize.width as u64;
        let height = metrics.contentSize.height as i64;

        let mut params = SetDeviceMetricsOverrideParams { ..Default::default() };
        params.width = width;
        params.height = height;
        params.deviceScaleFactor = 1.0;
        params.mobile = false;

        self.client.execute_with_session(
            Some(&self.session_id),
            params
        ).await?;

        let res = self.client.execute_with_session(
            Some(&self.session_id),
            CaptureScreenshotParams { ..Default::default() }
        ).await?;

        let _ = self.client.execute_with_session(
            Some(&self.session_id),
            ClearDeviceMetricsOverrideParams {}
        ).await?;

        self.decode_base64(res.data)
    }

    pub async fn pdf(&self) -> XcelerateResult<Vec<u8>> {
        let res = self.client.execute_with_session(
            Some(&self.session_id),
            browser_protocol::page::PrintToPDFParams { ..Default::default() }
        ).await?;
        self.decode_base64(res.data)
    }

    /// Evaluates a script on every new document.
    pub async fn add_script_to_evaluate_on_new_document(&self, source: String) -> XcelerateResult<String> {
        let res = self.client.execute_with_session(
            Some(&self.session_id),
            browser_protocol::page::AddScriptToEvaluateOnNewDocumentParams {
                source,
                ..Default::default()
            }
        ).await?;
        Ok(res.identifier)
    }

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

    fn decode_base64(&self, data: String) -> XcelerateResult<Vec<u8>> {
        use base64::{Engine as _, engine::general_purpose};
        general_purpose::STANDARD.decode(data).map_err(|e| XcelerateError::SerdeError(format!("Base64 decode failed: {}", e)))
    }
}
