"""Generated adapter - do not edit by hand.

Source profile: adapters/profiles/puppeteer.json
Regenerate with: python scripts/generate_adapters.py
"""

from __future__ import annotations

import types

from . import _runtime

async def launch(config=None, **kwargs):
    result = await _runtime.browser_launch(config, **kwargs)
    return Browser(result)

class Browser:
    """Puppeteer-style browser."""

    def __init__(self, wrapped):
        self._wrapped = wrapped

    async def newPage(self, url="about:blank", **kwargs):
        result = await _runtime.browser_new_page(self._wrapped, url, **kwargs)
        return Page(result)

    async def createBrowserContext(self, **kwargs):
        result = await _runtime.browser_new_context(self._wrapped, **kwargs)
        return BrowserContext(result)

    async def version(self, **kwargs):
        return await _runtime.browser_version(self._wrapped, **kwargs)

    async def close(self, **kwargs):
        return await _runtime.browser_close(self._wrapped, **kwargs)

    async def newIncognitoBrowserContext(self, **kwargs):
        return await _runtime.browser_new_context_id(self._wrapped, **kwargs)

    async def browserContexts(self, **kwargs):
        return await _runtime.browser_contexts(self._wrapped, **kwargs)

    async def pages(self, **kwargs):
        return await _runtime.browser_pages(self._wrapped, **kwargs)

    async def targets(self, **kwargs):
        return await _runtime.browser_targets(self._wrapped, **kwargs)

    async def waitForTarget(self, **kwargs):
        return await _runtime.browser_wait_for_target(self._wrapped, **kwargs)

    async def userAgent(self, **kwargs):
        return await _runtime.browser_user_agent(self._wrapped, **kwargs)

    async def wsEndpoint(self, **kwargs):
        return await _runtime.browser_ws_endpoint(self._wrapped, **kwargs)

    async def isConnected(self, **kwargs):
        return await _runtime.browser_is_connected(self._wrapped, **kwargs)

    async def startTracing(self, **kwargs):
        return await _runtime.browser_start_tracing(self._wrapped, **kwargs)

    async def stopTracing(self, **kwargs):
        return await _runtime.browser_stop_tracing(self._wrapped, **kwargs)

    async def add_listener(self, event_name, **kwargs):
        return await _runtime.browser_on(self._wrapped, event_name, **kwargs)

    async def createIncogniteBrowserContext(self, **kwargs):
        return await _runtime.browser_new_context_id(self._wrapped, **kwargs)

    async def createIncognitoBrowserContext(self, **kwargs):
        return await _runtime.browser_new_context_id(self._wrapped, **kwargs)

    async def event_names(self, **kwargs):
        return await _runtime.browser_event_names(self._wrapped, **kwargs)

    async def listeners(self, **kwargs):
        return await _runtime.browser_event_names(self._wrapped, **kwargs)

    async def listens_to(self, event_name, **kwargs):
        return await _runtime.browser_listens_to(self._wrapped, event_name, **kwargs)

    async def on(self, event_name, **kwargs):
        return await _runtime.browser_on(self._wrapped, event_name, **kwargs)

    async def once(self, event_name, **kwargs):
        return await _runtime.browser_once(self._wrapped, event_name, **kwargs)

    async def remove_all_listeners(self, **kwargs):
        return await _runtime.browser_remove_all_listeners(self._wrapped, **kwargs)

    async def remove_listener(self, event_name, **kwargs):
        return await _runtime.browser_remove_listener(self._wrapped, event_name, **kwargs)

    async def connected(self, **kwargs):
        return await _runtime.browser_is_connected(self._wrapped, **kwargs)

    async def cookies(self, **kwargs):
        return await _runtime.browser_cookies(self._wrapped, **kwargs)

    async def deleteCookie(self, name, **kwargs):
        return await _runtime.browser_delete_cookie(self._wrapped, name, **kwargs)

    async def deleteMatchingCookies(self, name, **kwargs):
        return await _runtime.browser_delete_cookie(self._wrapped, name, **kwargs)

    async def setCookie(self, cookie_json, **kwargs):
        return await _runtime.browser_set_cookie(self._wrapped, cookie_json, **kwargs)

    async def setPermission(self, origin, permissions_json, **kwargs):
        return await _runtime.browser_grant_permissions(self._wrapped, origin, permissions_json, **kwargs)

