"""Generated adapter - do not edit by hand.

Source profile: adapters/profiles/selenium.json
Regenerate with: python scripts/generate_adapters.py
"""

from __future__ import annotations

import types

from . import _runtime

async def launch(config=None, **kwargs):
    result = await _runtime.selenium_launch(config, **kwargs)
    return WebDriver(result)

class WebDriver:
    """Selenium-style driver. Owns navigation, element lookup, cookies and window management."""

    def __init__(self, wrapped):
        self._wrapped = wrapped

    async def get(self, url, **kwargs):
        return await _runtime.selenium_get(self._wrapped, url, **kwargs)

    async def find_element(self, by="css selector", value=None, **kwargs):
        result = await _runtime.selenium_find(self._wrapped, by, value, **kwargs)
        return WebElement(result)

    async def title(self, **kwargs):
        return await _runtime.selenium_title(self._wrapped, **kwargs)

    async def page_source(self, **kwargs):
        return await _runtime.selenium_page_source(self._wrapped, **kwargs)

    async def refresh(self, **kwargs):
        return await _runtime.selenium_refresh(self._wrapped, **kwargs)

    async def back(self, **kwargs):
        return await _runtime.selenium_back(self._wrapped, **kwargs)

    async def close(self, **kwargs):
        return await _runtime.selenium_close(self._wrapped, **kwargs)

    async def quit(self, **kwargs):
        return await _runtime.selenium_quit(self._wrapped, **kwargs)

    async def save_screenshot(self, path=None, full_page=False, **kwargs):
        return await _runtime.selenium_screenshot(self._wrapped, path, full_page, **kwargs)

    async def get_screenshot_as_file(self, filename=None, full_page=False, **kwargs):
        return await _runtime.selenium_screenshot(self._wrapped, filename, full_page, **kwargs)

    async def get_screenshot_as_png(self, **kwargs):
        return await _runtime.selenium_screenshot_png(self._wrapped, **kwargs)

    async def get_screenshot_as_base64(self, **kwargs):
        return await _runtime.selenium_screenshot_base64(self._wrapped, **kwargs)

    async def find_elements(self, by, value, **kwargs):
        result = await _runtime.selenium_find_all(self._wrapped, by, value, **kwargs)
        return [WebElement(item) for item in result]

    async def execute_script(self, script, **kwargs):
        return await _runtime.selenium_execute_script(self._wrapped, script, **kwargs)

    async def execute_async_script(self, script, **kwargs):
        return await _runtime.selenium_execute_async_script(self._wrapped, script, **kwargs)

    async def execute(self, script, **kwargs):
        return await _runtime.selenium_execute_script(self._wrapped, script, **kwargs)

    async def forward(self, **kwargs):
        return await _runtime.selenium_forward(self._wrapped, **kwargs)

    async def get_window_size(self, **kwargs):
        return await _runtime.selenium_window_size(self._wrapped, **kwargs)

    async def set_window_size(self, width, height, **kwargs):
        return await _runtime.selenium_set_window_size(self._wrapped, width, height, **kwargs)

    async def get_window_position(self, **kwargs):
        return await _runtime.selenium_window_position(self._wrapped, **kwargs)

    async def set_window_position(self, x, y, **kwargs):
        return await _runtime.selenium_set_window_position(self._wrapped, x, y, **kwargs)

    async def get_window_rect(self, **kwargs):
        return await _runtime.selenium_window_rect(self._wrapped, **kwargs)

    async def set_window_rect(self, x, y, width, height, **kwargs):
        return await _runtime.selenium_set_window_rect(self._wrapped, x, y, width, height, **kwargs)

    async def maximize_window(self, **kwargs):
        return await _runtime.selenium_maximize_window(self._wrapped, **kwargs)

    async def minimize_window(self, **kwargs):
        return await _runtime.selenium_minimize_window(self._wrapped, **kwargs)

    async def fullscreen_window(self, **kwargs):
        return await _runtime.selenium_fullscreen_window(self._wrapped, **kwargs)

    async def get_cookies(self, **kwargs):
        return await _runtime.selenium_get_cookies(self._wrapped, **kwargs)

    async def get_cookie(self, name, **kwargs):
        return await _runtime.selenium_get_cookie(self._wrapped, name, **kwargs)

    async def add_cookie(self, cookie_json, **kwargs):
        return await _runtime.selenium_add_cookie(self._wrapped, cookie_json, **kwargs)

    async def delete_cookie(self, name, **kwargs):
        return await _runtime.selenium_delete_cookie(self._wrapped, name, **kwargs)

    async def delete_all_cookies(self, **kwargs):
        return await _runtime.selenium_delete_all_cookies(self._wrapped, **kwargs)

    async def implicitly_wait(self, seconds, **kwargs):
        return await _runtime.selenium_implicitly_wait(self._wrapped, seconds, **kwargs)

    async def set_page_load_timeout(self, seconds, **kwargs):
        return await _runtime.selenium_set_page_load_timeout(self._wrapped, seconds, **kwargs)

    async def set_script_timeout(self, seconds, **kwargs):
        return await _runtime.selenium_set_script_timeout(self._wrapped, seconds, **kwargs)

    async def set_page_load_strategy(self, strategy, **kwargs):
        return await _runtime.selenium_set_page_load_strategy(self._wrapped, strategy, **kwargs)

    async def print_page(self, **kwargs):
        return await _runtime.selenium_print_page(self._wrapped, **kwargs)

    async def current_url(self, **kwargs):
        return await _runtime.selenium_current_url(self._wrapped, **kwargs)

    async def window_handles(self, **kwargs):
        return await _runtime.selenium_window_handles(self._wrapped, **kwargs)

    async def current_window_handle(self, **kwargs):
        return await _runtime.selenium_current_window_handle(self._wrapped, **kwargs)

    async def switch_to(self, **kwargs):
        result = await _runtime.driver_new_switch_to(self._wrapped, **kwargs)
        return SwitchTo(result)

    async def capabilities(self, **kwargs):
        return await _runtime.selenium_capabilities(self._wrapped, **kwargs)

    async def desired_capabilities(self, **kwargs):
        return await _runtime.selenium_capabilities(self._wrapped, **kwargs)

    async def active_element(self, **kwargs):
        result = await _runtime.selenium_active_element(self._wrapped, **kwargs)
        return WebElement(result)

    async def dialog(self, **kwargs):
        result = await _runtime.driver_new_dialog(self._wrapped, **kwargs)
        return Dialog(result)

    async def execute_cdp_cmd(self, method, params_json, **kwargs):
        return await _runtime.selenium_execute_cdp_cmd(self._wrapped, method, params_json, **kwargs)

    async def timeouts(self, **kwargs):
        result = await _runtime.driver_new_timeouts(self._wrapped, **kwargs)
        return Timeouts(result)

