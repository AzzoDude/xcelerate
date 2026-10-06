#!/usr/bin/env python3
"""Validate and (optionally) publish the Dart/Flutter package to pub.dev.

    python scripts/publish/dart.py            # regenerate + `dart pub publish --dry-run`
    python scripts/publish/dart.py --push     # `dart pub publish --force`

This is the local/manual path; authentication is a Google account via
``dart pub publish`` (OAuth on first run). CI does not use this script: pushing a
``vX.Y.Z`` tag publishes through ``.github/workflows/publish-dart.yml`` using
pub.dev's OIDC automated publishing.
"""

from __future__ import annotations

import os
import subprocess

try:
    from ._shared import (
        ROOT,
        find_tool,
        log,
        parse_args,
        regenerate,
        run_checked,
    )
except ImportError:  # pragma: no cover - executed as a standalone script
    from _shared import (
        ROOT,
        find_tool,
        log,
        parse_args,
        regenerate,
        run_checked,
    )


def dart_exe() -> str:
    """Locate `dart`, normalising an uppercase `.EXE` suffix (see the generator)."""
    found = find_tool("dart")
    if found.lower().endswith(".exe"):
        return found[:-4] + ".exe"
    return found


def main():
    options = parse_args()
    dart_dir = os.path.join(ROOT, "bindings", "dart")

    dart_path = dart_exe()
    if not os.path.exists(dart_path):
        log("WARNING", "dart not found; install the Dart SDK or Flutter (dart.dev)")
        return 0

    print("--- Regenerating Dart bindings ---")
    regenerate("dart")

    print("--- Validating the package ---")
    dry_run = subprocess.run(
        [dart_path, "pub", "publish", "--dry-run"], cwd=dart_dir, text=True, check=False
    )
    if dry_run.returncode != 0:
        log("WARNING", "`dart pub publish --dry-run` reported issues; not publishing")
        return 1

    if not options.push:
        log("INFO", "dry run - pass --push to publish to pub.dev")
        return 0

    print("--- Publishing to pub.dev ---")
    run_checked([dart_path, "pub", "publish", "--force"], cwd=dart_dir)
    log("SUCCESS", "published to pub.dev")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
