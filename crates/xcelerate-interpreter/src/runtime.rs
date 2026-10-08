//! The XCL runtime state and outcomes.
//!
//! [`Context`] is the single mutable environment shared across a run: variables,
//! the resolved permissions, the budget ledger, and builtins. It is deliberately
//! transport-agnostic — the browser/plugin glue lives in the engine and the
//! session, not here.

use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};

use super::security::{Permissions, Source};

/// Parses a `wait` argument into milliseconds. The language has no duration
/// literal (values are bool / number / string / `$var` / `{builtin}`), so a bare
/// integer is milliseconds and anything else returns `None` - the caller then
/// treats it as a selector.
pub fn parse_duration_ms(arg: &str) -> Option<u64> {
    arg.trim().parse().ok()
}

/// Sleeps for `count * unit_ms` milliseconds, where `count` is a bare number.
/// Used by the unit-suffixed verbs (`wait-sec 2`, `wait-min 1`, ...), so durations
/// never need a `2s`-style literal.
pub async fn wait_scaled(arg: &str, unit_ms: u64) -> Result<String, String> {
    let count: u64 = arg
        .trim()
        .parse()
        .map_err(|_| format!("expected a number, got {arg:?}"))?;
    let ms = count.saturating_mul(unit_ms);
    tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
    Ok(format!("waited {ms}ms"))
}

/// Hard runtime limits (defense against hangs).
#[derive(Debug, Clone)]
pub struct RuntimeLimits {
    pub max_steps: u32,
    pub max_output_chars: usize,
}

impl Default for RuntimeLimits {
    fn default() -> Self {
        Self {
            max_steps: super::security::MAX_STEPS,
            max_output_chars: super::security::MAX_OUTPUT_CHARS,
        }
    }
}

/// The result of executing one step.
#[derive(Debug, Clone, Default)]
pub struct Outcome {
    /// Whether the step succeeded (no error, no failed assertion).
    pub ok: bool,
    /// A short, one-line result the caller prints/returns (the exit line IS the
    /// result, keeping AI token cost low).
    pub message: String,
    /// The raw value produced by the step (a plugin result, a response body),
    /// when one exists.
    pub value: Option<String>,
    /// Whether the run should terminate after this step.
    pub should_quit: bool,
}

impl Outcome {
    pub fn ok(message: impl Into<String>) -> Self {
        Self {
            ok: true,
            message: message.into(),
            value: None,
            should_quit: false,
        }
    }

    pub fn ok_value(message: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            ok: true,
            message: message.into(),
            value: Some(value.into()),
            should_quit: false,
        }
    }

    pub fn fail(message: impl Into<String>) -> Self {
        Self {
            ok: false,
            message: message.into(),
            value: None,
            should_quit: false,
        }
    }

    pub fn quit(message: impl Into<String>) -> Self {
        Self {
            ok: true,
            message: message.into(),
            value: None,
            should_quit: true,
        }
    }

    /// A successful, terminal outcome that ends the run after this step. Used by
    /// `done`, so marking a task complete also lets the driver stop the browser.
    pub fn done(message: impl Into<String>) -> Self {
        Self {
            ok: true,
            message: message.into(),
            value: None,
            should_quit: true,
        }
    }
}

/// Builtin value resolution: `BASE_URL`, `TIMESTAMP`, `UUID`.
#[derive(Debug, Clone)]
pub struct Builtins {
    /// Frozen program-start timestamp (deterministic within a run).
    pub timestamp: String,
    /// Base URL (resolved from CLI `--base-url` or the current page).
    pub base_url: String,
    /// Monotonic unique counter for `{UUID}` (ensures uniqueness per reference).
    pub uuid_counter: u64,
}

/// A tiny SplitMix64 PRNG backing the randomized `wait` verb. Deliberately not
/// cryptographic: it exists only to vary timing.
#[derive(Debug, Clone)]
pub(crate) struct Rng {
    state: u64,
}

