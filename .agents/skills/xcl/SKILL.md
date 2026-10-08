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
- `--allow-private` — permits private/loopback hosts for `request` (SSRF guard
  blocks them by default).

File verbs (`download`, `upload`, `shot`/`shot-full`, `capture`) are confined to
the **workspace root** (the cwd, or `--output-dir <dir>`); absolute paths and
`..` escapes are refused.

## Useful flags

`--param k=v` (override a `param`) · `--user-data-dir <path>` (persist logins /
cookies between runs) · `--output-dir <dir>` (workspace root) · `--proxy <url>`
(repeatable) · `--verbose`.

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
