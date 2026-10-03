#![allow(unused_imports)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(clippy::needless_question_mark)]
//! Generated adapter: Playwright (async API). DO NOT EDIT.
//!
//! Regenerate with: python scripts/generate_adapters.py --target rust

use std::sync::Arc;

use crate::{
    Browser as CoreBrowser, BrowserConfig, Element as CoreElement, Page as CorePage,
    XcelerateError,
};

/// launch entry point.
pub async fn launch(config: Option<BrowserConfig>) -> Result<Browser, XcelerateError> {
    Ok(Browser::new(CoreBrowser::launch(config.unwrap_or_default()).await?))
}

/// Playwright-style browser.
pub struct Browser {
    inner: Arc<CoreBrowser>,
}

impl Browser {
    /// Wrap an existing xcelerate object.
    pub fn new(inner: Arc<CoreBrowser>) -> Self {
        Self { inner }
    }

    /// `new_page`.
    pub async fn new_page(&self, url: String) -> Result<Page, XcelerateError> {
        Ok(Page::new(Arc::clone(&self.inner).new_page(url).await?))
    }

    /// `new_context`.
    pub async fn new_context(&self) -> Result<BrowserContext, XcelerateError> {
        Ok(BrowserContext::new(super::support::ContextHandle::new(Arc::clone(&self.inner))))
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

    /// `is_connected`.
    pub async fn is_connected(&self) -> Result<bool, XcelerateError> {
        Ok(self.inner.is_connected().await)
    }

    /// `contexts`.
    pub async fn contexts(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.browser_contexts().await?)
    }

    /// `start_tracing`.
    pub async fn start_tracing(&self) -> Result<(), XcelerateError> {
        self.inner.start_tracing().await?;
        Ok(())
    }

    /// `stop_tracing`.
    pub async fn stop_tracing(&self) -> Result<(), XcelerateError> {
        self.inner.stop_tracing().await?;
        Ok(())
    }

    /// `bind`.
    pub async fn bind(&self, event_name: String) -> Result<(), XcelerateError> {
        self.inner.on(event_name).await;
        Ok(())
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

    /// `remove_listener`.
    pub async fn remove_listener(&self, event_name: String) -> Result<(), XcelerateError> {
        self.inner.remove_listener(event_name).await;
        Ok(())
    }

    /// `unbind`.
    pub async fn unbind(&self, event_name: String) -> Result<(), XcelerateError> {
        self.inner.remove_listener(event_name).await;
        Ok(())
    }
}

/// Playwright-style browser context (modelled as browser + init scripts).
pub struct BrowserContext {
    inner: super::support::ContextHandle,
}

impl BrowserContext {
    /// Wrap an existing xcelerate object.
    pub fn new(inner: super::support::ContextHandle) -> Self {
        Self { inner }
    }

    /// `new_page`.
    pub async fn new_page(&self, url: String) -> Result<Page, XcelerateError> {
        Ok(Page::new(self.inner.new_page(url).await?))
    }

    /// `add_init_script`.
    pub async fn add_init_script(&self, script: String) -> Result<(), XcelerateError> {
        self.inner.add_init_script(script).await;
        Ok(())
    }

    /// `close`.
    pub async fn close(&self) -> Result<(), XcelerateError> {
        Ok(())
    }

    /// `pages`.
    pub async fn pages(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.pages().await?)
    }

    /// `add_cookies`.
    pub async fn add_cookies(&self, cookies_json: String) -> Result<(), XcelerateError> {
        self.inner.add_cookies(cookies_json).await?;
        Ok(())
    }

    /// `clear_cookies`.
    pub async fn clear_cookies(&self) -> Result<(), XcelerateError> {
        self.inner.clear_cookies().await?;
        Ok(())
    }

