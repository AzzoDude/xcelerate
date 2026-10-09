//! Shared browser launch: applies the global options and opens the first page.

use std::sync::Arc;

use xcelerate::browser::known::Engine;
use xcelerate::{Browser, BrowserConfig, Page, XcelerateError, XcelerateResult};

use crate::cli::BrowserArgs;

/// The command to suggest after scaffolding. The CLI owns the build (it writes
/// `wit/plugin.wit` and stages the `.wasm`), so point the author at it rather
/// than a hand-rolled script.
pub fn build_hint() -> &'static str {
    "xcelerate build --wasm-only"
}

/// Whether `requested` names (or points at) a Firefox-family browser. The
/// command-line tools speak CDP; Firefox speaks WebDriver BiDi, so this is used
/// to refuse one up front rather than mislaunching it.
fn is_firefox_like(requested: &str) -> bool {
    if xcelerate::browser::known::engine_of(requested) == Some(Engine::Firefox) {
        return true;
    }
    // An explicit executable path: fall back to inspecting the file name.
    let name = std::path::Path::new(requested)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(requested)
        .to_ascii_lowercase();
    [
        "firefox",
        "librewolf",
        "waterfox",
        "floorp",
        "icecat",
        "mullvad",
    ]
    .iter()
    .any(|needle| name.contains(needle))
}

/// Launches a browser and opens `url`, applying the shared browser options.
pub async fn launch(args: &BrowserArgs, url: &str) -> XcelerateResult<(Arc<Browser>, Arc<Page>)> {
    // The command-line tools drive Chromium over CDP. A Firefox-family browser
    // speaks WebDriver BiDi instead, so refuse it up front with a clear message
    // rather than launching it with CDP flags and hanging on the handshake.
    let requested = args
        .browser
        .clone()
        .or_else(|| std::env::var("XCELERATE_BROWSER").ok())
        .or_else(|| args.executable_path.clone());
    if let Some(id) = requested.as_deref()
        && is_firefox_like(id)
    {
        return Err(XcelerateError::Unsupported(format!(
            "`{id}` is a Firefox-family browser; the xcelerate CLI drives Chromium over \
             CDP only. Pick a Chromium browser (e.g. --browser edge), or drive Firefox \
             from Rust with `xcelerate::firefox::FirefoxBrowser`."
        )));
    }

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
        headless: args.headless(),
        detached: args.detached,
        executable_path: args
            .browser
            .clone()
            .or_else(|| args.executable_path.clone()),
        plugins: {
            // `--plugins` takes paths, but a bare *name* (`random-app`) is resolved
            // against the plugin directories so a plugin can be dropped in and
            // loaded by name.
            let resolved = resolve_plugin_names(&args.plugins);
            if resolved.is_empty() {
                None
            } else {
                Some(resolved)
            }
        },
    };
    let browser = match args.connect.clone() {
        Some(ws_url) => Browser::connect(ws_url).await?,
        None => Browser::launch(config).await?,
    };
    // `--attach <id>` takes over an existing window (Electron/CEF/WebView2, or a
    // live browser) instead of opening a new tab. Nothing is navigated, so the
    // window keeps whatever it is showing.
    let attached = args.attach.is_some();
    let page = if let Some(target_id) = args.attach.clone() {
        Arc::clone(&browser).attach_page(target_id).await?
    } else if let Some(device) = args.device.clone() {
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
    // Human vs deterministic input. Human-like motion is the default; `--linear`
    // switches the mouse to a straight line and typing to a fixed fast cadence.
    page.set_human(!args.linear);
    // `new_page` returns as soon as navigation is issued; wait for the load
    // event so client-rendered pages are populated before we read them. An
    // attached target was not navigated, so there is no load event to await.
    if !attached {
        let _ = page.wait_for_navigation().await;
    }
    // A client-rendered page can keep rendering after the load event, so give
    // the DOM a brief, capped window to settle. This keeps the one-shot readers
    // (`content`, `text`, `evaluate`, `find`, `markdown`, `accessibility`,
    // `snapshot`, ...) from observing a half-rendered page. The cap stops a
    // long-polling or constantly-mutating page from hanging the CLI.
    let _ = page.wait_for_dom_stable(300, 2_000).await;

    // A live, visible run is watchable, so mark it with the in-page cursor dot
    // (and its input gate). Headless runs never show the cursor.
    if args.cursor_active() {
        crate::cursor::install(&page).await?;
    }

    Ok((browser, page))
}

/// Resolves each `--plugins` entry to a concrete path (see [`resolve_plugin_name`]).
pub(crate) fn resolve_plugin_names(names: &[String]) -> Vec<String> {
    names.iter().map(|name| resolve_plugin_name(name)).collect()
}

/// Resolves one `--plugins` entry.
///
/// An entry that already exists on disk is used unchanged. Otherwise it is
/// treated as a bare *name* and searched in `$XCELERATE_PLUGIN_DIR`, then the
/// user-global plugin home (`$XCELERATE_HOME`, else `~/.xcl`, `/plugins`),
/// then `./plugins`, then `.`, accepting:
///
/// * `<dir>/<name>/`  — a plugin directory (its `plugin.json` is loaded);
/// * `<dir>/<name>.json` — a manifest (with the `.wasm` beside it).
///
/// So `~/.xcl/plugins/browser/plugin.json` + `browser.wasm` is reachable from
/// anywhere as `--plugins browser` - one build, imported by every project. An
/// unresolved name is returned unchanged, so the loader reports the miss with
/// the name the user typed.
pub(crate) fn resolve_plugin_name(name: &str) -> String {
    if std::path::Path::new(name).exists() {
        return name.to_string();
    }
    let mut dirs: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(dir) = std::env::var("XCELERATE_PLUGIN_DIR")
        && !dir.is_empty()
    {
        dirs.push(dir.into());
    }
    if let Some(home) = plugin_home() {
        dirs.push(home);
    }
    dirs.push("plugins".into());
    dirs.push(".".into());
    for dir in dirs {
        for candidate in [dir.join(name), dir.join(format!("{name}.json"))] {
            if candidate.exists() {
                return candidate.to_string_lossy().into_owned();
            }
        }
    }
    name.to_string()
}

/// The user-global plugin directory: `$XCELERATE_HOME/plugins` when set,
/// otherwise `~/.xcl/plugins`. One shared home means a plugin built once is
/// importable by name from any project or script.
fn plugin_home() -> Option<std::path::PathBuf> {
    if let Ok(dir) = std::env::var("XCELERATE_HOME")
        && !dir.is_empty()
    {
        return Some(std::path::PathBuf::from(dir).join("plugins"));
    }
    let base = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(std::path::PathBuf::from)?;
    Some(base.join(".xcl").join("plugins"))
}

#[cfg(test)]
mod tests {
    use super::resolve_plugin_name;

    #[test]
    fn existing_paths_and_unknown_names_pass_through() {
        // A real path is returned unchanged.
        assert_eq!(resolve_plugin_name("Cargo.toml"), "Cargo.toml");
        // An unknown name is returned unchanged so the loader can name it.
        assert_eq!(
            resolve_plugin_name("no-such-plugin-xyz"),
            "no-such-plugin-xyz"
        );
    }
}
