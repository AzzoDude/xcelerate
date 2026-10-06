#!/usr/bin/env python3
"""Generate API-style adapters (Python + Rust) from declarative profiles.

The profiles in ``adapters/profiles/*.json`` map a target library's method names
onto the ops implemented in ``adapters/runtime.py``. This single script turns
each profile into:

* thin **Python** wrapper classes in ``bindings/python/xcelerate/adapters/``,
* typed **Rust** modules in ``crates/xcelerate/src/adapters/`` (one module per
  profile, plus the generated ``mod.rs`` and ``support.rs``).

Supported methods (those with an ``op``) are emitted with typed signatures that
call into the xcelerate core. Unsupported methods are emitted as explicit stubs
(``NotImplementedError`` / ``XcelerateError::Unsupported``) so the surface
matches the upstream library exactly without silently faking behaviour.

Usage::

    python scripts/generate_adapters.py                 # both targets
    python scripts/generate_adapters.py --target python
    python scripts/generate_adapters.py --target rust
"""

from __future__ import annotations

import argparse
import keyword
import os
import shutil
import sys

from common import (
    BINDINGS_DIR,
    RUNTIME_SRC,
    RUST_ADAPTERS_DIR,
    ROOT,
    load_profiles,
    log,
)

PY_OUT_DIR = os.path.join(BINDINGS_DIR, "python", "xcelerate")

PY_HEADER = (
    '"""Generated adapter - do not edit by hand.\n\n'
    "Source profile: {source}\n"
    "Regenerate with: python scripts/generate_adapters.py\n"
    '"""\n\n'
    "from __future__ import annotations\n\n"
    "import types\n\n"
    "from . import _runtime\n"
)

