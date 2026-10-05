#![allow(unused_imports)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(clippy::needless_question_mark)]
//! Generated adapter: Puppeteer (promise API). DO NOT EDIT.
//!
//! Regenerate with: python scripts/generate_adapters.py --target rust

use std::sync::Arc;

use crate::{
    Browser as CoreBrowser, BrowserConfig, Element as CoreElement, Page as CorePage, XcelerateError,
};

/// launch entry point.
pub async fn launch(config: Option<BrowserConfig>) -> Result<Browser, XcelerateError> {
    Ok(Browser::new(
        CoreBrowser::launch(config.unwrap_or_default()).await?,
    ))
}

/// Puppeteer-style browser.
pub struct Browser {
    inner: Arc<CoreBrowser>,
}

impl Browser {
    /// Wrap an existing xcelerate object.
    pub fn new(inner: Arc<CoreBrowser>) -> Self {
        Self { inner }
    }

    /// `newPage`.
    pub async fn newPage(&self, url: String) -> Result<Page, XcelerateError> {
        Ok(Page::new(Arc::clone(&self.inner).new_page(url).await?))
    }

    /// `createBrowserContext`.
    pub async fn createBrowserContext(&self) -> Result<BrowserContext, XcelerateError> {
        Ok(BrowserContext::new(super::support::ContextHandle::new(
            Arc::clone(&self.inner),
        )))
    }

    /// `version`.
    pub async fn version(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.version().await?)
    }

    /// `close`.
    pub async fn close(&self) -> Result<(), XcelerateError> {
        self.inner.close().await?;
        Ok(())
    }

    /// `newIncognitoBrowserContext`.
    pub async fn newIncognitoBrowserContext(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.new_context().await?)
    }

    /// `browserContexts`.
    pub async fn browserContexts(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.browser_contexts().await?)
    }

    /// `pages`.
    pub async fn pages(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.targets().await?)
    }

    /// `targets`.
    pub async fn targets(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.targets().await?)
    }

    /// `waitForTarget`.
    pub async fn waitForTarget(&self) -> Result<String, XcelerateError> {
        Ok(self
            .inner
            .wait_for_event_default("Target.targetCreated".to_string())
            .await?)
    }

    /// `userAgent`.
    pub async fn userAgent(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.user_agent().await?)
    }

    /// `wsEndpoint`.
    pub async fn wsEndpoint(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.ws_endpoint())
    }

    /// `isConnected`.
    pub async fn isConnected(&self) -> Result<bool, XcelerateError> {
        Ok(self.inner.is_connected().await)
    }

    /// `startTracing`.
    pub async fn startTracing(&self) -> Result<(), XcelerateError> {
        self.inner.start_tracing().await?;
        Ok(())
    }

    /// `stopTracing`.
    pub async fn stopTracing(&self) -> Result<(), XcelerateError> {
        self.inner.stop_tracing().await?;
        Ok(())
    }

    /// `add_listener`.
    pub async fn add_listener(&self, event_name: String) -> Result<(), XcelerateError> {
        self.inner.on(event_name).await;
        Ok(())
    }

    /// `createIncogniteBrowserContext`.
    pub async fn createIncogniteBrowserContext(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.new_context().await?)
    }

    /// `createIncognitoBrowserContext`.
    pub async fn createIncognitoBrowserContext(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.new_context().await?)
    }

    /// `event_names`.
    pub async fn event_names(&self) -> Result<Vec<String>, XcelerateError> {
        Ok(self.inner.event_names().await)
    }

    /// `listeners`.
    pub async fn listeners(&self) -> Result<Vec<String>, XcelerateError> {
        Ok(self.inner.event_names().await)
    }

    /// `listens_to`.
    pub async fn listens_to(&self, event_name: String) -> Result<bool, XcelerateError> {
        Ok(self.inner.listens_to(event_name).await)
    }

    /// `on`.
    pub async fn on(&self, event_name: String) -> Result<(), XcelerateError> {
        self.inner.on(event_name).await;
        Ok(())
    }

    /// `once`.
    pub async fn once(&self, event_name: String) -> Result<(), XcelerateError> {
        self.inner.once(event_name).await;
        Ok(())
    }

    /// `remove_all_listeners`.
    pub async fn remove_all_listeners(&self) -> Result<(), XcelerateError> {
        self.inner.remove_all_listeners().await;
        Ok(())
    }

    /// `remove_listener`.
    pub async fn remove_listener(&self, event_name: String) -> Result<(), XcelerateError> {
        self.inner.remove_listener(event_name).await;
        Ok(())
    }

    /// `connected`.
    pub async fn connected(&self) -> Result<bool, XcelerateError> {
        Ok(self.inner.is_connected().await)
    }

    /// `cookies`.
    pub async fn cookies(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.cookies().await?)
    }

    /// `deleteCookie`.
    pub async fn deleteCookie(&self, name: String) -> Result<(), XcelerateError> {
        self.inner.delete_cookie(name).await?;
        Ok(())
    }

    /// `deleteMatchingCookies`.
    pub async fn deleteMatchingCookies(&self, name: String) -> Result<(), XcelerateError> {
        self.inner.delete_cookie(name).await?;
        Ok(())
    }

    /// `setCookie`.
    pub async fn setCookie(&self, cookie_json: String) -> Result<(), XcelerateError> {
        self.inner.set_cookie(cookie_json).await?;
        Ok(())
    }

    /// `setPermission`.
    pub async fn setPermission(
        &self,
        origin: String,
        permissions_json: String,
    ) -> Result<(), XcelerateError> {
        self.inner
            .grant_permissions(origin, permissions_json)
            .await?;
        Ok(())
    }
}

/// Puppeteer-style browser context (modelled as browser + init scripts).
pub struct BrowserContext {
    inner: super::support::ContextHandle,
}

impl BrowserContext {
    /// Wrap an existing xcelerate object.
    pub fn new(inner: super::support::ContextHandle) -> Self {
        Self { inner }
    }

    /// `newPage`.
    pub async fn newPage(&self, url: String) -> Result<Page, XcelerateError> {
        Ok(Page::new(self.inner.new_page(url).await?))
    }

    /// `close`.
    pub async fn close(&self) -> Result<(), XcelerateError> {
        Ok(())
    }

    /// `pages`.
    pub async fn pages(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.pages().await?)
    }

    /// `targets`.
    pub async fn targets(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.targets().await?)
    }

    /// `newCDPSession`.
    pub async fn newCDPSession(&self) -> Result<CDPSession, XcelerateError> {
        Ok(CDPSession::new(self.inner.working_page().await?))
    }

    /// `overridePermissions`.
    pub async fn overridePermissions(
        &self,
        origin: String,
        permissions_json: String,
    ) -> Result<(), XcelerateError> {
        self.inner
            .grant_permissions(origin, permissions_json)
            .await?;
        Ok(())
    }

    /// `clearPermissionOverrides`.
    pub async fn clearPermissionOverrides(&self) -> Result<(), XcelerateError> {
        self.inner.clear_permissions().await?;
        Ok(())
    }

    /// `setCookie`.
    pub async fn setCookie(&self, cookie_json: String) -> Result<(), XcelerateError> {
        self.inner.set_cookie(cookie_json).await?;
        Ok(())
    }

    /// `deleteCookie`.
    pub async fn deleteCookie(&self, name: String) -> Result<(), XcelerateError> {
        self.inner.delete_cookie(name).await?;
        Ok(())
    }

    /// `cookies`.
    pub async fn cookies(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.cookies().await?)
    }

    /// `waitForTarget`.
    pub async fn waitForTarget(&self) -> Result<String, XcelerateError> {
        Ok(self
            .inner
            .wait_for_event("Target.targetCreated".to_string())
            .await?)
    }

    /// `addInitScript`.
    pub async fn addInitScript(&self, script: String) -> Result<(), XcelerateError> {
        self.inner.add_init_script(script).await;
        Ok(())
    }

    /// `deleteMatchingCookies`.
    pub async fn deleteMatchingCookies(&self, name: String) -> Result<(), XcelerateError> {
        self.inner.delete_cookie(name).await?;
        Ok(())
    }

    /// `setDownloadBehavior`.
    pub async fn setDownloadBehavior(&self, path: String) -> Result<(), XcelerateError> {
        self.inner.set_download_behavior(path).await?;
        Ok(())
    }

    /// `setPermission`.
    pub async fn setPermission(
        &self,
        origin: String,
        permissions_json: String,
    ) -> Result<(), XcelerateError> {
        self.inner
            .grant_permissions(origin, permissions_json)
            .await?;
        Ok(())
    }
}

