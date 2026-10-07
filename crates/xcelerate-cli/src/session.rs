//! The interactive session: a line-based REPL over one persistent browser + page.
//!
//! The one-shot commands launch a browser, act once, and close it, so there is
//! no continuity between invocations. A session keeps the page alive so steps can
//! be chained the way a person browses: open, skip an ad, click a result, keep
//! reading - without ever relaunching.
//!
//! Each step is echoed to the terminal as it runs. With `--codegen <LANG>` the
//! session also records what it did and, when it ends, renders the run as a
//! script in that language - written to `--codegen-out <PATH>` or printed.

use std::sync::Arc;
use std::time::Duration;

use xcelerate::Page;
use xcelerate_codegen::{Action, Language, Selector};

use crate::cli::{BrowserArgs, CodegenLang};
use crate::launch::launch;

/// Runs the REPL until `quit` or EOF.
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
    let (browser, first) = launch(args, &initial).await?;
    // The session can own several tabs; `page` is the active one.
    let mut page = first.clone();
    let mut tabs: Vec<Arc<Page>> = vec![first];
    let mut active_tab = 0usize;

    // Raise the input gate: while the run drives the page the human cannot click,
    // type or scroll it. Each step lowers the gate only for its own CDP input.
    if args.live() {
        crate::cursor::set_gate(&page, true).await;
    }

    if interactive {
        println!("xcelerate session - one browser, many steps.");
        if args.live() {
            println!("live mode: the screen is locked to the AI.");
        }
        println!("type `help` for commands, `quit` to exit.\n");
    } else {
        println!("# session start: {}", page.url().await.unwrap_or(initial));
    }

    if let Some(lang) = args.codegen {
        println!(
            "codegen: recording this run as {}.",
            codegen_language(lang).label()
        );
    }

    // The recorded run, rendered as code when `--codegen` is set.
    let mut recording: Vec<Action> = Vec::new();
    // The last selector a step acted on, so focus-scoped steps (`press`,
    // `submit`) can still be recorded: `fill #box text` then `submit` becomes a
    // Fill plus a Press of Enter on `#box`.
    let mut last_target: Option<Selector> = None;

    let mut lines = tokio::io::BufReader::new(tokio::io::stdin()).lines();
    'session: loop {
        if interactive {
            print!("xcelerate> ");
            let _ = std::io::stdout().flush();
        }

        // Wait for the next command (or EOF, which ends the run).
        let line = match lines.next_line().await {
            Ok(Some(line)) => line,
            Ok(None) => break 'session, // EOF (Ctrl-D, or piped input ended)
            Err(error) => {
                println!("input error: {error}");
                break 'session;
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
        // Self-heal: a step can fail because the CDP session was detached by a
        // target churn. Re-attach the page and run the same step exactly once
        // more, instead of leaving the session dead for the rest of the run.
        let mut healed = false;
        let mut outcome: Result<(), Box<dyn std::error::Error>>;
        loop {
            outcome = async {
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
                "press" | "submit" | "send" => {
                    // `submit` is the productive path: finish typing, then submit
                    // from the field you are already in instead of hunting for a
                    // Send button. It is Enter on the focused element.
                    let key = if verb == "press" {
                        rest.clone()
                    } else {
                        "Enter".to_string()
                    };
                    if key.is_empty() {
                        println!("usage: press <key>   (sends to the focused element)");
                        return Ok(());
                    }
                    let element = Arc::clone(&page)
                        .evaluate_handle("document.activeElement".to_string())
                        .await?;
                    crate::cursor::set_driving(&page, true).await;
                    element.press(key.clone()).await?;
                    println!("pressed {key}");
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
                "wait-stable" | "stable" => {
                    // Wait only as long as the page keeps changing, then stop -
                    // the productive wait for a page that renders progressively.
                    let quiet = rest.parse::<u64>().unwrap_or(700);
                    match page.wait_for_dom_stable(quiet, 60_000).await {
                        Ok(()) => println!("dom settled ({quiet}ms quiet)"),
                        Err(error) => println!("not settled: {error}"),
                    }
                    Ok(())
                }
                "wait-idle" | "idle" => {
                    let quiet = rest.parse::<u64>().unwrap_or(500);
                    match page.wait_for_network_idle(quiet, 60_000).await {
                        Ok(()) => println!("network idle ({quiet}ms quiet)"),
                        Err(error) => println!("not idle: {error}"),
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
                // DOM clicks that never move the mouse: transient menus close on
                // `mouseleave`, so a curved mouse move to a submenu item can shut
                // the menu before the click lands. These dispatch the click
                // straight on the element instead.
                "tap" | "click-js" | "js-click" | "dom-click" => {
                    if rest.is_empty() {
                        println!("usage: tap <selector>   (DOM click, no mouse movement)");
                    } else {
                        let element = Arc::clone(&page).wait_for_selector(rest.clone()).await?;
                        // Lower the gate first: it blocks click events, and a DOM
                        // click is still a click event.
                        crate::cursor::set_driving(&page, true).await;
                        element.click().await?;
                        println!("tapped {rest}");
                    }
                    Ok(())
                }
                "tap-text" | "click-text-js" | "text-tap" => {
                    if rest.is_empty() {
                        println!("usage: tap-text <visible text>");
                    } else {
                        let needle = serde_json::to_string(&rest)?;
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
                            // Lower the gate first: it blocks click events.
                            crate::cursor::set_driving(&page, true).await;
                            element.click().await?;
                            println!("tapped text {rest:?}");
                        } else {
                            println!("no visible element contains {rest:?}");
                        }
                    }
                    Ok(())
                }
                "click-xy" | "click-at" | "xy" => {
                    // Raw coordinate click: for canvases, maps and embedded
                    // (cross-origin) widgets that DOM selectors cannot reach.
                    let mut parts = rest.split_whitespace();
                    let x = parts.next().and_then(|v| v.parse::<f64>().ok());
                    let y = parts.next().and_then(|v| v.parse::<f64>().ok());
                    match (x, y) {
                        (Some(x), Some(y)) => {
                            crate::cursor::set_driving(&page, true).await;
                            Arc::clone(&page).click_mouse(x, y).await?;
                            println!("clicked at ({x}, {y})");
                        }
                        _ => println!("usage: click-xy <x> <y>"),
                    }
                    Ok(())
                }
                "upload" | "set-input-files" => {
                    // `<input type="file">` cannot be set from page JS; this uses
                    // CDP `DOM.setFileInputFiles` under the hood.
                    match rest.split_once(char::is_whitespace) {
                        Some((selector, path)) => {
                            let path = path.trim();
                            let files = serde_json::json!([path]).to_string();
                            Arc::clone(&page)
                                .set_input_files(selector.to_string(), files)
                                .await?;
                            println!("set {selector} <- {path}");
                        }
                        None => println!("usage: upload <selector> <path>"),
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
                "challenge" | "detect" => {
                    let report = page.detect_challenge().await?;
                    println!("{}", report.to_json());
                    Ok(())
                }
                "await-human" | "await" | "human" => {
                    // Pause for a person to clear an anti-bot challenge in the
                    // browser, then continue. xcelerate detects challenges; it
                    // does not solve them. The input gate is lowered first, or
                    // the person could not click the challenge either.
                    let limit = rest.parse::<u64>().unwrap_or(60);
                    crate::cursor::set_gate(&page, false).await;
                    println!("waiting up to {limit}s for a human to clear the challenge...");
                    let started = std::time::Instant::now();
                    loop {
                        let report = page.detect_challenge().await?;
                        if !report.detected {
                            println!("challenge cleared after {}s", started.elapsed().as_secs());
                            break;
                        }
                        if started.elapsed().as_secs() >= limit {
                            println!(
                                "still challenged after {limit}s ({})",
                                report.vendors.join(", ")
                            );
                            break;
                        }
                        tokio::time::sleep(Duration::from_millis(1000)).await;
                    }
                    // Re-raise the gate now that the human is done.
                    crate::cursor::set_gate(&page, true).await;
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
                "new-tab" | "newtab" | "tab-new" => {
                    let url = if rest.is_empty() {
                        "about:blank".to_string()
                    } else {
                        rest.clone()
                    };
                    let opened = Arc::clone(&browser).new_page(url).await?;
                    if args.live() {
                        let _ = crate::cursor::install(&opened).await;
                        crate::cursor::set_gate(&opened, true).await;
                    }
                    tabs.push(Arc::clone(&opened));
                    active_tab = tabs.len() - 1;
                    page = opened;
                    // Bring the new tab to the front so the switch is visible.
                    let _ = page
                        .execute_cdp_cmd("Page.bringToFront".to_string(), "{}".to_string())
                        .await;
                    println!(
                        "opened tab {active_tab} -> {}",
                        page.url().await.unwrap_or_default()
                    );
                    Ok(())
                }
                "switch" | "tab" | "use" => {
                    if rest.is_empty() {
                        // No argument cycles to the next tab.
                        active_tab = (active_tab + 1) % tabs.len();
                        page = tabs[active_tab].clone();
                    } else if let Ok(index) = rest.parse::<usize>() {
                        if index < tabs.len() {
                            active_tab = index;
                            page = tabs[index].clone();
                        } else {
                            println!("no tab {index} (have {} open)", tabs.len());
                            return Ok(());
                        }
                    } else if let Some(position) =
                        tabs.iter().position(|tab| tab.target_id() == rest)
                    {
                        active_tab = position;
                        page = tabs[position].clone();
                    } else {
                        // A target this session did not open (for example a popup).
                        match Arc::clone(&browser).attach_page(rest.clone()).await {
                            Ok(attached) => {
                                if args.live() {
                                    let _ = crate::cursor::install(&attached).await;
                                    crate::cursor::set_gate(&attached, true).await;
                                }
                                tabs.push(Arc::clone(&attached));
                                active_tab = tabs.len() - 1;
                                page = attached;
                            }
                            Err(error) => {
                                println!("could not switch to {rest}: {error}");
                                return Ok(());
                            }
                        }
                    }
                    println!(
                        "switched to tab {active_tab} -> {}",
                        page.url().await.unwrap_or_default()
                    );
                    // Bring the tab to the front so the switch is actually visible.
                    let _ = page
                        .execute_cdp_cmd("Page.bringToFront".to_string(), "{}".to_string())
                        .await;
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
                        // Drop it from the tab list, keeping the active tab valid.
                        if let Some(position) = tabs.iter().position(|tab| tab.target_id() == rest)
                            && tabs.len() > 1
                        {
                            tabs.remove(position);
                            if active_tab >= tabs.len() {
                                active_tab = tabs.len() - 1;
                            }
                            page = tabs[active_tab].clone();
                        }
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
            match &outcome {
                Ok(()) => break,
                Err(error) if !healed && is_session_error(error.as_ref()) => {
                    healed = true;
                    match Arc::clone(&browser).attach_page(page.target_id()).await {
                        Ok(fresh) => {
                            if args.live() {
                                let _ = crate::cursor::install(&fresh).await;
                                crate::cursor::set_gate(&fresh, true).await;
                            }
                            println!(
                                "session detached; re-attached the page and retrying the step"
                            );
                            page = fresh;
                        }
                        Err(_) => break,
                    }
                }
                Err(_) => break,
            }
        }

        let ok = outcome.is_ok();
        if quit {
            break;
        }
        if let Err(error) = outcome {
            // A failed step must not tear down the session: report and continue.
            println!("error: {error}");
        }

        // Record the step for `--codegen`, when one was requested.
        if ok
            && args.codegen.is_some()
            && let Some(action) = record_action(&verb, &rest, &mut last_target)
        {
            recording.push(action);
        }

        // Re-raise the cursor gate: outside a mouse step the cursor stays inert.
        crate::cursor::set_driving(&page, false).await;
    }

    let _ = tokio::time::timeout(Duration::from_secs(5), browser.close()).await;
    println!("session closed.");

    if let Some(lang) = args.codegen {
        emit_codegen(lang, &recording, args.codegen_out.as_deref());
    }
    Ok(())
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
         \x20 tap <selector>                 DOM-click without moving the mouse\n\
         \x20 tap-text <text>                DOM-click a control by text (no mouse move)\n\
         \x20 click-xy <x> <y>              raw coordinate click (canvas / embedded)\n\
         \x20 upload <selector> <path>      set a file input to a local file\n\
         \x20 fill <selector> <text>         focus + type slowly (50 ms/char)\n\
         \x20 type <text>                    type into the focused element\n\
         \x20 press <key>                    press a key on the focused element\n\
         \x20 submit                         press Enter on the focused element (send)\n\
         \x20 hover <selector>               move the mouse over an element\n\
         \x20 scroll <px|up|down|top|bottom> scroll the page\n\
         \x20 find <text>                    how many elements contain the text\n\
         \x20 wait <ms|selector>             sleep, or wait for an element\n\
         \x20 wait-stable [ms]               wait until the DOM stops changing\n\
         \x20 wait-idle [ms]                 wait until the network goes quiet\n\
         \x20 challenge                       detect an anti-bot / CAPTCHA challenge\n\
         \x20 await-human [s]                 wait until a person clears the challenge\n\
         \x20 eval <js>                      evaluate JavaScript, print the result\n\
         \x20 guard <path.js>               block popups/ads on this page and every new one\n\
         \x20 tabs                          list targets (id, type, url)\n\
         \x20 new-tab [url]                 open a new tab and make it active\n\
         \x20 switch <n|targetId>           switch the active tab (no arg cycles)\n\
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

/// Whether an error means the CDP session was detached, so re-attaching the
/// page and retrying the step can recover it.
fn is_session_error(error: &(dyn std::error::Error + 'static)) -> bool {
    let text = error.to_string().to_ascii_lowercase();
    text.contains("session with given id")
        || text.contains("-32001")
        || text.contains("session detached")
}

/// Maps a recorded session command into a codegen [`Action`], when it maps
/// cleanly, tracking the last selector so focus-scoped steps can be recorded.
///
/// Index clicks (`click 3`) and bare `type` carry no portable locator, so they
/// are left out. `press` / `submit` act on the focused element, which after a
/// `fill` is the field just typed into, so they are recorded against `last`.
fn record_action(verb: &str, rest: &str, last: &mut Option<Selector>) -> Option<Action> {
    let rest = rest.trim();
    match verb {
        "open" | "goto" if !rest.is_empty() => Some(Action::Navigate {
            url: rest.to_string(),
        }),
        "click" if !rest.is_empty() && rest.parse::<u32>().is_err() => {
            let selector = Selector::parse(rest);
            *last = Some(selector.clone());
            Some(Action::Click { selector })
        }
        "click-text" | "text-click" if !rest.is_empty() => {
            let selector = Selector::text(rest);
            *last = Some(selector.clone());
            Some(Action::Click { selector })
        }
        "tap" | "click-js" | "js-click" | "dom-click" if !rest.is_empty() => {
            let selector = Selector::parse(rest);
            *last = Some(selector.clone());
            Some(Action::Click { selector })
        }
        "tap-text" | "click-text-js" | "text-tap" if !rest.is_empty() => {
            let selector = Selector::text(rest);
            *last = Some(selector.clone());
            Some(Action::Click { selector })
        }
        "hover" if !rest.is_empty() => {
            let selector = Selector::parse(rest);
            *last = Some(selector.clone());
            Some(Action::Hover { selector })
        }
        "wait" | "sleep" if !rest.is_empty() && rest.parse::<u64>().is_err() => {
            let selector = Selector::parse(rest);
            *last = Some(selector.clone());
            Some(Action::WaitFor { selector })
        }
        "fill" => rest
            .split_once(char::is_whitespace)
            .map(|(selector, text)| {
                let selector = Selector::parse(selector);
                *last = Some(selector.clone());
                Action::Fill {
                    selector,
                    text: text.trim().to_string(),
                }
            }),
        // `press` / `submit` act on the focused element. After a `fill` that is
        // the field just typed into, so record it against the last selector -
        // otherwise the step would be lost from the generated script.
        "press" | "submit" | "send" => {
            let key = if verb == "press" {
                rest.to_string()
            } else {
                "Enter".to_string()
            };
            last.clone().map(|selector| Action::Press { selector, key })
        }
        "shot" | "screenshot" | "shot-full" | "screenshot-full" => Some(Action::Screenshot {
            path: if rest.is_empty() {
                "screenshot.png".to_string()
            } else {
                rest.to_string()
            },
        }),
        _ => None,
    }
}

/// Maps the CLI `--codegen` language onto the codegen target.
fn codegen_language(lang: CodegenLang) -> Language {
    match lang {
        CodegenLang::Rust => Language::Rust,
        CodegenLang::Python => Language::Python,
        CodegenLang::Javascript => Language::JavaScript,
        CodegenLang::Csharp => Language::CSharp,
        CodegenLang::Kotlin => Language::Kotlin,
        CodegenLang::Java => Language::Java,
        CodegenLang::Swift => Language::Swift,
        CodegenLang::Ruby => Language::Ruby,
        CodegenLang::Dart => Language::Dart,
        CodegenLang::Go => Language::Go,
        CodegenLang::Powershell => Language::PowerShell,
    }
}

/// Renders the recorded run and writes it to `output`, or prints it when no
/// path was given.
fn emit_codegen(lang: CodegenLang, actions: &[Action], output: Option<&std::path::Path>) {
    let language = codegen_language(lang);
    let code = language.generate(actions);
    match output {
        Some(path) => match std::fs::write(path, code.as_bytes()) {
            Ok(()) => println!(
                "codegen: wrote {} ({} actions, {})",
                path.display(),
                actions.len(),
                language.label()
            ),
            Err(error) => eprintln!("codegen: could not write {}: {error}", path.display()),
        },
        None => println!(
            "\n# ---- generated {} ({} actions) ----\n{code}",
            language.label(),
            actions.len()
        ),
    }
}
