#!/usr/bin/env python3
"""Inspect and validate the adapter profiles.

Two jobs in one place, because both walk ``adapters/profiles/*.json``:

* ``harvest`` (default) - introspect the installed Selenium / Playwright /
  Puppeteer libraries, compare their public members against the profiles, and
  optionally snapshot or stub the gaps.
* ``--check`` - validate the profiles against ``adapters/runtime.py`` and the
  Rust op table, print a coverage report, and exit non-zero on any problem (so
  it can gate CI). No libraries need to be installed for this.

Usage::

    python scripts/harvest_adapters.py                     # coverage report
    python scripts/harvest_adapters.py --write             # write discovered/*.json
    python scripts/harvest_adapters.py --update            # append missing stubs
    python scripts/harvest_adapters.py --write --update
    python scripts/harvest_adapters.py --check             # validate (CI gate)
    python scripts/harvest_adapters.py --from-json <path> --update
"""

from __future__ import annotations

import argparse
import collections
import importlib
import inspect
import json
import os
import sys

from common import (
    DISCOVERED_DIR,
    PROFILES_DIR,
    ROOT,
    RUNTIME_SRC,
    load_profile,
    load_profiles,
    log,
    safe_name,
)

# profile name -> [(module, [class names])]
TARGETS = {
    "playwright": [("playwright.async_api", ["Browser", "BrowserContext", "Page", "Locator"])],
    "selenium": [
        ("selenium.webdriver.remote.webdriver", ["WebDriver"]),
        ("selenium.webdriver.remote.webelement", ["WebElement"]),
    ],
    "puppeteer": [
        ("pyppeteer.browser", ["Browser"]),
        ("pyppeteer.page", ["Page"]),
        ("pyppeteer.element_handle", ["ElementHandle"]),
    ],
}

IGNORED = {"__init__", "__class__", "__module__", "__doc__", "__dict__", "__weakref__"}

STUB_REASON = "not yet implemented in xcelerate"


# ---------------------------------------------------------------------------
# Harvest
# ---------------------------------------------------------------------------

def public_members(cls):
    """Return [{name, property}] for the public members of a class."""
    members = []
    for name in dir(cls):
        if name.startswith("_") or name in IGNORED:
            continue
        try:
            member = inspect.getattr_static(cls, name)
        except AttributeError:
            continue
        is_property = isinstance(member, property)
        if is_property or callable(member):
            members.append({"name": name, "property": is_property})
    return sorted(members, key=lambda m: m["name"])


def harvest():
    discovered = {}
    for profile_name, modules in TARGETS.items():
        found = {}
        for module_name, class_names in modules:
            try:
                module = importlib.import_module(module_name)
            except ImportError:
                log("HARVEST", f"{profile_name}: '{module_name}' not installed, skipping")
                continue
            for class_name in class_names:
                cls = getattr(module, class_name, None)
                if cls is None:
                    log("HARVEST", f"{profile_name}: class {module_name}.{class_name} not found")
                    continue
                found[class_name] = public_members(cls)
        if found:
            discovered[profile_name] = {"classes": found}
    return discovered


def load_discovered(paths):
    """Load pre-harvested JSON files (e.g. from the Puppeteer JS introspector)."""
    discovered = {}
    for path in paths:
        with open(path, "r", encoding="utf-8") as handle:
            data = json.load(handle)
        name = data.get("name") or os.path.splitext(os.path.basename(path))[0]
        discovered[name] = {"classes": data["classes"]}
        log("HARVEST", f"loaded {name} from {os.path.relpath(path, ROOT)}")
    return discovered


def profile_names(methods):
    # Match on the library-facing name (which is what the harvester discovers),
    # not the emitted name (`as`), which may differ for reserved identifiers.
    return {m["name"] for m in methods}


def report(discovered):
    for profile_name, data in discovered.items():
        profile = load_profile(profile_name)
        print(f"\n=== {profile_name} ===")
        if profile is None:
            print("  no profile found")
            continue
        classes = profile.get("classes", {})
        for class_name, members in data["classes"].items():
            if class_name not in classes:
                print(f"  {class_name}: NOT in profile ({len(members)} members)")
                continue
            mapped = profile_names(classes[class_name].get("methods", []))
            missing = [m["name"] for m in members if m["name"] not in mapped]
            print(f"  {class_name}: {len(members)} members, {len(members) - len(missing)} mapped")
            if missing:
                print(f"    unmapped ({len(missing)}): {', '.join(missing)}")


def stub_for(member):
    entry = {"name": member["name"], "unsupported": STUB_REASON}
    if member.get("property"):
        entry["property"] = True
    emitted = safe_name(member["name"])
    if emitted != member["name"]:
        entry["as"] = emitted
    return entry


