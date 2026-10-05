#![allow(dead_code)]
#![allow(unused_imports)]
//! Shared helpers for the generated API-style adapters.
//!
//! The generated modules (`playwright`, `puppeteer`, `selenium`) are thin; the
//! small amount of non-trivial glue lives here.

use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{Browser, BrowserConfig, Page, XcelerateError};

/// Backing state for a Selenium-style driver: a browser plus its working page.
pub struct DriverHandle {
    pub(crate) browser: Arc<Browser>,
    pub(crate) page: Arc<Page>,
}

impl DriverHandle {
    pub(crate) async fn new(config: Option<BrowserConfig>) -> Result<Self, XcelerateError> {
        let browser = Browser::launch(config.unwrap_or_default()).await?;
        let page = Arc::clone(&browser)
            .new_page("about:blank".to_string())
            .await?;
        Ok(Self { browser, page })
    }
}

/// Backing state for a Playwright-style context: a browser plus init scripts.
pub struct ContextHandle {
    pub(crate) browser: Arc<Browser>,
    scripts: Mutex<Vec<String>>,
    page: Mutex<Option<Arc<Page>>>,
    events: Mutex<Vec<String>>,
}

impl ContextHandle {
    pub(crate) fn new(browser: Arc<Browser>) -> Self {
        Self {
            browser,
            scripts: Mutex::new(Vec::new()),
            page: Mutex::new(None),
            events: Mutex::new(Vec::new()),
        }
    }

    pub(crate) async fn new_page(&self, url: String) -> Result<Arc<Page>, XcelerateError> {
        let page = Arc::clone(&self.browser).new_page(url).await?;
        let scripts = self.scripts.lock().await.clone();
        for script in scripts {
            page.add_script_to_evaluate_on_new_document(script).await?;
        }
        *self.page.lock().await = Some(Arc::clone(&page));
        Ok(page)
    }

    pub(crate) async fn add_init_script(&self, script: String) {
        self.scripts.lock().await.push(script);
    }

    /// Returns a working page for the context, creating one on first use.
    pub(crate) async fn working_page(&self) -> Result<Arc<Page>, XcelerateError> {
        if let Some(page) = self.page.lock().await.clone() {
            return Ok(page);
        }
        self.new_page("about:blank".to_string()).await
    }

    pub(crate) async fn pages(&self) -> Result<String, XcelerateError> {
        self.browser.targets().await
    }

    pub(crate) async fn cookies(&self) -> Result<String, XcelerateError> {
        self.working_page().await?.cookies().await
    }

    pub(crate) async fn add_cookies(&self, cookies_json: String) -> Result<(), XcelerateError> {
        let page = self.working_page().await?;
        let cookies: Vec<serde_json::Value> =
            serde_json::from_str(&cookies_json).unwrap_or_default();
        for cookie in cookies {
            page.execute_cdp_cmd("Network.setCookie".to_string(), cookie.to_string())
                .await?;
        }
        Ok(())
    }

    pub(crate) async fn clear_cookies(&self) -> Result<(), XcelerateError> {
        self.working_page()
            .await?
            .execute_cdp_cmd("Network.clearBrowserCookies".to_string(), "{}".to_string())
            .await?;
        Ok(())
    }

    pub(crate) async fn set_extra_http_headers(
        &self,
        headers_json: String,
    ) -> Result<(), XcelerateError> {
        self.working_page()
            .await?
            .set_extra_http_headers(headers_json)
            .await
    }

    pub(crate) async fn grant_permissions(
        &self,
        origin: String,
        permissions_json: String,
    ) -> Result<(), XcelerateError> {
        self.browser
            .grant_permissions(origin, permissions_json)
            .await
    }

    pub(crate) async fn clear_permissions(&self) -> Result<(), XcelerateError> {
        self.browser.reset_permissions().await
    }

    pub(crate) async fn set_offline(&self, offline: bool) -> Result<(), XcelerateError> {
        self.working_page().await?.set_offline(offline).await
    }

    pub(crate) async fn set_geolocation(
        &self,
        latitude: f64,
        longitude: f64,
        accuracy: f64,
    ) -> Result<(), XcelerateError> {
        self.working_page()
            .await?
            .execute_cdp_cmd(
                "Emulation.setGeolocationOverride".to_string(),
                serde_json::json!({
                    "latitude": latitude,
                    "longitude": longitude,
                    "accuracy": accuracy
                })
                .to_string(),
            )
            .await
            .map(|_| ())
    }

    pub(crate) async fn storage_state(&self) -> Result<String, XcelerateError> {
        self.working_page().await?.storage_state().await
    }

    pub(crate) async fn set_storage_state(&self, state_json: String) -> Result<(), XcelerateError> {
        self.working_page()
            .await?
            .set_storage_state(state_json)
            .await
    }

