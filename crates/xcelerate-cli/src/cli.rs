//! Command-line surface: the clap structs/enums shared by the rest of the CLI.
//!
//! Kept free of behaviour so `main.rs` stays a thin entry point.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "xcelerate",
    version,
    about = "Browser automation from the command line, powered by xcelerate.",
    propagate_version = true
)]
pub struct Cli {
    #[command(flatten)]
    pub browser: BrowserArgs,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Args, Clone)]
pub struct BrowserArgs {
    /// Show the browser window (headless by default).
    #[arg(long, global = true)]
    pub no_headless: bool,
    /// AI-driven run: show the window with the cursor.
    #[arg(long, global = true, alias = "live")]
    pub ai: bool,
    /// Watchable run for codegen capture in LANG (`csharp`, `rust`, `python`,
    /// ...): implies a visible window and the in-page cursor.
    #[arg(long, global = true, value_name = "LANG")]
    pub codegen: Option<CodegenLang>,
    /// Where to write the generated script. Omit to print it to stdout at the
    /// end of the run. Pair with `--codegen <LANG>`.
    #[arg(long = "codegen-out", global = true, value_name = "PATH")]
    pub codegen_out: Option<PathBuf>,
    /// Detach the browser process so it outlives this command.
    #[arg(long, global = true)]
    pub detached: bool,
    /// Path to the browser executable.
    #[arg(long, global = true, value_name = "PATH")]
    pub executable_path: Option<String>,
    /// Browser to use: a known id (`chrome`, `chromium`, `edge`, `brave`,
    /// `vivaldi`, `opera`, `firefox`, `firefox-esr`) or a path to the executable.
    /// Overrides `--executable-path`; also settable via `XCELERATE_BROWSER`.
    #[arg(long, global = true, value_name = "ID")]
    pub browser: Option<String>,
    /// External plugin paths to load (comma-separated).
    #[arg(long, global = true, value_name = "PATH", value_delimiter = ',')]
    pub plugins: Vec<String>,
    /// Emulate a mobile device for this run (see `xcelerate list`).
    #[arg(long, global = true, value_name = "NAME")]
    pub device: Option<String>,
    /// Upstream proxy URL(s); repeat for a pool. `http://[user:pass@]host:port`.
    #[arg(long, global = true, value_name = "URL")]
    pub proxy: Vec<String>,
    /// Persistent profile directory; keeps logins/cookies between runs.
    #[arg(long, global = true, value_name = "PATH")]
    pub user_data_dir: Option<String>,
    /// Default wait timeout in milliseconds (0 disables it).
    #[arg(long, global = true, default_value_t = 30000)]
    pub timeout: u64,
    /// Allowed navigation domains (repeatable). When set, only these may load.
    #[arg(long = "allow-domain", global = true, value_name = "DOMAIN")]
    pub allow_domain: Vec<String>,
    /// Prohibited navigation domains (repeatable). Overrides the allow list.
    #[arg(long = "deny-domain", global = true, value_name = "DOMAIN")]
    pub deny_domain: Vec<String>,
    /// Attach to an existing browser's CDP websocket instead of launching one.
    #[arg(long, global = true, value_name = "WS_URL")]
    pub connect: Option<String>,
    /// Extra browser flag passed verbatim (repeatable).
    #[arg(long = "extra-arg", global = true, value_name = "FLAG")]
    pub extra_arg: Vec<String>,
    /// Allow the browser to download files.
    #[arg(long, global = true)]
    pub accept_downloads: bool,
    /// Enable deterministic-rendering flags.
    #[arg(long, global = true)]
    pub deterministic: bool,
    /// Disable web security / site isolation (testing only).
    #[arg(long, global = true)]
    pub disable_security: bool,
    /// Keep the browser process alive after the command exits.
    #[arg(long, global = true)]
    pub keep_alive: bool,
}

impl BrowserArgs {
    /// Whether the run is watchable: a visible window and the in-page cursor.
    /// True for `--ai` and `--codegen`.
    pub fn live(&self) -> bool {
        self.ai || self.codegen.is_some()
    }
}

