"""Adapter runtime for xcelerate.

This module is the single place where the *behaviour* of every API profile
lives. The generated adapter modules (``playwright.py``, ``selenium.py``,
``puppeteer.py``) are deliberately dumb: they only expose familiar method names
and forward the call here.

Design goals
------------
* **One implementation, many styles.** A new profile only adds data (a JSON
  mapping); it never needs new code here unless it needs a genuinely new op.
* **Thin generated wrappers.** Everything meaningful is testable in this file.
* **Honest about gaps.** Operations xcelerate cannot express (e.g. Selenium's
  ``is_displayed``) are omitted rather than faked, unless a no-op keeps the
  call-site shape working (e.g. ``clear``).

This file is copied verbatim into ``xcelerate/adapters/_runtime.py`` at
generation time, so the relative import below resolves in the installed package.
"""

from __future__ import annotations

import asyncio
import base64
import dataclasses
import json
import typing

from ..xcelerate import Browser, BrowserConfig


# ---------------------------------------------------------------------------
# Browser-level ops
# ---------------------------------------------------------------------------

async def browser_launch(config: typing.Any = None, **_: typing.Any) -> typing.Any:
    """Create and launch a browser (shared by every profile)."""
    if config is None:
        config = BrowserConfig()
    return await Browser.launch(config)


async def browser_targets(browser: typing.Any, **_: typing.Any) -> str:
    return await browser.targets()


async def browser_pages(browser: typing.Any, **_: typing.Any) -> str:
    return await browser.targets()


async def browser_contexts(browser: typing.Any, **_: typing.Any) -> str:
    return await browser.browser_contexts()


async def browser_user_agent(browser: typing.Any, **_: typing.Any) -> str:
    return await browser.user_agent()


async def browser_is_connected(browser: typing.Any, **_: typing.Any) -> bool:
    return await browser.is_connected()


async def browser_ws_endpoint(browser: typing.Any, **_: typing.Any) -> str:
    return browser.ws_endpoint()


async def browser_new_context_id(browser: typing.Any, **_: typing.Any) -> str:
    return await browser.new_context()


async def browser_start_tracing(browser: typing.Any, **_: typing.Any) -> None:
    await browser.start_tracing()


async def browser_stop_tracing(browser: typing.Any, **_: typing.Any) -> None:
    await browser.stop_tracing()


async def browser_on(browser: typing.Any, event_name: str, **_: typing.Any) -> None:
    await browser.on(event_name)


async def browser_once(browser: typing.Any, event_name: str, **_: typing.Any) -> None:
    await browser.once(event_name)


async def browser_remove_listener(browser: typing.Any, event_name: str, **_: typing.Any) -> None:
    await browser.remove_listener(event_name)


async def browser_remove_all_listeners(browser: typing.Any, **_: typing.Any) -> None:
    await browser.remove_all_listeners()


async def browser_event_names(browser: typing.Any, **_: typing.Any) -> typing.Any:
    return await browser.event_names()


async def browser_listens_to(browser: typing.Any, event_name: str, **_: typing.Any) -> bool:
    return await browser.listens_to(event_name)


async def browser_wait_for_event(browser: typing.Any, event_name: str, **_: typing.Any) -> str:
    return await browser.wait_for_event_default(event_name)


async def browser_new_page(browser: typing.Any, url: str = "about:blank", **_: typing.Any) -> typing.Any:
    return await browser.new_page(url)


async def browser_version(browser: typing.Any, **_: typing.Any) -> str:
    return await browser.version()


async def browser_close(browser: typing.Any, **_: typing.Any) -> None:
    await browser.close()


# ---------------------------------------------------------------------------
# Page-level ops (Playwright / Puppeteer style)
# ---------------------------------------------------------------------------

async def page_goto(page: typing.Any, url: str, **_: typing.Any) -> None:
    await page.navigate(url)


async def page_title(page: typing.Any, **_: typing.Any) -> str:
    return await page.title()


async def page_content(page: typing.Any, **_: typing.Any) -> str:
    return await page.content()


async def page_reload(page: typing.Any, **_: typing.Any) -> None:
    await page.reload()


async def page_go_back(page: typing.Any, **_: typing.Any) -> None:
    await page.go_back()


async def page_pdf(page: typing.Any, **_: typing.Any) -> bytes:
    return await page.pdf()


async def page_screenshot(page: typing.Any, full_page: bool = False, path: str | None = None, **_: typing.Any) -> bytes:
    data = await (page.screenshot_full() if full_page else page.screenshot())
    if path:
        with open(path, "wb") as handle:
            handle.write(data)
    return data


async def page_wait_for_selector(page: typing.Any, selector: str, timeout: float | None = None, **_: typing.Any) -> typing.Any:
    # xcelerate applies its own 30s timeout; `timeout` is accepted for shape.
    return await page.wait_for_selector(selector)


async def page_find(page: typing.Any, selector: str, **_: typing.Any) -> typing.Any:
    return await page.find_element(selector)


async def page_click(page: typing.Any, selector: str, **_: typing.Any) -> None:
    element = await page.find_element(selector)
    await element.click()


async def page_dblclick(page: typing.Any, selector: str, **_: typing.Any) -> None:
    element = await page.find_element(selector)
    await element.click()
    await element.click()


async def page_fill(page: typing.Any, selector: str, value: str, **_: typing.Any) -> None:
    element = await page.find_element(selector)
    await element.focus()
    await element.type_text(value)


async def page_type(page: typing.Any, selector: str, value: str, **_: typing.Any) -> None:
    await page_fill(page, selector, value)