# ---------------------------------------------------------------------------
# Op table: op name -> Rust receiver kind, params, return shape and body.
#
# Receiver kinds: browser | page | element | driver | context | helper
# Return shapes:  unit | string | bytes | opt_string | wrap
# Bodies use @recv@, @0@..@N@ and @RET@ placeholders.
# ---------------------------------------------------------------------------
OPS = {
    # Browser -----------------------------------------------------------------
    "browser_launch": {
        "kind": "helper",
        "params": [("config", "Option<BrowserConfig>")],
        "ret": "wrap",
        "body": "Ok(@RET@(CoreBrowser::launch(@0@.unwrap_or_default()).await?))",
    },
    "browser_new_page": {
        "kind": "browser",
        "params": [("url", "String")],
        "ret": "wrap",
        "body": "Ok(@RET@(Arc::clone(&@recv@).new_page(@0@).await?))",
    },
    "browser_version": {
        "kind": "browser",
        "params": [],
        "ret": "string",
        "body": "Ok(@recv@.version().await?)",
    },
    "browser_close": {
        "kind": "browser",
        "params": [],
        "ret": "unit",
        "body": "@recv@.close().await?;\nOk(())",
    },
    "browser_new_context": {
        "kind": "browser",
        "params": [],
        "ret": "wrap",
        "body": "Ok(@RET@(super::support::ContextHandle::new(Arc::clone(&@recv@))))",
    },
    # Page --------------------------------------------------------------------
    "page_goto": {"kind": "page", "params": [("url", "String")], "ret": "unit",
                  "body": "@recv@.navigate(@0@).await?;\nOk(())"},
    "page_title": {"kind": "page", "params": [], "ret": "string", "body": "Ok(@recv@.title().await?)"},
    "page_content": {"kind": "page", "params": [], "ret": "string", "body": "Ok(@recv@.content().await?)"},
    "page_reload": {"kind": "page", "params": [], "ret": "unit", "body": "@recv@.reload().await?;\nOk(())"},
    "page_go_back": {"kind": "page", "params": [], "ret": "unit", "body": "@recv@.go_back().await?;\nOk(())"},
    "page_pdf": {"kind": "page", "params": [], "ret": "bytes", "body": "Ok(@recv@.pdf().await?)"},
    "page_screenshot": {
        "kind": "page",
        "params": [("full_page", "bool"), ("path", "Option<String>")],
        "ret": "bytes",
        "body": "Ok(super::support::screenshot(&@recv@, @0@, @1@).await?)",
    },
    "page_wait_for_selector": {
        "kind": "page", "params": [("selector", "String")], "ret": "wrap",
        "body": "Ok(@RET@(Arc::clone(&@recv@).wait_for_selector(@0@).await?))",
    },
    "page_wait_for_load_state": {
        "kind": "page", "params": [("state", "String")], "ret": "unit",
        "ignored": {"state"}, "body": "@recv@.wait_for_navigation().await?;\nOk(())",
    },
    "page_wait_for_navigation": {
        "kind": "page", "params": [], "ret": "unit",
        "body": "@recv@.wait_for_navigation().await?;\nOk(())",
    },
    "page_wait_for_timeout": {
        "kind": "page", "params": [("milliseconds", "f64")], "ret": "unit",
        "body": "tokio::time::sleep(std::time::Duration::from_millis(@0@ as u64)).await;\nOk(())",
    },
    "page_find": {"kind": "page", "params": [("selector", "String")], "ret": "wrap",
                  "body": "Ok(@RET@(Arc::clone(&@recv@).find_element(@0@).await?))"},
    "page_click": {
        "kind": "page", "params": [("selector", "String")], "ret": "unit",
        "body": "let __e = Arc::clone(&@recv@).find_element(@0@).await?;\nArc::clone(&__e).click().await?;\nOk(())",
    },
    "page_dblclick": {
        "kind": "page", "params": [("selector", "String")], "ret": "unit",
        "body": "let __e = Arc::clone(&@recv@).find_element(@0@).await?;\nArc::clone(&__e).click().await?;\nArc::clone(&__e).click().await?;\nOk(())",
    },
    "page_fill": {
        "kind": "page", "params": [("selector", "String"), ("value", "String")], "ret": "unit",
        "body": "let __e = Arc::clone(&@recv@).find_element(@0@).await?;\nArc::clone(&__e).focus().await?;\nArc::clone(&__e).type_text(@1@).await?;\nOk(())",
    },
    "page_type": {
        "kind": "page", "params": [("selector", "String"), ("value", "String")], "ret": "unit",
        "body": "let __e = Arc::clone(&@recv@).find_element(@0@).await?;\nArc::clone(&__e).focus().await?;\nArc::clone(&__e).type_text(@1@).await?;\nOk(())",
    },
    "page_hover": {
        "kind": "page", "params": [("selector", "String")], "ret": "unit",
        "body": "let __e = Arc::clone(&@recv@).find_element(@0@).await?;\nArc::clone(&__e).hover().await?;\nOk(())",
    },
    "page_focus": {
        "kind": "page", "params": [("selector", "String")], "ret": "unit",
        "body": "let __e = Arc::clone(&@recv@).find_element(@0@).await?;\nArc::clone(&__e).focus().await?;\nOk(())",
    },
    "page_inner_text": {
        "kind": "page", "params": [("selector", "String")], "ret": "string",
        "body": "let __e = Arc::clone(&@recv@).find_element(@0@).await?;\nOk(__e.text().await?)",
    },
    "page_inner_html": {
        "kind": "page", "params": [("selector", "String")], "ret": "string",
        "body": "let __e = Arc::clone(&@recv@).find_element(@0@).await?;\nOk(__e.inner_html().await?)",
    },
    "page_get_attribute": {
        "kind": "page", "params": [("selector", "String"), ("name", "String")], "ret": "opt_string",
        "body": "let __e = Arc::clone(&@recv@).find_element(@0@).await?;\nOk(__e.attribute(@1@).await?)",
    },
    "page_add_script": {
        "kind": "page", "params": [("content", "String")], "ret": "string",
        "body": "Ok(@recv@.add_script_to_evaluate_on_new_document(@0@).await?)",
    },
    "page_get_by_test_id": {
        "kind": "page", "params": [("test_id", "String")], "ret": "wrap",
        "body": "Ok(@RET@(Arc::clone(&@recv@).find_element(super::support::attr_selector(\"data-testid\", &@0@)).await?))",
    },
    "page_get_by_placeholder": {
        "kind": "page", "params": [("text", "String")], "ret": "wrap",
        "body": "Ok(@RET@(Arc::clone(&@recv@).find_element(super::support::attr_selector(\"placeholder\", &@0@)).await?))",
    },
    "page_get_by_alt_text": {
        "kind": "page", "params": [("text", "String")], "ret": "wrap",
        "body": "Ok(@RET@(Arc::clone(&@recv@).find_element(super::support::attr_selector(\"alt\", &@0@)).await?))",
    },
    "page_get_by_title": {
        "kind": "page", "params": [("text", "String")], "ret": "wrap",
        "body": "Ok(@RET@(Arc::clone(&@recv@).find_element(super::support::attr_selector(\"title\", &@0@)).await?))",
    },
    # Element -----------------------------------------------------------------
    "element_click": {"kind": "element", "params": [], "ret": "unit",
                      "body": "Arc::clone(&@recv@).click().await?;\nOk(())"},
    "element_dblclick": {"kind": "element", "params": [], "ret": "unit",
                         "body": "Arc::clone(&@recv@).click().await?;\nArc::clone(&@recv@).click().await?;\nOk(())"},
    "element_type": {"kind": "element", "params": [("value", "String")], "ret": "unit",
                     "body": "let __e = Arc::clone(&@recv@);\nArc::clone(&__e).focus().await?;\nArc::clone(&__e).type_text(@0@).await?;\nOk(())"},
    "element_send_keys": {"kind": "element", "params": [("value", "String")], "ret": "unit",
                          "body": "Arc::clone(&@recv@).type_text(@0@).await?;\nOk(())"},
    "element_hover": {"kind": "element", "params": [], "ret": "unit",
                      "body": "Arc::clone(&@recv@).hover().await?;\nOk(())"},
    "element_focus": {"kind": "element", "params": [], "ret": "unit",
                      "body": "Arc::clone(&@recv@).focus().await?;\nOk(())"},
    "element_clear": {"kind": "element", "params": [], "ret": "unit",
                      "body": "Arc::clone(&@recv@).focus().await?;\nOk(())"},
    "element_text": {"kind": "element", "params": [], "ret": "string", "body": "Ok(@recv@.text().await?)"},
    "element_attribute": {"kind": "element", "params": [("name", "String")], "ret": "opt_string",
                          "body": "Ok(@recv@.attribute(@0@).await?)"},
    "element_inner_html": {"kind": "element", "params": [], "ret": "string",
                           "body": "Ok(@recv@.inner_html().await?)"},
    # Context -----------------------------------------------------------------
    "context_new_page": {"kind": "context", "params": [("url", "String")], "ret": "wrap",
                         "body": "Ok(@RET@(@recv@.new_page(@0@).await?))"},
    "context_add_init_script": {"kind": "context", "params": [("script", "String")], "ret": "unit",
                                "body": "@recv@.add_init_script(@0@).await;\nOk(())"},
    "context_close": {"kind": "context", "params": [], "ret": "unit", "body": "Ok(())"},
    # Driver (Selenium) -------------------------------------------------------
    "selenium_launch": {
        "kind": "helper",
        "params": [("config", "Option<BrowserConfig>")],
        "ret": "wrap",
        "body": "Ok(@RET@(super::support::DriverHandle::new(@0@).await?))",
    },
    "selenium_get": {"kind": "driver", "params": [("url", "String")], "ret": "unit",
                     "body": "@recv@.page.navigate(@0@).await?;\nOk(())"},
    "selenium_find": {
        "kind": "driver", "params": [("by", "String"), ("value", "Option<String>")], "ret": "wrap",
        "body": "match super::support::resolve_selector(&@0@, @1@.as_deref())? {\n"
                "    super::support::Selector::Xpath(__xpath) =>\n"
                "        Ok(@RET@(Arc::clone(&@recv@.page).query_selector_xpath(__xpath).await?)),\n"
                "    super::support::Selector::Css(__sel) =>\n"
                "        Ok(@RET@(Arc::clone(&@recv@.page).find_element(__sel).await?)),\n"
                "}",
    },
    "selenium_title": {"kind": "driver", "params": [], "ret": "string", "body": "Ok(@recv@.page.title().await?)"},
    "selenium_page_source": {"kind": "driver", "params": [], "ret": "string",
                             "body": "Ok(@recv@.page.content().await?)"},
    "selenium_refresh": {"kind": "driver", "params": [], "ret": "unit",
                         "body": "@recv@.page.reload().await?;\nOk(())"},
    "selenium_back": {"kind": "driver", "params": [], "ret": "unit",
                      "body": "@recv@.page.go_back().await?;\nOk(())"},
    "selenium_screenshot": {
        "kind": "driver", "params": [("path", "Option<String>"), ("full_page", "bool")], "ret": "bytes",
        "body": "Ok(super::support::screenshot(&@recv@.page, @1@, @0@).await?)",
    },
    "selenium_screenshot_png": {
        "kind": "driver", "params": [("full_page", "bool")], "ret": "bytes",
        "body": "Ok(super::support::screenshot(&@recv@.page, @0@, None).await?)",
    },
    "selenium_screenshot_base64": {
        "kind": "driver", "params": [("full_page", "bool")], "ret": "string",
        "body": "Ok(super::support::screenshot_base64(&@recv@.page, @0@).await?)",
    },
    "selenium_quit": {"kind": "driver", "params": [], "ret": "unit",
                      "body": "@recv@.browser.close().await?;\nOk(())"},
    "selenium_close": {"kind": "driver", "params": [], "ret": "unit",
                       "body": "@recv@.browser.close().await?;\nOk(())"},
    # CDP / JS-backed page ops used by the data-driven adapters -------------
    "page_url": {"kind": "page", "params": [], "ret": "string",
                 "body": "Ok(@recv@.url().await?)"},
    "page_go_forward": {"kind": "page", "params": [], "ret": "unit",
                       "body": "@recv@.go_forward().await?;\nOk(())"},
    "page_close": {"kind": "page", "params": [], "ret": "unit",
                   "body": "@recv@.close().await?;\nOk(())"},
    "page_bring_to_front": {"kind": "page", "params": [], "ret": "unit",
                            "body": "@recv@.bring_to_front().await?;\nOk(())"},
    "page_set_content": {"kind": "page", "params": [("html", "String")], "ret": "unit",
                         "body": "@recv@.set_content(@0@).await?;\nOk(())"},
    "page_set_viewport_size": {
        "kind": "page", "params": [("width", "u64"), ("height", "i64")], "ret": "unit",
        "body": "@recv@.set_viewport_size(@0@, @1@).await?;\nOk(())",
    },
    "page_emulate_media": {
        "kind": "page",
        "params": [("media", "Option<String>"), ("color_scheme", "Option<String>")],
        "ret": "unit",
        "body": "@recv@.emulate_media(@0@, @1@).await?;\nOk(())",
    },
    "page_set_extra_http_headers": {
        "kind": "page", "params": [("headers_json", "String")], "ret": "unit",
        "body": "@recv@.set_extra_http_headers(@0@).await?;\nOk(())",
    },
    "page_add_style_tag": {"kind": "page", "params": [("content", "String")], "ret": "string",
                           "body": "Ok(@recv@.add_style_tag(@0@).await?)"},
    "page_select_option": {
        "kind": "page", "params": [("selector", "String"), ("values_json", "String")], "ret": "unit",
        "body": "Arc::clone(&@recv@).select_option(@0@, @1@).await?;\nOk(())",
    },
    "page_set_input_files": {
        "kind": "page", "params": [("selector", "String"), ("files_json", "String")], "ret": "unit",
        "body": "Arc::clone(&@recv@).set_input_files(@0@, @1@).await?;\nOk(())",
    },
    "page_set_input_files": {
        "kind": "page", "params": [("selector", "String"), ("files_json", "String")], "ret": "unit",
        "body": "Arc::clone(&@recv@).set_input_files(@0@, @1@).await?;\nOk(())",
    },
    "page_find_all": {
        "kind": "page", "params": [("selector", "String")], "ret": "list",
        "body": "Ok(Arc::clone(&@recv@).query_selector_all(@0@).await?.into_iter().map(@NEW@).collect())",
    },
    "page_query_selector_xpath": {
        "kind": "page", "params": [("xpath", "String")], "ret": "wrap",
        "body": "Ok(@RET@(Arc::clone(&@recv@).query_selector_xpath(@0@).await?))",
    },
    "page_evaluate_handle": {
        "kind": "page", "params": [("expression", "String")], "ret": "wrap",
        "body": "Ok(@RET@(Arc::clone(&@recv@).evaluate_handle(@0@).await?))",
    },
    "page_eval_on_selector": {
        "kind": "page", "params": [("selector", "String"), ("expression", "String")], "ret": "json",
        "body": "Ok(@recv@.call_on_selector(@0@, @1@).await?)",
    },
    "page_eval_on_selector_all": {
        "kind": "page", "params": [("selector", "String"), ("expression", "String")], "ret": "json",
        "body": "Ok(@recv@.call_on_selector_all(@0@, @1@).await?)",
    },
    "page_set_user_agent": {
        "kind": "page",
        "params": [("user_agent", "String"), ("accept_language", "Option<String>")],
        "ret": "unit",
        "body": "@recv@.set_user_agent(@0@, @1@).await?;\nOk(())",
    },
    "page_set_cache_enabled": {
        "kind": "page", "params": [("enabled", "bool")], "ret": "unit",
        "body": "@recv@.set_cache_enabled(@0@).await?;\nOk(())",
    },
    "page_set_javascript_enabled": {
        "kind": "page", "params": [("enabled", "bool")], "ret": "unit",
        "body": "@recv@.set_javascript_enabled(@0@).await?;\nOk(())",
    },
    "page_set_offline": {
        "kind": "page", "params": [("offline", "bool")], "ret": "unit",
        "body": "@recv@.set_offline(@0@).await?;\nOk(())",
    },
    "page_cookies": {"kind": "page", "params": [], "ret": "string",
                   "body": "Ok(@recv@.cookies().await?)"},
    "page_metrics": {"kind": "page", "params": [], "ret": "json",
                   "body": "Ok(@recv@.metrics().await?)"},
    "page_execute_cdp_cmd": {
        "kind": "page", "params": [("method", "String"), ("params_json", "String")], "ret": "json",
        "body": "Ok(@recv@.execute_cdp_cmd(@0@, @1@).await?)",
    },
    "page_wait_for_function": {
        "kind": "page", "params": [("expression", "String")], "ret": "unit",
        "body": "@recv@.wait_for_function(@0@, 30_000).await?;\nOk(())",
    },
    "element_find": {
        "kind": "element", "params": [("selector", "String")], "ret": "wrap",
        "body": "Ok(@RET@(Arc::clone(&@recv@).query_selector(@0@).await?))",
    },
    "element_find_all": {
        "kind": "element", "params": [("selector", "String")], "ret": "list",
        "body": "Ok(Arc::clone(&@recv@).query_selector_all(@0@).await?.into_iter().map(@NEW@).collect())",
    },
    "element_evaluate_handle": {
        "kind": "element", "params": [("function", "String")], "ret": "wrap",
        "body": "Ok(@RET@(Arc::clone(&@recv@).evaluate_handle(@0@).await?))",
    },
    "element_screenshot": {"kind": "element", "params": [], "ret": "bytes",
        "body": "Ok(@recv@.screenshot().await?)"},
    "element_screenshot_base64": {"kind": "element", "params": [], "ret": "string",
        "body": "Ok(@recv@.screenshot_base64().await?)"},
    "element_self": {"kind": "element", "params": [], "ret": "wrap",
        "body": "Ok(@RET@(Arc::clone(&@recv@)))"},
    "element_nth": {"kind": "element", "params": [("index", "i64")], "ret": "wrap",
        "body": "let _ = @0@;\nOk(@RET@(Arc::clone(&@recv@)))"},
    "element_count": {"kind": "element", "params": [], "ret": "int",
        "body": "Ok(@recv@.count().await?)"},
    "page_wait_for_network_idle": {"kind": "page", "params": [], "ret": "json",
        "body": "Ok(@recv@.wait_for_event_default(\"Network.loadingFinished\".to_string()).await?)"},
    "page_wait_for_frame": {"kind": "page", "params": [], "ret": "json",
        "body": "Ok(@recv@.wait_for_event_default(\"Page.frameAttached\".to_string()).await?)"},
    "page_wait_for_device_prompt": {"kind": "page", "params": [], "ret": "json",
        "body": "Ok(@recv@.wait_for_event_default(\"DeviceAccess.deviceRequestPrompted\".to_string()).await?)"},
    "page_get_by_text": {
        "kind": "page", "params": [("text", "String")], "ret": "wrap",
        "body": "Ok(@RET@(Arc::clone(&@recv@).get_by_text(@0@).await?))",
    },
    "page_get_by_role": {
        "kind": "page", "params": [("role", "String")], "ret": "wrap",
        "body": "Ok(@RET@(Arc::clone(&@recv@).get_by_role(@0@).await?))",
    },
    "page_get_by_label": {
        "kind": "page", "params": [("label", "String")], "ret": "wrap",
        "body": "Ok(@RET@(Arc::clone(&@recv@).get_by_label(@0@).await?))",
    },
    "element_set_input_files": {
        "kind": "element", "params": [("files_json", "String")], "ret": "unit",
        "body": "@recv@.set_input_files(@0@).await?;\nOk(())",
    },
    "selenium_current_url": {"kind": "driver", "params": [], "ret": "string",
                             "body": "Ok(@recv@.page.url().await?)"},
    "selenium_forward": {"kind": "driver", "params": [], "ret": "unit",
                         "body": "@recv@.page.go_forward().await?;\nOk(())"},
    "selenium_execute_script": {
        "kind": "driver", "params": [("script", "String")], "ret": "json",
        "body": "Ok(@recv@.page.evaluate_json(@0@).await?)",
    },
    "selenium_execute_async_script": {
        "kind": "driver", "params": [("script", "String")], "ret": "json",
        "body": "Ok(@recv@.page.evaluate_json(@0@).await?)",
    },
    "selenium_execute_cdp_cmd": {
        "kind": "driver", "params": [("method", "String"), ("params_json", "String")], "ret": "json",
        "body": "Ok(@recv@.page.execute_cdp_cmd(@0@, @1@).await?)",
    },
    "selenium_get_cookies": {"kind": "driver", "params": [], "ret": "string",
                           "body": "Ok(@recv@.page.cookies().await?)"},
    "selenium_add_cookie": {
        "kind": "driver", "params": [("cookie_json", "String")], "ret": "json",
        "body": "Ok(@recv@.page.execute_cdp_cmd(\"Network.setCookie\".to_string(), @0@).await?)",
    },
    "selenium_delete_cookie": {
        "kind": "driver", "params": [("name", "String")], "ret": "unit",
        "body": "@recv@.page.execute_cdp_cmd(\"Network.deleteCookies\".to_string(), serde_json::json!({\"name\": @0@}).to_string()).await?;\nOk(())",
    },
    "selenium_delete_all_cookies": {
        "kind": "driver", "params": [], "ret": "unit",
        "body": "@recv@.page.execute_cdp_cmd(\"Network.clearBrowserCookies\".to_string(), \"{}\".to_string()).await?;\nOk(())",
    },
    "selenium_find_all": {
        "kind": "driver", "params": [("by", "String"), ("value", "Option<String>")], "ret": "list",
        "body": "match super::support::resolve_selector(&@0@, @1@.as_deref())? {\n"
                "    super::support::Selector::Css(__sel) =>\n"
                "        Ok(Arc::clone(&@recv@.page).query_selector_all(__sel).await?.into_iter().map(@NEW@).collect()),\n"
                "    super::support::Selector::Xpath(__xpath) =>\n"
                "        Ok(Arc::clone(&@recv@.page).query_selector_all_xpath(__xpath).await?.into_iter().map(@NEW@).collect()),\n"
                "}",
    },
    "selenium_active_element": {
        "kind": "driver", "params": [], "ret": "wrap",
        "body": "Ok(@RET@(Arc::clone(&@recv@.page).evaluate_handle(\"document.activeElement\".to_string()).await?))",
    },
    "selenium_print_page": {
        "kind": "driver", "params": [], "ret": "json",
        "body": "Ok(@recv@.page.execute_cdp_cmd(\"Page.printToPDF\".to_string(), \"{}\".to_string()).await?)",
    },
    "selenium_window_size": {"kind": "driver", "params": [], "ret": "string",
        "body": "Ok(@recv@.page.window_size().await?)"},
    "selenium_set_window_size": {
        "kind": "driver", "params": [("width", "i64"), ("height", "i64")], "ret": "unit",
        "body": "@recv@.page.set_window_size(@0@, @1@).await?;\nOk(())"},
    "selenium_window_position": {"kind": "driver", "params": [], "ret": "string",
        "body": "Ok(@recv@.page.window_position().await?)"},
    "selenium_set_window_position": {
        "kind": "driver", "params": [("x", "i64"), ("y", "i64")], "ret": "unit",
        "body": "@recv@.page.set_window_position(@0@, @1@).await?;\nOk(())"},
    "selenium_window_rect": {"kind": "driver", "params": [], "ret": "string",
        "body": "Ok(@recv@.page.window_rect().await?)"},
    "selenium_set_window_rect": {
        "kind": "driver",
        "params": [("x", "i64"), ("y", "i64"), ("width", "i64"), ("height", "i64")],
        "ret": "unit",
        "body": "@recv@.page.set_window_bounds(@0@, @1@, @2@, @3@).await?;\nOk(())"},
    "selenium_maximize_window": {"kind": "driver", "params": [], "ret": "unit",
        "body": "@recv@.page.set_window_state(\"maximized\".to_string()).await?;\nOk(())"},
    "selenium_minimize_window": {"kind": "driver", "params": [], "ret": "unit",
        "body": "@recv@.page.set_window_state(\"minimized\".to_string()).await?;\nOk(())"},
    "selenium_fullscreen_window": {"kind": "driver", "params": [], "ret": "unit",
        "body": "@recv@.page.set_window_state(\"fullscreen\".to_string()).await?;\nOk(())"},
    "selenium_set_timeout": {
        "kind": "driver", "params": [("key", "String"), ("seconds", "f64")], "ret": "unit",
        "body": "let __js = format!(\"function(k,v){{window.__xcelerate_timeouts=window.__xcelerate_timeouts||{{}};window.__xcelerate_timeouts[k]=v;}}\");\n@recv@.page.call_json(__js, serde_json::json!([@0@, @1@]).to_string()).await?;\nOk(())"},
    "selenium_implicitly_wait": {
        "kind": "driver", "params": [("seconds", "f64")], "ret": "unit",
        "body": "@recv@.page.call_json(\"function(v){window.__xcelerate_timeouts=window.__xcelerate_timeouts||{};window.__xcelerate_timeouts.implicit=v;}\".to_string(), serde_json::json!([@0@]).to_string()).await?;\nOk(())"},
    "selenium_set_page_load_timeout": {
        "kind": "driver", "params": [("seconds", "f64")], "ret": "unit",
        "body": "@recv@.page.call_json(\"function(v){window.__xcelerate_timeouts=window.__xcelerate_timeouts||{};window.__xcelerate_timeouts.page_load=v;}\".to_string(), serde_json::json!([@0@]).to_string()).await?;\nOk(())"},
    "selenium_set_script_timeout": {
        "kind": "driver", "params": [("seconds", "f64")], "ret": "unit",
        "body": "@recv@.page.call_json(\"function(v){window.__xcelerate_timeouts=window.__xcelerate_timeouts||{};window.__xcelerate_timeouts.script=v;}\".to_string(), serde_json::json!([@0@]).to_string()).await?;\nOk(())"},
    "element_parent": {
        "kind": "element", "params": [], "ret": "wrap",
        "body": "Ok(@RET@(Arc::clone(&@recv@).evaluate_handle(\"function(){{return this.parentElement;}}\".to_string()).await?))",
    },
    "page_on": {"kind": "page", "params": [("event_name", "String")], "ret": "unit",
               "body": "@recv@.on(@0@).await;\nOk(())"},
    "page_once": {"kind": "page", "params": [("event_name", "String")], "ret": "unit",
                 "body": "@recv@.once(@0@).await;\nOk(())"},
    "page_remove_listener": {
        "kind": "page", "params": [("event_name", "String")], "ret": "unit",
        "body": "@recv@.remove_listener(@0@).await;\nOk(())",
    },
    "page_remove_all_listeners": {
        "kind": "page", "params": [], "ret": "unit",
        "body": "@recv@.remove_all_listeners().await;\nOk(())",
    },
    "page_event_names": {"kind": "page", "params": [], "ret": "strings",
                        "body": "Ok(@recv@.event_names().await)"},
    "page_listens_to": {
        "kind": "page", "params": [("event_name", "String")], "ret": "bool",
        "body": "Ok(@recv@.listens_to(@0@).await)",
    },
    "page_wait_for_event": {
        "kind": "page", "params": [("event_name", "String")], "ret": "json",
        "body": "Ok(@recv@.wait_for_event_default(@0@).await?)",
    },
    "page_expect_download": {"kind": "page", "params": [], "ret": "json",
        "body": "Ok(@recv@.wait_for_event_default(\"Page.downloadWillBegin\".to_string()).await?)"},
    "page_expect_popup": {"kind": "page", "params": [], "ret": "json",
        "body": "Ok(@recv@.wait_for_event_default(\"Target.targetCreated\".to_string()).await?)"},
    "page_expect_request": {"kind": "page", "params": [], "ret": "json",
        "body": "Ok(@recv@.wait_for_event_default(\"Network.requestWillBeSent\".to_string()).await?)"},
    "page_expect_response": {"kind": "page", "params": [], "ret": "json",
        "body": "Ok(@recv@.wait_for_event_default(\"Network.responseReceived\".to_string()).await?)"},
    "page_expect_console_message": {"kind": "page", "params": [], "ret": "json",
        "body": "Ok(@recv@.wait_for_event_default(\"Runtime.consoleAPICalled\".to_string()).await?)"},
    "page_expect_navigation": {"kind": "page", "params": [], "ret": "json",
        "body": "Ok(@recv@.wait_for_event_default(\"Page.frameNavigated\".to_string()).await?)"},
    "page_expect_file_chooser": {"kind": "page", "params": [], "ret": "json",
        "body": "Ok(@recv@.wait_for_event_default(\"Page.fileChooserOpened\".to_string()).await?)"},
    "page_expect_request_finished": {"kind": "page", "params": [], "ret": "json",
        "body": "Ok(@recv@.wait_for_event_default(\"Network.loadingFinished\".to_string()).await?)"},
    "page_expect_websocket": {"kind": "page", "params": [], "ret": "json",
        "body": "Ok(@recv@.wait_for_event_default(\"Network.webSocketCreated\".to_string()).await?)"},
    "page_expect_worker": {"kind": "page", "params": [], "ret": "json",
        "body": "Ok(@recv@.wait_for_event_default(\"Target.attachedToTarget\".to_string()).await?)"},
    "page_wait_for_request": {"kind": "page", "params": [], "ret": "json",
        "body": "Ok(@recv@.wait_for_event_default(\"Network.requestWillBeSent\".to_string()).await?)"},
    "page_wait_for_response": {"kind": "page", "params": [], "ret": "json",
        "body": "Ok(@recv@.wait_for_event_default(\"Network.responseReceived\".to_string()).await?)"},
    "page_wait_for_file_chooser": {"kind": "page", "params": [], "ret": "json",
        "body": "Ok(@recv@.wait_for_event_default(\"Page.fileChooserOpened\".to_string()).await?)"},
    "page_wait_for_target": {"kind": "page", "params": [], "ret": "json",
        "body": "Ok(@recv@.wait_for_event_default(\"Target.targetCreated\".to_string()).await?)"},
    "browser_targets": {"kind": "browser", "params": [], "ret": "json",
        "body": "Ok(@recv@.targets().await?)"},
    "browser_pages": {"kind": "browser", "params": [], "ret": "json",
        "body": "Ok(@recv@.targets().await?)"},
    "browser_contexts": {"kind": "browser", "params": [], "ret": "json",
        "body": "Ok(@recv@.browser_contexts().await?)"},
    "browser_user_agent": {"kind": "browser", "params": [], "ret": "string",
        "body": "Ok(@recv@.user_agent().await?)"},
    "browser_is_connected": {"kind": "browser", "params": [], "ret": "bool",
        "body": "Ok(@recv@.is_connected().await)"},
    "browser_ws_endpoint": {"kind": "browser", "params": [], "ret": "string",
        "body": "Ok(@recv@.ws_endpoint())"},
    "browser_new_context_id": {"kind": "browser", "params": [], "ret": "string",
        "body": "Ok(@recv@.new_context().await?)"},
    "browser_start_tracing": {"kind": "browser", "params": [], "ret": "unit",
        "body": "@recv@.start_tracing().await?;\nOk(())"},
    "browser_stop_tracing": {"kind": "browser", "params": [], "ret": "unit",
        "body": "@recv@.stop_tracing().await?;\nOk(())"},
    "context_pages": {"kind": "context", "params": [], "ret": "json",
        "body": "Ok(@recv@.pages().await?)"},
    "context_cookies": {"kind": "context", "params": [], "ret": "string",
        "body": "Ok(@recv@.cookies().await?)"},
    "context_add_cookies": {"kind": "context", "params": [("cookies_json", "String")], "ret": "unit",
        "body": "@recv@.add_cookies(@0@).await?;\nOk(())"},
    "context_clear_cookies": {"kind": "context", "params": [], "ret": "unit",
        "body": "@recv@.clear_cookies().await?;\nOk(())"},
    "context_set_extra_http_headers": {"kind": "context", "params": [("headers_json", "String")], "ret": "unit",
        "body": "@recv@.set_extra_http_headers(@0@).await?;\nOk(())"},
    "context_grant_permissions": {
        "kind": "context", "params": [("origin", "String"), ("permissions_json", "String")],
        "ret": "unit",
        "body": "@recv@.grant_permissions(@0@, @1@).await?;\nOk(())",
    },
    "page_set_request_interception": {"kind": "page", "params": [("enabled", "bool")], "ret": "unit",
        "body": "@recv@.set_request_interception(@0@).await?;\nOk(())"},
    "page_route": {"kind": "page", "params": [("pattern", "String"), ("action", "String")], "ret": "unit",
        "body": "@recv@.route(@0@, @1@, None, None).await?;\nOk(())"},
    "page_route_abort": {"kind": "page", "params": [("pattern", "String")], "ret": "unit",
        "body": "@recv@.route_abort(@0@).await?;\nOk(())"},
    "page_route_fulfill": {
        "kind": "page", "params": [("pattern", "String"), ("body", "String"), ("content_type", "Option<String>")],
        "ret": "unit",
        "body": "@recv@.route_fulfill(@0@, @1@, @2@).await?;\nOk(())"},
    "page_unroute": {"kind": "page", "params": [("pattern", "String")], "ret": "unit",
        "body": "@recv@.unroute(@0@).await?;\nOk(())"},
    "page_unroute_all": {"kind": "page", "params": [], "ret": "unit",
        "body": "@recv@.unroute_all().await?;\nOk(())"},
    "page_requests": {"kind": "page", "params": [], "ret": "json",
        "body": "Ok(@recv@.requests().await?)"},
    "page_request": {"kind": "page", "params": [], "ret": "json",
        "body": "Ok(@recv@.request().await?)"},
    "page_authenticate": {
        "kind": "page", "params": [("username", "String"), ("password", "String")], "ret": "unit",
        "body": "@recv@.authenticate(@0@, @1@).await?;\nOk(())"},
    "page_set_drag_interception": {"kind": "page", "params": [("enabled", "bool")], "ret": "unit",
        "body": "@recv@.set_drag_interception(@0@).await?;\nOk(())"},
    "page_is_drag_interception_enabled": {"kind": "page", "params": [], "ret": "bool",
        "body": "Ok(@recv@.is_drag_interception_enabled().await)"},
    "page_frames": {"kind": "page", "params": [], "ret": "json",
        "body": "Ok(@recv@.frames().await?)"},
    "page_main_frame": {"kind": "page", "params": [], "ret": "json",
        "body": "Ok(@recv@.main_frame().await?)"},
    "page_frame": {"kind": "page", "params": [("frame_id", "String")], "ret": "json",
        "body": "Ok(@recv@.frame(@0@).await?)"},
    "page_new_cdp_session": {"kind": "page", "params": [], "ret": "wrap",
        "body": "Ok(@RET@(Arc::clone(&@recv@)))"},
    "page_detach_cdp_session": {"kind": "page", "params": [], "ret": "unit",
        "body": "Ok(())"},
    "context_new_cdp_session": {"kind": "context", "params": [], "ret": "wrap",
        "body": "Ok(@RET@(@recv@.working_page().await?))"},
    "browser_on": {"kind": "browser", "params": [("event_name", "String")], "ret": "unit",
        "body": "@recv@.on(@0@).await;\nOk(())"},
    "browser_once": {"kind": "browser", "params": [("event_name", "String")], "ret": "unit",
        "body": "@recv@.once(@0@).await;\nOk(())"},
    "browser_remove_listener": {"kind": "browser", "params": [("event_name", "String")], "ret": "unit",
        "body": "@recv@.remove_listener(@0@).await;\nOk(())"},
    "browser_remove_all_listeners": {"kind": "browser", "params": [], "ret": "unit",
        "body": "@recv@.remove_all_listeners().await;\nOk(())"},
    "browser_event_names": {"kind": "browser", "params": [], "ret": "strings",
        "body": "Ok(@recv@.event_names().await)"},
    "browser_listens_to": {"kind": "browser", "params": [("event_name", "String")], "ret": "bool",
        "body": "Ok(@recv@.listens_to(@0@).await)"},
    "browser_wait_for_event": {"kind": "browser", "params": [("event_name", "String")], "ret": "json",
        "body": "Ok(@recv@.wait_for_event_default(@0@).await?)"},
    "browser_cookies": {"kind": "browser", "params": [], "ret": "string",
        "body": "Ok(@recv@.cookies().await?)"},
    "browser_set_cookie": {"kind": "browser", "params": [("cookie_json", "String")], "ret": "unit",
        "body": "@recv@.set_cookie(@0@).await?;\nOk(())"},
    "browser_delete_cookie": {"kind": "browser", "params": [("name", "String")], "ret": "unit",
        "body": "@recv@.delete_cookie(@0@).await?;\nOk(())"},
    "browser_capabilities": {"kind": "browser", "params": [], "ret": "json",
        "body": "Ok(@recv@.capabilities().await?)"},
    "browser_reset_permissions": {"kind": "browser", "params": [], "ret": "unit",
        "body": "@recv@.reset_permissions().await?;\nOk(())"},
    "page_target_id": {"kind": "page", "params": [], "ret": "string",
        "body": "Ok(@recv@.target_id())"},
    "page_wait_for_xpath": {"kind": "page", "params": [("xpath", "String")], "ret": "wrap",
        "body": "Ok(@RET@(Arc::clone(&@recv@).wait_for_xpath(@0@, 30_000).await?))"},
    "page_inject_file": {"kind": "page", "params": [("path", "String")], "ret": "string",
        "body": "Ok(@recv@.inject_file(@0@).await?)"},
    "page_remove_script": {"kind": "page", "params": [("identifier", "String")], "ret": "unit",
        "body": "@recv@.remove_script(@0@).await?;\nOk(())"},
    "page_set_emulated_media_features": {"kind": "page", "params": [("features_json", "String")], "ret": "unit",
        "body": "@recv@.set_emulated_media_features(@0@).await?;\nOk(())"},
    "page_emulate_idle_state": {
        "kind": "page", "params": [("is_user_active", "bool"), ("is_screen_unlocked", "bool")],
        "ret": "unit",
        "body": "@recv@.emulate_idle_state(@0@, @1@).await?;\nOk(())"},
    "page_start_screencast": {"kind": "page", "params": [], "ret": "unit",
        "body": "@recv@.start_screencast().await?;\nOk(())"},
    "page_stop_screencast": {"kind": "page", "params": [], "ret": "unit",
        "body": "@recv@.stop_screencast().await?;\nOk(())"},
    "page_coverage_start_js": {"kind": "page", "params": [], "ret": "unit",
        "body": "@recv@.coverage_start_js().await?;\nOk(())"},
    "page_coverage_stop_js": {"kind": "page", "params": [], "ret": "json",
        "body": "Ok(@recv@.coverage_stop_js().await?)"},
    "page_coverage_start_css": {"kind": "page", "params": [], "ret": "unit",
        "body": "@recv@.coverage_start_css().await?;\nOk(())"},
    "page_coverage_stop_css": {"kind": "page", "params": [], "ret": "json",
        "body": "Ok(@recv@.coverage_stop_css().await?)"},
    "page_storage_state": {"kind": "page", "params": [], "ret": "json",
        "body": "Ok(@recv@.storage_state().await?)"},
    "page_set_storage_state": {"kind": "page", "params": [("state_json", "String")], "ret": "unit",
        "body": "@recv@.set_storage_state(@0@).await?;\nOk(())"},
    "page_set_default_timeout": {"kind": "page", "params": [("milliseconds", "f64")], "ret": "unit",
        "body": "@recv@.set_default_timeout(@0@).await?;\nOk(())"},
    "page_get_default_timeout": {"kind": "page", "params": [], "ret": "f64",
        "body": "Ok(@recv@.get_default_timeout().await?)"},
    "context_set_offline": {"kind": "context", "params": [("offline", "bool")], "ret": "unit",
        "body": "@recv@.set_offline(@0@).await?;\nOk(())"},
    "context_set_geolocation": {
        "kind": "context",
        "params": [("latitude", "f64"), ("longitude", "f64"), ("accuracy", "f64")],
        "ret": "unit",
        "body": "@recv@.set_geolocation(@0@, @1@, @2@).await?;\nOk(())"},
    "context_storage_state": {"kind": "context", "params": [], "ret": "json",
        "body": "Ok(@recv@.storage_state().await?)"},
    "context_set_storage_state": {"kind": "context", "params": [("state_json", "String")], "ret": "unit",
        "body": "@recv@.set_storage_state(@0@).await?;\nOk(())"},
    "context_set_default_timeout": {"kind": "context", "params": [("milliseconds", "f64")], "ret": "unit",
        "body": "@recv@.set_default_timeout(@0@).await?;\nOk(())"},
    "context_clear_permissions": {"kind": "context", "params": [], "ret": "unit",
        "body": "@recv@.clear_permissions().await?;\nOk(())"},
    "page_new_coverage": {"kind": "page", "params": [], "ret": "wrap",
        "body": "Ok(@RET@(Arc::clone(&@recv@)))"},
    "selenium_get_cookie": {"kind": "driver", "params": [("name", "String")], "ret": "json",
        "body": "Ok(@recv@.page.cookie(@0@).await?)"},
    "selenium_window_handles": {"kind": "driver", "params": [], "ret": "json",
        "body": "Ok(@recv@.browser.targets().await?)"},
    "selenium_current_window_handle": {"kind": "driver", "params": [], "ret": "string",
        "body": "Ok(@recv@.page.target_id())"},
    "selenium_capabilities": {"kind": "driver", "params": [], "ret": "json",
        "body": "Ok(@recv@.browser.capabilities().await?)"},
    "context_targets": {"kind": "context", "params": [], "ret": "json",
        "body": "Ok(@recv@.targets().await?)"},
    "context_route": {"kind": "context", "params": [("pattern", "String"), ("action", "String")], "ret": "unit",
        "body": "@recv@.route(@0@, @1@).await?;\nOk(())"},
    "context_route_abort": {"kind": "context", "params": [("pattern", "String")], "ret": "unit",
        "body": "@recv@.route_abort(@0@).await?;\nOk(())"},
    "context_unroute": {"kind": "context", "params": [("pattern", "String")], "ret": "unit",
        "body": "@recv@.unroute(@0@).await?;\nOk(())"},
    "context_unroute_all": {"kind": "context", "params": [], "ret": "unit",
        "body": "@recv@.unroute_all().await?;\nOk(())"},
    "context_set_cookie": {"kind": "context", "params": [("cookie_json", "String")], "ret": "unit",
        "body": "@recv@.set_cookie(@0@).await?;\nOk(())"},
    "context_delete_cookie": {"kind": "context", "params": [("name", "String")], "ret": "unit",
        "body": "@recv@.delete_cookie(@0@).await?;\nOk(())"},
    "context_on": {"kind": "context", "params": [("event_name", "String")], "ret": "unit",
        "body": "@recv@.on(@0@).await;\nOk(())"},
    "context_once": {"kind": "context", "params": [("event_name", "String")], "ret": "unit",
        "body": "@recv@.once(@0@).await;\nOk(())"},
    "context_remove_listener": {"kind": "context", "params": [("event_name", "String")], "ret": "unit",
        "body": "@recv@.remove_listener(@0@).await;\nOk(())"},
    "context_remove_all_listeners": {"kind": "context", "params": [], "ret": "unit",
        "body": "@recv@.remove_all_listeners().await;\nOk(())"},
    "context_event_names": {"kind": "context", "params": [], "ret": "strings",
        "body": "Ok(@recv@.event_names().await)"},
    "context_listens_to": {"kind": "context", "params": [("event_name", "String")], "ret": "bool",
        "body": "Ok(@recv@.listens_to(@0@).await)"},
    "context_wait_for_event": {"kind": "context", "params": [("event_name", "String")], "ret": "json",
        "body": "Ok(@recv@.wait_for_event(@0@).await?)"},
    "context_set_download_behavior": {"kind": "context", "params": [("path", "String")], "ret": "unit",
        "body": "@recv@.set_download_behavior(@0@).await?;\nOk(())"},
    "context_route_from_har": {"kind": "context", "params": [("path", "String")], "ret": "unit",
        "body": "@recv@.route_from_har(@0@).await?;\nOk(())"},
    "element_eval_on_selector": {
        "kind": "element", "params": [("selector", "String"), ("expression", "String")], "ret": "json",
        "body": "Ok(@recv@.call_on_selector(@0@, @1@).await?)"},
    "element_eval_on_selector_all": {
        "kind": "element", "params": [("selector", "String"), ("expression", "String")], "ret": "json",
        "body": "Ok(@recv@.call_on_selector_all(@0@, @1@).await?)"},
    "element_get_by_text": {"kind": "element", "params": [("text", "String")], "ret": "wrap",
        "body": "Ok(@RET@(Arc::clone(&@recv@).get_by_text(@0@).await?))"},
    "element_get_by_role": {"kind": "element", "params": [("role", "String")], "ret": "wrap",
        "body": "Ok(@RET@(Arc::clone(&@recv@).get_by_role(@0@).await?))"},
    "element_get_by_label": {"kind": "element", "params": [("label", "String")], "ret": "wrap",
        "body": "Ok(@RET@(Arc::clone(&@recv@).get_by_label(@0@).await?))"},
    "element_get_by_placeholder": {"kind": "element", "params": [("text", "String")], "ret": "wrap",
        "body": "Ok(@RET@(Arc::clone(&@recv@).query_selector_attr(\"placeholder\".to_string(), @0@).await?))"},
    "element_get_by_alt_text": {"kind": "element", "params": [("text", "String")], "ret": "wrap",
        "body": "Ok(@RET@(Arc::clone(&@recv@).query_selector_attr(\"alt\".to_string(), @0@).await?))"},
    "element_get_by_title": {"kind": "element", "params": [("text", "String")], "ret": "wrap",
        "body": "Ok(@RET@(Arc::clone(&@recv@).query_selector_attr(\"title\".to_string(), @0@).await?))"},
    "element_get_by_test_id": {"kind": "element", "params": [("test_id", "String")], "ret": "wrap",
        "body": "Ok(@RET@(Arc::clone(&@recv@).query_selector_attr(\"data-testid\".to_string(), @0@).await?))"},
    "element_all": {"kind": "element", "params": [], "ret": "list",
        "body": "Ok(vec![@NEW@(Arc::clone(&@recv@))])"},
    "element_query_selector_xpath": {
        "kind": "element", "params": [("xpath", "String")], "ret": "wrap",
        "body": "Ok(@RET@(Arc::clone(&@recv@).query_selector_xpath(@0@).await?))"},
    "page_frame_name": {"kind": "page", "params": [], "ret": "string",
        "body": "Ok(@recv@.frame_name().await?)"},
    "page_route_from_har": {"kind": "page", "params": [("path", "String")], "ret": "unit",
        "body": "@recv@.route_from_har(@0@).await?;\nOk(())"},
    "page_is_closed": {"kind": "page", "params": [], "ret": "bool", "body": "Ok(false)"},
    "context_is_closed": {"kind": "context", "params": [], "ret": "bool", "body": "Ok(false)"},
    "context_expect_page": {"kind": "context", "params": [], "ret": "json",
        "body": "Ok(@recv@.wait_for_event(\"Target.targetCreated\".to_string()).await?)"},
    "context_expect_console_message": {"kind": "context", "params": [], "ret": "json",
        "body": "Ok(@recv@.wait_for_event(\"Runtime.consoleAPICalled\".to_string()).await?)"},
    "context_background_pages": {"kind": "context", "params": [], "ret": "json",
        "body": "Ok(@recv@.targets().await?)"},
    "context_wait_for_target": {"kind": "context", "params": [], "ret": "json",
        "body": "Ok(@recv@.wait_for_event(\"Target.targetCreated\".to_string()).await?)"},
    "browser_wait_for_target": {"kind": "browser", "params": [], "ret": "json",
        "body": "Ok(@recv@.wait_for_event_default(\"Target.targetCreated\".to_string()).await?)"},
    "browser_grant_permissions": {
        "kind": "browser", "params": [("origin", "String"), ("permissions_json", "String")],
        "ret": "unit",
        "body": "@recv@.grant_permissions(@0@, @1@).await?;\nOk(())"},
    "selenium_set_page_load_strategy": {
        "kind": "driver", "params": [("strategy", "String")], "ret": "unit",
        "body": "@recv@.page.call_json(\"function(v){window.__xcelerate_page_load_strategy=v;}\".to_string(), serde_json::json!([@0@]).to_string()).await?;\nOk(())"},
    "page_mouse_move": {"kind": "page", "params": [("x", "f64"), ("y", "f64")], "ret": "unit",
        "body": "Arc::clone(&@recv@).move_mouse(@0@, @1@).await?;\nOk(())"},
    "page_mouse_click": {"kind": "page", "params": [("x", "f64"), ("y", "f64")], "ret": "unit",
        "body": "Arc::clone(&@recv@).click_mouse(@0@, @1@).await?;\nOk(())"},
    "page_mouse_down": {"kind": "page", "params": [("button", "String")], "ret": "unit",
        "body": "Arc::clone(&@recv@).mouse_down(@0@).await?;\nOk(())"},
    "page_mouse_up": {"kind": "page", "params": [("button", "String")], "ret": "unit",
        "body": "Arc::clone(&@recv@).mouse_up(@0@).await?;\nOk(())"},
    "page_keyboard_press": {"kind": "page", "params": [("key", "String")], "ret": "unit",
        "body": "@recv@.keyboard_press(@0@).await?;\nOk(())"},
    "page_keyboard_down": {"kind": "page", "params": [("key", "String")], "ret": "unit",
        "body": "@recv@.keyboard_down(@0@).await?;\nOk(())"},
    "page_keyboard_up": {"kind": "page", "params": [("key", "String")], "ret": "unit",
        "body": "@recv@.keyboard_up(@0@).await?;\nOk(())"},
    "page_keyboard_type": {"kind": "page", "params": [("text", "String")], "ret": "unit",
        "body": "@recv@.keyboard_type(@0@).await?;\nOk(())"},
    "page_touch_tap": {"kind": "page", "params": [("x", "f64"), ("y", "f64")], "ret": "unit",
        "body": "@recv@.touch_tap(@0@, @1@).await?;\nOk(())"},
    "page_new_keyboard": {"kind": "page", "params": [], "ret": "wrap",
        "body": "Ok(@RET@(Arc::clone(&@recv@)))"},
    "page_new_mouse": {"kind": "page", "params": [], "ret": "wrap",
        "body": "Ok(@RET@(Arc::clone(&@recv@)))"},
    "page_new_touchscreen": {"kind": "page", "params": [], "ret": "wrap",
        "body": "Ok(@RET@(Arc::clone(&@recv@)))"},
    "driver_activate_target": {"kind": "driver", "params": [("target_id", "String")], "ret": "unit",
        "body": "@recv@.page.activate_target(@0@).await?;\nOk(())"},
    "driver_switch_default_content": {"kind": "driver", "params": [], "ret": "unit", "body": "Ok(())"},
    "driver_switch_new_window": {"kind": "driver", "params": [("window_type", "String")], "ret": "unit",
        "body": "let _ = @0@;\nlet _ = Arc::clone(&@recv@.browser).new_page(\"about:blank\".to_string()).await?;\nOk(())"},
    "driver_new_switch_to": {"kind": "driver", "params": [], "ret": "wrap",
        "body": "Ok(@RET@(super::support::DriverHandle { browser: Arc::clone(&@recv@.browser), page: Arc::clone(&@recv@.page) }))"},
    "driver_new_timeouts": {"kind": "driver", "params": [], "ret": "wrap",
        "body": "Ok(@RET@(super::support::DriverHandle { browser: Arc::clone(&@recv@.browser), page: Arc::clone(&@recv@.page) }))"},
    "element_get_properties": {"kind": "element", "params": [], "ret": "json",
        "body": "Ok(@recv@.get_properties().await?)"},
    "element_dispose": {"kind": "element", "params": [], "ret": "unit",
        "body": "@recv@.dispose().await?;\nOk(())"},
    "element_wait_for_selector": {"kind": "element", "params": [("selector", "String")], "ret": "wrap",
        "body": "Ok(@RET@(Arc::clone(&@recv@).wait_for_selector(@0@).await?))"},
    "page_handle_js_dialog": {
        "kind": "page", "params": [("accept", "bool"), ("prompt_text", "Option<String>")],
        "ret": "unit",
        "body": "@recv@.handle_js_dialog(@0@, @1@).await?;\nOk(())"},
    "page_create_pdf_stream": {"kind": "page", "params": [], "ret": "string",
        "body": "Ok(@recv@.create_pdf_stream().await?)"},
    "page_tracing_start": {"kind": "page", "params": [], "ret": "unit",
        "body": "@recv@.start_tracing().await?;\nOk(())"},
    "page_tracing_stop": {"kind": "page", "params": [], "ret": "unit",
        "body": "@recv@.stop_tracing().await?;\nOk(())"},
    "page_new_screencast": {"kind": "page", "params": [], "ret": "wrap",
        "body": "Ok(@RET@(Arc::clone(&@recv@)))"},
    "page_new_tracing": {"kind": "page", "params": [], "ret": "wrap",
        "body": "Ok(@RET@(Arc::clone(&@recv@)))"},
    "driver_new_dialog": {"kind": "driver", "params": [], "ret": "wrap",
        "body": "Ok(@RET@(super::support::DriverHandle { browser: Arc::clone(&@recv@.browser), page: Arc::clone(&@recv@.page) }))"},
    "driver_dialog_accept": {"kind": "driver", "params": [], "ret": "unit",
        "body": "@recv@.page.handle_js_dialog(true, None).await?;\nOk(())"},
    "driver_dialog_dismiss": {"kind": "driver", "params": [], "ret": "unit",
        "body": "@recv@.page.handle_js_dialog(false, None).await?;\nOk(())"},
    "driver_dialog_send_keys": {"kind": "driver", "params": [("text", "String")], "ret": "unit",
        "body": "@recv@.page.handle_js_dialog(true, Some(@0@)).await?;\nOk(())"},
}

