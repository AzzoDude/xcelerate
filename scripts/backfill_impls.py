#!/usr/bin/env python3
"""Backfill real implementations into the adapter profiles.

The harvesters produce a *full* upstream surface, which means every member
xcelerate could not yet express was recorded as an explicit ``unsupported``
stub. This script replaces those stubs with real behaviour, expressed either as

* an inline JS implementation (``impl`` - evaluated in the page/element via the
  core ``call_*`` bridge), or
* a named op (``op``) implemented in ``adapters/runtime.py`` + the Rust op table.

It is a migration tool: it only rewrites entries that are still marked
``unsupported``, so re-running it is idempotent and it never touches methods
that already work.

Usage::

    python scripts/backfill_impls.py            # rewrite profiles in place
    python scripts/backfill_impls.py --dry-run  # report only
"""

from __future__ import annotations

import argparse
import re

from common import PROFILES_DIR, load_profiles, log, write_profile

# Profile class -> receiver kind, mirroring the generators' op table.
KIND = {
    "playwright": {
        "Browser": "browser",
        "BrowserContext": "context",
        "Page": "page",
        "Locator": "element",
    },
    "puppeteer": {
        "Browser": "browser",
        "BrowserContext": "context",
        "Page": "page",
        "ElementHandle": "element",
        "Frame": "page",
    },
    "selenium": {"WebDriver": "driver", "WebElement": "element"},
}

# Reusable JS snippets.
_VISIBLE = (
    'function(){const s=getComputedStyle(this);const r=this.getBoundingClientRect();'
    'return !!(r.width||r.height)&&s.visibility!=="hidden"&&s.display!=="none"&&s.opacity!=="0";}'
)
_HIDDEN = (
    'function(){const s=getComputedStyle(this);const r=this.getBoundingClientRect();'
    'return !(!!(r.width||r.height)&&s.visibility!=="hidden"&&s.display!=="none");}'
)
_RECT = 'function(){const r=this.getBoundingClientRect();return {x:r.x,y:r.y,width:r.width,height:r.height};}'

# (kind, upstream-name) -> method definition.
IMPLS = {}


def _impl(scope, returns, js, params=None):
    entry = {"impl": {"scope": scope, "returns": returns, "js": js}}
    if params:
        entry["params"] = params
    return entry


def _op(op, params=None, returns=None, many=False):
    entry = {"op": op, "returns": returns}
    if params:
        entry["params"] = params
    if many:
        entry["list"] = True
    return entry


def el(name, returns, js, params=None):
    IMPLS[("element", name)] = _impl("element", returns, js, params)


def pg(name, returns, js, params=None):
    IMPLS[("page", name)] = _impl("page", returns, js, params)


def pgo(name, op, params=None, returns=None, many=False):
    IMPLS[("page", name)] = _op(op, params, returns, many)


def elo(name, op, params=None, returns=None, many=False):
    IMPLS[("element", name)] = _op(op, params, returns, many)


# The wrapped element class has a different name in each profile.
ELEMENT_CLASS = {
    "playwright": "Locator",
    "puppeteer": "ElementHandle",
    "selenium": "WebElement",
}

# Classes injected into profiles that the harvester could not discover.
_CDP_SESSION = {
    "doc": "A CDP session (thin wrapper over the page connection).",
    "methods": [
        {"name": "send", "op": "page_execute_cdp_cmd",
         "params": [{"name": "method", "type": "String"},
                    {"name": "params_json", "type": "String"}],
         "returns": None},
        {"name": "detach", "op": "page_detach_cdp_session", "params": [], "returns": None},
        {"name": "on", "op": "page_on",
         "params": [{"name": "event_name", "type": "String"}], "returns": None},
        {"name": "off", "op": "page_remove_listener",
         "params": [{"name": "event_name", "type": "String"}], "returns": None},
    ],
}

_COVERAGE = {
    "doc": "Coverage collector (thin wrapper over the page).",
    "methods": [
        {"name": "startJSCoverage", "op": "page_coverage_start_js", "params": [], "returns": None},
        {"name": "stopJSCoverage", "op": "page_coverage_stop_js", "params": [], "returns": None},
        {"name": "startCSSCoverage", "op": "page_coverage_start_css", "params": [], "returns": None},
        {"name": "stopCSSCoverage", "op": "page_coverage_stop_css", "params": [], "returns": None},
    ],
}

_KEYBOARD = {
    "doc": "Keyboard input (thin wrapper over the page).",
    "methods": [
        {"name": "press", "op": "page_keyboard_press", "params": [{"name": "key", "type": "String"}], "returns": None},
        {"name": "down", "op": "page_keyboard_down", "params": [{"name": "key", "type": "String"}], "returns": None},
        {"name": "up", "op": "page_keyboard_up", "params": [{"name": "key", "type": "String"}], "returns": None},
        {"name": "type", "op": "page_keyboard_type", "params": [{"name": "text", "type": "String"}], "returns": None},
    ],
}

_MOUSE = {
    "doc": "Mouse input (thin wrapper over the page).",
    "methods": [
        {"name": "move", "op": "page_mouse_move", "params": [{"name": "x", "type": "f64"}, {"name": "y", "type": "f64"}], "returns": None},
        {"name": "click", "op": "page_mouse_click", "params": [{"name": "x", "type": "f64"}, {"name": "y", "type": "f64"}], "returns": None},
        {"name": "down", "op": "page_mouse_down", "params": [{"name": "button", "type": "String"}], "returns": None},
        {"name": "up", "op": "page_mouse_up", "params": [{"name": "button", "type": "String"}], "returns": None},
    ],
}

_TOUCHSCREEN = {
    "doc": "Touchscreen input (thin wrapper over the page).",
    "methods": [
        {"name": "tap", "op": "page_touch_tap", "params": [{"name": "x", "type": "f64"}, {"name": "y", "type": "f64"}], "returns": None},
    ],
}

_SWITCH_TO = {
    "doc": "Selenium-style target switching (thin wrapper over the driver).",
    "methods": [
        {"name": "window", "op": "driver_activate_target", "params": [{"name": "handle", "type": "String"}], "returns": None},
        {"name": "default_content", "op": "driver_switch_default_content", "params": [], "returns": None},
        {"name": "new_window", "op": "driver_switch_new_window", "params": [{"name": "window_type", "type": "String"}], "returns": None},
        {"name": "alert", "op": "driver_new_dialog", "params": [], "returns": "Dialog"},
    ],
}

_TIMEOUTS = {
    "doc": "Selenium-style timeouts (thin wrapper over the driver).",
    "methods": [
        {"name": "implicitly_wait", "op": "selenium_implicitly_wait", "params": [{"name": "seconds", "type": "f64"}], "returns": None},
        {"name": "set_script_timeout", "op": "selenium_set_script_timeout", "params": [{"name": "seconds", "type": "f64"}], "returns": None},
        {"name": "page_load_timeout", "op": "selenium_set_page_load_timeout", "params": [{"name": "seconds", "type": "f64"}], "returns": None},
    ],
}

_TRACING = {
    "doc": "Tracing (thin wrapper over the page session).",
    "methods": [
        {"name": "start", "op": "page_tracing_start", "params": [], "returns": None},
        {"name": "stop", "op": "page_tracing_stop", "params": [], "returns": None},
    ],
}

_SCREENCAST = {
    "doc": "Screencast (thin wrapper over the page).",
    "methods": [
        {"name": "start", "op": "page_start_screencast", "params": [], "returns": None},
        {"name": "stop", "op": "page_stop_screencast", "params": [], "returns": None},
    ],
}

