"""Synchronous Selenium compatibility layer for xcelerate.

Goal: run an **unmodified** Selenium script on top of xcelerate. You do not edit
the script; you only point xcelerate at it:

    python -m xcelerate.compat path/to/selenium_script.py

or, from Python:

    from xcelerate.compat import install_as_selenium
    install_as_selenium()          # register the shim as ``selenium``
    runpy.run_path("script.py")    # run the untouched script

Selenium's API is synchronous while :mod:`xcelerate.selenium` is async, so this
module owns a background event loop and blocks the calling thread on the
corresponding coroutine. It re-implements the pieces a normal Selenium script
touches (locators, elements, waits, dropdowns, hover) on top of xcelerate's CDP
engine, and fills xcelerate's gaps where it is stricter than Selenium (e.g.
XPath, blocking navigation, ``find_elements``).
"""

from __future__ import annotations

import asyncio
import json
import os
import runpy
import sys
import threading
import time
import types
import uuid
from typing import Any, Callable, Optional

from . import selenium as _selenium
from .adapters import _runtime
from .xcelerate import XcelerateError


# ---------------------------------------------------------------------------
# Locator strategies
# ---------------------------------------------------------------------------

class By:
    """Mirror of ``selenium.webdriver.common.by.By``."""

    ID = "id"
    NAME = "name"
    XPATH = "xpath"
    LINK_TEXT = "link text"
    PARTIAL_LINK_TEXT = "partial link text"
    TAG_NAME = "tag name"
    CLASS_NAME = "class name"
    CSS_SELECTOR = "css selector"


class Keys:
    """Common ``selenium.webdriver.common.keys.Keys`` constants."""

    NULL = "\ue000"
    BACKSPACE = "\ue003"
    TAB = "\ue004"
    RETURN = "\ue006"
    ENTER = "\ue007"
    SHIFT = "\ue008"
    CONTROL = "\ue009"
    ALT = "\ue00a"
    ESCAPE = "\ue00c"
    SPACE = "\ue00d"
    PAGE_UP = "\ue00e"
    PAGE_DOWN = "\ue00f"
    END = "\ue010"
    HOME = "\ue011"
    ARROW_LEFT = "\ue012"
    ARROW_UP = "\ue013"
    ARROW_RIGHT = "\ue014"
    ARROW_DOWN = "\ue015"
    INSERT = "\ue016"
    DELETE = "\ue017"


_KEYS_TO_PRESS = {
    Keys.BACKSPACE: "Backspace",
    Keys.TAB: "Tab",
    Keys.RETURN: "Enter",
    Keys.ENTER: "Enter",
    Keys.SHIFT: "Shift",
    Keys.CONTROL: "Control",
    Keys.ALT: "Alt",
    Keys.ESCAPE: "Escape",
    Keys.SPACE: " ",
    Keys.PAGE_UP: "PageUp",
    Keys.PAGE_DOWN: "PageDown",
    Keys.END: "End",
    Keys.HOME: "Home",
    Keys.ARROW_LEFT: "ArrowLeft",
    Keys.ARROW_UP: "ArrowUp",
    Keys.ARROW_RIGHT: "ArrowRight",
    Keys.ARROW_DOWN: "ArrowDown",
    Keys.INSERT: "Insert",
    Keys.DELETE: "Delete",
}


_BY_ALIASES = {
    "id": "id",
    "name": "name",
    "class name": "class name",
    "class_name": "class name",
    "tag name": "tag name",
    "tag_name": "tag name",
    "link text": "link text",
    "link_text": "link text",
    "partial link text": "partial link text",
    "partial_link_text": "partial link text",
    "css selector": "css selector",
    "css": "css selector",
    "css_selector": "css selector",
    "xpath": "xpath",
}


def _unpack(by: Any, value: Optional[str]) -> tuple[Any, Optional[str]]:
    if value is None and isinstance(by, (tuple, list)) and len(by) == 2:
        return by[0], by[1]
    return by, value


def _strategy(by: Any) -> str:
    name = getattr(by, "value", by)
    return _BY_ALIASES.get(str(name).strip().lower(), "css selector")


