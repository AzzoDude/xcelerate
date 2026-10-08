---
name: xcelerate-fast
description: Operate xcelerate efficiently — batch a whole automation into one session call, act by name instead of exploratory snapshots, prefer cursor-free UIA patterns, and always clean up (close apps you opened, delete temp files). Use when driving browsers or native apps with xcelerate so agent round-trips and thinking stay minimal.
---

# Driving xcelerate fast

Goal: **one tool call per task**, minimal snapshots, no leftover state, minimal
thinking between actions.

## Rules

1. **Session-first, batched — never one action per call.**
   Write a command file and pipe it in; the session keeps the process, the UIA
   tree, and the app attach alive, so *N* actions cost **one** setup.
   - native: `xcelerate-uia session "<window>" < cmds.txt`
   - browser / Electron / CEF / WebView2: `xcelerate --connect <ws> --attach <id> session < cmds.txt`
   - scripts: `xcelerate run job.xcl` (one invocation runs the whole flow)
2. **Discover once.** `xcelerate apps` (native windows) or `... targets` (CDP) —
   a single call. Do not re-list.
3. **Act by name, not index.** `click-name` skips the
   read-tree → pick-index round-trip. Run `tree` **only** when you must see
   structure, and only once.
4. **Prefer cursor-free actions.** `invoke` / `click` / `set-value` use UIA
   patterns (no cursor). `wheel` is the last resort, only for a Chromium surface.
5. **Wait inside the tool.** Put repeats/waits in the script; never poll with
   separate calls.
6. **Decide the whole sequence up front**, run it, check once, done. Do not
   think between actions.

## Always clean up

- **Close what you opened.** If you launched an app (Store, Explorer, etc.),
  close it when done — `click-name "<title>" "Close"`, or don't open it at all.
- **Temp files go in `target/`** (gitignored), never the repo root, and are
  **deleted** when the step finishes. Never leave scripts or artifacts behind.
- If a task needs no app UI, don't open one (e.g. `winget`/`git` beats clicking).

## Don'ts

- Don't launch an app you won't act on ("just to look").
- Don't dump a full `tree` (hundreds of lines) to see what's there.
- Don't re-run a step you already know succeeded.
- Don't leave windows open or files behind.

## Cheatsheet

Native (`xcelerate-uia`):
```text
xcelerate-uia apps                          # list windows: name [class] pid
xcelerate-uia tree "<title>"                # indexed tree (only if needed)
xcelerate-uia click-name "<title>" "<text>" # pattern-first, cursor-free
xcelerate-uia set-value "<title>" <index> "<text>"
xcelerate-uia key "<title>" next            # page down / prior / down / up
xcelerate-uia wheel "<title>" -10           # Chromium surfaces only
xcelerate-uia session "<title>" < cmds.txt
```

Browser / CDP apps:
```text
xcelerate --connect <ws> targets
xcelerate --connect <ws> --attach <id> session < cmds.txt
xcelerate run job.xcl
```

## Reference

`docs/xcl.md`, `.agents/skills/xcl/SKILL.md`.
