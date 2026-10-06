//! Well-known browsers and how to find them.
//!
//! One table drives every browser lookup in the crate. It replaces the per-engine
//! constant lists that used to live in `chromium.rs` (`find_chrome_executable`)
//! and `firefox.rs` (`find_firefox`), so adding a browser is a one-line change
//! here instead of an edit in two places.
//!
//! You choose the browser by id (`XCELERATE_BROWSER=brave`) or by path
//! (`BrowserConfig::executable_path`). When neither is given, the first installed
//! browser for the engine is used.

use std::path::PathBuf;
use std::sync::OnceLock;

/// The remote protocol an engine speaks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Engine {
    /// Chrome DevTools Protocol — Chrome, Chromium, Edge, Brave, Vivaldi, Opera.
    Chromium,
    /// WebDriver BiDi — Firefox.
    Firefox,
}

/// A browser xcelerate knows how to launch.
pub struct KnownBrowser {
    /// Stable id used in `XCELERATE_BROWSER`, e.g. `chrome`.
    pub id: &'static str,
    /// Human-readable name, e.g. `Google Chrome`.
    pub name: &'static str,
    /// Which protocol/engine it speaks.
    pub engine: Engine,
    /// Candidate executable paths, tried in order. Entries for other operating
    /// systems are harmless: a path that does not exist simply fails `is_file`.
    pub paths: &'static [&'static str],
}

/// Every browser the crate can discover, most common first within each engine.
pub const KNOWN: &[KnownBrowser] = &[
    KnownBrowser {
        id: "chrome",
        name: "Google Chrome",
        engine: Engine::Chromium,
        paths: &[
            r"C:\Program Files\Google\Chrome\Application\chrome.exe",
            r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            "/usr/bin/google-chrome",
            "/usr/bin/google-chrome-stable",
        ],
    },
    KnownBrowser {
        id: "chromium",
        name: "Chromium",
        engine: Engine::Chromium,
        paths: &[
            r"C:\Program Files\Chromium\Application\chrome.exe",
            "/Applications/Chromium.app/Contents/MacOS/Chromium",
            "/usr/bin/chromium",
            "/usr/bin/chromium-browser",
            "/snap/bin/chromium",
        ],
    },
    KnownBrowser {
        id: "edge",
        name: "Microsoft Edge",
        engine: Engine::Chromium,
        paths: &[
            r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
            r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
            "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
            "/usr/bin/microsoft-edge-stable",
            "/usr/bin/microsoft-edge",
        ],
    },
    KnownBrowser {
        id: "brave",
        name: "Brave",
        engine: Engine::Chromium,
        paths: &[
            r"C:\Program Files\BraveSoftware\Brave-Browser\Application\brave.exe",
            r"C:\Program Files (x86)\BraveSoftware\Brave-Browser\Application\brave.exe",
            "/Applications/Brave Browser.app/Contents/MacOS/Brave Browser",
            "/usr/bin/brave-browser",
        ],
    },
    KnownBrowser {
        id: "vivaldi",
        name: "Vivaldi",
        engine: Engine::Chromium,
        paths: &[
            r"C:\Program Files\Vivaldi\Application\vivaldi.exe",
            "/Applications/Vivaldi.app/Contents/MacOS/Vivaldi",
            "/usr/bin/vivaldi",
        ],
    },
    KnownBrowser {
        id: "opera",
        name: "Opera",
        engine: Engine::Chromium,
        paths: &[
            r"C:\Program Files\Opera\launcher.exe",
            "/Applications/Opera.app/Contents/MacOS/Opera",
            "/usr/bin/opera",
        ],
    },
    KnownBrowser {
        id: "firefox",
        name: "Mozilla Firefox",
        engine: Engine::Firefox,
        paths: &[
            r"C:\Program Files\Mozilla Firefox\firefox.exe",
            r"C:\Program Files (x86)\Mozilla Firefox\firefox.exe",
            "/Applications/Firefox.app/Contents/MacOS/firefox",
            "/usr/bin/firefox",
            "/snap/bin/firefox",
        ],
    },
    KnownBrowser {
        id: "firefox-esr",
        name: "Mozilla Firefox ESR",
        engine: Engine::Firefox,
        paths: &[
            r"C:\Program Files\Mozilla Firefox ESR\firefox.exe",
            "/Applications/Firefox ESR.app/Contents/MacOS/firefox",
            "/usr/bin/firefox-esr",
        ],
    },
];

