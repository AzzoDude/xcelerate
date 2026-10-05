//! Domain allow / deny policy (capability #3).
//!
//! A Rust-only global policy, mirroring [`crate::configure_proxy`]: hosts not on
//! the allow list (when one is set) or on the deny list are rejected before a
//! navigation is issued. Kept out of the UniFFI surface so bindings do not
//! change when the policy grows.

use std::sync::RwLock;

use crate::error::{XcelerateError, XcelerateResult};

static POLICY: RwLock<Option<(Vec<String>, Vec<String>)>> = RwLock::new(None);

/// Installs the process-wide domain policy. An empty allow list means "allow
/// all"; an entry may be a bare domain (`example.com`, matching subdomains) or a
/// glob (`*.example.com`).
pub fn configure_domain_policy(allowed: Vec<String>, prohibited: Vec<String>) {
    let mut guard = POLICY.write().unwrap_or_else(|poison| poison.into_inner());
    *guard = Some((allowed, prohibited));
}

/// Matches `text` against a simple glob `pattern` supporting `*` (any run of
/// characters, including none) and `?` (exactly one character).
///
/// Matching is ASCII case-insensitive so that callers do not have to normalise
/// hosts. This is intentionally a small, dependency-free matcher rather than a
/// full glob implementation.
pub(crate) fn glob_match(pattern: &str, text: &str) -> bool {
    let pattern: Vec<char> = pattern.to_ascii_lowercase().chars().collect();
    let text: Vec<char> = text.to_ascii_lowercase().chars().collect();

    let mut p = 0usize;
    let mut t = 0usize;
    let mut star: Option<usize> = None;
    let mut star_text = 0usize;

    while t < text.len() {
        if p < pattern.len() && (pattern[p] == '?' || pattern[p] == text[t]) {
            p += 1;
            t += 1;
        } else if p < pattern.len() && pattern[p] == '*' {
            star = Some(p);
            star_text = t;
            p += 1;
        } else if let Some(star_pos) = star {
            // Backtrack: let the last `*` swallow one more character.
            p = star_pos + 1;
            star_text += 1;
            t = star_text;
        } else {
            return false;
        }
    }

    while p < pattern.len() && pattern[p] == '*' {
        p += 1;
    }
    p == pattern.len()
}

/// Extracts the lowercased host from `url` without pulling in a URL crate.
///
/// Everything after `://` (or after a leading `//`) up to the first `/`, `?` or
/// `#` is treated as the authority; user-info before `@` and a trailing `:port`
/// are stripped. Returns `None` when no authority is present (for example
/// `about:blank` or a relative path).
pub(crate) fn host_of(url: &str) -> Option<String> {
    let trimmed = url.trim();
    let after_scheme = if let Some(index) = trimmed.find("://") {
        &trimmed[index + 3..]
    } else {
        trimmed.strip_prefix("//")?
    };

    let authority = after_scheme.split(['/', '?', '#']).next().unwrap_or("");

    let host_port = match authority.rsplit_once('@') {
        Some((_userinfo, host)) => host,
        None => authority,
    };

    let host = match host_port.rsplit_once(':') {
        Some((host, _port)) => host,
        None => host_port,
    };

    let host = host.trim();
    if host.is_empty() {
        None
    } else {
        Some(host.to_ascii_lowercase())
    }
}

/// Whether a single policy `entry` matches `host` (which must be lowercase).
///
/// An entry matches when it equals the host, when the host is a subdomain of it
/// (`host` ends with `.{entry}`), or — for entries containing `*`/`?` — when it
/// matches as a glob. A leading `*.` is ignored and the remainder treated as a
/// suffix match.
fn entry_matches(entry: &str, host: &str) -> bool {
    let entry = entry.trim().to_ascii_lowercase();
    if entry.is_empty() {
        return false;
    }

    if let Some(suffix) = entry.strip_prefix("*.") {
        return host == suffix || host.ends_with(&format!(".{suffix}"));
    }

    if entry.contains('*') || entry.contains('?') {
        return glob_match(&entry, host);
    }

    host == entry || host.ends_with(&format!(".{entry}"))
}