class WebElement:
    """Selenium-style element."""

    def __init__(self, wrapped):
        self._wrapped = wrapped

    async def click(self, **kwargs):
        return await _runtime.element_click(self._wrapped, **kwargs)

    async def send_keys(self, value, **kwargs):
        return await _runtime.element_send_keys(self._wrapped, value, **kwargs)

    async def clear(self, **kwargs):
        return await _runtime.element_clear(self._wrapped, **kwargs)

    async def text(self, **kwargs):
        return await _runtime.element_text(self._wrapped, **kwargs)

    async def get_attribute(self, name, **kwargs):
        return await _runtime.element_attribute(self._wrapped, name, **kwargs)

    async def get_dom_attribute(self, name, **kwargs):
        return await _runtime.element_attribute(self._wrapped, name, **kwargs)

    async def inner_html(self, **kwargs):
        return await _runtime.element_inner_html(self._wrapped, **kwargs)

    async def submit(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){if(this.form){this.form.submit();}else if(this.tagName==="FORM"){this.submit();}}', 'void', [], **kwargs)

    async def get_property(self, name, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(n){const v=this[n];return v==null?"":String(v);}', 'string', [name], **kwargs)

    async def is_selected(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){return this.selected===true||this.checked===true;}', 'bool', [], **kwargs)

    async def is_enabled(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){return !(this.disabled===true||this.hasAttribute("disabled"));}', 'bool', [], **kwargs)

    async def is_displayed(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){const s=getComputedStyle(this);const r=this.getBoundingClientRect();return !!(r.width||r.height)&&s.visibility!=="hidden"&&s.display!=="none"&&s.opacity!=="0";}', 'bool', [], **kwargs)

    async def find_element(self, selector, **kwargs):
        result = await _runtime.element_find(self._wrapped, selector, **kwargs)
        return WebElement(result)

    async def find_elements(self, selector, **kwargs):
        result = await _runtime.element_find_all(self._wrapped, selector, **kwargs)
        return [WebElement(item) for item in result]

    async def value_of_css_property(self, property_name, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(p){return getComputedStyle(this).getPropertyValue(p);}', 'string', [property_name], **kwargs)

    async def screenshot(self, **kwargs):
        return await _runtime.element_screenshot(self._wrapped, **kwargs)

    async def screenshot_as_png(self, **kwargs):
        return await _runtime.element_screenshot(self._wrapped, **kwargs)

    async def screenshot_as_base64(self, **kwargs):
        return await _runtime.element_screenshot_base64(self._wrapped, **kwargs)

    async def tag_name(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){return this.tagName?this.tagName.toLowerCase():"unknown";}', 'string', [], **kwargs)

    async def location(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){const r=this.getBoundingClientRect();return {x:r.x,y:r.y,width:r.width,height:r.height};}', 'json', [], **kwargs)

    async def location_once_scrolled_into_view(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){this.scrollIntoView({block:"center",inline:"center"});const r=this.getBoundingClientRect();return {x:r.x,y:r.y};}', 'json', [], **kwargs)

    async def size(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){const r=this.getBoundingClientRect();return {width:r.width,height:r.height};}', 'json', [], **kwargs)

    async def rect(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){const r=this.getBoundingClientRect();return {x:r.x,y:r.y,width:r.width,height:r.height};}', 'json', [], **kwargs)

    async def parent(self, **kwargs):
        result = await _runtime.element_parent(self._wrapped, **kwargs)
        return WebElement(result)

    async def id(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', "function(){return this.id||'';}", 'string', [], **kwargs)

    async def shadow_root(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){return this.shadowRoot?this.shadowRoot.innerHTML:"";}', 'string', [], **kwargs)

    async def aria_role(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){return this.getAttribute("role")||this.tagName.toLowerCase();}', 'string', [], **kwargs)

    async def accessible_name(self, **kwargs):
        return await _runtime.call_js(self._wrapped, 'element', 'function(){return (this.getAttribute("aria-label")||this.textContent||"").trim();}', 'string', [], **kwargs)

class SwitchTo:
    """Selenium-style target switching (thin wrapper over the driver)."""

    def __init__(self, wrapped):
        self._wrapped = wrapped

    async def window(self, handle, **kwargs):
        return await _runtime.driver_activate_target(self._wrapped, handle, **kwargs)

    async def default_content(self, **kwargs):
        return await _runtime.driver_switch_default_content(self._wrapped, **kwargs)

    async def new_window(self, window_type, **kwargs):
        return await _runtime.driver_switch_new_window(self._wrapped, window_type, **kwargs)

class Timeouts:
    """Selenium-style timeouts (thin wrapper over the driver)."""

    def __init__(self, wrapped):
        self._wrapped = wrapped

    async def implicitly_wait(self, seconds, **kwargs):
        return await _runtime.selenium_implicitly_wait(self._wrapped, seconds, **kwargs)

    async def set_script_timeout(self, seconds, **kwargs):
        return await _runtime.selenium_set_script_timeout(self._wrapped, seconds, **kwargs)

    async def page_load_timeout(self, seconds, **kwargs):
        return await _runtime.selenium_set_page_load_timeout(self._wrapped, seconds, **kwargs)

class Dialog:
    """JavaScript dialog (thin wrapper over the driver)."""

    def __init__(self, wrapped):
        self._wrapped = wrapped

    async def accept(self, **kwargs):
        return await _runtime.driver_dialog_accept(self._wrapped, **kwargs)

    async def dismiss(self, **kwargs):
        return await _runtime.driver_dialog_dismiss(self._wrapped, **kwargs)

    async def send_keys(self, text, **kwargs):
        return await _runtime.driver_dialog_send_keys(self._wrapped, text, **kwargs)


def use():
    """Return a namespace exposing the Selenium WebDriver API style."""
    return types.SimpleNamespace(
        launch=launch,
        WebDriver=WebDriver,
        WebElement=WebElement,
        SwitchTo=SwitchTo,
        Timeouts=Timeouts,
        Dialog=Dialog,
    )