def _css_for(strategy: str, value: Optional[str]) -> str:
    if strategy == "id":
        return f"#{value}"
    if strategy == "name":
        return f'[name="{value}"]'
    if strategy == "class name":
        return "." + (value or "").strip().replace(" ", ".")
    if strategy == "tag name":
        return value or ""
    return value or ""  # css selector


def _xpath_literal(value: str) -> str:
    if "'" not in value:
        return f"'{value}'"
    return f'"{value}"'


async def _locate(page: Any, strategy: str, value: Optional[str]) -> Any:
    if strategy == "xpath":
        return await page.query_selector_xpath(value)
    if strategy == "link text":
        xpath = f"//a[normalize-space(text())={_xpath_literal(value)}]"
        return await page.query_selector_xpath(xpath)
    if strategy == "partial link text":
        xpath = f"//a[contains(normalize-space(text()),{_xpath_literal(value)})]"
        return await page.query_selector_xpath(xpath)
    return await page.find_element(_css_for(strategy, value))


async def _find_all_xpath(page: Any, xpath: str) -> list[Any]:
    """Return every node matching ``xpath``.

    xcelerate exposes only a single-node XPath query, so we tag each match in
    the DOM with a unique attribute and collect them via the CSS engine.
    """
    token = "xc" + uuid.uuid4().hex
    script = (
        "(function(){var r=document.evaluate(%s,document,null,"
        "XPathResult.ORDERED_NODE_SNAPSHOT_TYPE,null);var n=r.snapshotLength;"
        "for(var i=0;i<n;i++){r.snapshotItem(i).setAttribute('data-xc-xpath','%s_'+i);}"
        "return n;})()" % (json.dumps(xpath), token)
    )
    await page.evaluate_json(script)
    return await page.query_selector_all('[data-xc-xpath^="%s_"]' % token)


# ---------------------------------------------------------------------------
# Exceptions (registered as ``selenium.common.exceptions``)
# ---------------------------------------------------------------------------

class WebDriverException(Exception):
    pass


class NoSuchElementException(WebDriverException):
    pass


class TimeoutException(WebDriverException):
    pass


class InvalidSelectorException(WebDriverException):
    pass


# ---------------------------------------------------------------------------
# Sync <-> async bridge
# ---------------------------------------------------------------------------

class _LoopThread:
    """A daemon thread running an asyncio loop for sync->async bridging."""

    def __init__(self) -> None:
        self._loop = asyncio.new_event_loop()
        self._thread = threading.Thread(
            target=self._run, name="xcelerate-selenium-compat", daemon=True
        )
        self._thread.start()

    def _run(self) -> None:
        asyncio.set_event_loop(self._loop)
        self._loop.run_forever()

    def run(self, coro: Any) -> Any:
        return asyncio.run_coroutine_threadsafe(coro, self._loop).result()


_LOOP: Optional[_LoopThread] = None


def _loop() -> _LoopThread:
    global _LOOP
    if _LOOP is None:
        _LOOP = _LoopThread()
    return _LOOP


def _await(coro: Any) -> Any:
    """Run a coroutine, translating xcelerate's NotFound into Selenium's error."""
    try:
        return _loop().run(coro)
    except XcelerateError.NotFound as exc:
        raise NoSuchElementException(str(exc)) from exc


# ---------------------------------------------------------------------------
# WebElement
# ---------------------------------------------------------------------------

