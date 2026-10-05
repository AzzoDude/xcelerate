#!/usr/bin/env python3
"""Orchestrate the full xcelerate bindgen pipeline.

Builds the Rust core once, regenerates the API-style adapters, then runs every
language generator from the ``generate_bindings`` library. The generators are
independent, so they run concurrently and print a single summary line each.

Set ``SKIP_RUST_BUILD=true`` to reuse pre-built native libraries (as CI does),
``XCELERATE_JOBS`` to cap concurrency, and ``XCELERATE_VERBOSE=1`` to stream the
full output of every step.
"""

from __future__ import annotations

import concurrent.futures
import os
import subprocess
import sys
import time

from common import ROOT, ensure_rust_build

SCRIPTS = os.path.dirname(os.path.abspath(__file__))
if SCRIPTS not in sys.path:
    sys.path.insert(0, SCRIPTS)

import generate_bindings as bindings

COLUMN = 12
# Generators that stage the .NET build and must therefore run last.
_DEFERRED = {"powershell"}


def _verbose():
    return os.environ.get("XCELERATE_VERBOSE") == "1"


def _workers(count):
    """Concurrency for the generator batch (``XCELERATE_JOBS`` overrides)."""
    configured = os.environ.get("XCELERATE_JOBS")
    if configured and configured.isdigit():
        return max(1, min(int(configured), count))
    return max(1, min(count, os.cpu_count() or 4))


def _report(label, status, seconds, output="", failed=False):
    print(f"  {label:<{COLUMN}} {status:<4} {seconds:6.1f}s", flush=True)
    if output and (failed or _verbose()):
        for line in output.rstrip("\n").splitlines():
            print(f"      {line}", flush=True)


def _run(label, cmd, env):
    """Run a step, timing it and surfacing its output only when it matters."""
    start = time.monotonic()
    result = subprocess.run(cmd, cwd=ROOT, env=env, text=True, capture_output=True, check=False)
    seconds = time.monotonic() - start
    failed = result.returncode != 0
    _report(label, "FAIL" if failed else "ok", seconds, (result.stdout or "") + (result.stderr or ""), failed)
    return failed


def main():
    started = time.monotonic()
    print("xcelerate bindgen", flush=True)

    env = dict(os.environ)

    # Build the Rust core once; every generator reuses it.
    start = time.monotonic()
    if env.get("SKIP_RUST_BUILD") == "true":
        _report("rust core", "skip", time.monotonic() - start)
    else:
        ensure_rust_build(quiet=True)
        env["SKIP_RUST_BUILD"] = "true"
        _report("rust core", "ok", time.monotonic() - start)

    failures = []

    # Adapters feed the generators, so they run first and in order.
    for label, script, extra in (
        ("profiles", "backfill_impls.py", ("--prune",)),
        ("adapters", "generate_adapters.py", ()),
    ):
        if _run(label, [sys.executable, os.path.join(SCRIPTS, script), *extra], env):
            failures.append(label)

    # Language generators: the independent ones run concurrently, then the
    # PowerShell module once the .NET build it stages has finished.
    parallel, deferred = [], None
    for module in bindings.PIPELINE:
        short = module.__name__.rsplit(".", 1)[-1]
        if short in _DEFERRED:
            deferred = (short, module.__file__)
        else:
            parallel.append((short, module.__file__))

    with concurrent.futures.ThreadPoolExecutor(max_workers=_workers(len(parallel))) as pool:
        futures = {
            pool.submit(_run, label, [sys.executable, path], env): label
            for label, path in parallel
        }
        for future in concurrent.futures.as_completed(futures):
            if future.result():
                failures.append(futures[future])

    if deferred is not None:
        label, path = deferred
        if "csharp" in failures:
            _report(label, "skip", 0.0)
        elif _run(label, [sys.executable, path], env):
            failures.append(label)

    seconds = time.monotonic() - started
    print(f"  {'total':<{COLUMN}} {'FAIL' if failures else 'ok':<4} {seconds:6.1f}s", flush=True)
    if failures:
        print(f"  failed: {', '.join(failures)}", flush=True)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