def update_profiles(discovered):
    for profile_name, data in discovered.items():
        profile = load_profile(profile_name)
        if profile is None:
            continue
        classes = profile.setdefault("classes", {})
        changed = 0
        for class_name, members in data["classes"].items():
            if class_name not in classes:
                classes[class_name] = {
                    "doc": f"{class_name} adapter (surface only; not obtainable from this runtime yet).",
                    "methods": [],
                }
            spec = classes[class_name]
            spec.setdefault("methods", [])
            mapped = profile_names(spec["methods"])
            for member in members:
                if member["name"] in mapped:
                    continue
                spec["methods"].append(stub_for(member))
                changed += 1
        if changed:
            with open(os.path.join(PROFILES_DIR, f"{profile_name}.json"), "w", encoding="utf-8") as handle:
                json.dump(profile, handle, indent=2)
                handle.write("\n")
            log("UPDATE", f"{profile_name}: added {changed} stubs")


# ---------------------------------------------------------------------------
# Check (validation + coverage)
# ---------------------------------------------------------------------------

def _emitted_names(spec):
    names = []
    for method in spec.get("methods", []):
        if not method.get("unsupported"):
            names.append(method.get("as") or method["name"])
    return names


def check():
    """Validate profiles against the runtime and the Rust op table."""
    with open(RUNTIME_SRC, "r", encoding="utf-8") as handle:
        runtime = handle.read()

    try:
        import generate_adapters

        rust_ops = set(generate_adapters.OPS)
    except Exception as exc:  # pragma: no cover - import is local and reliable
        log("CHECK", f"could not load Rust op table: {exc}")
        rust_ops = None

    problems = []
    profiles = load_profiles()
    if not profiles:
        log("CHECK", "no profiles found")
        return 1

    grand_total = grand_supported = 0
    for profile in profiles:
        name = profile["name"]

        ops = {h["op"] for h in profile.get("helpers", [])}
        for spec in profile.get("classes", {}).values():
            ops |= {m["op"] for m in spec.get("methods", []) if "op" in m}
        missing = sorted(op for op in ops if f"def {op}(" not in runtime)
        if missing:
            problems.append(f"{name}: missing Python ops {missing}")
        if rust_ops is not None:
            missing_rust = sorted(op for op in ops if op not in rust_ops)
            if missing_rust:
                problems.append(f"{name}: missing Rust ops {missing_rust}")

        print(f"\n=== {name} ===")
        total = supported = 0
        for class_name, spec in profile.get("classes", {}).items():
            count = len(spec.get("methods", []))
            sup = sum(1 for m in spec.get("methods", []) if "op" in m or "impl" in m)
            total += count
            supported += sup
            duplicates = [
                n for n, k in collections.Counter(_emitted_names(spec)).items() if k > 1
            ]
            if duplicates:
                problems.append(f"{name}.{class_name}: duplicate methods {duplicates}")
            print(f"  {class_name:16} total={count:3}  supported={sup:3}  stubs={count - sup:3}")
        print(f"  {'TOTAL':16} total={total:3}  supported={supported:3}  stubs={total - supported:3}")
        grand_total += total
        grand_supported += supported

    print(
        f"\n[CHECK] {grand_total} methods across {len(profiles)} profiles "
        f"({grand_supported} supported, {grand_total - grand_supported} stubs)"
    )

    if problems:
        print("[CHECK] problems:")
        for problem in problems:
            print(f"  - {problem}")
        return 1
    log("CHECK", "ok")
    return 0


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def main():
    parser = argparse.ArgumentParser(description="Harvest and validate adapter profiles.")
    parser.add_argument("--write", action="store_true", help="write adapters/discovered/<name>.json")
    parser.add_argument("--update", action="store_true", help="append missing methods as stubs")
    parser.add_argument(
        "--from-json",
        action="append",
        default=[],
        metavar="PATH",
        help="merge pre-harvested JSON (same shape) instead of importing; repeatable",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="validate profiles + print coverage (no libraries required)",
    )
    args = parser.parse_args()

    if args.check:
        return check()

    discovered = load_discovered(args.from_json) if args.from_json else harvest()
    if not discovered:
        log("HARVEST", "nothing harvested (are the libraries installed?)")
        return 0

    if args.write:
        os.makedirs(DISCOVERED_DIR, exist_ok=True)
        for name, data in discovered.items():
            path = os.path.join(DISCOVERED_DIR, f"{name}.json")
            with open(path, "w", encoding="utf-8") as handle:
                json.dump(data, handle, indent=2, sort_keys=True)
            log("HARVEST", f"wrote {os.path.relpath(path, ROOT)}")

    if args.update:
        update_profiles(discovered)

    report(discovered)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