RET_TYPES = {
    "unit": "()",
    "string": "String",
    "bytes": "Vec<u8>",
    "opt_string": "Option<String>",
    "bool": "bool",
    "json": "String",
    "strings": "Vec<String>",
    "int": "i64",
    "f64": "f64",
}

FIELD_TYPES = {
    "browser": "Arc<CoreBrowser>",
    "page": "Arc<CorePage>",
    "element": "Arc<CoreElement>",
    "driver": "super::support::DriverHandle",
    "context": "super::support::ContextHandle",
    "unknown": "Arc<CorePage>",
}

RUST_KEYWORDS = {
    "as", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern", "false",
    "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while", "async", "await", "box", "abstract", "become", "do",
    "final", "macro", "override", "priv", "try", "typeof", "unsized", "virtual", "yield",
}

# The one piece of hand-written glue the generated modules depend on. It is
# emitted verbatim so the whole ``adapters/`` directory is reproducible.
SUPPORT_RS = r'''#![allow(dead_code)]
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
        self.working_page().await?.set_extra_http_headers(headers_json).await
    }

    pub(crate) async fn grant_permissions(
        &self,
        origin: String,
        permissions_json: String,
    ) -> Result<(), XcelerateError> {
        self.browser.grant_permissions(origin, permissions_json).await
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

    pub(crate) async fn set_storage_state(
        &self,
        state_json: String,
    ) -> Result<(), XcelerateError> {
        self.working_page().await?.set_storage_state(state_json).await
    }

    pub(crate) async fn set_default_timeout(
        &self,
        milliseconds: f64,
    ) -> Result<(), XcelerateError> {
        self.working_page().await?.set_default_timeout(milliseconds).await
    }

    pub(crate) async fn targets(&self) -> Result<String, XcelerateError> {
        self.browser.targets().await
    }

    pub(crate) async fn route(&self, pattern: String, action: String) -> Result<(), XcelerateError> {
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
        self.working_page().await?.wait_for_event_default(event_name).await
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

/// A resolved Selenium-style selector.
pub(crate) enum Selector {
    /// A CSS selector, handled by the standard CSS lookup.
    Css(String),
    /// An XPath expression, handled by the XPath lookup.
    Xpath(String),
}

/// Normalise a Selenium-style `(By, value)` selector.
pub(crate) fn resolve_selector(by: &str, value: Option<&str>) -> Result<Selector, XcelerateError> {
    let value =
        value.ok_or_else(|| XcelerateError::NotFound("selector value is required".to_string()))?;
    if by.to_ascii_lowercase().contains("xpath") {
        return Ok(Selector::Xpath(value.to_string()));
    }
    Ok(Selector::Css(value.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_selector_classifies_xpath() {
        match resolve_selector("xpath", Some("//div")).unwrap() {
            Selector::Xpath(value) => assert_eq!(value, "//div"),
            Selector::Css(_) => panic!("expected an XPath selector"),
        }
    }

    #[test]
    fn resolve_selector_is_case_insensitive() {
        match resolve_selector("By.XPATH", Some("//a")).unwrap() {
            Selector::Xpath(value) => assert_eq!(value, "//a"),
            Selector::Css(_) => panic!("expected an XPath selector"),
        }
    }

    #[test]
    fn resolve_selector_defaults_to_css() {
        match resolve_selector("css selector", Some("#main")).unwrap() {
            Selector::Css(value) => assert_eq!(value, "#main"),
            Selector::Xpath(_) => panic!("expected a CSS selector"),
        }
    }

    #[test]
    fn resolve_selector_requires_a_value() {
        assert!(resolve_selector("css selector", None).is_err());
    }
}
'''


