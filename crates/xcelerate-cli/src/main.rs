//! Command-line interface for the xcelerate CDP engine.
//!
//! Each page command launches a fresh browser, performs one action, and exits.
//! `xcelerate-cli mcp` instead starts the Model Context Protocol server on stdio.

use mimalloc::MiMalloc;
use std::path::PathBuf;
use std::sync::Arc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use clap::{Args, Parser, Subcommand};

use xcelerate::{Browser, BrowserConfig, Page, VideoOptions, XcelerateResult};

#[derive(Parser)]
#[command(
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
    /// Path to the Chrome/Edge executable.
    #[arg(long, global = true, value_name = "PATH")]
    executable_path: Option<String>,
    /// First-party plugins to enable (comma-separated): stealth,human.
    #[arg(long, global = true, value_name = "LIST", value_delimiter = ',')]
    plugins: Vec<String>,
    /// Default wait timeout in milliseconds (0 disables it).
    #[arg(long, global = true, default_value_t = 30000)]
    timeout: u64,
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
    /// Evaluate a JavaScript expression and print the JSON result.
    Evaluate { url: String, expression: String },
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
    /// List the compiled-in first-party plugins.
    Plugins,
    /// Run the Model Context Protocol (MCP) server on stdio.
    Mcp,
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
        Command::Plugins => {
            for name in xcelerate::plugin::builtin_names() {
                println!("{name}");
            }
        }
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
            for element in Arc::clone(&page).query_selector_all(selector).await? {
                println!("{}", element.text().await.unwrap_or_default());
            }
            browser.close().await?;
        }
        Command::Evaluate { url, expression } => {
            let (browser, page) = launch(&cli.browser, &url).await?;
            println!("{}", page.evaluate_json(expression).await?);
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

/// Launches a browser and opens `url`, applying the shared browser options.
async fn launch(args: &BrowserArgs, url: &str) -> XcelerateResult<(Arc<Browser>, Arc<Page>)> {
    let config = BrowserConfig {
        headless: !args.no_headless,
        detached: args.detached,
        executable_path: args.executable_path.clone(),
        plugins: if args.plugins.is_empty() {
            None
        } else {
            Some(args.plugins.clone())
        },
    };
    let browser = Browser::launch(config).await?;
    let page = Arc::clone(&browser).new_page(url.to_string()).await?;
    if args.timeout > 0 {
        page.set_default_timeout(args.timeout as f64).await?;
    }
    // `new_page` returns as soon as navigation is issued; wait for the load
    // event so client-rendered pages are populated before we read them.
    let _ = page.wait_for_navigation().await;
    Ok((browser, page))
}
