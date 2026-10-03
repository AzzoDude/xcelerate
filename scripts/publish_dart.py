#!/usr/bin/env python3
"""Validate and (optionally) publish the Dart/Flutter package to pub.dev.

    python scripts/publish_dart.py            # regenerate + `dart pub publish --dry-run`
    python scripts/publish_dart.py --push     # `dart pub publish --force`

Authentication is a Google account via ``dart pub publish`` (OAuth on first run),
or a token in ``PUB_TOKEN`` for CI.
"""

from __future__ import annotations

import os
import sys

from common import ROOT, find_tool, log, run_checked


def main():
    push = "--push" in sys.argv
    dart_dir = os.path.join(ROOT, "bindings", "dart")

    dart = find_tool("dart")
    if not os.path.exists(dart):
        log("WARNING", "dart not found; install the Dart SDK or Flutter (dart.dev)")
        return 0

    print("--- Regenerating Dart bindings ---")
    run_checked(
        [sys.executable, os.path.join(ROOT, "scripts", "generate_dart_bindings.py")],
        cwd=ROOT,
    )

    print("--- Validating the package ---")
    run_checked([dart, "pub", "publish", "--dry-run"], cwd=dart_dir)

    if not push:
        log("INFO", "dry run - pass --push to publish to pub.dev")
        return 0

    print("--- Publishing to pub.dev ---")
    run_checked([dart, "pub", "publish", "--force"], cwd=dart_dir)
    log("SUCCESS", "published to pub.dev")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
