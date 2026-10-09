//! XCL security: boundedness constants and the default-deny guardrails.
//!
//! XCL is *executable input* from an untrusted source (an AI agent or a
//! downloaded `.xcl` file). This module centralises the hard limits and the
//! default-deny opt-in flags that keep a script from escaping the same trust
//! boundary the plugin/capability model already enforces.

use std::collections::HashSet;

/// Hard cap on total executed steps in a single run (defeats unbounded loops).
pub const MAX_STEPS: u32 = 10_000;

/// Default cap on a single `repeat`/`retry` iteration count.
pub const MAX_ITERATIONS: u32 = 10_000;

/// Default maximum parameters a `func` may declare.
pub const MAX_FUNC_PARAMS: u32 = 64;

/// Default maximum `func` definitions / imports per program.
pub const MAX_FUNCS: usize = 4_096;

/// Maximum characters accepted from a single command's output (truncated after).
pub const MAX_OUTPUT_CHARS: usize = 64 * 1024;

/// The full set of runtime/parse limits.
///
/// These are a **safety floor for untrusted input, not a fixed ceiling**: every
/// one can be raised by the invoking human through the matching `--max-*` flag.
/// A compiled-in default is what keeps a downloaded `.xcl` bounded; a trusted
/// author who genuinely needs more raises the number explicitly.
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    /// Total executed steps before the run halts.
    pub max_steps: u32,
    /// Largest allowed `repeat`/`retry` count.
    pub max_iterations: u32,
    /// Largest allowed `func` parameter list.
    pub max_func_params: u32,
    /// Largest allowed number of `func`s / `import`s.
    pub max_funcs: usize,
    /// Characters kept from a single command's output.
    pub max_output_chars: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_steps: MAX_STEPS,
            max_iterations: MAX_ITERATIONS,
            max_func_params: MAX_FUNC_PARAMS,
            max_funcs: MAX_FUNCS,
            max_output_chars: MAX_OUTPUT_CHARS,
        }
    }
}

/// Provenance tag recorded on every audit event so the source of an executed
/// step is always attributable (indicator-removal resistance).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// An interactive human driving the session REPL.
    Repl,
    /// An AI agent emitting commands.
    Ai,
    /// A `.xcl` script file.
    File,
    /// A one-shot CLI invocation.
    Cli,
}

impl Source {
    pub fn as_str(self) -> &'static str {
        match self {
            Source::Repl => "repl",
            Source::Ai => "ai",
            Source::File => "file",
            Source::Cli => "cli",
        }
    }
}

/// Runtime opt-in flags. Everything is denied by default; the invoking human/
/// process must explicitly grant each capability. The AI can never self-grant.
#[derive(Debug, Clone, Default)]
pub struct Permissions {
    /// Allow `eval <js>` (JavaScript). Highest risk.
    pub allow_eval: bool,
    /// Allow `request` / browserless HTTP (SSRF vector).
    pub allow_http: bool,
    /// Plugins that may be `import`ed/`run`. Empty = deny all plugin loading.
    pub allow_plugins: Vec<String>,
    /// Whether private/loopback/link-local/metadata hosts are reachable in
    /// browserless mode. Default `false` (deny).
    pub allow_private: bool,
    /// Native windows an AI/`.xcl` run may *drive* (click, type, keys), matched
    /// by window-title or process-name glob. Empty = deny all native actions;
    /// read-only discovery (the plugin's `windows` op) stays allowed. Driving a
    /// native app is the most powerful capability a script can hold, so it is
    /// opt-in per target exactly like `--allow-domain` is for navigation.
    pub allow_apps: Vec<String>,
}

impl Permissions {
    /// Whether `plugin` is permitted to load/run.
    pub fn plugin_allowed(&self, name: &str) -> bool {
        self.allow_plugins
            .iter()
            .any(|entry| glob_allows(entry, name))
    }

