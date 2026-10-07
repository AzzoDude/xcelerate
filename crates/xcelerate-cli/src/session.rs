//! The interactive session: a line-based REPL over one persistent browser + page.
//!
//! The one-shot commands launch a browser, act once, and close it, so there is
//! no continuity between invocations. A session keeps the page alive so steps can
//! be chained the way a person browses: open, skip an ad, click a result, keep
//! reading - without ever relaunching.

use std::sync::Arc;
use std::time::Duration;

use xcelerate::Page;

use crate::cli::BrowserArgs;
use crate::launch::launch;
use crate::overlay;

/// Runs the REPL until `quit`, EOF, or a Stop from the page HUD.
pub async fn run_session(
    args: &BrowserArgs,
    start: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::{IsTerminal, Write};
    use tokio::io::AsyncBufReadExt;

    // A real terminal gets a prompt; a pipe (scripts, agents, CI) gets a clean
    // transcript where every command is echoed with `>>>` instead.
    let interactive = std::io::stdin().is_terminal();
    let initial = start.unwrap_or_else(|| "about:blank".to_string());
    let (browser, page) = launch(args, &initial).await?;
    let mut hud_active = args.hud;

    if interactive {
        println!("xcelerate session - one browser, many steps.");
        if hud_active {
            println!("HUD is on: cursor, bottom bar, and the interceptor panel.");
        }
        println!("type `help` for commands, `quit` to exit.\n");
    } else {
        println!("# session start: {}", page.url().await.unwrap_or(initial));
    }

    let mut lines = tokio::io::BufReader::new(tokio::io::stdin()).lines();
    loop {
        if interactive {
            print!("xcelerate> ");
            let _ = std::io::stdout().flush();
        }

        let line = match lines.next_line().await {
            Ok(Some(line)) => line,
            Ok(None) => break, // EOF (Ctrl-D, or the end of piped input)
            Err(error) => {
                println!("input error: {error}");
                break;
            }
        };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if !interactive {
            println!(">>> {line}");
        }

        let (verb, rest) = match line.split_once(char::is_whitespace) {
            Some((verb, rest)) => (verb.to_ascii_lowercase(), rest.trim().to_string()),
            None => (line.to_ascii_lowercase(), String::new()),
        };

        // The whole dispatch runs inside an async block so a `?` failure is
        // captured here rather than propagating out of the session.
        let started = std::time::Instant::now();
        let mut quit = false;
        let outcome: Result<(), Box<dyn std::error::Error>> = async {
            match verb.as_str() {
                "help" | "?" => {
                    print_session_help();
                    Ok(())
                }
                "quit" | "exit" | "q" => {
                    quit = true;
                    Ok(())
                }
                "hud" => {
                    match rest.as_str() {
                        "off" => {
                            overlay::remove(&page).await;
                            hud_active = false;
                            println!("HUD off");
                        }
                        "" | "on" => {
                            overlay::install(&page).await?;
                            hud_active = true;
                            println!("HUD on (cursor + bottom bar + interceptor)");
                        }
                        other => println!("usage: hud [on|off]   (got {other:?})"),
                    }
                    Ok(())
                }
                "open" | "goto" => {
                    if rest.is_empty() {
                        println!("usage: open <url>");
                        return Ok(());
                    }
                    page.navigate(rest.clone()).await?;
                    let _ = page.wait_for_navigation().await;
                    let _ = page.wait_for_dom_stable(300, 2_000).await;
                    println!("title: {}", page.title().await.unwrap_or_default());
                    println!("url:   {}", page.url().await.unwrap_or_default());
                    Ok(())
                }
                "title" => {
                    println!("{}", page.title().await?);
                    Ok(())
                }
                "url" => {
                    println!("{}", page.url().await?);
                    Ok(())
                }
                "text" => {
                    let text = page
                        .evaluate_string("document.body ? document.body.innerText : ''".to_string())
                        .await?;
                    println!("{text}");
                    Ok(())
                }
                "snapshot" | "snap" => {
                    // `snapshot_json` also refreshes the index map `click` reads.
                    let json = page.snapshot_json().await?;
                    print_snapshot(&json);
                    Ok(())
                }
                "click" => {
                    if rest.is_empty() {
                        println!("usage: click <index|selector>   (index comes from `snapshot`)");
                        return Ok(());
                    }
                    // The bar dims and becomes click-through while the mouse is being
                    // driven, so an automated click cannot land on a HUD control.
                    set_driving(&page, true).await;
                    if let Ok(index) = rest.parse::<u32>() {
                        // The index map only exists after a snapshot; build it on
                        // first use so `click 3` works even if `snapshot` was never run.
                        if Arc::clone(&page).click_index(index).await.is_err() {
                            let _ = page.snapshot_json().await?;
                            Arc::clone(&page).click_index(index).await?;
                        }
                        tokio::time::sleep(Duration::from_millis(600)).await;
                        println!(
                            "clicked [{index}] -> {}",
                            page.url().await.unwrap_or_default()
                        );
                    } else {
                        let element = Arc::clone(&page).wait_for_selector(rest.clone()).await?;
                        element.click_mouse().await?;
                        tokio::time::sleep(Duration::from_millis(600)).await;
                        println!("clicked {rest} -> {}", page.url().await.unwrap_or_default());
                    }
                    Ok(())
                }
                "fill" => match rest.split_once(char::is_whitespace) {
                    Some((selector, text)) => {
                        // Typing emits page input events; mark them as agent-driven
                        // so the HUD's interceptor does not also record them as yours.
                        set_driving(&page, true).await;
                        let element = Arc::clone(&page)
                            .wait_for_selector(selector.to_string())
                            .await?;
                        let count = text.chars().count();
                        element.type_text(text.to_string()).await?;
                        println!("typed {count} chars into {selector}");
                        Ok(())
                    }
                    None => {
                        println!("usage: fill <selector> <text>");
                        Ok(())
                    }
                },
                "type" => {
                    if rest.is_empty() {
                        println!("usage: type <text>   (types into the focused element)");
                    } else {
                        set_driving(&page, true).await;
                        let element = Arc::clone(&page)
                            .evaluate_handle("document.activeElement".to_string())
                            .await?;
                        let count = rest.chars().count();
                        element.type_text(rest.clone()).await?;
                        println!("typed {count} chars into the focused element");
                    }
                    Ok(())
                }
                "press" => {
                    set_driving(&page, true).await;
                    let element = Arc::clone(&page)
                        .evaluate_handle("document.activeElement".to_string())
                        .await?;
                    element.press(rest.clone()).await?;
                    println!("pressed {rest}");
                    Ok(())
                }
                "eval" | "js" => {
                    let raw = page.evaluate_json(rest.clone()).await?;
                    // Unwrap a top-level string so `eval document.title` reads cleanly.
                    match serde_json::from_str::<serde_json::Value>(&raw) {
                        Ok(serde_json::Value::String(text)) => println!("{text}"),
                        _ => println!("{raw}"),
                    }
                    Ok(())
                }
                "shot" | "screenshot" => {
                    let path = if rest.is_empty() {
                        "screenshot.png".to_string()
                    } else {
                        rest.clone()
                    };
                    let png = page.screenshot().await?;
                    std::fs::write(&path, &png)?;
                    println!("wrote {path} ({} bytes)", png.len());
                    Ok(())
                }
                "shot-full" | "screenshot-full" => {
                    let path = if rest.is_empty() {
                        "screenshot.png".to_string()
                    } else {
                        rest.clone()
                    };
                    let png = page.screenshot_full().await?;
                    std::fs::write(&path, &png)?;
                    println!("wrote {path} ({} bytes)", png.len());
                    Ok(())
                }
                "back" => {
                    page.go_back().await?;
                    let _ = page.wait_for_navigation().await;
                    println!("{}", page.url().await.unwrap_or_default());
                    Ok(())
                }
                "reload" => {
                    page.reload().await?;
                    let _ = page.wait_for_navigation().await;
                    Ok(())
                }
                "wait" | "sleep" => {
                    if let Ok(ms) = rest.parse::<u64>() {
                        tokio::time::sleep(Duration::from_millis(ms)).await;
                        println!("waited {ms}ms");
                    } else if rest.is_empty() {
                        println!("usage: wait <ms|selector>");
                    } else {
                        let started = std::time::Instant::now();
                        match Arc::clone(&page).wait_for_selector(rest.clone()).await {
                            Ok(_) => {
                                println!("found {rest} after {}ms", started.elapsed().as_millis());
                            }
                            Err(_) => println!("timed out waiting for {rest}"),
                        }
                    }
                    Ok(())
                }
                "click-text" | "text-click" => {
                    if rest.is_empty() {
                        println!("usage: click-text <visible text>");
                    } else {
                        set_driving(&page, true).await;
                        let needle = serde_json::to_string(&rest)?;
                        // Only real controls: a large wrapper div matched by text
                        // would be clicked at its centre, which is not the button.
                        let decl = "const els=[...document.querySelectorAll('a,button,[role=\"button\"],[role=\"link\"],summary,input[type=\"submit\"]')];";
                        let expr = format!(
                            "els.find(e=>e.offsetParent!==null&&(((e.innerText||'')+' '+(e.getAttribute('aria-label')||'')).toLowerCase().includes({needle}.toLowerCase())))"
                        );
                        if page
                            .evaluate_bool(format!("(() => {{ {decl} return !!({expr}); }})()"))
                            .await?
                        {
                            let element = Arc::clone(&page)
                                .evaluate_handle(format!("(() => {{ {decl} return {expr}; }})()"))
                                .await?;
                            element.click_mouse().await?;
                            tokio::time::sleep(Duration::from_millis(600)).await;
                            println!(
                                "clicked text {rest:?} -> {}",
                                page.url().await.unwrap_or_default()
                            );
                        } else {
                            println!("no visible element contains {rest:?}");
                        }
                    }
                    Ok(())
                }
                "hover" => {
                    if rest.is_empty() {
                        println!("usage: hover <selector>");
                    } else {
                        set_driving(&page, true).await;
                        let element = Arc::clone(&page).wait_for_selector(rest.clone()).await?;
                        element.hover_mouse().await?;
                        println!("hovered {rest}");
                    }
                    Ok(())
                }
                "scroll" => {
                    let js = match rest.as_str() {
                        "" | "down" => "window.scrollBy(0, window.innerHeight * 0.9)".to_string(),
                        "up" => "window.scrollBy(0, -window.innerHeight * 0.9)".to_string(),
                        "top" => "window.scrollTo(0, 0)".to_string(),
                        "bottom" => "window.scrollTo(0, document.body.scrollHeight)".to_string(),
                        other => match other.parse::<i64>() {
                            Ok(pixels) => format!("window.scrollBy(0, {pixels})"),
                            Err(_) => {
                                println!("usage: scroll <pixels|up|down|top|bottom>");
                                String::new()
                            }
                        },
                    };
                    if !js.is_empty() {
                        set_driving(&page, true).await;
                        let _ = page.evaluate_string(js).await;
                        println!("scrolled {rest}");
                    }
                    Ok(())
                }
                "markdown" | "md" => {
                    println!("{}", page.markdown().await?);
                    Ok(())
                }
                "content" | "html" => {
                    println!("{}", page.content().await?);
                    Ok(())
                }
                "find" => {
                    let count = page.find_text(rest.clone()).await?;
                    println!("{count} match(es) for {rest:?}");
                    Ok(())
                }
                "guard" => {
                    if rest.is_empty() {
                        println!("usage: guard <path-to-js>");
                    } else {
                        match std::fs::read_to_string(&rest) {
                            Ok(source) => {
                                page.add_script_to_evaluate_on_new_document(source.clone())
                                    .await?;
                                // Also run it against the page that is already loaded.
                                let _ = page.evaluate_json(source).await;
                                println!(
                                    "guard installed from {rest} (this page and every new document)"
                                );
                            }
                            Err(error) => println!("cannot read {rest}: {error}"),
                        }
                    }
                    Ok(())
                }
                "tabs" => {
                    let raw = browser.targets().await?;
                    let parsed: serde_json::Value =
                        serde_json::from_str(&raw).unwrap_or(serde_json::Value::Null);
                    // `Target.getTargets` wraps the array in `targetInfos`.
                    let infos = parsed
                        .get("targetInfos")
                        .and_then(serde_json::Value::as_array)
                        .cloned()
                        .or_else(|| parsed.as_array().cloned());
                    match infos {
                        Some(items) => {
                            for item in items {
                                let id = item
                                    .get("targetId")
                                    .and_then(serde_json::Value::as_str)
                                    .unwrap_or("");
                                let kind = item
                                    .get("type")
                                    .and_then(serde_json::Value::as_str)
                                    .unwrap_or("");
                                let url = item
                                    .get("url")
                                    .and_then(serde_json::Value::as_str)
                                    .unwrap_or("");
                                println!("{id}  {kind}  {url}");
                            }
                        }
                        None => println!("{raw}"),
                    }
                    Ok(())
                }
                "close-tab" | "closetab" => {
                    if rest.is_empty() {
                        println!("usage: close-tab <targetId>   (ids come from `tabs`)");
                    } else {
                        let params = serde_json::json!({ "targetId": rest.clone() }).to_string();
                        page.execute_cdp_cmd("Target.closeTarget".to_string(), params)
                            .await?;
                        println!("closed target {rest}");
                    }
                    Ok(())
                }
                other => {
                    println!("unknown command: {other}   (try `help`)");
                    Ok(())
                }
            }
        }
        .await;

        let elapsed_ms = started.elapsed().as_millis();
        let ok = outcome.is_ok();
        if quit {
            break;
        }
        if let Err(error) = outcome {
            // A failed step must not tear down the session: report and continue.
            println!("error: {error}");
        }

        // The action is over: hand the bar back to the human. This is a no-op
        // when no HUD is installed, so it is safe to call unconditionally.
        set_driving(&page, false).await;
        if hud_active {
            log_action(&page, &verb, &rest, ok, elapsed_ms).await;
            if stop_requested(&page).await {
                println!("stop requested from the page HUD");
                break;
            }
            flush_hud_screenshot(&page).await;
            if wait_while_paused(&page).await {
                println!("stopped from the page HUD while paused");
                break;
            }
        }
    }

    browser.close().await.ok();
    println!("session closed.");
    Ok(())
}

