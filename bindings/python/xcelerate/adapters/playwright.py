"""Generated adapter - do not edit by hand.

Source profile: adapters/profiles/playwright.json
Regenerate with: python scripts/generate_adapters.py
"""

from __future__ import annotations

import types

from . import _runtime

async def launch(config=None, **kwargs):
    result = await _runtime.browser_launch(config, **kwargs)
    return Browser(result)

class Browser:
    """Playwright-style browser."""

    def __init__(self, wrapped):
        self._wrapped = wrapped

    async def new_page(self, url="about:blank", **kwargs):
        result = await _runtime.browser_new_page(self._wrapped, url, **kwargs)
        return Page(result)

    async def new_context(self, **kwargs):
        result = await _runtime.browser_new_context(self._wrapped, **kwargs)
        return BrowserContext(result)

    async def version(self, **kwargs):
        return await _runtime.browser_version(self._wrapped, **kwargs)

    async def close(self, **kwargs):
        return await _runtime.browser_close(self._wrapped, **kwargs)

    async def is_connected(self, **kwargs):
        return await _runtime.browser_is_connected(self._wrapped, **kwargs)

    async def contexts(self, **kwargs):
        return await _runtime.browser_contexts(self._wrapped, **kwargs)

    async def start_tracing(self, **kwargs):
        return await _runtime.browser_start_tracing(self._wrapped, **kwargs)

    async def stop_tracing(self, **kwargs):
        return await _runtime.browser_stop_tracing(self._wrapped, **kwargs)

    async def bind(self, event_name, **kwargs):
        return await _runtime.browser_on(self._wrapped, event_name, **kwargs)

    async def on(self, event_name, **kwargs):
        return await _runtime.browser_on(self._wrapped, event_name, **kwargs)

    async def once(self, event_name, **kwargs):
        return await _runtime.browser_once(self._wrapped, event_name, **kwargs)

    async def remove_listener(self, event_name, **kwargs):
        return await _runtime.browser_remove_listener(self._wrapped, event_name, **kwargs)

    async def unbind(self, event_name, **kwargs):
        return await _runtime.browser_remove_listener(self._wrapped, event_name, **kwargs)

