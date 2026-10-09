//! `browser` - the browser control surface as a WebAssembly plugin.
//!
//! This plugin owns the **verb logic** for the browser: target classification
//! (selector vs. visible text vs. snapshot index), JavaScript construction,
//! multi-step sequences, and the human-facing result strings. The host owns the
//! **protocol**: it holds the page and speaks CDP/BiDi, exposing only low-level
//! *primitives* (`goto`, `click-selector`, `evaluate`, `cdp`, …) through
//! `host.browser`. The guest never sees a protocol message or a socket.
//!
//! The interpreter relays a verb by name with its resolved positional
//! arguments; every handler lives in `run` below.
//!
//! Build: `xcelerate build --wasm-only` (run in this directory) stages
//! `browser.wasm` next to this manifest. Then drop the directory in
//! `~/.xcl/plugins/` and load it by name (`browser`).

wit_bindgen::generate!({
    path: "wit/plugin.wit",
    world: "plugin-world",
});

use serde::Deserialize;
use serde_json::{json, Value};
use xcelerate::plugin::host;
use xcelerate_plugin::guest::{decode, encode};

/// The verbs this plugin answers (the interpreter relays these by name).
const OPS: &[&str] = &[
    "open",
    "goto",
    "title",
    "url",
    "text",
    "markdown",
    "md",
    "content",
    "html",
    "hover",
    "mouse",
    "scroll",
    "find",
    "challenge",
    "detect",
    "media",
    "cookie",
    "cookies",
    "geolocation",
    "storage",
    "click",
    "tap",
    "fill",
    "select",
    "type",
    "press",
    "submit",
    "send",
    "wait",
    "wait-idle",
    "idle",
    "wait-stable",
    "stable",
    "back",
    "reload",
    "eval",
    "js",
    "dialog",
    "drag",
    "auth",
];

struct Browser;

/// What the interpreter sends: the resolved positional arguments, plus the
/// interpreter's `--allow-unsafe` decision (so `eval` gating stays a policy
/// choice of the host, not the plugin).
#[derive(Deserialize)]
struct Request {
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    allow_eval: bool,
}

impl exports::xcelerate::plugin::plugin::Guest for Browser {
    fn describe() -> Vec<u8> {
        #[derive(serde::Serialize)]
        struct Describe<'a> {
            name: &'a str,
            version: &'a str,
            ops: Vec<&'a str>,
        }
        let describe = Describe {
            name: "browser",
            version: env!("CARGO_PKG_VERSION"),
            ops: OPS.to_vec(),
        };
        rmp_serde::to_vec_named(&describe).unwrap_or_default()
    }

    fn invoke(op: String, args: Vec<u8>) -> Result<Vec<u8>, String> {
        let request: Request = decode(&args)?;
        let message = run(&op, &request)?;
        encode(&message)
    }
}

export!(Browser);

// ---------------------------------------------------------------------------
// Verb logic
// ---------------------------------------------------------------------------

/// Call a host primitive and decode its JSON reply.
fn page(op: &str, args: &Value) -> Result<Value, String> {
    let payload = rmp_serde::to_vec_named(args).map_err(|error| error.to_string())?;
    let reply = host::browser(op, &payload)?;
    rmp_serde::from_slice(&reply).map_err(|error| error.to_string())
}

/// The `value` field of a reading primitive's reply, as text.
fn value(primitive: &str, args: &Value) -> Result<String, String> {
    Ok(page(primitive, args)?
        .get("value")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string())
}

fn arg(args: &[String], index: usize) -> String {
    args.get(index).cloned().unwrap_or_default()
}

/// Whether a target reads as a selector rather than visible text.
fn is_selector(value: &str) -> bool {
    let value = value.trim();
    value.starts_with(['#', '.', '['])
        || value.starts_with("//")
        || value.starts_with("xpath=")
        || value.starts_with("role=")
        || value.starts_with("label=")
        || value.starts_with("text=")
        || (value.contains('[') && value.contains(']'))
}

/// A string as a JavaScript literal (safe against quotes/backslashes/newlines).
fn js_literal(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string())
}

/// Decode a JSON string result to its text, leaving non-strings untouched.
fn unwrap_json_string(raw: String) -> String {
    match serde_json::from_str::<Value>(&raw) {
        Ok(Value::String(text)) => text,
        _ => raw,
    }
}

/// Click or hover a target: an element resolved by selector or visible text.
fn point(pattern: &str, target: &str) -> Result<(), String> {
    if is_selector(target) {
        page(pattern, &json!({ "selector": target }))?;
    } else {
        page(&format!("{pattern}-text"), &json!({ "text": target }))?;
    }
    Ok(())
}