_DIALOG = {
    "doc": "JavaScript dialog (thin wrapper over the driver).",
    "methods": [
        {"name": "accept", "op": "driver_dialog_accept", "params": [], "returns": None},
        {"name": "dismiss", "op": "driver_dialog_dismiss", "params": [], "returns": None},
        {"name": "send_keys", "op": "driver_dialog_send_keys", "params": [{"name": "text", "type": "String"}], "returns": None},
    ],
}

EXTRA_CLASSES = {
    "playwright": {"CDPSession": _CDP_SESSION, "Keyboard": _KEYBOARD, "Mouse": _MOUSE, "Touchscreen": _TOUCHSCREEN, "Screencast": _SCREENCAST},
    "puppeteer": {"CDPSession": _CDP_SESSION, "Coverage": _COVERAGE, "Keyboard": _KEYBOARD, "Mouse": _MOUSE, "Touchscreen": _TOUCHSCREEN, "Tracing": _TRACING, "Screencast": _SCREENCAST},
    "selenium": {"SwitchTo": _SWITCH_TO, "Timeouts": _TIMEOUTS, "Dialog": _DIALOG},
}


def drv(name, op, params=None, returns=None, many=False):
    IMPLS[("driver", name)] = _op(op, params, returns, many)


def brw(name, op, params=None, returns=None, many=False):
    IMPLS[("browser", name)] = _op(op, params, returns, many)


def ctx(name, op, params=None, returns=None, many=False):
    IMPLS[("context", name)] = _op(op, params, returns, many)


# --- Element state / properties -------------------------------------------
el("is_visible", "bool", _VISIBLE)
el("is_displayed", "bool", _VISIBLE)
el("visible", "bool", _VISIBLE)
el("is_hidden", "bool", _HIDDEN)
el("is_enabled", "bool", 'function(){return !(this.disabled===true||this.hasAttribute("disabled"));}')
el("is_disabled", "bool", 'function(){return this.disabled===true||this.hasAttribute("disabled");}')
el("is_checked", "bool", "function(){return this.checked===true;}")
el("is_selected", "bool", "function(){return this.selected===true||this.checked===true;}")
el(
    "is_editable",
    "bool",
    'function(){return !(this.disabled===true||this.readOnly===true||this.hasAttribute("readonly"));}',
)
el("is_empty", "bool", "function(){return !this.value&&!this.textContent&&!this.innerHTML;}")
el("tag_name", "string", 'function(){return this.tagName?this.tagName.toLowerCase():"unknown";}')
el("inner_text", "string", "function(){return this.innerText;}")
el("text_content", "string", "function(){return this.textContent;}")
el("input_value", "string", 'function(){return this.value!=null?String(this.value):"";}')
el("inner_html", "string", "function(){return this.innerHTML;}")
el("shadow_root", "string", 'function(){return this.shadowRoot?this.shadowRoot.innerHTML:"";}')
el(
    "aria_role",
    "string",
    'function(){return this.getAttribute("role")||this.tagName.toLowerCase();}',
)
el(
    "accessible_name",
    "string",
    'function(){return (this.getAttribute("aria-label")||this.textContent||"").trim();}',
)
el("describe", "string", "function(){return this.outerHTML.slice(0,120);}")
el("bounding_box", "json", _RECT)
el("location", "json", _RECT)
el("rect", "json", _RECT)
el("size", "json", 'function(){const r=this.getBoundingClientRect();return {width:r.width,height:r.height};}')
el(
    "location_once_scrolled_into_view",
    "json",
    'function(){this.scrollIntoView({block:"center",inline:"center"});'
    "const r=this.getBoundingClientRect();return {x:r.x,y:r.y};}",
)

# --- Element actions -------------------------------------------------------
el("submit", "void", 'function(){if(this.form){this.form.submit();}else if(this.tagName==="FORM"){this.submit();}}')
el("blur", "void", "function(){this.blur();}")
el("scroll_into_view_if_needed", "void", 'function(){this.scrollIntoView({block:"center",inline:"center"});}')
el("highlight", "void", 'function(){this.style.outline="2px solid red";}')
el("hide_highlight", "void", 'function(){this.style.outline="";}')
el("check", "void", 'function(){if(this.checked!==true&&this.type!==undefined){this.click();}}')
el("uncheck", "void", "function(){if(this.checked===true){this.click();}}")
el("tap", "void", "function(){this.click();}")
el(
    "select_text",
    "void",
    "function(){const r=document.createRange();r.selectNodeContents(this);"
    "const s=getSelection();s.removeAllRanges();s.addRange(r);}",
)

el(
    "get_property",
    "string",
    'function(n){const v=this[n];return v==null?"":String(v);}',
    [{"name": "name", "type": "String"}],
)
el(
    "value_of_css_property",
    "string",
    "function(p){return getComputedStyle(this).getPropertyValue(p);}",
    [{"name": "property_name", "type": "String"}],
)
el(
    "set_checked",
    "void",
    "function(v){if(this.checked!==!!v){this.click();}}",
    [{"name": "checked", "type": "bool"}],
)
el(
    "dispatch_event",
    "void",
    "function(t){this.dispatchEvent(new Event(t,{bubbles:true,cancelable:true}));}",
    [{"name": "event_type", "type": "String"}],
)
el(
    "drag_to",
    "void",
    'function(sel){const t=document.querySelector(sel);if(!t)return;const dt=new DataTransfer();'
    'this.dispatchEvent(new DragEvent("dragstart",{bubbles:true,dataTransfer:dt}));'
    't.dispatchEvent(new DragEvent("dragenter",{bubbles:true,dataTransfer:dt}));'
    't.dispatchEvent(new DragEvent("dragover",{bubbles:true,dataTransfer:dt}));'
    't.dispatchEvent(new DragEvent("drop",{bubbles:true,dataTransfer:dt}));'
    'this.dispatchEvent(new DragEvent("dragend",{bubbles:true,dataTransfer:dt}));}',
    [{"name": "target", "type": "String"}],
)
el(
    "fill",
    "void",
    "function(v){this.focus();this.value=v;"
    'this.dispatchEvent(new Event("input",{bubbles:true}));'
    'this.dispatchEvent(new Event("change",{bubbles:true}));}',
    [{"name": "value", "type": "String"}],
)

# --- Page: navigation / content / JS --------------------------------------
pg("url", "string", "function(){return location.href;}")
pg("set_content", "void", "function(h){document.open();document.write(h);document.close();}",
   [{"name": "html", "type": "String"}])
pg("viewport_size", "json", "function(){return {width:window.innerWidth,height:window.innerHeight};}")
pg("evaluate", "json", 'function(src){return (new Function("return ("+src+")"))();}',
   [{"name": "expression", "type": "String"}])
pg(
    "wait_for_url",
    "void",
    "function(u){return new Promise(res=>{const check=()=>{"
    'if(location.href===u||location.href.indexOf(u)===0){res(true);}else{setTimeout(check,50);}};'
    "check();});}",
    [{"name": "url", "type": "String"}],
)
pg("add_style_tag", "string", "function(c){const s=document.createElement('style');s.textContent=c;"
   "document.head.appendChild(s);return s.textContent;}",
   [{"name": "content", "type": "String"}])

# --- Page: selector-based actions -----------------------------------------
pg("press", "void", "function(sel,key){const e=document.querySelector(sel);if(!e)return;e.focus();"
   'e.dispatchEvent(new KeyboardEvent("keydown",{key:key,bubbles:true}));'
   'e.dispatchEvent(new KeyboardEvent("keyup",{key:key,bubbles:true}));}',
   [{"name": "selector", "type": "String"}, {"name": "key", "type": "String"}])