    /// Whether a native window (`title` and/or process name) may be driven.
    pub fn app_allowed(&self, window: &str) -> bool {
        self.allow_apps
            .iter()
            .any(|entry| glob_allows(entry, window))
    }

    /// Compute the *narrower* permission set of this and `other`: a `.xcl` file
    /// can never widen the permissions of the session that runs it.
    pub fn intersect(&self, other: &Permissions) -> Permissions {
        Permissions {
            allow_eval: self.allow_eval && other.allow_eval,
            allow_http: self.allow_http && other.allow_http,
            allow_plugins: if self.allow_plugins.is_empty() || other.allow_plugins.is_empty() {
                Vec::new()
            } else {
                self.allow_plugins
                    .iter()
                    .filter(|entry| other.allow_plugins.iter().any(|o| o == *entry))
                    .cloned()
                    .collect()
            },
            allow_private: self.allow_private && other.allow_private,
            // English: keep only entries both sides permit (mutual glob match).
            // An empty list on either side denies everything, so a file cannot
            // widen native access it was not granted.
            allow_apps: {
                let mut out: Vec<String> = Vec::new();
                for entry in &self.allow_apps {
                    if other.app_allowed(entry) && !out.contains(entry) {
                        out.push(entry.clone());
                    }
                }
                for entry in &other.allow_apps {
                    if self.app_allowed(entry) && !out.contains(entry) {
                        out.push(entry.clone());
                    }
                }
                out
            },
        }
    }
}

/// Key chords whose effect escapes the app into the desktop/session (open a
/// shell, lock the screen, force-quit). Native `key` is refused for these unless
/// the invoking human opts in.
pub fn is_dangerous_key(name: &str) -> bool {
    let normalized: String = name.to_ascii_lowercase().replace(' ', "");
    matches!(
        normalized.as_str(),
        "win+r"
            | "meta+r"
            | "win+x"
            | "meta+x"
            | "win+l"
            | "meta+l"
            | "win+e"
            | "meta+e"
            | "alt+f4"
            | "ctrl+shift+esc"
            | "ctrl+alt+delete"
    )
}

/// A simple `*`/`?` glob matcher for plugin allow entries.
fn glob_allows(pattern: &str, name: &str) -> bool {
    let pattern = pattern.to_ascii_lowercase();
    let name = name.to_ascii_lowercase();
    if pattern == "*" {
        return true;
    }
    // Translate glob to a matched comparison manually (no regex dependency).
    let p: Vec<char> = pattern.chars().collect();
    let n: Vec<char> = name.chars().collect();
    let (mut pi, mut ni) = (0usize, 0usize);
    let (mut star, mut star_ni) = (None, 0usize);
    while ni < n.len() {
        // A `*` in the pattern is *always* a wildcard - check it before the
        // literal comparison, so a `*` in the name (e.g. a modified window
        // title like `*Untitled - Notepad`) is not consumed by it.
        if pi < p.len() && p[pi] == '*' {
            star = Some(pi);
            star_ni = ni;
            pi += 1;
        } else if pi < p.len() && (p[pi] == '?' || p[pi] == n[ni]) {
            pi += 1;
            ni += 1;
        } else if let Some(sp) = star {
            pi = sp + 1;
            star_ni += 1;
            ni = star_ni;
        } else {
            return false;
        }
    }
    while pi < p.len() && p[pi] == '*' {
        pi += 1;
    }
    pi == p.len()
}

/// Normalizes a URL authority host so equivalent hosts compare equal and the
/// numeric IP notations HTTP clients accept are canonicalized.
///
/// Defeats the string-only SSRF/policy bypass family: a surrounding `[` `]`
/// (IPv6 literal), one or more trailing dots (FQDN root), and the inet_aton IPv4
/// notations (dotted, bare integer, `0x` hex, leading-zero octal) that a client
/// silently folds to `127.0.0.1`. Returns `None` for an empty host.
pub fn normalize_host(host: &str) -> Option<String> {
    let host = host.trim().trim_end_matches('.');
    if host.is_empty() {
        return None;
    }
    // `[::1]` -> `::1`
    if let Some(inner) = host.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
        return Some(inner.to_ascii_lowercase());
    }
    if let Some(ip) = parse_ipv4_loose(host) {
        return Some(ip.to_string());
    }
    Some(host.to_ascii_lowercase())
}