/// Records one step in the on-page inspector (no-op without a HUD). A
/// structured record is sent so the panel can show the action, its target, the
/// status and how long it took.
async fn log_action(page: &Arc<Page>, action: &str, detail: &str, ok: bool, ms: u128) {
    let detail: String = {
        let mut text: String = detail.chars().take(120).collect();
        if detail.chars().count() > 120 {
            text.push('…');
        }
        text
    };
    let payload = serde_json::json!({
        "action": action,
        "detail": detail,
        "status": if ok { "ok" } else { "error" },
        "ms": ms.min(u64::MAX as u128) as u64,
    })
    .to_string();
    let _ = page
        .evaluate_json(format!(
            "(()=>{{if(window.__xcelerateLog)window.__xcelerateLog({payload});return true}})()"
        ))
        .await;
}

/// Toggles the HUD's "agent is driving" state. While set, the control bar is
/// dimmed and click-through, so an automated click cannot land on a control and
/// instead reaches the page underneath.
async fn set_driving(page: &Arc<Page>, on: bool) {
    let _ = page
        .evaluate_json(format!(
            "(()=>{{if(window.__xcelerateSetDriving){{window.__xcelerateSetDriving({on});}}return true}})()"
        ))
        .await;
}

/// Whether the page HUD's Stop button was pressed.
async fn stop_requested(page: &Arc<Page>) -> bool {
    page.evaluate_bool("!!window.__xcelerateStop".to_string())
        .await
        .unwrap_or(false)
}