pg("check", "void", "function(sel){const e=document.querySelector(sel);if(e&&e.checked!==true)e.click();}",
   [{"name": "selector", "type": "String"}])
pg("uncheck", "void", "function(sel){const e=document.querySelector(sel);if(e&&e.checked===true)e.click();}",
   [{"name": "selector", "type": "String"}])
pg("set_checked", "void",
   "function(sel,v){const e=document.querySelector(sel);if(e&&e.checked!==!!v)e.click();}",
   [{"name": "selector", "type": "String"}, {"name": "checked", "type": "bool"}])
pg("tap", "void", "function(sel){const e=document.querySelector(sel);if(e)e.click();}",
   [{"name": "selector", "type": "String"}])
pg("input_value", "string",
   'function(sel){const e=document.querySelector(sel);return e&&e.value!=null?String(e.value):"";}',
   [{"name": "selector", "type": "String"}])
pg("is_visible", "bool",
   "function(sel){const e=document.querySelector(sel);if(!e)return false;const s=getComputedStyle(e);"
   'const r=e.getBoundingClientRect();return !!(r.width||r.height)&&s.visibility!=="hidden"&&s.display!=="none";}',
   [{"name": "selector", "type": "String"}])
pg("is_hidden", "bool",
   "function(sel){const e=document.querySelector(sel);if(!e)return true;const s=getComputedStyle(e);"
   'const r=e.getBoundingClientRect();return !(!!(r.width||r.height)&&s.visibility!=="hidden"&&s.display!=="none");}',
   [{"name": "selector", "type": "String"}])
pg("is_enabled", "bool",
   'function(sel){const e=document.querySelector(sel);return !!e&&!(e.disabled===true||e.hasAttribute("disabled"));}',
   [{"name": "selector", "type": "String"}])
pg("is_disabled", "bool",
   'function(sel){const e=document.querySelector(sel);return !!e&&(e.disabled===true||e.hasAttribute("disabled"));}',
   [{"name": "selector", "type": "String"}])
pg("is_checked", "bool",
   "function(sel){const e=document.querySelector(sel);return !!e&&e.checked===true;}",
   [{"name": "selector", "type": "String"}])
pg("is_editable", "bool",
   'function(sel){const e=document.querySelector(sel);return !!e&&!(e.disabled===true||e.readOnly===true||e.hasAttribute("readonly"));}',
   [{"name": "selector", "type": "String"}])
pg("dispatch_event", "void",
   "function(sel,t){const e=document.querySelector(sel);if(e)e.dispatchEvent(new Event(t,{bubbles:true,cancelable:true}));}",
   [{"name": "selector", "type": "String"}, {"name": "event_type", "type": "String"}])

# --- Page: CDP-backed ------------------------------------------------------
pgo("go_forward", "page_go_forward")
pgo("close", "page_close")
pgo("bring_to_front", "page_bring_to_front")
pgo("set_input_files", "page_set_input_files",
    [{"name": "selector", "type": "String"}, {"name": "files_json", "type": "String"}], None)
pgo("select_option", "page_select_option",
    [{"name": "selector", "type": "String"}, {"name": "values_json", "type": "String"}], None)
pgo("set_extra_http_headers", "page_set_extra_http_headers",
    [{"name": "headers_json", "type": "String"}], None)
pgo("emulate_media", "page_emulate_media",
    [{"name": "media", "type": "Option<String>"}, {"name": "color_scheme", "type": "Option<String>"}], None)

# --- Selenium WebDriver ----------------------------------------------------
drv("current_url", "selenium_current_url", None, None)
drv("forward", "selenium_forward", None, None)
drv("execute_script", "selenium_execute_script", [{"name": "script", "type": "String"}], None)
drv("execute_async_script", "selenium_execute_async_script", [{"name": "script", "type": "String"}], None)

# --- Queries / handles (page) ---------------------------------------------
pgo("query_selector", "page_find", [{"name": "selector", "type": "String"}], "@ELEMENT@")
pgo("querySelector", "page_find", [{"name": "selector", "type": "String"}], "@ELEMENT@")
pgo("$", "page_find", [{"name": "selector", "type": "String"}], "@ELEMENT@")
pgo("query_selector_all", "page_find_all", [{"name": "selector", "type": "String"}],
    "@ELEMENT@", many=True)
pgo("querySelectorAll", "page_find_all", [{"name": "selector", "type": "String"}],
    "@ELEMENT@", many=True)
pgo("$$", "page_find_all", [{"name": "selector", "type": "String"}], "@ELEMENT@", many=True)
pgo("$x", "page_query_selector_xpath", [{"name": "xpath", "type": "String"}], "@ELEMENT@")
pgo("querySelectorEval", "page_eval_on_selector",
    [{"name": "selector", "type": "String"}, {"name": "expression", "type": "String"}], None)
pgo("$eval", "page_eval_on_selector",
    [{"name": "selector", "type": "String"}, {"name": "expression", "type": "String"}], None)
pgo("querySelectorAllEval", "page_eval_on_selector_all",
    [{"name": "selector", "type": "String"}, {"name": "expression", "type": "String"}], None)
pgo("$$eval", "page_eval_on_selector_all",
    [{"name": "selector", "type": "String"}, {"name": "expression", "type": "String"}], None)
pgo("evaluate_handle", "page_evaluate_handle", [{"name": "expression", "type": "String"}],
    "@ELEMENT@")
pgo("evaluateHandle", "page_evaluate_handle", [{"name": "expression", "type": "String"}],
    "@ELEMENT@")
pgo("wait_for_function", "page_wait_for_function", [{"name": "expression", "type": "String"}], None)
pgo("waitForFunction", "page_wait_for_function", [{"name": "expression", "type": "String"}], None)
pgo("set_user_agent", "page_set_user_agent",
    [{"name": "user_agent", "type": "String"}, {"name": "accept_language", "type": "Option<String>"}],
    None)
pgo("setUserAgent", "page_set_user_agent",
    [{"name": "user_agent", "type": "String"}, {"name": "accept_language", "type": "Option<String>"}],
    None)
pgo("set_cache_enabled", "page_set_cache_enabled", [{"name": "enabled", "type": "bool"}], None)
pgo("setCacheEnabled", "page_set_cache_enabled", [{"name": "enabled", "type": "bool"}], None)
pgo("set_javascript_enabled", "page_set_javascript_enabled", [{"name": "enabled", "type": "bool"}], None)
pgo("setJavaScriptEnabled", "page_set_javascript_enabled", [{"name": "enabled", "type": "bool"}], None)
pgo("set_offline", "page_set_offline", [{"name": "offline", "type": "bool"}], None)
pgo("setOffline", "page_set_offline", [{"name": "offline", "type": "bool"}], None)
pgo("setOfflineMode", "page_set_offline", [{"name": "offline", "type": "bool"}], None)
pgo("cookies", "page_cookies", None, None)
pgo("metrics", "page_metrics", None, None)

# --- Element child queries / uploads --------------------------------------
elo("$", "element_find", [{"name": "selector", "type": "String"}], "@ELEMENT@")
elo("query_selector", "element_find", [{"name": "selector", "type": "String"}], "@ELEMENT@")
elo("querySelector", "element_find", [{"name": "selector", "type": "String"}], "@ELEMENT@")
elo("uploadFile", "element_set_input_files", [{"name": "files_json", "type": "String"}], None)
elo("setInputFiles", "element_set_input_files", [{"name": "files_json", "type": "String"}], None)
el(
    "evaluate",
    "json",
    "function(src){return (new Function('el','return ('+src+')(el);'))(this);}",
    [{"name": "expression", "type": "String"}],
)
el(
    "press",
    "void",
    "function(key){this.focus();"
    'this.dispatchEvent(new KeyboardEvent("keydown",{key:key,bubbles:true}));'
    'this.dispatchEvent(new KeyboardEvent("keyup",{key:key,bubbles:true}));}',
    [{"name": "key", "type": "String"}],
)