class WebElement:
    """Synchronous view over an xcelerate element."""

    def __init__(self, element: Any, driver: "WebDriver") -> None:
        self._element = element
        self._driver = driver

    # -- interaction --------------------------------------------------------
    def click(self) -> None:
        _await(_runtime.element_click(self._element))

    def clear(self) -> None:
        _await(_runtime.element_clear(self._element))

    def send_keys(self, *values: Any) -> None:
        for value in values:
            if isinstance(value, str) and value in _KEYS_TO_PRESS:
                _await(self._element.press(_KEYS_TO_PRESS[value]))
            else:
                _await(_runtime.element_send_keys(self._element, str(value)))

    def hover(self) -> None:
        _await(_runtime.element_hover(self._element))

    def submit(self) -> None:
        _await(_runtime.call_js(
            self._element, "element",
            "function(){if(this.form){this.form.submit();}else if(this.tagName==='FORM'){this.submit();}}",
            "void", [],
        ))

    # -- lookup -------------------------------------------------------------
    def find_element(self, by: Any, value: Optional[str] = None) -> "WebElement":
        by, value = _unpack(by, value)
        strategy = _strategy(by)
        if strategy == "xpath":
            element = _await(self._driver._page().query_selector_xpath(value))
        else:
            element = _await(self._element.query_selector(_css_for(strategy, value)))
        return WebElement(element, self._driver)

    def find_elements(self, by: Any, value: Optional[str] = None) -> list["WebElement"]:
        by, value = _unpack(by, value)
        strategy = _strategy(by)
        if strategy == "xpath":
            raw = _await(_find_all_xpath(self._driver._page(), value))
        else:
            raw = _await(self._element.query_selector_all(_css_for(strategy, value)))
        return [WebElement(item, self._driver) for item in raw]

    # -- properties ---------------------------------------------------------
    @property
    def text(self) -> str:
        return _await(_runtime.element_text(self._element))

    @property
    def tag_name(self) -> str:
        return _await(_runtime.call_js(
            self._element, "element",
            "function(){return this.tagName?this.tagName.toLowerCase():'unknown';}",
            "string", [],
        ))

    def get_attribute(self, name: str) -> Optional[str]:
        return _await(_runtime.element_attribute(self._element, name))

    def get_property(self, name: str) -> Any:
        return _await(_runtime.call_js(
            self._element, "element",
            "function(n){const v=this[n];return v==null?'':String(v);}",
            "string", [name],
        ))

    def value_of_css_property(self, name: str) -> str:
        return _await(_runtime.call_js(
            self._element, "element",
            "function(p){return getComputedStyle(this).getPropertyValue(p);}",
            "string", [name],
        ))

    def is_displayed(self) -> bool:
        return _await(_runtime.call_js(
            self._element, "element",
            "function(){const s=getComputedStyle(this);const r=this.getBoundingClientRect();"
            "return !!(r.width||r.height)&&s.visibility!=='hidden'&&s.display!=='none';}",
            "bool", [],
        ))

    def is_enabled(self) -> bool:
        return _await(_runtime.call_js(
            self._element, "element",
            "function(){return !(this.disabled===true||this.hasAttribute('disabled'));}",
            "bool", [],
        ))

    def is_selected(self) -> bool:
        return _await(_runtime.call_js(
            self._element, "element",
            "function(){return this.selected===true||this.checked===true;}",
            "bool", [],
        ))

    def screenshot(self, filename: str) -> bool:
        return _write_element_screenshot(self, filename)

    def __repr__(self) -> str:
        return f"<WebElement tag={getattr(self, '_element', None)!r}>"


def _write_element_screenshot(element: WebElement, filename: str) -> bool:
    data = _await(_runtime.element_screenshot(element._element))
    with open(filename, "wb") as handle:
        handle.write(data)
    return True


# ---------------------------------------------------------------------------
# Select
# ---------------------------------------------------------------------------

class Select:
    """``selenium.webdriver.support.ui.Select`` for native ``<select>`` elements."""

    def __init__(self, webelement: WebElement) -> None:
        self._element = webelement

    @property
    def options(self) -> list[WebElement]:
        return self._element.find_elements("tag name", "option")

    @property
    def all_selected_options(self) -> list[WebElement]:
        return [opt for opt in self.options if opt.is_selected()]

    @property
    def first_selected_option(self) -> WebElement:
        selected = self.all_selected_options
        if not selected:
            raise NoSuchElementException("no option is selected")
        return selected[0]

    def select_by_index(self, index: int) -> None:
        self._select(self.options[index])

    def select_by_value(self, value: str) -> None:
        for option in self.options:
            if option.get_attribute("value") == value:
                self._select(option)
                return
        raise NoSuchElementException(f"no option with value {value!r}")

    def select_by_visible_text(self, text: str) -> None:
        for option in self.options:
            if (option.text or "").strip() == text:
                self._select(option)
                return
        raise NoSuchElementException(f"no option with visible text {text!r}")

    def deselect_all(self) -> None:
        # For a single-select, selecting the first option clears the rest.
        options = self.options
        if options:
            self._select(options[0])

    def _select(self, option: WebElement) -> None:
        _await(_runtime.call_js(
            option._element, "element",
            "function(){this.selected=true;"
            "this.dispatchEvent(new Event('input',{bubbles:true}));"
            "this.dispatchEvent(new Event('change',{bubbles:true}));}",
            "void", [],
        ))