class BrowserContext:
    """Playwright-style browser context (modelled as browser + init scripts)."""

    def __init__(self, wrapped):
        self._wrapped = wrapped

    async def new_page(self, url="about:blank", **kwargs):
        result = await _runtime.context_new_page(self._wrapped, url, **kwargs)
        return Page(result)

    async def add_init_script(self, script, **kwargs):
        return await _runtime.context_add_init_script(self._wrapped, script, **kwargs)

    async def close(self, **kwargs):
        return await _runtime.context_close(self._wrapped, **kwargs)

    async def pages(self, **kwargs):
        return await _runtime.context_pages(self._wrapped, **kwargs)

    async def add_cookies(self, cookies_json, **kwargs):
        return await _runtime.context_add_cookies(self._wrapped, cookies_json, **kwargs)

    async def clear_cookies(self, **kwargs):
        return await _runtime.context_clear_cookies(self._wrapped, **kwargs)

    async def cookies(self, **kwargs):
        return await _runtime.context_cookies(self._wrapped, **kwargs)

    async def grant_permissions(self, origin, permissions_json, **kwargs):
        return await _runtime.context_grant_permissions(self._wrapped, origin, permissions_json, **kwargs)

    async def clear_permissions(self, **kwargs):
        return await _runtime.context_clear_permissions(self._wrapped, **kwargs)

    async def route(self, pattern, action="continue", **kwargs):
        return await _runtime.context_route(self._wrapped, pattern, action, **kwargs)

    async def unroute(self, pattern, **kwargs):
        return await _runtime.context_unroute(self._wrapped, pattern, **kwargs)

    async def route_from_har(self, path, **kwargs):
        return await _runtime.page_route_from_har(self._wrapped, path, **kwargs)

    async def set_default_navigation_timeout(self, milliseconds, **kwargs):
        return await _runtime.context_set_default_timeout(self._wrapped, milliseconds, **kwargs)

    async def set_default_timeout(self, milliseconds, **kwargs):
        return await _runtime.context_set_default_timeout(self._wrapped, milliseconds, **kwargs)

    async def set_extra_http_headers(self, headers_json, **kwargs):
        return await _runtime.context_set_extra_http_headers(self._wrapped, headers_json, **kwargs)

    async def expect_page(self, **kwargs):
        return await _runtime.context_expect_page(self._wrapped, **kwargs)

    async def expect_event(self, event_name, **kwargs):
        return await _runtime.context_wait_for_event(self._wrapped, event_name, **kwargs)

    async def wait_for_event(self, event_name, **kwargs):
        return await _runtime.context_wait_for_event(self._wrapped, event_name, **kwargs)

    async def storage_state(self, **kwargs):
        return await _runtime.context_storage_state(self._wrapped, **kwargs)

    async def background_pages(self, **kwargs):
        return await _runtime.context_background_pages(self._wrapped, **kwargs)

    async def expect_console_message(self, **kwargs):
        return await _runtime.context_expect_console_message(self._wrapped, **kwargs)

    async def is_closed(self, **kwargs):
        return await _runtime.context_is_closed(self._wrapped, **kwargs)

    async def new_cdp_session(self, **kwargs):
        result = await _runtime.context_new_cdp_session(self._wrapped, **kwargs)
        return CDPSession(result)

    async def on(self, event_name, **kwargs):
        return await _runtime.context_on(self._wrapped, event_name, **kwargs)

    async def once(self, event_name, **kwargs):
        return await _runtime.context_once(self._wrapped, event_name, **kwargs)

    async def remove_listener(self, event_name, **kwargs):
        return await _runtime.context_remove_listener(self._wrapped, event_name, **kwargs)

    async def set_geolocation(self, latitude, longitude, accuracy, **kwargs):
        return await _runtime.context_set_geolocation(self._wrapped, latitude, longitude, accuracy, **kwargs)

    async def set_offline(self, offline, **kwargs):
        return await _runtime.context_set_offline(self._wrapped, offline, **kwargs)

    async def set_storage_state(self, state_json, **kwargs):
        return await _runtime.context_set_storage_state(self._wrapped, state_json, **kwargs)

    async def unroute_all(self, **kwargs):
        return await _runtime.context_unroute_all(self._wrapped, **kwargs)