/// Puppeteer-style page.
pub struct Page {
    inner: Arc<CorePage>,
}

impl Page {
    /// Wrap an existing xcelerate object.
    pub fn new(inner: Arc<CorePage>) -> Self {
        Self { inner }
    }

    /// `goto`.
    pub async fn goto(&self, url: String) -> Result<(), XcelerateError> {
        self.inner.navigate(url).await?;
        Ok(())
    }

    /// `goBack`.
    pub async fn goBack(&self) -> Result<(), XcelerateError> {
        self.inner.go_back().await?;
        Ok(())
    }

    /// `reload`.
    pub async fn reload(&self) -> Result<(), XcelerateError> {
        self.inner.reload().await?;
        Ok(())
    }

    /// `title`.
    pub async fn title(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.title().await?)
    }

    /// `content`.
    pub async fn content(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.content().await?)
    }

    /// `pdf`.
    pub async fn pdf(&self) -> Result<Vec<u8>, XcelerateError> {
        Ok(self.inner.pdf().await?)
    }

    /// `screenshot`.
    pub async fn screenshot(
        &self,
        full_page: bool,
        path: Option<String>,
    ) -> Result<Vec<u8>, XcelerateError> {
        Ok(super::support::screenshot(&self.inner, full_page, path).await?)
    }

    /// `$`.
    pub async fn query_selector(&self, selector: String) -> Result<ElementHandle, XcelerateError> {
        Ok(ElementHandle::new(
            Arc::clone(&self.inner).find_element(selector).await?,
        ))
    }

    /// `click`.
    pub async fn click(&self, selector: String) -> Result<(), XcelerateError> {
        let __e = Arc::clone(&self.inner).find_element(selector).await?;
        Arc::clone(&__e).click().await?;
        Ok(())
    }

    /// `type`.
    pub async fn r#type(&self, selector: String, value: String) -> Result<(), XcelerateError> {
        let __e = Arc::clone(&self.inner).find_element(selector).await?;
        Arc::clone(&__e).focus().await?;
        Arc::clone(&__e).type_text(value).await?;
        Ok(())
    }

    /// `hover`.
    pub async fn hover(&self, selector: String) -> Result<(), XcelerateError> {
        let __e = Arc::clone(&self.inner).find_element(selector).await?;
        Arc::clone(&__e).hover().await?;
        Ok(())
    }

    /// `focus`.
    pub async fn focus(&self, selector: String) -> Result<(), XcelerateError> {
        let __e = Arc::clone(&self.inner).find_element(selector).await?;
        Arc::clone(&__e).focus().await?;
        Ok(())
    }

    /// `waitForSelector`.
    pub async fn waitForSelector(&self, selector: String) -> Result<ElementHandle, XcelerateError> {
        Ok(ElementHandle::new(
            Arc::clone(&self.inner).wait_for_selector(selector).await?,
        ))
    }

    /// `waitForNavigation`.
    pub async fn waitForNavigation(&self) -> Result<(), XcelerateError> {
        self.inner.wait_for_navigation().await?;
        Ok(())
    }

    /// `waitForTimeout`.
    pub async fn waitForTimeout(&self, milliseconds: f64) -> Result<(), XcelerateError> {
        tokio::time::sleep(std::time::Duration::from_millis(milliseconds as u64)).await;
        Ok(())
    }

    /// `evaluateOnNewDocument`.
    pub async fn evaluateOnNewDocument(&self, content: String) -> Result<String, XcelerateError> {
        Ok(self
            .inner
            .add_script_to_evaluate_on_new_document(content)
            .await?)
    }

    /// `addScriptTag`.
    pub async fn addScriptTag(&self, content: String) -> Result<String, XcelerateError> {
        Ok(self
            .inner
            .add_script_to_evaluate_on_new_document(content)
            .await?)
    }

    /// `goForward`.
    pub async fn goForward(&self) -> Result<(), XcelerateError> {
        self.inner.go_forward().await?;
        Ok(())
    }

    /// `close`.
    pub async fn close(&self) -> Result<(), XcelerateError> {
        self.inner.close().await?;
        Ok(())
    }

    /// `setContent`.
    pub async fn setContent(&self, html: String) -> Result<(), XcelerateError> {
        self.inner
            .call_json(
                "function(h){document.open();document.write(h);document.close();}".to_string(),
                serde_json::json!([html]).to_string(),
            )
            .await?;
        Ok(())
    }

    /// `url`.
    pub async fn url(&self) -> Result<String, XcelerateError> {
        Ok(self
            .inner
            .call_string(
                "function(){return location.href;}".to_string(),
                "[]".to_string(),
            )
            .await?)
    }

    /// `$$`.
    pub async fn query_selector_all(
        &self,
        selector: String,
    ) -> Result<Vec<ElementHandle>, XcelerateError> {
        Ok(Arc::clone(&self.inner)
            .query_selector_all(selector)
            .await?
            .into_iter()
            .map(ElementHandle::new)
            .collect())
    }

    /// `$eval`.
    pub async fn eval_on_selector(
        &self,
        selector: String,
        expression: String,
    ) -> Result<String, XcelerateError> {
        Ok(self.inner.call_on_selector(selector, expression).await?)
    }

    /// `$$eval`.
    pub async fn eval_on_selector_all(
        &self,
        selector: String,
        expression: String,
    ) -> Result<String, XcelerateError> {
        Ok(self
            .inner
            .call_on_selector_all(selector, expression)
            .await?)
    }

    /// `$x`.
    pub async fn query_selector_xpath(
        &self,
        xpath: String,
    ) -> Result<ElementHandle, XcelerateError> {
        Ok(ElementHandle::new(
            Arc::clone(&self.inner).query_selector_xpath(xpath).await?,
        ))
    }

    /// `waitForXPath`.
    pub async fn waitForXPath(&self, xpath: String) -> Result<ElementHandle, XcelerateError> {
        Ok(ElementHandle::new(
            Arc::clone(&self.inner)
                .wait_for_xpath(xpath, 30_000)
                .await?,
        ))
    }

    /// `waitForFunction`.
    pub async fn waitForFunction(&self, expression: String) -> Result<(), XcelerateError> {
        self.inner.wait_for_function(expression, 30_000).await?;
        Ok(())
    }

    /// `waitForRequest`.
    pub async fn waitForRequest(&self) -> Result<String, XcelerateError> {
        Ok(self
            .inner
            .wait_for_event_default("Network.requestWillBeSent".to_string())
            .await?)
    }

    /// `waitForResponse`.
    pub async fn waitForResponse(&self) -> Result<String, XcelerateError> {
        Ok(self
            .inner
            .wait_for_event_default("Network.responseReceived".to_string())
            .await?)
    }

    /// `waitForNetworkIdle`.
    pub async fn waitForNetworkIdle(&self) -> Result<String, XcelerateError> {
        Ok(self
            .inner
            .wait_for_event_default("Network.loadingFinished".to_string())
            .await?)
    }

    /// `waitForFrame`.
    pub async fn waitForFrame(&self) -> Result<String, XcelerateError> {
        Ok(self
            .inner
            .wait_for_event_default("Page.frameAttached".to_string())
            .await?)
    }

    /// `tap`.
    pub async fn tap(&self, selector: String) -> Result<(), XcelerateError> {
        self.inner
            .call_json(
                "function(sel){const e=document.querySelector(sel);if(e)e.click();}".to_string(),
                serde_json::json!([selector]).to_string(),
            )
            .await?;
        Ok(())
    }

    /// `select`.
    pub async fn select(
        &self,
        selector: String,
        values_json: String,
    ) -> Result<(), XcelerateError> {
        self.inner.call_json("function(sel,valuesRaw){const e=document.querySelector(sel);if(!e)return;const w=JSON.parse(valuesRaw).map(String);for(const o of e.options){o.selected=w.includes(o.value)||w.includes(o.text);}e.dispatchEvent(new Event('change',{bubbles:true}));}".to_string(), serde_json::json!([selector, values_json]).to_string()).await?;
        Ok(())
    }

    /// `setInputFiles`.
    pub async fn setInputFiles(
        &self,
        selector: String,
        files_json: String,
    ) -> Result<(), XcelerateError> {
        Arc::clone(&self.inner)
            .set_input_files(selector, files_json)
            .await?;
        Ok(())
    }

    /// `uploadFile`.
    pub async fn uploadFile(
        &self,
        selector: String,
        files_json: String,
    ) -> Result<(), XcelerateError> {
        Arc::clone(&self.inner)
            .set_input_files(selector, files_json)
            .await?;
        Ok(())
    }

    /// `keyboard`.
    pub async fn keyboard(&self) -> Result<Keyboard, XcelerateError> {
        Ok(Keyboard::new(Arc::clone(&self.inner)))
    }

    /// `mouse`.
    pub async fn mouse(&self) -> Result<Mouse, XcelerateError> {
        Ok(Mouse::new(Arc::clone(&self.inner)))
    }

    /// `touchscreen`.
    pub async fn touchscreen(&self) -> Result<Touchscreen, XcelerateError> {
        Ok(Touchscreen::new(Arc::clone(&self.inner)))
    }

    /// `evaluate`.
    pub async fn evaluate(&self, expression: String) -> Result<String, XcelerateError> {
        Ok(self
            .inner
            .call_json(
                "function(src){return (new Function(\"return (\"+src+\")\"))();}".to_string(),
                serde_json::json!([expression]).to_string(),
            )
            .await?)
    }

    /// `evaluateHandle`.
    pub async fn evaluateHandle(
        &self,
        expression: String,
    ) -> Result<ElementHandle, XcelerateError> {
        Ok(ElementHandle::new(
            Arc::clone(&self.inner).evaluate_handle(expression).await?,
        ))
    }

    /// `addStyleTag`.
    pub async fn addStyleTag(&self, content: String) -> Result<String, XcelerateError> {
        Ok(self.inner.call_string("function(c){const s=document.createElement('style');s.textContent=c;document.head.appendChild(s);return s.textContent;}".to_string(), serde_json::json!([content]).to_string()).await?)
    }

    /// `setViewport`.
    pub async fn setViewport(&self, width: u64, height: i64) -> Result<(), XcelerateError> {
        self.inner.execute_cdp_cmd("Emulation.setDeviceMetricsOverride".to_string(), serde_json::json!({ "width": width, "height": height, "deviceScaleFactor": 1, "mobile": false }).to_string()).await?;
        Ok(())
    }

    /// `setUserAgent`.
    pub async fn setUserAgent(
        &self,
        user_agent: String,
        accept_language: Option<String>,
    ) -> Result<(), XcelerateError> {
        self.inner
            .set_user_agent(user_agent, accept_language)
            .await?;
        Ok(())
    }

    /// `setExtraHTTPHeaders`.
    pub async fn setExtraHTTPHeaders(&self, headers_json: String) -> Result<(), XcelerateError> {
        self.inner.set_extra_http_headers(headers_json).await?;
        Ok(())
    }

    /// `setBypassCSP`.
    pub async fn setBypassCSP(&self, enabled: bool) -> Result<(), XcelerateError> {
        self.inner
            .execute_cdp_cmd(
                "Page.setBypassCSP".to_string(),
                serde_json::json!({ "enabled": enabled }).to_string(),
            )
            .await?;
        Ok(())
    }

    /// `setCacheEnabled`.
    pub async fn setCacheEnabled(&self, enabled: bool) -> Result<(), XcelerateError> {
        self.inner.set_cache_enabled(enabled).await?;
        Ok(())
    }

    /// `setJavaScriptEnabled`.
    pub async fn setJavaScriptEnabled(&self, enabled: bool) -> Result<(), XcelerateError> {
        self.inner.set_javascript_enabled(enabled).await?;
        Ok(())
    }

    /// `setOfflineMode`.
    pub async fn setOfflineMode(&self, offline: bool) -> Result<(), XcelerateError> {
        self.inner.set_offline(offline).await?;
        Ok(())
    }

    /// `emulateNetworkConditions`.
    pub async fn emulateNetworkConditions(
        &self,
        offline: bool,
        latency: f64,
        download: f64,
        upload: f64,
    ) -> Result<(), XcelerateError> {
        self.inner.execute_cdp_cmd("Network.emulateNetworkConditions".to_string(), serde_json::json!({ "offline": offline, "latency": latency, "downloadThroughput": download, "uploadThroughput": upload }).to_string()).await?;
        Ok(())
    }

    /// `emulateCPUThrottling`.
    pub async fn emulateCPUThrottling(&self, rate: f64) -> Result<(), XcelerateError> {
        self.inner
            .execute_cdp_cmd(
                "Emulation.setCPUThrottlingRate".to_string(),
                serde_json::json!({ "rate": rate }).to_string(),
            )
            .await?;
        Ok(())
    }

    /// `emulateIdleState`.
    pub async fn emulateIdleState(
        &self,
        is_user_active: bool,
        is_screen_unlocked: bool,
    ) -> Result<(), XcelerateError> {
        self.inner
            .emulate_idle_state(is_user_active, is_screen_unlocked)
            .await?;
        Ok(())
    }

    /// `emulateMediaFeatures`.
    pub async fn emulateMediaFeatures(&self, features_json: String) -> Result<(), XcelerateError> {
        self.inner
            .set_emulated_media_features(features_json)
            .await?;
        Ok(())
    }

    /// `emulateTimezone`.
    pub async fn emulateTimezone(&self, timezone_id: String) -> Result<(), XcelerateError> {
        self.inner
            .execute_cdp_cmd(
                "Emulation.setTimezoneOverride".to_string(),
                serde_json::json!({ "timezoneId": timezone_id }).to_string(),
            )
            .await?;
        Ok(())
    }

    /// `emulateVisionDeficiency`.
    pub async fn emulateVisionDeficiency(&self, kind: String) -> Result<(), XcelerateError> {
        self.inner
            .execute_cdp_cmd(
                "Emulation.setEmulatedVisionDeficiency".to_string(),
                serde_json::json!({ "type": kind }).to_string(),
            )
            .await?;
        Ok(())
    }

    /// `setGeolocation`.
    pub async fn setGeolocation(
        &self,
        latitude: f64,
        longitude: f64,
        accuracy: f64,
    ) -> Result<(), XcelerateError> {
        self.inner.execute_cdp_cmd("Emulation.setGeolocationOverride".to_string(), serde_json::json!({ "latitude": latitude, "longitude": longitude, "accuracy": accuracy }).to_string()).await?;
        Ok(())
    }

    /// `setRequestInterception`.
    pub async fn setRequestInterception(&self, enabled: bool) -> Result<(), XcelerateError> {
        self.inner.set_request_interception(enabled).await?;
        Ok(())
    }

    /// `setDragInterception`.
    pub async fn setDragInterception(&self, enabled: bool) -> Result<(), XcelerateError> {
        self.inner.set_drag_interception(enabled).await?;
        Ok(())
    }

    /// `isDragInterceptionEnabled`.
    pub async fn isDragInterceptionEnabled(&self) -> Result<bool, XcelerateError> {
        Ok(self.inner.is_drag_interception_enabled().await)
    }

    /// `authenticate`.
    pub async fn authenticate(
        &self,
        username: String,
        password: String,
    ) -> Result<(), XcelerateError> {
        self.inner.authenticate(username, password).await?;
        Ok(())
    }

    /// `setCookie`.
    pub async fn setCookie(
        &self,
        name: String,
        value: String,
        url: String,
    ) -> Result<(), XcelerateError> {
        self.inner
            .execute_cdp_cmd(
                "Network.setCookie".to_string(),
                serde_json::json!({ "name": name, "value": value, "url": url }).to_string(),
            )
            .await?;
        Ok(())
    }

    /// `deleteCookie`.
    pub async fn deleteCookie(&self, name: String) -> Result<(), XcelerateError> {
        self.inner
            .execute_cdp_cmd(
                "Network.deleteCookies".to_string(),
                serde_json::json!({ "name": name }).to_string(),
            )
            .await?;
        Ok(())
    }

    /// `cookies`.
    pub async fn cookies(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.cookies().await?)
    }

    /// `metrics`.
    pub async fn metrics(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.metrics().await?)
    }

    /// `bringToFront`.
    pub async fn bringToFront(&self) -> Result<(), XcelerateError> {
        self.inner.bring_to_front().await?;
        Ok(())
    }

    /// `mainFrame`.
    pub async fn mainFrame(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.main_frame().await?)
    }

    /// `frames`.
    pub async fn frames(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.frames().await?)
    }

    /// `J`.
    pub async fn J(&self, selector: String) -> Result<ElementHandle, XcelerateError> {
        Ok(ElementHandle::new(
            Arc::clone(&self.inner).find_element(selector).await?,
        ))
    }

    /// `JJ`.
    pub async fn JJ(&self, selector: String) -> Result<Vec<ElementHandle>, XcelerateError> {
        Ok(Arc::clone(&self.inner)
            .query_selector_all(selector)
            .await?
            .into_iter()
            .map(ElementHandle::new)
            .collect())
    }

    /// `JJeval`.
    pub async fn JJeval(
        &self,
        selector: String,
        expression: String,
    ) -> Result<String, XcelerateError> {
        Ok(self
            .inner
            .call_on_selector_all(selector, expression)
            .await?)
    }

    /// `Jeval`.
    pub async fn Jeval(
        &self,
        selector: String,
        expression: String,
    ) -> Result<String, XcelerateError> {
        Ok(self.inner.call_on_selector(selector, expression).await?)
    }

    /// `Jx`.
    pub async fn Jx(&self, xpath: String) -> Result<ElementHandle, XcelerateError> {
        Ok(ElementHandle::new(
            Arc::clone(&self.inner).query_selector_xpath(xpath).await?,
        ))
    }

    /// `add_listener`.
    pub async fn add_listener(&self, event_name: String) -> Result<(), XcelerateError> {
        self.inner.on(event_name).await;
        Ok(())
    }

    /// `coverage`.
    pub async fn coverage(&self) -> Result<Coverage, XcelerateError> {
        Ok(Coverage::new(Arc::clone(&self.inner)))
    }

    /// `emulateMedia`.
    pub async fn emulateMedia(
        &self,
        media: Option<String>,
        color_scheme: Option<String>,
    ) -> Result<(), XcelerateError> {
        self.inner.emulate_media(media, color_scheme).await?;
        Ok(())
    }

    /// `event_names`.
    pub async fn event_names(&self) -> Result<Vec<String>, XcelerateError> {
        Ok(self.inner.event_names().await)
    }

    /// `injectFile`.
    pub async fn injectFile(&self, path: String) -> Result<String, XcelerateError> {
        Ok(self.inner.inject_file(path).await?)
    }

    /// `isClosed`.
    pub async fn isClosed(&self) -> Result<bool, XcelerateError> {
        Ok(false)
    }

    /// `listeners`.
    pub async fn listeners(&self) -> Result<Vec<String>, XcelerateError> {
        Ok(self.inner.event_names().await)
    }

    /// `listens_to`.
    pub async fn listens_to(&self, event_name: String) -> Result<bool, XcelerateError> {
        Ok(self.inner.listens_to(event_name).await)
    }

    /// `on`.
    pub async fn on(&self, event_name: String) -> Result<(), XcelerateError> {
        self.inner.on(event_name).await;
        Ok(())
    }

    /// `once`.
    pub async fn once(&self, event_name: String) -> Result<(), XcelerateError> {
        self.inner.once(event_name).await;
        Ok(())
    }

    /// `plainText`.
    pub async fn plainText(&self) -> Result<String, XcelerateError> {
        Ok(self
            .inner
            .call_string(
                "function(){return document.body?document.body.innerText:'';}".to_string(),
                "[]".to_string(),
            )
            .await?)
    }

    /// `querySelector`.
    pub async fn querySelector(&self, selector: String) -> Result<ElementHandle, XcelerateError> {
        Ok(ElementHandle::new(
            Arc::clone(&self.inner).find_element(selector).await?,
        ))
    }

    /// `querySelectorAll`.
    pub async fn querySelectorAll(
        &self,
        selector: String,
    ) -> Result<Vec<ElementHandle>, XcelerateError> {
        Ok(Arc::clone(&self.inner)
            .query_selector_all(selector)
            .await?
            .into_iter()
            .map(ElementHandle::new)
            .collect())
    }

    /// `querySelectorAllEval`.
    pub async fn querySelectorAllEval(
        &self,
        selector: String,
        expression: String,
    ) -> Result<String, XcelerateError> {
        Ok(self
            .inner
            .call_on_selector_all(selector, expression)
            .await?)
    }

    /// `querySelectorEval`.
    pub async fn querySelectorEval(
        &self,
        selector: String,
        expression: String,
    ) -> Result<String, XcelerateError> {
        Ok(self.inner.call_on_selector(selector, expression).await?)
    }

    /// `remove_all_listeners`.
    pub async fn remove_all_listeners(&self) -> Result<(), XcelerateError> {
        self.inner.remove_all_listeners().await;
        Ok(())
    }

    /// `remove_listener`.
    pub async fn remove_listener(&self, event_name: String) -> Result<(), XcelerateError> {
        self.inner.remove_listener(event_name).await;
        Ok(())
    }

    /// `setDefaultNavigationTimeout`.
    pub async fn setDefaultNavigationTimeout(
        &self,
        milliseconds: f64,
    ) -> Result<(), XcelerateError> {
        self.inner.set_default_timeout(milliseconds).await?;
        Ok(())
    }

    /// `tracing`.
    pub async fn tracing(&self) -> Result<Tracing, XcelerateError> {
        Ok(Tracing::new(Arc::clone(&self.inner)))
    }

    /// `viewport`.
    pub async fn viewport(&self) -> Result<String, XcelerateError> {
        Ok(self
            .inner
            .call_json(
                "function(){return {width:window.innerWidth,height:window.innerHeight};}"
                    .to_string(),
                "[]".to_string(),
            )
            .await?)
    }

    /// `xpath`.
    pub async fn xpath(&self, xpath: String) -> Result<ElementHandle, XcelerateError> {
        Ok(ElementHandle::new(
            Arc::clone(&self.inner).query_selector_xpath(xpath).await?,
        ))
    }

    /// `accessibility`.
    pub async fn accessibility(&self) -> Result<String, XcelerateError> {
        Ok(self
            .inner
            .execute_cdp_cmd(
                "Accessibility.getFullAXTree".to_string(),
                serde_json::json!({}).to_string(),
            )
            .await?)
    }

    /// `createCDPSession`.
    pub async fn createCDPSession(&self) -> Result<CDPSession, XcelerateError> {
        Ok(CDPSession::new(Arc::clone(&self.inner)))
    }

    /// `createPDFStream`.
    pub async fn createPDFStream(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.create_pdf_stream().await?)
    }

    /// `emulateFocusedPage`.
    pub async fn emulateFocusedPage(&self, enabled: bool) -> Result<(), XcelerateError> {
        self.inner
            .execute_cdp_cmd(
                "Emulation.setFocusEmulationEnabled".to_string(),
                serde_json::json!({ "enabled": enabled }).to_string(),
            )
            .await?;
        Ok(())
    }

    /// `emulateLocale`.
    pub async fn emulateLocale(&self, locale: String) -> Result<(), XcelerateError> {
        self.inner
            .execute_cdp_cmd(
                "Emulation.setLocaleOverride".to_string(),
                serde_json::json!({ "locale": locale }).to_string(),
            )
            .await?;
        Ok(())
    }

    /// `emulateMediaType`.
    pub async fn emulateMediaType(&self, media: String) -> Result<(), XcelerateError> {
        self.inner
            .execute_cdp_cmd(
                "Emulation.setEmulatedMedia".to_string(),
                serde_json::json!({ "media": media }).to_string(),
            )
            .await?;
        Ok(())
    }

    /// `getDefaultNavigationTimeout`.
    pub async fn getDefaultNavigationTimeout(&self) -> Result<f64, XcelerateError> {
        Ok(self.inner.get_default_timeout().await?)
    }

    /// `getDefaultTimeout`.
    pub async fn getDefaultTimeout(&self) -> Result<f64, XcelerateError> {
        Ok(self.inner.get_default_timeout().await?)
    }

    /// `off`.
    pub async fn off(&self, event_name: String) -> Result<(), XcelerateError> {
        self.inner.remove_listener(event_name).await;
        Ok(())
    }

    /// `removeScriptToEvaluateOnNewDocument`.
    pub async fn removeScriptToEvaluateOnNewDocument(
        &self,
        identifier: String,
    ) -> Result<(), XcelerateError> {
        self.inner.remove_script(identifier).await?;
        Ok(())
    }

    /// `screencast`.
    pub async fn screencast(&self) -> Result<Screencast, XcelerateError> {
        Ok(Screencast::new(Arc::clone(&self.inner)))
    }

    /// `setBypassServiceWorker`.
    pub async fn setBypassServiceWorker(&self, bypass: bool) -> Result<(), XcelerateError> {
        self.inner
            .execute_cdp_cmd(
                "Network.setBypassServiceWorker".to_string(),
                serde_json::json!({ "bypass": bypass }).to_string(),
            )
            .await?;
        Ok(())
    }

    /// `setDefaultTimeout`.
    pub async fn setDefaultTimeout(&self, milliseconds: f64) -> Result<(), XcelerateError> {
        self.inner.set_default_timeout(milliseconds).await?;
        Ok(())
    }

    /// `waitForDevicePrompt`.
    pub async fn waitForDevicePrompt(&self) -> Result<String, XcelerateError> {
        Ok(self
            .inner
            .wait_for_event_default("DeviceAccess.deviceRequestPrompted".to_string())
            .await?)
    }

    /// `waitForFileChooser`.
    pub async fn waitForFileChooser(&self) -> Result<String, XcelerateError> {
        Ok(self
            .inner
            .wait_for_event_default("Page.fileChooserOpened".to_string())
            .await?)
    }

    /// `waitForNetworkIdle$`.
    pub async fn waitForNetworkIdle_(&self) -> Result<String, XcelerateError> {
        Ok(self
            .inner
            .wait_for_event_default("Network.loadingFinished".to_string())
            .await?)
    }
}

