//! The XCL runtime state and outcomes.
//!
//! [`Context`] is the single mutable environment shared across a run: variables,
//! the resolved permissions, the budget ledger, and builtins. It is deliberately
//! transport-agnostic — the browser/plugin glue lives in the engine and the
//! session, not here.

use std::collections::HashMap;

use super::security::{Permissions, Source};

/// Parses a `wait` argument into milliseconds. Accepts an explicit unit (`2s`,
/// `500ms`) or a bare millisecond count; returns `None` when the argument is not
/// a duration, so the caller can treat it as a selector.
pub(crate) fn parse_duration_ms(arg: &str) -> Option<u64> {
    let arg = arg.trim();
    if let Some(value) = arg.strip_suffix("ms") {
        return value.trim().parse().ok();
    }
    if let Some(value) = arg.strip_suffix('s') {
        return value
            .trim()
            .parse::<u64>()
            .ok()
            .map(|seconds| seconds.saturating_mul(1000));
    }
    arg.parse().ok()
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

#[cfg(test)]
mod tests {
    use super::parse_duration_ms;

    #[test]
    fn parses_wait_durations() {
        assert_eq!(parse_duration_ms("2s"), Some(2000));
        assert_eq!(parse_duration_ms("500ms"), Some(500));
        assert_eq!(parse_duration_ms("1500"), Some(1500));
        assert_eq!(parse_duration_ms("#submit"), None);
        assert_eq!(parse_duration_ms(""), None);
    }
}
