#!/usr/bin/env python3
"""Publish the Swift package.

SwiftPM has no central registry: a package is a git repository whose **root**
contains ``Package.swift``. Our package lives in ``bindings/swift``, so it must
be published to a dedicated repository. This script uses ``git subtree`` to split
``bindings/swift`` onto a branch and push it to a remote:

    python scripts/publish_swift.py                       # split onto a local branch
    python scripts/publish_swift.py --push                # split + push to `swift` remote
    python scripts/publish_swift.py --push --remote swift

Point ``--remote`` at a repository such as ``xcelerate-swift``. The tree must be
committed first.
"""

from __future__ import annotations

import subprocess
import sys

from common import ROOT, log, workspace_version

BRANCH = "swift-release"


def git(*args) -> subprocess.CompletedProcess:
    return subprocess.run(["git", *args], cwd=ROOT, text=True, capture_output=True)


def main():
    push = "--push" in sys.argv
    remote = "swift"
    if "--remote" in sys.argv:
        remote = sys.argv[sys.argv.index("--remote") + 1]

    version = workspace_version()

    print("--- Splitting bindings/swift onto a branch ---")
    result = git("subtree", "split", "-P", "bindings/swift", "-b", BRANCH)
    if result.returncode != 0:
        log("ERROR", result.stderr.strip() or "git subtree split failed")
        return 1
    commit = result.stdout.strip().splitlines()[-1] if result.stdout.strip() else BRANCH
    log("SPLIT", f"{BRANCH} -> {commit}")

    if not push:
        log("INFO", f"dry run - pass --push to push {BRANCH} to the '{remote}' remote")
        return 0

    print(f"--- Pushing {BRANCH} to {remote} ---")
    result = git("push", remote, f"{BRANCH}:main")
    if result.returncode != 0:
        log("ERROR", result.stderr.strip() or "git push failed")
        return 1

    tag = f"{version}"
    result = git("tag", "-a", tag, "-m", f"xcelerate Swift SDK {version}")
    if result.returncode == 0:
        git("push", remote, tag)
    log("SUCCESS", f"pushed {BRANCH} (tag {version}) to {remote}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