/// Puppeteer-style element handle.
pub struct ElementHandle {
    inner: Arc<CoreElement>,
}

impl ElementHandle {
    /// Wrap an existing xcelerate object.
    pub fn new(inner: Arc<CoreElement>) -> Self {
        Self { inner }
    }

    /// `click`.
    pub async fn click(&self) -> Result<(), XcelerateError> {
        Arc::clone(&self.inner).click().await?;
        Ok(())
    }

    /// `type`.
    pub async fn r#type(&self, value: String) -> Result<(), XcelerateError> {
        let __e = Arc::clone(&self.inner);
        Arc::clone(&__e).focus().await?;
        Arc::clone(&__e).type_text(value).await?;
        Ok(())
    }

    /// `hover`.
    pub async fn hover(&self) -> Result<(), XcelerateError> {
        Arc::clone(&self.inner).hover().await?;
        Ok(())
    }

    /// `focus`.
    pub async fn focus(&self) -> Result<(), XcelerateError> {
        Arc::clone(&self.inner).focus().await?;
        Ok(())
    }

    /// `evaluate`.
    pub async fn evaluate(&self, expression: String) -> Result<String, XcelerateError> {
        Ok(self
            .inner
            .call_json(
                "function(src){return (new Function('el','return ('+src+')(el);'))(this);}"
                    .to_string(),
                serde_json::json!([expression]).to_string(),
            )
            .await?)
    }

    /// `evaluateHandle`.
    pub async fn evaluateHandle(&self, function: String) -> Result<ElementHandle, XcelerateError> {
        Ok(ElementHandle::new(
            Arc::clone(&self.inner).evaluate_handle(function).await?,
        ))
    }

    /// `$`.
    pub async fn query_selector(&self, selector: String) -> Result<ElementHandle, XcelerateError> {
        Ok(ElementHandle::new(
            Arc::clone(&self.inner).query_selector(selector).await?,
        ))
    }

    /// `$$`.
    pub async fn query_selector_all(
        &self,
        selector: String,
    ) -> Result<Vec<ElementHandle>, XcelerateError> {
        Ok(Arc::clone(&self.inner)
            .query_selector_all(selector)
            .await?
            .into_iter()
            .map(ElementHandle::new)
            .collect())
    }

    /// `$eval`.
    pub async fn eval_on_selector(
        &self,
        selector: String,
        expression: String,
    ) -> Result<String, XcelerateError> {
        Ok(self.inner.call_on_selector(selector, expression).await?)
    }

    /// `$$eval`.
    pub async fn eval_on_selector_all(
        &self,
        selector: String,
        expression: String,
    ) -> Result<String, XcelerateError> {
        Ok(self
            .inner
            .call_on_selector_all(selector, expression)
            .await?)
    }

    /// `$x`.
    pub async fn query_selector_xpath(
        &self,
        xpath: String,
    ) -> Result<ElementHandle, XcelerateError> {
        Ok(ElementHandle::new(
            Arc::clone(&self.inner).query_selector_xpath(xpath).await?,
        ))
    }

    /// `screenshot`.
    pub async fn screenshot(&self) -> Result<Vec<u8>, XcelerateError> {
        Ok(self.inner.screenshot().await?)
    }

    /// `boundingBox`.
    pub async fn boundingBox(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.call_json("function(){const r=this.getBoundingClientRect();return {x:r.x,y:r.y,width:r.width,height:r.height};}".to_string(), "[]".to_string()).await?)
    }

    /// `boxModel`.
    pub async fn boxModel(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.call_json("function(){const r=this.getBoundingClientRect();return {x:r.x,y:r.y,width:r.width,height:r.height};}".to_string(), "[]".to_string()).await?)
    }

    /// `clickablePoint`.
    pub async fn clickablePoint(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.call_json("function(){const r=this.getBoundingClientRect();return {x:r.x+r.width/2,y:r.y+r.height/2};}".to_string(), "[]".to_string()).await?)
    }

    /// `isIntersectingViewport`.
    pub async fn isIntersectingViewport(&self) -> Result<bool, XcelerateError> {
        Ok(self.inner.call_bool("function(){const r=this.getBoundingClientRect();return r.top>=0&&r.left>=0&&r.bottom<=(window.innerHeight||document.documentElement.clientHeight)&&r.right<=(window.innerWidth||document.documentElement.clientWidth);}".to_string(), "[]".to_string()).await?)
    }

    /// `scrollIntoViewIfNeeded`.
    pub async fn scrollIntoViewIfNeeded(&self) -> Result<(), XcelerateError> {
        self.inner
            .call_json(
                "function(){this.scrollIntoView({block:\"center\",inline:\"center\"});}"
                    .to_string(),
                "[]".to_string(),
            )
            .await?;
        Ok(())
    }

    /// `press`.
    pub async fn press(&self, key: String) -> Result<(), XcelerateError> {
        self.inner.call_json("function(key){this.focus();this.dispatchEvent(new KeyboardEvent(\"keydown\",{key:key,bubbles:true}));this.dispatchEvent(new KeyboardEvent(\"keyup\",{key:key,bubbles:true}));}".to_string(), serde_json::json!([key]).to_string()).await?;
        Ok(())
    }

    /// `tap`.
    pub async fn tap(&self) -> Result<(), XcelerateError> {
        self.inner
            .call_json("function(){this.click();}".to_string(), "[]".to_string())
            .await?;
        Ok(())
    }

    /// `select`.
    pub async fn select(&self, values_json: String) -> Result<(), XcelerateError> {
        self.inner.call_json("function(valuesRaw){const w=JSON.parse(valuesRaw).map(String);for(const o of this.options){o.selected=w.includes(o.value)||w.includes(o.text);}this.dispatchEvent(new Event('change',{bubbles:true}));}".to_string(), serde_json::json!([values_json]).to_string()).await?;
        Ok(())
    }

    /// `uploadFile`.
    pub async fn uploadFile(&self, files_json: String) -> Result<(), XcelerateError> {
        self.inner.set_input_files(files_json).await?;
        Ok(())
    }

    /// `setInputFiles`.
    pub async fn setInputFiles(&self, files_json: String) -> Result<(), XcelerateError> {
        self.inner.set_input_files(files_json).await?;
        Ok(())
    }

    /// `drag`.
    pub async fn drag(&self) -> Result<(), XcelerateError> {
        self.inner.call_json("function(){this.dispatchEvent(new DragEvent(\"dragstart\",{bubbles:true}));this.dispatchEvent(new DragEvent(\"dragend\",{bubbles:true}));}".to_string(), "[]".to_string()).await?;
        Ok(())
    }

    /// `dragAndDrop`.
    pub async fn dragAndDrop(&self, target: String) -> Result<(), XcelerateError> {
        self.inner.call_json("function(sel){const t=document.querySelector(sel);if(!t)return;this.dispatchEvent(new DragEvent(\"dragstart\",{bubbles:true}));t.dispatchEvent(new DragEvent(\"drop\",{bubbles:true}));this.dispatchEvent(new DragEvent(\"dragend\",{bubbles:true}));}".to_string(), serde_json::json!([target]).to_string()).await?;
        Ok(())
    }

    /// `dragEnter`.
    pub async fn dragEnter(&self) -> Result<(), XcelerateError> {
        self.inner
            .call_json(
                "function(){this.dispatchEvent(new DragEvent(\"dragenter\",{bubbles:true}));}"
                    .to_string(),
                "[]".to_string(),
            )
            .await?;
        Ok(())
    }

    /// `dragOver`.
    pub async fn dragOver(&self) -> Result<(), XcelerateError> {
        self.inner
            .call_json(
                "function(){this.dispatchEvent(new DragEvent(\"dragover\",{bubbles:true}));}"
                    .to_string(),
                "[]".to_string(),
            )
            .await?;
        Ok(())
    }

    /// `drop`.
    pub async fn drop(&self) -> Result<(), XcelerateError> {
        self.inner
            .call_json(
                "function(){this.dispatchEvent(new DragEvent(\"drop\",{bubbles:true}));}"
                    .to_string(),
                "[]".to_string(),
            )
            .await?;
        Ok(())
    }

    /// `waitForSelector`.
    pub async fn waitForSelector(&self, selector: String) -> Result<ElementHandle, XcelerateError> {
        Ok(ElementHandle::new(
            Arc::clone(&self.inner).wait_for_selector(selector).await?,
        ))
    }

    /// `asElement`.
    pub async fn asElement(&self) -> Result<ElementHandle, XcelerateError> {
        Ok(ElementHandle::new(Arc::clone(&self.inner)))
    }

    /// `getProperty`.
    pub async fn getProperty(&self, name: String) -> Result<String, XcelerateError> {
        Ok(self
            .inner
            .call_string(
                "function(n){const v=this[n];return v==null?\"\":String(v);}".to_string(),
                serde_json::json!([name]).to_string(),
            )
            .await?)
    }

    /// `dispose`.
    pub async fn dispose(&self) -> Result<(), XcelerateError> {
        self.inner.dispose().await?;
        Ok(())
    }

    /// `J`.
    pub async fn J(&self, selector: String) -> Result<ElementHandle, XcelerateError> {
        Ok(ElementHandle::new(
            Arc::clone(&self.inner).query_selector(selector).await?,
        ))
    }

    /// `JJ`.
    pub async fn JJ(&self, selector: String) -> Result<Vec<ElementHandle>, XcelerateError> {
        Ok(Arc::clone(&self.inner)
            .query_selector_all(selector)
            .await?
            .into_iter()
            .map(ElementHandle::new)
            .collect())
    }

    /// `JJeval`.
    pub async fn JJeval(
        &self,
        selector: String,
        expression: String,
    ) -> Result<String, XcelerateError> {
        Ok(self
            .inner
            .call_on_selector_all(selector, expression)
            .await?)
    }

    /// `Jeval`.
    pub async fn Jeval(
        &self,
        selector: String,
        expression: String,
    ) -> Result<String, XcelerateError> {
        Ok(self.inner.call_on_selector(selector, expression).await?)
    }

    /// `getProperties`.
    pub async fn getProperties(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.get_properties().await?)
    }

    /// `querySelector`.
    pub async fn querySelector(&self, selector: String) -> Result<ElementHandle, XcelerateError> {
        Ok(ElementHandle::new(
            Arc::clone(&self.inner).query_selector(selector).await?,
        ))
    }

    /// `querySelectorAll`.
    pub async fn querySelectorAll(
        &self,
        selector: String,
    ) -> Result<Vec<ElementHandle>, XcelerateError> {
        Ok(Arc::clone(&self.inner)
            .query_selector_all(selector)
            .await?
            .into_iter()
            .map(ElementHandle::new)
            .collect())
    }

    /// `querySelectorAllEval`.
    pub async fn querySelectorAllEval(
        &self,
        selector: String,
        expression: String,
    ) -> Result<String, XcelerateError> {
        Ok(self
            .inner
            .call_on_selector_all(selector, expression)
            .await?)
    }

    /// `querySelectorEval`.
    pub async fn querySelectorEval(
        &self,
        selector: String,
        expression: String,
    ) -> Result<String, XcelerateError> {
        Ok(self.inner.call_on_selector(selector, expression).await?)
    }

    /// `toString`.
    pub async fn toString(&self) -> Result<String, XcelerateError> {
        Ok(self
            .inner
            .call_string(
                "function(){return this.outerHTML;}".to_string(),
                "[]".to_string(),
            )
            .await?)
    }

    /// `xpath`.
    pub async fn xpath(&self, xpath: String) -> Result<ElementHandle, XcelerateError> {
        Ok(ElementHandle::new(
            Arc::clone(&self.inner).query_selector_xpath(xpath).await?,
        ))
    }

    /// `id`.
    pub async fn id(&self) -> Result<String, XcelerateError> {
        Ok(self
            .inner
            .call_string(
                "function(){return this.id||'';}".to_string(),
                "[]".to_string(),
            )
            .await?)
    }

    /// `isHidden`.
    pub async fn isHidden(&self) -> Result<bool, XcelerateError> {
        Ok(self.inner.call_bool("function(){const s=getComputedStyle(this);const r=this.getBoundingClientRect();return !(!!(r.width||r.height)&&s.visibility!==\"hidden\"&&s.display!==\"none\");}".to_string(), "[]".to_string()).await?)
    }

    /// `isVisible`.
    pub async fn isVisible(&self) -> Result<bool, XcelerateError> {
        Ok(self.inner.call_bool("function(){const s=getComputedStyle(this);const r=this.getBoundingClientRect();return !!(r.width||r.height)&&s.visibility!==\"hidden\"&&s.display!==\"none\"&&s.opacity!==\"0\";}".to_string(), "[]".to_string()).await?)
    }

    /// `scrollIntoView`.
    pub async fn scrollIntoView(&self) -> Result<(), XcelerateError> {
        self.inner
            .call_json(
                "function(){this.scrollIntoView();}".to_string(),
                "[]".to_string(),
            )
            .await?;
        Ok(())
    }

    /// `toElement`.
    pub async fn toElement(&self) -> Result<ElementHandle, XcelerateError> {
        Ok(ElementHandle::new(Arc::clone(&self.inner)))
    }

    /// `touchEnd`.
    pub async fn touchEnd(&self) -> Result<(), XcelerateError> {
        self.inner
            .call_json(
                "function(){this.dispatchEvent(new Event(\"touchend\",{bubbles:true}));}"
                    .to_string(),
                "[]".to_string(),
            )
            .await?;
        Ok(())
    }

    /// `touchMove`.
    pub async fn touchMove(&self) -> Result<(), XcelerateError> {
        self.inner
            .call_json(
                "function(){this.dispatchEvent(new Event(\"touchmove\",{bubbles:true}));}"
                    .to_string(),
                "[]".to_string(),
            )
            .await?;
        Ok(())
    }

    /// `touchStart`.
    pub async fn touchStart(&self) -> Result<(), XcelerateError> {
        self.inner
            .call_json(
                "function(){this.dispatchEvent(new Event(\"touchstart\",{bubbles:true}));}"
                    .to_string(),
                "[]".to_string(),
            )
            .await?;
        Ok(())
    }
}

