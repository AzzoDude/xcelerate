//! The top-level command dispatcher.
//!
//! Bare-bones by design: run a script, build/scaffold a plugin, list plugins, or
//! serve MCP. Everything a browser or app can *do* is a plugin op, not a command.

use crate::cli::{Cli, Command, PluginAction};

pub async fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    // No subcommand: print a short orientation rather than a raw usage error.
    let Some(command) = cli.command else {
        println!("xcelerate - run XCL automation scripts and host plugins.\n");
        println!("  xcelerate run <file.xcl>     execute a script");
        println!("  xcelerate plugins            list plugins");
        println!("  xcelerate build              build a plugin (wasm + bindings)");
        println!("  xcelerate plugin new <id>    scaffold a plugin");
        println!("  xcelerate mcp                serve the Model Context Protocol");
        println!("\nRun `xcelerate --help` for the full command and flag list.");
        return Ok(());
    };

    match command {
        Command::Run {
            path,
            allow_unsafe,
            allow_http,
            allow_plugin,
            allow_private,
            allow_app,
            plugins,
            output_dir,
            param,
            verbose,
            max_steps,
            max_iterations,
            max_func_params,
            max_funcs,
        } => {
            // Start from the safe defaults, then honour any `--max-*` override.
            let mut limits = xcelerate_interpreter::security::Limits::default();
            if let Some(value) = max_steps {
                limits.max_steps = value;
            }
            if let Some(value) = max_iterations {
                limits.max_iterations = value;
            }
            if let Some(value) = max_func_params {
                limits.max_func_params = value;
            }
            if let Some(value) = max_funcs {
                limits.max_funcs = value;
            }
            crate::run::run_file(
                &path,
                plugins,
                allow_unsafe,
                allow_http,
                allow_plugin,
                allow_private,
                allow_app,
                output_dir,
                param,
                verbose,
                limits,
            )
            .await?;
        }
        Command::Build {
            lang,
            wasm_only,
            bindings_only,
            check,
            out,
            offline,
        } => {
            crate::build::run(lang, wasm_only, bindings_only, check, out, offline)?;
        }
        Command::Plugin { action } => match action {
            PluginAction::New { name, dir, force } => {
                let path = crate::scaffold::new_mod(&name, dir, force)?;
                println!("created mod '{name}' in {}", path.display());
                println!(
                    "next: cd {} && xcelerate build --wasm-only   # builds the .wasm",
                    path.display()
                );
            }
        },
        Command::Plugins => crate::plugins::list_plugins(),
        Command::Mcp => {
            xcelerate_mcp::run_stdio().await?;
        }
    }
    Ok(())
}