# --- CDP-backed page methods (data-driven) --------------------------------
def pgc(name, returns, cdp_method, params=None, cdp_params=None):
    IMPLS[("page", name)] = {
        "impl": {"scope": "page", "returns": returns,
                 "cdp": {"method": cdp_method, "params": cdp_params or {}}},
        "params": params,
    }


pgc("set_timezone", "void", "Emulation.setTimezoneOverride",
    [{"name": "timezone_id", "type": "String"}], {"timezoneId": "${timezone_id}"})
pgc("emulate_timezone", "void", "Emulation.setTimezoneOverride",
    [{"name": "timezone_id", "type": "String"}], {"timezoneId": "${timezone_id}"})
pgc("emulateTimezone", "void", "Emulation.setTimezoneOverride",
    [{"name": "timezone_id", "type": "String"}], {"timezoneId": "${timezone_id}"})
pgc("set_locale", "void", "Emulation.setLocaleOverride",
    [{"name": "locale", "type": "String"}], {"locale": "${locale}"})
pgc("emulate_locale", "void", "Emulation.setLocaleOverride",
    [{"name": "locale", "type": "String"}], {"locale": "${locale}"})
pgc("emulateLocale", "void", "Emulation.setLocaleOverride",
    [{"name": "locale", "type": "String"}], {"locale": "${locale}"})
pgc("set_cpu_throttling", "void", "Emulation.setCPUThrottlingRate",
    [{"name": "rate", "type": "f64"}], {"rate": "${rate}"})
pgc("emulate_cpu_throttling", "void", "Emulation.setCPUThrottlingRate",
    [{"name": "rate", "type": "f64"}], {"rate": "${rate}"})
pgc("emulateCPUThrottling", "void", "Emulation.setCPUThrottlingRate",
    [{"name": "rate", "type": "f64"}], {"rate": "${rate}"})
pgc("set_vision_deficiency", "void", "Emulation.setEmulatedVisionDeficiency",
    [{"name": "kind", "type": "String"}], {"type": "${kind}"})
pgc("emulate_vision_deficiency", "void", "Emulation.setEmulatedVisionDeficiency",
    [{"name": "kind", "type": "String"}], {"type": "${kind}"})
pgc("emulateVisionDeficiency", "void", "Emulation.setEmulatedVisionDeficiency",
    [{"name": "kind", "type": "String"}], {"type": "${kind}"})
pgo("emulate_media", "page_emulate_media",
    [{"name": "media", "type": "Option<String>"},
     {"name": "color_scheme", "type": "Option<String>"}], None)
pgc("emulateMedia", "void", "Emulation.setEmulatedMedia",
    [{"name": "media", "type": "Option<String>"}], {"media": "${media}"})
pgc("emulate_media_type", "void", "Emulation.setEmulatedMedia",
    [{"name": "media", "type": "String"}], {"media": "${media}"})
pgc("emulate_network_conditions", "void", "Network.emulateNetworkConditions",
    [{"name": "offline", "type": "bool"}, {"name": "latency", "type": "f64"},
     {"name": "download", "type": "f64"}, {"name": "upload", "type": "f64"}],
    {"offline": "${offline}", "latency": "${latency}",
     "downloadThroughput": "${download}", "uploadThroughput": "${upload}"})
pgc("emulateNetworkConditions", "void", "Network.emulateNetworkConditions",
    [{"name": "offline", "type": "bool"}, {"name": "latency", "type": "f64"},
     {"name": "download", "type": "f64"}, {"name": "upload", "type": "f64"}],
    {"offline": "${offline}", "latency": "${latency}",
     "downloadThroughput": "${download}", "uploadThroughput": "${upload}"})
pgc("set_geolocation", "void", "Emulation.setGeolocationOverride",
    [{"name": "latitude", "type": "f64"}, {"name": "longitude", "type": "f64"},
     {"name": "accuracy", "type": "f64"}],
    {"latitude": "${latitude}", "longitude": "${longitude}", "accuracy": "${accuracy}"})
pgc("setGeolocation", "void", "Emulation.setGeolocationOverride",
    [{"name": "latitude", "type": "f64"}, {"name": "longitude", "type": "f64"},
     {"name": "accuracy", "type": "f64"}],
    {"latitude": "${latitude}", "longitude": "${longitude}", "accuracy": "${accuracy}"})
pgc("set_bypass_csp", "void", "Page.setBypassCSP",
    [{"name": "enabled", "type": "bool"}], {"enabled": "${enabled}"})
pgc("setBypassCSP", "void", "Page.setBypassCSP",
    [{"name": "enabled", "type": "bool"}], {"enabled": "${enabled}"})
pgc("setViewport", "void", "Emulation.setDeviceMetricsOverride",
    [{"name": "width", "type": "u64"}, {"name": "height", "type": "i64"}],
    {"width": "${width}", "height": "${height}", "deviceScaleFactor": 1, "mobile": False})
pgc("set_cookie", "void", "Network.setCookie",
    [{"name": "name", "type": "String"}, {"name": "value", "type": "String"},
     {"name": "url", "type": "String"}],
    {"name": "${name}", "value": "${value}", "url": "${url}"})
pgc("setCookie", "void", "Network.setCookie",
    [{"name": "name", "type": "String"}, {"name": "value", "type": "String"},
     {"name": "url", "type": "String"}],
    {"name": "${name}", "value": "${value}", "url": "${url}"})
pgc("delete_cookie", "void", "Network.deleteCookies",
    [{"name": "name", "type": "String"}], {"name": "${name}"})
pgc("deleteCookie", "void", "Network.deleteCookies",
    [{"name": "name", "type": "String"}], {"name": "${name}"})
pgc("delete_all_cookies", "void", "Network.clearBrowserCookies", [], {})
pgc("set_download_behavior", "void", "Page.setDownloadBehavior",
    [{"name": "path", "type": "String"}],
    {"behavior": "allow", "downloadPath": "${path}"})
pgc("setDownloadBehavior", "void", "Page.setDownloadBehavior",
    [{"name": "path", "type": "String"}],
    {"behavior": "allow", "downloadPath": "${path}"})

# --- Selenium cookies / raw CDP -------------------------------------------
drv("execute_cdp_cmd", "selenium_execute_cdp_cmd",
    [{"name": "method", "type": "String"}, {"name": "params_json", "type": "String"}], None)
drv("get_cookies", "selenium_get_cookies", None, None)
drv("add_cookie", "selenium_add_cookie", [{"name": "cookie_json", "type": "String"}], None)
drv("delete_cookie", "selenium_delete_cookie", [{"name": "name", "type": "String"}], None)
drv("delete_all_cookies", "selenium_delete_all_cookies", None, None)


# --- More page JS / storage ------------------------------------------------
pg("local_storage", "json", "function(){return Object.fromEntries(Object.entries(localStorage));}")
pg("session_storage", "json", "function(){return Object.fromEntries(Object.entries(sessionStorage));}")
pg("viewport", "json", "function(){return {width:window.innerWidth,height:window.innerHeight};}")
pg(
    "hide_highlight",
    "void",
    'function(){document.querySelectorAll("*").forEach(function(e){e.style.outline="";});}',
)
pg(
    "select",
    "void",
    "function(sel,valuesRaw){const e=document.querySelector(sel);if(!e)return;"
    "const w=JSON.parse(valuesRaw).map(String);for(const o of e.options){"
    "o.selected=w.includes(o.value)||w.includes(o.text);}"
    "e.dispatchEvent(new Event('change',{bubbles:true}));}",
    [{"name": "selector", "type": "String"}, {"name": "values_json", "type": "String"}],
)
pgo("eval_on_selector", "page_eval_on_selector",
    [{"name": "selector", "type": "String"}, {"name": "expression", "type": "String"}], None)
