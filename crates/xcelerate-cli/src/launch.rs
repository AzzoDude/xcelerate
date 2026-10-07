//! Shared browser launch: applies the global options and opens the first page.

use std::sync::Arc;

use xcelerate::{Browser, BrowserConfig, Page, XcelerateResult};

use crate::cli::BrowserArgs;

/// The build command to suggest after scaffolding, per platform.
pub fn build_hint() -> &'static str {
    if cfg!(windows) {
        ".\\build.ps1"
    } else {
        "./build.sh"
    }
}

/// Launches a browser and opens `url`, applying the shared browser options.
pub async fn launch(args: &BrowserArgs, url: &str) -> XcelerateResult<(Arc<Browser>, Arc<Page>)> {
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
        // A live run (`--ai` / `--codegen` / `--gate os`) must be on screen for the
        // cursor and input gate to matter.
        headless: !(args.no_headless || args.live()),
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

    // A live run is watchable, so mark it with the in-page cursor dot (and its
    // input gate).
    if args.live() {
        crate::cursor::install(&page).await?;
    }

    Ok((browser, page))
}
