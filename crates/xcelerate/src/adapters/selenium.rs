#![allow(unused_imports)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(clippy::needless_question_mark)]
//! Generated adapter: Selenium WebDriver. DO NOT EDIT.
//!
//! Regenerate with: python scripts/generate_adapters.py --target rust

use std::sync::Arc;

use crate::{
    Browser as CoreBrowser, BrowserConfig, Element as CoreElement, Page as CorePage, XcelerateError,
};

/// launch entry point.
pub async fn launch(config: Option<BrowserConfig>) -> Result<WebDriver, XcelerateError> {
    Ok(WebDriver::new(
        super::support::DriverHandle::new(config).await?,
    ))
}

/// Selenium-style driver. Owns navigation, element lookup, cookies and window management.
pub struct WebDriver {
    inner: super::support::DriverHandle,
}

impl WebDriver {
    /// Wrap an existing xcelerate object.
    pub fn new(inner: super::support::DriverHandle) -> Self {
        Self { inner }
    }

    /// `get`.
    pub async fn get(&self, url: String) -> Result<(), XcelerateError> {
        self.inner.page.navigate(url).await?;
        Ok(())
    }

    /// `find_element`.
    pub async fn find_element(
        &self,
        by: String,
        value: Option<String>,
    ) -> Result<WebElement, XcelerateError> {
        let __sel = super::support::resolve_selector(&by, value.as_deref())?;
        Ok(WebElement::new(
            Arc::clone(&self.inner.page).find_element(__sel).await?,
        ))
    }

