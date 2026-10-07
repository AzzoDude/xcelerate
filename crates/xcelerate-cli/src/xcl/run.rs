//! The `.xcl` file runner entry point: load, parse, and execute a script against
//! a live browser.

use std::path::Path;
use std::sync::Arc;

use super::exec::{Executor, dispatch};
use super::parse::parse_program;
use super::runtime::{Context, RuntimeLimits};
use super::security::{Permissions, Source};
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
    let exe = Executor {
        browser: Arc::clone(&browser),
        page,
    };

    // Build context + apply `--param key=value` overrides.
    let base_url = std::env::var("XCELERATE_BASE_URL").unwrap_or_default();
    let mut ctx = Context::new(permissions, Source::File, base_url);
    for entry in params {
        if let Some((k, v)) = entry.split_once('=') {
            ctx.vars.insert(k.to_string(), v.to_string());
        }
    }

    let mut engine = super::Engine::new(program, RuntimeLimits::default());

    // Execute: pull the next action, await its dispatch, feed the outcome back.
    let mut terminated = false;
    while !terminated {
        let Some((cmd, signal)) = engine.next_action(&mut ctx) else {
            break;
        };
        match signal {
            super::engine::RunOutcome::Halt(err) => {
                println!("fail {err}");
                break;
            }
            super::engine::RunOutcome::Continue => {}
        }

        let outcome = dispatch(&mut ctx, &cmd, &exe).await;
        engine.observe(outcome.ok);
        println!(
            "{} {}",
            if outcome.ok { "ok" } else { "fail" },
            outcome.message
        );
        if outcome.should_quit {
            terminated = true;
        }
        // Guard against a command that never advances (defensive, not expected).
        if ctx.steps_executed > super::security::MAX_STEPS * 2 {
            println!("fail runaway guard tripped");
            break;
        }
    }

    let _ = tokio::time::timeout(std::time::Duration::from_secs(5), browser.close()).await;
    Ok(())
}