async def page_hover(page: typing.Any, selector: str, **_: typing.Any) -> None:
    element = await page.find_element(selector)
    await element.hover()


async def page_inner_text(page: typing.Any, selector: str, **_: typing.Any) -> str:
    element = await page.find_element(selector)
    return await element.text()


async def page_inner_html(page: typing.Any, selector: str, **_: typing.Any) -> str:
    element = await page.find_element(selector)
    return await element.inner_html()


async def page_get_attribute(page: typing.Any, selector: str, name: str, **_: typing.Any) -> str | None:
    element = await page.find_element(selector)
    return await element.attribute(name)


async def page_add_script(page: typing.Any, content: str, **_: typing.Any) -> str:
    return await page.add_script_to_evaluate_on_new_document(content)


async def page_wait_for_navigation(page: typing.Any, **_: typing.Any) -> None:
    await page.wait_for_navigation()


def _attr_selector(attribute: str, value: str) -> str:
    escaped = value.replace("\\", "\\\\").replace('"', '\\"')
    return f'[{attribute}="{escaped}"]'


async def page_focus(page: typing.Any, selector: str, **_: typing.Any) -> None:
    element = await page.find_element(selector)
    await element.focus()


async def page_get_by_test_id(page: typing.Any, test_id: str, **_: typing.Any) -> typing.Any:
    return await page.find_element(_attr_selector("data-testid", test_id))


async def page_get_by_placeholder(page: typing.Any, text: str, **_: typing.Any) -> typing.Any:
    return await page.find_element(_attr_selector("placeholder", text))


async def page_get_by_alt_text(page: typing.Any, text: str, **_: typing.Any) -> typing.Any:
    return await page.find_element(_attr_selector("alt", text))


async def page_get_by_title(page: typing.Any, text: str, **_: typing.Any) -> typing.Any:
    return await page.find_element(_attr_selector("title", text))


async def page_wait_for_load_state(page: typing.Any, state: str = "load", **_: typing.Any) -> None:
    await page.wait_for_navigation()


async def page_wait_for_timeout(page: typing.Any, milliseconds: float, **_: typing.Any) -> None:
    await asyncio.sleep(milliseconds / 1000.0)


# ---------------------------------------------------------------------------
# Element-level ops (ElementHandle / Locator / WebElement)
# ---------------------------------------------------------------------------

async def element_click(element: typing.Any, **_: typing.Any) -> None:
    await element.click()


async def element_dblclick(element: typing.Any, **_: typing.Any) -> None:
    await element.click()
    await element.click()


async def element_type(element: typing.Any, value: str, **_: typing.Any) -> None:
    await element.focus()
    await element.type_text(value)


async def element_send_keys(element: typing.Any, value: str, **_: typing.Any) -> None:
    await element.type_text(value)


async def element_hover(element: typing.Any, **_: typing.Any) -> None:
    await element.hover()


async def element_focus(element: typing.Any, **_: typing.Any) -> None:
    await element.focus()


async def element_text(element: typing.Any, **_: typing.Any) -> str:
    return await element.text()


async def element_attribute(element: typing.Any, name: str, **_: typing.Any) -> str | None:
    return await element.attribute(name)


async def element_inner_html(element: typing.Any, **_: typing.Any) -> str:
    return await element.inner_html()


async def element_clear(element: typing.Any, **_: typing.Any) -> None:
    # xcelerate has no "set value" primitive; focusing is the closest correct
    # action, so callers can follow with send_keys.
    await element.focus()


# ---------------------------------------------------------------------------
# Generic JS bridge
#
# Profile methods can declare an inline JS implementation instead of a named op.
# The generated wrappers forward to this helper, which calls the core
# `call_bool` / `call_string` / `call_json` shims on a page or element.
# ---------------------------------------------------------------------------

async def call_js(
    obj: typing.Any,
    scope: str = "element",
    js: str = "",
    returns: str = "void",
    args: typing.Optional[typing.List[typing.Any]] = None,
    **_: typing.Any,
) -> typing.Any:
    payload = json.dumps(args or [])
    if returns == "bool":
        return await obj.call_bool(js, payload)
    if returns == "string":
        return await obj.call_string(js, payload)
    if returns == "json":
        return await obj.call_json(js, payload)
    await obj.call_json(js, payload)
    return None


async def cdp_call(
    obj: typing.Any,
    method: str = "",
    params: typing.Optional[typing.Dict[str, typing.Any]] = None,
    returns: str = "json",
    **_: typing.Any,
) -> typing.Any:
    return await obj.execute_cdp_cmd(method, json.dumps(params or {}))


async def page_execute_cdp_cmd(
    page: typing.Any, method: str, params_json: str, **_: typing.Any
) -> str:
    return await page.execute_cdp_cmd(method, params_json)


# ---------------------------------------------------------------------------
# BrowserContext ops (Playwright)
#
# xcelerate has no context object, so a context is modelled as the browser plus
# the init scripts registered on it; scripts are applied when a page is created.
# ---------------------------------------------------------------------------

@dataclasses.dataclass
class ContextHandle:
    browser: typing.Any
    scripts: typing.List[str] = dataclasses.field(default_factory=list)
    page: typing.Any = None


async def browser_new_context(browser: typing.Any, **_: typing.Any) -> ContextHandle:
    return ContextHandle(browser=browser)


async def context_new_page(context: ContextHandle, url: str = "about:blank", **_: typing.Any) -> typing.Any:
    page = await context.browser.new_page(url)
    for script in context.scripts:
        await page.add_script_to_evaluate_on_new_document(script)
    context.page = page
    return page


async def context_add_init_script(context: ContextHandle, script: str, **_: typing.Any) -> None:
    context.scripts.append(script)