impl Rng {
    pub(crate) fn from_clock() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9_7F4A_7C15);
        Self { state: seed | 1 }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A value in `[0, 1)`.
    fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    /// An integer in the inclusive, order-independent range `[min, max]`.
    pub(crate) fn range(&mut self, min: i64, max: i64) -> i64 {
        let (lo, hi) = if min <= max { (min, max) } else { (max, min) };
        if lo == hi {
            return lo;
        }
        let span = (hi - lo) as f64 + 1.0;
        let offset = (self.unit() * span).floor() as i64;
        lo + offset.min(hi - lo)
    }
}

impl Builtins {
    pub fn new(base_url: impl Into<String>) -> Self {
        let timestamp = {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            now.to_string()
        };
        Self {
            timestamp,
            base_url: base_url.into(),
            uuid_counter: 0,
        }
    }

    /// Resolves a builtin name to a value, returning `None` for unknown names.
    pub fn resolve(&mut self, name: &str) -> Option<String> {
        match name {
            "BASE_URL" => Some(self.base_url.clone()),
            "TIMESTAMP" | "NOW" => Some(self.timestamp.clone()),
            "UUID" => {
                self.uuid_counter += 1;
                Some(format!(
                    "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
                    self.uuid_counter, 0u32, 0u32, 0u32, self.uuid_counter
                ))
            }
            _ => None,
        }
    }
}

/// Parses a `wait` bound as a whole number of milliseconds.
pub fn parse_ms(arg: &str) -> Result<i64, String> {
    arg.trim()
        .parse()
        .map_err(|_| format!("expected a number of milliseconds, got {arg:?}"))
}

/// Normalizes a navigation target so a bare domain works: `facebook.com` becomes
/// `https://facebook.com`. Targets that already carry a scheme (`https://…`,
/// `about:blank`, `file://…`, `data:…`) are left untouched.
pub fn normalize_url(raw: &str) -> String {
    let url = raw.trim();
    if url.is_empty() || has_scheme(url) {
        return url.to_string();
    }
    format!("https://{url}")
}

/// Whether `url` starts with a URL scheme (`scheme://…`), or a known opaque
/// scheme such as `about:` / `data:`.
fn has_scheme(url: &str) -> bool {
    let Some(colon) = url.find(':') else {
        return false;
    };
    let scheme = &url[..colon];
    let valid = scheme
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic())
        && scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'));
    if !valid {
        return false;
    }
    // `host:port` (e.g. `facebook.com:8080`) is not a scheme: no `//` and not a
    // known opaque scheme, so it is treated as a bare host and gets `https://`.
    url[colon + 1..].starts_with("//")
        || matches!(
            scheme,
            "about" | "data" | "file" | "blob" | "chrome" | "view-source"
        )
}

/// The execution environment for a single XCL run.
#[derive(Debug, Clone)]
pub struct Context {
    /// User variables (`let`/`param`/`set`).
    pub vars: HashMap<String, String>,
    /// Builtins (namespaced, resolved lazily).
    pub builtins: Builtins,
    /// The effective permissions (already intersected with the session's).
    pub permissions: Permissions,
    /// The provenance source for audit tagging.
    pub source: Source,
    /// Budget ledger: executed step count.
    pub steps_executed: u32,
    /// The workspace root every file-writing/​reading verb is confined to. A
    /// script may not name an absolute path or climb out of this directory, so
    /// `download`/`capture`/`shot`/`upload` can only ever touch files under it.
    pub root: PathBuf,
}

impl Context {
    pub fn new(permissions: Permissions, source: Source, base_url: impl Into<String>) -> Self {
        Self {
            vars: HashMap::new(),
            builtins: Builtins::new(base_url),
            permissions,
            source,
            steps_executed: 0,
            root: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        }
    }

