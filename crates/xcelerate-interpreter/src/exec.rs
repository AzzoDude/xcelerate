//! The XCL executor: maps [`Command`]s onto the live browser, plugin, and HTTP
//! transports. This is the one place the language touches the engine facade.
//!
//! Dispatch is async, matching the rest of the CLI (which runs on a multi-thread
//! tokio runtime), so there are no thread bridges — a browser/plugin call is a
//! plain `.await`.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use xcelerate::{Browser, Page};

use super::ast::Command;
use super::runtime::{Context, Outcome};

/// Shared executor state: the browser plus the tabs opened during the run.
pub struct Executor {
    pub browser: Arc<Browser>,
    /// Tabs opened this run; index 0 is the page the run started on.
    tabs: Mutex<Vec<Arc<Page>>>,
    active: AtomicUsize,
}

impl Executor {
    /// A fresh executor with a single (active) tab.
    pub fn new(browser: Arc<Browser>, page: Arc<Page>) -> Self {
        Self {
            browser,
            tabs: Mutex::new(vec![page]),
            active: AtomicUsize::new(0),
        }
    }

    /// The currently active page.
    pub fn page(&self) -> Arc<Page> {
        let tabs = self.tabs.lock().unwrap();
        let index = self
            .active
            .load(Ordering::Relaxed)
            .min(tabs.len().saturating_sub(1));
        Arc::clone(&tabs[index])
    }
}

