//! One-shot subcommands: launch a browser, do one thing, exit. Also the
//! `list` helpers and the top-level `run` dispatcher.

use std::sync::Arc;

use xcelerate::VideoOptions;

use crate::cli::{Cli, Command, ListKind, PluginAction};
use crate::launch::{build_hint, launch};
use crate::session::run_session;

pub fn list_devices() {
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

pub fn list_browsers(all: bool) {
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

pub fn list_plugins() {
    println!("Plugins: none built in (external by design).");
    println!("  Load one with --plugins <PATH>, or scaffold a new one with");
    println!("  `xcelerate plugin new <id>`.");
}

pub async fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
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
                let path = crate::scaffold::new_mod(&name, dir, force)?;
                println!("created mod '{name}' in {}", path.display());
                println!(
                    "next: cd {} && {}   # builds the .wasm",
                    path.display(),
                    build_hint()
                );
            }
        },
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
        Command::Mcp => {
            xcelerate_mcp::run_stdio().await?;
        }
        Command::Session { start } => {
            run_session(&cli.browser, start).await?;
        }
        Command::Live { start } => {
            // Live mode is the session with the window and OS-level input gate
            // forced on: an AI drives while a human only watches. Visible is the
            // default (and `live()` forces it), so nothing else to set.
            let mut args = cli.browser.clone();
            args.ai = true;
            run_session(&args, start).await?;
        }
        Command::Run {
            path,
            allow_unsafe,
            allow_http,
            allow_plugin,
            allow_private,
            param,
            verbose,
        } => {
            crate::xcl::run_file(
                &cli.browser,
                &path,
                allow_unsafe,
                allow_http,
                allow_plugin,
                allow_private,
                param,
                verbose,
            )
            .await?;
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
        #[cfg(feature = "http")]
        Command::Fetch { url } => {
            let body = crate::net::fetch(&url).await?;
            println!("{body}");
        }
        #[cfg(feature = "http")]
        Command::FetchTo { url, output } => {
            let bytes = crate::net::download(&url, &output).await?;
            println!("wrote {} ({bytes} bytes)", output.display());
        }
    }
    Ok(())
}
