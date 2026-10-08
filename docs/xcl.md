# XCL — the Xcelerate Command Language

XCL (`.xcl`) is a small, line-oriented scripting language for authoring
browser-automation runs. It is designed to be **equally comfortable for a human
to read and an AI agent to emit**: one line = one action, no nesting, no
expression sublanguage, and bounded control flow.

> Run a script with `xcelerate run script.xcl`. See the [README](../README.md)
> for the installation and the security (default-deny) flags, and the
> [tutorial](xcl-tutorial.md) for a guided, hands-on walkthrough.

## Quick example

A script is just a list of lines, and each line is one action:

```text
# login.xcl
param base "https://www.practicesoftwaretesting.com"

func fill_field(id, value)
  fill $id $value
end

open $base/auth/register
wait 2000
fill_field "#email" "ada@example.com"
fill_field "#password" "correct-horse-battery"
click "Register"
wait 6000
assert url contains "/login"
done
```

## Commands at a glance

| Keyword | Meaning |
| --- | --- |
| `# comment` | Full-line comment (blank lines ignored). |
| `let` / `set` / `param` | Define, reassign, or declare a variable (`$name`, `--param k=v`). An `=` after the name is optional (`let base = https://example.com`). |
| `func` … `end` / `<name> …` | Define and call a bounded, non-recursive function. |
| `open` / `goto` / `back` / `reload` | Navigate. |
| `title` `url` `text` `markdown` `content` `snapshot` | Read the page. |
| `click` `mouse` `tap` `fill` `select` `type` `press` `submit` `hover` `scroll` | Interact. |
| `wait` `wait-idle` `wait-stable` | Wait for a selector, or sleep a number (`wait` = ms; `wait-sec`/`wait-min`/`wait-hr` for other units). |
| `wait-random` | Sleep a random number of milliseconds between two bounds (`wait-random <min> <max>`, inclusive). |
| `assert <subject> <op> <value>` | Fail-fast check (`url`, `title`, `status`, `contains`, `==`, …). |
| `print <arg>...` | Write the resolved arguments to stdout (the explicit log channel). |
| `repeat` / `retry` / `if-ok` / `if-fail` / `label` / `goto` | Bounded control flow. |
| `eval` / `request` | Opt-in: JavaScript (`--allow-unsafe`), browserless HTTP (`--allow-http`). |
| `import` / `run` / `plugins` / `plugin-config` | Plugins / workers (`--allow-plugin`). |
| `done` / `quit` | End the run. |

When a run ends - the last line, `done`, `quit`, an error, or Ctrl+C - the
browser this command launched is closed. (`--detached` / `--keep-alive` opt out
and let it outlive the process.)

Runs are **quiet by default**: a script prints only `print` output and failures
(`fail <reason>`). Pass `--verbose` to also see an `ok <step>` line for every step.

## Design contract

- **One line, one action.** Sequencing is the line order of the file.
- **No expression language.** There is no arithmetic, no operator precedence,
  no value stack. The only structured value is JSON (plugin args, `request`
  bodies).
- **Bounded control flow.** Loops are `repeat`/`retry` with a hard iteration cap;
  functions are flat and do not recurse. A script cannot hang the runner.
- **Default-deny security.** `eval`, `request`, and plugin `import`/`run` each
  require an explicit flag; the AI cannot self-grant.

## Lexical rules

- A `#` at line-start (after trimming) is a full-line comment. Inline comments
  are not supported — removing the `fill #selector text` ambiguity.
- Verb names are **case-insensitive** (`OPEN`, `Open`, `open` are the same).
  Variables and values are case-sensitive.
- Whitespace runs collapse to one separator.
- Strings: a bare token, `"double quoted"` (backslash escapes `\" \\ \n \t`),
  or `'single quoted'` (raw, no escapes). Only quoted strings may contain spaces.

## Variables and builtins

```text
let base "https://example.com"     # define a variable
set base "https://staging.example" # reassign
param email "user@example.com"     # runtime param with default
# override at run time:  xcelerate run s.xcl --param email=a@b.com
```

* Interpolate with `$name`.
* Builtins: `{BASE_URL}`, `{TIMESTAMP}` (frozen per run), `{UUID}` (unique per
  reference).
* An undefined `$name` is a hard error, never an empty string.

## Functions

Functions are named, parameterized, **no-return** callables. They perform actions;
they never compute a value. They must be defined before use, may not nest, and may
not recurse (depth is at most one).

```text
func register(email, password)
  open "{BASE_URL}/signup"
  fill "#email" $email
  fill "#password" $password
  click "#submit"
end

register "ada@example.com" "hunter2"
```

Invoke a function by its bare name; the `call` keyword is optional (`call register
…` still works). Function parameters substitute into the body's `$name` references at
the call site. Parameters do **not** inject into raw `eval <js>` bodies (that is a
code-injection boundary); use variables explicitly there.

## Control flow

| Keyword | Meaning |
| --- | --- |
| `repeat <n> <verb> <args…>` | Run one statement `n` times (1..=1000). |
| `retry <n> <verb> <args…>` | Repeat a statement until it succeeds, up to `n` times. |
| `if-ok <verb> <args…>` | Run only if the previous statement succeeded. |
| `if-fail <verb> <args…>` | Run only if the previous statement failed. |
| `label <name>` / `goto <name>` | Jump to a label. |