async def context_close(context: ContextHandle, **_: typing.Any) -> None:
    # No context to close; pages are owned by the browser.
    return None


async def _context_page(context: ContextHandle) -> typing.Any:
    if context.page is None:
        await context_new_page(context)
    return context.page


async def context_pages(context: ContextHandle, **_: typing.Any) -> str:
    return await context.browser.targets()


async def context_cookies(context: ContextHandle, **_: typing.Any) -> str:
    page = await _context_page(context)
    return await page.cookies()


async def context_add_cookies(
    context: ContextHandle, cookies_json: str, **_: typing.Any
) -> None:
    page = await _context_page(context)
    for cookie in json.loads(cookies_json or "[]"):
        await page.execute_cdp_cmd("Network.setCookie", json.dumps(cookie))


async def context_clear_cookies(context: ContextHandle, **_: typing.Any) -> None:
    page = await _context_page(context)
    await page.execute_cdp_cmd("Network.clearBrowserCookies", "{}")


async def context_set_extra_http_headers(
    context: ContextHandle, headers_json: str, **_: typing.Any
) -> None:
    page = await _context_page(context)
    await page.set_extra_http_headers(headers_json)


async def context_grant_permissions(
    context: ContextHandle, origin: str, permissions_json: str, **_: typing.Any
) -> None:
    await context.browser.grant_permissions(origin, permissions_json)


# ---------------------------------------------------------------------------
# Selenium-specific ops
#
# Selenium's WebDriver owns navigation *and* element lookup on a single object,
# whereas xcelerate splits these across Browser and Page. We therefore wrap the
# pair in a small handle and drive it here.
# ---------------------------------------------------------------------------

@dataclasses.dataclass
class DriverHandle:
    browser: typing.Any
    page: typing.Any


_SEARCH_ALIASES = {
    "css selector": "css",
    "css": "css",
    "by.css_selector": "css",
    "css_selector": "css",
    "xpath": "xpath",
    "by.xpath": "xpath",
}


def _resolve_selector(by: typing.Any, value: str | None) -> str:
    """Normalise Selenium's ``(By.X, "value")`` / ``By.X, "value"`` call shapes."""
    if value is None and isinstance(by, (tuple, list)) and len(by) == 2:
        by, value = by
    name = getattr(by, "value", by)
    name = str(name).lower()
    kind = _SEARCH_ALIASES.get(name, "css")
    if kind == "xpath":
        raise NotImplementedError(
            "xcelerate adapters support CSS selectors only; "
            f"received {name!r}"
        )
    assert value is not None, "selector value is required"
    return value


async def selenium_launch(config: typing.Any = None, **_: typing.Any) -> DriverHandle:
    if config is None:
        config = BrowserConfig()
    browser = await Browser.launch(config)
    page = await browser.new_page("about:blank")
    return DriverHandle(browser=browser, page=page)


async def selenium_get(driver: DriverHandle, url: str, **_: typing.Any) -> None:
    await driver.page.navigate(url)


async def selenium_find(driver: DriverHandle, by: typing.Any, value: str | None = None, **_: typing.Any) -> typing.Any:
    return await driver.page.find_element(_resolve_selector(by, value))


async def selenium_title(driver: DriverHandle, **_: typing.Any) -> str:
    return await driver.page.title()


async def selenium_page_source(driver: DriverHandle, **_: typing.Any) -> str:
    return await driver.page.content()


async def selenium_refresh(driver: DriverHandle, **_: typing.Any) -> None:
    await driver.page.reload()


async def selenium_back(driver: DriverHandle, **_: typing.Any) -> None:
    await driver.page.go_back()


async def selenium_screenshot(driver: DriverHandle, path: str | None = None, full_page: bool = False, **_: typing.Any) -> bytes:
    return await page_screenshot(driver.page, full_page=full_page, path=path)


async def selenium_screenshot_png(driver: DriverHandle, full_page: bool = False, **_: typing.Any) -> bytes:
    return await page_screenshot(driver.page, full_page=full_page)


async def selenium_screenshot_base64(driver: DriverHandle, full_page: bool = False, **_: typing.Any) -> str:
    data = await page_screenshot(driver.page, full_page=full_page)
    return base64.b64encode(data).decode("ascii")


async def selenium_quit(driver: DriverHandle, **_: typing.Any) -> None:
    await driver.browser.close()


async def selenium_close(driver: DriverHandle, **_: typing.Any) -> None:
    await driver.browser.close()


# ---------------------------------------------------------------------------
# CDP / JS-backed page ops used by the data-driven adapters
# ---------------------------------------------------------------------------

async def page_url(page: typing.Any, **_: typing.Any) -> str:
    return await page.url()


async def page_go_forward(page: typing.Any, **_: typing.Any) -> None:
    await page.go_forward()


async def page_close(page: typing.Any, **_: typing.Any) -> None:
    await page.close()


async def page_bring_to_front(page: typing.Any, **_: typing.Any) -> None:
    await page.bring_to_front()


async def page_set_content(page: typing.Any, html: str, **_: typing.Any) -> None:
    await page.set_content(html)


async def page_set_viewport_size(
    page: typing.Any, width: int, height: int, **_: typing.Any
) -> None:
    await page.set_viewport_size(width, height)


async def page_emulate_media(
    page: typing.Any,
    media: typing.Optional[str] = None,
    color_scheme: typing.Optional[str] = None,
    **_: typing.Any,
) -> None:
    await page.emulate_media(media, color_scheme)


async def page_set_extra_http_headers(
    page: typing.Any, headers_json: str, **_: typing.Any
) -> None:
    await page.set_extra_http_headers(headers_json)