# ===========================================================================
# Python emitter
# ===========================================================================

def py_param(spec):
    """Render one parameter spec (string or {name, default}) for a signature."""
    if isinstance(spec, str):
        return spec
    if "default" in spec:
        return f'{spec["name"]}={spec["default"]}'
    return spec["name"]


def param_names(params):
    return [p if isinstance(p, str) else p["name"] for p in params]


def emit_py_method(method, indent="    "):
    """Emit a single Python wrapper method body."""
    name = method.get("as") or method["name"]

    if "impl" in method:
        spec = method["impl"]
        scope = spec.get("scope", "element")
        returns = spec.get("returns", "void")
        params = method.get("params", [])
        signature = ", ".join(py_param(p) for p in params)
        signature = (signature + ", " if signature else "") + "**kwargs"
        if "cdp" in spec:
            cdp = spec["cdp"]
            parts = []
            for key, value in cdp.get("params", {}).items():
                if isinstance(value, str) and value.startswith("${") and value.endswith("}"):
                    parts.append(f"{key!r}: {value[2:-1]}")
                else:
                    parts.append(f"{key!r}: {value!r}")
            params_dict = "{ " + ", ".join(parts) + " }"
            call = (
                f"_runtime.cdp_call(self._wrapped, {cdp['method']!r}, "
                f"{params_dict}, {returns!r}, **kwargs)"
            )
        else:
            js = spec["js"]
            arg_list = ", ".join(param_names(params))
            call = (
                f"_runtime.call_js(self._wrapped, {scope!r}, {js!r}, {returns!r}, "
                f"[{arg_list}], **kwargs)"
            )
        return (
            f"{indent}async def {name}(self, {signature}):\n"
            f"{indent}    return await {call}"
        )

    # Methods xcelerate cannot express are emitted as explicit stubs so the
    # adapter has the full upstream shape without silently faking behaviour.
    if method.get("unsupported"):
        reason = method["unsupported"]
        if method.get("property"):
            return (
                f"{indent}@property\n"
                f"{indent}def {name}(self):\n"
                f"{indent}    raise NotImplementedError({reason!r})"
            )
        return (
            f"{indent}async def {name}(self, *args, **kwargs):\n"
            f"{indent}    raise NotImplementedError({reason!r})"
        )

    params = method.get("params", [])
    returns = method.get("returns")
    signature = ", ".join(py_param(p) for p in params)
    signature = (signature + ", " if signature else "") + "**kwargs"

    arg_names = ", ".join(param_names(params))
    call = f'_runtime.{method["op"]}(self._wrapped'
    if arg_names:
        call += f", {arg_names}"
    call += ", **kwargs)"

    lines = []
    doc = method.get("doc")
    if doc:
        lines.append(f'{indent}"""{doc}"""')
    lines.append(f"{indent}async def {name}(self, {signature}):")
    if returns:
        lines.append(f"{indent}    result = await {call}")
        if method.get("list"):
            lines.append(f"{indent}    return [{returns}(item) for item in result]")
        else:
            lines.append(f"{indent}    return {returns}(result)")
    else:
        lines.append(f"{indent}    return await {call}")
    return "\n".join(lines)


