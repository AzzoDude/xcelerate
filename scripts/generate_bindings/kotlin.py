#!/usr/bin/env python3
"""Generate the Kotlin bindings (uniffi) and assemble a Gradle JVM library.

Kotlin is a first-class UniFFI target, so the bundled ``uniffi-bindgen`` helper
produces the sources. The generated code depends on JNA (the FFI layer) and
kotlinx-coroutines (the suspend/async surface); both are declared in the Gradle
build that is written next to the sources.
"""

from __future__ import annotations

import os
import re
import shutil

try:
    from ._shared import (
        BINDINGS_DIR,
        BUILT_DLL,
        ROOT,
        copy_native_libs,
        ensure_rust_build,
        gradle_build_script,
        gradle_tool,
        jvm_env,
        log,
        run_checked,
        uniffi_bindgen,
        write_file,
        write_gradle_project,
    )
except ImportError:  # pragma: no cover - executed as a standalone script
    from _shared import (
        BINDINGS_DIR,
        BUILT_DLL,
        ROOT,
        copy_native_libs,
        ensure_rust_build,
        gradle_build_script,
        gradle_tool,
        jvm_env,
        log,
        run_checked,
        uniffi_bindgen,
        write_file,
        write_gradle_project,
    )

_GRADLE_BUILD = gradle_build_script(
    "scripts/generate_bindings/kotlin.py",
    plugins=(
        '    kotlin("jvm") version "2.1.0"\n'
        "    `java-library`\n"
        "    `maven-publish`\n"
        "    `signing`"
    ),
    dependencies=(
        "\n"
        "dependencies {\n"
        "    // The generated bindings use JNA for the FFI and kotlinx-coroutines for the\n"
        "    // suspend/asynchronous API surface.\n"
        '    api("net.java.dev.jna:jna:5.16.0")\n'
        '    api("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.10.1")\n'
        "}\n"
    ),
    compile_options="",
    artifact="xcelerate",
    pom_name="Xcelerate Kotlin SDK",
    pom_description="Kotlin/JVM bindings for the xcelerate CDP engine.",
)


def _patch_defaults(content):
    """Give the generated ``BrowserConfig`` (and ``launch``) Kotlin defaults.

    UniFFI emits required positional fields, so ``BrowserConfig()`` would not
    compile without them. The exact spacing of the generated ``data class`` is
    stable per UniFFI release; these regexes only add a default when one is not
    already present.
    """
    edits = (
        (r"var `headless`: kotlin\.Boolean(?!\s*=)", "var `headless`: kotlin.Boolean = true"),
        (r"var `detached`: kotlin\.Boolean(?!\s*=)", "var `detached`: kotlin.Boolean = true"),
        (r"var `executablePath`: kotlin\.String\?(?!\s*=)", "var `executablePath`: kotlin.String? = null"),
        (
            r"var `plugins`: List<kotlin\.String>\?(?!\s*=)",
            "var `plugins`: List<kotlin.String>? = null",
        ),
        (r"(suspend fun `launch`\(`config`: BrowserConfig)\)", r"\1 = BrowserConfig())"),
    )
    for pattern, replacement in edits:
        content, count = re.subn(pattern, replacement, content)
        if count == 0:
            log("WARNING", f"Kotlin default patch did not match: {pattern}")
    return content


def main():
    kotlin_dir = os.path.join(BINDINGS_DIR, "kotlin")
    src_dir = os.path.join(kotlin_dir, "src", "main", "kotlin")

    print("--- Phase: Generating Kotlin Bindings ---")
    os.makedirs(kotlin_dir, exist_ok=True)

    print("--- 0. Building Rust library (cdylib) ---")
    ensure_rust_build()

    print("--- 1. Generating Kotlin sources with UniFFI ---")
    if os.path.isdir(src_dir):
        shutil.rmtree(src_dir)
    os.makedirs(src_dir, exist_ok=True)
    config = os.path.join(ROOT, "scripts", "uniffi.toml")
    run_checked(
        uniffi_bindgen()
        + [
            "generate",
            "--library",
            BUILT_DLL,
            "--language",
            "kotlin",
            "--no-format",
            "-c",
            config,
            "--out-dir",
            src_dir,
        ],
        cwd=ROOT,
    )

    generated = os.path.join(src_dir, "uniffi", "xcelerate", "xcelerate.kt")
    if os.path.exists(generated):
        with open(generated, "r", encoding="utf-8") as handle:
            content = _patch_defaults(handle.read())
        write_file(generated, content)
        log("PATCH", "BrowserConfig defaults + launch() default applied")

    print("--- 2. Distributing native libraries ---")
    copy_native_libs(kotlin_dir)

    print("--- 3. Writing Gradle build ---")
    write_gradle_project(kotlin_dir, _GRADLE_BUILD)

    gradle = gradle_tool()
    if not gradle:
        log("SKIP", "gradle not found (see scripts/install_toolchains.py); sources generated only")
        return 0

    print("--- 4. Building Kotlin library (gradle) ---")
    run_checked([gradle, "build", "-x", "test", "--no-daemon"], cwd=kotlin_dir, env=jvm_env())
    log("SUCCESS", f"Kotlin bindings ready in {kotlin_dir}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