    /// `cookies`.
    pub async fn cookies(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.cookies().await?)
    }

    /// `grant_permissions`.
    pub async fn grant_permissions(&self, origin: String, permissions_json: String) -> Result<(), XcelerateError> {
        self.inner.grant_permissions(origin, permissions_json).await?;
        Ok(())
    }

    /// `clear_permissions`.
    pub async fn clear_permissions(&self) -> Result<(), XcelerateError> {
        self.inner.clear_permissions().await?;
        Ok(())
    }

    /// `route`.
    pub async fn route(&self, pattern: String, action: String) -> Result<(), XcelerateError> {
        self.inner.route(pattern, action).await?;
        Ok(())
    }

    /// `unroute`.
    pub async fn unroute(&self, pattern: String) -> Result<(), XcelerateError> {
        self.inner.unroute(pattern).await?;
        Ok(())
    }

    /// `route_from_har`.
    pub async fn route_from_har(&self, path: String) -> Result<(), XcelerateError> {
        self.inner.route_from_har(path).await?;
        Ok(())
    }

    /// `set_default_navigation_timeout`.
    pub async fn set_default_navigation_timeout(&self, milliseconds: f64) -> Result<(), XcelerateError> {
        self.inner.set_default_timeout(milliseconds).await?;
        Ok(())
    }

    /// `set_default_timeout`.
    pub async fn set_default_timeout(&self, milliseconds: f64) -> Result<(), XcelerateError> {
        self.inner.set_default_timeout(milliseconds).await?;
        Ok(())
    }

    /// `set_extra_http_headers`.
    pub async fn set_extra_http_headers(&self, headers_json: String) -> Result<(), XcelerateError> {
        self.inner.set_extra_http_headers(headers_json).await?;
        Ok(())
    }

    /// `expect_page`.
    pub async fn expect_page(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.wait_for_event("Target.targetCreated".to_string()).await?)
    }

    /// `expect_event`.
    pub async fn expect_event(&self, event_name: String) -> Result<String, XcelerateError> {
        Ok(self.inner.wait_for_event(event_name).await?)
    }

    /// `wait_for_event`.
    pub async fn wait_for_event(&self, event_name: String) -> Result<String, XcelerateError> {
        Ok(self.inner.wait_for_event(event_name).await?)
    }

    /// `storage_state`.
    pub async fn storage_state(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.storage_state().await?)
    }

    /// `background_pages`.
    pub async fn background_pages(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.targets().await?)
    }

    /// `expect_console_message`.
    pub async fn expect_console_message(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.wait_for_event("Runtime.consoleAPICalled".to_string()).await?)
    }

    /// `is_closed`.
    pub async fn is_closed(&self) -> Result<bool, XcelerateError> {
        Ok(false)
    }

    /// `new_cdp_session`.
    pub async fn new_cdp_session(&self) -> Result<CDPSession, XcelerateError> {
        Ok(CDPSession::new(self.inner.working_page().await?))
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

    /// `remove_listener`.
    pub async fn remove_listener(&self, event_name: String) -> Result<(), XcelerateError> {
        self.inner.remove_listener(event_name).await;
        Ok(())
    }

    /// `set_geolocation`.
    pub async fn set_geolocation(&self, latitude: f64, longitude: f64, accuracy: f64) -> Result<(), XcelerateError> {
        self.inner.set_geolocation(latitude, longitude, accuracy).await?;
        Ok(())
    }

    /// `set_offline`.
    pub async fn set_offline(&self, offline: bool) -> Result<(), XcelerateError> {
        self.inner.set_offline(offline).await?;
        Ok(())
    }

    /// `set_storage_state`.
    pub async fn set_storage_state(&self, state_json: String) -> Result<(), XcelerateError> {
        self.inner.set_storage_state(state_json).await?;
        Ok(())
    }

    /// `unroute_all`.
    pub async fn unroute_all(&self) -> Result<(), XcelerateError> {
        self.inner.unroute_all().await?;
        Ok(())
    }
}