pgo("eval_on_selector_all", "page_eval_on_selector_all",
    [{"name": "selector", "type": "String"}, {"name": "expression", "type": "String"}], None)
pgo("set_viewport_size", "page_set_viewport_size",
    [{"name": "width", "type": "u64"}, {"name": "height", "type": "i64"}], None)
pgo("get_by_role", "page_get_by_role", [{"name": "role", "type": "String"}], "@ELEMENT@")
pgo("get_by_text", "page_get_by_text", [{"name": "text", "type": "String"}], "@ELEMENT@")
pgo("get_by_label", "page_get_by_label", [{"name": "label", "type": "String"}], "@ELEMENT@")
pgo("get_by_placeholder", "page_get_by_placeholder", [{"name": "text", "type": "String"}], "@ELEMENT@")
pgo("get_by_alt_text", "page_get_by_alt_text", [{"name": "text", "type": "String"}], "@ELEMENT@")
pgo("get_by_title", "page_get_by_title", [{"name": "text", "type": "String"}], "@ELEMENT@")
pgo("get_by_test_id", "page_get_by_test_id", [{"name": "test_id", "type": "String"}], "@ELEMENT@")
pgo("set_extra_http_headers", "page_set_extra_http_headers",
    [{"name": "headers_json", "type": "String"}], None)
pgo("setExtraHTTPHeaders", "page_set_extra_http_headers",
    [{"name": "headers_json", "type": "String"}], None)
pgc("request_gc", "void", "HeapProfiler.collectGarbage", [], {})
pgc("clear_console_messages", "void", "Runtime.discardConsoleEntries", [], {})
pgc("clear_page_errors", "void", "Runtime.discardConsoleEntries", [], {})
pgc("aria_snapshot", "json", "Accessibility.getFullAXTree", [], {})
pgc("accessibility", "json", "Accessibility.getFullAXTree", [], {})
pgc("emulateFocusedPage", "void", "Emulation.setFocusEmulationEnabled",
    [{"name": "enabled", "type": "bool"}], {"enabled": "${enabled}"})

# --- Puppeteer J-aliases / xpath ------------------------------------------
pgo("J", "page_find", [{"name": "selector", "type": "String"}], "@ELEMENT@")
pgo("JJ", "page_find_all", [{"name": "selector", "type": "String"}], "@ELEMENT@", many=True)
pgo("Jx", "page_query_selector_xpath", [{"name": "xpath", "type": "String"}], "@ELEMENT@")
pgo("xpath", "page_query_selector_xpath", [{"name": "xpath", "type": "String"}], "@ELEMENT@")
pgo("Jeval", "page_eval_on_selector",
    [{"name": "selector", "type": "String"}, {"name": "expression", "type": "String"}], None)
pgo("JJeval", "page_eval_on_selector_all",
    [{"name": "selector", "type": "String"}, {"name": "expression", "type": "String"}], None)
pgo("uploadFile", "page_set_input_files",
    [{"name": "selector", "type": "String"}, {"name": "files_json", "type": "String"}], None)

# --- ElementHandle extras --------------------------------------------------
elo("$$", "element_find_all", [{"name": "selector", "type": "String"}], "@ELEMENT@", many=True)
elo("querySelectorAll", "element_find_all", [{"name": "selector", "type": "String"}],
    "@ELEMENT@", many=True)
elo("find_element", "element_find", [{"name": "selector", "type": "String"}], "@ELEMENT@")
elo("find_elements", "element_find_all", [{"name": "selector", "type": "String"}],
    "@ELEMENT@", many=True)
elo("parent", "element_parent", None, "@ELEMENT@")
elo("evaluateHandle", "element_evaluate_handle", [{"name": "expression", "type": "String"}],
    "@ELEMENT@")
elo("evaluate_handle", "element_evaluate_handle", [{"name": "expression", "type": "String"}],
    "@ELEMENT@")
elo("screenshot", "element_screenshot", None, None)
elo("screenshot_as_png", "element_screenshot", None, None)
elo("screenshot_as_base64", "element_screenshot_base64", None, None)
el("id", "string", "function(){return this.id||'';}")
el("all_inner_texts", "json", "function(){return [this.innerText];}")
el("all_text_contents", "json", "function(){return [this.textContent];}")
el("description", "string", "function(){return this.outerHTML;}")
elo("first", "element_self", None, "@ELEMENT@")
elo("last", "element_self", None, "@ELEMENT@")
elo("nth", "element_nth", [{"name": "index", "type": "i64"}], "@ELEMENT@")
elo("count", "element_count", None, None)
elo("asElement", "element_self", None, "@ELEMENT@")
elo("toElement", "element_self", None, "@ELEMENT@")
pg("plainText", "string", "function(){return document.body?document.body.innerText:'';}")
pgo("waitForNetworkIdle", "page_wait_for_network_idle", None, None)
pgo("waitForNetworkIdle$", "page_wait_for_network_idle", None, None)
pgo("waitForFrame", "page_wait_for_frame", None, None)
pgo("waitForDevicePrompt", "page_wait_for_device_prompt", None, None)
el("boxModel", "json", _RECT)
el("clickablePoint", "json",
   "function(){const r=this.getBoundingClientRect();return {x:r.x+r.width/2,y:r.y+r.height/2};}")
el("isIntersectingViewport", "bool",
   "function(){const r=this.getBoundingClientRect();return r.top>=0&&r.left>=0&&"
   "r.bottom<=(window.innerHeight||document.documentElement.clientHeight)&&"
   "r.right<=(window.innerWidth||document.documentElement.clientWidth);}")
el("scrollIntoView", "void", "function(){this.scrollIntoView();}")
el("toString", "string", "function(){return this.outerHTML;}")
el("dragEnter", "void", 'function(){this.dispatchEvent(new DragEvent("dragenter",{bubbles:true}));}')
el("dragOver", "void", 'function(){this.dispatchEvent(new DragEvent("dragover",{bubbles:true}));}')
el("drop", "void", 'function(){this.dispatchEvent(new DragEvent("drop",{bubbles:true}));}')
el("drag", "void", 'function(){this.dispatchEvent(new DragEvent("dragstart",{bubbles:true}));'
   'this.dispatchEvent(new DragEvent("dragend",{bubbles:true}));}')
el("dragAndDrop", "void",
   'function(sel){const t=document.querySelector(sel);if(!t)return;'
   'this.dispatchEvent(new DragEvent("dragstart",{bubbles:true}));'
   't.dispatchEvent(new DragEvent("drop",{bubbles:true}));'
   'this.dispatchEvent(new DragEvent("dragend",{bubbles:true}));}',
   [{"name": "target", "type": "String"}])