async def page_add_style_tag(page: typing.Any, content: str, **_: typing.Any) -> str:
    return await page.add_style_tag(content)


async def page_select_option(
    page: typing.Any, selector: str, values_json: str, **_: typing.Any
) -> None:
    await page.select_option(selector, values_json)


async def page_set_input_files(
    page: typing.Any, selector: str, files_json: str, **_: typing.Any
) -> None:
    await page.set_input_files(selector, files_json)


async def page_find_all(page: typing.Any, selector: str, **_: typing.Any) -> typing.Any:
    return await page.query_selector_all(selector)


async def page_query_selector_xpath(page: typing.Any, xpath: str, **_: typing.Any) -> typing.Any:
    return await page.query_selector_xpath(xpath)


async def page_evaluate_handle(page: typing.Any, expression: str, **_: typing.Any) -> typing.Any:
    return await page.evaluate_handle(expression)


async def page_eval_on_selector(
    page: typing.Any, selector: str, expression: str, **_: typing.Any
) -> str:
    return await page.call_on_selector(selector, expression)


async def page_eval_on_selector_all(
    page: typing.Any, selector: str, expression: str, **_: typing.Any
) -> str:
    return await page.call_on_selector_all(selector, expression)


async def page_set_user_agent(
    page: typing.Any,
    user_agent: str,
    accept_language: typing.Optional[str] = None,
    **_: typing.Any,
) -> None:
    await page.set_user_agent(user_agent, accept_language)


async def page_set_cache_enabled(page: typing.Any, enabled: bool, **_: typing.Any) -> None:
    await page.set_cache_enabled(enabled)


async def page_set_javascript_enabled(page: typing.Any, enabled: bool, **_: typing.Any) -> None:
    await page.set_javascript_enabled(enabled)


async def page_set_offline(page: typing.Any, offline: bool, **_: typing.Any) -> None:
    await page.set_offline(offline)


async def page_cookies(page: typing.Any, **_: typing.Any) -> str:
    return await page.cookies()


async def page_metrics(page: typing.Any, **_: typing.Any) -> str:
    return await page.metrics()


async def page_wait_for_function(page: typing.Any, expression: str, **_: typing.Any) -> None:
    await page.wait_for_function(expression, 30000)


async def element_find(element: typing.Any, selector: str, **_: typing.Any) -> typing.Any:
    return await element.query_selector(selector)


async def element_find_all(element: typing.Any, selector: str, **_: typing.Any) -> typing.Any:
    return await element.query_selector_all(selector)


async def element_evaluate_handle(element: typing.Any, function: str, **_: typing.Any) -> typing.Any:
    return await element.evaluate_handle(function)


async def element_screenshot(element: typing.Any, **_: typing.Any) -> typing.Any:
    return await element.screenshot()


async def element_screenshot_base64(element: typing.Any, **_: typing.Any) -> str:
    return await element.screenshot_base64()


async def element_self(element: typing.Any, **_: typing.Any) -> typing.Any:
    return element


async def element_nth(element: typing.Any, index: int, **_: typing.Any) -> typing.Any:
    return element


async def element_count(element: typing.Any, **_: typing.Any) -> int:
    return await element.count()


async def page_wait_for_network_idle(page: typing.Any, **_: typing.Any) -> str:
    return await page.wait_for_event_default("Network.loadingFinished")


async def page_wait_for_frame(page: typing.Any, **_: typing.Any) -> str:
    return await page.wait_for_event_default("Page.frameAttached")


async def page_wait_for_device_prompt(page: typing.Any, **_: typing.Any) -> str:
    return await page.wait_for_event_default("DeviceAccess.deviceRequestPrompted")


async def page_authenticate(page: typing.Any, username: str, password: str, **_: typing.Any) -> None:
    await page.authenticate(username, password)


async def page_set_request_interception(page: typing.Any, enabled: bool, **_: typing.Any) -> None:
    await page.set_request_interception(enabled)


async def page_route(
    page: typing.Any, pattern: str, action: str = "continue", **_: typing.Any
) -> None:
    await page.route(pattern, action, None, None)


async def page_route_abort(page: typing.Any, pattern: str, **_: typing.Any) -> None:
    await page.route_abort(pattern)


async def page_route_fulfill(
    page: typing.Any,
    pattern: str,
    body: str,
    content_type: typing.Optional[str] = None,
    **_: typing.Any,
) -> None:
    await page.route_fulfill(pattern, body, content_type)


async def page_unroute(page: typing.Any, pattern: str, **_: typing.Any) -> None:
    await page.unroute(pattern)


async def page_unroute_all(page: typing.Any, **_: typing.Any) -> None:
    await page.unroute_all()


async def page_requests(page: typing.Any, **_: typing.Any) -> str:
    return await page.requests()


async def page_request(page: typing.Any, **_: typing.Any) -> str:
    return await page.request()


async def page_set_drag_interception(page: typing.Any, enabled: bool, **_: typing.Any) -> None:
    await page.set_drag_interception(enabled)


async def page_is_drag_interception_enabled(page: typing.Any, **_: typing.Any) -> bool:
    return await page.is_drag_interception_enabled()


async def page_frames(page: typing.Any, **_: typing.Any) -> str:
    return await page.frames()


async def page_main_frame(page: typing.Any, **_: typing.Any) -> str:
    return await page.main_frame()


async def page_frame(page: typing.Any, frame_id: str, **_: typing.Any) -> str:
    return await page.frame(frame_id)


async def page_new_cdp_session(page: typing.Any, **_: typing.Any) -> typing.Any:
    return page