/// The whole verb table.
fn run(op: &str, req: &Request) -> Result<String, String> {
    let args = &req.args;
    match op {
        "open" | "goto" => {
            let url = arg(args, 0);
            let landed = page("goto", &json!({ "url": url }))?
                .get("url")
                .and_then(Value::as_str)
                .unwrap_or(&url)
                .to_string();
            Ok(format!("open {landed}"))
        }
        "title" => value("title", &json!({})),
        "url" => value("url", &json!({})),
        "text" => value("text", &json!({})),
        "markdown" | "md" => value("markdown", &json!({})),
        "content" | "html" => value("html", &json!({})),
        "media" => value("media-json", &json!({})),

        "hover" => {
            let target = arg(args, 0);
            point("hover", &target)?;
            Ok(format!("hover {target}"))
        }
        "mouse" => mouse(args),
        "scroll" => scroll(args),
        "find" => {
            let text = arg(args, 0);
            let count = page("find-text", &json!({ "text": text }))?
                .get("count")
                .cloned()
                .unwrap_or(Value::Null);
            Ok(format!("{count} match(es) for {text:?}"))
        }
        "challenge" | "detect" => {
            let report = page("detect-challenge", &json!({}))?;
            Ok(report
                .get("report")
                .cloned()
                .unwrap_or(Value::Null)
                .to_string())
        }
        "cookie" | "cookies" => cookie(args),
        "geolocation" => geolocation(args),
        "storage" => storage(args),

        "click" | "tap" => {
            let target = arg(args, 0);
            if target.is_empty() {
                return Err("usage: click <index|selector|text>".to_string());
            }
            if let Ok(index) = target.parse::<u64>() {
                page("click-index", &json!({ "index": index, "settle_ms": 600 }))?;
                Ok(format!("click {index}"))
            } else if is_selector(&target) {
                page("click-selector", &json!({ "selector": target }))?;
                Ok(format!("click {target}"))
            } else {
                page("click-text", &json!({ "text": target }))?;
                Ok(format!("click {target:?}"))
            }
        }
        "fill" => {
            let selector = arg(args, 0);
            let text = arg(args, 1);
            if let Ok(index) = selector.parse::<u64>() {
                page("type-index", &json!({ "index": index, "text": text }))?;
            } else {
                page(
                    "type-selector",
                    &json!({ "selector": selector, "text": text }),
                )?;
            }
            Ok(format!("fill {selector}"))
        }
        "select" => {
            let selector = arg(args, 0);
            let value = arg(args, 1);
            if selector.is_empty() || value.is_empty() {
                return Err("usage: select <selector> <value>".to_string());
            }
            page(
                "select-option",
                &json!({ "selector": selector, "value": value }),
            )?;
            Ok(format!("select {selector} = {value}"))
        }
        "type" => {
            let text = arg(args, 0);
            page("type-active", &json!({ "text": text }))?;
            Ok(format!("type {text}"))
        }
        "press" | "submit" | "send" => {
            let key = if op == "press" {
                arg(args, 0)
            } else {
                "Enter".to_string()
            };
            page("press-active", &json!({ "key": key }))?;
            Ok(format!("press {key}"))
        }

        "wait" => {
            let selector = arg(args, 0);
            page("wait-selector", &json!({ "selector": selector }))?;
            Ok(format!("wait for {selector}"))
        }
        "wait-idle" | "idle" => {
            page(
                "wait-network-idle",
                &json!({ "idle_ms": 500, "timeout_ms": 30_000 }),
            )?;
            Ok("network idle".to_string())
        }
        "wait-stable" | "stable" => {
            page(
                "wait-dom-stable",
                &json!({ "quiet_ms": 500, "timeout_ms": 30_000 }),
            )?;
            Ok("dom stable".to_string())
        }
        "back" => {
            page("back", &json!({}))?;
            Ok("back".to_string())
        }
        "reload" => {
            page("reload", &json!({}))?;
            Ok("reload".to_string())
        }
        "eval" | "js" => {
            if !req.allow_eval {
                return Err("`eval` is disabled (pass --allow-unsafe)".to_string());
            }
            value("evaluate", &json!({ "js": arg(args, 0), "as": "json" }))
        }
        "dialog" => {
            let raw = arg(args, 0);
            let policy = match raw.to_ascii_lowercase().as_str() {
                "" | "dismiss" => "dismiss",
                "accept" => "accept",
                other => return Err(format!("usage: dialog <dismiss|accept> (got {other:?})")),
            };
            page("dialog-policy", &json!({ "policy": policy }))?;
            Ok(format!("dialogs: {policy}"))
        }
        "drag" => {
            let from = arg(args, 0);
            let to = arg(args, 1);
            if from.is_empty() || to.is_empty() {
                return Err("usage: drag <from> <to>".to_string());
            }
            page("drag", &json!({ "from": from, "to": to }))?;
            Ok(format!("dragged {from} -> {to}"))
        }
        "auth" => {
            let username = arg(args, 0);
            let password = arg(args, 1);
            if username.is_empty() {
                return Err("usage: auth <username> <password>".to_string());
            }
            page(
                "auth",
                &json!({ "username": username, "password": password }),
            )?;
            Ok(format!("http auth credentials set for {username}"))
        }

        other => Err(format!("browser: unknown verb '{other}'")),
    }
}