el("touchStart", "void", 'function(){this.dispatchEvent(new Event("touchstart",{bubbles:true}));}')
el("touchEnd", "void", 'function(){this.dispatchEvent(new Event("touchend",{bubbles:true}));}')
el("touchMove", "void", 'function(){this.dispatchEvent(new Event("touchmove",{bubbles:true}));}')
el(
    "select_option",
    "void",
    "function(valuesRaw){const w=JSON.parse(valuesRaw).map(String);"
    "for(const o of this.options){o.selected=w.includes(o.value)||w.includes(o.text);}"
    "this.dispatchEvent(new Event('change',{bubbles:true}));}",
    [{"name": "values_json", "type": "String"}],
)
el(
    "select",
    "void",
    "function(valuesRaw){const w=JSON.parse(valuesRaw).map(String);"
    "for(const o of this.options){o.selected=w.includes(o.value)||w.includes(o.text);}"
    "this.dispatchEvent(new Event('change',{bubbles:true}));}",
    [{"name": "values_json", "type": "String"}],
)
el(
    "press_sequentially",
    "void",
    "function(text){this.focus();this.value=(this.value||\"\")+text;"
    'this.dispatchEvent(new Event("input",{bubbles:true}));}',
    [{"name": "text", "type": "String"}],
)

# --- Frame (page-scoped) ---------------------------------------------------
pgo("goto", "page_goto", [{"name": "url", "type": "String"}], None)
pgo("content", "page_content", None, None)
pgo("title", "page_title", None, None)
pgo("addScriptTag", "page_add_script", [{"name": "content", "type": "String"}], None)
pgo("addStyleTag", "page_add_style_tag", [{"name": "content", "type": "String"}], None)
pgo("setFrameContent", "page_set_content", [{"name": "html", "type": "String"}], None)
pgo("waitForNavigation", "page_wait_for_navigation", None, None)
pgo("waitForSelector", "page_wait_for_selector", [{"name": "selector", "type": "String"}],
    "@ELEMENT@")
pgo("waitForFunction", "page_wait_for_function", [{"name": "expression", "type": "String"}], None)
pgo("click", "page_click", [{"name": "selector", "type": "String"}], None)
pgo("hover", "page_hover", [{"name": "selector", "type": "String"}], None)
pgo("focus", "page_focus", [{"name": "selector", "type": "String"}], None)
pgo("type", "page_type",
    [{"name": "selector", "type": "String"}, {"name": "value", "type": "String"}], None)

# --- Selenium extras -------------------------------------------------------
drv("find_elements", "selenium_find_all",
    [{"name": "by", "type": "String"}, {"name": "value", "type": "Option<String>"}],
    "@ELEMENT@", many=True)
drv("active_element", "selenium_active_element", None, "@ELEMENT@")
drv("print_page", "selenium_print_page", None, None)
drv("get_window_size", "selenium_window_size", None, None)
drv("set_window_size", "selenium_set_window_size",
    [{"name": "width", "type": "i64"}, {"name": "height", "type": "i64"}], None)
drv("get_window_position", "selenium_window_position", None, None)
drv("set_window_position", "selenium_set_window_position",
    [{"name": "x", "type": "i64"}, {"name": "y", "type": "i64"}], None)
drv("get_window_rect", "selenium_window_rect", None, None)
drv("set_window_rect", "selenium_set_window_rect",
    [{"name": "x", "type": "i64"}, {"name": "y", "type": "i64"},
     {"name": "width", "type": "i64"}, {"name": "height", "type": "i64"}], None)
drv("maximize_window", "selenium_maximize_window", None, None)
drv("minimize_window", "selenium_minimize_window", None, None)
drv("fullscreen_window", "selenium_fullscreen_window", None, None)
drv("implicitly_wait", "selenium_implicitly_wait", [{"name": "seconds", "type": "f64"}], None)
drv("set_page_load_timeout", "selenium_set_page_load_timeout",
    [{"name": "seconds", "type": "f64"}], None)
drv("set_script_timeout", "selenium_set_script_timeout",
    [{"name": "seconds", "type": "f64"}], None)

# --- Events / expectations -------------------------------------------------
pgo("on", "page_on", [{"name": "event_name", "type": "String"}], None)
pgo("once", "page_once", [{"name": "event_name", "type": "String"}], None)
pgo("add_listener", "page_on", [{"name": "event_name", "type": "String"}], None)
pgo("remove_listener", "page_remove_listener", [{"name": "event_name", "type": "String"}], None)
pgo("off", "page_remove_listener", [{"name": "event_name", "type": "String"}], None)
pgo("remove_all_listeners", "page_remove_all_listeners", None, None)
pgo("event_names", "page_event_names", None, None)
pgo("listeners", "page_event_names", None, None)
pgo("listens_to", "page_listens_to", [{"name": "event_name", "type": "String"}], None)
pgo("wait_for_event", "page_wait_for_event", [{"name": "event_name", "type": "String"}], None)
pgo("expect_event", "page_wait_for_event", [{"name": "event_name", "type": "String"}], None)
pgo("expect_download", "page_expect_download", None, None)
pgo("expect_popup", "page_expect_popup", None, None)
pgo("expect_request", "page_expect_request", None, None)
pgo("expect_response", "page_expect_response", None, None)
pgo("expect_console_message", "page_expect_console_message", None, None)
pgo("expect_navigation", "page_expect_navigation", None, None)
pgo("expect_file_chooser", "page_expect_file_chooser", None, None)
pgo("expect_request_finished", "page_expect_request_finished", None, None)
pgo("expect_websocket", "page_expect_websocket", None, None)
pgo("expect_worker", "page_expect_worker", None, None)
pgo("waitForRequest", "page_wait_for_request", None, None)
pgo("waitForResponse", "page_wait_for_response", None, None)
pgo("waitForFileChooser", "page_wait_for_file_chooser", None, None)
pgo("waitForTarget", "page_wait_for_target", None, None)

# --- Browser --------------------------------------------------------------
brw("contexts", "browser_contexts")
brw("browserContexts", "browser_contexts")
brw("pages", "browser_pages")
brw("targets", "browser_targets")
brw("userAgent", "browser_user_agent")
brw("isConnected", "browser_is_connected")
brw("connected", "browser_is_connected")
brw("is_connected", "browser_is_connected")
brw("wsEndpoint", "browser_ws_endpoint")
brw("newIncognitoBrowserContext", "browser_new_context_id")
brw("createIncognitoBrowserContext", "browser_new_context_id")
brw("createIncogniteBrowserContext", "browser_new_context_id")
brw("startTracing", "browser_start_tracing")
brw("start_tracing", "browser_start_tracing")
brw("stopTracing", "browser_stop_tracing")
brw("stop_tracing", "browser_stop_tracing")
brw("on", "browser_on", [{"name": "event_name", "type": "String"}])
brw("once", "browser_once", [{"name": "event_name", "type": "String"}])
brw("add_listener", "browser_on", [{"name": "event_name", "type": "String"}])
brw("remove_listener", "browser_remove_listener", [{"name": "event_name", "type": "String"}])
brw("off", "browser_remove_listener", [{"name": "event_name", "type": "String"}])
brw("remove_all_listeners", "browser_remove_all_listeners")
brw("event_names", "browser_event_names")
brw("listeners", "browser_event_names")
brw("listens_to", "browser_listens_to", [{"name": "event_name", "type": "String"}])
brw("wait_for_event", "browser_wait_for_event", [{"name": "event_name", "type": "String"}])

# --- BrowserContext -------------------------------------------------------
ctx("pages", "context_pages")
ctx("cookies", "context_cookies")
ctx("add_cookies", "context_add_cookies", [{"name": "cookies_json", "type": "String"}])
ctx("clear_cookies", "context_clear_cookies")
ctx("set_extra_http_headers", "context_set_extra_http_headers",
    [{"name": "headers_json", "type": "String"}])
ctx("grant_permissions", "context_grant_permissions",
    [{"name": "origin", "type": "String"}, {"name": "permissions_json", "type": "String"}])
ctx("overridePermissions", "context_grant_permissions",
    [{"name": "origin", "type": "String"}, {"name": "permissions_json", "type": "String"}])