class BrowserContext:
    """Puppeteer-style browser context (modelled as browser + init scripts)."""

    def __init__(self, wrapped):
        self._wrapped = wrapped

    async def newPage(self, url="about:blank", **kwargs):
        result = await _runtime.context_new_page(self._wrapped, url, **kwargs)
        return Page(result)

    async def close(self, **kwargs):
        return await _runtime.context_close(self._wrapped, **kwargs)

    async def pages(self, **kwargs):
        return await _runtime.context_pages(self._wrapped, **kwargs)

    async def targets(self, **kwargs):
        return await _runtime.context_targets(self._wrapped, **kwargs)

    async def newCDPSession(self, **kwargs):
        result = await _runtime.context_new_cdp_session(self._wrapped, **kwargs)
        return CDPSession(result)

    async def overridePermissions(self, origin, permissions_json, **kwargs):
        return await _runtime.context_grant_permissions(self._wrapped, origin, permissions_json, **kwargs)

    async def clearPermissionOverrides(self, **kwargs):
        return await _runtime.context_clear_permissions(self._wrapped, **kwargs)

    async def setCookie(self, cookie_json, **kwargs):
        return await _runtime.context_set_cookie(self._wrapped, cookie_json, **kwargs)

    async def deleteCookie(self, name, **kwargs):
        return await _runtime.context_delete_cookie(self._wrapped, name, **kwargs)

    async def cookies(self, **kwargs):
        return await _runtime.context_cookies(self._wrapped, **kwargs)

    async def waitForTarget(self, **kwargs):
        return await _runtime.context_wait_for_target(self._wrapped, **kwargs)

    async def addInitScript(self, script, **kwargs):
        return await _runtime.context_add_init_script(self._wrapped, script, **kwargs)

    async def deleteMatchingCookies(self, name, **kwargs):
        return await _runtime.context_delete_cookie(self._wrapped, name, **kwargs)

    async def setDownloadBehavior(self, path, **kwargs):
        return await _runtime.context_set_download_behavior(self._wrapped, path, **kwargs)

    async def setPermission(self, origin, permissions_json, **kwargs):
        return await _runtime.context_grant_permissions(self._wrapped, origin, permissions_json, **kwargs)