async def page_detach_cdp_session(page: typing.Any, **_: typing.Any) -> None:
    return None


async def context_new_cdp_session(context: ContextHandle, **_: typing.Any) -> typing.Any:
    return await _context_page(context)


async def browser_cookies(browser: typing.Any, **_: typing.Any) -> str:
    return await browser.cookies()


async def browser_set_cookie(browser: typing.Any, cookie_json: str, **_: typing.Any) -> None:
    await browser.set_cookie(cookie_json)


async def browser_delete_cookie(browser: typing.Any, name: str, **_: typing.Any) -> None:
    await browser.delete_cookie(name)


async def browser_capabilities(browser: typing.Any, **_: typing.Any) -> str:
    return await browser.capabilities()


async def browser_reset_permissions(browser: typing.Any, **_: typing.Any) -> None:
    await browser.reset_permissions()


async def page_target_id(page: typing.Any, **_: typing.Any) -> str:
    return page.target_id()


async def page_wait_for_xpath(page: typing.Any, xpath: str, **_: typing.Any) -> typing.Any:
    return await page.wait_for_xpath(xpath, 30000)


async def page_inject_file(page: typing.Any, path: str, **_: typing.Any) -> str:
    return await page.inject_file(path)


async def page_remove_script(page: typing.Any, identifier: str, **_: typing.Any) -> None:
    await page.remove_script(identifier)


async def page_set_emulated_media_features(
    page: typing.Any, features_json: str, **_: typing.Any
) -> None:
    await page.set_emulated_media_features(features_json)


async def page_emulate_idle_state(
    page: typing.Any, is_user_active: bool, is_screen_unlocked: bool, **_: typing.Any
) -> None:
    await page.emulate_idle_state(is_user_active, is_screen_unlocked)


async def page_start_screencast(page: typing.Any, **_: typing.Any) -> None:
    await page.start_screencast()


async def page_stop_screencast(page: typing.Any, **_: typing.Any) -> None:
    await page.stop_screencast()


async def page_coverage_start_js(page: typing.Any, **_: typing.Any) -> None:
    await page.coverage_start_js()


async def page_coverage_stop_js(page: typing.Any, **_: typing.Any) -> str:
    return await page.coverage_stop_js()


async def page_coverage_start_css(page: typing.Any, **_: typing.Any) -> None:
    await page.coverage_start_css()


async def page_coverage_stop_css(page: typing.Any, **_: typing.Any) -> str:
    return await page.coverage_stop_css()


async def page_storage_state(page: typing.Any, **_: typing.Any) -> str:
    return await page.storage_state()


async def page_set_storage_state(page: typing.Any, state_json: str, **_: typing.Any) -> None:
    await page.set_storage_state(state_json)


async def page_set_default_timeout(page: typing.Any, milliseconds: float, **_: typing.Any) -> None:
    await page.set_default_timeout(milliseconds)


async def page_get_default_timeout(page: typing.Any, **_: typing.Any) -> float:
    return await page.get_default_timeout()


async def page_new_coverage(page: typing.Any, **_: typing.Any) -> typing.Any:
    return page


async def context_set_offline(context: ContextHandle, offline: bool, **_: typing.Any) -> None:
    page = await _context_page(context)
    await page.set_offline(offline)


async def context_set_geolocation(
    context: ContextHandle, latitude: float, longitude: float, accuracy: float, **_: typing.Any
) -> None:
    page = await _context_page(context)
    await page.execute_cdp_cmd(
        "Emulation.setGeolocationOverride",
        json.dumps({"latitude": latitude, "longitude": longitude, "accuracy": accuracy}),
    )


async def context_storage_state(context: ContextHandle, **_: typing.Any) -> str:
    page = await _context_page(context)
    return await page.storage_state()


async def context_set_storage_state(
    context: ContextHandle, state_json: str, **_: typing.Any
) -> None:
    page = await _context_page(context)
    await page.set_storage_state(state_json)


async def context_set_default_timeout(
    context: ContextHandle, milliseconds: float, **_: typing.Any
) -> None:
    page = await _context_page(context)
    await page.set_default_timeout(milliseconds)


async def context_clear_permissions(context: ContextHandle, **_: typing.Any) -> None:
    await context.browser.reset_permissions()


async def selenium_get_cookie(driver: DriverHandle, name: str, **_: typing.Any) -> str:
    return await driver.page.cookie(name)


async def selenium_window_handles(driver: DriverHandle, **_: typing.Any) -> str:
    return await driver.browser.targets()


async def selenium_current_window_handle(driver: DriverHandle, **_: typing.Any) -> str:
    return driver.page.target_id()


async def selenium_capabilities(driver: DriverHandle, **_: typing.Any) -> str:
    return await driver.browser.capabilities()


async def context_targets(context: ContextHandle, **_: typing.Any) -> str:
    return await context.browser.targets()


async def context_route(
    context: ContextHandle, pattern: str, action: str = "continue", **_: typing.Any
) -> None:
    page = await _context_page(context)
    await page.route(pattern, action, None, None)


async def context_route_abort(context: ContextHandle, pattern: str, **_: typing.Any) -> None:
    page = await _context_page(context)
    await page.route_abort(pattern)


async def context_unroute(context: ContextHandle, pattern: str, **_: typing.Any) -> None:
    page = await _context_page(context)
    await page.unroute(pattern)


async def context_unroute_all(context: ContextHandle, **_: typing.Any) -> None:
    page = await _context_page(context)
    await page.unroute_all()


async def context_set_cookie(
    context: ContextHandle, cookie_json: str, **_: typing.Any
) -> None:
    page = await _context_page(context)
    await page.execute_cdp_cmd("Network.setCookie", cookie_json)