/// Alternate ids accepted on top of the canonical [`KnownBrowser::id`].
const ALIASES: &[(&str, &str)] = &[
    ("google-chrome", "chrome"),
    ("google-chrome-stable", "chrome"),
    ("chromium-browser", "chromium"),
    ("msedge", "edge"),
    ("microsoft-edge", "edge"),
    ("microsoft-edge-stable", "edge"),
    ("brave-browser", "brave"),
    ("mozilla-firefox", "firefox"),
];

/// Every browser the crate can discover.
pub fn all() -> &'static [KnownBrowser] {
    KNOWN
}

/// Look up a known browser by id or alias (case-insensitive).
pub fn lookup(id: &str) -> Option<&'static KnownBrowser> {
    let id = id.trim().to_ascii_lowercase();
    if let Some(found) = KNOWN.iter().find(|browser| browser.id == id) {
        return Some(found);
    }
    let canonical = ALIASES
        .iter()
        .find(|(alias, _)| *alias == id)
        .map(|(_, target)| *target)?;
    KNOWN.iter().find(|browser| browser.id == canonical)
}

/// The first existing candidate path for `browser`.
pub fn first_existing(browser: &KnownBrowser) -> Option<PathBuf> {
    browser
        .paths
        .iter()
        .map(PathBuf::from)
        .find(|path| path.is_file())
}

/// Resolve the executable to launch.
///
/// Precedence: an explicit `requested` value (a file path, or a known id such as
/// `brave`), then `XCELERATE_BROWSER`, then the first installed browser of
/// `engine`. Returns `None` when nothing suitable is installed.
pub fn resolve(requested: Option<&str>, engine: Engine) -> Option<PathBuf> {
    if let Some(value) = requested.map(str::trim).filter(|value| !value.is_empty()) {
        let path = PathBuf::from(value);
        if path.is_file() {
            return Some(path);
        }
        if let Some(browser) = lookup(value) {
            return first_existing(browser);
        }
        // Not a known id and not on disk: hand the path back so the launch error
        // names exactly what the caller asked for.
        return Some(path);
    }

    if let Ok(value) = std::env::var("XCELERATE_BROWSER") {
        let value = value.trim().to_string();
        if !value.is_empty() {
            return resolve(Some(&value), engine);
        }
    }

    discovered(engine).clone()
}

/// The first installed browser of `engine`, probed once per process.
///
/// Discovery touches the filesystem; caching it keeps repeated launches cheap and
/// avoids re-stat'ing a fixed set of paths on every `Browser::launch`.
fn discovered(engine: Engine) -> &'static Option<PathBuf> {
    static CHROMIUM: OnceLock<Option<PathBuf>> = OnceLock::new();
    static FIREFOX: OnceLock<Option<PathBuf>> = OnceLock::new();
    let slot = match engine {
        Engine::Chromium => &CHROMIUM,
        Engine::Firefox => &FIREFOX,
    };
    slot.get_or_init(|| {
        KNOWN
            .iter()
            .filter(|browser| browser.engine == engine)
            .find_map(first_existing)
    })
}

/// All known browser ids, for error messages and the CLI.
pub fn ids() -> impl Iterator<Item = &'static str> {
    KNOWN.iter().map(|browser| browser.id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_accepts_ids_and_aliases() {
        assert_eq!(lookup("chrome").unwrap().id, "chrome");
        assert_eq!(lookup("Brave").unwrap().id, "brave");
        assert_eq!(lookup("google-chrome").unwrap().id, "chrome");
        assert_eq!(lookup("msedge").unwrap().id, "edge");
        assert_eq!(lookup("microsoft-edge-stable").unwrap().id, "edge");
        assert!(lookup("netscape").is_none());
    }

    #[test]
    fn every_engine_has_at_least_one_browser() {
        assert!(KNOWN.iter().any(|b| b.engine == Engine::Chromium));
        assert!(KNOWN.iter().any(|b| b.engine == Engine::Firefox));
    }

    #[test]
    fn an_unknown_but_real_path_is_returned_as_asked() {
        // An explicit path that does not exist is handed back unchanged so the
        // launch error names it, rather than silently falling back.
        let resolved = resolve(Some("/definitely/not/a/browser"), Engine::Chromium);
        assert_eq!(resolved, Some(PathBuf::from("/definitely/not/a/browser")));
    }

    #[test]
    fn an_installed_id_resolves_to_one_of_its_paths() {
        if let Some(path) = resolve(Some("chrome"), Engine::Chromium) {
            assert!(path.is_file());
        }
    }
}