/// `mouse [click|move] <x> <y> | <index> | <selector> | <text>`.
fn mouse(args: &[String]) -> Result<String, String> {
    let mut index = 0;
    let mut click = false;
    match args.first().map(|s| s.to_ascii_lowercase()) {
        Some(first) if first == "click" => {
            click = true;
            index = 1;
        }
        Some(first) if first == "move" => index = 1,
        _ => {}
    }
    let rest = &args[index.min(args.len())..];

    if rest.len() == 2 && {
        let parsed = (rest[0].parse::<f64>(), rest[1].parse::<f64>());
        matches!(parsed, (Ok(_), Ok(_)))
    } {
        let x = rest[0].parse::<f64>().unwrap_or_default();
        let y = rest[1].parse::<f64>().unwrap_or_default();
        let primitive = if click { "click-mouse" } else { "move-mouse" };
        page(primitive, &json!({ "x": x, "y": y }))?;
        let verb = if click { "click" } else { "mouse" };
        return Ok(format!("{verb} {x} {y}"));
    }

    let target = rest.first().cloned().unwrap_or_default();
    if target.is_empty() {
        return Err("usage: mouse [click] <index|selector|text> | mouse <x> <y>".to_string());
    }

    let (verb, index) = if let Ok(index) = target.parse::<u64>() {
        let primitive = if click { "click-index" } else { "move-index" };
        page(primitive, &json!({ "index": index }))?;
        (if click { "click" } else { "mouse" }, index.to_string())
    } else {
        let pattern = if click { "click" } else { "hover" };
        point(pattern, &target)?;
        (if click { "click" } else { "mouse" }, target.clone())
    };
    Ok(format!("{verb} {index}"))
}

/// `scroll <pixels|up|down|top|bottom>` - the plugin builds the script.
fn scroll(args: &[String]) -> Result<String, String> {
    let to = arg(args, 0);
    let js = match to.as_str() {
        "" | "down" => "window.scrollBy(0, window.innerHeight * 0.9)".to_string(),
        "up" => "window.scrollBy(0, -window.innerHeight * 0.9)".to_string(),
        "top" => "window.scrollTo(0, 0)".to_string(),
        "bottom" => "window.scrollTo(0, document.body.scrollHeight)".to_string(),
        other => match other.parse::<i64>() {
            Ok(pixels) => format!("window.scrollBy(0, {pixels})"),
            Err(_) => return Err("usage: scroll <pixels|up|down|top|bottom>".to_string()),
        },
    };
    page("scroll", &json!({ "js": js }))?;
    Ok(format!(
        "scroll {}",
        if to.is_empty() { "down" } else { &to }
    ))
}

