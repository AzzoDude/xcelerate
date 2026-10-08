//! The XCL runtime state and outcomes.
//!
//! [`Context`] is the single mutable environment shared across a run: variables,
//! the resolved permissions, the budget ledger, and builtins. It is deliberately
//! transport-agnostic — the browser/plugin glue lives in the engine and the
//! session, not here.

use std::collections::HashMap;

use super::security::{Permissions, Source};

/// Parses a `wait` argument into milliseconds. The language has no duration
/// literal (values are bool / number / string / `$var` / `{builtin}`), so a bare
/// integer is milliseconds and anything else returns `None` - the caller then
/// treats it as a selector.
pub(crate) fn parse_duration_ms(arg: &str) -> Option<u64> {
    arg.trim().parse().ok()
}

/// Sleeps for `count * unit_ms` milliseconds, where `count` is a bare number.
/// Used by the unit-suffixed verbs (`wait-sec 2`, `wait-min 1`, ...), so durations
/// never need a `2s`-style literal.
pub(crate) async fn wait_scaled(arg: &str, unit_ms: u64) -> Result<String, String> {
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
pub(crate) fn parse_ms(arg: &str) -> Result<i64, String> {
    arg.trim()
        .parse()
        .map_err(|_| format!("expected a number of milliseconds, got {arg:?}"))
}

/// Normalizes a navigation target so a bare domain works: `facebook.com` becomes
/// `https://facebook.com`. Targets that already carry a scheme (`https://…`,
/// `about:blank`, `file://…`, `data:…`) are left untouched.
pub(crate) fn normalize_url(raw: &str) -> String {
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
}

impl Context {
    pub fn new(permissions: Permissions, source: Source, base_url: impl Into<String>) -> Self {
        Self {
            vars: HashMap::new(),
            builtins: Builtins::new(base_url),
            permissions,
            source,
            steps_executed: 0,
        }
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
        }
    }
}

/// A random millisecond value in `[min, max]` for the `wait-random` verb. Shared
/// by the script runner and the interactive session via a process-wide PRNG.
pub(crate) fn random_ms(min: i64, max: i64) -> i64 {
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