    /// Sets the workspace root (the directory relative paths resolve against and
    /// the boundary no path may escape). Used by the CLI `--output-dir`.
    pub fn with_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.root = root.into();
        self
    }

    /// Resolves a script-supplied path against the workspace root.
    ///
    /// Rejects empty paths, absolute paths (`/etc/x`, `C:\x`), and any `..`
    /// component that would climb above the root. When the destination already
    /// exists the resolved parent is canonicalized and re-checked, so a symlink
    /// inside the root cannot point back out of it. Every file verb routes through
    /// here; the resulting path is what the run actually writes to or reads from.
    pub fn resolve_path(&self, raw: &str) -> Result<PathBuf, String> {
        resolve_in_root(&self.root, raw)
    }

    /// Interpolates an [`Arg`] against the current variables/builtins.
    pub fn resolve(&mut self, arg: &super::ast::Arg) -> Result<String, String> {
        match arg {
            super::ast::Arg::Literal(s) => Ok(s.clone()),
            super::ast::Arg::Var(name) => self
                .vars
                .get(name)
                .cloned()
                .ok_or_else(|| format!("undefined variable `${name}`")),
            super::ast::Arg::Builtin(name) => self
                .builtins
                .resolve(name)
                .ok_or_else(|| format!("unknown builtin `{{{name}}}`")),
            super::ast::Arg::Template(pieces) => {
                let mut out = String::new();
                for piece in pieces {
                    out.push_str(&self.resolve(piece)?);
                }
                Ok(out)
            }
        }
    }
}

/// Resolves `raw` against `root`, keeping the result inside `root`.
///
/// Shared by [`Context::resolve_path`] and the interactive session so every
/// file verb in the language obeys the same boundary. Refuses empty paths,
/// absolute paths, `..` traversal above the root, and (on a best-effort basis)
/// symlinked directories that resolve outside it.
pub fn resolve_in_root(root: &Path, raw: &str) -> Result<PathBuf, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err("empty path".to_string());
    }

    let mut normalized = PathBuf::new();
    for component in Path::new(raw).components() {
        match component {
            Component::Normal(part) => normalized.push(part),
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err(format!("path escapes the workspace root: {raw:?}"));
                }
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(format!("absolute paths are not allowed: {raw:?}"));
            }
        }
    }
    if normalized.as_os_str().is_empty() {
        return Err("empty path".to_string());
    }

    let full = root.join(&normalized);
    // When the root and the destination's nearest existing ancestor both
    // resolve, make sure the ancestor still sits inside the root — this catches
    // a symlinked directory that would otherwise escape it.
    if let (Ok(root), Ok(existing)) = (
        std::fs::canonicalize(root),
        canonical_existing_ancestor(&full),
    ) && !existing.starts_with(&root)
    {
        return Err(format!("path escapes the workspace root: {raw:?}"));
    }
    Ok(full)
}

/// Canonicalizes the nearest ancestor of `path` that exists on disk (the parent
/// chain is walked upward because the final file may not have been created yet).
fn canonical_existing_ancestor(path: &Path) -> Result<PathBuf, std::io::Error> {
    let mut probe = path;
    loop {
        if probe.exists() {
            return std::fs::canonicalize(probe);
        }
        match probe.parent() {
            Some(parent) => probe = parent,
            None => return std::fs::canonicalize(path),
        }
    }
}

/// A random millisecond value in `[min, max]` for the `wait-random` verb. Shared
/// by the script runner and the interactive session via a process-wide PRNG.
pub fn random_ms(min: i64, max: i64) -> i64 {
    static RNG: std::sync::OnceLock<std::sync::Mutex<Rng>> = std::sync::OnceLock::new();
    let rng = RNG.get_or_init(|| std::sync::Mutex::new(Rng::from_clock()));
    rng.lock()
        .map(|mut r| r.range(min, max))
        .unwrap_or(min)
        .max(0)
}

#[cfg(test)]
mod tests {
    use super::super::security::{Permissions, Source};
    use super::{Context, normalize_url, parse_duration_ms, parse_ms, random_ms};

    #[test]
    fn parses_wait_durations() {
        // A bare integer is milliseconds; there is no `2s`/`500ms` literal.
        assert_eq!(parse_duration_ms("2000"), Some(2000));
        assert_eq!(parse_duration_ms("500"), Some(500));
        assert_eq!(parse_duration_ms("2s"), None);
        assert_eq!(parse_duration_ms("500ms"), None);
        assert_eq!(parse_duration_ms("#submit"), None);
        assert_eq!(parse_duration_ms(""), None);
    }

