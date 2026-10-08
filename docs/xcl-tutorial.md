# XCL tutorial — from your first script to a real run

A guided, hands-on tour of **XCL**, the line-oriented scripting language built
into Xcelerate. By the end you will have run a script, read a page, driven a form,
used variables, functions, control flow and assertions, and reached for HTTP and
plugins — safely.

This is the tutorial. For the terse language reference see
[`xcl.md`](xcl.md); for the CLI, MCP server, and bindings see the
[README](../README.md).

## What XCL is

- **One line, one action.** Sequencing is the order of the lines.
- **No expression language.** No arithmetic, no operator precedence, no value
  stack — the only structured value is JSON (plugin args, `request` bodies).
- **Bounded.** Loops are capped, functions are flat and never recurse, and a
  whole-run step budget applies. A script cannot hang the runner.
- **Default-deny.** The risky verbs (`eval`, `request`, plugin `import`/`run`)
  need an explicit flag from the human who runs the script.

By default a run is **quiet**: it prints only `print` output and stops (printing
`fail <reason>`) when a step fails — unless you catch the failure with `retry` or
`if-fail`. Pass `--verbose` to also see an `ok <step>` line for every step.

## 0. Install

```bash
cargo install --path crates/xcelerate-cli   # from a checkout (binary: xcelerate-cli)
winget install Chaosware.Xcelerate          # Windows (command: xcelerate)
```

> The release archives and winget install the command as `xcelerate`; a local
> `cargo install` produces the same program named `xcelerate-cli` (the Cargo bin
> target avoids an output-filename collision with the core `xcelerate` library).

You need Chrome or Edge installed; Xcelerate finds it automatically. Confirm it
works:

```bash
xcelerate title https://example.com
```

## 1. Your first script

Create `hello.xcl`:

```text
open https://example.com
title
markdown
done
```

Run it:

```bash
xcelerate run hello.xcl
```

You will see one line of output per action:

```text
ok open https://example.com
ok Example Domain
ok # Example Domain

...
ok done
```

`title` prints the page title; `markdown` prints the main content as clean
Markdown. That is the whole shape of XCL: a list of actions, top to bottom.

## 2. Reading the page

| Verb | What it prints |
| --- | --- |
| `title` | The page title. |
| `url` | The current URL. |
| `text` | The visible text. |
| `markdown` | The main content as clean Markdown (scripts/nav stripped). |
| `content` | The full HTML. |
| `snapshot` | An indexed, agent-friendly view of the interactive elements. |
| `eval <js>` | The JSON result of a JavaScript expression (needs `--allow-unsafe`). |

`snapshot` is the one to reach for when you need to *act* on a page. It tags
every interactive element with a stable index, and where it can derive a usable
CSS selector from the element's DOM attributes (`#id`, `[name="…"]`, …) it shows
that too:

```text
open https://www.practicesoftwaretesting.com
wait-idle
snapshot
done
```

```text
ok snapshot
[0]<a> "Home"  '#home'
[1]<a> "Categories"  '[aria-label="Categories"]'
[2]<button> "Sign in"  '#signin'
```

Selectors are shown as quoted string literals so you can copy them straight into a
command (`fill '#email' …`).

