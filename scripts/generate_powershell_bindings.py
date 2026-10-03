#!/usr/bin/env python3
"""Stage the PowerShell module payload from the .NET binding.

PowerShell has no UniFFI generator, so the module drives the generated .NET SDK
directly. The module itself (``bindings/powershell/Xcelerate.psm1``) is
hand-written; this script only copies the managed assembly and the native cdylib
next to each other, per target framework, so the module can load the pair that
matches the runtime hosting PowerShell.
"""

from __future__ import annotations

import os
import shutil
import sys

from common import ROOT, log, run_checked

SCRIPTS = os.path.dirname(os.path.abspath(__file__))
TFMS = ("net8.0", "net9.0", "net10.0")
MANAGED = "Xcelerate.Net.dll"
NATIVE_LIBS = ("xcelerate.dll", "libxcelerate.so", "libxcelerate.dylib")


def _built_assembly(csharp_dir):
    return os.path.join(csharp_dir, "bin", "Release", "net10.0", MANAGED)


def main():
    csharp_dir = os.path.join(ROOT, "bindings", "csharp")
    out_dir = os.path.join(ROOT, "bindings", "powershell", "lib")

    if not os.path.exists(_built_assembly(csharp_dir)):
        # Reuse the .NET generator: it builds the Rust core, generates the source,
        # and produces the managed assembly + native cdylib in one step.
        run_checked([sys.executable, os.path.join(SCRIPTS, "generate_csharp_bindings.py")], cwd=ROOT)

    staged = []
    for tfm in TFMS:
        src_dir = os.path.join(csharp_dir, "bin", "Release", tfm)
        managed = os.path.join(src_dir, MANAGED)
        if not os.path.exists(managed):
            continue
        dst_dir = os.path.join(out_dir, tfm)
        os.makedirs(dst_dir, exist_ok=True)
        shutil.copy2(managed, dst_dir)
        for lib in NATIVE_LIBS:
            native = os.path.join(src_dir, lib)
            if os.path.exists(native):
                shutil.copy2(native, os.path.join(dst_dir, lib))
        staged.append(tfm)

    if not staged:
        log("ERROR", "no PowerShell payload staged; the .NET build produced no assemblies")
        return 1

    log("SUCCESS", f"PowerShell payload staged ({', '.join(staged)}) in {os.path.relpath(out_dir, ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