class Page:
    """Playwright-style page."""

    def __init__(self, wrapped):
        self._wrapped = wrapped

    async def goto(self, url, **kwargs):
        return await _runtime.page_goto(self._wrapped, url, **kwargs)

    async def reload(self, **kwargs):
        return await _runtime.page_reload(self._wrapped, **kwargs)

    async def go_back(self, **kwargs):
        return await _runtime.page_go_back(self._wrapped, **kwargs)

    async def title(self, **kwargs):
        return await _runtime.page_title(self._wrapped, **kwargs)

    async def content(self, **kwargs):
        return await _runtime.page_content(self._wrapped, **kwargs)

    async def pdf(self, **kwargs):
        return await _runtime.page_pdf(self._wrapped, **kwargs)

    async def screenshot(self, full_page=False, path=None, **kwargs):
        return await _runtime.page_screenshot(self._wrapped, full_page, path, **kwargs)

    async def wait_for_selector(self, selector, **kwargs):
        result = await _runtime.page_wait_for_selector(self._wrapped, selector, **kwargs)
        return Locator(result)

    async def wait_for_load_state(self, state="load", **kwargs):
        return await _runtime.page_wait_for_load_state(self._wrapped, state, **kwargs)

    async def wait_for_timeout(self, milliseconds, **kwargs):
        return await _runtime.page_wait_for_timeout(self._wrapped, milliseconds, **kwargs)

    async def locator(self, selector, **kwargs):
        result = await _runtime.page_find(self._wrapped, selector, **kwargs)
        return Locator(result)

    async def query_selector(self, selector, **kwargs):
        result = await _runtime.page_find(self._wrapped, selector, **kwargs)
        return Locator(result)

    async def get_by_test_id(self, test_id, **kwargs):
        result = await _runtime.page_get_by_test_id(self._wrapped, test_id, **kwargs)
        return Locator(result)

    async def get_by_placeholder(self, text, **kwargs):
        result = await _runtime.page_get_by_placeholder(self._wrapped, text, **kwargs)
        return Locator(result)

    async def get_by_alt_text(self, text, **kwargs):
        result = await _runtime.page_get_by_alt_text(self._wrapped, text, **kwargs)
        return Locator(result)

    async def get_by_title(self, text, **kwargs):
        result = await _runtime.page_get_by_title(self._wrapped, text, **kwargs)
        return Locator(result)

    async def click(self, selector, **kwargs):
        return await _runtime.page_click(self._wrapped, selector, **kwargs)

    async def dblclick(self, selector, **kwargs):
        return await _runtime.page_dblclick(self._wrapped, selector, **kwargs)

    async def fill(self, selector, value, **kwargs):
        return await _runtime.page_fill(self._wrapped, selector, value, **kwargs)

    async def type(self, selector, value, **kwargs):
        return await _runtime.page_type(self._wrapped, selector, value, **kwargs)

    async def hover(self, selector, **kwargs):
        return await _runtime.page_hover(self._wrapped, selector, **kwargs)

    async def focus(self, selector, **kwargs):
        return await _runtime.page_focus(self._wrapped, selector, **kwargs)

    async def inner_text(self, selector, **kwargs):
        return await _runtime.page_inner_text(self._wrapped, selector, **kwargs)

    async def text_content(self, selector, **kwargs):
        return await _runtime.page_inner_text(self._wrapped, selector, **kwargs)

    async def inner_html(self, selector, **kwargs):
        return await _runtime.page_inner_html(self._wrapped, selector, **kwargs)

    async def get_attribute(self, selector, name, **kwargs):
        return await _runtime.page_get_attribute(self._wrapped, selector, name, **kwargs)

    async def add_script_tag(self, content, **kwargs):
        return await _runtime.page_add_script(self._wrapped, content, **kwargs)

    async def go_forward(self, **kwargs):
        return await _runtime.page_go_forward(self._wrapped, **kwargs)

    async def close(self, **kwargs):
        return await _runtime.page_close(self._wrapped, **kwargs)

    async def set_content(self, html, **kwargs):
        return await _runtime.call_js(self._wrapped, 'page', 'function(h){document.open();document.write(h);document.close();}', 'void', [html], **kwargs)

    async def url(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'page', 'function(){return location.href;}', 'string', [], **kwargs)

    async def get_by_role(self, role, **kwargs):
        result = await _runtime.page_get_by_role(self._wrapped, role, **kwargs)
        return Locator(result)

    async def get_by_text(self, text, **kwargs):
        result = await _runtime.page_get_by_text(self._wrapped, text, **kwargs)
        return Locator(result)

    async def get_by_label(self, label, **kwargs):
        result = await _runtime.page_get_by_label(self._wrapped, label, **kwargs)
        return Locator(result)

    async def press(self, selector, key, **kwargs):
        return await _runtime.call_js(self._wrapped, 'page', 'function(sel,key){const e=document.querySelector(sel);if(!e)return;e.focus();e.dispatchEvent(new KeyboardEvent("keydown",{key:key,bubbles:true}));e.dispatchEvent(new KeyboardEvent("keyup",{key:key,bubbles:true}));}', 'void', [selector, key], **kwargs)

    async def check(self, selector, **kwargs):
        return await _runtime.call_js(self._wrapped, 'page', 'function(sel){const e=document.querySelector(sel);if(e&&e.checked!==true)e.click();}', 'void', [selector], **kwargs)

    async def uncheck(self, selector, **kwargs):
        return await _runtime.call_js(self._wrapped, 'page', 'function(sel){const e=document.querySelector(sel);if(e&&e.checked===true)e.click();}', 'void', [selector], **kwargs)

    async def set_checked(self, selector, checked, **kwargs):
        return await _runtime.call_js(self._wrapped, 'page', 'function(sel,v){const e=document.querySelector(sel);if(e&&e.checked!==!!v)e.click();}', 'void', [selector, checked], **kwargs)

    async def set_input_files(self, selector, files_json, **kwargs):
        return await _runtime.page_set_input_files(self._wrapped, selector, files_json, **kwargs)

    async def select_option(self, selector, values_json, **kwargs):
        return await _runtime.page_select_option(self._wrapped, selector, values_json, **kwargs)

    async def tap(self, selector, **kwargs):
        return await _runtime.call_js(self._wrapped, 'page', 'function(sel){const e=document.querySelector(sel);if(e)e.click();}', 'void', [selector], **kwargs)

    async def dispatch_event(self, selector, event_type, **kwargs):
        return await _runtime.call_js(self._wrapped, 'page', 'function(sel,t){const e=document.querySelector(sel);if(e)e.dispatchEvent(new Event(t,{bubbles:true,cancelable:true}));}', 'void', [selector, event_type], **kwargs)

    async def drag_and_drop(self, source, target, **kwargs):
        return await _runtime.call_js(self._wrapped, 'page', 'function(src,dst){const s=document.querySelector(src),t=document.querySelector(dst);if(!s||!t)return;const dt=new DataTransfer();s.dispatchEvent(new DragEvent("dragstart",{bubbles:true,dataTransfer:dt}));t.dispatchEvent(new DragEvent("dragenter",{bubbles:true,dataTransfer:dt}));t.dispatchEvent(new DragEvent("dragover",{bubbles:true,dataTransfer:dt}));t.dispatchEvent(new DragEvent("drop",{bubbles:true,dataTransfer:dt}));s.dispatchEvent(new DragEvent("dragend",{bubbles:true,dataTransfer:dt}));}', 'void', [source, target], **kwargs)

    async def input_value(self, selector, **kwargs):
        return await _runtime.call_js(self._wrapped, 'page', 'function(sel){const e=document.querySelector(sel);return e&&e.value!=null?String(e.value):"";}', 'string', [selector], **kwargs)

    async def is_visible(self, selector, **kwargs):
        return await _runtime.call_js(self._wrapped, 'page', 'function(sel){const e=document.querySelector(sel);if(!e)return false;const s=getComputedStyle(e);const r=e.getBoundingClientRect();return !!(r.width||r.height)&&s.visibility!=="hidden"&&s.display!=="none";}', 'bool', [selector], **kwargs)

    async def is_hidden(self, selector, **kwargs):
        return await _runtime.call_js(self._wrapped, 'page', 'function(sel){const e=document.querySelector(sel);if(!e)return true;const s=getComputedStyle(e);const r=e.getBoundingClientRect();return !(!!(r.width||r.height)&&s.visibility!=="hidden"&&s.display!=="none");}', 'bool', [selector], **kwargs)

    async def is_enabled(self, selector, **kwargs):
        return await _runtime.call_js(self._wrapped, 'page', 'function(sel){const e=document.querySelector(sel);return !!e&&!(e.disabled===true||e.hasAttribute("disabled"));}', 'bool', [selector], **kwargs)

    async def is_disabled(self, selector, **kwargs):
        return await _runtime.call_js(self._wrapped, 'page', 'function(sel){const e=document.querySelector(sel);return !!e&&(e.disabled===true||e.hasAttribute("disabled"));}', 'bool', [selector], **kwargs)

    async def is_checked(self, selector, **kwargs):
        return await _runtime.call_js(self._wrapped, 'page', 'function(sel){const e=document.querySelector(sel);return !!e&&e.checked===true;}', 'bool', [selector], **kwargs)

    async def is_editable(self, selector, **kwargs):
        return await _runtime.call_js(self._wrapped, 'page', 'function(sel){const e=document.querySelector(sel);return !!e&&!(e.disabled===true||e.readOnly===true||e.hasAttribute("readonly"));}', 'bool', [selector], **kwargs)

    async def wait_for_url(self, url, **kwargs):
        return await _runtime.call_js(self._wrapped, 'page', 'function(u){return new Promise(res=>{const check=()=>{if(location.href===u||location.href.indexOf(u)===0){res(true);}else{setTimeout(check,50);}};check();});}', 'void', [url], **kwargs)

    async def wait_for_event(self, event_name, **kwargs):
        return await _runtime.page_wait_for_event(self._wrapped, event_name, **kwargs)

    async def wait_for_function(self, expression, **kwargs):
        return await _runtime.page_wait_for_function(self._wrapped, expression, **kwargs)

    async def add_style_tag(self, content, **kwargs):
        return await _runtime.call_js(self._wrapped, 'page', "function(c){const s=document.createElement('style');s.textContent=c;document.head.appendChild(s);return s.textContent;}", 'string', [content], **kwargs)

    async def evaluate(self, expression, **kwargs):
        return await _runtime.call_js(self._wrapped, 'page', 'function(src){return (new Function("return ("+src+")"))();}', 'json', [expression], **kwargs)

    async def eval_on_selector(self, selector, expression, **kwargs):
        return await _runtime.page_eval_on_selector(self._wrapped, selector, expression, **kwargs)

    async def eval_on_selector_all(self, selector, expression, **kwargs):
        return await _runtime.page_eval_on_selector_all(self._wrapped, selector, expression, **kwargs)

    async def route(self, pattern, action="continue", **kwargs):
        return await _runtime.page_route(self._wrapped, pattern, action, **kwargs)

    async def unroute(self, pattern, **kwargs):
        return await _runtime.page_unroute(self._wrapped, pattern, **kwargs)

    async def emulate_media(self, media, color_scheme, **kwargs):
        return await _runtime.page_emulate_media(self._wrapped, media, color_scheme, **kwargs)

    async def set_viewport_size(self, width, height, **kwargs):
        return await _runtime.page_set_viewport_size(self._wrapped, width, height, **kwargs)

    async def set_extra_http_headers(self, headers_json, **kwargs):
        return await _runtime.page_set_extra_http_headers(self._wrapped, headers_json, **kwargs)

    async def bring_to_front(self, **kwargs):
        return await _runtime.page_bring_to_front(self._wrapped, **kwargs)

    async def keyboard(self, **kwargs):
        result = await _runtime.page_new_keyboard(self._wrapped, **kwargs)
        return Keyboard(result)

    async def mouse(self, **kwargs):
        result = await _runtime.page_new_mouse(self._wrapped, **kwargs)
        return Mouse(result)

    async def touchscreen(self, **kwargs):
        result = await _runtime.page_new_touchscreen(self._wrapped, **kwargs)
        return Touchscreen(result)

    async def frames(self, **kwargs):
        return await _runtime.page_frames(self._wrapped, **kwargs)

    async def main_frame(self, **kwargs):
        return await _runtime.page_main_frame(self._wrapped, **kwargs)

    async def on(self, event_name, **kwargs):
        return await _runtime.page_on(self._wrapped, event_name, **kwargs)

    async def once(self, event_name, **kwargs):
        return await _runtime.page_once(self._wrapped, event_name, **kwargs)

    async def remove_listener(self, event_name, **kwargs):
        return await _runtime.page_remove_listener(self._wrapped, event_name, **kwargs)

    async def expect_popup(self, **kwargs):
        return await _runtime.page_expect_popup(self._wrapped, **kwargs)

    async def expect_download(self, **kwargs):
        return await _runtime.page_expect_download(self._wrapped, **kwargs)

    async def expect_request(self, **kwargs):
        return await _runtime.page_expect_request(self._wrapped, **kwargs)

    async def expect_response(self, **kwargs):
        return await _runtime.page_expect_response(self._wrapped, **kwargs)

    async def expect_console_message(self, **kwargs):
        return await _runtime.page_expect_console_message(self._wrapped, **kwargs)

    async def expect_file_chooser(self, **kwargs):
        return await _runtime.page_expect_file_chooser(self._wrapped, **kwargs)

    async def expect_event(self, event_name, **kwargs):
        return await _runtime.page_wait_for_event(self._wrapped, event_name, **kwargs)

    async def add_init_script(self, script, **kwargs):
        return await _runtime.page_add_script(self._wrapped, script, **kwargs)

    async def aria_snapshot(self, **kwargs):
        return await _runtime.cdp_call(self._wrapped, 'Accessibility.getFullAXTree', {  }, 'json', **kwargs)

    async def clear_console_messages(self, **kwargs):
        return await _runtime.cdp_call(self._wrapped, 'Runtime.discardConsoleEntries', {  }, 'void', **kwargs)

    async def clear_page_errors(self, **kwargs):
        return await _runtime.cdp_call(self._wrapped, 'Runtime.discardConsoleEntries', {  }, 'void', **kwargs)

    async def evaluate_handle(self, expression, **kwargs):
        result = await _runtime.page_evaluate_handle(self._wrapped, expression, **kwargs)
        return Locator(result)

    async def expect_navigation(self, **kwargs):
        return await _runtime.page_expect_navigation(self._wrapped, **kwargs)

    async def expect_request_finished(self, **kwargs):
        return await _runtime.page_expect_request_finished(self._wrapped, **kwargs)

    async def expect_websocket(self, **kwargs):
        return await _runtime.page_expect_websocket(self._wrapped, **kwargs)

    async def expect_worker(self, **kwargs):
        return await _runtime.page_expect_worker(self._wrapped, **kwargs)

    async def frame(self, frame_id, **kwargs):
        return await _runtime.page_frame(self._wrapped, frame_id, **kwargs)

    async def hide_highlight(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'page', 'function(){document.querySelectorAll("*").forEach(function(e){e.style.outline="";});}', 'void', [], **kwargs)

    async def is_closed(self, **kwargs):
        return await _runtime.page_is_closed(self._wrapped, **kwargs)

    async def local_storage(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'page', 'function(){return Object.fromEntries(Object.entries(localStorage));}', 'json', [], **kwargs)

    async def query_selector_all(self, selector, **kwargs):
        result = await _runtime.page_find_all(self._wrapped, selector, **kwargs)
        return [Locator(item) for item in result]

    async def request(self, **kwargs):
        return await _runtime.page_request(self._wrapped, **kwargs)

    async def request_gc(self, **kwargs):
        return await _runtime.cdp_call(self._wrapped, 'HeapProfiler.collectGarbage', {  }, 'void', **kwargs)

    async def requests(self, **kwargs):
        return await _runtime.page_requests(self._wrapped, **kwargs)

    async def route_from_har(self, path, **kwargs):
        return await _runtime.page_route_from_har(self._wrapped, path, **kwargs)

    async def screencast(self, **kwargs):
        result = await _runtime.page_new_screencast(self._wrapped, **kwargs)
        return Screencast(result)

    async def session_storage(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'page', 'function(){return Object.fromEntries(Object.entries(sessionStorage));}', 'json', [], **kwargs)

    async def set_default_navigation_timeout(self, milliseconds, **kwargs):
        return await _runtime.page_set_default_timeout(self._wrapped, milliseconds, **kwargs)

    async def set_default_timeout(self, milliseconds, **kwargs):
        return await _runtime.page_set_default_timeout(self._wrapped, milliseconds, **kwargs)

    async def unroute_all(self, **kwargs):
        return await _runtime.page_unroute_all(self._wrapped, **kwargs)

    async def viewport_size(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'page', 'function(){return {width:window.innerWidth,height:window.innerHeight};}', 'json', [], **kwargs)

