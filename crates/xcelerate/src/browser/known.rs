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
//!
//! Paths are best-effort: a candidate that does not exist simply does not match,
//! so extra entries for other operating systems cost nothing. Windows candidates
//! may use `%LOCALAPPDATA%`-style tokens, expanded from the environment, because
//! many browsers install per-user.

use std::path::PathBuf;
use std::sync::OnceLock;

/// The remote protocol an engine speaks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Engine {
    /// Chrome DevTools Protocol — Chrome, Chromium, Edge, Brave, Vivaldi, Opera
    /// and every other Chromium-based browser.
    Chromium,
    /// WebDriver BiDi — Firefox and its forks (Firefox 141+, where Mozilla
    /// removed CDP; older forks may not speak BiDi).
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

/// Every browser the crate can discover, grouped by engine, most common first.
pub const KNOWN: &[KnownBrowser] = &[
    // --- Chromium family (Chrome DevTools Protocol) -------------------------
    KnownBrowser {
        id: "chrome",
        name: "Google Chrome",
        engine: Engine::Chromium,
        paths: &[
            r"C:\Program Files\Google\Chrome\Application\chrome.exe",
            r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
            r"%LOCALAPPDATA%\Google\Chrome\Application\chrome.exe",
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            "/usr/bin/google-chrome",
            "/usr/bin/google-chrome-stable",
        ],
    },
    KnownBrowser {
        id: "chrome-beta",
        name: "Google Chrome Beta",
        engine: Engine::Chromium,
        paths: &[
            r"C:\Program Files\Google\Chrome Beta\Application\chrome.exe",
            "%LOCALAPPDATA%\\Google\\Chrome Beta\\Application\\chrome.exe",
            "/Applications/Google Chrome Beta.app/Contents/MacOS/Google Chrome Beta",
            "/usr/bin/google-chrome-beta",
        ],
    },
    KnownBrowser {
        id: "chrome-canary",
        name: "Google Chrome Canary",
        engine: Engine::Chromium,
        paths: &[
            r"%LOCALAPPDATA%\Google\Chrome SxS\Application\chrome.exe",
            "/Applications/Google Chrome Canary.app/Contents/MacOS/Google Chrome Canary",
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
            r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
            r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
            r"%LOCALAPPDATA%\Microsoft\Edge\Application\msedge.exe",
            "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
            "/usr/bin/microsoft-edge-stable",
            "/usr/bin/microsoft-edge",
        ],
    },
    KnownBrowser {
        id: "edge-beta",
        name: "Microsoft Edge Beta",
        engine: Engine::Chromium,
        paths: &[
            r"C:\Program Files (x86)\Microsoft\Edge Beta\Application\msedge.exe",
            "/Applications/Microsoft Edge Beta.app/Contents/MacOS/Microsoft Edge Beta",
            "/usr/bin/microsoft-edge-beta",
        ],
    },
    KnownBrowser {
        id: "edge-dev",
        name: "Microsoft Edge Dev",
        engine: Engine::Chromium,
        paths: &[
            r"C:\Program Files (x86)\Microsoft\Edge Dev\Application\msedge.exe",
            "/Applications/Microsoft Edge Dev.app/Contents/MacOS/Microsoft Edge Dev",
            "/usr/bin/microsoft-edge-dev",
        ],
    },
    KnownBrowser {
        id: "edge-canary",
        name: "Microsoft Edge Canary",
        engine: Engine::Chromium,
        paths: &[
            r"%LOCALAPPDATA%\Microsoft\Edge SxS\Application\msedge.exe",
            "/Applications/Microsoft Edge Canary.app/Contents/MacOS/Microsoft Edge Canary",
        ],
    },
    KnownBrowser {
        id: "brave",
        name: "Brave",
        engine: Engine::Chromium,
        paths: &[
            r"C:\Program Files\BraveSoftware\Brave-Browser\Application\brave.exe",
            r"C:\Program Files (x86)\BraveSoftware\Brave-Browser\Application\brave.exe",
            r"%LOCALAPPDATA%\BraveSoftware\Brave-Browser\Application\brave.exe",
            "/Applications/Brave Browser.app/Contents/MacOS/Brave Browser",
            "/usr/bin/brave-browser",
            "/snap/bin/brave",
        ],
    },
    KnownBrowser {
        id: "vivaldi",
        name: "Vivaldi",
        engine: Engine::Chromium,
        paths: &[
            r"C:\Program Files\Vivaldi\Application\vivaldi.exe",
            r"%LOCALAPPDATA%\Vivaldi\Application\vivaldi.exe",
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
            r"%LOCALAPPDATA%\Programs\Opera\launcher.exe",
            "/Applications/Opera.app/Contents/MacOS/Opera",
            "/usr/bin/opera",
        ],
    },
    KnownBrowser {
        id: "opera-gx",
        name: "Opera GX",
        engine: Engine::Chromium,
        paths: &[
            r"C:\Program Files\Opera GX\launcher.exe",
            r"%LOCALAPPDATA%\Programs\Opera GX\launcher.exe",
            "/Applications/Opera GX.app/Contents/MacOS/Opera GX",
            "/usr/bin/opera-gx",
        ],
    },
    KnownBrowser {
        id: "coc-coc",
        name: "Cốc Cốc",
        engine: Engine::Chromium,
        paths: &[
            r"%LOCALAPPDATA%\CocCoc\Browser\Application\browser.exe",
            r"C:\Program Files\CocCoc\Browser\Application\browser.exe",
            r"C:\Program Files (x86)\CocCoc\Browser\Application\browser.exe",
            "/Applications/Cốc Cốc.app/Contents/MacOS/Cốc Cốc",
        ],
    },
    KnownBrowser {
        id: "yandex",
        name: "Yandex Browser",
        engine: Engine::Chromium,
        paths: &[
            r"%LOCALAPPDATA%\Yandex\YandexBrowser\Application\browser.exe",
            r"C:\Program Files (x86)\Yandex\YandexBrowser\Application\browser.exe",
            "/Applications/Yandex.app/Contents/MacOS/Yandex",
            "/usr/bin/yandex-browser",
        ],
    },
    KnownBrowser {
        id: "whale",
        name: "Naver Whale",
        engine: Engine::Chromium,
        paths: &[
            r"%LOCALAPPDATA%\Naver\Naver Whale\Application\whale.exe",
            r"C:\Program Files\Naver\Naver Whale\Application\whale.exe",
            "/Applications/Naver Whale.app/Contents/MacOS/Naver Whale",
        ],
    },
    KnownBrowser {
        id: "arc",
        name: "Arc",
        engine: Engine::Chromium,
        paths: &[
            r"%LOCALAPPDATA%\Programs\Arc\Arc.exe",
            "/Applications/Arc.app/Contents/MacOS/Arc",
        ],
    },
    KnownBrowser {
        id: "thorium",
        name: "Thorium",
        engine: Engine::Chromium,
        paths: &[
            r"%LOCALAPPDATA%\Thorium\Application\thorium.exe",
            "/usr/bin/thorium",
            "/usr/bin/thorium-browser",
        ],
    },
    KnownBrowser {
        id: "iridium",
        name: "Iridium",
        engine: Engine::Chromium,
        paths: &[
            r"C:\Program Files\Iridium\Application\iridium.exe",
            "/usr/bin/iridium-browser",
        ],
    },
    KnownBrowser {
        id: "ungoogled-chromium",
        name: "ungoogled-chromium",
        engine: Engine::Chromium,
        paths: &[
            "/usr/bin/ungoogled-chromium",
            "/usr/local/bin/ungoogled-chromium",
        ],
    },
    KnownBrowser {
        id: "falkon",
        name: "Falkon",
        engine: Engine::Chromium,
        paths: &[
            r"C:\Program Files\Falkon\falkon.exe",
            "/Applications/Falkon.app/Contents/MacOS/Falkon",
            "/usr/bin/falkon",
        ],
    },
    KnownBrowser {
        id: "qutebrowser",
        name: "qutebrowser",
        engine: Engine::Chromium,
        paths: &["/usr/bin/qutebrowser"],
    },
    KnownBrowser {
        id: "360-chrome",
        name: "360 Chrome",
        engine: Engine::Chromium,
        paths: &[
            r"C:\Program Files (x86)\360\360Chrome\Chrome\Application\360chrome.exe",
            r"%LOCALAPPDATA%\360Chrome\Chrome\Application\360chrome.exe",
        ],
    },
    KnownBrowser {
        id: "qq-browser",
        name: "QQ Browser",
        engine: Engine::Chromium,
        paths: &[
            r"C:\Program Files (x86)\Tencent\QQBrowser\QQBrowser.exe",
            "%LOCALAPPDATA%\\Tencent\\QQBrowser\\QQBrowser.exe",
        ],
    },
    // --- Firefox family (WebDriver BiDi) -----------------------------------
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
    KnownBrowser {
        id: "firefox-developer",
        name: "Firefox Developer Edition",
        engine: Engine::Firefox,
        paths: &[
            r"C:\Program Files\Firefox Developer Edition\firefox.exe",
            "/Applications/Firefox Developer Edition.app/Contents/MacOS/firefox",
            "/usr/bin/firefox-developer-edition",
        ],
    },
    KnownBrowser {
        id: "librewolf",
        name: "LibreWolf",
        engine: Engine::Firefox,
        paths: &[
            r"C:\Program Files\LibreWolf\librewolf.exe",
            "/Applications/LibreWolf.app/Contents/MacOS/librewolf",
            "/usr/bin/librewolf",
            "/usr/lib/librewolf/librewolf",
        ],
    },
    KnownBrowser {
        id: "waterfox",
        name: "Waterfox",
        engine: Engine::Firefox,
        paths: &[
            r"C:\Program Files\Waterfox\waterfox.exe",
            "/Applications/Waterfox.app/Contents/MacOS/waterfox",
            "/usr/bin/waterfox",
        ],
    },
    KnownBrowser {
        id: "floorp",
        name: "Floorp",
        engine: Engine::Firefox,
        paths: &[
            r"%LOCALAPPDATA%\Floorp\floorp.exe",
            r"C:\Program Files\Floorp\floorp.exe",
            "/Applications/Floorp.app/Contents/MacOS/floorp",
            "/usr/bin/floorp",
        ],
    },
    KnownBrowser {
        id: "mullvad-browser",
        name: "Mullvad Browser",
        engine: Engine::Firefox,
        paths: &[
            "/Applications/Mullvad Browser.app/Contents/MacOS/Mullvad Browser",
            "/usr/bin/mullvad-browser",
        ],
    },
    KnownBrowser {
        id: "tor-browser",
        name: "Tor Browser",
        engine: Engine::Firefox,
        paths: &[
            "/Applications/Tor Browser.app/Contents/MacOS/Tor Browser",
            "/usr/bin/torbrowser-launcher",
            r"%USERPROFILE%\Desktop\Tor Browser\Browser\firefox.exe",
        ],
    },
    KnownBrowser {
        id: "icecat",
        name: "GNU IceCat",
        engine: Engine::Firefox,
        paths: &[r"C:\Program Files\GNU\IceCat\icecat.exe", "/usr/bin/icecat"],
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
    ("opera_gx", "opera-gx"),
    ("coccoc", "coc-coc"),
    ("cốc-cốc", "coc-coc"),
    ("yandex-browser", "yandex"),
    ("naver-whale", "whale"),
    ("mozilla-firefox", "firefox"),
    ("librewolf-browser", "librewolf"),
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

/// Every known browser whose executable is present on this machine.
pub fn installed() -> Vec<(&'static KnownBrowser, PathBuf)> {
    KNOWN
        .iter()
        .filter_map(|browser| first_existing(browser).map(|path| (browser, path)))
        .collect()
}

/// The first existing candidate path for `browser`.
pub fn first_existing(browser: &KnownBrowser) -> Option<PathBuf> {
    browser
        .paths
        .iter()
        .map(|candidate| expand(candidate))
        .find(|path| path.is_file())
}

/// Resolve the executable to launch.
///
/// Precedence: an explicit `requested` value (a file path, or a known id such as
/// `brave`), then `XCELERATE_BROWSER`, then the first installed browser of
/// `engine`. Returns `None` when nothing suitable is installed.
pub fn resolve(requested: Option<&str>, engine: Engine) -> Option<PathBuf> {
    if let Some(value) = requested.map(str::trim).filter(|value| !value.is_empty()) {
        let path = expand(value);
        if path.is_file() {
            return Some(path);
        }
        if let Some(browser) = lookup(value) {
            // A known id names exactly one engine. Asking for a Firefox id on the
            // Chromium path (or vice versa) must not hand back the wrong
            // executable - it should fail so the caller can report it.
            if browser.engine != engine {
                return None;
            }
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

/// Every known id for `engine` (e.g. so an error can list only the ids that
/// actually work for the backend the caller is using).
pub fn ids_for(engine: Engine) -> impl Iterator<Item = &'static str> {
    KNOWN
        .iter()
        .filter(move |browser| browser.engine == engine)
        .map(|browser| browser.id)
}

/// The engine a known id (or alias) belongs to, if `id` names a known browser.
/// Lets a caller tell "you asked for Firefox" from "nothing is installed".
pub fn engine_of(id: &str) -> Option<Engine> {
    lookup(id).map(|browser| browser.engine)
}

/// Expand `%NAME%` environment tokens in a candidate path.
///
/// Windows browsers install per-user under `%LOCALAPPDATA%` and friends, and a
/// `const` table cannot hold an expanded path. Unknown tokens are left as-is so a
/// candidate simply fails `is_file` rather than matching something wrong.
fn expand(path: &str) -> PathBuf {
    let mut out = String::with_capacity(path.len());
    let mut rest = path;
    while let Some(start) = rest.find('%') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        match after.find('%') {
            Some(end) => {
                let name = &after[..end];
                match std::env::var(name) {
                    Ok(value) => out.push_str(&value),
                    Err(_) => {
                        out.push('%');
                        out.push_str(name);
                        out.push('%');
                    }
                }
                rest = &after[end + 1..];
            }
            None => {
                out.push('%');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    PathBuf::from(out)
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
        assert_eq!(lookup("coccoc").unwrap().id, "coc-coc");
        assert_eq!(lookup("opera_gx").unwrap().id, "opera-gx");
        assert!(lookup("netscape").is_none());
    }

    #[test]
    fn every_engine_has_at_least_one_browser() {
        assert!(KNOWN.iter().any(|b| b.engine == Engine::Chromium));
        assert!(KNOWN.iter().any(|b| b.engine == Engine::Firefox));
    }

    #[test]
    fn ids_are_unique() {
        let mut ids: Vec<_> = ids().collect();
        let count = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), count, "duplicate browser id");
    }

    #[test]
    fn expand_leaves_plain_and_unknown_paths_alone() {
        assert_eq!(
            expand("/usr/bin/firefox").to_string_lossy(),
            "/usr/bin/firefox"
        );
        assert_eq!(expand("%NOSUCHVAR%/x").to_string_lossy(), "%NOSUCHVAR%/x");
    }

    #[test]
    fn an_unknown_but_real_path_is_returned_as_asked() {
        // An explicit path that does not exist is handed back unchanged so the
        // launch error names it, rather than silently falling back.
        let resolved = resolve(Some("/definitely/not/a/browser"), Engine::Chromium);
        assert_eq!(resolved, Some(PathBuf::from("/definitely/not/a/browser")));
    }
}
