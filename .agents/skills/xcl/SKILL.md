---
name: xcl
description: Write, run, and debug XCL (Xcelerate Command Language) browser-automation scripts — the line-oriented .xcl language for the xcelerate CLI. Use when authoring or fixing .xcl files, choosing XCL verbs, handling variables/functions/cookies/plugins, or running scripts with `xcelerate run`.
---

# Writing XCL scripts

XCL is xcelerate's line-oriented scripting language. One action per line; no
expression tree, no value stack, no string concatenation. Scripts run with:

```bash
xcelerate run script.xcl [--param key=value] [flags]
```

Runs are **quiet**: only `print` output and `fail …` lines show. Add `--verbose`
for a per-step `ok …` transcript while developing.

## Lexical rules (get these right — they cause most errors)

- `# …` at line start is a comment; blank lines are ignored.
- Tokens are whitespace-separated. `"double"` quotes group spaces and process
  `\n \t \" \\`; `'single'` quotes group spaces with **no** escapes (use for
  JSON: `cookie add '[{"name":"x","value":"y"}]'`).
- A token **may be** a reference: `$name` (variable) or `{BUILTIN}`. These are
  substituted at run time.
- **Only a token that is *entirely* a reference is substituted.** There is no
  concatenation and no interpolation inside a larger literal. So `$base/account`
  is looked up as a variable named `base/account` (→ `undefined variable`), and
  `"{BASE_URL}/x"` stays the literal text `{BASE_URL}/x`.
  - Fix: keep the whole string in one variable —
    `let account "https://example.com/account"` then `open $account`.

## Variables and builtins

```text
let base "https://example.com"     # define (read-only by convention)
set n 5                            # reassign
param email "user@example.com"     # runtime parameter, override: --param email=…
```

Builtins: `{BASE_URL}`, `{TIMESTAMP}`, `{UUID}` (must be the whole token).

## Functions

Named, no-return callables. Defined before use; cannot nest; depth ≤ 1.

```text
func login(email, password)
  open "{BASE_URL}/login"
  fill "#email" $email
  fill "#password" $password
  submit
end

login "ada@example.com" "secret"   # bare name, or: `call login …`
```

- Parameters substitute into the body's `$name` references. They do **not**
  inject into `eval <js>` bodies (code-injection boundary).
- **Overloading by arity:** same name with *different* parameter counts is fine;
  an exact duplicate (same name **and** count) is a parse error. A call whose
  argument count matches no overload fails with the arities that do exist.

## Plugins / workers

```text
import acme.mod echo        # load the plugin AND bind `echo` as a callable
echo {"message":"hi"}       # -> run acme.mod echo {"message":"hi"}   (0 or 1 arg)
run acme.mod other {}       # explicit form always works
plugins                     # list loaded workers
plugin-config acme.mod      # dump an op's schema + defaults
```

`import <plugin>` with no ops only gates the plugin. Imported names share the
function namespace, so a name collision with a `func` (or between two plugins)
is a duplicate. Requires `--allow-plugin <id>` (`*` glob allowed).

### Loading a plugin by name

A plugin is a **directory** — `plugin.json` (declaring `ops`, `capabilities`,
`entrypoint`) plus its `.wasm`. Build one, drop it in `./plugins/`, and load it
by name (no path):

```bash
xcelerate plugin new random-app      # scaffold
cd random-app && xcelerate build --wasm-only && cd ..
mv random-app plugins/               # plugins/random-app/{plugin.json,random-app.wasm}

# load by name (searched in $XCELERATE_PLUGIN_DIR, then ./plugins, then .)
xcelerate --plugins random-app run --allow-plugin random-app job.xcl
```
```xcl
import random-app                    # gate it
random-app do-thing "arg"            # call an op (or: run random-app do-thing)
```

A bare `random-app.wasm` cannot be imported by itself: `ops`/`capabilities`
live in the manifest, so the `.wasm` needs `plugin.json` beside it (a
`plugins/random-app/` directory, or a sibling `random-app.json`).

### Importing another XCL file

`import "path.xcl"` includes another script and merges its `func`s (resolved
relative to the importing file). Circular imports and name clashes are errors.

```xcl
import "./lib.xcl"     # brings lib.xcl's funcs into scope
greet world            # call one
```

## Control flow

`repeat <n> <verb> <args…>`, `retry <n> <verb> <args…>`, `if-ok <verb> …`,
`if-fail <verb> …`, `label <name>` / `goto <name>`. Control flow cannot nest;
counts are positive and capped (≤ 1000). Whole-run budget is 10 000 steps.

## Assertions

```text
assert url contains "/account"
assert title == "Dashboard"
assert content contains "Welcome"
assert $STATUS == 200
```

Subjects: `url`, `title`, `content` (HTML), `text` (visible), `status`,
`body`/`response` (after `request`), or any variable. Operators: `==`/`eq`,
`!=`/`ne`, `contains`, `matches` (alias of `contains`), `>`, `<`, `>=`, `<=`.
A failed `assert` fails the **step** (feeds `if-fail`/`retry`) and prints
`fail …`, but does not abort the whole run by itself.

## Common browser verbs