/// Frame adapter (surface only; not obtainable from this runtime yet).
pub struct Frame {
    inner: Arc<CorePage>,
}

impl Frame {
    /// Wrap an existing xcelerate object.
    pub fn new(inner: Arc<CorePage>) -> Self {
        Self { inner }
    }

    /// `$`.
    pub async fn query_selector(&self, selector: String) -> Result<ElementHandle, XcelerateError> {
        Ok(ElementHandle::new(
            Arc::clone(&self.inner).find_element(selector).await?,
        ))
    }

    /// `$$`.
    pub async fn query_selector_all(
        &self,
        selector: String,
    ) -> Result<Vec<ElementHandle>, XcelerateError> {
        Ok(Arc::clone(&self.inner)
            .query_selector_all(selector)
            .await?
            .into_iter()
            .map(ElementHandle::new)
            .collect())
    }

    /// `$$eval`.
    pub async fn __eval(
        &self,
        selector: String,
        expression: String,
    ) -> Result<String, XcelerateError> {
        Ok(self
            .inner
            .call_on_selector_all(selector, expression)
            .await?)
    }

    /// `$eval`.
    pub async fn _eval(
        &self,
        selector: String,
        expression: String,
    ) -> Result<String, XcelerateError> {
        Ok(self.inner.call_on_selector(selector, expression).await?)
    }

