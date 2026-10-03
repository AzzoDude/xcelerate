"""Generated API-style adapters - do not edit by hand.

Regenerate with: python scripts/generate_adapters.py
"""

from . import playwright
from . import puppeteer
from . import selenium

_MODULES = {
    "playwright": playwright,
    "puppeteer": puppeteer,
    "selenium": selenium,
}


def use(name):
    """Return the API-style namespace for ``name``.

    ``name`` is one of: playwright, puppeteer, selenium.
    """
    try:
        module = _MODULES[name]
    except KeyError:
        raise ValueError(
            f"unknown adapter {name!r}; available: {', '.join(sorted(_MODULES))}"
        ) from None
    return module.use()


__all__ = ["use", "playwright", "puppeteer", "selenium"]