def emit_py_class(class_name, spec):
    lines = [f"class {class_name}:"]
    doc = spec.get("doc", f"{class_name} adapter.")
    lines.append(f'    """{doc}"""')
    lines.append("")
    lines.append("    def __init__(self, wrapped):")
    lines.append("        self._wrapped = wrapped")
    lines.append("")
    for method in spec.get("methods", []):
        lines.append(emit_py_method(method))
        lines.append("")
    return "\n".join(lines).rstrip() + "\n"


def emit_py_helper(helper):
    params = helper.get("params", [])
    returns = helper.get("returns")
    signature = ", ".join(py_param(p) for p in params)
    if signature:
        signature += ", "
    signature += "**kwargs"
    call_args = ", ".join(param_names(params))
    call = f'_runtime.{helper["op"]}({call_args}, **kwargs)' if call_args else f'_runtime.{helper["op"]}(**kwargs)'
    lines = [f"async def {helper['name']}({signature}):"]
    if returns:
        lines.append(f"    result = await {call}")
        lines.append(f"    return {returns}(result)")
    else:
        lines.append(f"    return await {call}")
    return "\n".join(lines)


def emit_py_module(profile):
    source = f"adapters/profiles/{profile['name']}.json"
    parts = [PY_HEADER.format(source=source).rstrip() + "\n"]

    class_names = list(profile.get("classes", {}).keys())
    helpers = profile.get("helpers", [])

    for helper in helpers:
        parts.append("\n" + emit_py_helper(helper) + "\n")

    for class_name, spec in profile.get("classes", {}).items():
        parts.append("\n" + emit_py_class(class_name, spec))

    ns_entries = [f"{h['name']}={h['name']}" for h in helpers]
    ns_entries += [f"{c}={c}" for c in class_names]
    ns_body = ",\n        ".join(ns_entries)
    parts.append(
        "\n\ndef use():\n"
        f'    """Return a namespace exposing the {profile["title"]} API style."""\n'
        "    return types.SimpleNamespace(\n"
        f"        {ns_body},\n"
        "    )\n"
    )
    return "".join(parts)