    /// `addPreloadScript`.
    pub async fn addPreloadScript(&self, content: String) -> Result<String, XcelerateError> {
        Ok(self
            .inner
            .add_script_to_evaluate_on_new_document(content)
            .await?)
    }

    /// `addScriptTag`.
    pub async fn addScriptTag(&self, content: String) -> Result<String, XcelerateError> {
        Ok(self
            .inner
            .add_script_to_evaluate_on_new_document(content)
            .await?)
    }

    /// `addStyleTag`.
    pub async fn addStyleTag(&self, content: String) -> Result<String, XcelerateError> {
        Ok(self.inner.call_string("function(c){const s=document.createElement('style');s.textContent=c;document.head.appendChild(s);return s.textContent;}".to_string(), serde_json::json!([content]).to_string()).await?)
    }

    /// `childFrames`.
    pub async fn childFrames(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.frames().await?)
    }

    /// `click`.
    pub async fn click(&self, selector: String) -> Result<(), XcelerateError> {
        let __e = Arc::clone(&self.inner).find_element(selector).await?;
        Arc::clone(&__e).click().await?;
        Ok(())
    }

    /// `content`.
    pub async fn content(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.content().await?)
    }

    /// `evaluate`.
    pub async fn evaluate(&self, expression: String) -> Result<String, XcelerateError> {
        Ok(self
            .inner
            .call_json(
                "function(src){return (new Function(\"return (\"+src+\")\"))();}".to_string(),
                serde_json::json!([expression]).to_string(),
            )
            .await?)
    }

    /// `evaluateHandle`.
    pub async fn evaluateHandle(
        &self,
        expression: String,
    ) -> Result<ElementHandle, XcelerateError> {
        Ok(ElementHandle::new(
            Arc::clone(&self.inner).evaluate_handle(expression).await?,
        ))
    }

    /// `focus`.
    pub async fn focus(&self, selector: String) -> Result<(), XcelerateError> {
        let __e = Arc::clone(&self.inner).find_element(selector).await?;
        Arc::clone(&__e).focus().await?;
        Ok(())
    }

    /// `goto`.
    pub async fn goto(&self, url: String) -> Result<(), XcelerateError> {
        self.inner.navigate(url).await?;
        Ok(())
    }

    /// `hover`.
    pub async fn hover(&self, selector: String) -> Result<(), XcelerateError> {
        let __e = Arc::clone(&self.inner).find_element(selector).await?;
        Arc::clone(&__e).hover().await?;
        Ok(())
    }

    /// `name`.
    pub async fn name(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.frame_name().await?)
    }

    /// `parentFrame`.
    pub async fn parentFrame(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.main_frame().await?)
    }

    /// `select`.
    pub async fn select(
        &self,
        selector: String,
        values_json: String,
    ) -> Result<(), XcelerateError> {
        self.inner.call_json("function(sel,valuesRaw){const e=document.querySelector(sel);if(!e)return;const w=JSON.parse(valuesRaw).map(String);for(const o of e.options){o.selected=w.includes(o.value)||w.includes(o.text);}e.dispatchEvent(new Event('change',{bubbles:true}));}".to_string(), serde_json::json!([selector, values_json]).to_string()).await?;
        Ok(())
    }

    /// `setContent`.
    pub async fn setContent(&self, html: String) -> Result<(), XcelerateError> {
        self.inner
            .call_json(
                "function(h){document.open();document.write(h);document.close();}".to_string(),
                serde_json::json!([html]).to_string(),
            )
            .await?;
        Ok(())
    }

    /// `setFrameContent`.
    pub async fn setFrameContent(&self, html: String) -> Result<(), XcelerateError> {
        self.inner.set_content(html).await?;
        Ok(())
    }

    /// `tap`.
    pub async fn tap(&self, selector: String) -> Result<(), XcelerateError> {
        self.inner
            .call_json(
                "function(sel){const e=document.querySelector(sel);if(e)e.click();}".to_string(),
                serde_json::json!([selector]).to_string(),
            )
            .await?;
        Ok(())
    }

    /// `title`.
    pub async fn title(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.title().await?)
    }

    /// `type`.
    pub async fn r#type(&self, selector: String, value: String) -> Result<(), XcelerateError> {
        let __e = Arc::clone(&self.inner).find_element(selector).await?;
        Arc::clone(&__e).focus().await?;
        Arc::clone(&__e).type_text(value).await?;
        Ok(())
    }

    /// `url`.
    pub async fn url(&self) -> Result<String, XcelerateError> {
        Ok(self
            .inner
            .call_string(
                "function(){return location.href;}".to_string(),
                "[]".to_string(),
            )
            .await?)
    }

    /// `waitForDevicePrompt`.
    pub async fn waitForDevicePrompt(&self) -> Result<String, XcelerateError> {
        Ok(self
            .inner
            .wait_for_event_default("DeviceAccess.deviceRequestPrompted".to_string())
            .await?)
    }

    /// `waitForFunction`.
    pub async fn waitForFunction(&self, expression: String) -> Result<(), XcelerateError> {
        self.inner.wait_for_function(expression, 30_000).await?;
        Ok(())
    }

    /// `waitForNavigation`.
    pub async fn waitForNavigation(&self) -> Result<(), XcelerateError> {
        self.inner.wait_for_navigation().await?;
        Ok(())
    }

    /// `waitForSelector`.
    pub async fn waitForSelector(&self, selector: String) -> Result<ElementHandle, XcelerateError> {
        Ok(ElementHandle::new(
            Arc::clone(&self.inner).wait_for_selector(selector).await?,
        ))
    }
}