#[derive(Subcommand)]
pub enum Command {
    /// Open a URL and print its title and URL.
    Open { url: String },
    /// Print the page title.
    Title { url: String },
    /// Print the page's HTML.
    Content { url: String },
    /// Print the page's visible text.
    Text { url: String },
    /// Save a PNG screenshot.
    Screenshot {
        url: String,
        #[arg(short, long, default_value = "screenshot.png")]
        output: PathBuf,
        /// Capture the full page rather than the viewport.
        #[arg(long)]
        full: bool,
    },
    /// Save the page as a PDF.
    Pdf {
        url: String,
        #[arg(short, long, default_value = "page.pdf")]
        output: PathBuf,
    },
    /// Query a selector: print its text (default), an attribute, or its HTML.
    Query {
        url: String,
        selector: String,
        #[arg(long)]
        attr: Option<String>,
        #[arg(long)]
        html: bool,
    },
    /// Print the text of every element matching a selector.
    QueryAll { url: String, selector: String },
    /// Print the text of the first node matching an XPath expression.
    Xpath { url: String, xpath: String },
    /// Evaluate a JavaScript expression and print the JSON result.
    Evaluate { url: String, expression: String },
    /// Print the page's accessibility snapshot (semantic role/name/value nodes).
    Accessibility { url: String },
    /// Print an agent-friendly, indexed snapshot of the page's interactive elements.
    Snapshot { url: String },
    /// Click the element at `index` from the snapshot of this same run.
    ClickIndex { url: String, index: u32 },
    /// Print the page's main content as Markdown.
    Markdown { url: String },
    /// Report anti-bot / challenge markers found on the page (JSON).
    Challenge { url: String },
    /// Print how many elements contain the given text (1 = found).
    Find { url: String, text: String },
    /// Wait until the network is idle, then print the current URL.
    WaitIdle { url: String },
    /// Record network activity while loading a URL and save a HAR 1.2 file.
    Har {
        url: String,
        #[arg(short, long, default_value = "network.har")]
        output: PathBuf,
        /// Embed response bodies in the HAR (via Network.getResponseBody).
        #[arg(long)]
        bodies: bool,
    },
    /// Set a download directory, open a URL, and wait for the first download.
    Download {
        url: String,
        #[arg(short, long, value_name = "DIR")]
        output: PathBuf,
    },
    /// Copy a Chrome profile directory (e.g. the system `Default` profile).
    ReuseProfile { source: String, dest: PathBuf },
    /// List installed Chrome profiles discovered on this machine.
    Profiles,
    /// Print a browser health report (JSON) for a URL.
    Health { url: String },
    /// Record a video of a page for a fixed duration.
    Record {
        url: String,
        #[arg(short, long, default_value = "video.mp4")]
        output: PathBuf,
        /// How long to record, in seconds.
        #[arg(short, long, default_value_t = 5.0)]
        duration: f64,
        /// JPEG quality for each captured frame (1-100).
        #[arg(long, default_value_t = 80)]
        quality: u32,
        /// Output frame rate for the native (AVI) back end.
        #[arg(long, default_value_t = 12)]
        fps: u32,
        /// Never use ffmpeg; always write a native Motion-JPEG AVI.
        #[arg(long)]
        no_ffmpeg: bool,
    },
    /// List known devices, browsers, or plugins.
    List {
        /// Which category to list. Omit to list all of them.
        #[arg(value_enum)]
        kind: Option<ListKind>,
        /// Include browsers that are not installed.
        #[arg(long)]
        all: bool,
    },
    /// Alias for `list plugin`.
    Plugins,
    /// Create a new mod (plugin) from the starter template.
    Plugin {
        #[command(subcommand)]
        action: PluginAction,
    },
    /// Run the Model Context Protocol (MCP) server on stdio.
    Mcp,
    /// Keep one browser open and read commands from stdin (continuous interaction).
    Session {
        /// Optional URL to open at startup.
        #[arg(long, value_name = "URL")]
        start: Option<String>,
    },
    /// Run an AI-driven session: a visible window, the cursor, and commands on
    /// stdin. A human can watch and step in at any time.
    Live {
        /// Optional URL to open at startup.
        #[arg(long, value_name = "URL")]
        start: Option<String>,
    },
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

/// Categories for `xcelerate list`.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ListKind {
    /// Known browsers and whether they are installed.
    #[value(alias = "browsers")]
    Browser,
    /// External plugins and how to load them.
    #[value(alias = "plugins")]
    Plugin,
    /// Built-in device profiles.
    #[value(alias = "devices")]
    Device,
}

/// Target language for `--codegen`.
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
