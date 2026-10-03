__version__ = "1.0.8"
from .xcelerate import Browser, BrowserConfig, Page, Element, XcelerateError

# API-style adapters (Selenium / Playwright / Puppeteer profiles).
try:
    from . import adapters
    from .adapters import use, playwright, puppeteer, selenium
except Exception:
    pass

__all__ = ["Browser", "BrowserConfig", "Page", "Element", "XcelerateError", "adapters", "use", "playwright", "puppeteer", "selenium"]
