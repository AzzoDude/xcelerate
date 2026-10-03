#!/usr/bin/env python3
"""Orchestrate the full xcelerate bindgen pipeline.

Runs, in order:

1. (optional) release build of the Rust core,
2. API-style adapters (Python + Rust) from the profiles,
3. C# / Python / JavaScript / Kotlin / Java / Swift / Ruby / Dart / Go uniffi bindings.

Set ``SKIP_RUST_BUILD=true`` to reuse pre-built native libraries (as CI does).
"""

from __future__ import annotations

import os
import sys

from common import ROOT, run_checked

SCRIPTS = os.path.dirname(os.path.abspath(__file__))


def phase(label, script, *extra):
    print(f"\n[PHASE] {label}")
    cmd = [sys.executable, os.path.join(SCRIPTS, script), *extra]
    run_checked(cmd, cwd=ROOT)


def main():
    print("=== Xcelerate Universal Bindgen Pipeline ===")

    if os.environ.get("SKIP_RUST_BUILD") == "true":
        print("\n--- Phase 1: Skipping Rust build (using pre-built binaries) ---")
    else:
        print("\n--- Phase 1: Building Rust core (release) ---")
        run_checked(["cargo", "build", "--release"], cwd=ROOT)

    phase("Populate + prune adapter profiles", "backfill_impls.py", "--prune")
    phase("API-style adapters (Python + Rust)", "generate_adapters.py")
    phase("C# bindings", "generate_csharp_bindings.py")
    phase("Python bindings", "generate_python_bindings.py")
    phase("JavaScript bindings", "generate_javascript_bindings.py")
    phase("Kotlin bindings", "generate_kotlin_bindings.py")
    phase("Java bindings", "generate_java_bindings.py")
    phase("Swift bindings", "generate_swift_bindings.py")
    phase("Ruby bindings", "generate_ruby_bindings.py")
    phase("Dart bindings", "generate_dart_bindings.py")
    phase("Go bindings", "generate_go_bindings.py")
    phase("PowerShell module", "generate_powershell_bindings.py")

    print("\n=== Universal Pipeline Finished Successfully ===")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