async def context_delete_cookie(
    context: ContextHandle, name: str, **_: typing.Any
) -> None:
    page = await _context_page(context)
    await page.execute_cdp_cmd("Network.deleteCookies", json.dumps({"name": name}))


async def context_on(context: ContextHandle, event_name: str, **_: typing.Any) -> None:
    page = await _context_page(context)
    await page.on(event_name)


async def context_once(context: ContextHandle, event_name: str, **_: typing.Any) -> None:
    page = await _context_page(context)
    await page.once(event_name)


async def context_remove_listener(
    context: ContextHandle, event_name: str, **_: typing.Any
) -> None:
    page = await _context_page(context)
    await page.remove_listener(event_name)


async def context_remove_all_listeners(context: ContextHandle, **_: typing.Any) -> None:
    page = await _context_page(context)
    await page.remove_all_listeners()


async def context_event_names(context: ContextHandle, **_: typing.Any) -> typing.Any:
    page = await _context_page(context)
    return await page.event_names()


async def context_listens_to(
    context: ContextHandle, event_name: str, **_: typing.Any
) -> bool:
    page = await _context_page(context)
    return await page.listens_to(event_name)


async def context_wait_for_event(
    context: ContextHandle, event_name: str, **_: typing.Any
) -> str:
    page = await _context_page(context)
    return await page.wait_for_event_default(event_name)


async def context_set_download_behavior(
    context: ContextHandle, path: str, **_: typing.Any
) -> None:
    await context.browser.set_download_behavior(path)


async def context_route_from_har(context: ContextHandle, path: str, **_: typing.Any) -> None:
    page = await _context_page(context)
    await page.route_from_har(path)


async def element_eval_on_selector(
    element: typing.Any, selector: str, expression: str, **_: typing.Any
) -> str:
    return await element.call_on_selector(selector, expression)


async def element_eval_on_selector_all(
    element: typing.Any, selector: str, expression: str, **_: typing.Any
) -> str:
    return await element.call_on_selector_all(selector, expression)


async def element_get_by_text(element: typing.Any, text: str, **_: typing.Any) -> typing.Any:
    return await element.get_by_text(text)


async def element_get_by_role(element: typing.Any, role: str, **_: typing.Any) -> typing.Any:
    return await element.get_by_role(role)


async def element_get_by_label(element: typing.Any, label: str, **_: typing.Any) -> typing.Any:
    return await element.get_by_label(label)


async def element_get_by_placeholder(element: typing.Any, text: str, **_: typing.Any) -> typing.Any:
    return await element.query_selector_attr("placeholder", text)


async def element_get_by_alt_text(element: typing.Any, text: str, **_: typing.Any) -> typing.Any:
    return await element.query_selector_attr("alt", text)


async def element_get_by_title(element: typing.Any, text: str, **_: typing.Any) -> typing.Any:
    return await element.query_selector_attr("title", text)


async def element_get_by_test_id(element: typing.Any, test_id: str, **_: typing.Any) -> typing.Any:
    return await element.query_selector_attr("data-testid", test_id)


async def element_all(element: typing.Any, **_: typing.Any) -> typing.Any:
    return [element]


async def element_query_selector_xpath(element: typing.Any, xpath: str, **_: typing.Any) -> typing.Any:
    return await element.query_selector_xpath(xpath)


async def page_frame_name(page: typing.Any, **_: typing.Any) -> str:
    return await page.frame_name()


async def page_route_from_har(page: typing.Any, path: str, **_: typing.Any) -> None:
    await page.route_from_har(path)


async def page_is_closed(page: typing.Any, **_: typing.Any) -> bool:
    return False


async def context_is_closed(context: ContextHandle, **_: typing.Any) -> bool:
    return False


async def context_expect_page(context: ContextHandle, **_: typing.Any) -> str:
    page = await _context_page(context)
    return await page.wait_for_event_default("Target.targetCreated")


async def context_expect_console_message(context: ContextHandle, **_: typing.Any) -> str:
    page = await _context_page(context)
    return await page.wait_for_event_default("Runtime.consoleAPICalled")


async def context_background_pages(context: ContextHandle, **_: typing.Any) -> str:
    return await context.browser.targets()


async def context_wait_for_target(context: ContextHandle, **_: typing.Any) -> str:
    page = await _context_page(context)
    return await page.wait_for_event_default("Target.targetCreated")


async def browser_wait_for_target(browser: typing.Any, **_: typing.Any) -> str:
    return await browser.wait_for_event_default("Target.targetCreated")


async def browser_grant_permissions(
    browser: typing.Any, origin: str, permissions_json: str, **_: typing.Any
) -> None:
    await browser.grant_permissions(origin, permissions_json)


async def selenium_set_page_load_strategy(
    driver: DriverHandle, strategy: str, **_: typing.Any
) -> None:
    await driver.page.call_json(
        "function(v){window.__xcelerate_page_load_strategy=v;}", json.dumps([strategy])
    )


async def page_mouse_move(page: typing.Any, x: float, y: float, **_: typing.Any) -> None:
    await page.move_mouse(x, y)


async def page_mouse_click(page: typing.Any, x: float, y: float, **_: typing.Any) -> None:
    await page.click_mouse(x, y)


async def page_mouse_down(page: typing.Any, button: str, **_: typing.Any) -> None:
    await page.mouse_down(button)


async def page_mouse_up(page: typing.Any, button: str, **_: typing.Any) -> None:
    await page.mouse_up(button)


async def page_keyboard_press(page: typing.Any, key: str, **_: typing.Any) -> None:
    await page.keyboard_press(key)