/// Playwright-style page.
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

    /// `reload`.
    pub async fn reload(&self) -> Result<(), XcelerateError> {
        self.inner.reload().await?;
        Ok(())
    }

    /// `go_back`.
    pub async fn go_back(&self) -> Result<(), XcelerateError> {
        self.inner.go_back().await?;
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
    pub async fn screenshot(&self, full_page: bool, path: Option<String>) -> Result<Vec<u8>, XcelerateError> {
        Ok(super::support::screenshot(&self.inner, full_page, path).await?)
    }

    /// `wait_for_selector`.
    pub async fn wait_for_selector(&self, selector: String) -> Result<Locator, XcelerateError> {
        Ok(Locator::new(Arc::clone(&self.inner).wait_for_selector(selector).await?))
    }

    /// `wait_for_load_state`.
    pub async fn wait_for_load_state(&self, _state: String) -> Result<(), XcelerateError> {
        self.inner.wait_for_navigation().await?;
        Ok(())
    }

    /// `wait_for_timeout`.
    pub async fn wait_for_timeout(&self, milliseconds: f64) -> Result<(), XcelerateError> {
        tokio::time::sleep(std::time::Duration::from_millis(milliseconds as u64)).await;
        Ok(())
    }

    /// `locator`.
    pub async fn locator(&self, selector: String) -> Result<Locator, XcelerateError> {
        Ok(Locator::new(Arc::clone(&self.inner).find_element(selector).await?))
    }

    /// `query_selector`.
    pub async fn query_selector(&self, selector: String) -> Result<Locator, XcelerateError> {
        Ok(Locator::new(Arc::clone(&self.inner).find_element(selector).await?))
    }

    /// `get_by_test_id`.
    pub async fn get_by_test_id(&self, test_id: String) -> Result<Locator, XcelerateError> {
        Ok(Locator::new(Arc::clone(&self.inner).find_element(super::support::attr_selector("data-testid", &test_id)).await?))
    }

    /// `get_by_placeholder`.
    pub async fn get_by_placeholder(&self, text: String) -> Result<Locator, XcelerateError> {
        Ok(Locator::new(Arc::clone(&self.inner).find_element(super::support::attr_selector("placeholder", &text)).await?))
    }

    /// `get_by_alt_text`.
    pub async fn get_by_alt_text(&self, text: String) -> Result<Locator, XcelerateError> {
        Ok(Locator::new(Arc::clone(&self.inner).find_element(super::support::attr_selector("alt", &text)).await?))
    }

    /// `get_by_title`.
    pub async fn get_by_title(&self, text: String) -> Result<Locator, XcelerateError> {
        Ok(Locator::new(Arc::clone(&self.inner).find_element(super::support::attr_selector("title", &text)).await?))
    }

    /// `click`.
    pub async fn click(&self, selector: String) -> Result<(), XcelerateError> {
        let __e = Arc::clone(&self.inner).find_element(selector).await?;
        Arc::clone(&__e).click().await?;
        Ok(())
    }

    /// `dblclick`.
    pub async fn dblclick(&self, selector: String) -> Result<(), XcelerateError> {
        let __e = Arc::clone(&self.inner).find_element(selector).await?;
        Arc::clone(&__e).click().await?;
        Arc::clone(&__e).click().await?;
        Ok(())
    }

    /// `fill`.
    pub async fn fill(&self, selector: String, value: String) -> Result<(), XcelerateError> {
        let __e = Arc::clone(&self.inner).find_element(selector).await?;
        Arc::clone(&__e).focus().await?;
        Arc::clone(&__e).type_text(value).await?;
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

    /// `inner_text`.
    pub async fn inner_text(&self, selector: String) -> Result<String, XcelerateError> {
        let __e = Arc::clone(&self.inner).find_element(selector).await?;
        Ok(__e.text().await?)
    }

    /// `text_content`.
    pub async fn text_content(&self, selector: String) -> Result<String, XcelerateError> {
        let __e = Arc::clone(&self.inner).find_element(selector).await?;
        Ok(__e.text().await?)
    }

    /// `inner_html`.
    pub async fn inner_html(&self, selector: String) -> Result<String, XcelerateError> {
        let __e = Arc::clone(&self.inner).find_element(selector).await?;
        Ok(__e.inner_html().await?)
    }

    /// `get_attribute`.
    pub async fn get_attribute(&self, selector: String, name: String) -> Result<Option<String>, XcelerateError> {
        let __e = Arc::clone(&self.inner).find_element(selector).await?;
        Ok(__e.attribute(name).await?)
    }

    /// `add_script_tag`.
    pub async fn add_script_tag(&self, content: String) -> Result<String, XcelerateError> {
        Ok(self.inner.add_script_to_evaluate_on_new_document(content).await?)
    }

    /// `go_forward`.
    pub async fn go_forward(&self) -> Result<(), XcelerateError> {
        self.inner.go_forward().await?;
        Ok(())
    }

    /// `close`.
    pub async fn close(&self) -> Result<(), XcelerateError> {
        self.inner.close().await?;
        Ok(())
    }

    /// `set_content`.
    pub async fn set_content(&self, html: String) -> Result<(), XcelerateError> {
        self.inner.call_json("function(h){document.open();document.write(h);document.close();}".to_string(), serde_json::json!([html]).to_string()).await?;
        Ok(())
    }

    /// `url`.
    pub async fn url(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.call_string("function(){return location.href;}".to_string(), "[]".to_string()).await?)
    }

    /// `get_by_role`.
    pub async fn get_by_role(&self, role: String) -> Result<Locator, XcelerateError> {
        Ok(Locator::new(Arc::clone(&self.inner).get_by_role(role).await?))
    }

    /// `get_by_text`.
    pub async fn get_by_text(&self, text: String) -> Result<Locator, XcelerateError> {
        Ok(Locator::new(Arc::clone(&self.inner).get_by_text(text).await?))
    }

    /// `get_by_label`.
    pub async fn get_by_label(&self, label: String) -> Result<Locator, XcelerateError> {
        Ok(Locator::new(Arc::clone(&self.inner).get_by_label(label).await?))
    }

    /// `press`.
    pub async fn press(&self, selector: String, key: String) -> Result<(), XcelerateError> {
        self.inner.call_json("function(sel,key){const e=document.querySelector(sel);if(!e)return;e.focus();e.dispatchEvent(new KeyboardEvent(\"keydown\",{key:key,bubbles:true}));e.dispatchEvent(new KeyboardEvent(\"keyup\",{key:key,bubbles:true}));}".to_string(), serde_json::json!([selector, key]).to_string()).await?;
        Ok(())
    }

    /// `check`.
    pub async fn check(&self, selector: String) -> Result<(), XcelerateError> {
        self.inner.call_json("function(sel){const e=document.querySelector(sel);if(e&&e.checked!==true)e.click();}".to_string(), serde_json::json!([selector]).to_string()).await?;
        Ok(())
    }

    /// `uncheck`.
    pub async fn uncheck(&self, selector: String) -> Result<(), XcelerateError> {
        self.inner.call_json("function(sel){const e=document.querySelector(sel);if(e&&e.checked===true)e.click();}".to_string(), serde_json::json!([selector]).to_string()).await?;
        Ok(())
    }

    /// `set_checked`.
    pub async fn set_checked(&self, selector: String, checked: bool) -> Result<(), XcelerateError> {
        self.inner.call_json("function(sel,v){const e=document.querySelector(sel);if(e&&e.checked!==!!v)e.click();}".to_string(), serde_json::json!([selector, checked]).to_string()).await?;
        Ok(())
    }

    /// `set_input_files`.
    pub async fn set_input_files(&self, selector: String, files_json: String) -> Result<(), XcelerateError> {
        Arc::clone(&self.inner).set_input_files(selector, files_json).await?;
        Ok(())
    }

    /// `select_option`.
    pub async fn select_option(&self, selector: String, values_json: String) -> Result<(), XcelerateError> {
        Arc::clone(&self.inner).select_option(selector, values_json).await?;
        Ok(())
    }

    /// `tap`.
    pub async fn tap(&self, selector: String) -> Result<(), XcelerateError> {
        self.inner.call_json("function(sel){const e=document.querySelector(sel);if(e)e.click();}".to_string(), serde_json::json!([selector]).to_string()).await?;
        Ok(())
    }

    /// `dispatch_event`.
    pub async fn dispatch_event(&self, selector: String, event_type: String) -> Result<(), XcelerateError> {
        self.inner.call_json("function(sel,t){const e=document.querySelector(sel);if(e)e.dispatchEvent(new Event(t,{bubbles:true,cancelable:true}));}".to_string(), serde_json::json!([selector, event_type]).to_string()).await?;
        Ok(())
    }

    /// `drag_and_drop`.
    pub async fn drag_and_drop(&self, source: String, target: String) -> Result<(), XcelerateError> {
        self.inner.call_json("function(src,dst){const s=document.querySelector(src),t=document.querySelector(dst);if(!s||!t)return;const dt=new DataTransfer();s.dispatchEvent(new DragEvent(\"dragstart\",{bubbles:true,dataTransfer:dt}));t.dispatchEvent(new DragEvent(\"dragenter\",{bubbles:true,dataTransfer:dt}));t.dispatchEvent(new DragEvent(\"dragover\",{bubbles:true,dataTransfer:dt}));t.dispatchEvent(new DragEvent(\"drop\",{bubbles:true,dataTransfer:dt}));s.dispatchEvent(new DragEvent(\"dragend\",{bubbles:true,dataTransfer:dt}));}".to_string(), serde_json::json!([source, target]).to_string()).await?;
        Ok(())
    }

    /// `input_value`.
    pub async fn input_value(&self, selector: String) -> Result<String, XcelerateError> {
        Ok(self.inner.call_string("function(sel){const e=document.querySelector(sel);return e&&e.value!=null?String(e.value):\"\";}".to_string(), serde_json::json!([selector]).to_string()).await?)
    }

    /// `is_visible`.
    pub async fn is_visible(&self, selector: String) -> Result<bool, XcelerateError> {
        Ok(self.inner.call_bool("function(sel){const e=document.querySelector(sel);if(!e)return false;const s=getComputedStyle(e);const r=e.getBoundingClientRect();return !!(r.width||r.height)&&s.visibility!==\"hidden\"&&s.display!==\"none\";}".to_string(), serde_json::json!([selector]).to_string()).await?)
    }

    /// `is_hidden`.
    pub async fn is_hidden(&self, selector: String) -> Result<bool, XcelerateError> {
        Ok(self.inner.call_bool("function(sel){const e=document.querySelector(sel);if(!e)return true;const s=getComputedStyle(e);const r=e.getBoundingClientRect();return !(!!(r.width||r.height)&&s.visibility!==\"hidden\"&&s.display!==\"none\");}".to_string(), serde_json::json!([selector]).to_string()).await?)
    }

    /// `is_enabled`.
    pub async fn is_enabled(&self, selector: String) -> Result<bool, XcelerateError> {
        Ok(self.inner.call_bool("function(sel){const e=document.querySelector(sel);return !!e&&!(e.disabled===true||e.hasAttribute(\"disabled\"));}".to_string(), serde_json::json!([selector]).to_string()).await?)
    }

    /// `is_disabled`.
    pub async fn is_disabled(&self, selector: String) -> Result<bool, XcelerateError> {
        Ok(self.inner.call_bool("function(sel){const e=document.querySelector(sel);return !!e&&(e.disabled===true||e.hasAttribute(\"disabled\"));}".to_string(), serde_json::json!([selector]).to_string()).await?)
    }

    /// `is_checked`.
    pub async fn is_checked(&self, selector: String) -> Result<bool, XcelerateError> {
        Ok(self.inner.call_bool("function(sel){const e=document.querySelector(sel);return !!e&&e.checked===true;}".to_string(), serde_json::json!([selector]).to_string()).await?)
    }

    /// `is_editable`.
    pub async fn is_editable(&self, selector: String) -> Result<bool, XcelerateError> {
        Ok(self.inner.call_bool("function(sel){const e=document.querySelector(sel);return !!e&&!(e.disabled===true||e.readOnly===true||e.hasAttribute(\"readonly\"));}".to_string(), serde_json::json!([selector]).to_string()).await?)
    }

    /// `wait_for_url`.
    pub async fn wait_for_url(&self, url: String) -> Result<(), XcelerateError> {
        self.inner.call_json("function(u){return new Promise(res=>{const check=()=>{if(location.href===u||location.href.indexOf(u)===0){res(true);}else{setTimeout(check,50);}};check();});}".to_string(), serde_json::json!([url]).to_string()).await?;
        Ok(())
    }

    /// `wait_for_event`.
    pub async fn wait_for_event(&self, event_name: String) -> Result<String, XcelerateError> {
        Ok(self.inner.wait_for_event_default(event_name).await?)
    }

    /// `wait_for_function`.
    pub async fn wait_for_function(&self, expression: String) -> Result<(), XcelerateError> {
        self.inner.wait_for_function(expression, 30_000).await?;
        Ok(())
    }

    /// `add_style_tag`.
    pub async fn add_style_tag(&self, content: String) -> Result<String, XcelerateError> {
        Ok(self.inner.call_string("function(c){const s=document.createElement('style');s.textContent=c;document.head.appendChild(s);return s.textContent;}".to_string(), serde_json::json!([content]).to_string()).await?)
    }

    /// `evaluate`.
    pub async fn evaluate(&self, expression: String) -> Result<String, XcelerateError> {
        Ok(self.inner.call_json("function(src){return (new Function(\"return (\"+src+\")\"))();}".to_string(), serde_json::json!([expression]).to_string()).await?)
    }

    /// `eval_on_selector`.
    pub async fn eval_on_selector(&self, selector: String, expression: String) -> Result<String, XcelerateError> {
        Ok(self.inner.call_on_selector(selector, expression).await?)
    }

    /// `eval_on_selector_all`.
    pub async fn eval_on_selector_all(&self, selector: String, expression: String) -> Result<String, XcelerateError> {
        Ok(self.inner.call_on_selector_all(selector, expression).await?)
    }

    /// `route`.
    pub async fn route(&self, pattern: String, action: String) -> Result<(), XcelerateError> {
        self.inner.route(pattern, action, None, None).await?;
        Ok(())
    }

    /// `unroute`.
    pub async fn unroute(&self, pattern: String) -> Result<(), XcelerateError> {
        self.inner.unroute(pattern).await?;
        Ok(())
    }

    /// `emulate_media`.
    pub async fn emulate_media(&self, media: Option<String>, color_scheme: Option<String>) -> Result<(), XcelerateError> {
        self.inner.emulate_media(media, color_scheme).await?;
        Ok(())
    }

    /// `set_viewport_size`.
    pub async fn set_viewport_size(&self, width: u64, height: i64) -> Result<(), XcelerateError> {
        self.inner.set_viewport_size(width, height).await?;
        Ok(())
    }

    /// `set_extra_http_headers`.
    pub async fn set_extra_http_headers(&self, headers_json: String) -> Result<(), XcelerateError> {
        self.inner.set_extra_http_headers(headers_json).await?;
        Ok(())
    }

    /// `bring_to_front`.
    pub async fn bring_to_front(&self) -> Result<(), XcelerateError> {
        self.inner.bring_to_front().await?;
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

    /// `frames`.
    pub async fn frames(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.frames().await?)
    }

    /// `main_frame`.
    pub async fn main_frame(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.main_frame().await?)
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

    /// `remove_listener`.
    pub async fn remove_listener(&self, event_name: String) -> Result<(), XcelerateError> {
        self.inner.remove_listener(event_name).await;
        Ok(())
    }

    /// `expect_popup`.
    pub async fn expect_popup(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.wait_for_event_default("Target.targetCreated".to_string()).await?)
    }

    /// `expect_download`.
    pub async fn expect_download(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.wait_for_event_default("Page.downloadWillBegin".to_string()).await?)
    }

    /// `expect_request`.
    pub async fn expect_request(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.wait_for_event_default("Network.requestWillBeSent".to_string()).await?)
    }

    /// `expect_response`.
    pub async fn expect_response(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.wait_for_event_default("Network.responseReceived".to_string()).await?)
    }

    /// `expect_console_message`.
    pub async fn expect_console_message(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.wait_for_event_default("Runtime.consoleAPICalled".to_string()).await?)
    }

    /// `expect_file_chooser`.
    pub async fn expect_file_chooser(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.wait_for_event_default("Page.fileChooserOpened".to_string()).await?)
    }

    /// `expect_event`.
    pub async fn expect_event(&self, event_name: String) -> Result<String, XcelerateError> {
        Ok(self.inner.wait_for_event_default(event_name).await?)
    }

    /// `add_init_script`.
    pub async fn add_init_script(&self, content: String) -> Result<String, XcelerateError> {
        Ok(self.inner.add_script_to_evaluate_on_new_document(content).await?)
    }

    /// `aria_snapshot`.
    pub async fn aria_snapshot(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.execute_cdp_cmd("Accessibility.getFullAXTree".to_string(), serde_json::json!({  }).to_string()).await?)
    }

    /// `clear_console_messages`.
    pub async fn clear_console_messages(&self) -> Result<(), XcelerateError> {
        self.inner.execute_cdp_cmd("Runtime.discardConsoleEntries".to_string(), serde_json::json!({  }).to_string()).await?;
        Ok(())
    }

    /// `clear_page_errors`.
    pub async fn clear_page_errors(&self) -> Result<(), XcelerateError> {
        self.inner.execute_cdp_cmd("Runtime.discardConsoleEntries".to_string(), serde_json::json!({  }).to_string()).await?;
        Ok(())
    }

    /// `evaluate_handle`.
    pub async fn evaluate_handle(&self, expression: String) -> Result<Locator, XcelerateError> {
        Ok(Locator::new(Arc::clone(&self.inner).evaluate_handle(expression).await?))
    }

    /// `expect_navigation`.
    pub async fn expect_navigation(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.wait_for_event_default("Page.frameNavigated".to_string()).await?)
    }

    /// `expect_request_finished`.
    pub async fn expect_request_finished(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.wait_for_event_default("Network.loadingFinished".to_string()).await?)
    }

    /// `expect_websocket`.
    pub async fn expect_websocket(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.wait_for_event_default("Network.webSocketCreated".to_string()).await?)
    }

    /// `expect_worker`.
    pub async fn expect_worker(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.wait_for_event_default("Target.attachedToTarget".to_string()).await?)
    }

    /// `frame`.
    pub async fn frame(&self, frame_id: String) -> Result<String, XcelerateError> {
        Ok(self.inner.frame(frame_id).await?)
    }

    /// `hide_highlight`.
    pub async fn hide_highlight(&self) -> Result<(), XcelerateError> {
        self.inner.call_json("function(){document.querySelectorAll(\"*\").forEach(function(e){e.style.outline=\"\";});}".to_string(), "[]".to_string()).await?;
        Ok(())
    }

    /// `is_closed`.
    pub async fn is_closed(&self) -> Result<bool, XcelerateError> {
        Ok(false)
    }

    /// `local_storage`.
    pub async fn local_storage(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.call_json("function(){return Object.fromEntries(Object.entries(localStorage));}".to_string(), "[]".to_string()).await?)
    }

    /// `query_selector_all`.
    pub async fn query_selector_all(&self, selector: String) -> Result<Vec<Locator>, XcelerateError> {
        Ok(Arc::clone(&self.inner).query_selector_all(selector).await?.into_iter().map(Locator::new).collect())
    }

    /// `request`.
    pub async fn request(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.request().await?)
    }

    /// `request_gc`.
    pub async fn request_gc(&self) -> Result<(), XcelerateError> {
        self.inner.execute_cdp_cmd("HeapProfiler.collectGarbage".to_string(), serde_json::json!({  }).to_string()).await?;
        Ok(())
    }

    /// `requests`.
    pub async fn requests(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.requests().await?)
    }

    /// `route_from_har`.
    pub async fn route_from_har(&self, path: String) -> Result<(), XcelerateError> {
        self.inner.route_from_har(path).await?;
        Ok(())
    }

    /// `screencast`.
    pub async fn screencast(&self) -> Result<Screencast, XcelerateError> {
        Ok(Screencast::new(Arc::clone(&self.inner)))
    }

    /// `session_storage`.
    pub async fn session_storage(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.call_json("function(){return Object.fromEntries(Object.entries(sessionStorage));}".to_string(), "[]".to_string()).await?)
    }

    /// `set_default_navigation_timeout`.
    pub async fn set_default_navigation_timeout(&self, milliseconds: f64) -> Result<(), XcelerateError> {
        self.inner.set_default_timeout(milliseconds).await?;
        Ok(())
    }

    /// `set_default_timeout`.
    pub async fn set_default_timeout(&self, milliseconds: f64) -> Result<(), XcelerateError> {
        self.inner.set_default_timeout(milliseconds).await?;
        Ok(())
    }

    /// `unroute_all`.
    pub async fn unroute_all(&self) -> Result<(), XcelerateError> {
        self.inner.unroute_all().await?;
        Ok(())
    }

    /// `viewport_size`.
    pub async fn viewport_size(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.call_json("function(){return {width:window.innerWidth,height:window.innerHeight};}".to_string(), "[]".to_string()).await?)
    }
}

