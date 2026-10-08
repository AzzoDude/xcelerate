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
use crate::interact::{control_by_text, looks_like_a_selector};
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
    if args.cursor_active() {
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
    // Video recording state: `record` writes to this path, `stop-record`
    // finalizes it. Only meaningful while `--codegen` is active.
    let mut video_path: Option<String> = None;
    // Whether the AI has declared the task complete with `done`. While an
    // AI-driven run is still in flight, `quit` is treated as a first-class
    // last resort and gated behind an explicit override.
    let mut job_done = false;

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

        // Parse the line with the XCL lexer, exactly as a `.xcl` script is parsed,
        // so quotes group arguments and a leading `#` is a comment. The verb is the
        // first token; the remaining tokens are rejoined for the arms that take
        // free text.
        let tokens = match crate::xcl::lex::lex_line(line) {
            Ok(crate::xcl::lex::Line::Statement(tokens)) if !tokens.is_empty() => tokens,
            Ok(_) => continue,
            Err(error) => {
                println!("input error: {error}");
                continue;
            }
        };
        let verb = tokens[0].to_ascii_lowercase();
        let rest = tokens[1..].join(" ");

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
                "quit" => {
                    // An AI-driven run is not allowed to walk away from the
                    // browser mid-task. Exiting is a last resort: the job must
                    // be declared done (`done`), or the agent must override it
                    // explicitly (`quit!`) when it has genuinely hit a wall -
                    // missing information or a feature gap it should surface to
                    // the user instead.
                    if args.ai && !job_done {
                        println!(
                            "not exiting: the job is not marked done.\n\
                             \x20 finish the task and run `done`, or if you truly cannot\n\
                             \x20 proceed (missing info / a needed feature), run `quit!` and\n\
                             \x20 tell the user what is blocking you."
                        );
                        return Ok(());
                    }
                    quit = true;
                    Ok(())
                }
                "quit!" | "exit!" | "q!" => {
                    // Explicit last-resort override: the agent acknowledges it
                    // is leaving before the job is complete.
                    if args.ai && !job_done {
                        println!(
                            "exiting before the job is done - please explain to the user\n\
                             \x20 what is missing or blocked."
                        );
                    }
                    quit = true;
                    Ok(())
                }
                "done" | "complete" => {
                    if rest.is_empty() {
                        println!("usage: done [short summary]   (marks the task complete)");
                    } else {
                        println!("marked done: {rest}");
                    }
                    job_done = true;
                    Ok(())
                }
                "open" | "goto" => {
                    if rest.is_empty() {
                        println!("usage: open <url>");
                        return Ok(());
                    }
                    page.navigate(crate::xcl::runtime::normalize_url(&rest)).await?;
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
                        println!("usage: click <index|selector|text>");
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
                    } else if looks_like_a_selector(&rest) {
                        let element = Arc::clone(&page).wait_for_selector(rest.clone()).await?;
                        crate::cursor::set_driving(&page, true).await;
                        element.click_mouse().await?;
                        tokio::time::sleep(Duration::from_millis(600)).await;
                        println!("clicked {rest} -> {}", page.url().await.unwrap_or_default());
                    } else {
                        // Neither a snapshot index nor a selector: match visible text.
                        match control_by_text(&page, &rest).await? {
                            Some(element) => {
                                crate::cursor::set_driving(&page, true).await;
                                element.click_mouse().await?;
                                tokio::time::sleep(Duration::from_millis(600)).await;
                                println!(
                                    "clicked {rest:?} -> {}",
                                    page.url().await.unwrap_or_default()
                                );
                            }
                            None => report_no_text_match(&rest),
                        }
                    }
                    Ok(())
                }
                "fill" => match split_selector_text(&tokens) {
                    Some((selector, text)) => {
                        crate::cursor::set_driving(&page, true).await;
                        // A bare integer is a snapshot index: many framework-
                        // rendered fields (e.g. Facebook signup) expose no stable
                        // selector. Focus it by index, then type.
                        let element = if let Ok(index) = selector.parse::<u32>() {
                            if Arc::clone(&page).click_index(index).await.is_err() {
                                let _ = page.snapshot_json().await;
                                Arc::clone(&page).click_index(index).await?;
                            }
                            Arc::clone(&page)
                                .evaluate_handle("document.activeElement".to_string())
                                .await?
                        } else {
                            Arc::clone(&page).wait_for_selector(selector.clone()).await?
                        };
                        let count = text.chars().count();
                        element.type_text(text).await?;
                        println!("typed {count} chars into {selector}");
                        Ok(())
                    }
                    None => {
                        println!("usage: fill <selector|index> <text>");
                        Ok(())
                    }
                },
                "select" => match split_selector_text(&tokens) {
                    Some((selector, value)) => {
                        // Native `<select>` only (matches option by value or label).
                        // For a custom listbox, click the control to open it, then
                        // `click "<option>"`.
                        let values = serde_json::json!([value]).to_string();
                        Arc::clone(&page)
                            .select_option(selector.clone(), values)
                            .await?;
                        println!("selected {value:?} in {selector}");
                        Ok(())
                    }
                    None => {
                        println!("usage: select <selector> <value>   (native <select>)");
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
                // Unit-suffixed sleeps (the unit is in the verb; the value is a
                // plain number - there is no `2s` literal).
                "wait-ms" => {
                    match crate::xcl::runtime::wait_scaled(&rest, 1).await {
                        Ok(msg) => println!("{msg}"),
                        Err(e) => println!("usage: wait-ms <number>  ({e})"),
                    }
                    Ok(())
                }
                "wait-sec" => {
                    match crate::xcl::runtime::wait_scaled(&rest, 1_000).await {
                        Ok(msg) => println!("{msg}"),
                        Err(e) => println!("usage: wait-sec <number>  ({e})"),
                    }
                    Ok(())
                }
                "wait-min" => {
                    match crate::xcl::runtime::wait_scaled(&rest, 60_000).await {
                        Ok(msg) => println!("{msg}"),
                        Err(e) => println!("usage: wait-min <number>  ({e})"),
                    }
                    Ok(())
                }
                "wait-hr" => {
                    match crate::xcl::runtime::wait_scaled(&rest, 3_600_000).await {
                        Ok(msg) => println!("{msg}"),
                        Err(e) => println!("usage: wait-hr <number>  ({e})"),
                    }
                    Ok(())
                }
                "wait-random" => {
                    let mut bounds = rest.split_whitespace();
                    match (
                        bounds.next().map(crate::xcl::runtime::parse_ms),
                        bounds.next().map(crate::xcl::runtime::parse_ms),
                    ) {
                        (Some(Ok(min)), Some(Ok(max))) => {
                            let ms = crate::xcl::runtime::random_ms(min, max);
                            tokio::time::sleep(Duration::from_millis(ms as u64)).await;
                            println!("waited {ms}ms");
                        }
                        _ => println!("usage: wait-random <min> <max>   (milliseconds)"),
                    }
                    Ok(())
                }
                "wait" | "sleep" => {
                    if let Some(ms) = crate::xcl::runtime::parse_duration_ms(&rest) {
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
                // DOM clicks that never move the mouse: transient menus close on
                // `mouseleave`, so a curved mouse move to a submenu item can shut
                // the menu before the click lands. These dispatch the click
                // straight on the element instead.
                "tap" => {
                    if rest.is_empty() {
                        println!("usage: tap <selector|text>   (DOM click, no mouse movement)");
                    } else if looks_like_a_selector(&rest) {
                        let element = Arc::clone(&page).wait_for_selector(rest.clone()).await?;
                        // Lower the gate first: it blocks click events, and a DOM
                        // click is still a click event.
                        crate::cursor::set_driving(&page, true).await;
                        element.click().await?;
                        println!("tapped {rest}");
                    } else {
                        match control_by_text(&page, &rest).await? {
                            Some(element) => {
                                crate::cursor::set_driving(&page, true).await;
                                element.click().await?;
                                println!("tapped {rest:?}");
                            }
                            None => report_no_text_match(&rest),
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
                    // CDP `DOM.setFileInputFiles` under the hood. Tokens (not
                    // `rest`) so a selector containing spaces stays intact.
                    match split_selector_text(&tokens) {
                        Some((selector, path)) => {
                            let files = serde_json::json!([path]).to_string();
                            Arc::clone(&page)
                                .set_input_files(selector.clone(), files)
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
                "mouse" => {
                    // Move the real cursor without clicking: by snapshot index,
                    // CSS selector, visible text, or raw `x y` coordinates.
                    if rest.is_empty() {
                        println!("usage: mouse <index|selector|text> | mouse <x> <y>");
                    } else if let Some((x, y)) = parse_coordinates(&rest) {
                        crate::cursor::set_driving(&page, true).await;
                        Arc::clone(&page).move_mouse(x, y).await?;
                        println!("moved to ({x}, {y})");
                    } else if let Ok(index) = rest.parse::<u32>() {
                        crate::cursor::set_driving(&page, true).await;
                        // The index map only exists after a snapshot; build it on
                        // first use so `mouse 3` works even if `snapshot` was never run.
                        if Arc::clone(&page).move_to_index(index).await.is_err() {
                            let _ = page.snapshot_json().await?;
                            Arc::clone(&page).move_to_index(index).await?;
                        }
                        println!("moved to [{index}]");
                    } else if looks_like_a_selector(&rest) {
                        let element = Arc::clone(&page).wait_for_selector(rest.clone()).await?;
                        crate::cursor::set_driving(&page, true).await;
                        element.hover_mouse().await?;
                        println!("moved to {rest}");
                    } else {
                        match control_by_text(&page, &rest).await? {
                            Some(element) => {
                                crate::cursor::set_driving(&page, true).await;
                                element.hover_mouse().await?;
                                println!("moved to {rest:?}");
                            }
                            None => report_no_text_match(&rest),
                        }
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
                    opened.set_human(!args.linear);
                    if args.cursor_active() {
                        let _ = crate::cursor::install(&opened).await;
                        crate::cursor::set_gate(&opened, true).await;
                    }
                    tabs.push(Arc::clone(&opened));
                    active_tab = tabs.len() - 1;
                    page = opened;
                    // Bring the new tab (and its window) to the front.
                    focus_page(&page).await;
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
                                attached.set_human(!args.linear);
                                if args.cursor_active() {
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
                    // Bring the tab (and its window) to the front.
                    focus_page(&page).await;
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
                    let Some(items) = infos else {
                        println!("{raw}");
                        return Ok(());
                    };

                    // The tabs this session opened, in the order `switch` /
                    // `close-tab` index them. The active one is marked `*`.
                    let session_ids: Vec<String> =
                        tabs.iter().map(|tab| tab.target_id().to_string()).collect();
                    for (index, tab) in tabs.iter().enumerate() {
                        let mark = if index == active_tab { "*" } else { " " };
                        let url = tab.url().await.unwrap_or_default();
                        println!("{mark} [{index}]  {}  {url}", tab.target_id());
                    }
                    // Everything else the browser reports (extensions, workers,
                    // popups this session did not open): no index, id only.
                    for item in items {
                        let id = item
                            .get("targetId")
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or("");
                        if session_ids.iter().any(|known| known == id) {
                            continue;
                        }
                        let kind = item
                            .get("type")
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or("");
                        let url = item
                            .get("url")
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or("");
                        println!("  -   {id}  {kind}  {url}");
                    }
                    Ok(())
                }
                "close-tab" | "closetab" => {
                    if rest.is_empty() {
                        println!("usage: close-tab <index|targetId>   (indices/ids come from `tabs`)");
                        return Ok(());
                    }
                    // Accept the same session index `switch` uses, or a raw id.
                    let target_id = if let Ok(index) = rest.parse::<usize>() {
                        match tabs.get(index) {
                            Some(tab) => tab.target_id().to_string(),
                            None => {
                                println!("no tab {index} (have {} open)", tabs.len());
                                return Ok(());
                            }
                        }
                    } else {
                        rest.clone()
                    };
                    // Never close the session's last tab: that would leave the
                    // session with nothing to drive.
                    if tabs.len() <= 1
                        && tabs
                            .iter()
                            .any(|tab| tab.target_id() == target_id.as_str())
                    {
                        println!("cannot close the last tab");
                        return Ok(());
                    }
                    let params = serde_json::json!({ "targetId": target_id }).to_string();
                    match page
                        .execute_cdp_cmd("Target.closeTarget".to_string(), params)
                        .await
                    {
                        Ok(_) => {
                            println!("closed tab {target_id}");
                            // Drop it from the list, keeping the active tab valid.
                            if let Some(position) = tabs
                                .iter()
                                .position(|tab| tab.target_id() == target_id.as_str())
                            {
                                tabs.remove(position);
                                if active_tab >= tabs.len() {
                                    active_tab = tabs.len().saturating_sub(1);
                                }
                                if !tabs.is_empty() {
                                    page = tabs[active_tab].clone();
                                    focus_page(&page).await;
                                }
                            }
                        }
                        Err(error) => println!("could not close {target_id}: {error}"),
                    }
                    Ok(())
                }
                "record" | "rec" => {
                    // Start a video capture of the live run. Tied to codegen: a
                    // bare `--codegen` run records *actions*; `record` adds a
                    // screen capture of the same run.
                    if args.codegen.is_none() {
                        println!("record is only available with --codegen <LANG>");
                        return Ok(());
                    }
                    if video_path.is_some() {
                        println!("already recording; `stop-record` first");
                        return Ok(());
                    }
                    let output = if rest.is_empty() {
                        "recording.mp4".to_string()
                    } else {
                        rest.to_string()
                    };
                    page.start_video(output.clone()).await?;
                    video_path = Some(output);
                    println!("recording started (will write on `stop-record`)");
                    Ok(())
                }
                "stop-record" | "stop-rec" | "record-stop" => {
                    if args.codegen.is_none() {
                        println!("stop-record is only available with --codegen <LANG>");
                        return Ok(());
                    }
                    match page.stop_video().await? {
                        Some(path) => {
                            println!("wrote {path}");
                            video_path = None;
                        }
                        None => println!("no recording was in progress"),
                    }
                    Ok(())
                }
                "codegen" | "gen" => {
                    if args.codegen.is_none() {
                        println!("codegen is only available with --codegen <LANG>");
                        return Ok(());
                    }
                    let (sub, arg) = match rest.split_once(char::is_whitespace) {
                        Some((sub, arg)) => (sub.to_ascii_lowercase(), arg.trim().to_string()),
                        None => (rest.to_ascii_lowercase(), String::new()),
                    };
                    let lang = args.codegen.expect("checked above");
                    match sub.as_str() {
                        // `codegen preview` / `codegen`: print the script so far.
                        "" | "preview" | "show" | "print" => {
                            let code = codegen_language(lang).generate(&recording);
                            println!(
                                "\n# ---- generated {} ({} actions, preview) ----\n{code}",
                                codegen_language(lang).label(),
                                recording.len()
                            );
                        }
                        // `codegen out [path]`: write a snapshot without ending
                        // the session (useful when the AI wants to hand off a draft).
                        "out" | "write" | "save" => {
                            let code = codegen_language(lang).generate(&recording);
                            let path = if arg.is_empty() {
                                codegen_language(lang).file_name().to_string()
                            } else {
                                arg
                            };
                            match std::fs::write(&path, code.as_bytes()) {
                                Ok(()) => println!("codegen: wrote {path} ({} actions)", recording.len()),
                                Err(error) => eprintln!("codegen: could not write {path}: {error}"),
                            }
                        }
                        // `codegen undo` / `codegen remove <n>`: prune mistakes
                        // from the recorded run before it is rendered.
                        "undo" => {
                            if recording.pop().is_some() {
                                println!("removed the last recorded action ({} left)", recording.len());
                            } else {
                                println!("nothing recorded yet");
                            }
                        }
                        "remove" | "drop" => {
                            let n = arg.parse::<usize>().unwrap_or(1);
                            if recording.len() <= n {
                                recording.clear();
                            } else {
                                recording.truncate(recording.len() - n);
                            }
                            println!("removed {n} action(s) ({} left)", recording.len());
                        }
                        // `codegen clear`: start the recording over.
                        "clear" | "reset" => {
                            recording.clear();
                            last_target = None;
                            println!("recording cleared");
                        }
                        "status" | "stats" | "count" => {
                            println!("{} action(s) recorded", recording.len());
                        }
                        _ => {
                            println!(
                                "usage: codegen [preview|out|undo|remove <n>|clear|status]"
                            );
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
                            fresh.set_human(!args.linear);
                            if args.cursor_active() {
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
            && let Some(action) = record_action(&tokens, &mut last_target)
        {
            recording.push(action);
        }

        // Re-raise the cursor gate: outside a mouse step the cursor stays inert.
        crate::cursor::set_driving(&page, false).await;
    }

    // Leave room for `Browser::close`'s own graceful wait + force-kill fallback
    // so the browser is never orphaned when the session ends.
    let _ = tokio::time::timeout(Duration::from_secs(20), browser.close()).await;
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
         \x20 click <index|selector|text>   click by snapshot index, CSS selector, or visible text\n\
         \x20 tap <selector|text>           same pick, but a DOM click that never moves the mouse\n\
         \x20 click-xy <x> <y>              raw coordinate click (canvas / embedded)\n\
         \x20 upload <selector> <path>      set a file input to a local file\n\
         \x20 fill <selector|index> <text>  focus + type slowly (50 ms/char)\n\
         \x20 select <selector> <value>     choose an option in a native <select>\n\
         \x20 type <text>                    type into the focused element\n\
         \x20 press <key>                    press a key on the focused element\n\
         \x20 submit                         press Enter on the focused element (send)\n\
         \x20 hover <selector>               move the mouse over an element\n\
         \x20 mouse <index|selector|text>   move the cursor there (or `mouse <x> <y>`); no click\n\
         \x20 scroll <px|up|down|top|bottom> scroll the page\n\
         \x20 find <text>                    how many elements contain the text\n\
         \x20 wait <ms|selector>             sleep, or wait for an element\n\
         \x20 wait-stable [ms]               wait until the DOM stops changing\n\
         \x20 wait-idle [ms]                 wait until the network goes quiet\n\
         \x20 wait-random <min> <max>        sleep a uniform random hold (ms)\n\
         \x20 challenge                       detect an anti-bot / CAPTCHA challenge\n\
         \x20 await-human [s]                 wait until a person clears the challenge\n\
         \x20 eval <js>                      evaluate JavaScript, print the result\n\
         \x20 guard <path.js>               block popups/ads on this page and every new one\n\
         \x20 tabs                          list targets (id, type, url)\n\
         \x20 new-tab [url]                 open a new tab and make it active\n\
         \x20 switch <n|targetId>            make a tab active (index from `tabs`; no arg cycles)\n\
         \x20 close-tab <index|targetId>     close a tab (index or id from `tabs`)\n\
         \x20 shot [path]                    viewport screenshot (default screenshot.png)\n\
         \x20 shot-full [path]               full-page screenshot\n\
         \x20 record [path]                   start a video (only with --codegen)\n\
         \x20 stop-record                     stop the video (only with --codegen)\n\
         \x20 codegen [preview|out|undo|remove <n>|clear|status]\n\
         \x20                                 preview / edit the recorded script\n\
         \x20 done [summary]                  mark the task complete\n\
         \x20 help | quit                    (an AI run must `done` or `quit!` to exit)"
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
        // A derived CSS selector, when one exists, so the element can be driven by
        // `fill "#email" …` / `click "#submit"` instead of only by index.
        let selector = entry
            .get("selector")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        let suffix = if selector.is_empty() {
            String::new()
        } else {
            format!("  {}", quote_selector(selector))
        };
        if value.is_empty() {
            println!("[{index}] <{role}> {name:?}{suffix}");
        } else {
            println!("[{index}] <{role}> {name:?} = {value:?}{suffix}");
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
fn record_action(tokens: &[String], last: &mut Option<Selector>) -> Option<Action> {
    let verb = tokens
        .first()
        .map(|v| v.to_ascii_lowercase())
        .unwrap_or_default();
    let rest = tokens.get(1..).map_or(String::new(), |rest| rest.join(" "));
    let rest = rest.trim();
    match verb.as_str() {
        "open" | "goto" if !rest.is_empty() => Some(Action::Navigate {
            url: rest.to_string(),
        }),
        // `click` / `tap` take an index, a selector, or visible text. An index
        // carries no portable locator, so it is left out of the recording.
        "click" | "tap" if !rest.is_empty() && rest.parse::<u32>().is_err() => {
            let selector = if looks_like_a_selector(rest) {
                Selector::parse(rest)
            } else {
                Selector::text(rest)
            };
            *last = Some(selector.clone());
            Some(Action::Click { selector })
        }
        "hover" if !rest.is_empty() => {
            let selector = Selector::parse(rest);
            *last = Some(selector.clone());
            Some(Action::Hover { selector })
        }
        // `mouse` moves the pointer; recorded as a hover. Coordinates and
        // snapshot indices carry no portable locator, so they are left out.
        "mouse"
            if !rest.is_empty()
                && parse_coordinates(rest).is_none()
                && rest.parse::<u32>().is_err() =>
        {
            let selector = if looks_like_a_selector(rest) {
                Selector::parse(rest)
            } else {
                Selector::text(rest)
            };
            *last = Some(selector.clone());
            Some(Action::Hover { selector })
        }
        "wait" | "sleep" if !rest.is_empty() && rest.parse::<u64>().is_err() => {
            let selector = Selector::parse(rest);
            *last = Some(selector.clone());
            Some(Action::WaitFor { selector })
        }
        "fill" => {
            // Tokens, not `rest`: a quoted selector may contain spaces.
            let selector = tokens.get(1)?;
            let text = tokens.get(2..).filter(|rest| !rest.is_empty())?.join(" ");
            let selector = Selector::parse(selector);
            *last = Some(selector.clone());
            Some(Action::Fill {
                selector,
                text: text.trim().to_string(),
            })
        }
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

/// Splits a session statement into the `(selector, text)` pair used by `fill`
/// and `upload`: the token right after the verb is the selector and the rest are
/// rejoined with single spaces, so a quoted selector that itself contains spaces
/// (`fill '[aria-label="Email address"]' hi`) stays intact. `None` when either
/// half is missing.
fn split_selector_text(tokens: &[String]) -> Option<(String, String)> {
    let selector = tokens.get(1)?;
    let rest = tokens.get(2..)?;
    if rest.is_empty() {
        return None;
    }
    Some((selector.clone(), rest.join(" ")))
}

/// Renders a selector as a quoted string literal so it can be copied verbatim into
/// a command (`fill '[name="email"]' …`). Single quotes unless the selector
/// itself contains one, in which case double quotes with `\"` escapes are used.
fn quote_selector(selector: &str) -> String {
    if selector.contains('\'') {
        format!("\"{}\"", selector.replace('"', "\\\""))
    } else {
        format!("'{selector}'")
    }
}

/// Reports that a text-based click matched nothing.
fn report_no_text_match(text: &str) {
    println!("no visible element contains {text:?}");
}

/// Parses a bare `"x y"` pair into cursor coordinates, when both parts are
/// numbers and there is nothing else.
fn parse_coordinates(value: &str) -> Option<(f64, f64)> {
    let mut parts = value.split_whitespace();
    let x = parts.next()?.parse::<f64>().ok()?;
    let y = parts.next()?.parse::<f64>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((x, y))
}

/// Brings a tab and its window to the front. `Target.activateTarget` (browser
/// domain) raises the window; `Page.bringToFront` is a page-scoped fallback for
/// browsers or modes that reject it.
async fn focus_page(page: &std::sync::Arc<xcelerate::Page>) {
    if page.activate().await.is_err() {
        let _ = page.bring_to_front().await;
    }
}

#[cfg(test)]
mod tests {
    use super::split_selector_text;
    use crate::interact::looks_like_a_selector;
    use crate::xcl::lex::{Line, lex_line};

    fn tokens(line: &str) -> Vec<String> {
        match lex_line(line).expect("lexes") {
            Line::Statement(tokens) => tokens,
            other => panic!("expected a statement, got {other:?}"),
        }
    }

    #[test]
    fn splits_selector_and_text() {
        assert_eq!(
            split_selector_text(&tokens("fill #email hello")),
            Some(("#email".into(), "hello".into()))
        );
        // Text keeps its internal spaces.
        assert_eq!(
            split_selector_text(&tokens("fill #name Ada Lovelace")),
            Some(("#name".into(), "Ada Lovelace".into()))
        );
    }

    #[test]
    fn keeps_spaces_inside_a_quoted_selector() {
        assert_eq!(
            split_selector_text(&tokens("fill '[aria-label=\"Email address\"]' hi")),
            Some(("[aria-label=\"Email address\"]".into(), "hi".into()))
        );
    }

    #[test]
    fn requires_both_selector_and_text() {
        assert_eq!(split_selector_text(&tokens("fill")), None);
        assert_eq!(split_selector_text(&tokens("fill #email")), None);
    }

    #[test]
    fn parses_a_bare_coordinate_pair() {
        use super::parse_coordinates;
        assert_eq!(parse_coordinates("120 90"), Some((120.0, 90.0)));
        assert_eq!(parse_coordinates("-4.5 3"), Some((-4.5, 3.0)));
        // A single number (a snapshot index) is not a coordinate pair.
        assert_eq!(parse_coordinates("3"), None);
        assert_eq!(parse_coordinates("#email"), None);
        assert_eq!(parse_coordinates("1 2 3"), None);
    }

    #[test]
    fn spots_selectors_passed_to_text_clicks() {
        assert!(looks_like_a_selector("[name=\"pass\"]"));
        assert!(looks_like_a_selector("#email"));
        assert!(looks_like_a_selector(".btn"));
        assert!(looks_like_a_selector("//a[@id='x']"));
        assert!(!looks_like_a_selector("Log in"));
        assert!(!looks_like_a_selector("Sign up"));
    }

    #[test]
    fn click_records_index_selector_and_text() {
        use super::record_action;
        use xcelerate_codegen::{Action, Selector};

        // A bare index carries no portable locator, so it is left out.
        let mut last = None;
        assert_eq!(record_action(&tokens("click 3"), &mut last), None);

        // A selector-looking argument records a CSS selector.
        let mut last = None;
        assert_eq!(
            record_action(&tokens("click '#email'"), &mut last),
            Some(Action::Click {
                selector: Selector::parse("#email")
            })
        );

        // Anything else records a visible-text match, for `click` and `tap` alike.
        let mut last = None;
        assert_eq!(
            record_action(&tokens("click 'Sign in'"), &mut last),
            Some(Action::Click {
                selector: Selector::text("Sign in")
            })
        );
        let mut last = None;
        assert_eq!(
            record_action(&tokens("tap 'Accept all'"), &mut last),
            Some(Action::Click {
                selector: Selector::text("Accept all")
            })
        );
    }
}