Framework-generated ids (React's `_R_…`, Radix/MUI `:r0:`, long random tokens)
are unstable between renders, so they are skipped and a stable
`name`/`aria-label`/`placeholder` is used instead - or no selector is shown, in
which case use the index.

## 3. Acting on a page

| Verb | Action |
| --- | --- |
| `click <index\|selector\|text>` | Click by snapshot index (`click 2`), CSS selector, or visible text (`click "Sign in"`). |
| `tap <selector\|text>` | The same pick, but a DOM click that never moves the mouse. |
| `fill <selector|index> <text>` | Type text into a field, by CSS selector or snapshot index. |
| `select <selector> <value>` | Choose an option in a native `<select>` (by value or label). |
| `type <text>` | Type into the already-focused element. |
| `press <key>` | Press a key on the focused element (`press Enter`). |
| `submit` | Press Enter on the focused element — the productive way to finish a form. |
| `hover <selector>` | Move the mouse over an element. |
| `mouse <index\|selector\|text>` | Move the cursor there without clicking (or `mouse <x> <y>`). |
| `scroll <pixels\|up\|down\|top\|bottom>` | Scroll the page. |

`click` takes an index, a selector, or visible text, so every style works:

```text
open https://www.practicesoftwaretesting.com
wait-idle

click "Sign in"
wait "#email"
fill "#email" "customer@practicesoftwaretesting.com"
fill "#password" "welcome01"
submit
wait-stable

assert url contains "/account"
done
```

The `wait "#email"` line waits for the field before filling it (see the next
section), and `submit` presses Enter from the field you are already in.

## 4. Waiting for the page

Fixed sleeps are the root of flaky automation. XCL offers three waits:

| Verb | Waits for |
| --- | --- |
| `wait <ms>` | A fixed time in milliseconds; `wait-sec` / `wait-min` / `wait-hr` for other units. |
| `wait <selector>` | A selector to appear (`wait "#results"`). |
| `wait-idle` | The network to go quiet. |
| `wait-stable` | The DOM to stop mutating. |

```text
open https://example.com
wait 2000
wait-idle
click "More information..."
wait-stable
```

Prefer `wait-idle` / `wait-stable` (or a selector wait) over a fixed `wait`
whenever you can — they stop as soon as the page is ready.

## 5. Variables and parameters

```text
let base "https://www.practicesoftwaretesting.com"
set base "https://staging.practicesoftwaretesting.com"

param email "user@example.com"
param password "correct-horse-battery"

open $base/auth/login
fill "#email" $email
fill "#password" $password
submit
```

- `let` defines a variable, `set` reassigns it, and `param` declares a runtime
  parameter. Interpolate with `$name`. An `=` after the name is optional, so
  `let base = https://example.com` and `let base https://example.com` are the
  same.
- An **undefined** `$name` is a hard error — never a silent empty string.
- Builtins: `{BASE_URL}` (from the `XCELERATE_BASE_URL` environment variable),
  `{TIMESTAMP}` (frozen once per run), and `{UUID}` (unique per mention).

For varied timing, use `wait-random <min> <max>`:

```text
wait-random 200 700
```

Override a `param` from the command line:

```bash
xcelerate run login.xcl --param email=ada@example.com
xcelerate run login.xcl --param base=https://staging.app
```

`--param` wins over the script default. A `param` with no default and no
`--param` fails with `missing required param`.

## 6. Functions

A function is a named, **no-return** callable: it performs actions, never computes
a value, and cannot recurse.

```text
func login(email, password)
  open "{BASE_URL}/auth/login"
  fill "#email" $email
  fill "#password" $password
  submit
  wait-stable
end

login "ada@example.com" "correct-horse-battery"
```

Invoke a function by its bare name; the `call` keyword is optional
(`call login "…" "…"` also works). Parameters substitute into the body's `$name`
references. Functions must be defined before use and cannot nest.

## 7. Control flow

Control flow is bounded and cannot nest.

| Keyword | Meaning |
| --- | --- |
| `repeat <n> <verb> <args…>` | Run one statement `n` times (1..=1000). |
| `retry <n> <verb> <args…>` | Repeat a statement until it succeeds. |
| `if-ok <verb> <args…>` | Run only if the previous statement succeeded. |
| `if-fail <verb> <args…>` | Run only if the previous statement failed. |
| `label <name>` / `goto <name>` | Jump to a label. |

`retry` is the flake-killer — it turns a race into a wait:

```text
open $base
click "Sign in"
retry 10 wait "#email"
fill "#email" $email
```

A whole-run `max_steps` budget (10 000) bounds any script.

## 8. Assertions

`assert <subject> <op> <value>` stops the run at the offending line when a check
fails. This is what makes a script trustworthy: it fails loudly instead of
silently doing the wrong thing.

```text
assert url contains "/account"
assert title == "My account"
assert content matches "Welcome back"
assert $STATUS == 200
```

- Subjects: `url`, `title`, `content`, `text`, `status`, `body`, or any `$var`.
- Operators: `==`, `!=`, `contains`, `matches`, `>`, `<`, `>=`, `<=`.

## 9. Capturing output

```text
open https://example.com
shot home.png           # viewport PNG
shot-full home-full.png # whole page
find "Example"          # how many elements contain the text
challenge               # report anti-bot markers as JSON (detection only)
```

`challenge` never tries to *defeat* a control — it reports reCAPTCHA, hCaptcha,
Turnstile, DataDome and friends so a human can decide. In the interactive session
you can hand off with `await-human`.

## 10. Browserless HTTP

`request` performs a plain HTTP call with no browser for that step. It is
**off by default**:

```bash
xcelerate run api.xcl --allow-http
```

```text
request GET "{BASE_URL}/api/health"
assert $STATUS == 200
assert $RESPONSE_BODY contains "ok"

request POST "{BASE_URL}/api/login" {"Content-Type":"application/json"} {"email":"ada","password":"pw"}
assert $STATUS == 200
```

- `request <METHOD> <url> [headers] [body]`; `headers` is a JSON object, `body` a
  JSON value or string.
- The result lands in `$STATUS` and `$RESPONSE_BODY`.
- Private/loopback/link-local/metadata hosts are denied unless you pass
  `--allow-private` (the SSRF guard).

## 11. Plugins (workers)

```text
plugins                       # list loaded workers
import example.echo           # load a worker
run example.echo info {}      # invoke an op with JSON args
plugin-config example.echo    # show an op's JSON schema + defaults
```

Plugin loading is **off by default**:

```bash
xcelerate run plugins.xcl --allow-plugin example.echo --allow-plugin 'acme.*'
```

`--allow-plugin` accepts `*` globs. A plugin op behaves like a worker; plugins
call each other only through the audited host bridge. See the
[plugin authoring guide](plugins/README.md) to build your own.

## 12. A complete run

Putting it together — register a user, then log in:

```text
# register-then-login.xcl
param base "https://www.practicesoftwaretesting.com"
param email "user+{TIMESTAMP}@example.com"
param password "R8!wQp2#zAx4"

func fill_field(field_id, value)
  fill $field_id $value
end

open $base/auth/register
wait-idle

fill_field "#first_name" "Ada"
fill_field "#last_name" "Lovelace"
fill_field "#email" $email
fill_field "#password" $password
submit
wait-stable

assert url contains "/login"

fill_field "#email" $email
fill_field "#password" $password
submit
wait-stable

assert url contains "/account"
done
```

```bash
xcelerate run register-then-login.xcl
xcelerate run register-then-login.xcl --param password=Secret123!
```

## 13. Security, in one place

XCL is executable input (often written by an AI), so every risky capability is
**default-deny** and the flags must come from the invoking human:

| Surface | Flag |
| --- | --- |
| `eval <js>` | `--allow-unsafe` |
| `request` (browserless HTTP) | `--allow-http` |
| `import` / `run` plugin | `--allow-plugin <id>` |
| Private/metadata hosts in `request` | `--allow-private` |

There is no way for a script or an agent to grant itself a flag. `--allow-unsafe`
also implies `--allow-http`.

## 14. Debugging and tips

- Every line is reported: read the `ok` / `fail` output top to bottom to find the
  first wrong step.
- `fail` stops the run. Wrap a step in `retry` / `if-fail` when failure is
  expected.
- Insert `snapshot`, `markdown`, `title`, or `url` to see what the script sees.
- Prefer `snapshot` + `click <index>` over long CSS selectors for robustness.
- Prefer `wait-idle` / `wait-stable` over fixed `wait`s.
- For a human pace, use `wait-random <min> <max>` between steps and prefer
  `click` (real mouse) over `tap` (instant DOM click). Human-like
  typing and mouse travel are built in; pass `--linear` for a straight-line,
  fast-input run. See [`examples/human-login.xcl`](../examples/human-login.xcl).
- Keep scripts small and linear — XCL is deliberately not a real programming
  language, and that is the point.

## 15. Interactive modes

The same verbs work live, so you can explore before you write a script:

```bash
xcelerate session --start https://example.com   # one browser, commands on stdin
xcelerate live --start https://example.com      # a visible window a human can step into
```

## Where to go next

- [`xcl.md`](xcl.md) — the full language reference.
- [`plugins/README.md`](plugins/README.md) — write your own plugin.
- [README](../README.md) — the CLI, MCP server, and the eleven language bindings.