class Locator:
    """Playwright-style locator (wraps an element handle)."""

    def __init__(self, wrapped):
        self._wrapped = wrapped

    async def click(self, **kwargs):
        return await _runtime.element_click(self._wrapped, **kwargs)

    async def dblclick(self, **kwargs):
        return await _runtime.element_dblclick(self._wrapped, **kwargs)

    async def fill(self, value, **kwargs):
        return await _runtime.element_type(self._wrapped, value, **kwargs)

    async def type(self, value, **kwargs):
        return await _runtime.element_type(self._wrapped, value, **kwargs)

    async def hover(self, **kwargs):
        return await _runtime.element_hover(self._wrapped, **kwargs)

    async def focus(self, **kwargs):
        return await _runtime.element_focus(self._wrapped, **kwargs)

    async def inner_text(self, **kwargs):
        return await _runtime.element_text(self._wrapped, **kwargs)

    async def text_content(self, **kwargs):
        return await _runtime.element_text(self._wrapped, **kwargs)

    async def get_attribute(self, name, **kwargs):
        return await _runtime.element_attribute(self._wrapped, name, **kwargs)

    async def inner_html(self, **kwargs):
        return await _runtime.element_inner_html(self._wrapped, **kwargs)

    async def press(self, key, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(key){this.focus();this.dispatchEvent(new KeyboardEvent("keydown",{key:key,bubbles:true}));this.dispatchEvent(new KeyboardEvent("keyup",{key:key,bubbles:true}));}', 'void', [key], **kwargs)

    async def check(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){if(this.checked!==true&&this.type!==undefined){this.click();}}', 'void', [], **kwargs)

    async def uncheck(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){if(this.checked===true){this.click();}}', 'void', [], **kwargs)

    async def set_checked(self, checked, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(v){if(this.checked!==!!v){this.click();}}', 'void', [checked], **kwargs)

    async def select_option(self, values_json, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', "function(valuesRaw){const w=JSON.parse(valuesRaw).map(String);for(const o of this.options){o.selected=w.includes(o.value)||w.includes(o.text);}this.dispatchEvent(new Event('change',{bubbles:true}));}", 'void', [values_json], **kwargs)

    async def input_value(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){return this.value!=null?String(this.value):"";}', 'string', [], **kwargs)

    async def is_visible(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){const s=getComputedStyle(this);const r=this.getBoundingClientRect();return !!(r.width||r.height)&&s.visibility!=="hidden"&&s.display!=="none"&&s.opacity!=="0";}', 'bool', [], **kwargs)

    async def is_hidden(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){const s=getComputedStyle(this);const r=this.getBoundingClientRect();return !(!!(r.width||r.height)&&s.visibility!=="hidden"&&s.display!=="none");}', 'bool', [], **kwargs)

    async def is_enabled(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){return !(this.disabled===true||this.hasAttribute("disabled"));}', 'bool', [], **kwargs)

    async def is_disabled(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){return this.disabled===true||this.hasAttribute("disabled");}', 'bool', [], **kwargs)

    async def is_checked(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){return this.checked===true;}', 'bool', [], **kwargs)

    async def is_editable(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){return !(this.disabled===true||this.readOnly===true||this.hasAttribute("readonly"));}', 'bool', [], **kwargs)

    async def is_empty(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){return !this.value&&!this.textContent&&!this.innerHTML;}', 'bool', [], **kwargs)

    async def count(self, **kwargs):
        return await _runtime.element_count(self._wrapped, **kwargs)

    async def all(self, **kwargs):
        result = await _runtime.element_all(self._wrapped, **kwargs)
        return [Locator(item) for item in result]

    async def all_inner_texts(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){return [this.innerText];}', 'json', [], **kwargs)

    async def all_text_contents(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){return [this.textContent];}', 'json', [], **kwargs)

    async def first(self, **kwargs):
        result = await _runtime.element_self(self._wrapped, **kwargs)
        return Locator(result)

    async def last(self, **kwargs):
        result = await _runtime.element_self(self._wrapped, **kwargs)
        return Locator(result)

    async def nth(self, index, **kwargs):
        result = await _runtime.element_nth(self._wrapped, index, **kwargs)
        return Locator(result)

    async def get_by_role(self, role, **kwargs):
        result = await _runtime.element_get_by_role(self._wrapped, role, **kwargs)
        return Locator(result)

    async def get_by_text(self, text, **kwargs):
        result = await _runtime.element_get_by_text(self._wrapped, text, **kwargs)
        return Locator(result)

    async def get_by_label(self, label, **kwargs):
        result = await _runtime.element_get_by_label(self._wrapped, label, **kwargs)
        return Locator(result)

    async def get_by_placeholder(self, text, **kwargs):
        result = await _runtime.element_get_by_placeholder(self._wrapped, text, **kwargs)
        return Locator(result)

    async def get_by_alt_text(self, text, **kwargs):
        result = await _runtime.element_get_by_alt_text(self._wrapped, text, **kwargs)
        return Locator(result)

    async def get_by_title(self, text, **kwargs):
        result = await _runtime.element_get_by_title(self._wrapped, text, **kwargs)
        return Locator(result)

    async def get_by_test_id(self, test_id, **kwargs):
        result = await _runtime.element_get_by_test_id(self._wrapped, test_id, **kwargs)
        return Locator(result)

    async def evaluate(self, expression, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', "function(src){return (new Function('el','return ('+src+')(el);'))(this);}", 'json', [expression], **kwargs)

    async def screenshot(self, **kwargs):
        return await _runtime.element_screenshot(self._wrapped, **kwargs)

    async def set_input_files(self, files_json, **kwargs):
        return await _runtime.element_set_input_files(self._wrapped, files_json, **kwargs)

    async def drag_to(self, target, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(sel){const t=document.querySelector(sel);if(!t)return;const dt=new DataTransfer();this.dispatchEvent(new DragEvent("dragstart",{bubbles:true,dataTransfer:dt}));t.dispatchEvent(new DragEvent("dragenter",{bubbles:true,dataTransfer:dt}));t.dispatchEvent(new DragEvent("dragover",{bubbles:true,dataTransfer:dt}));t.dispatchEvent(new DragEvent("drop",{bubbles:true,dataTransfer:dt}));this.dispatchEvent(new DragEvent("dragend",{bubbles:true,dataTransfer:dt}));}', 'void', [target], **kwargs)

    async def blur(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){this.blur();}', 'void', [], **kwargs)

    async def bounding_box(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){const r=this.getBoundingClientRect();return {x:r.x,y:r.y,width:r.width,height:r.height};}', 'json', [], **kwargs)

    async def clear(self, **kwargs):
        return await _runtime.element_clear(self._wrapped, **kwargs)

    async def describe(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){return this.outerHTML.slice(0,120);}', 'string', [], **kwargs)

    async def description(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){return this.outerHTML;}', 'string', [], **kwargs)

    async def dispatch_event(self, event_type, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(t){this.dispatchEvent(new Event(t,{bubbles:true,cancelable:true}));}', 'void', [event_type], **kwargs)

    async def drop(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){this.dispatchEvent(new DragEvent("drop",{bubbles:true}));}', 'void', [], **kwargs)

    async def element_handle(self, **kwargs):
        result = await _runtime.element_self(self._wrapped, **kwargs)
        return Locator(result)

    async def element_handles(self, **kwargs):
        result = await _runtime.element_all(self._wrapped, **kwargs)
        return [Locator(item) for item in result]

    async def evaluate_handle(self, expression, **kwargs):
        result = await _runtime.element_evaluate_handle(self._wrapped, expression, **kwargs)
        return Locator(result)

    async def hide_highlight(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){this.style.outline="";}', 'void', [], **kwargs)

    async def highlight(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){this.style.outline="2px solid red";}', 'void', [], **kwargs)

    async def press_sequentially(self, text, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(text){this.focus();this.value=(this.value||"")+text;this.dispatchEvent(new Event("input",{bubbles:true}));}', 'void', [text], **kwargs)

    async def scroll_into_view_if_needed(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){this.scrollIntoView({block:"center",inline:"center"});}', 'void', [], **kwargs)

    async def select_text(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){const r=document.createRange();r.selectNodeContents(this);const s=getSelection();s.removeAllRanges();s.addRange(r);}', 'void', [], **kwargs)

    async def tap(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){this.click();}', 'void', [], **kwargs)

    async def visible(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){const s=getComputedStyle(this);const r=this.getBoundingClientRect();return !!(r.width||r.height)&&s.visibility!=="hidden"&&s.display!=="none"&&s.opacity!=="0";}', 'bool', [], **kwargs)

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

class Screencast:
    """Screencast (thin wrapper over the page)."""

    def __init__(self, wrapped):
        self._wrapped = wrapped

    async def start(self, **kwargs):
        return await _runtime.page_start_screencast(self._wrapped, **kwargs)

    async def stop(self, **kwargs):
        return await _runtime.page_stop_screencast(self._wrapped, **kwargs)


def use():
    """Return a namespace exposing the Playwright (async API) API style."""
    return types.SimpleNamespace(
        launch=launch,
        Browser=Browser,
        BrowserContext=BrowserContext,
        Page=Page,
        Locator=Locator,
        CDPSession=CDPSession,
        Keyboard=Keyboard,
        Mouse=Mouse,
        Touchscreen=Touchscreen,
        Screencast=Screencast,
    )