class Page:
    """Puppeteer-style page."""

    def __init__(self, wrapped):
        self._wrapped = wrapped

    async def goto(self, url, **kwargs):
        return await _runtime.page_goto(self._wrapped, url, **kwargs)

    async def goBack(self, **kwargs):
        return await _runtime.page_go_back(self._wrapped, **kwargs)

    async def reload(self, **kwargs):
        return await _runtime.page_reload(self._wrapped, **kwargs)

    async def title(self, **kwargs):
        return await _runtime.page_title(self._wrapped, **kwargs)

    async def content(self, **kwargs):
        return await _runtime.page_content(self._wrapped, **kwargs)

    async def pdf(self, **kwargs):
        return await _runtime.page_pdf(self._wrapped, **kwargs)

    async def screenshot(self, full_page=False, path=None, **kwargs):
        return await _runtime.page_screenshot(self._wrapped, full_page, path, **kwargs)

    async def query_selector(self, selector, **kwargs):
        result = await _runtime.page_find(self._wrapped, selector, **kwargs)
        return ElementHandle(result)

    async def click(self, selector, **kwargs):
        return await _runtime.page_click(self._wrapped, selector, **kwargs)

    async def type(self, selector, value, **kwargs):
        return await _runtime.page_type(self._wrapped, selector, value, **kwargs)

    async def hover(self, selector, **kwargs):
        return await _runtime.page_hover(self._wrapped, selector, **kwargs)

    async def focus(self, selector, **kwargs):
        return await _runtime.page_focus(self._wrapped, selector, **kwargs)

    async def waitForSelector(self, selector, **kwargs):
        result = await _runtime.page_wait_for_selector(self._wrapped, selector, **kwargs)
        return ElementHandle(result)

    async def waitForNavigation(self, **kwargs):
        return await _runtime.page_wait_for_navigation(self._wrapped, **kwargs)

    async def waitForTimeout(self, milliseconds, **kwargs):
        return await _runtime.page_wait_for_timeout(self._wrapped, milliseconds, **kwargs)

    async def evaluateOnNewDocument(self, script, **kwargs):
        return await _runtime.page_add_script(self._wrapped, script, **kwargs)

    async def addScriptTag(self, content, **kwargs):
        return await _runtime.page_add_script(self._wrapped, content, **kwargs)

    async def goForward(self, **kwargs):
        return await _runtime.page_go_forward(self._wrapped, **kwargs)

    async def close(self, **kwargs):
        return await _runtime.page_close(self._wrapped, **kwargs)

    async def setContent(self, html, **kwargs):
        return await _runtime.call_js(self._wrapped, 'page', 'function(h){document.open();document.write(h);document.close();}', 'void', [html], **kwargs)

    async def url(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'page', 'function(){return location.href;}', 'string', [], **kwargs)

    async def query_selector_all(self, selector, **kwargs):
        result = await _runtime.page_find_all(self._wrapped, selector, **kwargs)
        return [ElementHandle(item) for item in result]

    async def eval_on_selector(self, selector, expression, **kwargs):
        return await _runtime.page_eval_on_selector(self._wrapped, selector, expression, **kwargs)

    async def eval_on_selector_all(self, selector, expression, **kwargs):
        return await _runtime.page_eval_on_selector_all(self._wrapped, selector, expression, **kwargs)

    async def query_selector_xpath(self, xpath, **kwargs):
        result = await _runtime.page_query_selector_xpath(self._wrapped, xpath, **kwargs)
        return ElementHandle(result)

    async def waitForXPath(self, xpath, **kwargs):
        result = await _runtime.page_wait_for_xpath(self._wrapped, xpath, **kwargs)
        return ElementHandle(result)

    async def waitForFunction(self, expression, **kwargs):
        return await _runtime.page_wait_for_function(self._wrapped, expression, **kwargs)

    async def waitForRequest(self, **kwargs):
        return await _runtime.page_wait_for_request(self._wrapped, **kwargs)

    async def waitForResponse(self, **kwargs):
        return await _runtime.page_wait_for_response(self._wrapped, **kwargs)

    async def waitForNetworkIdle(self, **kwargs):
        return await _runtime.page_wait_for_network_idle(self._wrapped, **kwargs)

    async def waitForFrame(self, **kwargs):
        return await _runtime.page_wait_for_frame(self._wrapped, **kwargs)

    async def tap(self, selector, **kwargs):
        return await _runtime.call_js(self._wrapped, 'page', "function(sel){const __xp=(scope,sel)=>{const visit=(s,out)=>{if(!s||!s.querySelectorAll)return out;for(const el of s.querySelectorAll(sel))out.push(el);for(const el of s.querySelectorAll('*')){if(el.shadowRoot)visit(el.shadowRoot,out);}return out;};const out=[];visit(scope,out);if(scope.shadowRoot)visit(scope.shadowRoot,out);return out;};const e=(__xp(document,sel)[0]||null);if(e)e.click();}", 'void', [selector], **kwargs)

    async def select(self, selector, values_json, **kwargs):
        return await _runtime.call_js(self._wrapped, 'page', "function(sel,valuesRaw){const __xp=(scope,sel)=>{const visit=(s,out)=>{if(!s||!s.querySelectorAll)return out;for(const el of s.querySelectorAll(sel))out.push(el);for(const el of s.querySelectorAll('*')){if(el.shadowRoot)visit(el.shadowRoot,out);}return out;};const out=[];visit(scope,out);if(scope.shadowRoot)visit(scope.shadowRoot,out);return out;};const e=(__xp(document,sel)[0]||null);if(!e)return;const w=JSON.parse(valuesRaw).map(String);for(const o of e.options){o.selected=w.includes(o.value)||w.includes(o.text);}e.dispatchEvent(new Event('change',{bubbles:true}));}", 'void', [selector, values_json], **kwargs)

    async def setInputFiles(self, selector, files_json, **kwargs):
        return await _runtime.page_set_input_files(self._wrapped, selector, files_json, **kwargs)

    async def uploadFile(self, selector, files_json, **kwargs):
        return await _runtime.page_set_input_files(self._wrapped, selector, files_json, **kwargs)

    async def keyboard(self, **kwargs):
        result = await _runtime.page_new_keyboard(self._wrapped, **kwargs)
        return Keyboard(result)

    async def mouse(self, **kwargs):
        result = await _runtime.page_new_mouse(self._wrapped, **kwargs)
        return Mouse(result)

    async def touchscreen(self, **kwargs):
        result = await _runtime.page_new_touchscreen(self._wrapped, **kwargs)
        return Touchscreen(result)

    async def evaluate(self, expression, **kwargs):
        return await _runtime.call_js(self._wrapped, 'page', 'function(src){return (new Function("return ("+src+")"))();}', 'json', [expression], **kwargs)

    async def evaluateHandle(self, expression, **kwargs):
        result = await _runtime.page_evaluate_handle(self._wrapped, expression, **kwargs)
        return ElementHandle(result)

    async def addStyleTag(self, content, **kwargs):
        return await _runtime.call_js(self._wrapped, 'page', "function(c){const s=document.createElement('style');s.textContent=c;document.head.appendChild(s);return s.textContent;}", 'string', [content], **kwargs)

    async def setViewport(self, width, height, **kwargs):
        return await _runtime.cdp_call(self._wrapped, 'Emulation.setDeviceMetricsOverride', { 'width': width, 'height': height, 'deviceScaleFactor': 1, 'mobile': False }, 'void', **kwargs)

    async def setUserAgent(self, user_agent, accept_language, **kwargs):
        return await _runtime.page_set_user_agent(self._wrapped, user_agent, accept_language, **kwargs)

    async def setExtraHTTPHeaders(self, headers_json, **kwargs):
        return await _runtime.page_set_extra_http_headers(self._wrapped, headers_json, **kwargs)

    async def setBypassCSP(self, enabled, **kwargs):
        return await _runtime.cdp_call(self._wrapped, 'Page.setBypassCSP', { 'enabled': enabled }, 'void', **kwargs)

    async def setCacheEnabled(self, enabled, **kwargs):
        return await _runtime.page_set_cache_enabled(self._wrapped, enabled, **kwargs)

    async def setJavaScriptEnabled(self, enabled, **kwargs):
        return await _runtime.page_set_javascript_enabled(self._wrapped, enabled, **kwargs)

    async def setOfflineMode(self, offline, **kwargs):
        return await _runtime.page_set_offline(self._wrapped, offline, **kwargs)

    async def emulateNetworkConditions(self, offline, latency, download, upload, **kwargs):
        return await _runtime.cdp_call(self._wrapped, 'Network.emulateNetworkConditions', { 'offline': offline, 'latency': latency, 'downloadThroughput': download, 'uploadThroughput': upload }, 'void', **kwargs)

    async def emulateCPUThrottling(self, rate, **kwargs):
        return await _runtime.cdp_call(self._wrapped, 'Emulation.setCPUThrottlingRate', { 'rate': rate }, 'void', **kwargs)

    async def emulateIdleState(self, is_user_active, is_screen_unlocked, **kwargs):
        return await _runtime.page_emulate_idle_state(self._wrapped, is_user_active, is_screen_unlocked, **kwargs)

    async def emulateMediaFeatures(self, features_json, **kwargs):
        return await _runtime.page_set_emulated_media_features(self._wrapped, features_json, **kwargs)

    async def emulateTimezone(self, timezone_id, **kwargs):
        return await _runtime.cdp_call(self._wrapped, 'Emulation.setTimezoneOverride', { 'timezoneId': timezone_id }, 'void', **kwargs)

    async def emulateVisionDeficiency(self, kind, **kwargs):
        return await _runtime.cdp_call(self._wrapped, 'Emulation.setEmulatedVisionDeficiency', { 'type': kind }, 'void', **kwargs)

    async def setGeolocation(self, latitude, longitude, accuracy, **kwargs):
        return await _runtime.cdp_call(self._wrapped, 'Emulation.setGeolocationOverride', { 'latitude': latitude, 'longitude': longitude, 'accuracy': accuracy }, 'void', **kwargs)

    async def setRequestInterception(self, enabled, **kwargs):
        return await _runtime.page_set_request_interception(self._wrapped, enabled, **kwargs)

    async def setDragInterception(self, enabled, **kwargs):
        return await _runtime.page_set_drag_interception(self._wrapped, enabled, **kwargs)

    async def isDragInterceptionEnabled(self, **kwargs):
        return await _runtime.page_is_drag_interception_enabled(self._wrapped, **kwargs)

    async def authenticate(self, username, password, **kwargs):
        return await _runtime.page_authenticate(self._wrapped, username, password, **kwargs)

    async def setCookie(self, name, value, url, **kwargs):
        return await _runtime.cdp_call(self._wrapped, 'Network.setCookie', { 'name': name, 'value': value, 'url': url }, 'void', **kwargs)

    async def deleteCookie(self, name, **kwargs):
        return await _runtime.cdp_call(self._wrapped, 'Network.deleteCookies', { 'name': name }, 'void', **kwargs)

    async def cookies(self, **kwargs):
        return await _runtime.page_cookies(self._wrapped, **kwargs)

    async def metrics(self, **kwargs):
        return await _runtime.page_metrics(self._wrapped, **kwargs)

    async def bringToFront(self, **kwargs):
        return await _runtime.page_bring_to_front(self._wrapped, **kwargs)

    async def mainFrame(self, **kwargs):
        return await _runtime.page_main_frame(self._wrapped, **kwargs)

    async def frames(self, **kwargs):
        return await _runtime.page_frames(self._wrapped, **kwargs)

    async def J(self, selector, **kwargs):
        result = await _runtime.page_find(self._wrapped, selector, **kwargs)
        return ElementHandle(result)

    async def JJ(self, selector, **kwargs):
        result = await _runtime.page_find_all(self._wrapped, selector, **kwargs)
        return [ElementHandle(item) for item in result]

    async def JJeval(self, selector, expression, **kwargs):
        return await _runtime.page_eval_on_selector_all(self._wrapped, selector, expression, **kwargs)

    async def Jeval(self, selector, expression, **kwargs):
        return await _runtime.page_eval_on_selector(self._wrapped, selector, expression, **kwargs)

    async def Jx(self, xpath, **kwargs):
        result = await _runtime.page_query_selector_xpath(self._wrapped, xpath, **kwargs)
        return ElementHandle(result)

    async def add_listener(self, event_name, **kwargs):
        return await _runtime.page_on(self._wrapped, event_name, **kwargs)

    async def coverage(self, **kwargs):
        result = await _runtime.page_new_coverage(self._wrapped, **kwargs)
        return Coverage(result)

    async def emulateMedia(self, media, color_scheme, **kwargs):
        return await _runtime.page_emulate_media(self._wrapped, media, color_scheme, **kwargs)

    async def event_names(self, **kwargs):
        return await _runtime.page_event_names(self._wrapped, **kwargs)

    async def injectFile(self, path, **kwargs):
        return await _runtime.page_inject_file(self._wrapped, path, **kwargs)

    async def isClosed(self, **kwargs):
        return await _runtime.page_is_closed(self._wrapped, **kwargs)

    async def listeners(self, **kwargs):
        return await _runtime.page_event_names(self._wrapped, **kwargs)

    async def listens_to(self, event_name, **kwargs):
        return await _runtime.page_listens_to(self._wrapped, event_name, **kwargs)

    async def on(self, event_name, **kwargs):
        return await _runtime.page_on(self._wrapped, event_name, **kwargs)

    async def once(self, event_name, **kwargs):
        return await _runtime.page_once(self._wrapped, event_name, **kwargs)

    async def plainText(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'page', "function(){return document.body?document.body.innerText:'';}", 'string', [], **kwargs)

    async def querySelector(self, selector, **kwargs):
        result = await _runtime.page_find(self._wrapped, selector, **kwargs)
        return ElementHandle(result)

    async def querySelectorAll(self, selector, **kwargs):
        result = await _runtime.page_find_all(self._wrapped, selector, **kwargs)
        return [ElementHandle(item) for item in result]

    async def querySelectorAllEval(self, selector, expression, **kwargs):
        return await _runtime.page_eval_on_selector_all(self._wrapped, selector, expression, **kwargs)

    async def querySelectorEval(self, selector, expression, **kwargs):
        return await _runtime.page_eval_on_selector(self._wrapped, selector, expression, **kwargs)

    async def remove_all_listeners(self, **kwargs):
        return await _runtime.page_remove_all_listeners(self._wrapped, **kwargs)

    async def remove_listener(self, event_name, **kwargs):
        return await _runtime.page_remove_listener(self._wrapped, event_name, **kwargs)

    async def setDefaultNavigationTimeout(self, milliseconds, **kwargs):
        return await _runtime.page_set_default_timeout(self._wrapped, milliseconds, **kwargs)

    async def tracing(self, **kwargs):
        result = await _runtime.page_new_tracing(self._wrapped, **kwargs)
        return Tracing(result)

    async def viewport(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'page', 'function(){return {width:window.innerWidth,height:window.innerHeight};}', 'json', [], **kwargs)

    async def xpath(self, xpath, **kwargs):
        result = await _runtime.page_query_selector_xpath(self._wrapped, xpath, **kwargs)
        return ElementHandle(result)

    async def accessibility(self, **kwargs):
        return await _runtime.cdp_call(self._wrapped, 'Accessibility.getFullAXTree', {  }, 'json', **kwargs)

    async def createCDPSession(self, **kwargs):
        result = await _runtime.page_new_cdp_session(self._wrapped, **kwargs)
        return CDPSession(result)

    async def createPDFStream(self, **kwargs):
        return await _runtime.page_create_pdf_stream(self._wrapped, **kwargs)

    async def emulateFocusedPage(self, enabled, **kwargs):
        return await _runtime.cdp_call(self._wrapped, 'Emulation.setFocusEmulationEnabled', { 'enabled': enabled }, 'void', **kwargs)

    async def emulateLocale(self, locale, **kwargs):
        return await _runtime.cdp_call(self._wrapped, 'Emulation.setLocaleOverride', { 'locale': locale }, 'void', **kwargs)

    async def emulateMediaType(self, media, **kwargs):
        return await _runtime.cdp_call(self._wrapped, 'Emulation.setEmulatedMedia', { 'media': media }, 'void', **kwargs)

    async def getDefaultNavigationTimeout(self, **kwargs):
        return await _runtime.page_get_default_timeout(self._wrapped, **kwargs)

    async def getDefaultTimeout(self, **kwargs):
        return await _runtime.page_get_default_timeout(self._wrapped, **kwargs)

    async def off(self, event_name, **kwargs):
        return await _runtime.page_remove_listener(self._wrapped, event_name, **kwargs)

    async def removeScriptToEvaluateOnNewDocument(self, identifier, **kwargs):
        return await _runtime.page_remove_script(self._wrapped, identifier, **kwargs)

    async def screencast(self, **kwargs):
        result = await _runtime.page_new_screencast(self._wrapped, **kwargs)
        return Screencast(result)

    async def setBypassServiceWorker(self, bypass, **kwargs):
        return await _runtime.cdp_call(self._wrapped, 'Network.setBypassServiceWorker', { 'bypass': bypass }, 'void', **kwargs)

    async def setDefaultTimeout(self, milliseconds, **kwargs):
        return await _runtime.page_set_default_timeout(self._wrapped, milliseconds, **kwargs)

    async def waitForDevicePrompt(self, **kwargs):
        return await _runtime.page_wait_for_device_prompt(self._wrapped, **kwargs)

    async def waitForFileChooser(self, **kwargs):
        return await _runtime.page_wait_for_file_chooser(self._wrapped, **kwargs)

    async def waitForNetworkIdle_(self, **kwargs):
        return await _runtime.page_wait_for_network_idle(self._wrapped, **kwargs)

