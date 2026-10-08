//! The `.xcl` file runner entry point: load, parse, and execute a script against
//! a live browser.

use std::path::Path;
use std::sync::Arc;

use xcelerate_interpreter::Engine;
use xcelerate_interpreter::ast::Command;
use xcelerate_interpreter::engine::Next;
use xcelerate_interpreter::exec::{Executor, dispatch};
use xcelerate_interpreter::parse::parse_program;
use xcelerate_interpreter::runtime::{Context, RuntimeLimits};
use xcelerate_interpreter::security::{MAX_STEPS, Permissions, Source};

use crate::cli::BrowserArgs;
use crate::launch::launch;

/// Executes an `.xcl` file, launching a browser as needed.
pub async fn run_file(
    args: &BrowserArgs,
    path: &Path,
    allow_unsafe: bool,
    allow_http: bool,
    allow_plugin: Vec<String>,
    allow_private: bool,
    params: Vec<String>,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;

    // Parse before launching anything (fail fast on syntax errors).
    let program = parse_program(&source)?;

    // Resolve permissions.
    let permissions = Permissions {
        allow_eval: allow_unsafe,
        allow_http: allow_http || allow_unsafe,
        allow_plugins: allow_plugin,
        allow_private,
    };

    // Launch the browser (browserless scripts would skip this; for now every
    // script gets a browser, with `request` still going over HTTP).
    let (browser, page) = launch(args, "about:blank").await?;
    let exe = Executor::new(Arc::clone(&browser), page);

    // Build context + apply `--param key=value` overrides.
    let base_url = std::env::var("XCELERATE_BASE_URL").unwrap_or_default();
    let mut ctx = Context::new(permissions, Source::File, base_url);
    if let Some(dir) = &args.output_dir {
        ctx = ctx.with_root(dir);
    }
    for entry in params {
        if let Some((k, v)) = entry.split_once('=') {
            ctx.vars.insert(k.to_string(), v.to_string());
        }
    }

    let mut engine = Engine::new(program, RuntimeLimits::default());

    // Execute: pull the next action, await its dispatch, feed the outcome back.
    loop {
        let cmd = match engine.next_action(&mut ctx) {
            Next::End => break,
            Next::Halt(err) => {
                println!("fail {err}");
                break;
            }
            Next::Action(cmd) => cmd,
        };
        // Read anything we need off the borrowed command before dispatching it
        // (the borrow ends once `dispatch` returns).
        let is_print = matches!(cmd, Command::Print { .. });

        let outcome = dispatch(&mut ctx, cmd, &exe).await;
        engine.observe(outcome.ok);
        // Quiet by default: a run reads as its own log, so only failures and
        // explicit `print` output are shown. `--verbose` restores the per-step
        // `ok <step>` transcript.
        if !outcome.ok {
            println!("fail {}", outcome.message);
        } else if is_print {
            println!("{}", outcome.message);
        } else if verbose {
            println!("ok {}", outcome.message);
        }
        if outcome.should_quit {
            break;
        }
        // Guard against a command that never advances (defensive, not expected).
        if ctx.steps_executed > MAX_STEPS * 2 {
            println!("fail runaway guard tripped");
            break;
        }
    }

    // Give `Browser::close` room to finish its own graceful wait and force-kill
    // fallback. Wrapping it in the same 5s window would cancel the kill and
    // orphan the browser window (it would outlive the run).
    let _ = tokio::time::timeout(std::time::Duration::from_secs(20), browser.close()).await;
    Ok(())
}