/// Dispatches a single [`Command`] to the transport, returning its [`Outcome`].
pub async fn dispatch(ctx: &mut Context, cmd: &Command, exe: &Executor) -> Outcome {
    match cmd {
        Command::Let { name, value } => match ctx.resolve(value) {
            Ok(v) => {
                ctx.vars.insert(name.clone(), v);
                Outcome::ok(format!("let {name}"))
            }
            Err(e) => Outcome::fail(e),
        },
        Command::Set { name, value } => match ctx.resolve(value) {
            Ok(v) => {
                ctx.vars.insert(name.clone(), v);
                Outcome::ok(format!("set {name}"))
            }
            Err(e) => Outcome::fail(e),
        },
        Command::Param { name, default } => {
            if ctx.vars.contains_key(name) {
                Outcome::ok(format!("param {name}"))
            } else if let Some(default) = default {
                match ctx.resolve(default) {
                    Ok(v) => {
                        ctx.vars.insert(name.clone(), v);
                        Outcome::ok(format!("param {name} (default)"))
                    }
                    Err(e) => Outcome::fail(e),
                }
            } else {
                Outcome::fail(format!("missing required param `{name}`"))
            }
        }
        Command::Print { args } => {
            let mut parts = Vec::with_capacity(args.len());
            for a in args {
                match ctx.resolve(a) {
                    Ok(v) => parts.push(v),
                    Err(e) => return Outcome::fail(e),
                }
            }
            Outcome::ok(parts.join(" "))
        }
        Command::Import { name } => {
            if !ctx.permissions.plugin_allowed(name) {
                return Outcome::fail(format!(
                    "plugin `{name}` is not allowed (use --allow-plugin)"
                ));
            }
            // `import` is a load-time hint; actual loading happens through the
            // plugin manager when the browser is built. Here we only gated it.
            Outcome::ok(format!("import {name}"))
        }
        Command::Plugins => {
            let names = exe.browser.plugin_names();
            let text = if names.is_empty() {
                "none".into()
            } else {
                names.join(", ")
            };
            Outcome::ok(format!("loaded: {text}"))
        }
        Command::PluginConfig { name, op } => match exe.browser.plugin(name.clone()) {
            Ok(handle) => match handle.config() {
                Ok(json) => {
                    let parsed: serde_json::Value =
                        serde_json::from_str(&json).unwrap_or(serde_json::Value::Null);
                    match op {
                        Some(op) => Outcome::ok_value(
                            format!("config for {name}.{op}"),
                            parsed
                                .get(op)
                                .cloned()
                                .unwrap_or(serde_json::Value::Null)
                                .to_string(),
                        ),
                        None => Outcome::ok_value(format!("config for {name}"), json),
                    }
                }
                Err(e) => Outcome::fail(e.to_string()),
            },
            Err(e) => Outcome::fail(e.to_string()),
        },
        Command::Run { plugin, op, json } => {
            // `lower_step` routes an unresolvable `call` here as a deliberate
            // failure, with the human-readable reason carried in `op`.
            if plugin == "__call__" {
                return Outcome::fail(op.clone());
            }
            if !ctx.permissions.plugin_allowed(plugin) {
                return Outcome::fail(format!(
                    "plugin `{plugin}` is not allowed (use --allow-plugin)"
                ));
            }
            let args = match json {
                Some(a) => ctx.resolve(a).unwrap_or_else(|e| e),
                None => "{}".to_string(),
            };
            match exe.browser.plugin(plugin.clone()) {
                Ok(handle) => match handle.invoke(op.clone(), args).await {
                    Ok(result) => Outcome::ok_value(format!("{plugin}.{op}"), result),
                    Err(e) => Outcome::fail(format!("{plugin}.{op}: {e}")),
                },
                Err(e) => Outcome::fail(e.to_string()),
            }
        }
        Command::Request {
            method,
            url,
            headers,
            body,
        } => request(ctx, method, url, headers, body).await,
        Command::Raw { verb, args } => match dispatch_raw(ctx, verb, args, exe).await {
            Ok(msg) => Outcome::ok(msg),
            Err(e) => Outcome::fail(e),
        },
        Command::Assert { subject, op, value } => {
            let expected = match ctx.resolve(value) {
                Ok(v) => v,
                Err(e) => return Outcome::fail(e),
            };
            let actual: String = match subject.as_str() {
                "url" => exe.page().url().await.unwrap_or_default(),
                "title" => exe.page().title().await.unwrap_or_default(),
                "content" => exe.page().content().await.unwrap_or_default(),
                "text" => exe
                    .page()
                    .evaluate_string("document.body ? document.body.innerText : ''".to_string())
                    .await
                    .unwrap_or_default(),
                "status" => ctx.vars.get("STATUS").cloned().unwrap_or_default(),
                "body" | "response" => ctx.vars.get("RESPONSE_BODY").cloned().unwrap_or_default(),
                other => ctx
                    .vars
                    .get(other.strip_prefix('$').unwrap_or(other))
                    .cloned()
                    .unwrap_or_default(),
            };
            let passed = match op.as_str() {
                "==" | "eq" => actual == expected,
                "!=" | "ne" => actual != expected,
                "contains" => actual.contains(&expected),
                "matches" => actual.contains(&expected),
                ">" => actual > expected,
                "<" => actual < expected,
                ">=" => actual >= expected,
                "<=" => actual <= expected,
                _ => false,
            };
            if passed {
                Outcome::ok(format!("assert {subject} {op} {expected:?}"))
            } else {
                Outcome::fail(format!(
                    "assert {subject} {op} {expected:?} failed (got {actual:?})"
                ))
            }
        }
        Command::Done => Outcome::done("done".to_string()),
        Command::Quit => Outcome::quit("quit".to_string()),
        Command::FuncStart(_)
        | Command::FuncEnd
        | Command::Label { .. }
        | Command::Goto { .. }
        | Command::Repeat { .. }
        | Command::Retry { .. }
        | Command::IfOk(_)
        | Command::IfFail(_)
        | Command::Call { .. } => Outcome::ok(String::new()),
    }
}

