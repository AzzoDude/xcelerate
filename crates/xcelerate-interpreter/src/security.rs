//! XCL security: boundedness constants and the default-deny guardrails.
//!
//! XCL is *executable input* from an untrusted source (an AI agent or a
//! downloaded `.xcl` file). This module centralises the hard limits and the
//! default-deny opt-in flags that keep a script from escaping the same trust
//! boundary the plugin/capability model already enforces.

use std::collections::HashSet;

/// Hard cap on total executed steps in a single run (defeats unbounded loops).
pub const MAX_STEPS: u32 = 10_000;

/// Hard cap on a single `repeat`/`retry` iteration count.
pub const MAX_ITERATIONS: u32 = 1_000;

/// Maximum parameters a `func` may declare.
pub const MAX_FUNC_PARAMS: u32 = 8;

/// Maximum `func` definitions per program.
pub const MAX_FUNCS: usize = 256;

/// Maximum characters accepted from a single command's output (truncated after).
pub const MAX_OUTPUT_CHARS: usize = 64 * 1024;

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
}

impl Permissions {
    /// Whether `plugin` is permitted to load/run.
    pub fn plugin_allowed(&self, name: &str) -> bool {
        self.allow_plugins
            .iter()
            .any(|entry| glob_allows(entry, name))
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
        }
    }
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
        if pi < p.len() && (p[pi] == '?' || p[pi] == n[ni]) {
            pi += 1;
            ni += 1;
        } else if pi < p.len() && p[pi] == '*' {
            star = Some(pi);
            star_ni = ni;
            pi += 1;
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

/// The private/loopback/link-local/metadata ranges that browserless HTTP denies
/// by default (the SSRF guard). Returns `true` when `host` is such
/// a range and should be blocked unless `allow_private`.
pub fn is_private_host(host: &str) -> bool {
    let host = host.to_ascii_lowercase();
    if host == "localhost" || host.ends_with(".localhost") || host == "localhost.localdomain" {
        return true;
    }
    if host == "::1" || host == "0.0.0.0" {
        return true;
    }
    if host.ends_with(".local") || host.ends_with(".internal") || host.ends_with(".lan") {
        return true;
    }
    // RFC1918 / link-local / metadata resolvable literals.
    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        return match ip {
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
        };
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
    fn private_host_detection() {
        assert!(is_private_host("localhost"));
        assert!(is_private_host("127.0.0.1"));
        assert!(is_private_host("192.168.1.1"));
        assert!(is_private_host("169.254.169.254"));
        assert!(is_private_host("metadata.google.internal"));
        assert!(!is_private_host("example.com"));
    }

    #[test]
    fn permissions_intersect_narrows() {
        let a = Permissions {
            allow_eval: true,
            allow_http: true,
            allow_plugins: vec!["acme.*".into()],
            allow_private: false,
        };
        let b = Permissions {
            allow_eval: false,
            allow_http: true,
            allow_plugins: vec!["acme.kv".into()],
            allow_private: false,
        };
        let n = a.intersect(&b);
        assert!(!n.allow_eval);
        assert!(n.allow_http);
        assert!(n.allow_plugins.is_empty());
    }
}
