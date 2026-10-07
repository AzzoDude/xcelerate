//! The interactive session: a line-based REPL over one persistent browser + page.
//!
//! The one-shot commands launch a browser, act once, and close it, so there is
//! no continuity between invocations. A session keeps the page alive so steps can
//! be chained the way a person browses: open, skip an ad, click a result, keep
//! reading - without ever relaunching.
//!
//! Every step is mirrored into the native overlay (see `crate::overlay`), which
//! also carries the human's Stop / Pause / screenshot requests back to this loop.

use std::sync::Arc;
use std::time::Duration;

use xcelerate::Page;

use crate::cli::BrowserArgs;
use crate::launch::launch;
use crate::overlay::{Bounds, OverlayHandle, Step, record_step};

/// Runs the REPL until `quit`, EOF, or a Stop from the overlay.
pub async fn run_session(
    args: &BrowserArgs,
    start: Option<String>,
    overlay: OverlayHandle,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::{IsTerminal, Write};
    use tokio::io::AsyncBufReadExt;

    // A real terminal gets a prompt; a pipe (scripts, agents, CI) gets a clean
    // transcript where every command is echoed with `>>>` instead.
    let interactive = std::io::stdin().is_terminal();
    let initial = start.unwrap_or_else(|| "about:blank".to_string());
    let (browser, page) = launch(args, &initial).await?;

    // Keep the overlay pinned to the browser window while it is on screen, so it
    // moves and resizes with the browser instead of floating at a fixed spot.
    // Skipped when the browser is headless: there is no window to follow.
    let follow = if args.live() {
        Some(tokio::spawn(follow_browser(
            Arc::clone(&page),
            overlay.clone(),
        )))
    } else {
        None
    };

    // Raise the input gate: while the run drives the page the human cannot click,
    // type or scroll it. Each step lowers the gate only for its own CDP input.
    if args.live() {
        crate::cursor::set_gate(&page, true).await;
    }

    if interactive {
        println!("xcelerate session - one browser, many steps.");
        if args.live() {
            println!(
                "overlay is on: the control bar and the interceptor log float over the browser."
            );
        }
        println!("type `help` for commands, `quit` to exit.\n");
    } else {
        println!("# session start: {}", page.url().await.unwrap_or(initial));
    }

    let mut lines = tokio::io::BufReader::new(tokio::io::stdin()).lines();
    'session: loop {
        if interactive {
            print!("xcelerate> ");
            let _ = std::io::stdout().flush();
        }

        // Wait for the next command, but also notice the human closing the
        // overlay: that ends the run - and shuts the browser down - rather than
        // leaving the session blocked on input.
        let line = {
            let mut ticker = tokio::time::interval(Duration::from_millis(250));
            loop {
                tokio::select! {
                    next = lines.next_line() => match next {
                        Ok(Some(line)) => break line,
                        Ok(None) => break 'session, // EOF (Ctrl-D, or piped input ended)
                        Err(error) => {
                            println!("input error: {error}");
                            break 'session;
                        }
                    },
                    _ = ticker.tick() => {
                        if overlay.is_closed() {
                            println!("overlay closed; shutting down");
                            break 'session;
                        }
                    }
                }
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
                    if let Ok(index) = rest.parse::<u32>() {
                        crate::cursor::set_driving(&page, true).await;
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
                        crate::cursor::set_driving(&page, true).await;
                        element.click_mouse().await?;
                        tokio::time::sleep(Duration::from_millis(600)).await;
                        println!("clicked {rest} -> {}", page.url().await.unwrap_or_default());
                    }
                    Ok(())
                }
                "fill" => match rest.split_once(char::is_whitespace) {
                    Some((selector, text)) => {
                        let element = Arc::clone(&page)
                            .wait_for_selector(selector.to_string())
                            .await?;
                        crate::cursor::set_driving(&page, true).await;
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
                        let element = Arc::clone(&page)
                            .evaluate_handle("document.activeElement".to_string())
                            .await?;
                        crate::cursor::set_driving(&page, true).await;
                        let count = rest.chars().count();
                        element.type_text(rest.clone()).await?;
                        println!("typed {count} chars into the focused element");
                    }
                    Ok(())
                }
                "press" => {
                    if rest.is_empty() {
                        println!("usage: press <key>   (sends to the focused element)");
                        return Ok(());
                    }
                    let element = Arc::clone(&page)
                        .evaluate_handle("document.activeElement".to_string())
                        .await?;
                    crate::cursor::set_driving(&page, true).await;
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
                            crate::cursor::set_driving(&page, true).await;
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
                        let element = Arc::clone(&page).wait_for_selector(rest.clone()).await?;
                        crate::cursor::set_driving(&page, true).await;
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

        let ok = outcome.is_ok();
        if quit {
            break;
        }
        if let Err(error) = outcome {
            // A failed step must not tear down the session: report and continue.
            println!("error: {error}");
        }

        // Record the step for the codegen overlay, then re-raise the cursor gate:
        // outside a mouse step the cursor stays inert.
        if ok && let Some(step) = Step::parse(&verb, &rest) {
            record_step(&overlay, step);
        }
        crate::cursor::set_driving(&page, false).await;
    }

    if let Some(handle) = follow {
        handle.abort();
    }
    browser.close().await.ok();
    println!("session closed.");
    Ok(())
}

/// Follow cadence while the browser is moving (the fastest polling rate).
const FOLLOW_ACTIVE: Duration = Duration::from_millis(6);
/// Follow cadence once the browser has settled, to keep idle traffic low.
const FOLLOW_IDLE: Duration = Duration::from_millis(24);
/// Consecutive unchanged reads before backing off to [`FOLLOW_IDLE`].
const FOLLOW_SETTLE: u32 = 4;
/// Consecutive failed reads that mean the browser is gone.
const FOLLOW_MISS_LIMIT: u32 = 40;

/// Pins the overlay to the browser window, re-reading its rectangle often enough
/// to track a drag, and closes the overlay once the browser is gone.
async fn follow_browser(page: Arc<Page>, overlay: OverlayHandle) {
    // Resolve the browser window once; without one there is nothing to follow.
    let Some(window_id) = browser_window_id(&page).await else {
        return;
    };
    let mut misses = 0u32;
    let mut stable = FOLLOW_SETTLE + 1;
    let mut last: Option<Bounds> = None;
    loop {
        // A dead connection can leave a CDP request pending forever, so every
        // probe is bounded: a timeout counts as a miss like any other failure.
        let bounds = tokio::time::timeout(
            std::time::Duration::from_millis(500),
            window_bounds(&page, &window_id),
        )
        .await
        .ok()
        .flatten();
        match bounds {
            Some(bounds) => {
                // Only push a change, and go back to the fast cadence while it is
                // moving; settle to a slower rate so an idle browser costs little.
                if last.is_none_or(|previous| moved(previous, bounds)) {
                    overlay.set_bounds(Some(bounds));
                    last = Some(bounds);
                    stable = 0;
                } else {
                    stable = stable.saturating_add(1);
                }
                misses = 0;
            }
            None => {
                misses += 1;
                // A stretch of silence: the browser is closed, so the overlay has
                // nothing left to sit on and goes with it.
                if misses >= FOLLOW_MISS_LIMIT {
                    overlay.request_close();
                    return;
                }
            }
        }
        let interval = if stable > FOLLOW_SETTLE {
            FOLLOW_IDLE
        } else {
            FOLLOW_ACTIVE
        };
        tokio::time::sleep(interval).await;
    }
}

/// Whether two window rectangles differ enough to be worth pushing.
fn moved(a: Bounds, b: Bounds) -> bool {
    (a.x - b.x).abs() > 0.5
        || (a.y - b.y).abs() > 0.5
        || (a.width - b.width).abs() > 0.5
        || (a.height - b.height).abs() > 0.5
}

/// The window id that contains this page, or `None` when there is no real window.
async fn browser_window_id(page: &Arc<Page>) -> Option<serde_json::Value> {
    let params = serde_json::json!({ "targetId": page.target_id() }).to_string();
    let raw = page
        .execute_cdp_cmd("Browser.getWindowForTarget".to_string(), params)
        .await
        .ok()?;
    let value: serde_json::Value = serde_json::from_str(&raw).ok()?;
    value.get("windowId").cloned()
}

/// The browser window's screen rectangle, or `None` when CDP cannot report it
/// (headless, or the browser is gone).
async fn window_bounds(page: &Arc<Page>, window_id: &serde_json::Value) -> Option<Bounds> {
    let params = serde_json::json!({ "windowId": window_id }).to_string();
    let raw = page
        .execute_cdp_cmd("Browser.getWindowBounds".to_string(), params)
        .await
        .ok()?;
    let value: serde_json::Value = serde_json::from_str(&raw).ok()?;
    parse_bounds(value.get("bounds")?)
}

/// Reads `{left, top, width, height}` from a CDP `Bounds` object.
fn parse_bounds(bounds: &serde_json::Value) -> Option<Bounds> {
    let x = bounds.get("left")?.as_f64()?;
    let y = bounds.get("top")?.as_f64()?;
    let width = bounds.get("width")?.as_f64()?;
    let height = bounds.get("height")?.as_f64()?;
    if width <= 1.0 || height <= 1.0 {
        return None;
    }
    Some(Bounds {
        x,
        y,
        width,
        height,
    })
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
