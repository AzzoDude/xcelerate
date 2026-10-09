---
name: xcelerate-fast
description: Operate xcelerate efficiently — batch a whole automation into one session call, act by name instead of exploratory snapshots, prefer cursor-free UIA patterns, and always clean up (close apps you opened, delete temp files). Use when driving browsers or native apps with xcelerate so agent round-trips and thinking stay minimal.
---

# Driving xcelerate fast

Goal: **one tool call per task**, minimal snapshots, no leftover state, minimal
thinking between actions.

## Rules

1. **Batched — never one action per call.**
   - native (Windows): `xcelerate --plugins app run --native job.xcl` — the `xcelerate.app` plugin
   - browser / Electron / CEF / WebView2: `xcelerate --connect <ws> --attach <id> session < cmds.txt`
   - scripts: `xcelerate run job.xcl` (one invocation runs the whole flow)
   - no subcommand: `xcelerate` alone drops into the interactive session (REPL)
2. **Discover once.** `run xcelerate.app windows` (native windows) or `... targets` (CDP) —
   a single call. Do not re-list.
3. **Act by name, not index.** `click "<text>"` (or `run xcelerate.app click
   {"window":…,"name":"<text>"}`) skips the
   read-tree → pick-index round-trip. Run `tree` **only** when you must see
   structure, and only once.
4. **Prefer cursor-free actions.** `click` / `fill` (and the plugin's `click` /
   `set_value` ops) use UIA
   patterns (no cursor). `wheel` is the last resort, only for a Chromium surface.
5. **Native actions are gated.** Load the plugin and grant its capability
   (`--plugins app`, `XCELERATE_PLUGIN_ALLOW=app`) and allow the window
   (`--allow-app <glob>` matching its title); dangerous chords (`win+r`, `alt+f4`, …) are refused.
6. **Decide the whole sequence up front**, run it, check once, done. Do not
   think between actions.

## Always clean up

- **Close what you opened.** If you launched an app (Store, Explorer, etc.),
  close it when done — `click "Close"` (or the plugin's `click` op), or don't open it at all.
- **Temp files go in `target/`** (gitignored), never the repo root, and are
  **deleted** when the step finishes. Never leave scripts or artifacts behind.
- If a task needs no app UI, don't open one (e.g. `winget`/`git` beats clicking).

## Don'ts

- Don't launch an app you won't act on ("just to look").
- Don't dump a full `tree` (hundreds of lines) to see what's there.
- Don't re-run a step you already know succeeded.
- Don't leave windows open or files behind.

## Cheatsheet

Native (Windows) — the `xcelerate.app` plugin. Load it and grant its capability:
```text
XCELERATE_PLUGIN_ALLOW=app xcelerate --plugins app run --native --allow-app "<glob>" job.xcl
```
```text
# job.xcl — native verbs route through the plugin (no browser needed)
launch "<exe|uri>" [title] [ms]    # start it if not open, then wait
window "<title>"                   # ...or select a running window
tree                               # indexed tree (only if needed)
find "<text>"                      # matching elements only
click "<text>"                     # pattern-first, cursor-free
click <index>
fill <index> "<text>"              # set a value (no cursor)
key next                           # next / prior / down / up / enter
wheel -10                          # Chromium surfaces only
wait "<text>" [ms]                 # wait for an element
```
Or call the plugin's ops directly (JSON args):
```text
run xcelerate.app launch {"target":"calc","title":"Calculator"}
run xcelerate.app windows
run xcelerate.app tree {"window":"Calculator"}
run xcelerate.app click {"window":"Calculator","name":"Equals"}
```

Native and browser **in one XCL script** — same verbs, no `app-` prefix. Drivers:
`import browser` (CDP) and `import desktop` (UIA). `window`/`launch` select the
desktop driver; `open`/`goto`/`import browser` select the browser; shared verbs
(`click` `fill` `find` `wait` `scroll`) follow the active driver. The browser
launches lazily, so a native-only script never starts one.

Native verbs now run the `xcelerate.app` plugin, so load it and grant `app`
(`--plugins app`, `XCELERATE_PLUGIN_ALLOW=app`); browser verbs bind the run's
active page so the `xcelerate.browser` plugin can drive it.
```text
import browser               # load the browser driver (lazy)
open "https://example.com"

import desktop               # switch to UI Automation
launch "<exe|uri>" [title] [ms]   # spawn-or-attach
window "<title>"             # ...or select a running window
tree                         # indexed element tree
find "<text>"                # matching elements
wait "<text>" [ms]           # wait for an element
click <index>                # click by tree index
click "<text>"               # click by name (pattern-first, cursor-free)
fill <index> "<text>"        # set a value (no cursor, no keystrokes)
key next                     # next / prior / down / up / space / enter
wheel <notches>              # scroll
scroll <notches>

import browser               # switch back to the page
```
`launch` is the only way to start an app from a script — the plugin's `windows`
op can only list
windows that are already open. A launched app shows its window (there is no
hide/headless mode for a native app — headless is a *browser* concept). It is gated
too: allow the target or its title (`--allow-app "Calculator*"`). `window
"<title>"` may select a *different* window, but only one that is granted
(`--allow-app`); a script can never widen its own access.

Browser / CDP apps:
```text
xcelerate --connect <ws> targets
xcelerate --connect <ws> --attach <id> session < cmds.txt
xcelerate run job.xcl
```

## Reference

`docs/xcl.md`, `.agents/skills/xcl/SKILL.md`.
