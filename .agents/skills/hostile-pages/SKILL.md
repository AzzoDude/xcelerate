---
name: hostile-pages
description: Handle disruptive and hostile web pages while driving a browser with xcelerate — ad interstitials, popups and new tabs, JavaScript dialogs, cookie/consent walls, navigation traps and anti-bot challenges. Use when a page spawns extra tabs, hijacks the viewport with ads or modals, redirects unexpectedly, blocks clicks, or otherwise breaks the automation step.
---

# Surviving hostile pages

Real sites fight automation. This skill is the playbook for the ways a page
disrupts a run, and the concrete xcelerate primitives that neutralise each one.

## When to use this

Load this skill when any of these happen mid-run:

- a second tab / window appears on its own, or a click opens `target="_blank"`;
- a full-screen **ad** or "sponsor" interstitial covers the element you need;
- a cookie/consent wall or newsletter modal blocks the page;
- clicks or typing silently do nothing (an overlay is eating them);
- the page redirects away, or a "are you sure you want to leave?" prompt freezes
  teardown;
- the `video` element you read turns out to be a 5-15 second pre-roll ad;
- the page returns a bot-check / challenge instead of content.

## The model: neutralise at the edges, never chase one-off

Do **not** write "click the close button after every action" scattered through a
script — ads reappear, modals re-render, new tabs spawn on their own. Instead,
push defence to the four *edges* where the page gets control:

| Edge | Defence |
|------|---------|
| **Before launch** | Browser flags + domain policy (`--extra-arg`, `--deny-domain`) |
| **Every document** | Inject the guardian **before page script runs** (`add_script_to_evaluate_on_new_document`) |
| **Every new target** | Popup policy: block, detect, list, close |
| **Every step** | Verify state (`page.url()`, `find`), don't assume the click landed |

## Layer 1 — harden at launch

CLI flags that remove whole categories of noise (compose them):

```sh
xcelerate \
  --user-data-dir .run/profile \        # keep consent cookies so walls don't return
  --extra-arg --mute-audio \            # ad video audio can't hijack the session
  --extra-arg --autoplay-policy=user-gesture-required \
  --extra-arg --disable-notifications \
  --deny-domain doubleclick.net \
  --deny-domain googlesyndication.com \
  --deny-domain adservice.google.com \
  --deny-domain criteo.com \
  --deny-domain openx.net \
  --deny-domain 3lift.com \
  --deny-domain adnxs.com \
  --deny-domain pubmatic.com \
  --deny-domain rubiconproject.com \
  --deny-domain inmobi.com \
  --deny-domain amazon-adsystem.com \
  --deny-domain adform.net \
  session
```

`--deny-domain` is enforced **before navigation** (`configure_domain_policy`),
so denied hosts never load — the strongest lever against third-party ad frames.
`--allow-domain` goes further and pins the run to one origin.

Rust equivalents (call once, before `Browser::launch`):

```rust
xcelerate::configure_domain_policy(vec![], vec!["doubleclick.net".into()]);
xcelerate::configure_user_data_dir(Some(".run/profile".into()))?;
xcelerate::configure_launch_options(xcelerate::LaunchOptions {
    extra_args: vec!["--mute-audio".into(), "--autoplay-policy=user-gesture-required".into()],
    ..Default::default()
});
```

## Layer 2 — the guardian (inject on every document)

The keystone. `Page::add_script_to_evaluate_on_new_document` runs a script
**before any page script** on the current document *and every future
navigation*, so the page can never re-install its popup/overlay behaviour.

`guardian.js` (in this skill directory) neutralises, in one shot:

1. `window.open` → returns `null` (kills scripted popups);
2. rewrites `target="_blank"` on click (kills click-spawned new tabs);
3. `beforeunload` traps (kills the "are you sure you want to leave?" freeze);
4. autoplay media muting;
5. remove common ad frames/containers (`doubleclick`, `adsbygoogle`, `[data-ad]`, …);
6. auto-click ad interstitials whose accessible name matches `close|skip|dismiss … ad`;
7. an escape hatch: `window.__xcelerateDismiss(/regex/i)` clicks matching controls.

