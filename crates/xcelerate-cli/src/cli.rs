//! Command-line surface: the clap structs/enums shared by the rest of the CLI.
//!
//! The CLI is deliberately **bare-bones**: it runs XCL scripts and hosts plugins.
//! Browser and app actions are plugin **ops**, not subcommands - there is no
//! `open`, `click`, or `launch` here. Load a plugin (or import it in a script)
//! and drive it from XCL; the browser/app surfaces live entirely in the plugins.
//!
//! Kept free of behaviour so `main.rs` stays a thin entry point.

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "xcelerate",
    version,
    about = "Run XCL automation scripts and host plugins.",
    propagate_version = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Execute an XCL script (`.xcl`). Shares the session command language.
    Run {
        /// Path to the `.xcl` file.
        path: PathBuf,
        /// Allow `eval <js>` (otherwise denied by default).
        #[arg(long)]
        allow_unsafe: bool,
        /// Allow `request` (browserless HTTP) specifically.
        #[arg(long)]
        allow_http: bool,
        /// Allow `import`/`run` of these plugins (repeatable, supports `*`).
        #[arg(long = "allow-plugin", value_name = "ID")]
        allow_plugin: Vec<String>,
        /// Allow private/loopback/metadata hosts in browserless requests.
        #[arg(long)]
        allow_private: bool,
        /// Native windows (by title/process glob) an XCL script may drive.
        #[arg(long = "allow-app", value_name = "PATTERN")]
        allow_app: Vec<String>,
        /// Extra plugins to load, by path or name (comma-separated).
        #[arg(long, value_name = "PATH|NAME", value_delimiter = ',')]
        plugins: Vec<String>,
        /// Directory that every file verb (`download`, `capture`, `shot`, `upload`)
        /// is confined to. Defaults to the current directory.
        #[arg(long = "output-dir", value_name = "DIR")]
        output_dir: Option<PathBuf>,
        /// Override a script `param` (`key=value`).
        #[arg(long = "param", value_name = "KEY=VALUE")]
        param: Vec<String>,
        /// Print one `ok <step>` line per step. Quiet by default; use the `print`
        /// op to log from the script.
        #[arg(long, short)]
        verbose: bool,
        /// Raise the total-step cap (default 10000).
        #[arg(long, value_name = "N")]
        max_steps: Option<u32>,
        /// Raise the `repeat`/`retry` iteration cap (default 10000).
        #[arg(long, value_name = "N")]
        max_iterations: Option<u32>,
        /// Raise the maximum `func` parameters (default 64).
        #[arg(long, value_name = "N")]
        max_func_params: Option<u32>,
        /// Raise the maximum number of `func`s / imports (default 4096).
        #[arg(long, value_name = "N")]
        max_funcs: Option<usize>,
    },
    /// Build a scaffolded plugin: compile the `.wasm` core and/or generate +
    /// package the typed client binding for a target language.
    Build {
        /// Target language (see `--lang`). Optional when a default is set or
        /// exactly one toolchain is detected.
        lang: Option<CodegenLang>,
        /// Compile only the `.wasm` core; skip binding generation/package.
        #[arg(long)]
        wasm_only: bool,
        /// Generate + package bindings only against an existing `.wasm`.
        #[arg(long)]
        bindings_only: bool,
        /// Inspect the toolchain matrix and report, building nothing.
        #[arg(long)]
        check: bool,
        /// Output directory (default `dist`).
        #[arg(long, value_name = "DIR")]
        out: Option<PathBuf>,
        /// Forbid network fetches (registry/pub/dependency downloads).
        #[arg(long)]
        offline: bool,
    },
    /// Create a new mod (plugin) from the starter template.
    Plugin {
        #[command(subcommand)]
        action: PluginAction,
    },
    /// List the plugins this machine exposes and how to load them.
    Plugins,
    /// Run the Model Context Protocol (MCP) server on stdio.
    Mcp,
}

#[derive(Subcommand)]
pub enum PluginAction {
    /// Scaffold a new mod from the starter template.
    New {
        /// The mod id, e.g. `acme.hello` (last segment names the directory).
        name: String,
        /// Directory to create (default: the last id segment).
        #[arg(long, value_name = "PATH")]
        dir: Option<PathBuf>,
        /// Write into the directory even if it already exists.
        #[arg(long)]
        force: bool,
    },
}

/// Target language for `xcelerate build`.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum CodegenLang {
    Rust,
    Python,
    #[value(alias = "js")]
    Javascript,
    #[value(alias = "c#", alias = "cs")]
    Csharp,
    Kotlin,
    Java,
    Swift,
    Ruby,
    Dart,
    Go,
    #[value(alias = "pwsh", alias = "ps")]
    Powershell,
}