# ---------------------------------------------------------------------------
# ActionChains
# ---------------------------------------------------------------------------

class ActionChains:
    """Minimal ``selenium.webdriver.common.action_chains.ActionChains``."""

    def __init__(self, driver: "WebDriver") -> None:
        self._driver = driver
        self._queue: list[tuple[str, Optional[WebElement]]] = []

    def move_to_element(self, element: WebElement) -> "ActionChains":
        self._queue.append(("hover", element))
        return self

    def click(self, element: Optional[WebElement] = None) -> "ActionChains":
        if element is not None:
            self._queue.append(("click", element))
        return self

    def perform(self) -> None:
        for kind, element in self._queue:
            if element is None:
                continue
            if kind == "hover":
                element.hover()
            elif kind == "click":
                element.click()
        self._queue.clear()


# ---------------------------------------------------------------------------
# Waits
# ---------------------------------------------------------------------------

class WebDriverWait:
    """Minimal ``selenium.webdriver.support.ui.WebDriverWait``."""

    def __init__(
        self,
        driver: "WebDriver",
        timeout: float,
        poll_frequency: float = 0.5,
        ignored_exceptions: Any = None,
    ) -> None:
        self._driver = driver
        self._timeout = timeout
        self._poll = poll_frequency
        if ignored_exceptions is None:
            ignored = [NoSuchElementException]
        elif isinstance(ignored_exceptions, (list, tuple)):
            ignored = list(ignored_exceptions)
        else:
            ignored = [ignored_exceptions]
        self._ignored = tuple(ignored)

    def until(self, method: Callable[["WebDriver"], Any], message: str = "") -> Any:
        return self._wait(method, message, want_truthy=True)

    def until_not(self, method: Callable[["WebDriver"], Any], message: str = "") -> Any:
        return self._wait(method, message, want_truthy=False)

    def _wait(self, method: Callable[["WebDriver"], Any], message: str, want_truthy: bool) -> Any:
        end = time.monotonic() + self._timeout
        last_error: Optional[BaseException] = None
        while True:
            try:
                value = method(self._driver)
            except self._ignored:
                value = None
            except Exception as exc:  # noqa: BLE001 - surface unexpected errors
                last_error = exc
                value = None
            else:
                if bool(value) is want_truthy:
                    return value
            if time.monotonic() >= end:
                detail = message or f"condition not satisfied within {self._timeout}s"
                if last_error is not None:
                    raise TimeoutException(f"{detail} (last error: {last_error})") from last_error
                raise TimeoutException(detail)
            time.sleep(self._poll)


def _locator_pair(locator: Any) -> tuple[Any, Any]:
    if isinstance(locator, (tuple, list)) and len(locator) == 2:
        return locator[0], locator[1]
    raise InvalidSelectorException(f"invalid locator: {locator!r}")


class expected_conditions:
    """Common ``selenium.webdriver.support.expected_conditions`` conditions."""

    @staticmethod
    def presence_of_element_located(locator: Any) -> Callable[["WebDriver"], Any]:
        by, value = _locator_pair(locator)

        def _condition(driver: "WebDriver") -> Any:
            elements = driver.find_elements(by, value)
            return elements[0] if elements else False

        return _condition

    @staticmethod
    def presence_of_all_elements_located(locator: Any) -> Callable[["WebDriver"], Any]:
        by, value = _locator_pair(locator)

        def _condition(driver: "WebDriver") -> Any:
            elements = driver.find_elements(by, value)
            return elements or False

        return _condition

    @staticmethod
    def visibility_of_element_located(locator: Any) -> Callable[["WebDriver"], Any]:
        by, value = _locator_pair(locator)

        def _condition(driver: "WebDriver") -> Any:
            elements = driver.find_elements(by, value)
            if elements and elements[0].is_displayed():
                return elements[0]
            return False

        return _condition

    @staticmethod
    def element_to_be_clickable(locator: Any) -> Callable[["WebDriver"], Any]:
        by, value = _locator_pair(locator)

        def _condition(driver: "WebDriver") -> Any:
            elements = driver.find_elements(by, value)
            if elements and elements[0].is_displayed() and elements[0].is_enabled():
                return elements[0]
            return False

        return _condition

    @staticmethod
    def title_contains(text: str) -> Callable[["WebDriver"], bool]:
        return lambda driver: text in driver.title

    @staticmethod
    def title_is(text: str) -> Callable[["WebDriver"], bool]:
        return lambda driver: driver.title == text

    @staticmethod
    def url_contains(text: str) -> Callable[["WebDriver"], bool]:
        return lambda driver: text in driver.current_url