    #[test]
    fn parse_ms_accepts_whole_numbers_only() {
        assert_eq!(parse_ms("250").unwrap(), 250);
        assert_eq!(parse_ms(" -10 ").unwrap(), -10);
        assert!(parse_ms("2s").is_err());
        assert!(parse_ms("").is_err());
    }

    fn ctx() -> Context {
        Context::new(Permissions::default(), Source::Repl, "https://x")
    }

    #[test]
    fn resolve_path_confines_to_the_root() {
        let ctx = ctx().with_root(std::env::temp_dir());
        let root = ctx.root.clone();

        // A plain relative path lands under the root.
        assert_eq!(ctx.resolve_path("logo.png").unwrap(), root.join("logo.png"));
        // Nested paths and `.` are fine, and `..` that stays inside is folded.
        assert_eq!(
            ctx.resolve_path("a/b/../c.mp4").unwrap(),
            root.join("a").join("c.mp4")
        );
        assert_eq!(ctx.resolve_path("./x").unwrap(), root.join("x"));
    }

    #[test]
    fn resolve_path_rejects_escapes_and_absolute() {
        let ctx = ctx().with_root(std::env::temp_dir());

        // Climbing above the root is refused.
        assert!(ctx.resolve_path("../secrets.txt").is_err());
        assert!(ctx.resolve_path("a/../../x").is_err());
        // Absolute paths (POSIX and Windows-style) are refused.
        assert!(ctx.resolve_path("/etc/passwd").is_err());
        assert!(ctx.resolve_path("C:\\Windows\\x").is_err());
        // Empty and `.`-only paths are refused.
        assert!(ctx.resolve_path("").is_err());
        assert!(ctx.resolve_path(".").is_err());
    }

    #[test]
    fn random_range_stays_within_bounds() {
        for _ in 0..2_000 {
            let n = random_ms(5, 10);
            assert!((5..=10).contains(&n), "out of range: {n}");
        }
    }

    #[test]
    fn random_range_is_order_independent() {
        for _ in 0..500 {
            let n = random_ms(10, 5);
            assert!((5..=10).contains(&n), "out of range: {n}");
        }
    }

    #[test]
    fn random_ms_is_clamped_to_zero() {
        // A negative/nonsensical range can never produce a negative sleep.
        for _ in 0..100 {
            assert!(random_ms(-50, -10) >= 0);
        }
    }

    #[test]
    fn normalizes_bare_domains_to_https() {
        assert_eq!(normalize_url("facebook.com"), "https://facebook.com");
        assert_eq!(
            normalize_url("example.com/a?b=1"),
            "https://example.com/a?b=1"
        );
        assert_eq!(normalize_url("  facebook.com  "), "https://facebook.com");
        assert_eq!(
            normalize_url("facebook.com:8080"),
            "https://facebook.com:8080"
        );
    }

    #[test]
    fn keeps_targets_that_already_have_a_scheme() {
        assert_eq!(normalize_url("https://x.com"), "https://x.com");
        assert_eq!(normalize_url("http://x.com"), "http://x.com");
        assert_eq!(normalize_url("about:blank"), "about:blank");
        assert_eq!(normalize_url("file:///c:/x"), "file:///c:/x");
        assert_eq!(normalize_url("data:text/html,x"), "data:text/html,x");
    }

    #[test]
    fn empty_target_stays_empty() {
        assert_eq!(normalize_url("   "), "");
    }

    #[test]
    fn builtins_that_were_removed_are_unknown() {
        let mut ctx = ctx();
        assert!(
            ctx.resolve(&super::super::ast::Arg::Builtin("RANDOM".into()))
                .is_err()
        );
        assert!(
            ctx.resolve(&super::super::ast::Arg::Builtin("TIMESTAMP".into()))
                .is_ok()
        );
    }
}
