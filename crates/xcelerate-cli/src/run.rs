//! The `.xcl` file runner entry point: load, parse, and execute a script against
//! a live browser.

use std::path::Path;
use std::sync::Arc;

use xcelerate_interpreter::Engine;
use xcelerate_interpreter::ast::Command;
use xcelerate_interpreter::engine::Next;
use xcelerate_interpreter::exec::{BrowserFactory, BrowserFuture, Executor, dispatch};
use xcelerate_interpreter::parse::parse_program_file;
use xcelerate_interpreter::runtime::{Context, RuntimeLimits};
use xcelerate_interpreter::security::{Limits, Permissions, Source};

use crate::cli::BrowserArgs;

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
    limits: Limits,
    app: Option<String>,
    native: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;

    // Parse before launching anything (fail fast on syntax errors). Relative
    // `import "*.xcl"` paths resolve against the script's own directory.
    let program = parse_program_file(&source, limits, path.parent().map(|dir| dir.to_path_buf()))?;

    // Resolve permissions.
    let mut allow_apps = args.allow_app.clone();
    // The window the run is explicitly attached to is granted, so acting verbs
    // pass the gate without a redundant `--allow-app` for the same window. A
    // *script* can never add to this: only the human's `--app`/`--allow-app` do.
    if let Some(window) = &app
        && !allow_apps.iter().any(|entry| entry == window)
    {
        allow_apps.push(window.clone());
    }
    let permissions = Permissions {
        allow_eval: allow_unsafe,
        allow_http: allow_http || allow_unsafe,
        allow_plugins: allow_plugin,
        allow_private,
        // Native-app control is default-deny: a script may only drive windows the
        // invoking human explicitly allowlisted with `--allow-app`.
        allow_apps,
    };

    // The drivers a run exposes. `--app`/`--native` are browserless (desktop
    // only); otherwise the browser driver is available but is launched **lazily**,
    // on the first `import browser` (or browser verb), so a native-only script
    // never pays for a browser it does not use.
    #[cfg(windows)]
    let exe = if let Some(window) = app {
        Executor::native(window)
    } else if native {
        Executor::desktop()
    } else {
        Executor::lazy(browser_factory(args))
    };
    #[cfg(not(windows))]
    let exe = {
        if app.is_some() || native {
            return Err("--app/--native (native window control) is Windows only".into());
        }
        Executor::lazy(browser_factory(args))
    };

    // Load the run's plugins into the executor's **own** host, so `run` and the
    // native verbs work even with no browser (a `--native` run): the plugin host
    // is no longer the browser.
    for path in crate::launch::resolve_plugin_names(&args.plugins) {
        exe.load_plugin(&path)
            .map_err(|error| -> Box<dyn std::error::Error> {
                format!("cannot load plugin `{path}`: {error}").into()
            })?;
    }

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
        if ctx.steps_executed > limits.max_steps * 2 {
            println!("fail runaway guard tripped");
            break;
        }
    }

    // Give `Browser::close` room to finish its own graceful wait and force-kill
    // fallback. Wrapping it in the same 5s window would cancel the kill and
    // orphan the browser window (it would outlive the run). A run that never
    // imported the browser (or a native run) has none to close.
    if let Some(browser) = exe.take_browser() {
        let _ = tokio::time::timeout(std::time::Duration::from_secs(20), browser.close()).await;
    }
    Ok(())
}

/// A one-shot launcher for the browser driver: the first `import browser` (or
/// browser verb) calls it. The CLI owns the launch configuration, so the closure
/// captures a clone of the parsed `BrowserArgs`.
fn browser_factory(args: &BrowserArgs) -> BrowserFactory {
    let args = args.clone();
    Arc::new(move || {
        let args = args.clone();
        let launched: BrowserFuture = Box::pin(async move {
            let (browser, page) = crate::launch::launch(&args, "about:blank")
                .await
                .map_err(|e| e.to_string())?;
            Ok((browser, page))
        });
        launched
    })
}