# ---------------------------------------------------------------------------
# WebDriver
# ---------------------------------------------------------------------------

class WebDriver:
    """Synchronous driver mirroring the Selenium WebDriver API in use."""

    def __init__(self, config: Any = None, **_ignored: Any) -> None:
        self._driver = _loop().run(_selenium.launch(config))

    def _page(self) -> Any:
        return self._driver._wrapped.page

    # -- navigation ---------------------------------------------------------
    # Selenium's navigation commands block until the page has loaded. xcelerate's
    # navigate/reload/go_back return as soon as the request is committed, so the
    # compat layer waits for document.readyState to reach "complete" to preserve
    # the Selenium contract (otherwise `driver.title` is read mid-navigation).
    def get(self, url: str) -> None:
        self._navigate(self._driver.get(url))

    def refresh(self) -> None:
        self._navigate(self._driver.refresh())

    def back(self) -> None:
        self._navigate(self._driver.back())

    def forward(self) -> None:
        self._navigate(self._driver.forward())

    def _navigate(self, coro: Any, timeout: float = 30.0) -> None:
        _await(coro)
        self._wait_for_load(timeout=timeout)

    def _wait_for_load(self, timeout: float = 30.0, poll: float = 0.1) -> None:
        deadline = time.monotonic() + timeout
        while True:
            try:
                raw = _loop().run(self._driver.execute_script("document.readyState"))
                state = json.loads(raw)
            except Exception:
                state = None
            if state == "complete":
                return
            if time.monotonic() >= deadline:
                return
            time.sleep(poll)

    # -- properties (Selenium exposes these as attributes, not methods) -----
    @property
    def title(self) -> str:
        return _await(self._driver.title())

    @property
    def current_url(self) -> str:
        return _await(self._driver.current_url())

    @property
    def page_source(self) -> str:
        return _await(self._driver.page_source())

    # -- element lookup -----------------------------------------------------
    def find_element(self, by: Any, value: Optional[str] = None) -> WebElement:
        by, value = _unpack(by, value)
        strategy = _strategy(by)
        if strategy == "xpath":
            element = _await(self._page().query_selector_xpath(value))
        else:
            element = _await(self._page().find_element(_css_for(strategy, value)))
        return WebElement(element, self)

    def find_elements(self, by: Any, value: Optional[str] = None) -> list[WebElement]:
        by, value = _unpack(by, value)
        strategy = _strategy(by)
        if strategy == "xpath":
            raw = _await(_find_all_xpath(self._page(), value))
        else:
            raw = _await(self._page().query_selector_all(_css_for(strategy, value)))
        return [WebElement(item, self) for item in raw]

    # -- scripts ------------------------------------------------------------
    def execute_script(self, script: str, *args: Any) -> Any:
        expr = "(function(){%s}).apply(null,%s)" % (script, json.dumps(list(args)))
        raw = _await(self._driver.execute_script(expr))
        return json.loads(raw) if raw not in (None, "") else None

    execute_async_script = execute_script

    # -- window / misc ------------------------------------------------------
    def maximize_window(self) -> None:
        _await(self._driver.maximize_window())

    def set_window_size(self, width: int, height: int) -> None:
        _await(self._driver.set_window_size(width, height))

    def save_screenshot(self, filename: str) -> bool:
        _await(self._driver.save_screenshot(filename))
        return True

    def implicitly_wait(self, seconds: float) -> None:
        # xcelerate has no polling implicit wait; kept as a shape-compatible no-op.
        return None

    def set_page_load_timeout(self, seconds: float) -> None:
        _await(self._driver.set_page_load_timeout(seconds))

    # -- teardown -----------------------------------------------------------
    def quit(self) -> None:
        _await(self._driver.quit())

    def close(self) -> None:
        self.quit()