`open`/`goto <url>` · `back` `reload` · `title` `url` `text` `markdown`
`content` · `click <selector|text>` `tap` `mouse [click] <target>|<x> <y>` ·
`fill <sel> <text>` `type` `select <sel> <value>` `press <key>` `submit` ·
`hover` `scroll <px|up|down|top|bottom>` `find <text>` · `wait <ms|selector>`
`wait-sec` `wait-min` `wait-hr` `wait-idle` `wait-stable` `wait-random <min> <max>` ·
`shot` `shot-full` `media` `download <url> <path>` `upload <sel> <path>`
`capture` · `tabs` `new-tab` `switch` · `challenge` (detect anti-bot) ·
`eval <js>` (needs `--allow-unsafe`) · `done` / `quit`.

`click`/`fill` accept a CSS selector, visible text, (interactive session) a
snapshot index. Input is human-like by default; `--linear` makes it
straight-line/fast.

## Drivers — browser and desktop (Windows)

XCL drives everything through **drivers**. The browser is an application too,
driven through **CDP**; any native window is an application driven through **UI
Automation** (the *desktop* driver). Bring a driver in with `import`, and the
**same verbs** act on it — there is no `app-` prefix:

```text
import browser                               # load the browser driver (launch is lazy)
open "https://example.com"

import desktop                               # switch to UI Automation
launch "calc" "Calculator" 20000            # spawn-or-attach
window "Notepad"                             # ...or select a running window
tree                                         # indexed element tree
click "Seven"                                # click by name (pattern-first, cursor-free)
click 43                                     # ...or by tree index
fill 5 "hello"                               # set a value (no cursor)
find "Display is"                            # matching elements
wait "Equals" 5000                           # wait for an element
key next                                     # next/prior/down/up/space/enter
wheel -3                                     # scroll (negative = down)

import browser                               # switch back to the page
```

- **Shared verbs** (follow the active driver): `click` `fill` `find` `wait` `scroll`.
- **Desktop-only** (always a window): `launch` `window` `tree` `key` `wheel`.
- **Browser-only** (always the page): `open` `goto` `title` `url` `text` `press` … .
  `window`/`launch` select the desktop driver; `open`/`goto`/`import browser` select
  the browser.

So `click "Sign in"` hits the page when the browser driver is active, and
`click "Seven"` hits the Calculator button when the desktop driver is active — one
verb, routed by context. One file can do both. The browser is the default driver
and is launched **lazily**, so a native-only script never starts a browser.
`drivers` lists what this run exposes and marks the active one (like `plugins`).

`launch "<exe|uri>" [title] [ms]` is the only way to start an app from a script
(the plugin's `windows` op lists only windows already open). A launched app shows
its window; there is
no hide/headless mode (headless is a *browser* concept, `--headless`). Desktop
**acting** verbs and `launch` need `--allow-app "<glob>"` (default-deny); read-only
`tree`/`find`/`wait` and `window` selection run once granted. `run --native` starts
browserless with the desktop driver active; `run --app "<title>"` attaches to one
window.

Desktop verbs are backed by the **`xcelerate.app` plugin**, so load it and grant
its capability: `--plugins app` and `XCELERATE_PLUGIN_ALLOW=app`. Browser verbs
bind the run's active page so the `xcelerate.browser` plugin can drive the page.

For pure computation with no window at all, use a plugin, `request`, or `eval`.

## Cookies

```text
cookie                                  # all cookies (JSON array)
cookie get sessionid                    # one cookie
cookie set sessionid $value example.com /   # name, value, [domain], [path]
cookie add '[{"name":"sid","value":"x","domain":".example.com","httpOnly":true,"secure":true}]'
cookie delete sessionid
cookie clear
```

Goes through CDP `Network.setCookie`, so `HttpOnly` cookies can be restored.
Omitting the domain scopes to the current page URL (so `open` first). Test
cookie logic against a host you control (e.g. `httpbin.org` or a local server)
before relying on it.

## Security flags (opt-in; default-deny)

- `--allow-unsafe` — enables `eval`.
- `--allow-http` — enables browserless `request <METHOD> <url> [headers] [body]`.
- `--allow-plugin <id>` — enables a plugin (`*` glob); repeatable.
- `--allow-app <glob>` — enables driving/launching a native window (default-deny); repeatable.
- `--allow-private` — permits private/loopback hosts for `request` (SSRF guard
  blocks them by default).

File verbs (`download`, `upload`, `shot`/`shot-full`, `capture`) are confined to
the **workspace root** (the cwd, or `--output-dir <dir>`); absolute paths and
`..` escapes are refused.

## Useful flags

`--param k=v` (override a `param`) · `--user-data-dir <path>` (persist logins /
cookies between runs) · `--output-dir <dir>` (workspace root) · `--proxy <url>`
(repeatable) · `--native` / `--app "<title>"` (Windows: browserless native run) ·
`--verbose`.

## Authoring checklist

1. Prefer `param` + `--param` for values that vary or are secret — don't hardcode.
2. Never split a URL/variable across a `$var` and extra text (no concat).
3. `wait-stable` / `wait-idle` after navigation before asserting.
4. Assert after each navigation so failures point at the right step.
5. Wrap risky steps in `retry`/`if-fail` rather than letting a step fail quietly.
6. To get a stable selector, run the interactive `xcelerate session` and use
   `snapshot`, or read `docs/xcl.md` / `docs/xcl-tutorial.md`.

## Reference

Full spec: `docs/xcl.md`. Tutorial: `docs/xcl-tutorial.md`. Runnable examples:
`examples/*.xcl`.
