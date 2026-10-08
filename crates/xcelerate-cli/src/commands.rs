//! One-shot subcommands: launch a browser, do one thing, exit. Also the
//! `list` helpers and the top-level `run` dispatcher.

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

/// `apps`: list open native windows, fastest path (one UIA `FindAll`).
fn list_apps() {
    #[cfg(windows)]
    match xcelerate_uia::Uia::new().and_then(|uia| uia.windows()) {
        Ok(windows) => {
            for window in windows {
                println!("{}", xcelerate_uia::format_window(&window));
            }
        }
        Err(error) => println!("apps: {error}"),
    }
    #[cfg(not(windows))]
    println!("`apps` (native window listing) is Windows only.");
}

/// Formats `Target.getTargets` output as printable lines (page targets only), so
/// the ids can be fed straight to `--attach <id>`.
fn format_targets(json: &str) -> Vec<String> {
    let parsed: serde_json::Value = match serde_json::from_str(json) {
        Ok(value) => value,
        Err(_) => return vec![json.to_string()],
    };
    let infos = parsed
        .get("targetInfos")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut lines = Vec::new();
    let mut index = 0usize;
    for info in infos {
        if info.get("type").and_then(serde_json::Value::as_str) != Some("page") {
            continue;
        }
        let id = info
            .get("targetId")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        let title = info
            .get("title")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        let url = info
            .get("url")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        lines.push(format!("[{index}] \"{title}\"  {url}\n      id: {id}"));
        index += 1;
    }
    if lines.is_empty() {
        lines.push("(no page targets)".to_string());
    }
    lines
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
        Command::Apps => list_apps(),
        Command::Targets => {
            let ws_url = cli
                .browser
                .connect
                .clone()
                .ok_or("`targets` requires --connect <WS_URL> (list a running browser/app)")?;
            let browser = xcelerate::Browser::connect(ws_url).await?;
            let json = browser.targets().await?;
            for line in format_targets(&json) {
                println!("{line}");
            }
        }
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
            crate::run::run_file(
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
        Command::Save { url, output } => {
            let (browser, page) = launch(&cli.browser, &url).await?;
            let target = output
                .map(|path| path.to_string_lossy().into_owned())
                .unwrap_or_else(|| {
                    // Fall back to the URL's last path segment, or `download`.
                    url.rsplit('/')
                        .next()
                        .filter(|name| !name.is_empty())
                        .unwrap_or("download")
                        .to_string()
                });
            let bytes = page.save_url(url.clone(), target.clone()).await?;
            println!("{target} ({bytes} bytes)");
            browser.close().await?;
        }
        Command::Grab { url, output } => {
            let target = output.to_string_lossy().into_owned();
            let (browser, page) = launch(&cli.browser, "about:blank").await?;
            let summary = page.grab(url.clone(), target.clone()).await?;
            println!("{summary}");
            browser.close().await?;
        }
        Command::Capture {
            url,
            output,
            seconds,
        } => {
            let target = output.to_string_lossy().into_owned();
            let (browser, page) = launch(&cli.browser, "about:blank").await?;
            // Start capturing before navigating so the init segment is seen too.
            page.start_media_capture().await?;
            page.navigate(url.clone()).await?;
            let _ = page.wait_for_navigation().await;
            // Best effort: give the player a nudge (autoplay is often blocked).
            let _ = page
                .evaluate_string(
                    "(function(){ const v=document.querySelector('video'); \
                     if (v) { v.muted = true; if (v.play) v.play().catch(function(){}); } \
                     return true; })()"
                        .to_string(),
                )
                .await;
            tokio::time::sleep(std::time::Duration::from_secs(seconds)).await;
            let summary = page.save_capture(target.clone()).await?;
            println!("{summary}");
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
            let body = xcelerate_interpreter::net::fetch(&url).await?;
            println!("{body}");
        }
        #[cfg(feature = "http")]
        Command::FetchTo { url, output } => {
            let bytes = xcelerate_interpreter::net::download(&url, &output).await?;
            println!("wrote {} ({bytes} bytes)", output.display());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::format_targets;

    #[test]
    fn formats_only_page_targets() {
        let json = r#"{"targetInfos":[
            {"targetId":"A1","type":"page","title":"GitHub Desktop","url":"app://index.html"},
            {"targetId":"B2","type":"background_page","title":"bg","url":"devtools://bg"}
        ]}"#;
        let lines = format_targets(json);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("GitHub Desktop"));
        assert!(
            lines[0].contains("A1"),
            "the id must be printable for --attach"
        );
    }

    #[test]
    fn reports_when_there_are_no_page_targets() {
        assert_eq!(
            format_targets(r#"{"targetInfos":[]}"#),
            vec!["(no page targets)".to_string()]
        );
    }
}