/// Parses the inet_aton IPv4 notations (used by `curl`, `ping`, `getaddrinfo`):
/// `127.0.0.1`, `127.1`, `2130706433`, `0x7f000001`, `0177.0.0.1`. Any other shape
/// (including a real hostname) returns `None`.
fn parse_ipv4_loose(host: &str) -> Option<std::net::Ipv4Addr> {
    if host.contains(|c: char| !c.is_ascii_alphanumeric() && c != '.' && c != 'x' && c != 'X') {
        return None;
    }
    let parts: Vec<&str> = host.split('.').collect();
    if parts.is_empty() || parts.len() > 4 {
        return None;
    }
    let mut nums: Vec<u32> = Vec::with_capacity(parts.len());
    for part in &parts {
        let value = if let Some(hex) = part.strip_prefix("0x").or_else(|| part.strip_prefix("0X")) {
            u32::from_str_radix(hex, 16).ok()?
        } else if part.len() > 1 && part.starts_with('0') {
            u32::from_str_radix(&part[1..], 8).ok()?
        } else {
            part.parse::<u32>().ok()?
        };
        nums.push(value);
    }
    // Each part's permitted range shrinks as the notation gets shorter.
    let max: Vec<u64> = match nums.len() {
        1 => vec![u32::MAX as u64],
        2 => vec![0xff, 0xff_ffff],
        3 => vec![0xff, 0xff, 0xffff],
        _ => vec![0xff, 0xff, 0xff, 0xff],
    };
    for (index, value) in nums.iter().enumerate() {
        if *value as u64 > max[index] {
            return None;
        }
    }
    let value = match nums.len() {
        1 => nums[0],
        2 => (nums[0] << 24) | nums[1],
        3 => (nums[0] << 24) | (nums[1] << 16) | nums[2],
        _ => (nums[0] << 24) | (nums[1] << 16) | (nums[2] << 8) | nums[3],
    };
    Some(std::net::Ipv4Addr::from(value))
}

/// Whether an IP is loopback/private/link-local/unique-local/unspecified — the
/// ranges the SSRF guard denies by default.
pub fn is_private_ip(ip: std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(v4) => {
            v4.is_private()
                || v4.is_loopback()
                || v4.is_link_local()
                || v4.is_unspecified()
                || v4.octets()[0] == 169 && v4.octets()[1] == 254
        }
        std::net::IpAddr::V6(v6) => {
            v6.is_loopback() || v6.is_unique_local() || v6.is_unicast_link_local()
        }
    }
}

/// The private/loopback/link-local/metadata ranges that browserless HTTP denies
/// by default (the SSRF guard). Returns `true` when `host` is such a range and
/// should be blocked unless `allow_private`.
///
/// The host is *normalized* first, so the alternate IPv4 notations and trailing-
/// dot/bracketed forms cannot slip a loopback request past a string-only check.
pub fn is_private_host(host: &str) -> bool {
    let Some(host) = normalize_host(host) else {
        return false;
    };
    if host == "localhost" || host.ends_with(".localhost") || host == "localhost.localdomain" {
        return true;
    }
    if host == "0.0.0.0" {
        return true;
    }
    if host.ends_with(".local") || host.ends_with(".internal") || host.ends_with(".lan") {
        return true;
    }
    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        return is_private_ip(ip);
    }
    // Cloud metadata hostnames commonly targeted by SSRF.
    matches!(
        host.as_str(),
        "metadata.google.internal"
            | "169.254.169.254"
            | "instance-data"
            | "metadata.azure.internal"
    )
}