/// Dispatches a browser/session `Raw` verb against the live page.
/// Returns `Result<String, String>` so the `?` operator resolves cleanly; the
/// caller (`dispatch`) maps it onto [`Outcome`].
async fn dispatch_raw(
    ctx: &mut Context,
    verb: &str,
    args: &[super::ast::Arg],
    exe: &Executor,
) -> Result<String, String> {
    let mut resolved = Vec::new();
    for a in args {
        match ctx.resolve(a) {
            Ok(v) => resolved.push(v),
            Err(e) => return Err(e),
        }
    }
    let page = exe.page();

    match verb {
        "open" | "goto" => {
            let url = super::runtime::normalize_url(&resolved.first().cloned().unwrap_or_default());
            page.navigate(url.clone())
                .await
                .map_err(|e| e.to_string())?;
            let _ = page.wait_for_navigation().await;
            Ok(format!("open {url}"))
        }
        "title" => Ok(page.title().await.unwrap_or_default()),
        "url" => Ok(page.url().await.unwrap_or_default()),
        "text" => Ok(page
            .evaluate_string("document.body ? document.body.innerText : ''".to_string())
            .await
            .unwrap_or_default()),
        "markdown" | "md" => Ok(page.markdown().await.unwrap_or_default()),
        "content" | "html" => Ok(page.content().await.unwrap_or_default()),
        "hover" => {
            let sel = resolved.first().cloned().unwrap_or_default();
            let el = Arc::clone(&page)
                .wait_for_selector(sel.clone())
                .await
                .map_err(|e| e.to_string())?;
            el.hover_mouse().await.map_err(|e| e.to_string())?;
            Ok(format!("hover {sel}"))
        }
        "mouse" => {
            // `mouse [click|move] <x> <y>|<index>|<selector>|<text>`. A leading
            // `click` moves the cursor to the target and clicks it.
            let mut args: &[String] = &resolved;
            let mut click = false;
            if let Some(first) = args.first() {
                if first.eq_ignore_ascii_case("click") {
                    click = true;
                    args = &args[1..];
                } else if first.eq_ignore_ascii_case("move") {
                    args = &args[1..];
                }
            }
            if args.len() == 2
                && let (Ok(x), Ok(y)) = (args[0].parse::<f64>(), args[1].parse::<f64>())
            {
                if click {
                    Arc::clone(&page)
                        .click_mouse(x, y)
                        .await
                        .map_err(|e| e.to_string())?;
                    return Ok(format!("click {x} {y}"));
                }
                Arc::clone(&page)
                    .move_mouse(x, y)
                    .await
                    .map_err(|e| e.to_string())?;
                return Ok(format!("mouse {x} {y}"));
            }
            let target = args.first().cloned().unwrap_or_default();
            if target.is_empty() {
                return Err(
                    "usage: mouse [click] <index|selector|text> | mouse <x> <y>".to_string()
                );
            }
            if let Ok(index) = target.parse::<u32>() {
                if click {
                    Arc::clone(&page)
                        .click_index(index)
                        .await
                        .map_err(|e| e.to_string())?;
                    Ok(format!("click {index}"))
                } else {
                    Arc::clone(&page)
                        .move_to_index(index)
                        .await
                        .map_err(|e| e.to_string())?;
                    Ok(format!("mouse {index}"))
                }
            } else if crate::interact::looks_like_a_selector(&target) {
                let el = Arc::clone(&page)
                    .wait_for_selector(target.clone())
                    .await
                    .map_err(|e| e.to_string())?;
                if click {
                    el.click_mouse().await.map_err(|e| e.to_string())?;
                    Ok(format!("click {target}"))
                } else {
                    el.hover_mouse().await.map_err(|e| e.to_string())?;
                    Ok(format!("mouse {target}"))
                }
            } else {
                match crate::interact::control_by_text(&page, &target)
                    .await
                    .map_err(|e| e.to_string())?
                {
                    Some(el) => {
                        if click {
                            el.click_mouse().await.map_err(|e| e.to_string())?;
                            Ok(format!("click {target:?}"))
                        } else {
                            el.hover_mouse().await.map_err(|e| e.to_string())?;
                            Ok(format!("mouse {target:?}"))
                        }
                    }
                    None => Err(format!("no visible element contains {target:?}")),
                }
            }
        }
        "scroll" => {
            let arg = resolved.first().cloned().unwrap_or_default();
            let js = match arg.as_str() {
                "" | "down" => "window.scrollBy(0, window.innerHeight * 0.9)".to_string(),
                "up" => "window.scrollBy(0, -window.innerHeight * 0.9)".to_string(),
                "top" => "window.scrollTo(0, 0)".to_string(),
                "bottom" => "window.scrollTo(0, document.body.scrollHeight)".to_string(),
                other => match other.parse::<i64>() {
                    Ok(pixels) => format!("window.scrollBy(0, {pixels})"),
                    Err(_) => return Err("usage: scroll <pixels|up|down|top|bottom>".to_string()),
                },
            };
            let _ = page.evaluate_string(js).await;
            Ok(format!(
                "scroll {}",
                if arg.is_empty() { "down" } else { arg.as_str() }
            ))
        }
        "find" => {
            let text = resolved.first().cloned().unwrap_or_default();
            let count = page
                .find_text(text.clone())
                .await
                .map_err(|e| e.to_string())?;
            Ok(format!("{count} match(es) for {text:?}"))
        }
        "challenge" | "detect" => {
            let report = page.detect_challenge().await.map_err(|e| e.to_string())?;
            Ok(report.to_json().to_string())
        }
        "shot" | "screenshot" => {
            let raw = resolved
                .first()
                .cloned()
                .unwrap_or_else(|| "screenshot.png".to_string());
            let path = ctx.resolve_path(&raw)?;
            let png = page.screenshot().await.map_err(|e| e.to_string())?;
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            std::fs::write(&path, &png).map_err(|e| e.to_string())?;
            Ok(format!("wrote {} ({} bytes)", path.display(), png.len()))
        }
        "shot-full" | "screenshot-full" => {
            let raw = resolved
                .first()
                .cloned()
                .unwrap_or_else(|| "screenshot.png".to_string());
            let path = ctx.resolve_path(&raw)?;
            let png = page.screenshot_full().await.map_err(|e| e.to_string())?;
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            std::fs::write(&path, &png).map_err(|e| e.to_string())?;
            Ok(format!("wrote {} ({} bytes)", path.display(), png.len()))
        }
        "media" => page.media_json().await.map_err(|e| e.to_string()),
        "download" => {
            // `download <url> <path>`: fetch through the browser. A direct file is
            // streamed; an HLS (`.m3u8`) stream is assembled from its segments.
            // The path is confined to the workspace root (see `resolve_path`).
            let url = resolved.first().cloned().unwrap_or_default();
            let raw = resolved.get(1).cloned().unwrap_or_default();
            if url.is_empty() || raw.is_empty() {
                return Err("usage: download <url> <path>".to_string());
            }
            let path = ctx.resolve_path(&raw)?;
            let target = path.to_string_lossy().into_owned();
            page.grab(url.clone(), target)
                .await
                .map_err(|e| e.to_string())
        }
        "capture" => {
            // `capture <url> <path> [seconds]`: open a page, let it play, and
            // reassemble the media it fetches itself (Media Source Extensions —
            // YouTube, Facebook). Capturing starts before navigating so the init
            // segment is seen. The path is confined to the workspace root.
            let url = resolved.first().cloned().unwrap_or_default();
            let raw = resolved.get(1).cloned().unwrap_or_default();
            if url.is_empty() || raw.is_empty() {
                return Err("usage: capture <url> <path> [seconds]".to_string());
            }
            let path = ctx.resolve_path(&raw)?;
            let seconds = resolved
                .get(2)
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(15);
            page.start_media_capture()
                .await
                .map_err(|e| e.to_string())?;
            page.navigate(super::runtime::normalize_url(&url))
                .await
                .map_err(|e| e.to_string())?;
            let _ = page.wait_for_navigation().await;
            let _ = page
                .evaluate_string(
                    "(function(){ const v=document.querySelector('video'); \
                     if (v) { v.muted = true; if (v.play) v.play().catch(function(){}); } \
                     return true; })()"
                        .to_string(),
                )
                .await;
            tokio::time::sleep(std::time::Duration::from_secs(seconds)).await;
            page.save_capture(path.to_string_lossy().into_owned())
                .await
                .map_err(|e| e.to_string())
        }
        "upload" | "set-input-files" => {
            // `<input type="file">` cannot be set from page JS; this uses CDP
            // `DOM.setFileInputFiles` under the hood. The source is confined to
            // the workspace root so a script cannot exfiltrate files by path.
            let selector = resolved.first().cloned().unwrap_or_default();
            let raw = resolved.get(1).cloned().unwrap_or_default();
            if selector.is_empty() || raw.is_empty() {
                return Err("usage: upload <selector> <path>".to_string());
            }
            let path = ctx.resolve_path(&raw)?;
            let files = serde_json::json!([path.to_string_lossy()]).to_string();
            Arc::clone(&page)
                .set_input_files(selector.clone(), files)
                .await
                .map_err(|e| e.to_string())?;
            Ok(format!("upload {selector} <- {}", path.display()))
        }
        "cookie" | "cookies" => {
            // `cookie [get [name]]` / `cookie set <name> <value> [domain] [path]`
            // / `cookie add <json>` / `cookie delete <name>` / `cookie clear`.
            // Cookies are set through CDP (`Network.setCookie`) so `HttpOnly`
            // session cookies - which page JS cannot write - can be restored.
            let sub = resolved
                .first()
                .map(|s| s.to_ascii_lowercase())
                .unwrap_or_default();
            match sub.as_str() {
                // No subcommand, or `get`/`list`: dump cookies as JSON. A name
                // argument narrows it to that one cookie.
                "" | "get" | "list" => match resolved.get(1) {
                    Some(name) if !name.is_empty() => {
                        page.cookie(name.clone()).await.map_err(|e| e.to_string())
                    }
                    _ => page.cookies().await.map_err(|e| e.to_string()),
                },
                "set" => {
                    let name = resolved.get(1).cloned().unwrap_or_default();
                    let value = resolved.get(2).cloned().unwrap_or_default();
                    if name.is_empty() {
                        return Err("usage: cookie set <name> <value> [domain] [path]".to_string());
                    }
                    let mut cookie =
                        serde_json::json!({ "name": name, "value": value, "path": "/" });
                    match resolved.get(3).filter(|d| !d.is_empty()) {
                        Some(domain) => {
                            cookie["domain"] = serde_json::Value::String(domain.clone());
                            if let Some(path) = resolved.get(4).filter(|p| !p.is_empty()) {
                                cookie["path"] = serde_json::Value::String(path.clone());
                            }
                        }
                        // `Network.setCookie` needs a `url` or a `domain`; fall
                        // back to the page's own URL so `cookie set` works on the
                        // page that is open right now.
                        None => {
                            cookie["url"] =
                                serde_json::Value::String(page.url().await.unwrap_or_default());
                        }
                    }
                    page.execute_cdp_cmd("Network.setCookie".to_string(), cookie.to_string())
                        .await
                        .map_err(|e| e.to_string())?;
                    Ok(format!("cookie set {name}"))
                }
                // `cookie add <json>` - full control (object or array) so
                // `httpOnly`/`secure`/`sameSite`/expiry survive a round-trip.
                "add" | "import" => {
                    let json = resolved.get(1).cloned().unwrap_or_default();
                    if json.is_empty() {
                        return Err("usage: cookie add <json>".to_string());
                    }
                    let parsed: serde_json::Value = serde_json::from_str(&json)
                        .map_err(|e| format!("cookie add: invalid JSON: {e}"))?;
                    let list = match parsed {
                        serde_json::Value::Array(list) => list,
                        other => vec![other],
                    };
                    for cookie in &list {
                        page.execute_cdp_cmd("Network.setCookie".to_string(), cookie.to_string())
                            .await
                            .map_err(|e| e.to_string())?;
                    }
                    Ok(format!("added {} cookie(s)", list.len()))
                }
                "delete" | "remove" | "rm" => {
                    let name = resolved.get(1).cloned().unwrap_or_default();
                    if name.is_empty() {
                        return Err("usage: cookie delete <name>".to_string());
                    }
                    page.execute_cdp_cmd(
                        "Network.deleteCookies".to_string(),
                        serde_json::json!({ "name": name }).to_string(),
                    )
                    .await
                    .map_err(|e| e.to_string())?;
                    Ok(format!("cookie delete {name}"))
                }
                "clear" | "clear-all" => {
                    page.execute_cdp_cmd(
                        "Network.clearBrowserCookies".to_string(),
                        "{}".to_string(),
                    )
                    .await
                    .map_err(|e| e.to_string())?;
                    Ok("cookies cleared".to_string())
                }
                other => Err(format!(
                    "usage: cookie [get [name]|set <name> <value> [domain] [path]|add <json>|delete <name>|clear] (got `{other}`)"
                )),
            }
        }
        "click" | "tap" => {
            let target = resolved.first().cloned().unwrap_or_default();
            if target.is_empty() {
                return Err("usage: click <index|selector|text>".to_string());
            }
            // A bare integer is a snapshot index, a selector-looking string is a
            // CSS selector, anything else is matched against visible text - the
            // same rule the interactive session uses.
            if let Ok(index) = target.parse::<u32>() {
                Arc::clone(&page)
                    .click_index(index)
                    .await
                    .map_err(|e| e.to_string())?;
                tokio::time::sleep(std::time::Duration::from_millis(600)).await;
                Ok(format!("click {index}"))
            } else if crate::interact::looks_like_a_selector(&target) {
                let el = Arc::clone(&page)
                    .wait_for_selector(target.clone())
                    .await
                    .map_err(|e| e.to_string())?;
                // Move the real mouse to the element and click, so the cursor
                // travels to the target instead of firing a synthetic DOM click.
                el.click_mouse().await.map_err(|e| e.to_string())?;
                Ok(format!("click {target}"))
            } else {
                match crate::interact::control_by_text(&page, &target)
                    .await
                    .map_err(|e| e.to_string())?
                {
                    Some(el) => {
                        el.click_mouse().await.map_err(|e| e.to_string())?;
                        Ok(format!("click {target:?}"))
                    }
                    None => Err(format!("no visible element contains {target:?}")),
                }
            }
        }
        "fill" => {
            let sel = resolved.first().cloned().unwrap_or_default();
            let text = resolved.get(1).cloned().unwrap_or_default();
            // A bare integer is a snapshot index (framework-rendered fields often
            // expose no stable selector) - the same rule `click` already uses.
            let el = if let Ok(index) = sel.parse::<u32>() {
                if Arc::clone(&page).click_index(index).await.is_err() {
                    let _ = page.agent_snapshot().await;
                    Arc::clone(&page)
                        .click_index(index)
                        .await
                        .map_err(|e| e.to_string())?;
                }
                page.evaluate_handle("document.activeElement".to_string())
                    .await
                    .map_err(|e| e.to_string())?
            } else {
                Arc::clone(&page)
                    .wait_for_selector(sel.clone())
                    .await
                    .map_err(|e| e.to_string())?
            };
            el.type_text(text.clone())
                .await
                .map_err(|e| e.to_string())?;
            Ok(format!("fill {sel}"))
        }
        "select" => {
            let selector = resolved.first().cloned().unwrap_or_default();
            let value = resolved.get(1).cloned().unwrap_or_default();
            if selector.is_empty() || value.is_empty() {
                return Err("usage: select <selector> <value>".to_string());
            }
            // Native `<select>` only (matches by value or label). A custom
            // listbox is driven with `click` (open it, then click the option).
            let values = serde_json::json!([value]).to_string();
            Arc::clone(&page)
                .select_option(selector.clone(), values)
                .await
                .map_err(|e| e.to_string())?;
            Ok(format!("select {selector} = {value}"))
        }
        "type" => {
            let text = resolved.first().cloned().unwrap_or_default();
            let el = page
                .evaluate_handle("document.activeElement".to_string())
                .await
                .map_err(|e| e.to_string())?;
            el.type_text(text.clone())
                .await
                .map_err(|e| e.to_string())?;
            Ok(format!("type {text}"))
        }
        "press" | "submit" | "send" => {
            let key = if verb == "press" {
                resolved.first().cloned().unwrap_or_default()
            } else {
                "Enter".to_string()
            };
            let el = page
                .evaluate_handle("document.activeElement".to_string())
                .await
                .map_err(|e| e.to_string())?;
            el.press(key.clone()).await.map_err(|e| e.to_string())?;
            Ok(format!("press {key}"))
        }
        // Unit-suffixed sleeps: the unit lives in the verb, the value is a plain
        // number (there is no `2s`/`500ms` literal in the language).
        "wait-ms" => {
            super::runtime::wait_scaled(&resolved.first().cloned().unwrap_or_default(), 1).await
        }
        "wait-sec" => {
            super::runtime::wait_scaled(&resolved.first().cloned().unwrap_or_default(), 1_000).await
        }
        "wait-min" => {
            super::runtime::wait_scaled(&resolved.first().cloned().unwrap_or_default(), 60_000)
                .await
        }
        "wait-hr" => {
            super::runtime::wait_scaled(&resolved.first().cloned().unwrap_or_default(), 3_600_000)
                .await
        }
        // Randomized sleep: two millisecond bounds (inclusive).
        "wait-random" => {
            let min = super::runtime::parse_ms(&resolved.first().cloned().unwrap_or_default())?;
            let max = super::runtime::parse_ms(&resolved.get(1).cloned().unwrap_or_default())?;
            let ms = super::runtime::random_ms(min, max);
            tokio::time::sleep(std::time::Duration::from_millis(ms as u64)).await;
            Ok(format!("wait-random {ms}ms"))
        }
        "wait" | "sleep" => {
            let arg = resolved.first().cloned().unwrap_or_default();
            if let Some(ms) = super::runtime::parse_duration_ms(&arg) {
                tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
                Ok(format!("wait {ms}ms"))
            } else if !arg.is_empty() {
                Arc::clone(&page)
                    .wait_for_selector(arg.clone())
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(format!("wait for {arg}"))
            } else {
                Ok("wait".to_string())
            }
        }
        "wait-idle" | "idle" => {
            page.wait_for_network_idle(500, 30_000)
                .await
                .map_err(|e| e.to_string())?;
            Ok("network idle".to_string())
        }
        "wait-stable" | "stable" => {
            page.wait_for_dom_stable(500, 30_000)
                .await
                .map_err(|e| e.to_string())?;
            Ok("dom stable".to_string())
        }
        "back" => {
            page.go_back().await.map_err(|e| e.to_string())?;
            Ok("back".to_string())
        }
        "reload" => {
            page.reload().await.map_err(|e| e.to_string())?;
            Ok("reload".to_string())
        }
        "eval" | "js" => {
            if !ctx.permissions.allow_eval {
                Err("`eval` is disabled (pass --allow-unsafe)".to_string())
            } else {
                Ok(page
                    .evaluate_json(resolved.first().cloned().unwrap_or_default())
                    .await
                    .unwrap_or_default())
            }
        }
        "tabs" => {
            let tabs = exe.tabs.lock().unwrap().clone();
            let active = exe.active.load(Ordering::Relaxed);
            for (i, tab) in tabs.iter().enumerate() {
                let url = tab.url().await.unwrap_or_default();
                let mark = if i == active { "*" } else { " " };
                println!("{mark} [{i}] {url}");
            }
            Ok(format!("{} tab(s)", tabs.len()))
        }
        "new-tab" | "newtab" | "tab-new" => {
            let url = super::runtime::normalize_url(
                &resolved
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "about:blank".to_string()),
            );
            let opened = Arc::clone(&exe.browser)
                .new_page(url.clone())
                .await
                .map_err(|e| e.to_string())?;
            let index = {
                let mut tabs = exe.tabs.lock().unwrap();
                tabs.push(Arc::clone(&opened));
                tabs.len() - 1
            };
            exe.active.store(index, Ordering::Relaxed);
            Ok(format!("new-tab [{index}] {url}"))
        }
        "switch" | "tab" | "use" => {
            let count = exe.tabs.lock().unwrap().len();
            let index = match resolved.first() {
                Some(arg) => arg
                    .parse::<usize>()
                    .map_err(|_| "usage: switch <index>".to_string())?,
                None => (exe.active.load(Ordering::Relaxed) + 1) % count,
            };
            if index >= count {
                return Err(format!("no tab {index}; {count} open"));
            }
            exe.active.store(index, Ordering::Relaxed);
            let url = exe.page().url().await.unwrap_or_default();
            Ok(format!("switch [{index}] {url}"))
        }
        other => Err(format!("unknown verb `{other}`")),
    }
}

