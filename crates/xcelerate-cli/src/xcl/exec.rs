//! The XCL executor: maps [`Command`]s onto the live browser, plugin, and HTTP
//! transports. This is the one place the language touches the engine facade.
//!
//! Dispatch is async, matching the rest of the CLI (which runs on a multi-thread
//! tokio runtime), so there are no thread bridges — a browser/plugin call is a
//! plain `.await`.

use std::sync::Arc;

use xcelerate::{Browser, Page};

use super::ast::Command;
use super::runtime::{Context, Outcome};

/// Shared executor state: a browser/page plus the current base URL.
pub struct Executor {
    pub browser: Arc<Browser>,
    pub page: Arc<Page>,
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
                "url" => exe.page.url().await.unwrap_or_default(),
                "title" => exe.page.title().await.unwrap_or_default(),
                "status" => ctx.vars.get("STATUS").cloned().unwrap_or_default(),
                "body" | "response" => ctx.vars.get("RESPONSE_BODY").cloned().unwrap_or_default(),
                other => ctx.vars.get(other).cloned().unwrap_or_default(),
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
        Command::Done => Outcome::ok("done".to_string()),
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
    let page = Arc::clone(&exe.page);

    match verb {
        "open" | "goto" => {
            let url = resolved.first().cloned().unwrap_or_default();
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
        "snapshot" | "snap" => Ok(page.agent_snapshot().await.unwrap_or_default()),
        "click" => {
            let sel = resolved.first().cloned().unwrap_or_default();
            if let Ok(index) = sel.parse::<u32>() {
                Arc::clone(&page)
                    .click_index(index)
                    .await
                    .map_err(|e| e.to_string())?;
                tokio::time::sleep(std::time::Duration::from_millis(600)).await;
                Ok(format!("click {index}"))
            } else {
                let el = Arc::clone(&page)
                    .wait_for_selector(sel.clone())
                    .await
                    .map_err(|e| e.to_string())?;
                el.click().await.map_err(|e| e.to_string())?;
                Ok(format!("click {sel}"))
            }
        }
        "click-text" => {
            let text = resolved.first().cloned().unwrap_or_default();
            let needle = serde_json::to_string(&text).unwrap_or_default();
            let decl = "const els=[...document.querySelectorAll('a,button,[role=\"button\"],[role=\"link\"],summary,input[type=\"submit\"]')];";
            let expr = format!(
                "els.find(e=>e.offsetParent!==null&&(((e.innerText||'')+' '+(e.getAttribute('aria-label')||'')).toLowerCase().includes({needle}.toLowerCase())))"
            );
            let found = page
                .evaluate_bool(format!("(() => {{ {decl} return !!({expr}); }})()"))
                .await
                .unwrap_or(false);
            if found {
                let el = page
                    .evaluate_handle(format!("(() => {{ {decl} return {expr}; }})()"))
                    .await
                    .map_err(|e| e.to_string())?;
                el.click_mouse().await.map_err(|e| e.to_string())?;
                Ok(format!("click-text {text:?}"))
            } else {
                Err(format!("no visible element contains {text:?}"))
            }
        }
        "fill" => {
            let sel = resolved.first().cloned().unwrap_or_default();
            let text = resolved.get(1).cloned().unwrap_or_default();
            let el = Arc::clone(&page)
                .wait_for_selector(sel.clone())
                .await
                .map_err(|e| e.to_string())?;
            el.type_text(text.clone())
                .await
                .map_err(|e| e.to_string())?;
            Ok(format!("fill {sel}"))
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
        "wait" | "sleep" => {
            let ms = resolved
                .first()
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(1000);
            tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
            Ok(format!("wait {ms}ms"))
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
    if let Some(host) = host_of(&url) {
        if crate::xcl::security::is_private_host(&host) && !ctx.permissions.allow_private {
            return Outcome::fail(format!(
                "blocked: `{host}` is a private/metadata host (pass --allow-private)"
            ));
        }
    }

    #[cfg(not(feature = "http"))]
    {
        let _ = (method, url, headers, body);
        return Outcome::fail("`request` requires a build with the `http` feature");
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
