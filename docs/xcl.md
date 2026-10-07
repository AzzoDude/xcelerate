# XCL — the Xcelerate Command Language

XCL (`.xcl`) is a small, line-oriented scripting language for authoring
browser-automation runs. It is designed to be **equally comfortable for a human
to read and an AI agent to emit**: one line = one action, no nesting, no
expression sublanguage, and bounded control flow.

> Run a script with `xcelerate run script.xcl`. See the [README](../README.md)
> for the installation and the security (MITRE ATT&CK) flags.

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

Any session command works verbatim in a script: `open`, `goto`, `back`,
`reload`, `title`, `url`, `text`, `markdown`, `content`, `snapshot`, `click`,
`click-text`, `tap`, `tap-text`, `click-xy`, `upload`, `fill`, `type`, `press`,
`submit`, `hover`, `scroll`, `find`, `wait`, `wait-stable`, `wait-idle`,
`challenge`, `await-human`, `eval`, `guard`, `shot`, `shot-full`, `done`, `quit`.

`wait` resolves its argument as: a selector (`#id`, `.class`, `//xpath`, `input`,
…), else an integer (0–60 = seconds, >60 = milliseconds), or an explicitly
suffixed `5s` / `500ms`.

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
wait 2s

fill_field "#first_name" "Ada"
fill_field "#last_name" "Lovelace"
fill_field "#email" $email
fill_field "#password" $password
click-text "Register"
wait 6s

retry 10 assert url contains "/login"
done
```