    pub(crate) async fn set_default_timeout(
        &self,
        milliseconds: f64,
    ) -> Result<(), XcelerateError> {
        self.working_page()
            .await?
            .set_default_timeout(milliseconds)
            .await
    }

    pub(crate) async fn targets(&self) -> Result<String, XcelerateError> {
        self.browser.targets().await
    }

    pub(crate) async fn route(
        &self,
        pattern: String,
        action: String,
    ) -> Result<(), XcelerateError> {
        self.working_page()
            .await?
            .route(pattern, action, None, None)
            .await
    }

    pub(crate) async fn route_abort(&self, pattern: String) -> Result<(), XcelerateError> {
        self.working_page().await?.route_abort(pattern).await
    }

    pub(crate) async fn unroute(&self, pattern: String) -> Result<(), XcelerateError> {
        self.working_page().await?.unroute(pattern).await
    }

    pub(crate) async fn unroute_all(&self) -> Result<(), XcelerateError> {
        self.working_page().await?.unroute_all().await
    }

    pub(crate) async fn set_cookie(&self, cookie_json: String) -> Result<(), XcelerateError> {
        self.working_page()
            .await?
            .execute_cdp_cmd("Network.setCookie".to_string(), cookie_json)
            .await
            .map(|_| ())
    }

    pub(crate) async fn delete_cookie(&self, name: String) -> Result<(), XcelerateError> {
        self.working_page()
            .await?
            .execute_cdp_cmd(
                "Network.deleteCookies".to_string(),
                serde_json::json!({ "name": name }).to_string(),
            )
            .await
            .map(|_| ())
    }

    pub(crate) async fn on(&self, event_name: String) {
        let mut events = self.events.lock().await;
        if !events.contains(&event_name) {
            events.push(event_name);
        }
    }

    pub(crate) async fn once(&self, event_name: String) {
        self.on(event_name).await;
    }

    pub(crate) async fn remove_listener(&self, event_name: String) {
        self.events.lock().await.retain(|name| name != &event_name);
    }

    pub(crate) async fn remove_all_listeners(&self) {
        self.events.lock().await.clear();
    }

    pub(crate) async fn event_names(&self) -> Vec<String> {
        self.events.lock().await.clone()
    }

    pub(crate) async fn listens_to(&self, event_name: String) -> bool {
        self.events.lock().await.contains(&event_name)
    }

    pub(crate) async fn wait_for_event(
        &self,
        event_name: String,
    ) -> Result<String, XcelerateError> {
        self.working_page()
            .await?
            .wait_for_event_default(event_name)
            .await
    }

    pub(crate) async fn set_download_behavior(&self, path: String) -> Result<(), XcelerateError> {
        self.browser
            .client
            .execute_raw(
                "Browser.setDownloadBehavior",
                serde_json::json!({ "behavior": "allow", "downloadPath": path }),
            )
            .await
            .map(|_| ())
            .map_err(XcelerateError::from)
    }

    pub(crate) async fn route_from_har(&self, path: String) -> Result<(), XcelerateError> {
        self.working_page().await?.route_from_har(path).await
    }
}

/// Capture a screenshot, optionally writing it to `path`.
pub(crate) async fn screenshot(
    page: &Page,
    full_page: bool,
    path: Option<String>,
) -> Result<Vec<u8>, XcelerateError> {
    let data = if full_page {
        page.screenshot_full().await?
    } else {
        page.screenshot().await?
    };
    if let Some(path) = path {
        tokio::fs::write(path, &data)
            .await
            .map_err(|e| XcelerateError::NotFound(format!("failed to write screenshot: {e}")))?;
    }
    Ok(data)
}

/// Capture a screenshot and base64-encode it (Selenium compatibility).
pub(crate) async fn screenshot_base64(
    page: &Page,
    full_page: bool,
) -> Result<String, XcelerateError> {
    use base64::{Engine as _, engine::general_purpose};
    let data = screenshot(page, full_page, None).await?;
    Ok(general_purpose::STANDARD.encode(data))
}

/// Build an attribute selector, escaping the value.
pub(crate) fn attr_selector(attribute: &str, value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("[{attribute}=\"{escaped}\"]")
}

/// Normalise a Selenium-style `(By, value)` selector into a CSS selector.
pub(crate) fn resolve_selector(by: &str, value: Option<&str>) -> Result<String, XcelerateError> {
    let value =
        value.ok_or_else(|| XcelerateError::NotFound("selector value is required".to_string()))?;
    if by.to_ascii_lowercase().contains("xpath") {
        return Err(XcelerateError::Unsupported(
            "XPath selectors are not supported".to_string(),
        ));
    }
    Ok(value.to_string())
}
