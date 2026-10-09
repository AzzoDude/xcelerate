//! The `.xcl` file runner entry point: load, parse, and execute a script.
//!
//! The runner is browser- and app-agnostic. It constructs a generic
//! [`Executor`] (which boots the browser lazily from the environment and keeps
//! the desktop driver available), loads the run's plugins, and drives the script.
//! Every browser/app action is a plugin op.

use std::path::{Path, PathBuf};

use xcelerate_interpreter::Engine;
use xcelerate_interpreter::ast::Command;
use xcelerate_interpreter::engine::Next;
use xcelerate_interpreter::exec::{Executor, dispatch};
use xcelerate_interpreter::parse::parse_program_file;
use xcelerate_interpreter::runtime::{Context, RuntimeLimits};
use xcelerate_interpreter::security::{Limits, Permissions, Source};

/// Executes an `.xcl` file, booting the browser (or desktop) driver only as the
/// script's plugins require.
pub async fn run_file(
    path: &Path,
    plugins: Vec<String>,
    allow_unsafe: bool,
    allow_http: bool,
    allow_plugin: Vec<String>,
    allow_private: bool,
    allow_app: Vec<String>,
    output_dir: Option<PathBuf>,
    params: Vec<String>,
    verbose: bool,
    limits: Limits,
) -> Result<(), Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;

    // Parse before launching anything (fail fast on syntax errors). Relative
    // `import "*.xcl"` paths resolve against the script's own directory.
    let program = parse_program_file(&source, limits, path.parent().map(|dir| dir.to_path_buf()))?;

    let permissions = Permissions {
        allow_eval: allow_unsafe,
        allow_http: allow_http || allow_unsafe,
        allow_plugins: allow_plugin,
        allow_private,
        // Native-window control is default-deny: a script may only drive windows
        // the invoking human explicitly allowlisted with `--allow-app`.
        allow_apps: allow_app,
    };

    // A generic executor: the browser is launched lazily on first use (configured
    // by the environment - see `xcelerate::session`), and the desktop driver is
    // available through the `window`/`launch` verbs. The runner picks no driver.
    let exe = Executor::from_env();

    // Load the run's plugins into the executor's **own** host, so `run` and the
    // native verbs work even with no browser: the plugin host is not the browser.
    for path in crate::plugins::resolve_plugin_names(&plugins) {
        exe.load_plugin(&path)
            .map_err(|error| -> Box<dyn std::error::Error> {
                format!("cannot load plugin `{path}`: {error}").into()
            })?;
    }
    // Auto-load the standard `core` / `browser` / `app` plugins when they are
    // installed (a plugin home, or ./plugins), so a script uses the plain verbs
    // without naming them. Capabilities stay default-deny, so this only makes the
    // ops *available*; the grant is still required. A missing/duplicate load is fine.
    for name in crate::plugins::STANDARD_PLUGINS {
        let path = crate::plugins::resolve_plugin_name(name);
        if std::path::Path::new(&path).exists() {
            let _ = exe.load_plugin(&path);
        }
    }

    // Build context + apply `--param key=value` overrides.
    let base_url = std::env::var("XCELERATE_BASE_URL").unwrap_or_default();
    let mut ctx = Context::new(permissions, Source::File, base_url);
    if let Some(dir) = &output_dir {
        ctx = ctx.with_root(dir);
    }
    for entry in params {
        if let Some((k, v)) = entry.split_once('=') {
            ctx.vars.insert(k.to_string(), v.to_string());
        }
    }

    let mut engine = Engine::new(
        program,
        RuntimeLimits {
            max_steps: limits.max_steps,
            max_output_chars: limits.max_output_chars,
        },
    );

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
        // A browser/app verb or a plugin op leaves its result in `$RESULT`; control
        // steps (`import`, `let`, …) do not clobber it.
        let captures_result = matches!(cmd, Command::Raw { .. } | Command::Run { .. });

        let outcome = dispatch(&mut ctx, cmd, &exe).await;
        engine.observe(outcome.ok);
        // Carry this step's result forward - the same convention `request` uses
        // for `$STATUS` / `$RESPONSE_BODY`.
        if outcome.ok && captures_result {
            let result = outcome
                .value
                .clone()
                .unwrap_or_else(|| outcome.message.clone());
            ctx.vars.insert("RESULT".to_string(), result);
        }
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
        if ctx.steps_executed > limits.max_steps * 2 {
            println!("fail runaway guard tripped");
            break;
        }
    }

    // Close the browser this run launched, if any, before returning.
    exe.shutdown().await;
    Ok(())
}