    /// `title`.
    pub async fn title(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.page.title().await?)
    }

    /// `page_source`.
    pub async fn page_source(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.page.content().await?)
    }

    /// `refresh`.
    pub async fn refresh(&self) -> Result<(), XcelerateError> {
        self.inner.page.reload().await?;
        Ok(())
    }

    /// `back`.
    pub async fn back(&self) -> Result<(), XcelerateError> {
        self.inner.page.go_back().await?;
        Ok(())
    }

    /// `close`.
    pub async fn close(&self) -> Result<(), XcelerateError> {
        self.inner.browser.close().await?;
        Ok(())
    }

    /// `quit`.
    pub async fn quit(&self) -> Result<(), XcelerateError> {
        self.inner.browser.close().await?;
        Ok(())
    }

    /// `save_screenshot`.
    pub async fn save_screenshot(
        &self,
        path: Option<String>,
        full_page: bool,
    ) -> Result<Vec<u8>, XcelerateError> {
        Ok(super::support::screenshot(&self.inner.page, full_page, path).await?)
    }

    /// `get_screenshot_as_file`.
    pub async fn get_screenshot_as_file(
        &self,
        path: Option<String>,
        full_page: bool,
    ) -> Result<Vec<u8>, XcelerateError> {
        Ok(super::support::screenshot(&self.inner.page, full_page, path).await?)
    }

    /// `get_screenshot_as_png`.
    pub async fn get_screenshot_as_png(&self, full_page: bool) -> Result<Vec<u8>, XcelerateError> {
        Ok(super::support::screenshot(&self.inner.page, full_page, None).await?)
    }

    /// `get_screenshot_as_base64`.
    pub async fn get_screenshot_as_base64(
        &self,
        full_page: bool,
    ) -> Result<String, XcelerateError> {
        Ok(super::support::screenshot_base64(&self.inner.page, full_page).await?)
    }

    /// `find_elements`.
    pub async fn find_elements(
        &self,
        by: String,
        value: Option<String>,
    ) -> Result<Vec<WebElement>, XcelerateError> {
        let __sel = super::support::resolve_selector(&by, value.as_deref())?;
        Ok(Arc::clone(&self.inner.page)
            .query_selector_all(__sel)
            .await?
            .into_iter()
            .map(WebElement::new)
            .collect())
    }

    /// `execute_script`.
    pub async fn execute_script(&self, script: String) -> Result<String, XcelerateError> {
        Ok(self.inner.page.evaluate_json(script).await?)
    }

    /// `execute_async_script`.
    pub async fn execute_async_script(&self, script: String) -> Result<String, XcelerateError> {
        Ok(self.inner.page.evaluate_json(script).await?)
    }

    /// `execute`.
    pub async fn execute(&self, script: String) -> Result<String, XcelerateError> {
        Ok(self.inner.page.evaluate_json(script).await?)
    }

    /// `forward`.
    pub async fn forward(&self) -> Result<(), XcelerateError> {
        self.inner.page.go_forward().await?;
        Ok(())
    }

    /// `get_window_size`.
    pub async fn get_window_size(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.page.window_size().await?)
    }

    /// `set_window_size`.
    pub async fn set_window_size(&self, width: i64, height: i64) -> Result<(), XcelerateError> {
        self.inner.page.set_window_size(width, height).await?;
        Ok(())
    }

    /// `get_window_position`.
    pub async fn get_window_position(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.page.window_position().await?)
    }

    /// `set_window_position`.
    pub async fn set_window_position(&self, x: i64, y: i64) -> Result<(), XcelerateError> {
        self.inner.page.set_window_position(x, y).await?;
        Ok(())
    }

    /// `get_window_rect`.
    pub async fn get_window_rect(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.page.window_rect().await?)
    }

    /// `set_window_rect`.
    pub async fn set_window_rect(
        &self,
        x: i64,
        y: i64,
        width: i64,
        height: i64,
    ) -> Result<(), XcelerateError> {
        self.inner
            .page
            .set_window_bounds(x, y, width, height)
            .await?;
        Ok(())
    }

    /// `maximize_window`.
    pub async fn maximize_window(&self) -> Result<(), XcelerateError> {
        self.inner
            .page
            .set_window_state("maximized".to_string())
            .await?;
        Ok(())
    }

    /// `minimize_window`.
    pub async fn minimize_window(&self) -> Result<(), XcelerateError> {
        self.inner
            .page
            .set_window_state("minimized".to_string())
            .await?;
        Ok(())
    }

    /// `fullscreen_window`.
    pub async fn fullscreen_window(&self) -> Result<(), XcelerateError> {
        self.inner
            .page
            .set_window_state("fullscreen".to_string())
            .await?;
        Ok(())
    }

    /// `get_cookies`.
    pub async fn get_cookies(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.page.cookies().await?)
    }

    /// `get_cookie`.
    pub async fn get_cookie(&self, name: String) -> Result<String, XcelerateError> {
        Ok(self.inner.page.cookie(name).await?)
    }

    /// `add_cookie`.
    pub async fn add_cookie(&self, cookie_json: String) -> Result<String, XcelerateError> {
        Ok(self
            .inner
            .page
            .execute_cdp_cmd("Network.setCookie".to_string(), cookie_json)
            .await?)
    }

    /// `delete_cookie`.
    pub async fn delete_cookie(&self, name: String) -> Result<(), XcelerateError> {
        self.inner
            .page
            .execute_cdp_cmd(
                "Network.deleteCookies".to_string(),
                serde_json::json!({"name": name}).to_string(),
            )
            .await?;
        Ok(())
    }

    /// `delete_all_cookies`.
    pub async fn delete_all_cookies(&self) -> Result<(), XcelerateError> {
        self.inner
            .page
            .execute_cdp_cmd("Network.clearBrowserCookies".to_string(), "{}".to_string())
            .await?;
        Ok(())
    }

    /// `implicitly_wait`.
    pub async fn implicitly_wait(&self, seconds: f64) -> Result<(), XcelerateError> {
        self.inner.page.call_json("function(v){window.__xcelerate_timeouts=window.__xcelerate_timeouts||{};window.__xcelerate_timeouts.implicit=v;}".to_string(), serde_json::json!([seconds]).to_string()).await?;
        Ok(())
    }

    /// `set_page_load_timeout`.
    pub async fn set_page_load_timeout(&self, seconds: f64) -> Result<(), XcelerateError> {
        self.inner.page.call_json("function(v){window.__xcelerate_timeouts=window.__xcelerate_timeouts||{};window.__xcelerate_timeouts.page_load=v;}".to_string(), serde_json::json!([seconds]).to_string()).await?;
        Ok(())
    }

    /// `set_script_timeout`.
    pub async fn set_script_timeout(&self, seconds: f64) -> Result<(), XcelerateError> {
        self.inner.page.call_json("function(v){window.__xcelerate_timeouts=window.__xcelerate_timeouts||{};window.__xcelerate_timeouts.script=v;}".to_string(), serde_json::json!([seconds]).to_string()).await?;
        Ok(())
    }

    /// `set_page_load_strategy`.
    pub async fn set_page_load_strategy(&self, strategy: String) -> Result<(), XcelerateError> {
        self.inner
            .page
            .call_json(
                "function(v){window.__xcelerate_page_load_strategy=v;}".to_string(),
                serde_json::json!([strategy]).to_string(),
            )
            .await?;
        Ok(())
    }

    /// `print_page`.
    pub async fn print_page(&self) -> Result<String, XcelerateError> {
        Ok(self
            .inner
            .page
            .execute_cdp_cmd("Page.printToPDF".to_string(), "{}".to_string())
            .await?)
    }

    /// `current_url`.
    pub async fn current_url(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.page.url().await?)
    }

    /// `window_handles`.
    pub async fn window_handles(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.browser.targets().await?)
    }

    /// `current_window_handle`.
    pub async fn current_window_handle(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.page.target_id())
    }

    /// `switch_to`.
    pub async fn switch_to(&self) -> Result<SwitchTo, XcelerateError> {
        Ok(SwitchTo::new(super::support::DriverHandle {
            browser: Arc::clone(&self.inner.browser),
            page: Arc::clone(&self.inner.page),
        }))
    }

    /// `capabilities`.
    pub async fn capabilities(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.browser.capabilities().await?)
    }

    /// `desired_capabilities`.
    pub async fn desired_capabilities(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.browser.capabilities().await?)
    }

    /// `active_element`.
    pub async fn active_element(&self) -> Result<WebElement, XcelerateError> {
        Ok(WebElement::new(
            Arc::clone(&self.inner.page)
                .evaluate_handle("document.activeElement".to_string())
                .await?,
        ))
    }

    /// `dialog`.
    pub async fn dialog(&self) -> Result<Dialog, XcelerateError> {
        Ok(Dialog::new(super::support::DriverHandle {
            browser: Arc::clone(&self.inner.browser),
            page: Arc::clone(&self.inner.page),
        }))
    }

    /// `execute_cdp_cmd`.
    pub async fn execute_cdp_cmd(
        &self,
        method: String,
        params_json: String,
    ) -> Result<String, XcelerateError> {
        Ok(self.inner.page.execute_cdp_cmd(method, params_json).await?)
    }

    /// `timeouts`.
    pub async fn timeouts(&self) -> Result<Timeouts, XcelerateError> {
        Ok(Timeouts::new(super::support::DriverHandle {
            browser: Arc::clone(&self.inner.browser),
            page: Arc::clone(&self.inner.page),
        }))
    }
}

