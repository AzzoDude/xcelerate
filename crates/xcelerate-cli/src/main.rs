//! Command-line interface for the xcelerate CDP engine.
//!
//! Each page command launches a fresh browser, performs one action, and exits.
//! `xcelerate mcp` instead starts the Model Context Protocol server on stdio.

use mimalloc::MiMalloc;
use std::path::PathBuf;
use std::sync::Arc;

mod scaffold;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use clap::{Args, Parser, Subcommand, ValueEnum};

use xcelerate::{Browser, BrowserConfig, Page, VideoOptions, XcelerateResult};

#[derive(Parser)]
#[command(
    name = "xcelerate",
    version,
    about = "Browser automation from the command line, powered by xcelerate.",
    propagate_version = true
)]
struct Cli {
    #[command(flatten)]
    browser: BrowserArgs,
    #[command(subcommand)]
    command: Command,
}

#[derive(Args, Clone)]
struct BrowserArgs {
    /// Show the browser window (headless by default).
    #[arg(long, global = true)]
    no_headless: bool,
    /// Detach the browser process so it outlives this command.
    #[arg(long, global = true)]
    detached: bool,
    /// Path to the browser executable.
    #[arg(long, global = true, value_name = "PATH")]
    executable_path: Option<String>,
    /// Browser to use: a known id (`chrome`, `chromium`, `edge`, `brave`,
    /// `vivaldi`, `opera`, `firefox`, `firefox-esr`) or a path to the executable.
    /// Overrides `--executable-path`; also settable via `XCELERATE_BROWSER`.
    #[arg(long, global = true, value_name = "ID")]
    browser: Option<String>,
    /// External plugin paths to load (comma-separated).
    #[arg(long, global = true, value_name = "PATH", value_delimiter = ',')]
    plugins: Vec<String>,
    /// Emulate a mobile device for this run (see `xcelerate list`).
    #[arg(long, global = true, value_name = "NAME")]
    device: Option<String>,
    /// Upstream proxy URL(s); repeat for a pool. `http://[user:pass@]host:port`.
    #[arg(long, global = true, value_name = "URL")]
    proxy: Vec<String>,
    /// Persistent profile directory; keeps logins/cookies between runs.
    #[arg(long, global = true, value_name = "PATH")]
    user_data_dir: Option<String>,
    /// Default wait timeout in milliseconds (0 disables it).
    #[arg(long, global = true, default_value_t = 30000)]
    timeout: u64,
    /// Allowed navigation domains (repeatable). When set, only these may load.
    #[arg(long = "allow-domain", global = true, value_name = "DOMAIN")]
    allow_domain: Vec<String>,
    /// Prohibited navigation domains (repeatable). Overrides the allow list.
    #[arg(long = "deny-domain", global = true, value_name = "DOMAIN")]
    deny_domain: Vec<String>,
    /// Attach to an existing browser's CDP websocket instead of launching one.
    #[arg(long, global = true, value_name = "WS_URL")]
    connect: Option<String>,
    /// Extra browser flag passed verbatim (repeatable).
    #[arg(long = "extra-arg", global = true, value_name = "FLAG")]
    extra_arg: Vec<String>,
    /// Allow the browser to download files.
    #[arg(long, global = true)]
    accept_downloads: bool,
    /// Enable deterministic-rendering flags.
    #[arg(long, global = true)]
    deterministic: bool,
    /// Disable web security / site isolation (testing only).
    #[arg(long, global = true)]
    disable_security: bool,
    /// Keep the browser process alive after the command exits.
    #[arg(long, global = true)]
    keep_alive: bool,
}

#[derive(Subcommand)]
enum Command {
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
}

