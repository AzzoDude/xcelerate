#!/usr/bin/env python3
"""Generate the JavaScript/TypeScript bindings (uniffi) and pack the npm module."""

from __future__ import annotations

import json
import os
import re
import shutil

from common import ROOT, find_tool, log, run_checked, workspace_version


def main():
    dll_name = "xcelerate.dll"
    built_dll = os.path.join(ROOT, "target", "release", dll_name)
    js_dir = os.path.join(ROOT, "bindings", "javascript")

    print("--- Phase: Generating JavaScript Bindings ---")

    print("--- 0. Building Rust library (cdylib) ---")
    if os.environ.get("SKIP_RUST_BUILD") == "true" and os.path.exists(built_dll):
        log("SKIP", f"Rust build skipped, using existing: {built_dll}")
    else:
        run_checked(["cargo", "build", "--release"], cwd=ROOT)

    # Remove stale generated files (but never package.json). Tarballs are
    # cleared too so a previous `npm pack` output can't be bundled into the next.
    if os.path.exists(js_dir):
        for name in os.listdir(js_dir):
            if name == "package.json":
                continue
            if name.endswith((".ts", ".js", ".d.ts", ".tgz")):
                os.remove(os.path.join(js_dir, name))
    os.makedirs(js_dir, exist_ok=True)

    version = workspace_version()
    tool_cmd = find_tool("uniffi-bindgen-node-js")
    run_checked(
        [tool_cmd, "generate", "--out-dir", js_dir, "--package-name", "xcelerate", built_dll],
        cwd=ROOT,
    )

    # Patch package.json with version and metadata.
    package_json = os.path.join(js_dir, "package.json")
    if os.path.exists(package_json):
        with open(package_json, "r", encoding="utf-8") as handle:
            pj = json.load(handle)
        pj.update(
            {
                "version": version,
                "description": "A high-performance, lightweight Chrome DevTools Protocol (CDP) client for Node.js",
                "author": "AzzoDude",
                "license": "MIT OR Apache-2.0",
                "engines": {"node": ">=12"},
                "repository": {"type": "git", "url": "git+https://github.com/AzzoDude/xcelerate.git"},
            }
        )
        with open(package_json, "w", encoding="utf-8") as handle:
            json.dump(pj, handle, indent=2)
        log("PATCH", f"updated package.json to version {version}")

    for lib in ("xcelerate.dll", "libxcelerate.so", "libxcelerate.dylib"):
        src = os.path.join(ROOT, "target", "release", lib)
        if os.path.exists(src):
            shutil.copy2(src, os.path.join(js_dir, lib))
            log("COPY", f"{lib} -> {js_dir}")

    # POST-PROCESS: optional args and camelCase renames.
    js_file = os.path.join(js_dir, "xcelerate.js")
    if os.path.exists(js_file):
        with open(js_file, "r", encoding="utf-8") as handle:
            content = handle.read()

        content = re.sub(
            r"static async launch\(config\) \{",
            "static async launch(config = {}) {\n"
            "    const finalConfig = {\n"
            "      headless: true,\n"
            "      detached: true,\n"
            "      executable_path: null,\n"
            "      plugins: null,\n"
            "      ...config\n"
            "    };\n"
            "    config = finalConfig;",
            content,
        )

        for old, new in (
            ("async new_page(", "async newPage("),
            ("async inner_html(", "async innerHtml("),
            ("async screenshot_full(", "async screenshotFull("),
            ("async find_element(", "async findElement("),
            ("async wait_for_navigation(", "async waitForNavigation("),
            ("async wait_for_selector(", "async waitForSelector("),
            ("async type_text(", "async typeText("),
            ("async add_script_to_evaluate_on_new_document(", "async addScriptToEvaluateOnNewDocument("),
            ("async use_plugin(", "async usePlugin("),
            ("load_plugin(", "loadPlugin("),
            ("plugin_names(", "pluginNames("),
            ("available_plugins(", "availablePlugins("),
            ("plugin_name(", "pluginName("),
            ("audit_verify(", "auditVerify("),
            ("audit_log(", "auditLog("),
        ):
            content = content.replace(old, new)

        with open(js_file, "w", encoding="utf-8") as handle:
            handle.write(content)

    log("SUCCESS", f"JavaScript bindings ready in {js_dir}")

    run_checked(["npm", "pack"], cwd=js_dir)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
