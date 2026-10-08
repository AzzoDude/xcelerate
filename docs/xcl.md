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
| `title` `url` `text` `markdown` `content` | Read the page. |
| `click` `mouse` `tap` `fill` `select` `type` `press` `submit` `hover` `scroll` | Interact. |
| `media` `download <url> <path>` `upload <sel> <path>` | Media: discover, fetch, or send files. |
| `wait` `wait-idle` `wait-stable` | Wait for a selector, or sleep a number (`wait` = ms; `wait-sec`/`wait-min`/`wait-hr` for other units). |
| `wait-random` | Sleep a random number of milliseconds between two bounds (`wait-random <min> <max>`, inclusive). |
| `assert <subject> <op> <value>` | Fail-fast check (`url`, `title`, `status`, `contains`, `==`, …). |
| `print <arg>...` | Write the resolved arguments to stdout (the explicit log channel). |
| `repeat` / `retry` / `if-ok` / `if-fail` / `label` / `goto` | Bounded control flow. |
| `eval` / `request` | Opt-in: JavaScript (`--allow-unsafe`), browserless HTTP (`--allow-http`). |
| `import <id> [op]…` / `run` / `plugins` / `plugin-config` | Plugins / workers (`--allow-plugin`); each imported `op` becomes a bare callable. |
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
- **Confined file access.** Every path (`download`, `capture`, `shot`, `upload`)
  resolves inside one workspace root; absolute paths and `..` escapes are refused.

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

### Overloading by arity

A name may be **overloaded by parameter count**: two functions with the same name
but different arities coexist, and the call site picks the overload by how many
arguments it passes.

```text
func human(a, b)
  print "two"
end
func human(a, b, c, d)
  print "four"
end

human 1 2          # the two-parameter overload
human 1 2 3 4      # the four-parameter overload
```

An **exact duplicate** - same name *and* same arity - is rejected at parse time, as
is a call whose argument count matches no overload (the error names the arities
that exist).

### Imported ops as callables

`import <plugin> <op>...` binds each named op as a bare callable, so an imported
op is invoked like a function instead of through `run`:

```text
import acme.mod echo

echo                      # -> run acme.mod echo
echo {"message":"hi"}     # -> run acme.mod echo {"message":"hi"}
```

An imported op is callable with **zero** arguments (no payload) or **one** (a JSON
payload). Imported names share the function namespace, so a collision with a
`func` - or two plugins exporting the same op name - is a duplicate and fails at
parse time. `import <plugin>` with no ops listed only gates the plugin (no names
are bound), and `run <plugin> <op> [json]` always works regardless.

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
| `import <id> [op...]` | Load a plugin (a "worker"); each named `op` becomes a bare callable. |
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
`reload`, `title`, `url`, `text`, `markdown`, `content`, `click`/`tap`,
`mouse`, `fill`, `select`, `type`, `press`/`submit`, `hover`, `scroll`, `find`,
`media`, `download`, `upload`, `capture`, `cookie`,
`wait`, `wait-ms`, `wait-sec`, `wait-min`, `wait-hr`, `wait-stable`, `wait-idle`,
`wait-random`, `challenge`, `eval`, `shot`, `shot-full`, `tabs`, `new-tab`,
`switch`, `done`, `quit`.

`click` and `tap` take a CSS selector (`click '#email'`) or visible text
(`click "Sign in"`). Text is matched against the control's visible text **and**
its `aria-label`, so icon-only buttons are reachable too. `click` moves the real
mouse to the target; `tap` fires a DOM click without moving the mouse (for menus
that close on `mouseleave`). A snapshot `[index]` (`click 3`) is a *session*
concept - scripts act by selector or text, since XCL has no `snapshot` verb.

`mouse` moves the real cursor **without clicking** — to a CSS selector, visible
text, or raw `mouse <x> <y>` coordinates. Use it to reveal a hover menu before a
click, or to make the cursor travel visibly across the page.
`mouse click <target>` moves the cursor there and then clicks — the same thing
`click` does, spelled with the `mouse` verb.
The path is human-like by default (a curved, jittered Bezier); `--linear` switches
to a straight line.

