#!/usr/bin/env python3
"""Generate the Python bindings (uniffi) and build the wheel."""

from __future__ import annotations

import os
import re
import sys

try:
    from ._shared import (
        BUILT_DLL,
        ROOT,
        copy_native_libs,
        ensure_rust_build,
        log,
        run_checked,
        uniffi_bindgen,
        workspace_version,
    )
except ImportError:  # pragma: no cover - executed as a standalone script
    from _shared import (
        BUILT_DLL,
        ROOT,
        copy_native_libs,
        ensure_rust_build,
        log,
        run_checked,
        uniffi_bindgen,
        workspace_version,
    )


def main():
    python_dir = os.path.join(ROOT, "bindings", "python")

    print("--- Phase: Generating Python Bindings ---")
    os.makedirs(python_dir, exist_ok=True)

    print("--- 0. Building Rust library (cdylib) ---")
    ensure_rust_build()

    print("--- 1. Generating Python code with UniFFI ---")
    run_checked(
        uniffi_bindgen()
        + ["generate", "--library", BUILT_DLL, "--language", "python", "--out-dir", python_dir],
        cwd=ROOT,
    )

    # Move the native libraries into the package folder.
    package_dir = os.path.join(python_dir, "xcelerate")
    copy_native_libs(package_dir)

    # Ensure __init__.py exists with the workspace version and adapter surface.
    with open(os.path.join(package_dir, "__init__.py"), "w", encoding="utf-8") as handle:
        handle.write(f'__version__ = "{workspace_version()}"\n')
        handle.write("from .xcelerate import Browser, BrowserConfig, Page, Element, XcelerateError\n")
        handle.write("\n# API-style adapters (Selenium / Playwright / Puppeteer profiles).\n")
        handle.write("try:\n")
        handle.write("    from . import adapters\n")
        handle.write("    from .adapters import use, playwright, puppeteer, selenium\n")
        handle.write("except Exception:\n")
        handle.write("    pass\n")
        handle.write(
            "\n__all__ = [\"Browser\", \"BrowserConfig\", \"Page\", \"Element\", "
            "\"XcelerateError\", \"adapters\", \"use\", \"playwright\", \"puppeteer\", \"selenium\"]\n"
        )

    # Move the generated module into the package, making BrowserConfig args optional.
    generated_py = os.path.join(python_dir, "xcelerate.py")
    if os.path.exists(generated_py):
        with open(generated_py, "r", encoding="utf-8") as handle:
            content = handle.read()
        pattern = (
            r"def __init__\(self, \*, headless:\s*[\"']?bool[\"']?, "
            r"detached:\s*[\"']?bool[\"']?, executable_path:\s*[\"']?typing\.Optional\[str\][\"']?, "
            r"plugins:\s*[\"']?typing\.Optional\[typing\.(?:List|Sequence)\[str\]\][\"']?\):"
        )
        replacement = (
            r"def __init__(self, *, headless: bool = True, "
            r"detached: bool = True, executable_path: typing.Optional[str] = None, "
            r"plugins: typing.Optional[typing.List[str]] = None):"
        )
        content = re.sub(pattern, replacement, content)
        with open(os.path.join(package_dir, "xcelerate.py"), "w", encoding="utf-8") as handle:
            handle.write(content)
        os.remove(generated_py)

    print("[PACKAGING] Building Python wheel...")
    run_checked([sys.executable, "-m", "build"], cwd=python_dir)
    log("SUCCESS", f"Python package ready in {os.path.join(python_dir, 'dist')}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