# ---------------------------------------------------------------------------
# Registration
# ---------------------------------------------------------------------------

def install_as_selenium() -> None:
    """Register this facade as the ``selenium`` package in ``sys.modules``.

    Idempotent. Must run before the target script imports ``selenium``; this is
    what lets an untouched Selenium script run on xcelerate.
    """
    if sys.modules.get("selenium") is not None and getattr(
        sys.modules["selenium"], "__version__", None
    ) == "xcelerate-compat":
        return

    def package(name: str) -> types.ModuleType:
        module = types.ModuleType(name)
        module.__path__ = []  # type: ignore[attr-defined]
        return module

    pkg = package("selenium")
    pkg.__version__ = "xcelerate-compat"

    webdriver = package("selenium.webdriver")
    webdriver.WebDriver = WebDriver
    webdriver.Chrome = WebDriver
    webdriver.Firefox = WebDriver
    webdriver.Remote = WebDriver

    common = package("selenium.webdriver.common")
    by_module = types.ModuleType("selenium.webdriver.common.by")
    by_module.By = By
    keys_module = types.ModuleType("selenium.webdriver.common.keys")
    keys_module.Keys = Keys
    action_chains = types.ModuleType("selenium.webdriver.common.action_chains")
    action_chains.ActionChains = ActionChains
    common.by = by_module
    common.keys = keys_module
    common.action_chains = action_chains

    exceptions = types.ModuleType("selenium.common.exceptions")
    exceptions.WebDriverException = WebDriverException
    exceptions.NoSuchElementException = NoSuchElementException
    exceptions.TimeoutException = TimeoutException
    exceptions.InvalidSelectorException = InvalidSelectorException
    common_pkg = package("selenium.common")
    common_pkg.exceptions = exceptions

    support = package("selenium.webdriver.support")
    ui = types.ModuleType("selenium.webdriver.support.ui")
    ui.WebDriverWait = WebDriverWait
    ui.Select = Select
    ec = types.ModuleType("selenium.webdriver.support.expected_conditions")
    ec.presence_of_element_located = expected_conditions.presence_of_element_located
    ec.presence_of_all_elements_located = expected_conditions.presence_of_all_elements_located
    ec.visibility_of_element_located = expected_conditions.visibility_of_element_located
    ec.element_to_be_clickable = expected_conditions.element_to_be_clickable
    ec.title_contains = expected_conditions.title_contains
    ec.title_is = expected_conditions.title_is
    ec.url_contains = expected_conditions.url_contains
    support.ui = ui
    support.expected_conditions = ec

    webdriver.common = common
    webdriver.support = support
    pkg.webdriver = webdriver
    pkg.common = common_pkg

    sys.modules.update(
        {
            "selenium": pkg,
            "selenium.webdriver": webdriver,
            "selenium.webdriver.common": common,
            "selenium.webdriver.common.by": by_module,
            "selenium.webdriver.common.keys": keys_module,
            "selenium.webdriver.common.action_chains": action_chains,
            "selenium.common": common_pkg,
            "selenium.common.exceptions": exceptions,
            "selenium.webdriver.support": support,
            "selenium.webdriver.support.ui": ui,
            "selenium.webdriver.support.expected_conditions": ec,
        }
    )


def run(script: str, *args: str) -> None:
    """Install the shim and execute ``script`` unmodified via ``runpy``.

    The script file is only read, never rewritten. Its directory is added to
    ``sys.path`` so sibling imports resolve exactly as with ``python script.py``.
    """
    install_as_selenium()
    script_path = os.path.abspath(script)
    script_dir = os.path.dirname(script_path)
    if script_dir and script_dir not in sys.path:
        sys.path.insert(0, script_dir)
    sys.argv = [script, *args]
    runpy.run_path(script_path, run_name="__main__")


def _main(argv: Optional[list[str]] = None) -> int:
    argv = list(sys.argv[1:] if argv is None else argv)
    if not argv:
        print("usage: python -m xcelerate.compat SCRIPT.py [args...]", file=sys.stderr)
        return 2
    run(argv[0], *argv[1:])
    return 0


if __name__ == "__main__":
    raise SystemExit(_main())