It deliberately **does not** touch non-ad modals ("got it", "continue", "accept
cookies") — those are app UI the agent should decide about, not the guardian.

Install it:

```sh
xcelerate session
xcelerate> guard .agents/skills/hostile-pages/guardian.js
```

```rust
page.inject_file(".agents/skills/hostile-pages/guardian.js".into()).await?;
```

`inject_file` / `add_script_to_evaluate_on_new_document` apply to the next
document; `guard` also runs the script against the page that is already loaded.

## Layer 3 — popups and new tabs

Four independent tools; use them together.

**Block proactively** — the guardian (Layer 2) handles scripted `window.open`
and clicked `target="_blank"`. This is usually enough.

**Detect** a tab that still got through:

```rust
let target_id = page.wait_for_popup(5_000).await?;      // or wait_for_popup_url(..)
```

```sh
xcelerate> eval (()=>{ /* after a click, check for a new tab */ })()
```

**List** every target (tabs, iframes, service workers) with `Browser::targets`
(a JSON array of `Target.getTargets` results). Note it lists **iframes too**, so
this is also how you *discover* the ad hosts to deny (see the case study):

```sh
xcelerate> tabs
  1E4A…  page     https://example.com/
  9C02…  page     https://ad-network.example/trap        <-- the popup
  7543…  iframe   https://cm.g.doubleclick.net/partnerpixels   <-- an ad host
```

**Close** a target you do not want. xcelerate has no dedicated close-target
method; use the CDP escape hatch `Page::execute_cdp_cmd` → `Target.closeTarget`:

```sh
xcelerate> close-tab 9C02…
```

```rust
page.execute_cdp_cmd(
    "Target.closeTarget".into(),
    serde_json::json!({ "targetId": "9C02…" }).to_string(),
).await?;
```

The event-driven path (`wait_for_popup`) is the reliable one; polling `tabs` is
the fallback when the popup fires before you start waiting.

## Layer 4 — JavaScript dialogs (alert / confirm / prompt / beforeunload)

A blocking dialog freezes every subsequent step. Detect and answer it:

```rust
use xcelerate::page::Page;
// The dialog event arrives on the page session; enable Page domain first.
let _ = page.wait_for_event("Page.javascriptDialogOpening", 3_000).await;
page.handle_js_dialog(true, None).await?;   // accept=true, or None text for prompt
```

Guardian rule 3 already clears `beforeunload`, so teardown never hangs. For
`alert`/`confirm` that appear during navigation, run a small watcher that
listens for `Page.javascriptDialogOpening` and calls `handle_js_dialog`
(accept or dismiss per your policy).

## Layer 5 — overlays, interstitials, cookie walls

These are ordinary DOM; the failures come from *timing*, not detection.

1. **Wait for the blocker to leave, not just for the target to exist.**
   `wait_for_function("document.querySelector('.modal').offsetParent === null")`
   before clicking what was underneath.
2. **Dismiss with an accessible name**, not coordinates:
   `click close`, `click accept all`, `click got it`. `click`
   matches visible text **and** `aria-label`, so icon-only close buttons work.
3. **Verify the click landed**: re-read `page.url()` / `find <text>` instead of
   assuming.
4. **Remove-persistently** for the worst offenders: put a `remove` rule in
   `guardian.js` so the overlay never comes back on the next navigation.

## Layer 6 — navigation traps and redirects

- Pin the origin with `--allow-domain` when you never intend to leave it.
- After any click that *might* navigate, confirm where you are:
  `page.wait_for_navigation()` then read `page.url()`.
- Interrupted back-button / hash-trap loops: `page.go_back()` then re-verify.
- A redirect chain that ends on an unknown host is a signal, not a success —
  check the final URL against an expected prefix before trusting the page.

## Layer 7 — anti-bot challenges

Detect before you scrape:

```sh
xcelerate challenge https://target/          # JSON: {"detected":…,"vendors":…,"signals":…}
```

```rust
let report = page.detect_challenge().await?;
if report.detected { /* wait it out, or stop and report */ }
```

Do not loop-retry a challenge blindly; a detected challenge usually means the
session needs a real profile (`--user-data-dir`) or a slower, human-paced run.

## CLI session command reference (relevant subset)

| Command | Purpose |
|---|---|
| `guard <path.js>` | install the guardian in this page and every future document |
| `tabs` | list targets: id, type, url |
| `close-tab <id>` | `Target.closeTarget` for one target |
| `click <index\|selector\|text>` | click by snapshot index, CSS selector, or visible text / `aria-label` |
| `eval <js>` | run JS; `window.__xcelerateDismiss(/…/i)` to click by pattern |
| `wait <ms\|selector>` | sleep, or wait for a selector |
| `snapshot` | indexed interactive elements (a fresh view after ads are removed) |

## Recipe — a hardened session for a hostile site

```sh
xcelerate --user-data-dir .run/profile \
  --extra-arg --mute-audio session
```
```
>>> guard .agents/skills/hostile-pages/guardian.js
>>> open https://hostile.example/
>>> wait 2500
>>> snapshot                      # confirm what is actually interactable
>>> click got it                  # app modal (not an ad) — dismiss consciously
>>> tabs                          # any popup that slipped through?
>>> close-tab 9C02…               # if yes
>>> click textarea
>>> type hello
>>> press Enter
>>> wait 2000
>>> shot after.png
```

## Pitfalls

- **Removing the app, not the ad.** Keep guardian selectors tight (`AD_HOST` /
  `AD_HINT`); never blanket-remove `div.modal` — the game's own dialog is a modal.
- **Racing the ad.** Read state *after* it settles (`wait_for_function`), or you
  will screenshot a black interstitial (the exact failure in the case study).
- **Trusting `video`.** The first `video` element is often a pre-roll ad; check
  `document.querySelector('.ad-showing')` and duration before "watching".
- **Typing into a blocked/disabled field.** If `type` produces nothing, the field
  may be `disabled`/`readonly` or have a `maxlength` of `0` — and often an overlay
  is still covering it. Check `document.activeElement.tagName`, `textarea.disabled`
  and `textarea.readOnly`, and confirm `textarea.value` before assuming success.
- **Coordinates drift.** Ad re-flows move elements; prefer `click <index>`
  (`backendNodeId`-pinned) or `click "<text>"` over stored x/y.
- **Forgetting new documents.** A guard installed only via `evaluate` is lost on
  the next navigation. Use `add_script_to_evaluate_on_new_document` / `inject_file`.