# --- Network interception -------------------------------------------------
pgo("route", "page_route",
    [{"name": "pattern", "type": "String"},
     {"name": "action", "type": "String", "default": '"continue"'}], None)
pgo("unroute", "page_unroute", [{"name": "pattern", "type": "String"}], None)
pgo("unroute_all", "page_unroute_all", None, None)
pgo("route_abort", "page_route_abort", [{"name": "pattern", "type": "String"}], None)
pgo("route_fulfill", "page_route_fulfill",
    [{"name": "pattern", "type": "String"}, {"name": "body", "type": "String"},
     {"name": "content_type", "type": "Option<String>"}], None)
pgo("requests", "page_requests", None, None)
pgo("request", "page_request", None, None)
pgo("setRequestInterception", "page_set_request_interception",
    [{"name": "enabled", "type": "bool"}], None)
pgo("setDragInterception", "page_set_drag_interception",
    [{"name": "enabled", "type": "bool"}], None)
pgo("isDragInterceptionEnabled", "page_is_drag_interception_enabled", None, None)
pgo("authenticate", "page_authenticate",
    [{"name": "username", "type": "String"}, {"name": "password", "type": "String"}], None)

# --- Browser cookies / capabilities ---------------------------------------
brw("cookies", "browser_cookies")
brw("setCookie", "browser_set_cookie", [{"name": "cookie_json", "type": "String"}])
brw("deleteCookie", "browser_delete_cookie", [{"name": "name", "type": "String"}])
brw("deleteMatchingCookies", "browser_delete_cookie", [{"name": "name", "type": "String"}])

# --- Page coverage / screencast / options ---------------------------------
pgo("waitForXPath", "page_wait_for_xpath", [{"name": "xpath", "type": "String"}], "@ELEMENT@")
pgo("injectFile", "page_inject_file", [{"name": "path", "type": "String"}], None)
pgo("removeScriptToEvaluateOnNewDocument", "page_remove_script",
    [{"name": "identifier", "type": "String"}], None)
pgo("emulateMediaFeatures", "page_set_emulated_media_features",
    [{"name": "features_json", "type": "String"}], None)
pgo("emulateIdleState", "page_emulate_idle_state",
    [{"name": "is_user_active", "type": "bool"},
     {"name": "is_screen_unlocked", "type": "bool"}], None)
pgo("setDefaultTimeout", "page_set_default_timeout",
    [{"name": "milliseconds", "type": "f64"}], None)
pgo("setDefaultNavigationTimeout", "page_set_default_timeout",
    [{"name": "milliseconds", "type": "f64"}], None)
pgo("getDefaultTimeout", "page_get_default_timeout", None, None)
pgo("getDefaultNavigationTimeout", "page_get_default_timeout", None, None)
pgo("coverage", "page_new_coverage", None, "Coverage")

# --- BrowserContext storage / offline / geolocation -----------------------
ctx("set_offline", "context_set_offline", [{"name": "offline", "type": "bool"}])
ctx("set_geolocation", "context_set_geolocation",
    [{"name": "latitude", "type": "f64"}, {"name": "longitude", "type": "f64"},
     {"name": "accuracy", "type": "f64"}])
ctx("storage_state", "context_storage_state")
ctx("set_storage_state", "context_set_storage_state", [{"name": "state_json", "type": "String"}])
ctx("set_default_timeout", "context_set_default_timeout",
    [{"name": "milliseconds", "type": "f64"}])
ctx("set_default_navigation_timeout", "context_set_default_timeout",
    [{"name": "milliseconds", "type": "f64"}])
ctx("clear_permissions", "context_clear_permissions")

# --- Selenium cookies / windows / capabilities ----------------------------
drv("get_cookie", "selenium_get_cookie", [{"name": "name", "type": "String"}])
drv("window_handles", "selenium_window_handles")
drv("current_window_handle", "selenium_current_window_handle")
drv("execute", "selenium_execute_script", [{"name": "script", "type": "String"}])
drv("capabilities", "selenium_capabilities")
drv("desired_capabilities", "selenium_capabilities")

# --- Context routes / events / targets ------------------------------------
ctx("targets", "context_targets")
ctx("route", "context_route",
    [{"name": "pattern", "type": "String"},
     {"name": "action", "type": "String", "default": '"continue"'}])
ctx("route_abort", "context_route_abort", [{"name": "pattern", "type": "String"}])
ctx("unroute", "context_unroute", [{"name": "pattern", "type": "String"}])
ctx("unroute_all", "context_unroute_all")
ctx("setCookie", "context_set_cookie", [{"name": "cookie_json", "type": "String"}])
ctx("deleteCookie", "context_delete_cookie", [{"name": "name", "type": "String"}])
ctx("on", "context_on", [{"name": "event_name", "type": "String"}])
ctx("once", "context_once", [{"name": "event_name", "type": "String"}])
ctx("remove_listener", "context_remove_listener", [{"name": "event_name", "type": "String"}])
ctx("remove_all_listeners", "context_remove_all_listeners")
ctx("event_names", "context_event_names")
ctx("listens_to", "context_listens_to", [{"name": "event_name", "type": "String"}])
ctx("wait_for_event", "context_wait_for_event", [{"name": "event_name", "type": "String"}])
ctx("expect_event", "context_wait_for_event", [{"name": "event_name", "type": "String"}])
ctx("setDownloadBehavior", "context_set_download_behavior", [{"name": "path", "type": "String"}])
ctx("setPermission", "context_grant_permissions",
    [{"name": "origin", "type": "String"}, {"name": "permissions_json", "type": "String"}])

# --- Element-scoped evals / browser bind ----------------------------------
elo("$eval", "element_eval_on_selector",
    [{"name": "selector", "type": "String"}, {"name": "expression", "type": "String"}])
elo("querySelectorEval", "element_eval_on_selector",
    [{"name": "selector", "type": "String"}, {"name": "expression", "type": "String"}])
elo("$$eval", "element_eval_on_selector_all",
    [{"name": "selector", "type": "String"}, {"name": "expression", "type": "String"}])
elo("querySelectorAllEval", "element_eval_on_selector_all",
    [{"name": "selector", "type": "String"}, {"name": "expression", "type": "String"}])
brw("bind", "browser_on", [{"name": "event_name", "type": "String"}])
brw("unbind", "browser_remove_listener", [{"name": "event_name", "type": "String"}])
brw("waitForTarget", "browser_wait_for_target")
brw("setPermission", "browser_grant_permissions",
    [{"name": "origin", "type": "String"}, {"name": "permissions_json", "type": "String"}])

# --- Element locators / lists ---------------------------------------------
elo("get_by_text", "element_get_by_text", [{"name": "text", "type": "String"}], "@ELEMENT@")
elo("get_by_role", "element_get_by_role", [{"name": "role", "type": "String"}], "@ELEMENT@")
elo("get_by_label", "element_get_by_label", [{"name": "label", "type": "String"}], "@ELEMENT@")
elo("get_by_placeholder", "element_get_by_placeholder", [{"name": "text", "type": "String"}], "@ELEMENT@")
elo("get_by_alt_text", "element_get_by_alt_text", [{"name": "text", "type": "String"}], "@ELEMENT@")
elo("get_by_title", "element_get_by_title", [{"name": "text", "type": "String"}], "@ELEMENT@")
elo("get_by_test_id", "element_get_by_test_id", [{"name": "test_id", "type": "String"}], "@ELEMENT@")
elo("all", "element_all", None, "@ELEMENT@", many=True)
elo("element_handle", "element_self", None, "@ELEMENT@")
elo("element_handles", "element_all", None, "@ELEMENT@", many=True)
elo("set_input_files", "element_set_input_files", [{"name": "files_json", "type": "String"}], None)
elo("J", "element_find", [{"name": "selector", "type": "String"}], "@ELEMENT@")
elo("JJ", "element_find_all", [{"name": "selector", "type": "String"}], "@ELEMENT@", many=True)
elo("Jeval", "element_eval_on_selector",
    [{"name": "selector", "type": "String"}, {"name": "expression", "type": "String"}])
