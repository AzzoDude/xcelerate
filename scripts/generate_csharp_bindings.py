#!/usr/bin/env python3
"""Generate the C# bindings (uniffi) and package the NuGet SDK."""

from __future__ import annotations

import os
import re
import shutil

from common import ROOT, find_tool, log, run_checked


def _with_browser_config_defaults(content):
    """Add default values to the generated ``BrowserConfig`` record fields.

    ``uniffi-bindgen-cs`` emits required positional fields, which forces every
    caller to pass all four values. Supplying defaults lets callers write
    ``new BrowserConfig()``. The exact formatting (doc comments, field casing,
    trailing commas) changes between bindgen releases, so this walks the record
    body line by line instead of matching a fixed multi-line block.
    """
    lines = content.split("\n")
    inside = False
    for index, line in enumerate(lines):
        if "record BrowserConfig (" in line:
            inside = True
            continue
        if not inside:
            continue
        stripped = line.strip()
        if stripped.startswith(")"):
            break
        if "=" in stripped:
            continue
        if stripped.startswith("bool "):
            default = "true"
        elif stripped.startswith("string? "):
            default = "null"
        else:
            continue
        end = "," if stripped.endswith(",") else ""
        core = line.rstrip().rstrip(",").rstrip()
        lines[index] = f"{core} = {default}{end}"
    return "\n".join(lines)


def main():
    dll_name = "xcelerate.dll"
    built_dll = os.path.join(ROOT, "target", "release", dll_name)
    csharp_dir = os.path.join(ROOT, "bindings", "csharp")

    print("--- 1. Building Rust library (cdylib) ---")
    if os.environ.get("SKIP_RUST_BUILD") == "true" and os.path.exists(built_dll):
        log("SKIP", f"Rust build skipped, using existing: {built_dll}")
    else:
        run_checked(["cargo", "build", "--release"], cwd=ROOT)

    print("\n--- 2. Generating UniFFI C# bindings ---")
    tool_cmd = find_tool("uniffi-bindgen-cs")
    log("DEBUG", f"using bindgen: {tool_cmd}")
    config = os.path.join(ROOT, "scripts", "uniffi-csharp.toml")
    run_checked(
        [tool_cmd, "-c", config, "--library", "--out-dir", csharp_dir, built_dll],
        cwd=ROOT,
    )

    print("\n--- 3. Fixing visibility (internal -> public) ---")
    # UniFFI's C# generator defaults to internal; make it public for consumers.
    generated_cs = os.path.join(csharp_dir, "xcelerate.cs")
    with open(generated_cs, "r", encoding="utf-8") as handle:
        content = handle.read()

    for old, new in (
        ("internal class", "public class"),
        ("internal interface", "public interface"),
        ("internal record", "public record"),
        ("internal enum", "public enum"),
        ("internal struct", "public struct"),
        ("internal abstract class", "public abstract class"),
    ):
        content = content.replace(old, new)

    for keyword in ("class", "struct", "interface", "enum"):
        content = re.sub(rf"^{keyword} ", f"public {keyword} ", content, flags=re.MULTILINE)

    # Make BrowserConfig arguments optional so `new BrowserConfig()` works.
    content = _with_browser_config_defaults(content)

    with open(generated_cs, "w", encoding="utf-8") as handle:
        handle.write(content)

    print("\n--- 4. Distributing native libraries ---")
    for lib in ("xcelerate.dll", "libxcelerate.so", "libxcelerate.dylib"):
        src = os.path.join(ROOT, "target", "release", lib)
        if os.path.exists(src):
            shutil.copy2(src, os.path.join(csharp_dir, lib))
            log("COPY", f"{lib} -> {csharp_dir}")

    test_app_bin = os.path.join(csharp_dir, "Xcelerate.TestApp", "bin", "Debug", "net10.0")
    if os.path.exists(test_app_bin):
        shutil.copy2(built_dll, os.path.join(test_app_bin, dll_name))
        log("COPY", f"{dll_name} -> {test_app_bin}")

    print("\n--- 5. Building C# wrapper ---")
    run_checked(["dotnet", "build", "-c", "Release"], cwd=csharp_dir)

    print("\n--- 6. Packaging NuGet (.nupkg) ---")
    run_checked(["dotnet", "pack", "-c", "Release"], cwd=csharp_dir)

    print("\n--- DONE ---")
    log("SUCCESS", f"C# SDK ready in {os.path.join(csharp_dir, 'bin', 'Release')}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