/// Selenium-style element.
pub struct WebElement {
    inner: Arc<CoreElement>,
}

impl WebElement {
    /// Wrap an existing xcelerate object.
    pub fn new(inner: Arc<CoreElement>) -> Self {
        Self { inner }
    }

    /// `click`.
    pub async fn click(&self) -> Result<(), XcelerateError> {
        Arc::clone(&self.inner).click().await?;
        Ok(())
    }

    /// `send_keys`.
    pub async fn send_keys(&self, value: String) -> Result<(), XcelerateError> {
        Arc::clone(&self.inner).type_text(value).await?;
        Ok(())
    }

    /// `clear`.
    pub async fn clear(&self) -> Result<(), XcelerateError> {
        Arc::clone(&self.inner).focus().await?;
        Ok(())
    }

    /// `text`.
    pub async fn text(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.text().await?)
    }

    /// `get_attribute`.
    pub async fn get_attribute(&self, name: String) -> Result<Option<String>, XcelerateError> {
        Ok(self.inner.attribute(name).await?)
    }

    /// `get_dom_attribute`.
    pub async fn get_dom_attribute(&self, name: String) -> Result<Option<String>, XcelerateError> {
        Ok(self.inner.attribute(name).await?)
    }

    /// `inner_html`.
    pub async fn inner_html(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.inner_html().await?)
    }

    /// `submit`.
    pub async fn submit(&self) -> Result<(), XcelerateError> {
        self.inner.call_json("function(){if(this.form){this.form.submit();}else if(this.tagName===\"FORM\"){this.submit();}}".to_string(), "[]".to_string()).await?;
        Ok(())
    }

    /// `get_property`.
    pub async fn get_property(&self, name: String) -> Result<String, XcelerateError> {
        Ok(self
            .inner
            .call_string(
                "function(n){const v=this[n];return v==null?\"\":String(v);}".to_string(),
                serde_json::json!([name]).to_string(),
            )
            .await?)
    }

    /// `is_selected`.
    pub async fn is_selected(&self) -> Result<bool, XcelerateError> {
        Ok(self
            .inner
            .call_bool(
                "function(){return this.selected===true||this.checked===true;}".to_string(),
                "[]".to_string(),
            )
            .await?)
    }

    /// `is_enabled`.
    pub async fn is_enabled(&self) -> Result<bool, XcelerateError> {
        Ok(self
            .inner
            .call_bool(
                "function(){return !(this.disabled===true||this.hasAttribute(\"disabled\"));}"
                    .to_string(),
                "[]".to_string(),
            )
            .await?)
    }

    /// `is_displayed`.
    pub async fn is_displayed(&self) -> Result<bool, XcelerateError> {
        Ok(self.inner.call_bool("function(){const s=getComputedStyle(this);const r=this.getBoundingClientRect();return !!(r.width||r.height)&&s.visibility!==\"hidden\"&&s.display!==\"none\"&&s.opacity!==\"0\";}".to_string(), "[]".to_string()).await?)
    }

    /// `find_element`.
    pub async fn find_element(&self, selector: String) -> Result<WebElement, XcelerateError> {
        Ok(WebElement::new(
            Arc::clone(&self.inner).query_selector(selector).await?,
        ))
    }

    /// `find_elements`.
    pub async fn find_elements(&self, selector: String) -> Result<Vec<WebElement>, XcelerateError> {
        Ok(Arc::clone(&self.inner)
            .query_selector_all(selector)
            .await?
            .into_iter()
            .map(WebElement::new)
            .collect())
    }

    /// `value_of_css_property`.
    pub async fn value_of_css_property(
        &self,
        property_name: String,
    ) -> Result<String, XcelerateError> {
        Ok(self
            .inner
            .call_string(
                "function(p){return getComputedStyle(this).getPropertyValue(p);}".to_string(),
                serde_json::json!([property_name]).to_string(),
            )
            .await?)
    }

    /// `screenshot`.
    pub async fn screenshot(&self) -> Result<Vec<u8>, XcelerateError> {
        Ok(self.inner.screenshot().await?)
    }

    /// `screenshot_as_png`.
    pub async fn screenshot_as_png(&self) -> Result<Vec<u8>, XcelerateError> {
        Ok(self.inner.screenshot().await?)
    }

    /// `screenshot_as_base64`.
    pub async fn screenshot_as_base64(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.screenshot_base64().await?)
    }

    /// `tag_name`.
    pub async fn tag_name(&self) -> Result<String, XcelerateError> {
        Ok(self
            .inner
            .call_string(
                "function(){return this.tagName?this.tagName.toLowerCase():\"unknown\";}"
                    .to_string(),
                "[]".to_string(),
            )
            .await?)
    }

    /// `location`.
    pub async fn location(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.call_json("function(){const r=this.getBoundingClientRect();return {x:r.x,y:r.y,width:r.width,height:r.height};}".to_string(), "[]".to_string()).await?)
    }

    /// `location_once_scrolled_into_view`.
    pub async fn location_once_scrolled_into_view(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.call_json("function(){this.scrollIntoView({block:\"center\",inline:\"center\"});const r=this.getBoundingClientRect();return {x:r.x,y:r.y};}".to_string(), "[]".to_string()).await?)
    }

    /// `size`.
    pub async fn size(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.call_json("function(){const r=this.getBoundingClientRect();return {width:r.width,height:r.height};}".to_string(), "[]".to_string()).await?)
    }

    /// `rect`.
    pub async fn rect(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.call_json("function(){const r=this.getBoundingClientRect();return {x:r.x,y:r.y,width:r.width,height:r.height};}".to_string(), "[]".to_string()).await?)
    }

    /// `parent`.
    pub async fn parent(&self) -> Result<WebElement, XcelerateError> {
        Ok(WebElement::new(
            Arc::clone(&self.inner)
                .evaluate_handle("function(){{return this.parentElement;}}".to_string())
                .await?,
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

    /// `shadow_root`.
    pub async fn shadow_root(&self) -> Result<String, XcelerateError> {
        Ok(self
            .inner
            .call_string(
                "function(){return this.shadowRoot?this.shadowRoot.innerHTML:\"\";}".to_string(),
                "[]".to_string(),
            )
            .await?)
    }

    /// `aria_role`.
    pub async fn aria_role(&self) -> Result<String, XcelerateError> {
        Ok(self
            .inner
            .call_string(
                "function(){return this.getAttribute(\"role\")||this.tagName.toLowerCase();}"
                    .to_string(),
                "[]".to_string(),
            )
            .await?)
    }

    /// `accessible_name`.
    pub async fn accessible_name(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.call_string("function(){return (this.getAttribute(\"aria-label\")||this.textContent||\"\").trim();}".to_string(), "[]".to_string()).await?)
    }
}

/// Selenium-style target switching (thin wrapper over the driver).
pub struct SwitchTo {
    inner: super::support::DriverHandle,
}

impl SwitchTo {
    /// Wrap an existing xcelerate object.
    pub fn new(inner: super::support::DriverHandle) -> Self {
        Self { inner }
    }

    /// `window`.
    pub async fn window(&self, target_id: String) -> Result<(), XcelerateError> {
        self.inner.page.activate_target(target_id).await?;
        Ok(())
    }

    /// `default_content`.
    pub async fn default_content(&self) -> Result<(), XcelerateError> {
        Ok(())
    }

    /// `new_window`.
    pub async fn new_window(&self, window_type: String) -> Result<(), XcelerateError> {
        let _ = window_type;
        let _ = Arc::clone(&self.inner.browser)
            .new_page("about:blank".to_string())
            .await?;
        Ok(())
    }
}

/// Selenium-style timeouts (thin wrapper over the driver).
pub struct Timeouts {
    inner: super::support::DriverHandle,
}

impl Timeouts {
    /// Wrap an existing xcelerate object.
    pub fn new(inner: super::support::DriverHandle) -> Self {
        Self { inner }
    }

    /// `implicitly_wait`.
    pub async fn implicitly_wait(&self, seconds: f64) -> Result<(), XcelerateError> {
        self.inner.page.call_json("function(v){window.__xcelerate_timeouts=window.__xcelerate_timeouts||{};window.__xcelerate_timeouts.implicit=v;}".to_string(), serde_json::json!([seconds]).to_string()).await?;
        Ok(())
    }

    /// `set_script_timeout`.
    pub async fn set_script_timeout(&self, seconds: f64) -> Result<(), XcelerateError> {
        self.inner.page.call_json("function(v){window.__xcelerate_timeouts=window.__xcelerate_timeouts||{};window.__xcelerate_timeouts.script=v;}".to_string(), serde_json::json!([seconds]).to_string()).await?;
        Ok(())
    }

    /// `page_load_timeout`.
    pub async fn page_load_timeout(&self, seconds: f64) -> Result<(), XcelerateError> {
        self.inner.page.call_json("function(v){window.__xcelerate_timeouts=window.__xcelerate_timeouts||{};window.__xcelerate_timeouts.page_load=v;}".to_string(), serde_json::json!([seconds]).to_string()).await?;
        Ok(())
    }
}

/// JavaScript dialog (thin wrapper over the driver).
pub struct Dialog {
    inner: super::support::DriverHandle,
}

impl Dialog {
    /// Wrap an existing xcelerate object.
    pub fn new(inner: super::support::DriverHandle) -> Self {
        Self { inner }
    }

    /// `accept`.
    pub async fn accept(&self) -> Result<(), XcelerateError> {
        self.inner.page.handle_js_dialog(true, None).await?;
        Ok(())
    }

    /// `dismiss`.
    pub async fn dismiss(&self) -> Result<(), XcelerateError> {
        self.inner.page.handle_js_dialog(false, None).await?;
        Ok(())
    }

    /// `send_keys`.
    pub async fn send_keys(&self, text: String) -> Result<(), XcelerateError> {
        self.inner.page.handle_js_dialog(true, Some(text)).await?;
        Ok(())
    }
}