`fill` takes a CSS selector (`fill '#email' "ada@example.com"`).

`select <selector> <value>` chooses an option in a native `<select>` (matched by
value or label). A page that draws its own dropdown (a `div[role="combobox"]` with
a `listbox`) is driven instead by clicking the control, then `click "<option>"` -
the same text match reaches dropdown options.

The interactive session adds a few verbs a script does not need: `click-xy`,
`await-human`, and `guard`.

## Media and files

`media` lists the media the page references — `<video>`/`<audio>` sources,
`<img>` (with `srcset`), and CSS background images — as JSON with absolute URLs:

```text
media
# {"count":1,"media":[{"kind":"image","url":"https://…/logo.png","width":580,"height":164}]}
```

`download <url> <path>` fetches a URL **through the browser** and streams it to a
file. Because the request is made by the browser, the page's cookies and headers
apply and cross-origin media is reachable (CORS does not apply):

```text
download "https://www.python.org/static/img/python-logo.png" "logo.png"
```

When the URL is an **HLS playlist** (`.m3u8`) or a **DASH manifest** (`.mpd`),
`download` assembles the stream from its segments instead — natively, with no
external tool. HLS follows the master to its best variant; DASH picks the best
video and audio representations (a stream with separate audio writes
`<name>.audio.m4a` beside the target):

```text
download "https://test-streams.mux.dev/x36xhzz/x36xhzz.m3u8" "movie.ts"
download "https://…/manifest.mpd" "movie.mp4"
```

The output is the concatenated segments (MPEG-TS or fragmented MP4), which any
player opens. The CLI one-shot `xcelerate grab <url> -o out` does the same.

`upload <selector> <path>` sets a `<input type="file">` to a local file (via CDP
`DOM.setFileInputFiles`; page JS cannot do this).

`capture <url> <path> [seconds]` opens a page, lets it play for `seconds`
(default 15), and reassembles the media the player fetches itself — the segments
behind a `blob:` URL on a Media Source Extensions page (YouTube, Facebook). It is
the scriptable form of `xcelerate capture`.

**The honest limit:** a page that plays video through Media Source Extensions
exposes the `<video>` element only a `blob:` URL, so `media` reports
`{"streaming": true}` and there is no file URL to fetch. HLS and DASH name their
segments as URLs, so `download` can assemble those. For an MSE page (YouTube,
Facebook), `xcelerate capture <page> -o out.mp4` records the segment requests the
player makes — the already-signed URLs — and reassembles them, so no external tool
is needed; the page must actually play. Encrypted HLS (`EXT-X-KEY`, AES-128) and
DRM (Widevine/PlayReady) are refused rather than silently corrupted.

### Where files go

Every path in `download`, `capture`, `shot`/`shot-full`, and `upload` is
resolved against a **workspace root** and confined to it. The root is the
directory you run from, or the `--output-dir <DIR>` you pass to the CLI. A
script cannot name an absolute path (`/etc/x`, `C:\Windows\x`) or climb out with
`..`; those steps fail with `path escapes the workspace root`, and a symlinked
directory inside the root that resolves outside it is refused too. Parent
directories are created as needed, so `shot "reports/a.png"` makes `reports/`.
This keeps a `.xcl` file — which is executable input — from reading or writing
anything outside the workspace you chose.

## Cookies

`cookie` reads and writes the browser's cookies through CDP, so it can restore
`HttpOnly` session cookies — which `document.cookie` (and `eval`) cannot write:

```text
cookie                                        # all cookies as a JSON array
cookie get sessionid                          # one cookie by name (JSON, or null)
cookie set sessionid "$SESSION" example.com    # name, value, [domain], [path]
cookie add '[{"name":"sessionid","value":"…","domain":".example.com","httpOnly":true,"secure":true}]'
cookie delete sessionid                        # remove one cookie
cookie clear                                   # remove every cookie
```

`cookie set` falls back to the current page's URL when no domain is given, and
defaults `path` to `/`. Use `cookie add` with a JSON object (or array) when you
need full attributes (`httpOnly`, `secure`, `sameSite`, `expires`). Pair this with
a persistent profile (`--user-data-dir`) to keep a login between runs.

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
