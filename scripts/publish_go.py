#!/usr/bin/env python3
"""Publish the Go module by tagging the repository.

Go modules are distributed through the module proxy (proxy.golang.org) from git,
so publishing is just a tag. Because the module lives in ``bindings/go``, the tag
must be prefixed with the module directory: ``bindings/go/vX.Y.Z``.

    python scripts/publish_go.py            # create the tag locally
    python scripts/publish_go.py --push     # create and push the tag
    python scripts/publish_go.py --push --remote upstream

The tree must be committed first; tags point at commits, not the working tree.
Pushing needs push access to the remote (default: ``origin``).
"""

from __future__ import annotations

import os
import subprocess
import sys

from common import ROOT, log, workspace_version


def git(*args) -> subprocess.CompletedProcess:
    return subprocess.run(["git", *args], cwd=ROOT, text=True, capture_output=True)


def main():
    push = "--push" in sys.argv
    remote = "origin"
    if "--remote" in sys.argv:
        remote = sys.argv[sys.argv.index("--remote") + 1]

    version = workspace_version()
    tag = f"bindings/go/v{version}"

    if git("rev-parse", "-q", "--verify", f"refs/tags/{tag}").returncode == 0:
        log("SKIP", f"tag {tag} already exists")
    else:
        result = git("tag", "-a", tag, "-m", f"xcelerate Go bindings v{version}")
        if result.returncode != 0:
            log("ERROR", result.stderr.strip() or "git tag failed")
            return 1
        log("TAGGED", tag)

    if not push:
        log("INFO", "dry run - pass --push to push the tag to the remote")
        return 0

    print(f"--- Pushing {tag} to {remote} ---")
    result = git("push", remote, tag)
    if result.returncode != 0:
        log("ERROR", result.stderr.strip() or "git push failed")
        return 1
    log("SUCCESS", f"pushed {tag}; indexed by pkg.go.dev shortly")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