#[derive(Subcommand)]
enum PluginAction {
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
enum ListKind {
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

fn list_devices() {
    println!("Devices:");
    for device in xcelerate::devices::all() {
        println!(
            "  {:<22} {}x{}  dpr {:<5} {}{}",
            device.name,
            device.width,
            device.height,
            device.device_scale_factor,
            if device.mobile { "mobile" } else { "desktop" },
            if device.has_touch { " touch" } else { "" }
        );
    }
}

fn list_browsers(all: bool) {
    let rows: Vec<_> = xcelerate::browser::known::all()
        .iter()
        .filter_map(|browser| {
            let path = xcelerate::browser::known::first_existing(browser);
            if path.is_none() && !all {
                return None;
            }
            Some((browser, path))
        })
        .collect();

    if rows.is_empty() {
        println!("Browsers: none installed.");
        println!("  Install one, pass --executable-path, or run `xcelerate list browser --all`");
        println!("  to see every id xcelerate knows.");
        return;
    }

    println!("Browsers (choose with --browser <id>):");
    for (browser, path) in rows {
        let engine = match browser.engine {
            xcelerate::browser::known::Engine::Chromium => "chromium",
            xcelerate::browser::known::Engine::Firefox => "firefox",
        };
        let location = path
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "(not installed)".to_string());
        println!(
            "  {:<16} {:<26} {:<9} {}",
            browser.id, browser.name, engine, location
        );
    }
}

fn list_plugins() {
    println!("Plugins: none built in (external by design).");
    println!("  Load one with --plugins <PATH>, or scaffold a new one with");
    println!("  `xcelerate plugin new <id>`.");
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if let Err(error) = run(cli).await {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Command::List { kind, all } => match kind {
            Some(ListKind::Device) => list_devices(),
            Some(ListKind::Browser) => list_browsers(all),
            Some(ListKind::Plugin) => list_plugins(),
            None => {
                list_devices();
                println!();
                list_browsers(all);
                println!();
                list_plugins();
            }
        },
        Command::Plugins => list_plugins(),
        Command::Plugin { action } => match action {
            PluginAction::New { name, dir, force } => {
                let path = scaffold::new_mod(&name, dir, force)?;
                println!("created mod '{name}' in {}", path.display());
                println!(
                    "next: cd {} && {}   # builds the .wasm",
                    path.display(),
                    build_hint()
                );
            }
        },
        Command::Mcp => {
            xcelerate_mcp::run_stdio().await?;
        }
        Command::Open { url } => {
            let (browser, page) = launch(&cli.browser, &url).await?;
            println!("title: {}", page.title().await?);
            println!("url:   {}", page.url().await?);
            browser.close().await?;
        }
        Command::Title { url } => {
            let (browser, page) = launch(&cli.browser, &url).await?;
            println!("{}", page.title().await?);
            browser.close().await?;
        }
        Command::Content { url } => {
            let (browser, page) = launch(&cli.browser, &url).await?;
            println!("{}", page.content().await?);
            browser.close().await?;
        }
        Command::Text { url } => {
            let (browser, page) = launch(&cli.browser, &url).await?;
            let text = page
                .evaluate_string("document.body ? document.body.innerText : ''".to_string())
                .await?;
            println!("{text}");
            browser.close().await?;
        }
        Command::Screenshot { url, output, full } => {
            let (browser, page) = launch(&cli.browser, &url).await?;
            let png = if full {
                page.screenshot_full().await?
            } else {
                page.screenshot().await?
            };
            std::fs::write(&output, &png)?;
            println!("wrote {} ({} bytes)", output.display(), png.len());
            browser.close().await?;
        }
        Command::Pdf { url, output } => {
            let (browser, page) = launch(&cli.browser, &url).await?;
            let pdf = page.pdf().await?;
            std::fs::write(&output, &pdf)?;
            println!("wrote {} ({} bytes)", output.display(), pdf.len());
            browser.close().await?;
        }
        Command::Query {
            url,
            selector,
            attr,
            html,
        } => {
            let (browser, page) = launch(&cli.browser, &url).await?;
            let element = Arc::clone(&page).wait_for_selector(selector).await?;
            if let Some(attribute) = attr {
                println!(
                    "{}",
                    element.attribute(attribute).await?.unwrap_or_default()
                );
            } else if html {
                println!("{}", element.inner_html().await?);
            } else {
                println!("{}", element.text().await?);
            }
            browser.close().await?;
        }
        Command::QueryAll { url, selector } => {
            let (browser, page) = launch(&cli.browser, &url).await?;
            // Like `query`, wait for the selector to appear before reading: a
            // client-rendered page may not have populated the DOM when the load
            // event fires. The wait is bounded by the page timeout; if nothing
            // shows up we still print nothing rather than failing.
            let _ = Arc::clone(&page).wait_for_selector(selector.clone()).await;
            for element in Arc::clone(&page).query_selector_all(selector).await? {
                println!("{}", element.text().await.unwrap_or_default());
            }
            browser.close().await?;
        }
        Command::Xpath { url, xpath } => {
            let (browser, page) = launch(&cli.browser, &url).await?;
            // Wait: client-rendered pages may not have the element yet when the
            // load event fires.
            let element = Arc::clone(&page).wait_for_xpath(xpath, 30_000).await?;
            println!("{}", element.text().await?);
            browser.close().await?;
        }
        Command::Evaluate { url, expression } => {
            let (browser, page) = launch(&cli.browser, &url).await?;
            println!("{}", page.evaluate_json(expression).await?);
            browser.close().await?;
        }
        Command::Accessibility { url } => {
            let (browser, page) = launch(&cli.browser, &url).await?;
            println!("{}", page.accessibility_snapshot().await?);
            browser.close().await?;
        }
        Command::Snapshot { url } => {
            let (browser, page) = launch(&cli.browser, &url).await?;
            println!("{}", page.agent_snapshot().await?);
            browser.close().await?;
        }
        Command::ClickIndex { url, index } => {
            let (browser, page) = launch(&cli.browser, &url).await?;
            // Build a snapshot so the index refers to this page state, then act on it.
            page.agent_snapshot().await?;
            Arc::clone(&page).click_index(index).await?;
            println!("clicked snapshot index {index}");
            browser.close().await?;
        }
        Command::Markdown { url } => {
            let (browser, page) = launch(&cli.browser, &url).await?;
            println!("{}", page.markdown().await?);
            browser.close().await?;
        }
        Command::Challenge { url } => {
            let (browser, page) = launch(&cli.browser, &url).await?;
            println!("{}", page.detect_challenge().await?.to_json());
            browser.close().await?;
        }
        Command::Find { url, text } => {
            let (browser, page) = launch(&cli.browser, &url).await?;
            println!("{}", page.find_text(text).await?);
            browser.close().await?;
        }
        Command::WaitIdle { url } => {
            let (browser, page) = launch(&cli.browser, &url).await?;
            page.wait_for_network_idle(500, cli.browser.timeout.max(1))
                .await?;
            println!("{}", page.url().await?);
            browser.close().await?;
        }
        Command::Har {
            url,
            output,
            bodies,
        } => {
            // Load about:blank first so the recorded HAR includes the real
            // navigation instead of only late-arriving subresources.
            let (browser, page) = launch(&cli.browser, "about:blank").await?;
            if bodies {
                page.set_har_body_mode("embed".to_string()).await?;
            }
            page.start_har_recording().await?;
            page.navigate(url).await?;
            let _ = page
                .wait_for_network_idle(500, cli.browser.timeout.max(1))
                .await;
            let path = page.save_har(output.to_string_lossy().into_owned()).await?;
            println!("wrote {path}");
            browser.close().await?;
        }
        Command::Download { url, output } => {
            let (browser, page) = launch(&cli.browser, "about:blank").await?;
            page.set_download_path(output.to_string_lossy().into_owned())
                .await?;
            // Start waiting *before* navigating: a small download can begin and
            // finish during `navigate`, and `wait_for_download` only observes
            // events that arrive after it subscribes.
            let (download, _) = tokio::join!(
                page.wait_for_download(cli.browser.timeout.max(1)),
                page.navigate(url),
            );
            println!("{}", download?);
            browser.close().await?;
        }
        Command::ReuseProfile { source, dest } => {
            let path = xcelerate::profile::reuse_system_profile(&source, &dest.to_string_lossy())?;
            println!("{path}");
        }
        Command::Profiles => {
            let profiles = xcelerate::profile::list_chrome_profiles();
            if profiles.is_empty() {
                println!("no Chrome profiles found");
            }
            for profile in profiles {
                println!("{}\t{}\t{}", profile.name, profile.directory, profile.path);
            }
        }
        Command::Health { url } => {
            let (browser, page) = launch(&cli.browser, &url).await?;
            println!("{}", page.health().await?);
            browser.close().await?;
        }
        Command::Record {
            url,
            output,
            duration,
            quality,
            fps,
            no_ffmpeg,
        } => {
            let (browser, page) = launch(&cli.browser, &url).await?;
            let options = VideoOptions {
                quality,
                max_width: 0,
                max_height: 0,
                fps,
                ffmpeg: !no_ffmpeg,
            };
            page.start_video_with_options(output.to_string_lossy().into_owned(), options)
                .await?;
            tokio::time::sleep(std::time::Duration::from_secs_f64(duration.max(0.1))).await;
            match page.stop_video().await? {
                Some(path) => println!("wrote {path}"),
                None => println!("recording did not start"),
            }
            browser.close().await?;
        }
    }
    Ok(())
}

/// The build command to suggest after scaffolding, per platform.
fn build_hint() -> &'static str {
    if cfg!(windows) {
        ".\\build.ps1"
    } else {
        "./build.sh"
    }
}

