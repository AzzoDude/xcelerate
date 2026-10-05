#!/usr/bin/env python3
"""Build and (optionally) publish the Ruby gem.

    python scripts/publish/ruby.py            # regenerate + gem build (dry run)
    python scripts/publish/ruby.py --push     # gem build + gem push

Authentication for ``gem push`` comes from ``~/.gem/credentials`` (run
``gem signin`` once) or the ``GEM_HOST_API_KEY`` environment variable.
"""

from __future__ import annotations

import os

try:
    from ._shared import (
        ROOT,
        find_tool,
        log,
        parse_args,
        regenerate,
        run_checked,
        workspace_version,
    )
except ImportError:  # pragma: no cover - executed as a standalone script
    from _shared import (
        ROOT,
        find_tool,
        log,
        parse_args,
        regenerate,
        run_checked,
        workspace_version,
    )


def main():
    options = parse_args()
    ruby_dir = os.path.join(ROOT, "bindings", "ruby")

    gem = find_tool("gem")
    if not os.path.exists(gem):
        log("WARNING", "gem not found; install Ruby 3.1+ (ruby-lang.org)")
        return 0

    print("--- Regenerating Ruby bindings ---")
    regenerate("ruby")

    print("--- Building the gem ---")
    run_checked([gem, "build", "xcelerate.gemspec"], cwd=ruby_dir)
    built = f"xcelerate-{workspace_version()}.gem"
    log("BUILT", built)

    if not options.push:
        log("INFO", "dry run - pass --push to publish to RubyGems")
        return 0

    if not os.environ.get("GEM_HOST_API_KEY"):
        log("WARNING", "GEM_HOST_API_KEY not set and no ~/.gem/credentials; skipping push")
        return 0

    print("--- Pushing to RubyGems ---")
    run_checked([gem, "push", built], cwd=ruby_dir)
    log("SUCCESS", "published to RubyGems")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
