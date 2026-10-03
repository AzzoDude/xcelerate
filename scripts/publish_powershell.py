#!/usr/bin/env python3
"""Stage and (optionally) publish the PowerShell module.

    python scripts/publish_powershell.py           # stage + dry run
    python scripts/publish_powershell.py --push     # publish to the PowerShell Gallery

Authentication for the push comes from the ``PSGALLERY_API_KEY`` environment
variable. Never pass a key on the command line: it would leak into shell history,
logs, and the process list. Create a key (it is a GUID) at
https://www.powershellgallery.com/account/apikeys and rotate it if it is ever
exposed. Override the feed with ``PSGALLERY_REPOSITORY`` (default ``PSGallery``).
"""

from __future__ import annotations

import os
import sys

from common import ROOT, log, run_checked

MODULE_DIR = os.path.join(ROOT, "bindings", "powershell")
REPOSITORY = os.environ.get("PSGALLERY_REPOSITORY", "PSGallery")

# Prefer the modern PSResourceGet cmdlet, falling back to the legacy
# PowerShellGet one. The key is read from the child environment, so it never
# appears in the command line.
PUBLISH_SCRIPT = f"""
$ErrorActionPreference = 'Stop'
$key = $env:PSGALLERY_API_KEY
if ([string]::IsNullOrEmpty($key)) {{ throw 'PSGALLERY_API_KEY is not set' }}
if (Get-Command Publish-PSResource -ErrorAction SilentlyContinue) {{
    Publish-PSResource -Path '{MODULE_DIR}' -Repository '{REPOSITORY}' -ApiKey $key
}} else {{
    Publish-Module -Path '{MODULE_DIR}' -Repository '{REPOSITORY}' -NuGetApiKey $key
}}
"""


def main():
    push = "--push" in sys.argv

    print("--- Staging the PowerShell payload ---")
    run_checked(
        [sys.executable, os.path.join(ROOT, "scripts", "generate_powershell_bindings.py")],
        cwd=ROOT,
    )

    if not push:
        log("INFO", "dry run - pass --push to publish to the PowerShell Gallery")
        return 0

    if not os.environ.get("PSGALLERY_API_KEY"):
        log("WARNING", "PSGALLERY_API_KEY is not set; skipping push")
        return 0

    print(f"--- Publishing to {REPOSITORY} ---")
    run_checked(
        ["pwsh", "-NoProfile", "-NonInteractive", "-Command", PUBLISH_SCRIPT],
        cwd=ROOT,
    )
    log("SUCCESS", f"published xcelerate to {REPOSITORY}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