/// A CDP session (thin wrapper over the page connection).
pub struct CDPSession {
    inner: Arc<CorePage>,
}

impl CDPSession {
    /// Wrap an existing xcelerate object.
    pub fn new(inner: Arc<CorePage>) -> Self {
        Self { inner }
    }

    /// `send`.
    pub async fn send(
        &self,
        method: String,
        params_json: String,
    ) -> Result<String, XcelerateError> {
        Ok(self.inner.execute_cdp_cmd(method, params_json).await?)
    }

    /// `detach`.
    pub async fn detach(&self) -> Result<(), XcelerateError> {
        Ok(())
    }

    /// `on`.
    pub async fn on(&self, event_name: String) -> Result<(), XcelerateError> {
        self.inner.on(event_name).await;
        Ok(())
    }

    /// `off`.
    pub async fn off(&self, event_name: String) -> Result<(), XcelerateError> {
        self.inner.remove_listener(event_name).await;
        Ok(())
    }
}

/// Coverage collector (thin wrapper over the page).
pub struct Coverage {
    inner: Arc<CorePage>,
}

impl Coverage {
    /// Wrap an existing xcelerate object.
    pub fn new(inner: Arc<CorePage>) -> Self {
        Self { inner }
    }

    /// `startJSCoverage`.
    pub async fn startJSCoverage(&self) -> Result<(), XcelerateError> {
        self.inner.coverage_start_js().await?;
        Ok(())
    }