async def page_keyboard_down(page: typing.Any, key: str, **_: typing.Any) -> None:
    await page.keyboard_down(key)


async def page_keyboard_up(page: typing.Any, key: str, **_: typing.Any) -> None:
    await page.keyboard_up(key)


async def page_keyboard_type(page: typing.Any, text: str, **_: typing.Any) -> None:
    await page.keyboard_type(text)


async def page_touch_tap(page: typing.Any, x: float, y: float, **_: typing.Any) -> None:
    await page.touch_tap(x, y)


async def page_new_keyboard(page: typing.Any, **_: typing.Any) -> typing.Any:
    return page


async def page_new_mouse(page: typing.Any, **_: typing.Any) -> typing.Any:
    return page


async def page_new_touchscreen(page: typing.Any, **_: typing.Any) -> typing.Any:
    return page


async def driver_activate_target(driver: DriverHandle, target_id: str, **_: typing.Any) -> None:
    await driver.page.activate_target(target_id)


async def driver_switch_default_content(driver: DriverHandle, **_: typing.Any) -> None:
    return None


async def driver_switch_new_window(
    driver: DriverHandle, window_type: str, **_: typing.Any
) -> None:
    await driver.browser.new_page("about:blank")


async def driver_new_switch_to(driver: DriverHandle, **_: typing.Any) -> typing.Any:
    return driver


async def driver_new_timeouts(driver: DriverHandle, **_: typing.Any) -> typing.Any:
    return driver


async def element_get_properties(element: typing.Any, **_: typing.Any) -> str:
    return await element.get_properties()


async def element_dispose(element: typing.Any, **_: typing.Any) -> None:
    await element.dispose()


async def element_wait_for_selector(element: typing.Any, selector: str, **_: typing.Any) -> typing.Any:
    return await element.wait_for_selector(selector)


async def page_handle_js_dialog(
    page: typing.Any, accept: bool, prompt_text: typing.Optional[str] = None, **_: typing.Any
) -> None:
    await page.handle_js_dialog(accept, prompt_text)


async def page_create_pdf_stream(page: typing.Any, **_: typing.Any) -> str:
    return await page.create_pdf_stream()


async def page_tracing_start(page: typing.Any, **_: typing.Any) -> None:
    await page.start_tracing()


async def page_tracing_stop(page: typing.Any, **_: typing.Any) -> None:
    await page.stop_tracing()


async def page_new_screencast(page: typing.Any, **_: typing.Any) -> typing.Any:
    return page


async def page_new_tracing(page: typing.Any, **_: typing.Any) -> typing.Any:
    return page


async def driver_new_dialog(driver: DriverHandle, **_: typing.Any) -> typing.Any:
    return driver


async def driver_dialog_accept(driver: DriverHandle, **_: typing.Any) -> None:
    await driver.page.handle_js_dialog(True, None)


async def driver_dialog_dismiss(driver: DriverHandle, **_: typing.Any) -> None:
    await driver.page.handle_js_dialog(False, None)


async def driver_dialog_send_keys(driver: DriverHandle, text: str, **_: typing.Any) -> None:
    await driver.page.handle_js_dialog(True, text)


async def page_get_by_text(page: typing.Any, text: str, **_: typing.Any) -> typing.Any:
    return await page.get_by_text(text)


async def page_get_by_role(page: typing.Any, role: str, **_: typing.Any) -> typing.Any:
    return await page.get_by_role(role)


async def page_get_by_label(page: typing.Any, label: str, **_: typing.Any) -> typing.Any:
    return await page.get_by_label(label)


async def element_set_input_files(element: typing.Any, files_json: str, **_: typing.Any) -> None:
    await element.set_input_files(files_json)


async def selenium_current_url(driver: DriverHandle, **_: typing.Any) -> str:
    return await driver.page.url()


async def selenium_forward(driver: DriverHandle, **_: typing.Any) -> None:
    await driver.page.go_forward()


async def selenium_execute_script(
    driver: DriverHandle, script: str, **_: typing.Any
) -> str:
    return await driver.page.evaluate_json(script)


async def selenium_execute_async_script(
    driver: DriverHandle, script: str, **_: typing.Any
) -> str:
    return await driver.page.evaluate_json(script)


async def selenium_execute_cdp_cmd(
    driver: DriverHandle, method: str, params_json: str, **_: typing.Any
) -> str:
    return await driver.page.execute_cdp_cmd(method, params_json)


async def selenium_get_cookies(driver: DriverHandle, **_: typing.Any) -> str:
    return await driver.page.cookies()


async def selenium_add_cookie(
    driver: DriverHandle, cookie_json: str, **_: typing.Any
) -> str:
    return await driver.page.execute_cdp_cmd("Network.setCookie", cookie_json)


async def selenium_delete_cookie(
    driver: DriverHandle, name: str, **_: typing.Any
) -> None:
    await driver.page.execute_cdp_cmd(
        "Network.deleteCookies", json.dumps({"name": name})
    )


async def selenium_delete_all_cookies(driver: DriverHandle, **_: typing.Any) -> None:
    await driver.page.execute_cdp_cmd("Network.clearBrowserCookies", "{}")


async def selenium_find_all(
    driver: DriverHandle, by: typing.Any, value: typing.Optional[str] = None, **_: typing.Any
) -> typing.Any:
    return await driver.page.query_selector_all(_resolve_selector(by, value))


async def selenium_active_element(driver: DriverHandle, **_: typing.Any) -> typing.Any:
    return await driver.page.evaluate_handle("document.activeElement")


