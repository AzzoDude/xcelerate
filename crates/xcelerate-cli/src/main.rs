//! Command-line interface for the xcelerate CDP engine.
//!
//! Most commands launch a fresh browser, perform one action, and exit. Two
//! exceptions keep state: `xcelerate session` reads commands from stdin against
//! one persistent browser (`session`), and `xcelerate mcp` serves the Model
//! Context Protocol on stdio.
//!
//! The modules: `cli` holds the clap surface, `commands` the one-shot
//! subcommands and the dispatcher, `launch` the shared browser setup, `session`
//! the REPL, and `scaffold` the plugin template.

use mimalloc::MiMalloc;

mod cli;
mod commands;
mod cursor;
mod launch;
#[cfg(feature = "http")]
mod net;
mod scaffold;
mod session;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {
    use clap::Parser;

    let cli = cli::Cli::parse();
    if let Err(error) = run(cli) {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

/// Runs the whole command on a worker thread with a roomy stack.
///
/// The command dispatcher is a large async future, and a live session drives the
/// browser from it; the platform's default main-thread stack can overflow while
/// polling it, so the runtime runs on a thread with an explicit, generous stack.
fn run(cli: cli::Cli) -> Result<(), Box<dyn std::error::Error>> {
    let worker = std::thread::Builder::new()
        .name("xcelerate".to_string())
        .stack_size(32 * 1024 * 1024)
        .spawn(move || -> Result<(), String> {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .map_err(|error| error.to_string())?;
            runtime
                .block_on(commands::run(cli))
                .map_err(|error| error.to_string())
        })?;

    match worker.join() {
        Ok(result) => result.map_err(|error| -> Box<dyn std::error::Error> { error.into() }),
        Err(_) => Err("worker thread panicked".into()),
    }
}