    /// `stopJSCoverage`.
    pub async fn stopJSCoverage(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.coverage_stop_js().await?)
    }

    /// `startCSSCoverage`.
    pub async fn startCSSCoverage(&self) -> Result<(), XcelerateError> {
        self.inner.coverage_start_css().await?;
        Ok(())
    }

    /// `stopCSSCoverage`.
    pub async fn stopCSSCoverage(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.coverage_stop_css().await?)
    }
}

/// Keyboard input (thin wrapper over the page).
pub struct Keyboard {
    inner: Arc<CorePage>,
}

impl Keyboard {
    /// Wrap an existing xcelerate object.
    pub fn new(inner: Arc<CorePage>) -> Self {
        Self { inner }
    }

    /// `press`.
    pub async fn press(&self, key: String) -> Result<(), XcelerateError> {
        self.inner.keyboard_press(key).await?;
        Ok(())
    }

    /// `down`.
    pub async fn down(&self, key: String) -> Result<(), XcelerateError> {
        self.inner.keyboard_down(key).await?;
        Ok(())
    }

    /// `up`.
    pub async fn up(&self, key: String) -> Result<(), XcelerateError> {
        self.inner.keyboard_up(key).await?;
        Ok(())
    }

    /// `type`.
    pub async fn r#type(&self, text: String) -> Result<(), XcelerateError> {
        self.inner.keyboard_type(text).await?;
        Ok(())
    }
}