/// `cookie [get [name]|set <name> <value> [domain] [path]|add <json>|delete <name>|clear]`.
fn cookie(args: &[String]) -> Result<String, String> {
    let sub = arg(args, 0).to_ascii_lowercase();
    match sub.as_str() {
        "" | "get" | "list" => {
            let name = arg(args, 1);
            if name.is_empty() {
                value("cookies", &json!({}))
            } else {
                value("cookie", &json!({ "name": name }))
            }
        }
        "set" => {
            let name = arg(args, 1);
            let cookie_value = arg(args, 2);
            if name.is_empty() {
                return Err("usage: cookie set <name> <value> [domain] [path]".to_string());
            }
            let mut cookie = json!({ "name": name, "value": cookie_value, "path": "/" });
            let domain = arg(args, 3);
            if domain.is_empty() {
                // `Network.setCookie` needs a `url` or a `domain`; fall back to the
                // page's own URL so `cookie set` works on the open page.
                cookie["url"] = json!(value("url", &json!({}))?);
            } else {
                cookie["domain"] = json!(domain);
                let path = arg(args, 4);
                if !path.is_empty() {
                    cookie["path"] = json!(path);
                }
            }
            page(
                "cdp",
                &json!({ "method": "Network.setCookie", "params": cookie }),
            )?;
            Ok(format!("cookie set {name}"))
        }
        "add" | "import" => {
            let raw = arg(args, 1);
            if raw.is_empty() {
                return Err("usage: cookie add <json>".to_string());
            }
            let parsed: Value =
                serde_json::from_str(&raw).map_err(|e| format!("cookie add: invalid JSON: {e}"))?;
            let list = match parsed {
                Value::Array(list) => list,
                other => vec![other],
            };
            for cookie in &list {
                page(
                    "cdp",
                    &json!({ "method": "Network.setCookie", "params": cookie }),
                )?;
            }
            Ok(format!("added {} cookie(s)", list.len()))
        }
        "delete" | "remove" | "rm" => {
            let name = arg(args, 1);
            if name.is_empty() {
                return Err("usage: cookie delete <name>".to_string());
            }
            page(
                "cdp",
                &json!({ "method": "Network.deleteCookies", "params": { "name": name } }),
            )?;
            Ok(format!("cookie delete {name}"))
        }
        "clear" | "clear-all" => {
            page(
                "cdp",
                &json!({ "method": "Network.clearBrowserCookies", "params": {} }),
            )?;
            Ok("cookies cleared".to_string())
        }
        other => Err(format!(
            "usage: cookie [get [name]|set <name> <value> [domain] [path]|add <json>|delete <name>|clear] (got `{other}`)"
        )),
    }
}

/// `geolocation <latitude> <longitude> [accuracy] | geolocation clear`.
fn geolocation(args: &[String]) -> Result<String, String> {
    let first = arg(args, 0);
    if first.eq_ignore_ascii_case("clear") {
        page(
            "cdp",
            &json!({ "method": "Emulation.clearGeolocationOverride", "params": {} }),
        )?;
        return Ok("geolocation cleared".to_string());
    }
    let latitude = first.parse::<f64>().ok();
    let longitude = arg(args, 1).parse::<f64>().ok();
    match (latitude, longitude) {
        (Some(latitude), Some(longitude)) => {
            let accuracy = arg(args, 2).parse::<f64>().unwrap_or(0.0);
            page(
                "cdp",
                &json!({
                    "method": "Emulation.setGeolocationOverride",
                    "params": { "latitude": latitude, "longitude": longitude, "accuracy": accuracy }
                }),
            )?;
            Ok(format!(
                "geolocation {latitude},{longitude} (+/-{accuracy}m)"
            ))
        }
        _ => Err(
            "usage: geolocation <latitude> <longitude> [accuracy] | geolocation clear".to_string(),
        ),
    }
}

/// `storage <local|session> [get [key]|set <key> <value>|clear]`.
fn storage(args: &[String]) -> Result<String, String> {
    let sub = arg(args, 0).to_ascii_lowercase();
    let store = match sub.as_str() {
        "local" => "localStorage",
        "session" => "sessionStorage",
        _ => {
            return Err(
                "usage: storage <local|session> [get [key]|set <key> <value>|clear]".to_string(),
            );
        }
    };
    let action = arg(args, 1).to_ascii_lowercase();
    let evaluate = |js: String| value("evaluate", &json!({ "js": js, "as": "json" }));
    match action.as_str() {
        "" | "get" | "list" => {
            let key = arg(args, 2);
            if key.is_empty() {
                evaluate(format!("Object.fromEntries(Object.entries({store}))"))
            } else {
                Ok(unwrap_json_string(evaluate(format!(
                    "{store}.getItem({})",
                    js_literal(&key)
                ))?))
            }
        }
        "set" => {
            let key = arg(args, 2);
            let value = arg(args, 3);
            if key.is_empty() {
                return Err(format!("usage: storage {sub} set <key> <value>"));
            }
            evaluate(format!(
                "{store}.setItem({}, {})",
                js_literal(&key),
                js_literal(&value)
            ))?;
            Ok(format!("storage {sub} set {key}"))
        }
        "clear" => {
            evaluate(format!("{store}.clear()"))?;
            Ok(format!("storage {sub} cleared"))
        }
        other => Err(format!(
            "usage: storage {sub} [get [key]|set <key> <value>|clear] (got `{other}`)"
        )),
    }
}
