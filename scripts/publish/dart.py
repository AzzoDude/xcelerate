#!/usr/bin/env python3
"""Validate and (optionally) publish the Dart/Flutter package to pub.dev.

    python scripts/publish/dart.py            # regenerate + `dart pub publish --dry-run`
    python scripts/publish/dart.py --push     # `dart pub publish --force`

Authentication is a Google account via ``dart pub publish`` (OAuth on first run),
or a token in ``PUB_TOKEN`` for CI.
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

    # CI provides a token in PUB_TOKEN; `dart pub publish` does not read that
    # variable on its own, so register it as the credential for pub.dev first.
    if os.environ.get("PUB_TOKEN"):
        run_checked(
            [dart_path, "pub", "token", "add", "https://pub.dev", "--env-var", "PUB_TOKEN"],
            cwd=dart_dir,
        )

    print("--- Publishing to pub.dev ---")
    run_checked([dart_path, "pub", "publish", "--force"], cwd=dart_dir)
    log("SUCCESS", "published to pub.dev")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