/// Playwright-style locator (wraps an element handle).
pub struct Locator {
    inner: Arc<CoreElement>,
}

impl Locator {
    /// Wrap an existing xcelerate object.
    pub fn new(inner: Arc<CoreElement>) -> Self {
        Self { inner }
    }

    /// `click`.
    pub async fn click(&self) -> Result<(), XcelerateError> {
        Arc::clone(&self.inner).click().await?;
        Ok(())
    }

    /// `dblclick`.
    pub async fn dblclick(&self) -> Result<(), XcelerateError> {
        Arc::clone(&self.inner).click().await?;
        Arc::clone(&self.inner).click().await?;
        Ok(())
    }

    /// `fill`.
    pub async fn fill(&self, value: String) -> Result<(), XcelerateError> {
        let __e = Arc::clone(&self.inner);
        Arc::clone(&__e).focus().await?;
        Arc::clone(&__e).type_text(value).await?;
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

    /// `inner_text`.
    pub async fn inner_text(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.text().await?)
    }

    /// `text_content`.
    pub async fn text_content(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.text().await?)
    }

    /// `get_attribute`.
    pub async fn get_attribute(&self, name: String) -> Result<Option<String>, XcelerateError> {
        Ok(self.inner.attribute(name).await?)
    }

    /// `inner_html`.
    pub async fn inner_html(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.inner_html().await?)
    }

    /// `press`.
    pub async fn press(&self, key: String) -> Result<(), XcelerateError> {
        self.inner.call_json("function(key){this.focus();this.dispatchEvent(new KeyboardEvent(\"keydown\",{key:key,bubbles:true}));this.dispatchEvent(new KeyboardEvent(\"keyup\",{key:key,bubbles:true}));}".to_string(), serde_json::json!([key]).to_string()).await?;
        Ok(())
    }

    /// `check`.
    pub async fn check(&self) -> Result<(), XcelerateError> {
        self.inner.call_json("function(){if(this.checked!==true&&this.type!==undefined){this.click();}}".to_string(), "[]".to_string()).await?;
        Ok(())
    }

    /// `uncheck`.
    pub async fn uncheck(&self) -> Result<(), XcelerateError> {
        self.inner.call_json("function(){if(this.checked===true){this.click();}}".to_string(), "[]".to_string()).await?;
        Ok(())
    }

    /// `set_checked`.
    pub async fn set_checked(&self, checked: bool) -> Result<(), XcelerateError> {
        self.inner.call_json("function(v){if(this.checked!==!!v){this.click();}}".to_string(), serde_json::json!([checked]).to_string()).await?;
        Ok(())
    }

    /// `select_option`.
    pub async fn select_option(&self, values_json: String) -> Result<(), XcelerateError> {
        self.inner.call_json("function(valuesRaw){const w=JSON.parse(valuesRaw).map(String);for(const o of this.options){o.selected=w.includes(o.value)||w.includes(o.text);}this.dispatchEvent(new Event('change',{bubbles:true}));}".to_string(), serde_json::json!([values_json]).to_string()).await?;
        Ok(())
    }

    /// `input_value`.
    pub async fn input_value(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.call_string("function(){return this.value!=null?String(this.value):\"\";}".to_string(), "[]".to_string()).await?)
    }

    /// `is_visible`.
    pub async fn is_visible(&self) -> Result<bool, XcelerateError> {
        Ok(self.inner.call_bool("function(){const s=getComputedStyle(this);const r=this.getBoundingClientRect();return !!(r.width||r.height)&&s.visibility!==\"hidden\"&&s.display!==\"none\"&&s.opacity!==\"0\";}".to_string(), "[]".to_string()).await?)
    }

    /// `is_hidden`.
    pub async fn is_hidden(&self) -> Result<bool, XcelerateError> {
        Ok(self.inner.call_bool("function(){const s=getComputedStyle(this);const r=this.getBoundingClientRect();return !(!!(r.width||r.height)&&s.visibility!==\"hidden\"&&s.display!==\"none\");}".to_string(), "[]".to_string()).await?)
    }

    /// `is_enabled`.
    pub async fn is_enabled(&self) -> Result<bool, XcelerateError> {
        Ok(self.inner.call_bool("function(){return !(this.disabled===true||this.hasAttribute(\"disabled\"));}".to_string(), "[]".to_string()).await?)
    }

    /// `is_disabled`.
    pub async fn is_disabled(&self) -> Result<bool, XcelerateError> {
        Ok(self.inner.call_bool("function(){return this.disabled===true||this.hasAttribute(\"disabled\");}".to_string(), "[]".to_string()).await?)
    }

    /// `is_checked`.
    pub async fn is_checked(&self) -> Result<bool, XcelerateError> {
        Ok(self.inner.call_bool("function(){return this.checked===true;}".to_string(), "[]".to_string()).await?)
    }

    /// `is_editable`.
    pub async fn is_editable(&self) -> Result<bool, XcelerateError> {
        Ok(self.inner.call_bool("function(){return !(this.disabled===true||this.readOnly===true||this.hasAttribute(\"readonly\"));}".to_string(), "[]".to_string()).await?)
    }

    /// `is_empty`.
    pub async fn is_empty(&self) -> Result<bool, XcelerateError> {
        Ok(self.inner.call_bool("function(){return !this.value&&!this.textContent&&!this.innerHTML;}".to_string(), "[]".to_string()).await?)
    }

    /// `count`.
    pub async fn count(&self) -> Result<i64, XcelerateError> {
        Ok(self.inner.count().await?)
    }

    /// `all`.
    pub async fn all(&self) -> Result<Vec<Locator>, XcelerateError> {
        Ok(vec![Locator::new(Arc::clone(&self.inner))])
    }

    /// `all_inner_texts`.
    pub async fn all_inner_texts(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.call_json("function(){return [this.innerText];}".to_string(), "[]".to_string()).await?)
    }

    /// `all_text_contents`.
    pub async fn all_text_contents(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.call_json("function(){return [this.textContent];}".to_string(), "[]".to_string()).await?)
    }

    /// `first`.
    pub async fn first(&self) -> Result<Locator, XcelerateError> {
        Ok(Locator::new(Arc::clone(&self.inner)))
    }

    /// `last`.
    pub async fn last(&self) -> Result<Locator, XcelerateError> {
        Ok(Locator::new(Arc::clone(&self.inner)))
    }

    /// `nth`.
    pub async fn nth(&self, index: i64) -> Result<Locator, XcelerateError> {
        let _ = index;
        Ok(Locator::new(Arc::clone(&self.inner)))
    }

    /// `get_by_role`.
    pub async fn get_by_role(&self, role: String) -> Result<Locator, XcelerateError> {
        Ok(Locator::new(Arc::clone(&self.inner).get_by_role(role).await?))
    }

    /// `get_by_text`.
    pub async fn get_by_text(&self, text: String) -> Result<Locator, XcelerateError> {
        Ok(Locator::new(Arc::clone(&self.inner).get_by_text(text).await?))
    }

    /// `get_by_label`.
    pub async fn get_by_label(&self, label: String) -> Result<Locator, XcelerateError> {
        Ok(Locator::new(Arc::clone(&self.inner).get_by_label(label).await?))
    }

    /// `get_by_placeholder`.
    pub async fn get_by_placeholder(&self, text: String) -> Result<Locator, XcelerateError> {
        Ok(Locator::new(Arc::clone(&self.inner).query_selector_attr("placeholder".to_string(), text).await?))
    }

    /// `get_by_alt_text`.
    pub async fn get_by_alt_text(&self, text: String) -> Result<Locator, XcelerateError> {
        Ok(Locator::new(Arc::clone(&self.inner).query_selector_attr("alt".to_string(), text).await?))
    }

    /// `get_by_title`.
    pub async fn get_by_title(&self, text: String) -> Result<Locator, XcelerateError> {
        Ok(Locator::new(Arc::clone(&self.inner).query_selector_attr("title".to_string(), text).await?))
    }

    /// `get_by_test_id`.
    pub async fn get_by_test_id(&self, test_id: String) -> Result<Locator, XcelerateError> {
        Ok(Locator::new(Arc::clone(&self.inner).query_selector_attr("data-testid".to_string(), test_id).await?))
    }

    /// `evaluate`.
    pub async fn evaluate(&self, expression: String) -> Result<String, XcelerateError> {
        Ok(self.inner.call_json("function(src){return (new Function('el','return ('+src+')(el);'))(this);}".to_string(), serde_json::json!([expression]).to_string()).await?)
    }

    /// `screenshot`.
    pub async fn screenshot(&self) -> Result<Vec<u8>, XcelerateError> {
        Ok(self.inner.screenshot().await?)
    }

    /// `set_input_files`.
    pub async fn set_input_files(&self, files_json: String) -> Result<(), XcelerateError> {
        self.inner.set_input_files(files_json).await?;
        Ok(())
    }

    /// `drag_to`.
    pub async fn drag_to(&self, target: String) -> Result<(), XcelerateError> {
        self.inner.call_json("function(sel){const t=document.querySelector(sel);if(!t)return;const dt=new DataTransfer();this.dispatchEvent(new DragEvent(\"dragstart\",{bubbles:true,dataTransfer:dt}));t.dispatchEvent(new DragEvent(\"dragenter\",{bubbles:true,dataTransfer:dt}));t.dispatchEvent(new DragEvent(\"dragover\",{bubbles:true,dataTransfer:dt}));t.dispatchEvent(new DragEvent(\"drop\",{bubbles:true,dataTransfer:dt}));this.dispatchEvent(new DragEvent(\"dragend\",{bubbles:true,dataTransfer:dt}));}".to_string(), serde_json::json!([target]).to_string()).await?;
        Ok(())
    }

    /// `blur`.
    pub async fn blur(&self) -> Result<(), XcelerateError> {
        self.inner.call_json("function(){this.blur();}".to_string(), "[]".to_string()).await?;
        Ok(())
    }

    /// `bounding_box`.
    pub async fn bounding_box(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.call_json("function(){const r=this.getBoundingClientRect();return {x:r.x,y:r.y,width:r.width,height:r.height};}".to_string(), "[]".to_string()).await?)
    }

    /// `clear`.
    pub async fn clear(&self) -> Result<(), XcelerateError> {
        Arc::clone(&self.inner).focus().await?;
        Ok(())
    }

    /// `describe`.
    pub async fn describe(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.call_string("function(){return this.outerHTML.slice(0,120);}".to_string(), "[]".to_string()).await?)
    }

    /// `description`.
    pub async fn description(&self) -> Result<String, XcelerateError> {
        Ok(self.inner.call_string("function(){return this.outerHTML;}".to_string(), "[]".to_string()).await?)
    }

    /// `dispatch_event`.
    pub async fn dispatch_event(&self, event_type: String) -> Result<(), XcelerateError> {
        self.inner.call_json("function(t){this.dispatchEvent(new Event(t,{bubbles:true,cancelable:true}));}".to_string(), serde_json::json!([event_type]).to_string()).await?;
        Ok(())
    }

    /// `drop`.
    pub async fn drop(&self) -> Result<(), XcelerateError> {
        self.inner.call_json("function(){this.dispatchEvent(new DragEvent(\"drop\",{bubbles:true}));}".to_string(), "[]".to_string()).await?;
        Ok(())
    }

    /// `element_handle`.
    pub async fn element_handle(&self) -> Result<Locator, XcelerateError> {
        Ok(Locator::new(Arc::clone(&self.inner)))
    }

    /// `element_handles`.
    pub async fn element_handles(&self) -> Result<Vec<Locator>, XcelerateError> {
        Ok(vec![Locator::new(Arc::clone(&self.inner))])
    }

    /// `evaluate_handle`.
    pub async fn evaluate_handle(&self, function: String) -> Result<Locator, XcelerateError> {
        Ok(Locator::new(Arc::clone(&self.inner).evaluate_handle(function).await?))
    }

    /// `hide_highlight`.
    pub async fn hide_highlight(&self) -> Result<(), XcelerateError> {
        self.inner.call_json("function(){this.style.outline=\"\";}".to_string(), "[]".to_string()).await?;
        Ok(())
    }

    /// `highlight`.
    pub async fn highlight(&self) -> Result<(), XcelerateError> {
        self.inner.call_json("function(){this.style.outline=\"2px solid red\";}".to_string(), "[]".to_string()).await?;
        Ok(())
    }

    /// `press_sequentially`.
    pub async fn press_sequentially(&self, text: String) -> Result<(), XcelerateError> {
        self.inner.call_json("function(text){this.focus();this.value=(this.value||\"\")+text;this.dispatchEvent(new Event(\"input\",{bubbles:true}));}".to_string(), serde_json::json!([text]).to_string()).await?;
        Ok(())
    }

    /// `scroll_into_view_if_needed`.
    pub async fn scroll_into_view_if_needed(&self) -> Result<(), XcelerateError> {
        self.inner.call_json("function(){this.scrollIntoView({block:\"center\",inline:\"center\"});}".to_string(), "[]".to_string()).await?;
        Ok(())
    }

    /// `select_text`.
    pub async fn select_text(&self) -> Result<(), XcelerateError> {
        self.inner.call_json("function(){const r=document.createRange();r.selectNodeContents(this);const s=getSelection();s.removeAllRanges();s.addRange(r);}".to_string(), "[]".to_string()).await?;
        Ok(())
    }

    /// `tap`.
    pub async fn tap(&self) -> Result<(), XcelerateError> {
        self.inner.call_json("function(){this.click();}".to_string(), "[]".to_string()).await?;
        Ok(())
    }

    /// `visible`.
    pub async fn visible(&self) -> Result<bool, XcelerateError> {
        Ok(self.inner.call_bool("function(){const s=getComputedStyle(this);const r=this.getBoundingClientRect();return !!(r.width||r.height)&&s.visibility!==\"hidden\"&&s.display!==\"none\"&&s.opacity!==\"0\";}".to_string(), "[]".to_string()).await?)
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
    pub async fn send(&self, method: String, params_json: String) -> Result<String, XcelerateError> {
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