/// Mouse input (thin wrapper over the page).
pub struct Mouse {
    inner: Arc<CorePage>,
}

impl Mouse {
    /// Wrap an existing xcelerate object.
    pub fn new(inner: Arc<CorePage>) -> Self {
        Self { inner }
    }

    /// `move`.
    pub async fn r#move(&self, x: f64, y: f64) -> Result<(), XcelerateError> {
        Arc::clone(&self.inner).move_mouse(x, y).await?;
        Ok(())
    }

    /// `click`.
    pub async fn click(&self, x: f64, y: f64) -> Result<(), XcelerateError> {
        Arc::clone(&self.inner).click_mouse(x, y).await?;
        Ok(())
    }

    /// `down`.
    pub async fn down(&self, button: String) -> Result<(), XcelerateError> {
        Arc::clone(&self.inner).mouse_down(button).await?;
        Ok(())
    }

    /// `up`.
    pub async fn up(&self, button: String) -> Result<(), XcelerateError> {
        Arc::clone(&self.inner).mouse_up(button).await?;
        Ok(())
    }
}

/// Touchscreen input (thin wrapper over the page).
pub struct Touchscreen {
    inner: Arc<CorePage>,
}

impl Touchscreen {
    /// Wrap an existing xcelerate object.
    pub fn new(inner: Arc<CorePage>) -> Self {
        Self { inner }
    }

    /// `tap`.
    pub async fn tap(&self, x: f64, y: f64) -> Result<(), XcelerateError> {
        self.inner.touch_tap(x, y).await?;
        Ok(())
    }
}

/// Tracing (thin wrapper over the page session).
pub struct Tracing {
    inner: Arc<CorePage>,
}

impl Tracing {
    /// Wrap an existing xcelerate object.
    pub fn new(inner: Arc<CorePage>) -> Self {
        Self { inner }
    }

    /// `start`.
    pub async fn start(&self) -> Result<(), XcelerateError> {
        self.inner.start_tracing().await?;
        Ok(())
    }

    /// `stop`.
    pub async fn stop(&self) -> Result<(), XcelerateError> {
        self.inner.stop_tracing().await?;
        Ok(())
    }
}

/// Screencast (thin wrapper over the page).
pub struct Screencast {
    inner: Arc<CorePage>,
}

impl Screencast {
    /// Wrap an existing xcelerate object.
    pub fn new(inner: Arc<CorePage>) -> Self {
        Self { inner }
    }

    /// `start`.
    pub async fn start(&self) -> Result<(), XcelerateError> {
        self.inner.start_screencast().await?;
        Ok(())
    }

    /// `stop`.
    pub async fn stop(&self) -> Result<(), XcelerateError> {
        self.inner.stop_screencast().await?;
        Ok(())
    }
}