elo("JJeval", "element_eval_on_selector_all",
    [{"name": "selector", "type": "String"}, {"name": "expression", "type": "String"}])
elo("$x", "element_query_selector_xpath", [{"name": "xpath", "type": "String"}], "@ELEMENT@")
elo("xpath", "element_query_selector_xpath", [{"name": "xpath", "type": "String"}], "@ELEMENT@")
pgo("name", "page_frame_name", None, None)
pgo("route_from_har", "page_route_from_har", [{"name": "path", "type": "String"}], None)
ctx("route_from_har", "page_route_from_har", [{"name": "path", "type": "String"}])
pgc("setBypassServiceWorker", "void", "Network.setBypassServiceWorker",
    [{"name": "bypass", "type": "bool"}], {"bypass": "${bypass}"})

# --- Page misc ------------------------------------------------------------
pgo("is_closed", "page_is_closed", None, None)
pgo("isClosed", "page_is_closed", None, None)
pgo("set_default_timeout", "page_set_default_timeout", [{"name": "milliseconds", "type": "f64"}], None)
pgo("set_default_navigation_timeout", "page_set_default_timeout",
    [{"name": "milliseconds", "type": "f64"}], None)
pg("drag_and_drop", "void",
   'function(src,dst){const s=document.querySelector(src),t=document.querySelector(dst);'
   'if(!s||!t)return;const dt=new DataTransfer();'
   's.dispatchEvent(new DragEvent("dragstart",{bubbles:true,dataTransfer:dt}));'
   't.dispatchEvent(new DragEvent("dragenter",{bubbles:true,dataTransfer:dt}));'
   't.dispatchEvent(new DragEvent("dragover",{bubbles:true,dataTransfer:dt}));'
   't.dispatchEvent(new DragEvent("drop",{bubbles:true,dataTransfer:dt}));'
   's.dispatchEvent(new DragEvent("dragend",{bubbles:true,dataTransfer:dt}));}',
   [{"name": "source", "type": "String"}, {"name": "target", "type": "String"}])

# --- Context misc ---------------------------------------------------------
ctx("is_closed", "context_is_closed")
ctx("expect_page", "context_expect_page")
ctx("expect_console_message", "context_expect_console_message")
ctx("background_pages", "context_background_pages")
ctx("waitForTarget", "context_wait_for_target")
ctx("clearPermissionOverrides", "context_clear_permissions")
ctx("deleteMatchingCookies", "context_delete_cookie", [{"name": "name", "type": "String"}])

# --- Driver misc ----------------------------------------------------------
drv("set_page_load_strategy", "selenium_set_page_load_strategy",
    [{"name": "strategy", "type": "String"}])

# --- Input / sub-API objects ----------------------------------------------
pgo("keyboard", "page_new_keyboard", None, "Keyboard")
pgo("mouse", "page_new_mouse", None, "Mouse")
pgo("touchscreen", "page_new_touchscreen", None, "Touchscreen")
drv("switch_to", "driver_new_switch_to", None, "SwitchTo")
drv("timeouts", "driver_new_timeouts", None, "Timeouts")
drv("dialog", "driver_new_dialog", None, "Dialog")
pgo("tracing", "page_new_tracing", None, "Tracing")
pgo("screencast", "page_new_screencast", None, "Screencast")
pgo("createPDFStream", "page_create_pdf_stream", None, None)
elo("getProperties", "element_get_properties", None, None)
elo("dispose", "element_dispose", None, None)
elo("waitForSelector", "element_wait_for_selector", [{"name": "selector", "type": "String"}],
    "@ELEMENT@")

# --- Frames ---------------------------------------------------------------
pgo("frames", "page_frames", None, None)
pgo("mainFrame", "page_main_frame", None, None)
pgo("main_frame", "page_main_frame", None, None)
pgo("childFrames", "page_frames", None, None)
pgo("parentFrame", "page_main_frame", None, None)
pgo("frame", "page_frame", [{"name": "frame_id", "type": "String"}], None)
pgo("addPreloadScript", "page_add_script", [{"name": "content", "type": "String"}], None)

# --- CDP sessions ---------------------------------------------------------
pgo("createCDPSession", "page_new_cdp_session", None, "CDPSession")
pgo("new_cdp_session", "page_new_cdp_session", None, "CDPSession")
ctx("new_cdp_session", "context_new_cdp_session", None, "CDPSession")
ctx("newCDPSession", "context_new_cdp_session", None, "CDPSession")


def lookup(kind, name):
    """Find an implementation by exact name, else by camelCase-normalised name."""
    entry = IMPLS.get((kind, name))
    if entry is not None:
        return entry
    snake = re.sub(r"(?<!^)(?=[A-Z])", "_", name).lower()
    return IMPLS.get((kind, snake))


def apply(profile_name, profile, dry_run, prune):
    kinds = KIND.get(profile_name, {})
    changed = 0
    for class_name, spec in profile.get("classes", {}).items():
        kind = kinds.get(class_name)
        if kind is None:
            continue
        for method in spec.get("methods", []):
            if not method.get("unsupported"):
                continue
            entry = lookup(kind, method["name"])
            if entry is None:
                continue
            method.pop("unsupported", None)
            method.pop("property", None)
            method.pop("op", None)
            if "params" in entry:
                method["params"] = entry["params"]
            else:
                method.pop("params", None)
            method.pop("list", None)
            if "impl" in entry:
                method["impl"] = entry["impl"]
                method.pop("returns", None)
            else:
                method["op"] = entry["op"]
                returns = entry.get("returns")
                if returns == "@ELEMENT@":
                    returns = ELEMENT_CLASS.get(profile_name)
                method["returns"] = returns
                if entry.get("list"):
                    method["list"] = True
            changed += 1
    injected = 0
    for class_name, spec in EXTRA_CLASSES.get(profile_name, {}).items():
        classes = profile.setdefault("classes", {})
        if class_name not in classes:
            classes[class_name] = spec
            injected += 1

    pruned = 0
    if prune:
        classes = profile.get("classes", {})
        for class_name in list(classes.keys()):
            spec = classes[class_name]
            before = len(spec.get("methods", []))
            spec["methods"] = [
                method
                for method in spec.get("methods", [])
                if not method.get("unsupported")
            ]
            pruned += before - len(spec["methods"])
            if not spec["methods"]:
                del classes[class_name]

    if (changed or injected or pruned) and not dry_run:
        write_profile(profile_name, profile)
    return changed, injected, pruned


def main():
    parser = argparse.ArgumentParser(description="Backfill adapter profile implementations.")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument(
        "--prune",
        action="store_true",
        help="drop methods that have no faithful implementation (out of scope)",
    )
    args = parser.parse_args()

    totals = [0, 0, 0]
    for profile in load_profiles():
        changed, injected, pruned = apply(profile["name"], profile, args.dry_run, args.prune)
        totals[0] += changed
        totals[1] += injected
        totals[2] += pruned
        log(
            "BACKFILL",
            f"{profile['name']}: {changed} implemented, {injected} classes injected, {pruned} pruned",
        )

    log(
        "BACKFILL",
        f"total: {totals[0]} implemented, {totals[1]} classes, {totals[2]} pruned "
        f"({'dry-run' if args.dry_run else 'written'})",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