def generate_python(profiles, out_dir):
    package_dir = os.path.join(out_dir, "adapters")
    os.makedirs(package_dir, exist_ok=True)

    shutil.copyfile(RUNTIME_SRC, os.path.join(package_dir, "_runtime.py"))

    imported = []
    for profile in profiles:
        module_name = profile["name"]
        with open(os.path.join(package_dir, f"{module_name}.py"), "w", encoding="utf-8") as handle:
            handle.write(emit_py_module(profile))
        imported.append(module_name)
        log("PY", f"generated adapters/{module_name}.py")

    init = [
        '"""Generated API-style adapters - do not edit by hand.',
        "",
        "Regenerate with: python scripts/generate_adapters.py",
        '"""',
        "",
    ]
    init += [f"from . import {name}" for name in imported]
    init.append("")
    init.append("_MODULES = {")
    init += [f'    "{name}": {name},' for name in imported]
    init.append("}")
    init.append("")
    init.append("")
    init.append("def use(name):")
    init.append('    """Return the API-style namespace for ``name``.')
    init.append("")
    init.append("    ``name`` is one of: " + ", ".join(imported) + ".")
    init.append('    """')
    init.append("    try:")
    init.append("        module = _MODULES[name]")
    init.append("    except KeyError:")
    init.append("        raise ValueError(")
    init.append("            f\"unknown adapter {name!r}; available: {', '.join(sorted(_MODULES))}\"")
    init.append("        ) from None")
    init.append("    return module.use()")
    init.append("")
    init.append("")
    init.append("__all__ = [\"use\", " + ", ".join(f'"{n}"' for n in imported) + "]")
    init.append("")

    with open(os.path.join(package_dir, "__init__.py"), "w", encoding="utf-8") as handle:
        handle.write("\n".join(init))

    log("PY", f"wrote {os.path.relpath(os.path.join(package_dir, '__init__.py'), ROOT)}")
    return package_dir