/// Decides whether `host` is permitted by `allowed` / `prohibited`.
///
/// `prohibited` always wins. An empty `allowed` list means "allow all";
/// otherwise the host must match at least one allowed entry. Entries are
/// compared case-insensitively.
pub(crate) fn host_allowed(host: &str, allowed: &[String], prohibited: &[String]) -> bool {
    let host = host.trim().to_ascii_lowercase();

    if prohibited.iter().any(|entry| entry_matches(entry, &host)) {
        return false;
    }

    if allowed.is_empty() {
        return true;
    }

    allowed.iter().any(|entry| entry_matches(entry, &host))
}

/// Asserts that `url` is permitted by the installed policy.
pub fn ensure_allowed(url: &str) -> XcelerateResult<()> {
    let policy = {
        let guard = POLICY.read().unwrap_or_else(|poison| poison.into_inner());
        guard.clone()
    };

    let Some((allowed, prohibited)) = policy else {
        return Ok(());
    };

    // URLs with no parseable host (e.g. `about:blank`) are always allowed.
    let Some(host) = host_of(url) else {
        return Ok(());
    };

    if host_allowed(&host, &allowed, &prohibited) {
        Ok(())
    } else {
        Err(XcelerateError::Unsupported(format!(
            "navigation blocked by domain policy: {url}"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entries(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn glob_matches_wildcards() {
        assert!(glob_match("*.example.com", "a.example.com"));
        assert!(glob_match("a*c", "abbbc"));
        assert!(glob_match("a?c", "abc"));
        assert!(glob_match("*", "anything"));
        assert!(glob_match("", ""));
        assert!(!glob_match("a?c", "ac"));
        assert!(!glob_match("a*c", "abd"));
        assert!(!glob_match("*.example.com", "example.org"));
    }

    #[test]
    fn glob_matching_is_case_insensitive() {
        assert!(glob_match("*.EXAMPLE.com", "WWW.example.COM"));
    }

    #[test]
    fn host_of_strips_scheme_userinfo_and_port() {
        assert_eq!(
            host_of("https://a.example.com:8443/x").as_deref(),
            Some("a.example.com")
        );
        assert_eq!(
            host_of("http://user:pass@Host.Example.COM:80/path?q=1").as_deref(),
            Some("host.example.com")
        );
        assert_eq!(host_of("//example.com").as_deref(), Some("example.com"));
        assert_eq!(
            host_of("wss://example.com/socket").as_deref(),
            Some("example.com")
        );
        assert_eq!(
            host_of("https://example.com?x=1").as_deref(),
            Some("example.com")
        );
    }

    #[test]
    fn host_of_rejects_hostless_urls() {
        assert_eq!(host_of("about:blank"), None);
        assert_eq!(host_of("https://"), None);
        assert_eq!(host_of("relative/path"), None);
        assert_eq!(host_of("data:text/plain,hello"), None);
    }

    #[test]
    fn empty_allow_list_permits_everything() {
        assert!(host_allowed("example.com", &[], &[]));
        assert!(host_allowed("anything.test", &[], &entries(&["nope.test"])));
    }

    #[test]
    fn prohibited_always_wins() {
        assert!(!host_allowed(
            "evil.example.com",
            &entries(&["example.com"]),
            &entries(&["evil.example.com"])
        ));
    }

    #[test]
    fn allow_list_matches_domains_and_subdomains() {
        let allowed = entries(&["example.com"]);
        assert!(host_allowed("example.com", &allowed, &[]));
        assert!(host_allowed("www.example.com", &allowed, &[]));
        assert!(!host_allowed("notexample.com", &allowed, &[]));
        assert!(!host_allowed("example.org", &allowed, &[]));
    }

    #[test]
    fn allow_list_supports_globs_and_leading_wildcards() {
        assert!(host_allowed(
            "api.example.com",
            &entries(&["*.example.com"]),
            &[]
        ));
        assert!(host_allowed(
            "example.com",
            &entries(&["*.example.com"]),
            &[]
        ));
        assert!(host_allowed(
            "cdn.assets.net",
            &entries(&["cdn.*.net"]),
            &[]
        ));
        assert!(!host_allowed("other.org", &entries(&["cdn.*.net"]), &[]));
    }

    #[test]
    fn matching_is_case_insensitive() {
        assert!(host_allowed(
            "WWW.Example.COM",
            &entries(&["example.com"]),
            &[]
        ));
        assert!(!host_allowed(
            "WWW.Example.COM",
            &[],
            &entries(&["example.com"])
        ));
    }
}