/// Saves a screenshot if the HUD's camera button was pressed, then clears it.
async fn flush_hud_screenshot(page: &Arc<Page>) {
    let requested = page
        .evaluate_bool("!!window.__xcelerateScreenshot".to_string())
        .await
        .unwrap_or(false);
    if !requested {
        return;
    }
    let _ = page
        .evaluate_json("window.__xcelerateScreenshot=false".to_string())
        .await;
    match page.screenshot().await {
        Ok(png) => {
            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let path = format!("hud-screenshot-{stamp}.png");
            match std::fs::write(&path, &png) {
                Ok(()) => println!("HUD requested a screenshot -> {path}"),
                Err(error) => println!("could not write {path}: {error}"),
            }
        }
        Err(error) => println!("HUD screenshot failed: {error}"),
    }
}

/// Blocks while the HUD's Pause is engaged. Returns `true` if Stop fired meanwhile.
async fn wait_while_paused(page: &Arc<Page>) -> bool {
    loop {
        if stop_requested(page).await {
            return true;
        }
        let paused = page
            .evaluate_bool("!!window.__xceleratePause".to_string())
            .await
            .unwrap_or(false);
        if !paused {
            return false;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}

fn print_session_help() {
    println!(
        "commands:\n\
         \x20 open <url>                     navigate the live page\n\
         \x20 back | reload                  history / refresh\n\
         \x20 title | url | text | markdown  read the page\n\
         \x20 content                        raw HTML\n\
         \x20 snapshot                       indexed interactive elements\n\
         \x20 click <index|selector>         click by snapshot index or CSS selector\n\
         \x20 click-text <text>              click the first element containing the text\n\
         \x20 fill <selector> <text>         focus + type slowly (50 ms/char)\n\
         \x20 type <text>                    type into the focused element\n\
         \x20 press <key>                    press a key on the focused element\n\
         \x20 hover <selector>               move the mouse over an element\n\
         \x20 scroll <px|up|down|top|bottom> scroll the page\n\
         \x20 find <text>                    how many elements contain the text\n\
         \x20 wait <ms|selector>             sleep, or wait for an element\n\
         \x20 eval <js>                      evaluate JavaScript, print the result\n\
         \x20 hud [on|off]                   show/hide the HUD (cursor, bar, interceptor)\n\
         \x20 guard <path.js>               block popups/ads on this page and every new one\n\
         \x20 tabs                          list targets (id, type, url)\n\
         \x20 close-tab <targetId>          close one target (ids come from `tabs`)\n\
         \x20 shot [path]                    viewport screenshot (default screenshot.png)\n\
         \x20 shot-full [path]               full-page screenshot\n\
         \x20 help | quit"
    );
}

/// Renders `snapshot_json` output as one line per element, e.g.
/// `[3] <link> "Sign in"`.
fn print_snapshot(json: &str) {
    let parsed: serde_json::Value = match serde_json::from_str(json) {
        Ok(value) => value,
        Err(_) => {
            println!("{json}");
            return;
        }
    };
    let Some(entries) = parsed.as_array() else {
        println!("{json}");
        return;
    };
    if entries.is_empty() {
        println!("(no interactive elements)");
        return;
    }
    for entry in entries {
        let index = entry
            .get("index")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        let role = entry
            .get("role")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        let name = entry
            .get("name")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        let value = entry
            .get("value")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        if value.is_empty() {
            println!("[{index}] <{role}> {name:?}");
        } else {
            println!("[{index}] <{role}> {name:?} = {value:?}");
        }
    }
}