/// Executes a `request` command: enforces HTTP permission + SSRF private-range
/// guard, then performs the HTTP call (when the `http` feature is enabled).
async fn request(
    ctx: &mut Context,
    method: &str,
    url: &super::ast::Arg,
    headers: &Option<super::ast::Arg>,
    body: &Option<super::ast::Arg>,
) -> Outcome {
    if !ctx.permissions.allow_http {
        return Outcome::fail("`request` is disabled (pass --allow-http or --allow-unsafe)");
    }
    let url = match ctx.resolve(url) {
        Ok(u) => u,
        Err(e) => return Outcome::fail(e),
    };
    if let Some(host) = host_of(&url)
        && crate::security::is_private_host(&host)
        && !ctx.permissions.allow_private
    {
        return Outcome::fail(format!(
            "blocked: `{host}` is a private/metadata host (pass --allow-private)"
        ));
    }

    #[cfg(not(feature = "http"))]
    {
        let _ = (method, url, headers, body);
        Outcome::fail("`request` requires a build with the `http` feature")
    }

    #[cfg(feature = "http")]
    {
        let headers_json = headers.as_ref().map(|h| ctx.resolve(h)).transpose();
        let body_val = body.as_ref().map(|b| ctx.resolve(b)).transpose();
        let header_map = match headers_json {
            Ok(Some(h)) => serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&h)
                .map(|m| {
                    m.into_iter()
                        .map(|(k, v)| (k, v.as_str().unwrap_or_default().to_string()))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
            _ => Vec::new(),
        };
        let body_str = match body_val {
            Ok(Some(b)) => b,
            _ => String::new(),
        };
        match crate::net::request(method, &url, header_map, body_str).await {
            Ok((status, response)) => {
                ctx.vars.insert("STATUS".into(), status.to_string());
                ctx.vars.insert("RESPONSE_BODY".into(), response);
                Outcome::ok(format!("http {status}"))
            }
            Err(e) => Outcome::fail(e.to_string()),
        }
    }
}

/// Extracts a lowercased host from a URL (mirrors `policy::host_of`).
fn host_of(url: &str) -> Option<String> {
    let after = url
        .find("://")
        .map(|i| &url[i + 3..])
        .or_else(|| url.strip_prefix("//"))?;
    let authority = after.split(['/', '?', '#']).next().unwrap_or("");
    let host_port = authority
        .rsplit_once('@')
        .map(|(_, h)| h)
        .unwrap_or(authority);
    let host = host_port
        .rsplit_once(':')
        .map(|(h, _)| h)
        .unwrap_or(host_port);
    let host = host.trim();
    if host.is_empty() {
        None
    } else {
        Some(host.to_ascii_lowercase())
    }
}