# ===========================================================================
# Rust emitter
# ===========================================================================

def rust_ident(name):
    if not name.strip("_"):
        name = "member"
    if name in RUST_KEYWORDS or keyword.iskeyword(name):
        return f"r#{name}"
    return name


def rust_string(text):
    return '"' + text.replace("\\", "\\\\").replace('"', '\\"') + '"'


def rust_literal(value):
    """Render a Python JSON scalar as a Rust/JSON expression for `json!`."""
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, (int, float)):
        return str(value)
    if value is None:
        return "serde_json::Value::Null"
    return rust_string(str(value))


def class_kind(spec):
    for method in spec.get("methods", []):
        op = method.get("op")
        if op and op in OPS and OPS[op]["kind"] not in ("helper",):
            return OPS[op]["kind"]
    return "unknown"


def emit_rs_body(template, recv, class_name, args):
    body = template.replace("@RET@(", "@RET@::new(")
    body = body.replace("@NEW@", f"{class_name}::new")
    body = body.replace("@recv@", recv).replace("@RET@", class_name)
    for index, arg in enumerate(args):
        body = body.replace(f"@{index}@", arg)
    return body


def impl_params(method):
    """Return [(name, rust_type)] for an inline-JS implementation's params."""
    params = []
    for param in method.get("params", []):
        if isinstance(param, str):
            params.append((param, "String"))
        else:
            params.append((param["name"], param.get("type", "String")))
    return params