Control flow cannot nest. `repeat`/`retry` counts must be positive integers within
the hard cap. A whole-run `max_steps` budget (10 000) bounds any script.

## HTTP (`request`) — browserless mode

`request <METHOD> <url> [headers] [body]` performs a plain HTTP call with
`reqwest`; no browser is launched for that step. `headers` is a JSON object and
`body` a JSON value or string; both are optional, and an empty `""` stands in
for an omitted positional argument.

```text
request GET  "https://api.example.com/health"
request POST "https://api.example.com/login" {"Content-Type":"application/json"} {"email":"ada","password":"pw"}
assert $STATUS == 200
assert $RESPONSE_BODY contains "ok"
```

Requires `--allow-http`. Private/loopback/link-local/metadata hosts (SSRF
targets) are denied unless `--allow-private`.

## Plugins / workers

| Keyword | Meaning |
| --- | --- |
| `import <id>` | Load a plugin (a "worker"). |
| `plugins` | List loaded workers. |
| `run <plugin> <op> [json]` | Invoke an op; `json` is a JSON argument object. |
| `plugin-config <id> [op]` | Dump an op's JSON schema + defaults. |

Requires `--allow-plugin <id>` (supports `*` glob). A plugin op is a "worker"
task; plugins may call each other through the audited host bridge.

## Assertions

`assert <subject> <op> <value>` fails the step (and thus any enclosing `retry`/
`if-fail`) when the check does not hold.

```text
assert url contains "/dashboard"
assert title == "Accounts"
assert $STATUS == 200
assert content matches "Welcome back"
```

Subjects: `url`, `title`, `content`, `text`, `status`, `body`, or any `$var`.
Operators: `==` `!=` `contains` `matches` `>` `<` `>=` `<=`.

## Browser commands

A script and the interactive session share one verb set: `open`, `goto`, `back`,
`reload`, `title`, `url`, `text`, `markdown`, `content`, `snapshot`, `click`/`tap`,
`mouse`, `fill`, `select`, `type`, `press`/`submit`, `hover`, `scroll`, `find`,
`wait`, `wait-ms`, `wait-sec`, `wait-min`, `wait-hr`, `wait-stable`, `wait-idle`,
`wait-random`, `challenge`, `eval`, `shot`, `shot-full`, `tabs`, `new-tab`,
`switch`, `done`, `quit`.

`click` and `tap` take a snapshot `[index]` (`click 3`), a CSS selector
(`click '#email'`), or visible text (`click "Sign in"`), in that order. Text is
matched against the control's visible text **and** its `aria-label`, so icon-only
buttons are reachable too. `click` moves the real mouse to the target; `tap`
fires a DOM click without moving the mouse (for menus that close on `mouseleave`).

`mouse` moves the real cursor **without clicking** — to a snapshot `[index]`, a
CSS selector, visible text, or raw `mouse <x> <y>` coordinates. Use it to reveal a
hover menu before a click, or to make the cursor travel visibly across the page.
The path is human-like by default (a curved, jittered Bezier); `--linear` switches
to a straight line.

`fill` accepts either a CSS selector or a snapshot `[index]` (`fill 3 "text"`) -
useful when a framework-rendered field exposes no stable selector (the snapshot
then shows the element with no quoted selector).

`select <selector> <value>` chooses an option in a native `<select>` (matched by
value or label). A page that draws its own dropdown (a `div[role="combobox"]` with
a `listbox`) is driven instead by clicking the control, then `click "<option>"` -
the same text match reaches dropdown options.

The interactive session adds a few verbs a script does not need: `click-xy`,
`upload`, `await-human`, and `guard`.

`open` (and `new-tab`) accept either a full URL or a bare host: a target with no
scheme gets `https://` prepended, so `open facebook.com` navigates to
`https://facebook.com`. Targets that already have a scheme (`https://…`,
`about:blank`, `file://…`, `data:…`) are used as-is. As everywhere in XCL the
argument is a **string** - `open facebook.com` and `open "facebook.com"` are the
same value; quotes are only needed when the value contains spaces.

`wait` takes a **number** (milliseconds) or, when the argument is not a number, a
selector (`#id`, `.class`, `//xpath`, `input`, …). For other units use the unit
verb: `wait-sec 2`, `wait-min 1`, `wait-hr 1` (there is no `2s` literal).

For varied timing, `wait-random <min> <max>` sleeps a uniform random hold in
`[min, max]` ms. The bounds are ordinary values, so `$vars` and `func` parameters
work:

```text
wait-random 200 700

func beat(lo, hi)
  wait-random $lo $hi
end
beat 80 400
```

## A complete example

```text
# register-and-verify.xcl
param base "https://www.practicesoftwaretesting.com"
param email "user+{TIMESTAMP}@example.com"
param password "R8!wQp2#zAx4"

func fill_field(field_id, value)
  fill $field_id $value
end

open $base/auth/register
wait 2000

fill_field "#first_name" "Ada"
fill_field "#last_name" "Lovelace"
fill_field "#email" $email
fill_field "#password" $password
click "Register"
wait 6s

retry 10 assert url contains "/login"
done
```
