#!/usr/bin/env python3
"""Generate the Python bindings (uniffi) and build the wheel."""

from __future__ import annotations

import os
import re
import shutil
import sys

from common import ROOT, find_tool, log, run_checked, workspace_version


def bindgen_tool():
    """Prefer a pre-built ``uniffi-bindgen`` binary over ``cargo run``."""
    for candidate in (
        os.path.join(ROOT, "target", "debug", "uniffi-bindgen.exe"),
        os.path.join(ROOT, "target", "release", "uniffi-bindgen.exe"),
    ):
        if os.path.exists(candidate):
            log("INFO", f"using pre-built bindgen: {candidate}")
            return [candidate]
    found = find_tool("uniffi-bindgen")
    if os.path.exists(found):
        return [found]
    return ["cargo", "run", "-p", "xcelerate-bindgen", "--"]


def main():
    dll_name = "xcelerate.dll"
    built_dll = os.path.join(ROOT, "target", "release", dll_name)
    python_dir = os.path.join(ROOT, "bindings", "python")

    print("--- Phase: Generating Python Bindings ---")
    os.makedirs(python_dir, exist_ok=True)

    print("--- 0. Building Rust library (cdylib) ---")
    if os.environ.get("SKIP_RUST_BUILD") == "true" and os.path.exists(built_dll):
        log("SKIP", f"Rust build skipped, using existing: {built_dll}")
    else:
        run_checked(["cargo", "build", "--release"], cwd=ROOT)

    print("--- 1. Generating Python code with UniFFI ---")
    run_checked(
        bindgen_tool()
        + ["generate", "--library", built_dll, "--language", "python", "--out-dir", python_dir],
        cwd=ROOT,
    )

    # Move the native libraries into the package folder.
    package_dir = os.path.join(python_dir, "xcelerate")
    os.makedirs(package_dir, exist_ok=True)
    for lib in ("xcelerate.dll", "libxcelerate.so", "libxcelerate.dylib"):
        src = os.path.join(ROOT, "target", "release", lib)
        if os.path.exists(src):
            shutil.copy2(src, os.path.join(package_dir, lib))
            log("COPY", f"{lib} -> {package_dir}")

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
            r"def __init__\(self, \*, headless:\s*[\"']?bool[\"']?, stealth:\s*[\"']?bool[\"']?, "
            r"detached:\s*[\"']?bool[\"']?, executable_path:\s*[\"']?typing\.Optional\[str\][\"']?, "
            r"plugins:\s*[\"']?typing\.Optional\[typing\.(?:List|Sequence)\[str\]\][\"']?\):"
        )
        replacement = (
            r"def __init__(self, *, headless: bool = True, stealth: bool = False, "
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