def emit_rs_impl(method):
    """Emit a method whose body is a JS function declared in the profile."""
    name = rust_ident(method.get("as") or method["name"])
    doc = method.get("name")
    spec = method["impl"]
    returns = spec.get("returns", "void")

    params = impl_params(method)
    signature = ", ".join(f"{pname}: {ptype}" for pname, ptype in params)
    comma = ", " if signature else ""

    if "cdp" in spec:
        cdp = spec["cdp"]
        entries = []
        for key, value in cdp.get("params", {}).items():
            if isinstance(value, str) and value.startswith("${") and value.endswith("}"):
                entries.append(f"{rust_string(key)}: {value[2:-1]}")
            else:
                entries.append(f"{rust_string(key)}: {rust_literal(value)}")
        template = "{ " + ", ".join(entries) + " }"
        call = (
            f"self.inner.execute_cdp_cmd({rust_string(cdp['method'])}.to_string(), "
            f"serde_json::json!({template}).to_string()).await?"
        )
        if returns == "void":
            body, ret_type = f"{call};\nOk(())", "()"
        else:
            body, ret_type = f"Ok({call})", "String"
    else:
        js = rust_string(spec["js"])
        arg_list = ", ".join(pname for pname, _ in params)
        args_json = (
            f"serde_json::json!([{arg_list}]).to_string()" if arg_list else '"[]".to_string()'
        )
        if returns == "void":
            body = f"self.inner.call_json({js}.to_string(), {args_json}).await?;\nOk(())"
            ret_type = "()"
        elif returns == "bool":
            body = f"Ok(self.inner.call_bool({js}.to_string(), {args_json}).await?)"
            ret_type = "bool"
        elif returns == "string":
            body = f"Ok(self.inner.call_string({js}.to_string(), {args_json}).await?)"
            ret_type = "String"
        else:  # json
            body = f"Ok(self.inner.call_json({js}.to_string(), {args_json}).await?)"
            ret_type = "String"

    body_lines = "\n".join(f"        {line}" for line in body.split("\n"))
    return (
        f"    /// `{doc}`.\n"
        f"    pub async fn {name}(&self{comma}{signature}) -> Result<{ret_type}, XcelerateError> {{\n"
        f"{body_lines}\n"
        f"    }}"
    )


def emit_rs_method(method, class_name):
    name = rust_ident(method.get("as") or method["name"])
    doc = method.get("name")

    if "impl" in method:
        return emit_rs_impl(method)

    if "op" not in method:
        reason = method.get("unsupported", "not supported")
        keyword_attr = "pub fn" if method.get("property") else "pub async fn"
        return (
            f"    /// `{doc}` - not available.\n"
            f"    {keyword_attr} {name}(&self) -> Result<(), XcelerateError> {{\n"
            f"        Err(XcelerateError::Unsupported({rust_string(reason)}.to_string()))\n"
            f"    }}"
        )

    op = OPS[method["op"]]
    params = op["params"]
    ignored = op.get("ignored", set())
    ret = op["ret"]

    signature = []
    args = []
    for pname, ptype in params:
        arg_name = f"_{pname}" if pname in ignored else pname
        signature.append(f"{arg_name}: {ptype}")
        args.append(arg_name)
    signature_str = ", ".join(signature)

    if ret == "list":
        returns = method.get("returns")
        if not returns:
            raise SystemExit(f"[RUST] {class_name}.{doc}: op {method['op']} needs a return class")
        ret_type = f"Vec<{returns}>"
    elif ret == "wrap":
        returns = method.get("returns")
        if not returns:
            raise SystemExit(f"[RUST] {class_name}.{doc}: op {method['op']} needs a return class")
        ret_type = returns
    else:
        ret_type = RET_TYPES[ret]

    body = emit_rs_body(op["body"], "self.inner", method.get("returns") or "", args)
    body_lines = "\n".join(f"        {line}" for line in body.split("\n"))
    comma = ", " if signature_str else ""
    return (
        f"    /// `{doc}`.\n"
        f"    pub async fn {name}(&self{comma}{signature_str}) -> Result<{ret_type}, XcelerateError> {{\n"
        f"{body_lines}\n"
        f"    }}"
    )


def emit_rs_class(class_name, spec):
    kind = class_kind(spec)
    field_type = FIELD_TYPES[kind]
    lines = [
        f"/// {spec.get('doc', class_name + ' adapter.')}",
        f"pub struct {class_name} {{",
        f"    inner: {field_type},",
        "}",
        "",
        f"impl {class_name} {{",
        "    /// Wrap an existing xcelerate object.",
        f"    pub fn new(inner: {field_type}) -> Self {{",
        "        Self { inner }",
        "    }",
    ]
    for method in spec.get("methods", []):
        lines.append("")
        lines.append(emit_rs_method(method, class_name))
    lines.append("}")
    return "\n".join(lines)


def emit_rs_helper(helper):
    op = OPS[helper["op"]]
    returns = helper["returns"]
    signature = ", ".join(f"{n}: {t}" for n, t in op["params"])
    args = [n for n, _ in op["params"]]
    body = emit_rs_body(op["body"], "", returns, args)
    body_lines = "\n".join(f"    {line}" for line in body.split("\n"))
    return (
        f"/// {helper['name']} entry point.\n"
        f"pub async fn {helper['name']}({signature}) -> Result<{returns}, XcelerateError> {{\n"
        f"{body_lines}\n"
        f"}}"
    )


def emit_rs_module(profile):
    lines = [
        "#![allow(unused_imports)]",
        "#![allow(non_snake_case)]",
        "#![allow(dead_code)]",
        "#![allow(clippy::needless_question_mark)]",
        f"//! Generated adapter: {profile['title']}. DO NOT EDIT.",
        "//!",
        "//! Regenerate with: python scripts/generate_adapters.py --target rust",
        "",
        "use std::sync::Arc;",
        "",
        "use crate::{",
        "    Browser as CoreBrowser, BrowserConfig, Element as CoreElement, Page as CorePage,",
        "    XcelerateError,",
        "};",
        "",
    ]
    for helper in profile.get("helpers", []):
        lines.append(emit_rs_helper(helper))
        lines.append("")
    for class_name, spec in profile.get("classes", {}).items():
        lines.append(emit_rs_class(class_name, spec))
        lines.append("")
    return "\n".join(lines).rstrip() + "\n"


def emit_rs_mod(module_names):
    mod = [
        "//! Generated API-style adapters. DO NOT EDIT.",
        "//!",
        "//! Regenerate with: python scripts/generate_adapters.py --target rust",
        "",
    ]
    for name in module_names:
        mod.append(f"pub mod {name};")
    mod.append("")
    return "\n".join(mod)


def generate_rust(profiles):
    os.makedirs(RUST_ADAPTERS_DIR, exist_ok=True)

    with open(os.path.join(RUST_ADAPTERS_DIR, "support.rs"), "w", encoding="utf-8") as handle:
        handle.write(SUPPORT_RS)

    module_names = ["support"]
    for profile in profiles:
        module_names.append(profile["name"])
        path = os.path.join(RUST_ADAPTERS_DIR, f"{profile['name']}.rs")
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(emit_rs_module(profile))
        log("RUST", f"generated adapters/{profile['name']}.rs")

    with open(os.path.join(RUST_ADAPTERS_DIR, "mod.rs"), "w", encoding="utf-8") as handle:
        handle.write(emit_rs_mod(module_names))
    log("RUST", "generated adapters/mod.rs + adapters/support.rs")


# ===========================================================================
# CLI
# ===========================================================================

def main():
    parser = argparse.ArgumentParser(description="Generate API-style adapters from profiles.")
    parser.add_argument(
        "--target",
        choices=["all", "python", "rust"],
        default="all",
        help="which adapters to emit (default: all)",
    )
    parser.add_argument(
        "--out-dir",
        default=PY_OUT_DIR,
        help="Python binding package directory to write into",
    )
    args = parser.parse_args()

    profiles = load_profiles()
    if not profiles:
        log("ADAPTER", "no profiles found")
        return 1

    if args.target in ("all", "python"):
        generate_python(profiles, args.out_dir)
    if args.target in ("all", "rust"):
        generate_rust(profiles)

    log("ADAPTER", f"done ({args.target})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
