//! Command-line interface for the xcelerate CDP engine.
//!
//! Most commands launch a fresh browser, perform one action, and exit. Two
//! exceptions keep state: `xcelerate session` reads commands from stdin against
//! one persistent browser (`session`), and `xcelerate mcp` serves the Model
//! Context Protocol on stdio.
//!
//! The modules: `cli` holds the clap surface, `commands` the one-shot
//! subcommands and the dispatcher, `launch` the shared browser setup, `session`
//! the REPL, `overlay` the native overlay window, and `scaffold` the plugin
//! template.
//!
//! When a run asks for the overlay (`--hud`, `--ai`, or `live`), the overlay owns
//! this thread's winit event loop and the session runs on a background runtime;
//! winit insists the event loop live on the main thread.

use mimalloc::MiMalloc;

mod cli;
mod commands;
mod cursor;
mod launch;
mod overlay;
mod scaffold;
mod session;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {
    use clap::Parser;

    let cli = cli::Cli::parse();
    let result = if overlay::wanted(&cli) {
        overlay::run_with_overlay(cli)
    } else {
        run_session_only(cli)
    };
    if let Err(error) = result {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

/// The ordinary path: no overlay, so the whole command runs on the main thread.
fn run_session_only(cli: cli::Cli) -> Result<(), Box<dyn std::error::Error>> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    runtime.block_on(commands::run(cli, overlay::OverlayHandle::new()))
}
