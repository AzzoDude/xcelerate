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
    pub command: Option<Command>,
}

#[derive(Args, Clone)]
pub struct BrowserArgs {
    /// Run without a visible window. The browser is shown by default.
    #[arg(long, global = true)]
    pub headless: bool,
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
    /// Native windows (by title/process glob) an XCL script may drive. Empty
    /// denies all native actions (listing windows is still allowed); repeat for
    /// more, e.g. `--allow-app "Steam*"`.
    #[arg(long = "allow-app", global = true, value_name = "PATTERN")]
    pub allow_app: Vec<String>,
    /// Attach to an existing browser's CDP websocket instead of launching one.
    #[arg(long, global = true, value_name = "WS_URL")]
    pub connect: Option<String>,
    /// Attach to an existing target (window/tab) by id instead of opening a new
    /// one. Pair with `--connect` to drive a running browser or an
    /// Electron/CEF/WebView2 app without spawning a window. Find ids with
    /// `xcelerate targets`.
    #[arg(long, global = true, value_name = "TARGET_ID")]
    pub attach: Option<String>,
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
    /// Input with a straight-line mouse move and fast typing instead of the
    /// default human-like motion (curved, jittered path; paced typing). Use it
    /// for reproducible runs and CI.
    #[arg(long, global = true)]
    pub linear: bool,
    /// Directory that every file verb (`download`, `capture`, `shot`, `upload`)
    /// is confined to. Defaults to the current directory; a script can never
    /// name a path outside it (no absolute paths, no `..` escapes).
    #[arg(long = "output-dir", global = true, value_name = "DIR")]
    pub output_dir: Option<PathBuf>,
}

impl BrowserArgs {
    /// Whether the run is watchable: a visible window and the in-page cursor.
    /// True for `--ai` and `--codegen`.
    pub fn live(&self) -> bool {
        self.ai || self.codegen.is_some()
    }

    /// Whether the browser will run headless (no visible window).
    pub fn headless(&self) -> bool {
        // Visible by default; `--headless` opts into a hidden window. A live run
        // (`--ai` / `--codegen`) always needs the window on screen.
        self.headless && !self.live()
    }

    /// Whether the in-page cursor dot should be installed. The cursor is only
    /// meaningful on a visible window, so headless runs never show it — even if
    /// some flag combination would otherwise set `live()`.
    pub fn cursor_active(&self) -> bool {
        self.live() && !self.headless()
    }
}

#[derive(Subcommand)]
pub enum Command {
    /// Open a URL and print its title and URL.
    Open { url: String },
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
    /// Download a URL through the browser and save it to a file.
    Save {
        url: String,
        /// Destination file (default: the URL's last path segment).
        #[arg(short, long, value_name = "PATH")]
        output: Option<PathBuf>,
    },
    /// Download media from a URL: a direct file, an HLS (`.m3u8`) stream, or a
    /// DASH (`.mpd`) manifest, assembled natively.
    Grab {
        url: String,
        /// Destination file (default: `out.mp4`).
        #[arg(short, long, value_name = "PATH", default_value = "out.mp4")]
        output: PathBuf,
    },
    /// Capture the media a page plays itself (MSE/HLS/DASH — YouTube, Facebook)
    /// by recording the segment requests it makes, then reassembling them. No
    /// external tool; the page must actually play.
    Capture {
        url: String,
        /// Destination file (default: `out.mp4`); separate audio goes beside it.
        #[arg(short, long, value_name = "PATH", default_value = "out.mp4")]
        output: PathBuf,
        /// How long to let the page play before assembling (seconds).
        #[arg(long, default_value_t = 15)]
        seconds: u64,
    },
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
    /// Alias for `list plugin`.
    Plugins,
    /// List the pages/windows of a running browser or app (requires `--connect`).
    Targets,
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
    /// Execute an XCL script (`.xcl`). Shares the session command language.
    Run {
        /// Path to the `.xcl` file.
        path: PathBuf,
        /// Allow `eval <js>` and `request <...>` (otherwise denied by default).
        #[arg(long)]
        allow_unsafe: bool,
        /// Allow `request` (browserless HTTP) specifically.
        #[arg(long)]
        allow_http: bool,
        /// Allow `import`/`run` of these plugins (repeatable, supports `*`).
        #[arg(long = "allow-plugin", value_name = "ID", global = true)]
        allow_plugin: Vec<String>,
        /// Allow private/loopback/metadata hosts in browserless requests.
        #[arg(long)]
        allow_private: bool,
        /// Override a script `param` (`key=value`).
        #[arg(long = "param", value_name = "KEY=VALUE")]
        param: Vec<String>,
        /// Print one `ok <step>` line per step. Quiet by default; use the `print`
        /// command to log from the script.
        #[arg(long, short)]
        verbose: bool,
        /// Raise the total-step cap (default 10000). A safety floor, not a
        /// ceiling: raise it for a long, trusted script.
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
        /// Attach to a native window (Windows) instead of exposing the browser
        /// driver; the run drives it through the desktop driver (`launch`,
        /// `window`, `click`, `fill`, ...) and the browser is unavailable.
        #[arg(long, value_name = "TITLE")]
        app: Option<String>,
        /// Run with no browser driver at all (Windows): the desktop driver is
        /// active and the window is chosen in-script by `window`/`launch`.
        #[arg(long)]
        native: bool,
    },
    /// Fetch a URL and print the body (JSON is pretty-printed). Requires the
    /// `http` feature.
    #[cfg(feature = "http")]
    Fetch { url: String },
    /// Download a URL straight to a file, streamed to disk. Requires the `http`
    /// feature.
    #[cfg(feature = "http")]
    FetchTo {
        url: String,
        #[arg(short, long)]
        output: PathBuf,
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