/// Launches a browser and opens `url`, applying the shared browser options.
async fn launch(args: &BrowserArgs, url: &str) -> XcelerateResult<(Arc<Browser>, Arc<Page>)> {
    if !args.proxy.is_empty() {
        xcelerate::configure_proxy(&args.proxy)?;
    }
    if args.user_data_dir.is_some() {
        xcelerate::configure_user_data_dir(args.user_data_dir.clone())?;
    }
    if !args.allow_domain.is_empty() || !args.deny_domain.is_empty() {
        xcelerate::configure_domain_policy(args.allow_domain.clone(), args.deny_domain.clone());
    }
    if !args.extra_arg.is_empty()
        || args.accept_downloads
        || args.deterministic
        || args.disable_security
        || args.keep_alive
    {
        xcelerate::configure_launch_options(xcelerate::LaunchOptions {
            extra_args: args.extra_arg.clone(),
            accept_downloads: args.accept_downloads.then_some(true),
            deterministic_rendering: args.deterministic,
            disable_security: args.disable_security,
            keep_alive: args.keep_alive,
        });
    }
    let config = BrowserConfig {
        headless: !args.no_headless,
        detached: args.detached,
        executable_path: args
            .browser
            .clone()
            .or_else(|| args.executable_path.clone()),
        plugins: if args.plugins.is_empty() {
            None
        } else {
            Some(args.plugins.clone())
        },
    };
    let browser = match args.connect.clone() {
        Some(ws_url) => Browser::connect(ws_url).await?,
        None => Browser::launch(config).await?,
    };
    let page = if let Some(device) = args.device.clone() {
        // Emulate before navigating so the UA, touch, and viewport are in place
        // for the first request and the initial layout.
        let page = Arc::clone(&browser)
            .new_page("about:blank".to_string())
            .await?;
        page.emulate_device(device).await?;
        page.navigate(url.to_string()).await?;
        page
    } else {
        Arc::clone(&browser).new_page(url.to_string()).await?
    };
    if args.timeout > 0 {
        page.set_default_timeout(args.timeout as f64).await?;
    }
    // `new_page` returns as soon as navigation is issued; wait for the load
    // event so client-rendered pages are populated before we read them.
    let _ = page.wait_for_navigation().await;
    // A client-rendered page can keep rendering after the load event, so give
    // the DOM a brief, capped window to settle. This keeps the one-shot readers
    // (`content`, `text`, `evaluate`, `find`, `markdown`, `accessibility`,
    // `snapshot`, ...) from observing a half-rendered page. The cap stops a
    // long-polling or constantly-mutating page from hanging the CLI.
    let _ = page.wait_for_dom_stable(300, 2_000).await;
    Ok((browser, page))
}