/// Metadata/private IP literals (kept as a set for quick membership checks).
pub fn private_metadata_hosts() -> &'static HashSet<&'static str> {
    static HOSTS: std::sync::OnceLock<HashSet<&'static str>> = std::sync::OnceLock::new();
    HOSTS.get_or_init(|| {
        [
            "169.254.169.254",
            "metadata.google.internal",
            "metadata.azure.internal",
            "instance-data",
        ]
        .iter()
        .copied()
        .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_allows_star_and_literal() {
        assert!(glob_allows("*", "any.plugin"));
        assert!(glob_allows("acme.*", "acme.kv"));
        assert!(!glob_allows("acme.*", "other.kv"));
    }

    #[test]
    fn glob_star_wildcard_matches_a_literal_star_in_the_name() {
        // A window title can begin with `*` (a modified document); the pattern's
        // `*` is a wildcard and must not be consumed by that literal `*`.
        assert!(glob_allows("*Notepad*", "*Untitled - Notepad"));
        assert!(glob_allows("*Notepad", "*Untitled - Notepad"));
        assert!(!glob_allows("Notepad*", "*Untitled - Notepad"));
    }

    #[test]
    fn private_host_detection() {
        assert!(is_private_host("localhost"));
        assert!(is_private_host("127.0.0.1"));
        assert!(is_private_host("192.168.1.1"));
        assert!(is_private_host("169.254.169.254"));
        assert!(is_private_host("metadata.google.internal"));
        assert!(!is_private_host("example.com"));
    }

    #[test]
    fn ssrf_notation_bypasses_are_normalized() {
        // Every one of these reaches 127.0.0.1 but evaded the old string check.
        for host in [
            "127.0.0.1",
            "127.1",
            "2130706433",
            "0x7f000001",
            "0177.0.0.1",
            "localhost.",
            "[::1]",
            "0.0.0.0",
        ] {
            assert!(is_private_host(host), "{host} should be private");
        }
        assert!(!is_private_host("example.com"));
        assert!(!is_private_host("8.8.8.8"));
    }

    #[test]
    fn normalize_host_canonicalizes() {
        assert_eq!(
            normalize_host("Example.COM.").as_deref(),
            Some("example.com")
        );
        assert_eq!(normalize_host("[::1]").as_deref(), Some("::1"));
        assert_eq!(normalize_host("2130706433").as_deref(), Some("127.0.0.1"));
        assert_eq!(
            normalize_host("example.com").as_deref(),
            Some("example.com")
        );
    }

    #[test]
    fn permissions_intersect_narrows() {
        let a = Permissions {
            allow_eval: true,
            allow_http: true,
            allow_plugins: vec!["acme.*".into()],
            allow_private: false,
            allow_apps: vec!["Steam*".into()],
        };
        let b = Permissions {
            allow_eval: false,
            allow_http: true,
            allow_plugins: vec!["acme.kv".into()],
            allow_private: false,
            allow_apps: Vec::new(),
        };
        let n = a.intersect(&b);
        assert!(!n.allow_eval);
        assert!(n.allow_http);
        assert!(n.allow_plugins.is_empty());
        // An empty side denies all native apps, so the file cannot widen access.
        assert!(n.allow_apps.is_empty());
        assert!(!n.app_allowed("Steam"));
    }

    #[test]
    fn native_app_allowlist_is_default_deny() {
        let none = Permissions::default();
        assert!(!none.app_allowed("Steam"));
        let some = Permissions {
            allow_apps: vec!["Steam*".into()],
            ..Permissions::default()
        };
        assert!(some.app_allowed("Steam"));
        assert!(!some.app_allowed("Calculator"));
    }

    #[test]
    fn dangerous_keys_are_recognized() {
        assert!(is_dangerous_key("Win+R"));
        assert!(is_dangerous_key("ctrl+shift+esc"));
        assert!(is_dangerous_key("Alt+F4"));
        assert!(!is_dangerous_key("ctrl+a"));
        assert!(!is_dangerous_key("next"));
    }
}
