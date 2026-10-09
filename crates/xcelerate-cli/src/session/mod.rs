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
//!
//! The loop itself lives here; the verbs it dispatches to are grouped by concern
//! across the sibling modules (`nav`, `interact`, `media`, `tabs`, `net`,
//! `storage`, `codegen`, `cookies`, `misc`), with the small helpers in `input`,
//! `help` and `record`.

mod codegen;
mod cookies;
mod dispatch;
mod help;
mod input;
mod interact;
mod media;
mod misc;
mod nav;
mod net;
mod record;
mod state;
mod storage;
mod tabs;

use std::sync::Arc;
use std::time::Duration;

use crate::cli::BrowserArgs;
use crate::launch::launch;

use self::dispatch::Control;
use self::state::Session;

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
    let mut session = Session {
        args: args.clone(),
        browser,
        page: first.clone(),
        tabs: vec![first],
        active_tab: 0,
        recording: Vec::new(),
        last_target: None,
        video_path: None,
        job_done: false,
        interactive,
        root: args
            .output_dir
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| ".".into())),
        step_no_op: false,
    };

    // Raise the input gate: while the run drives the page the human cannot click,
    // type or scroll it. Each step lowers the gate only for its own CDP input.
    if session.args.cursor_active() {
        crate::cursor::set_gate(&session.page, true).await;
    }

    if interactive {
        println!("xcelerate session - one browser, many steps.");
        if session.args.live() {
            println!("live mode: the screen is locked to the AI.");
        }
        println!("type `help` for commands, `quit` to exit.\n");
    } else {
        println!(
            "# session start: {}",
            session.page.url().await.unwrap_or(initial)
        );
    }

    if let Some(lang) = session.args.codegen {
        println!(
            "codegen: recording this run as {}.",
            record::codegen_language(lang).label()
        );
    }

    let mut lines = tokio::io::BufReader::new(tokio::io::stdin()).lines();
    'session: loop {
        if session.interactive {
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
        if !session.interactive {
            println!(">>> {line}");
        }

        // Parse the line with the XCL lexer, exactly as a `.xcl` script is parsed,
        // so quotes group arguments and a leading `#` is a comment. The verb is the
        // first token; the remaining tokens are rejoined for the arms that take
        // free text.
        let tokens = match xcelerate_interpreter::lex::lex_line(line) {
            Ok(xcelerate_interpreter::lex::Line::Statement(tokens)) if !tokens.is_empty() => tokens,
            Ok(_) => continue,
            Err(error) => {
                println!("input error: {error}");
                continue;
            }
        };
        let verb = tokens[0].to_ascii_lowercase();
        let rest = tokens[1..].join(" ");
        session.step_no_op = false;

        // Self-heal: a step can fail because the CDP session was detached by a
        // target churn. Re-attach the page and run the same step exactly once
        // more, instead of leaving the session dead for the rest of the run.
        let mut quit = false;
        let mut healed = false;
        let outcome: Result<(), Box<dyn std::error::Error>> = loop {
            match session.dispatch(&verb, &tokens, &rest).await {
                Ok(Control::Quit) => {
                    quit = true;
                    break Ok(());
                }
                Ok(Control::Continue) => break Ok(()),
                Err(error) => {
                    if !healed && input::is_session_error(error.as_ref()) {
                        healed = true;
                        match Arc::clone(&session.browser)
                            .attach_page(session.page.target_id())
                            .await
                        {
                            Ok(fresh) => {
                                fresh.set_human(!session.args.linear);
                                if session.args.cursor_active() {
                                    let _ = crate::cursor::install(&fresh).await;
                                    crate::cursor::set_gate(&fresh, true).await;
                                }
                                println!(
                                    "session detached; re-attached the page and retrying the step"
                                );
                                // Replace the active slot too, or a later
                                // `switch` would restore the dead page.
                                let active = session.active_tab;
                                session.page = Arc::clone(&fresh);
                                match session.tabs.get_mut(active) {
                                    Some(slot) => *slot = fresh,
                                    None => {
                                        session.tabs.push(fresh);
                                        session.active_tab = session.tabs.len() - 1;
                                    }
                                }
                                continue;
                            }
                            Err(_) => break Err(error),
                        }
                    }
                    break Err(error);
                }
            }
        };

        let ok = outcome.is_ok();
        if quit {
            break;
        }
        if let Err(error) = outcome {
            // A failed step must not tear down the session: report and continue.
            println!("error: {error}");
        }

        // Record the step for `--codegen`, when one was requested. A step that
        // only reported "nothing matched" (or a timed-out wait) is skipped, so the
        // generated script never contains an action that did not happen.
        if ok
            && !session.step_no_op
            && session.args.codegen.is_some()
            && let Some(action) = record::record_action(&tokens, &mut session.last_target)
        {
            session.recording.push(action);
        }

        // Re-raise the cursor gate: outside a mouse step the cursor stays inert.
        crate::cursor::set_driving(&session.page, false).await;
    }

    // Leave room for `Browser::close`'s own graceful wait + force-kill fallback
    // so the browser is never orphaned when the session ends.
    let _ = tokio::time::timeout(Duration::from_secs(20), session.browser.close()).await;
    println!("session closed.");

    if let Some(lang) = session.args.codegen {
        record::emit_codegen(
            lang,
            &session.recording,
            session.args.codegen_out.as_deref(),
        );
    }
    Ok(())
}
