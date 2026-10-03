#!/usr/bin/env python3
"""Ensure the JVM toolchains needed by the Kotlin and Java bindings.

What is installed, and how:

* **JDK 22-24** - via **winget** (``EclipseAdoptium.Temurin.22.JDK``) on Windows.
  Java 22+ is required for the Panama FFI used by the Java bindings, and
  Gradle 8.10 does not yet run on JDK 25.
* **Gradle** - winget ships no Gradle package, so the official distribution is
  downloaded into ``tools/gradle/``.
* **uniffi-bindgen-java 0.4.2** - installed with ``cargo install``; this is the
  release that targets UniFFI 0.31 (matching the Rust core).
* **Kotlin** - no separate install: the Gradle Kotlin plugin provides it.

Usage::

    python scripts/install_toolchains.py           # install anything missing
    python scripts/install_toolchains.py --check    # report only (exit 1 if missing)
"""

from __future__ import annotations

import os
import shutil
import sys
import urllib.request
import zipfile

from common import ROOT, find_tool, gradle_tool, java_home, jdk_major, log, run_checked

JDK_WINGET_ID = "EclipseAdoptium.Temurin.22.JDK"  # 22-24: Panama FFM needs 22+, Gradle 8.10 needs <= 24
GRADLE_VERSION = "8.10.2"
GRADLE_URL = f"https://services.gradle.org/distributions/gradle-{GRADLE_VERSION}-bin.zip"
JAVA_BINDGEN_VERSION = "0.4.2"


def _have(cmd):
    return shutil.which(cmd) is not None


def _cargo_env():
    env = dict(os.environ)
    cargo_bin = os.path.join(os.path.expanduser("~"), ".cargo", "bin")
    if os.path.isdir(cargo_bin):
        env["PATH"] = cargo_bin + os.pathsep + env.get("PATH", "")
    return env


def _winget_install(package_id):
    run_checked(
        [
            "winget",
            "install",
            "--id",
            package_id,
            "--silent",
            "--accept-package-agreements",
            "--accept-source-agreements",
            "--disable-interactivity",
        ]
    )


def ensure_java(install):
    home = java_home()
    major = jdk_major(home)
    if home and major in (22, 23, 24):
        log("OK", f"JDK {major}: {home}")
        return True
    if major:
        log("WARNING", f"JDK {major} found, but Gradle 8.10 needs a 22-24 JDK; installing Temurin 22")
    else:
        log("MISS", "no JDK found")
    if not install:
        return False
    if os.name == "nt" and _have("winget"):
        log("INSTALL", f"winget install {JDK_WINGET_ID}")
        _winget_install(JDK_WINGET_ID)
        home = java_home()
        ok = home is not None and jdk_major(home) in (22, 23, 24)
        log("OK" if ok else "WARNING", f"JDK: {home or 'not found (restart your shell?)'}")
        return ok
    log("MANUAL", "install a JDK 22-24 and set JAVA_HOME")
    return False


def ensure_gradle(install):
    found = gradle_tool()
    if found:
        log("OK", f"gradle: {found}")
        return True
    if not install:
        log("MISS", "gradle not found")
        return False
    tools_dir = os.path.join(ROOT, "tools", "gradle")
    os.makedirs(tools_dir, exist_ok=True)
    archive = os.path.join(tools_dir, f"gradle-{GRADLE_VERSION}-bin.zip")
    log("DOWNLOAD", GRADLE_URL)
    urllib.request.urlretrieve(GRADLE_URL, archive)
    log("EXTRACT", f"{archive} -> {tools_dir}")
    with zipfile.ZipFile(archive) as bundle:
        bundle.extractall(tools_dir)
    found = gradle_tool()
    log("OK" if found else "WARNING", f"gradle: {found or 'extraction failed'}")
    return bool(found)


def ensure_java_bindgen(install):
    found = find_tool("uniffi-bindgen-java")
    if os.path.exists(found):
        log("OK", f"uniffi-bindgen-java: {found}")
        return True
    if not install:
        log("MISS", "uniffi-bindgen-java not found")
        return False
    log("INSTALL", f"cargo install uniffi-bindgen-java --version {JAVA_BINDGEN_VERSION}")
    run_checked(
        ["cargo", "install", "uniffi-bindgen-java", "--version", JAVA_BINDGEN_VERSION, "--force"],
        env=_cargo_env(),
    )
    return True


def main():
    install = "--check" not in sys.argv
    if not install:
        print("=== Checking JVM toolchains (report only) ===")

    ok = True
    ok = ensure_java(install) and ok
    ok = ensure_gradle(install) and ok
    ok = ensure_java_bindgen(install) and ok

    if not ok:
        log("WARNING", "some JVM toolchains are missing; Kotlin/Java packaging will be skipped")
        return 1
    log("SUCCESS", "JVM toolchains ready (Kotlin compiler comes from the Gradle plugin)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
