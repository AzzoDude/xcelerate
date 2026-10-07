//! Command-line interface for the xcelerate CDP engine.
//!
//! Most commands launch a fresh browser, perform one action, and exit. Two
//! exceptions keep state: `xcelerate session` reads commands from stdin against
//! one persistent browser (`session`), and `xcelerate mcp` serves the Model
//! Context Protocol on stdio.
//!
//! The modules: `cli` holds the clap surface, `commands` the one-shot
//! subcommands and the dispatcher, `launch` the shared browser setup, `session`
//! the REPL, `overlay` the in-page HUD, and `scaffold` the plugin template.

use mimalloc::MiMalloc;

mod cli;
mod commands;
mod launch;
mod overlay;
mod scaffold;
mod session;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() {
    use clap::Parser;

    let cli = cli::Cli::parse();
    if let Err(error) = commands::run(cli).await {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