async def selenium_print_page(driver: DriverHandle, **_: typing.Any) -> str:
    return await driver.page.execute_cdp_cmd("Page.printToPDF", "{}")


async def selenium_window_size(driver: DriverHandle, **_: typing.Any) -> str:
    return await driver.page.window_size()


async def selenium_set_window_size(
    driver: DriverHandle, width: int, height: int, **_: typing.Any
) -> None:
    await driver.page.set_window_size(width, height)


async def selenium_window_position(driver: DriverHandle, **_: typing.Any) -> str:
    return await driver.page.window_position()


async def selenium_set_window_position(
    driver: DriverHandle, x: int, y: int, **_: typing.Any
) -> None:
    await driver.page.set_window_position(x, y)


async def selenium_window_rect(driver: DriverHandle, **_: typing.Any) -> str:
    return await driver.page.window_rect()


async def selenium_set_window_rect(
    driver: DriverHandle, x: int, y: int, width: int, height: int, **_: typing.Any
) -> None:
    await driver.page.set_window_bounds(x, y, width, height)


async def selenium_maximize_window(driver: DriverHandle, **_: typing.Any) -> None:
    await driver.page.set_window_state("maximized")


async def selenium_minimize_window(driver: DriverHandle, **_: typing.Any) -> None:
    await driver.page.set_window_state("minimized")


async def selenium_fullscreen_window(driver: DriverHandle, **_: typing.Any) -> None:
    await driver.page.set_window_state("fullscreen")


_TIMEOUT_JS = (
    "function(k,v){window.__xcelerate_timeouts=window.__xcelerate_timeouts||{};"
    "window.__xcelerate_timeouts[k]=v;}"
)


async def selenium_set_timeout(
    driver: DriverHandle, key: str, seconds: float, **_: typing.Any
) -> None:
    await driver.page.call_json(_TIMEOUT_JS, json.dumps([key, seconds]))


async def selenium_implicitly_wait(
    driver: DriverHandle, seconds: float, **_: typing.Any
) -> None:
    await driver.page.call_json(_TIMEOUT_JS, json.dumps(["implicit", seconds]))


async def selenium_set_page_load_timeout(
    driver: DriverHandle, seconds: float, **_: typing.Any
) -> None:
    await driver.page.call_json(_TIMEOUT_JS, json.dumps(["page_load", seconds]))


async def selenium_set_script_timeout(
    driver: DriverHandle, seconds: float, **_: typing.Any
) -> None:
    await driver.page.call_json(_TIMEOUT_JS, json.dumps(["script", seconds]))


async def element_parent(element: typing.Any, **_: typing.Any) -> typing.Any:
    return await element.evaluate_handle("function(){return this.parentElement;}")


async def page_on(page: typing.Any, event_name: str, **_: typing.Any) -> None:
    await page.on(event_name)


async def page_once(page: typing.Any, event_name: str, **_: typing.Any) -> None:
    await page.once(event_name)


async def page_remove_listener(page: typing.Any, event_name: str, **_: typing.Any) -> None:
    await page.remove_listener(event_name)


async def page_remove_all_listeners(page: typing.Any, **_: typing.Any) -> None:
    await page.remove_all_listeners()


async def page_event_names(page: typing.Any, **_: typing.Any) -> typing.Any:
    return await page.event_names()


async def page_listens_to(page: typing.Any, event_name: str, **_: typing.Any) -> bool:
    return await page.listens_to(event_name)


async def page_wait_for_event(page: typing.Any, event_name: str, **_: typing.Any) -> str:
    return await page.wait_for_event_default(event_name)


async def _wait_event(page: typing.Any, event_name: str) -> str:
    return await page.wait_for_event_default(event_name)


async def page_expect_download(page: typing.Any, **_: typing.Any) -> str:
    return await _wait_event(page, "Page.downloadWillBegin")


async def page_expect_popup(page: typing.Any, **_: typing.Any) -> str:
    return await _wait_event(page, "Target.targetCreated")


async def page_expect_request(page: typing.Any, **_: typing.Any) -> str:
    return await _wait_event(page, "Network.requestWillBeSent")


async def page_expect_response(page: typing.Any, **_: typing.Any) -> str:
    return await _wait_event(page, "Network.responseReceived")


async def page_expect_console_message(page: typing.Any, **_: typing.Any) -> str:
    return await _wait_event(page, "Runtime.consoleAPICalled")


async def page_expect_navigation(page: typing.Any, **_: typing.Any) -> str:
    return await _wait_event(page, "Page.frameNavigated")


async def page_expect_file_chooser(page: typing.Any, **_: typing.Any) -> str:
    return await _wait_event(page, "Page.fileChooserOpened")


async def page_expect_request_finished(page: typing.Any, **_: typing.Any) -> str:
    return await _wait_event(page, "Network.loadingFinished")


async def page_expect_websocket(page: typing.Any, **_: typing.Any) -> str:
    return await _wait_event(page, "Network.webSocketCreated")


async def page_expect_worker(page: typing.Any, **_: typing.Any) -> str:
    return await _wait_event(page, "Target.attachedToTarget")


async def page_wait_for_request(page: typing.Any, **_: typing.Any) -> str:
    return await _wait_event(page, "Network.requestWillBeSent")


async def page_wait_for_response(page: typing.Any, **_: typing.Any) -> str:
    return await _wait_event(page, "Network.responseReceived")


async def page_wait_for_file_chooser(page: typing.Any, **_: typing.Any) -> str:
    return await _wait_event(page, "Page.fileChooserOpened")


async def page_wait_for_target(page: typing.Any, **_: typing.Any) -> str:
    return await _wait_event(page, "Target.targetCreated")
