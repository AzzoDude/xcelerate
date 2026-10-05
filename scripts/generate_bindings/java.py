#!/usr/bin/env python3
"""Generate the Java bindings (uniffi) and assemble a Gradle JVM library.

Java is not a built-in UniFFI target; it needs the external
``uniffi-bindgen-java`` generator (IronCoreLabs). Version ``0.4.2`` is the
release that targets UniFFI ``0.31`` - the same metadata version as the Rust
core - so keep the two in step.

The generated code uses Java's Foreign Function & Memory API (Project Panama),
so it has **no external runtime dependencies** and requires **Java 22+**.
Install the toolchain with ``python scripts/install_toolchains.py``.
"""

from __future__ import annotations

import os
import shutil

try:
    from ._shared import (
        BINDINGS_DIR,
        BUILT_DLL,
        ROOT,
        copy_native_libs,
        ensure_rust_build,
        find_binding_tool,
        gradle_build_script,
        gradle_tool,
        jvm_env,
        log,
        run_checked,
        write_gradle_project,
    )
except ImportError:  # pragma: no cover - executed as a standalone script
    from _shared import (
        BINDINGS_DIR,
        BUILT_DLL,
        ROOT,
        copy_native_libs,
        ensure_rust_build,
        find_binding_tool,
        gradle_build_script,
        gradle_tool,
        jvm_env,
        log,
        run_checked,
        write_gradle_project,
    )

JAVA_BINDGEN = "uniffi-bindgen-java"

_GRADLE_BUILD = gradle_build_script(
    "scripts/generate_bindings/java.py",
    plugins="    `java-library`\n    `maven-publish`\n    `signing`",
    dependencies="",
    compile_options=(
        "\n"
        "tasks.withType<JavaCompile>().configureEach {\n"
        "    // The bindings use the Foreign Function & Memory API, finalized in Java 22.\n"
        "    options.release = 22\n"
        "}\n"
    ),
    artifact="xcelerate-java",
    pom_name="Xcelerate Java SDK",
    pom_description="Java/JVM bindings for the xcelerate CDP engine.",
)


def main():
    java_dir = os.path.join(BINDINGS_DIR, "java")
    src_dir = os.path.join(java_dir, "src", "main", "java")

    print("--- Phase: Generating Java Bindings ---")
    os.makedirs(java_dir, exist_ok=True)

    tool = find_binding_tool(
        JAVA_BINDGEN, "run `python scripts/install_toolchains.py` to install it via cargo"
    )
    if not tool:
        return 0

    print("--- 0. Building Rust library (cdylib) ---")
    ensure_rust_build()

    print("--- 1. Generating Java sources with uniffi-bindgen-java ---")
    if os.path.isdir(src_dir):
        shutil.rmtree(src_dir)
    os.makedirs(src_dir, exist_ok=True)
    config = os.path.join(ROOT, "scripts", "uniffi.toml")
    run_checked(
        [tool, "generate", "-c", config, "--out-dir", src_dir, BUILT_DLL], cwd=ROOT
    )

    print("--- 2. Distributing native libraries ---")
    copy_native_libs(java_dir)

    print("--- 3. Writing Gradle build ---")
    write_gradle_project(java_dir, _GRADLE_BUILD)

    gradle = gradle_tool()
    if not gradle:
        log("SKIP", "gradle not found (see scripts/install_toolchains.py); sources generated only")
        return 0

    print("--- 4. Building Java library (gradle) ---")
    run_checked([gradle, "build", "-x", "test", "--no-daemon"], cwd=java_dir, env=jvm_env())
    log("SUCCESS", f"Java bindings ready in {java_dir}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