class ElementHandle:
    """Puppeteer-style element handle."""

    def __init__(self, wrapped):
        self._wrapped = wrapped

    async def click(self, **kwargs):
        return await _runtime.element_click(self._wrapped, **kwargs)

    async def type(self, value, **kwargs):
        return await _runtime.element_type(self._wrapped, value, **kwargs)

    async def hover(self, **kwargs):
        return await _runtime.element_hover(self._wrapped, **kwargs)

    async def focus(self, **kwargs):
        return await _runtime.element_focus(self._wrapped, **kwargs)

    async def evaluate(self, expression, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', "function(src){return (new Function('el','return ('+src+')(el);'))(this);}", 'json', [expression], **kwargs)

    async def evaluateHandle(self, expression, **kwargs):
        result = await _runtime.element_evaluate_handle(self._wrapped, expression, **kwargs)
        return ElementHandle(result)

    async def query_selector(self, selector, **kwargs):
        result = await _runtime.element_find(self._wrapped, selector, **kwargs)
        return ElementHandle(result)

    async def query_selector_all(self, selector, **kwargs):
        result = await _runtime.element_find_all(self._wrapped, selector, **kwargs)
        return [ElementHandle(item) for item in result]

    async def eval_on_selector(self, selector, expression, **kwargs):
        return await _runtime.element_eval_on_selector(self._wrapped, selector, expression, **kwargs)

    async def eval_on_selector_all(self, selector, expression, **kwargs):
        return await _runtime.element_eval_on_selector_all(self._wrapped, selector, expression, **kwargs)

    async def query_selector_xpath(self, xpath, **kwargs):
        result = await _runtime.element_query_selector_xpath(self._wrapped, xpath, **kwargs)
        return ElementHandle(result)

    async def screenshot(self, **kwargs):
        return await _runtime.element_screenshot(self._wrapped, **kwargs)

    async def boundingBox(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){const r=this.getBoundingClientRect();return {x:r.x,y:r.y,width:r.width,height:r.height};}', 'json', [], **kwargs)

    async def boxModel(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){const r=this.getBoundingClientRect();return {x:r.x,y:r.y,width:r.width,height:r.height};}', 'json', [], **kwargs)

    async def clickablePoint(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){const r=this.getBoundingClientRect();return {x:r.x+r.width/2,y:r.y+r.height/2};}', 'json', [], **kwargs)

    async def isIntersectingViewport(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){const r=this.getBoundingClientRect();return r.top>=0&&r.left>=0&&r.bottom<=(window.innerHeight||document.documentElement.clientHeight)&&r.right<=(window.innerWidth||document.documentElement.clientWidth);}', 'bool', [], **kwargs)

    async def scrollIntoViewIfNeeded(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){this.scrollIntoView({block:"center",inline:"center"});}', 'void', [], **kwargs)

    async def press(self, key, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(key){this.focus();this.dispatchEvent(new KeyboardEvent("keydown",{key:key,bubbles:true}));this.dispatchEvent(new KeyboardEvent("keyup",{key:key,bubbles:true}));}', 'void', [key], **kwargs)

    async def tap(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){this.click();}', 'void', [], **kwargs)

    async def select(self, values_json, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', "function(valuesRaw){const w=JSON.parse(valuesRaw).map(String);for(const o of this.options){o.selected=w.includes(o.value)||w.includes(o.text);}this.dispatchEvent(new Event('change',{bubbles:true}));}", 'void', [values_json], **kwargs)

    async def uploadFile(self, files_json, **kwargs):
        return await _runtime.element_set_input_files(self._wrapped, files_json, **kwargs)

    async def setInputFiles(self, files_json, **kwargs):
        return await _runtime.element_set_input_files(self._wrapped, files_json, **kwargs)

    async def drag(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){this.dispatchEvent(new DragEvent("dragstart",{bubbles:true}));this.dispatchEvent(new DragEvent("dragend",{bubbles:true}));}', 'void', [], **kwargs)

    async def dragAndDrop(self, target, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(sel){const __xp=(scope,sel)=>{const visit=(s,out)=>{if(!s||!s.querySelectorAll)return out;for(const el of s.querySelectorAll(sel))out.push(el);for(const el of s.querySelectorAll(\'*\')){if(el.shadowRoot)visit(el.shadowRoot,out);}return out;};const out=[];visit(scope,out);if(scope.shadowRoot)visit(scope.shadowRoot,out);return out;};const t=(__xp(document,sel)[0]||null);if(!t)return;this.dispatchEvent(new DragEvent("dragstart",{bubbles:true}));t.dispatchEvent(new DragEvent("drop",{bubbles:true}));this.dispatchEvent(new DragEvent("dragend",{bubbles:true}));}', 'void', [target], **kwargs)

    async def dragEnter(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){this.dispatchEvent(new DragEvent("dragenter",{bubbles:true}));}', 'void', [], **kwargs)

    async def dragOver(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){this.dispatchEvent(new DragEvent("dragover",{bubbles:true}));}', 'void', [], **kwargs)

    async def drop(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){this.dispatchEvent(new DragEvent("drop",{bubbles:true}));}', 'void', [], **kwargs)

    async def waitForSelector(self, selector, **kwargs):
        result = await _runtime.element_wait_for_selector(self._wrapped, selector, **kwargs)
        return ElementHandle(result)

    async def asElement(self, **kwargs):
        result = await _runtime.element_self(self._wrapped, **kwargs)
        return ElementHandle(result)

    async def getProperty(self, name, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(n){const v=this[n];return v==null?"":String(v);}', 'string', [name], **kwargs)

    async def dispose(self, **kwargs):
        return await _runtime.element_dispose(self._wrapped, **kwargs)

    async def J(self, selector, **kwargs):
        result = await _runtime.element_find(self._wrapped, selector, **kwargs)
        return ElementHandle(result)

    async def JJ(self, selector, **kwargs):
        result = await _runtime.element_find_all(self._wrapped, selector, **kwargs)
        return [ElementHandle(item) for item in result]

    async def JJeval(self, selector, expression, **kwargs):
        return await _runtime.element_eval_on_selector_all(self._wrapped, selector, expression, **kwargs)

    async def Jeval(self, selector, expression, **kwargs):
        return await _runtime.element_eval_on_selector(self._wrapped, selector, expression, **kwargs)

    async def getProperties(self, **kwargs):
        return await _runtime.element_get_properties(self._wrapped, **kwargs)

    async def querySelector(self, selector, **kwargs):
        result = await _runtime.element_find(self._wrapped, selector, **kwargs)
        return ElementHandle(result)

    async def querySelectorAll(self, selector, **kwargs):
        result = await _runtime.element_find_all(self._wrapped, selector, **kwargs)
        return [ElementHandle(item) for item in result]

    async def querySelectorAllEval(self, selector, expression, **kwargs):
        return await _runtime.element_eval_on_selector_all(self._wrapped, selector, expression, **kwargs)

    async def querySelectorEval(self, selector, expression, **kwargs):
        return await _runtime.element_eval_on_selector(self._wrapped, selector, expression, **kwargs)

    async def toString(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){return this.outerHTML;}', 'string', [], **kwargs)

    async def xpath(self, xpath, **kwargs):
        result = await _runtime.element_query_selector_xpath(self._wrapped, xpath, **kwargs)
        return ElementHandle(result)

    async def id(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', "function(){return this.id||'';}", 'string', [], **kwargs)

    async def isHidden(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){const s=getComputedStyle(this);const r=this.getBoundingClientRect();return !(!!(r.width||r.height)&&s.visibility!=="hidden"&&s.display!=="none");}', 'bool', [], **kwargs)

    async def isVisible(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){const s=getComputedStyle(this);const r=this.getBoundingClientRect();return !!(r.width||r.height)&&s.visibility!=="hidden"&&s.display!=="none"&&s.opacity!=="0";}', 'bool', [], **kwargs)

    async def scrollIntoView(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){this.scrollIntoView();}', 'void', [], **kwargs)

    async def toElement(self, **kwargs):
        result = await _runtime.element_self(self._wrapped, **kwargs)
        return ElementHandle(result)

    async def touchEnd(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){this.dispatchEvent(new Event("touchend",{bubbles:true}));}', 'void', [], **kwargs)

    async def touchMove(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){this.dispatchEvent(new Event("touchmove",{bubbles:true}));}', 'void', [], **kwargs)

    async def touchStart(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){this.dispatchEvent(new Event("touchstart",{bubbles:true}));}', 'void', [], **kwargs)

class Frame:
    """Frame adapter (surface only; not obtainable from this runtime yet)."""

    def __init__(self, wrapped):
        self._wrapped = wrapped

    async def query_selector(self, selector, **kwargs):
        result = await _runtime.page_find(self._wrapped, selector, **kwargs)
        return ElementHandle(result)

    async def query_selector_all(self, selector, **kwargs):
        result = await _runtime.page_find_all(self._wrapped, selector, **kwargs)
        return [ElementHandle(item) for item in result]

    async def __eval(self, selector, expression, **kwargs):
        return await _runtime.page_eval_on_selector_all(self._wrapped, selector, expression, **kwargs)

    async def _eval(self, selector, expression, **kwargs):
        return await _runtime.page_eval_on_selector(self._wrapped, selector, expression, **kwargs)

    async def addPreloadScript(self, content, **kwargs):
        return await _runtime.page_add_script(self._wrapped, content, **kwargs)

    async def addScriptTag(self, content, **kwargs):
        return await _runtime.page_add_script(self._wrapped, content, **kwargs)

    async def addStyleTag(self, content, **kwargs):
        return await _runtime.call_js(self._wrapped, 'page', "function(c){const s=document.createElement('style');s.textContent=c;document.head.appendChild(s);return s.textContent;}", 'string', [content], **kwargs)

    async def childFrames(self, **kwargs):
        return await _runtime.page_frames(self._wrapped, **kwargs)

    async def click(self, selector, **kwargs):
        return await _runtime.page_click(self._wrapped, selector, **kwargs)

    async def content(self, **kwargs):
        return await _runtime.page_content(self._wrapped, **kwargs)

    async def evaluate(self, expression, **kwargs):
        return await _runtime.call_js(self._wrapped, 'page', 'function(src){return (new Function("return ("+src+")"))();}', 'json', [expression], **kwargs)

    async def evaluateHandle(self, expression, **kwargs):
        result = await _runtime.page_evaluate_handle(self._wrapped, expression, **kwargs)
        return ElementHandle(result)

    async def focus(self, selector, **kwargs):
        return await _runtime.page_focus(self._wrapped, selector, **kwargs)

    async def goto(self, url, **kwargs):
        return await _runtime.page_goto(self._wrapped, url, **kwargs)

    async def hover(self, selector, **kwargs):
        return await _runtime.page_hover(self._wrapped, selector, **kwargs)

    async def name(self, **kwargs):
        return await _runtime.page_frame_name(self._wrapped, **kwargs)

    async def parentFrame(self, **kwargs):
        return await _runtime.page_main_frame(self._wrapped, **kwargs)

    async def select(self, selector, values_json, **kwargs):
        return await _runtime.call_js(self._wrapped, 'page', "function(sel,valuesRaw){const __xp=(scope,sel)=>{const visit=(s,out)=>{if(!s||!s.querySelectorAll)return out;for(const el of s.querySelectorAll(sel))out.push(el);for(const el of s.querySelectorAll('*')){if(el.shadowRoot)visit(el.shadowRoot,out);}return out;};const out=[];visit(scope,out);if(scope.shadowRoot)visit(scope.shadowRoot,out);return out;};const e=(__xp(document,sel)[0]||null);if(!e)return;const w=JSON.parse(valuesRaw).map(String);for(const o of e.options){o.selected=w.includes(o.value)||w.includes(o.text);}e.dispatchEvent(new Event('change',{bubbles:true}));}", 'void', [selector, values_json], **kwargs)

    async def setContent(self, html, **kwargs):
        return await _runtime.call_js(self._wrapped, 'page', 'function(h){document.open();document.write(h);document.close();}', 'void', [html], **kwargs)

    async def setFrameContent(self, html, **kwargs):
        return await _runtime.page_set_content(self._wrapped, html, **kwargs)

    async def tap(self, selector, **kwargs):
        return await _runtime.call_js(self._wrapped, 'page', "function(sel){const __xp=(scope,sel)=>{const visit=(s,out)=>{if(!s||!s.querySelectorAll)return out;for(const el of s.querySelectorAll(sel))out.push(el);for(const el of s.querySelectorAll('*')){if(el.shadowRoot)visit(el.shadowRoot,out);}return out;};const out=[];visit(scope,out);if(scope.shadowRoot)visit(scope.shadowRoot,out);return out;};const e=(__xp(document,sel)[0]||null);if(e)e.click();}", 'void', [selector], **kwargs)

    async def title(self, **kwargs):
        return await _runtime.page_title(self._wrapped, **kwargs)

    async def type(self, selector, value, **kwargs):
        return await _runtime.page_type(self._wrapped, selector, value, **kwargs)

    async def url(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'page', 'function(){return location.href;}', 'string', [], **kwargs)

    async def waitForDevicePrompt(self, **kwargs):
        return await _runtime.page_wait_for_device_prompt(self._wrapped, **kwargs)

    async def waitForFunction(self, expression, **kwargs):
        return await _runtime.page_wait_for_function(self._wrapped, expression, **kwargs)

    async def waitForNavigation(self, **kwargs):
        return await _runtime.page_wait_for_navigation(self._wrapped, **kwargs)

    async def waitForSelector(self, selector, **kwargs):
        result = await _runtime.page_wait_for_selector(self._wrapped, selector, **kwargs)
        return ElementHandle(result)

class CDPSession:
    """A CDP session (thin wrapper over the page connection)."""

    def __init__(self, wrapped):
        self._wrapped = wrapped

    async def send(self, method, params_json, **kwargs):
        return await _runtime.page_execute_cdp_cmd(self._wrapped, method, params_json, **kwargs)

    async def detach(self, **kwargs):
        return await _runtime.page_detach_cdp_session(self._wrapped, **kwargs)

    async def on(self, event_name, **kwargs):
        return await _runtime.page_on(self._wrapped, event_name, **kwargs)

    async def off(self, event_name, **kwargs):
        return await _runtime.page_remove_listener(self._wrapped, event_name, **kwargs)

class Coverage:
    """Coverage collector (thin wrapper over the page)."""

    def __init__(self, wrapped):
        self._wrapped = wrapped

    async def startJSCoverage(self, **kwargs):
        return await _runtime.page_coverage_start_js(self._wrapped, **kwargs)

    async def stopJSCoverage(self, **kwargs):
        return await _runtime.page_coverage_stop_js(self._wrapped, **kwargs)

    async def startCSSCoverage(self, **kwargs):
        return await _runtime.page_coverage_start_css(self._wrapped, **kwargs)

    async def stopCSSCoverage(self, **kwargs):
        return await _runtime.page_coverage_stop_css(self._wrapped, **kwargs)

class Keyboard:
    """Keyboard input (thin wrapper over the page)."""

    def __init__(self, wrapped):
        self._wrapped = wrapped

    async def press(self, key, **kwargs):
        return await _runtime.page_keyboard_press(self._wrapped, key, **kwargs)

    async def down(self, key, **kwargs):
        return await _runtime.page_keyboard_down(self._wrapped, key, **kwargs)

    async def up(self, key, **kwargs):
        return await _runtime.page_keyboard_up(self._wrapped, key, **kwargs)

    async def type(self, text, **kwargs):
        return await _runtime.page_keyboard_type(self._wrapped, text, **kwargs)

class Mouse:
    """Mouse input (thin wrapper over the page)."""

    def __init__(self, wrapped):
        self._wrapped = wrapped

    async def move(self, x, y, **kwargs):
        return await _runtime.page_mouse_move(self._wrapped, x, y, **kwargs)

    async def click(self, x, y, **kwargs):
        return await _runtime.page_mouse_click(self._wrapped, x, y, **kwargs)

    async def down(self, button, **kwargs):
        return await _runtime.page_mouse_down(self._wrapped, button, **kwargs)

    async def up(self, button, **kwargs):
        return await _runtime.page_mouse_up(self._wrapped, button, **kwargs)

class Touchscreen:
    """Touchscreen input (thin wrapper over the page)."""

    def __init__(self, wrapped):
        self._wrapped = wrapped

    async def tap(self, x, y, **kwargs):
        return await _runtime.page_touch_tap(self._wrapped, x, y, **kwargs)

class Tracing:
    """Tracing (thin wrapper over the page session)."""

    def __init__(self, wrapped):
        self._wrapped = wrapped

    async def start(self, **kwargs):
        return await _runtime.page_tracing_start(self._wrapped, **kwargs)

    async def stop(self, **kwargs):
        return await _runtime.page_tracing_stop(self._wrapped, **kwargs)

class Screencast:
    """Screencast (thin wrapper over the page)."""

    def __init__(self, wrapped):
        self._wrapped = wrapped

    async def start(self, **kwargs):
        return await _runtime.page_start_screencast(self._wrapped, **kwargs)

    async def stop(self, **kwargs):
        return await _runtime.page_stop_screencast(self._wrapped, **kwargs)


def use():
    """Return a namespace exposing the Puppeteer (promise API) API style."""
    return types.SimpleNamespace(
        launch=launch,
        Browser=Browser,
        BrowserContext=BrowserContext,
        Page=Page,
        ElementHandle=ElementHandle,
        Frame=Frame,
        CDPSession=CDPSession,
        Coverage=Coverage,
        Keyboard=Keyboard,
        Mouse=Mouse,
        Touchscreen=Touchscreen,
        Tracing=Tracing,
        Screencast=Screencast,
    )
