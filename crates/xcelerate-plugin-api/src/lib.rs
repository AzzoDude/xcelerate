//! Plugin API for `xcelerate`.
//!
//! This crate is the boundary between the engine and its plugins. It defines
//! the [`Plugin`] trait, the [`Manifest`] specification, the [`Capability`] /
//! [`Tier`] trust model, the append-only [`audit`] log, and the [`PageHost`]
//! interface a plugin uses to touch a page.
//!
//! It deliberately depends only on `serde`/`serde_json` - never on the engine
//! facade - so first-party plugin crates (and, later, a sandboxed runner) can
//! implement plugins without a dependency cycle.
//!
//! # Security posture
//!
//! * **default-deny** - a plugin does nothing unless it is explicitly enabled;
//! * plugins never see a raw page or transport handle - only [`PageHost`];
//! * every privileged action is recorded in an append-only, hash-chained log;
//! * third-party manifests may not request a first-party-only [`Capability`].

use std::collections::BTreeMap;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock};

// ---------------------------------------------------------------------------
// Result type
// ---------------------------------------------------------------------------

/// Errors raised by a plugin or by the plugin host interface.
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("{0}")]
    Message(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("unsupported: {0}")]
    Unsupported(String),
}

impl From<serde_json::Error> for PluginError {
    fn from(error: serde_json::Error) -> Self {
        PluginError::Message(error.to_string())
    }
}

/// Convenience alias for plugin results.
pub type PluginResult<T> = std::result::Result<T, PluginError>;

/// A boxed, `Send` future used by plugin hooks and op handlers.
pub type BoxFut<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;

// ---------------------------------------------------------------------------
// Trust tiers and capabilities
// ---------------------------------------------------------------------------

/// The trust tier a plugin belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Tier {
    /// Compiled in, in-process, may use privileged primitives. Shipped by us.
    FirstParty,
    /// Untrusted; must run out-of-process, sandboxed, capability-gated.
    ThirdParty,
}

/// A capability a plugin may request. Predicates/scoping are enforced by the
/// capability proxy once the third-party tier lands; the enum already records
/// the classification so first-party-only primitives can never be granted to a
/// third-party plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    // On by default for a granted origin/context.
    Navigate,
    Query,
    Click,
    Fill,
    TypeKeys,
    WaitFor,
    WaitForNavigation,
    GetText,
    GetAttribute,
    // Dangerous: off by default, consent + audit.
    Evaluate,
    CdpProxy,
    ReadCookies,
    WriteCookies,
    InitScript,
    Screenshot,
    NetworkCapture,
    // First-party only: never granted to third-party plugins.
    LaunchControl,
    BinaryPatch,
    DetachedSpawn,
}

impl Capability {
    /// Dangerous capabilities require explicit consent and are audited.
    pub fn is_dangerous(self) -> bool {
        matches!(
            self,
            Capability::Evaluate
                | Capability::CdpProxy
                | Capability::ReadCookies
                | Capability::WriteCookies
                | Capability::InitScript
                | Capability::Screenshot
                | Capability::NetworkCapture
        )
    }

    /// Privileged capabilities reserved for the first-party tier.
    pub fn is_first_party_only(self) -> bool {
        matches!(
            self,
            Capability::LaunchControl | Capability::BinaryPatch | Capability::DetachedSpawn
        )
    }

    /// Stable identifier used in manifests and audit records.
    pub fn as_str(self) -> &'static str {
        match self {
            Capability::Navigate => "navigate",
            Capability::Query => "query",
            Capability::Click => "click",
            Capability::Fill => "fill",
            Capability::TypeKeys => "type_keys",
            Capability::WaitFor => "wait_for",
            Capability::WaitForNavigation => "wait_for_navigation",
            Capability::GetText => "get_text",
            Capability::GetAttribute => "get_attribute",
            Capability::Evaluate => "evaluate",
            Capability::CdpProxy => "cdp_proxy",
            Capability::ReadCookies => "read_cookies",
            Capability::WriteCookies => "write_cookies",
            Capability::InitScript => "init_script",
            Capability::Screenshot => "screenshot",
            Capability::NetworkCapture => "network_capture",
            Capability::LaunchControl => "launch_control",
            Capability::BinaryPatch => "binary_patch",
            Capability::DetachedSpawn => "detached_spawn",
        }
    }
}

// ---------------------------------------------------------------------------
// Manifest
// ---------------------------------------------------------------------------

/// Declarative description of what a plugin is and what it needs.
///
/// First-party manifests are built in Rust by [`Plugin::manifest`]. Third-party
/// manifests are authored as `plugin.json` and parsed with [`Manifest::from_json`]
/// or [`Manifest::load`]. Every manifest is validated by [`Manifest::validate`]
/// **before** any third-party code is allowed to run.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    /// Unique plugin name, e.g. `"example.echo"`.
    pub name: String,
    pub version: String,
    pub tier: Tier,
    /// Host interface range this plugin targets, e.g. `">=1.0 <2.0"`.
    #[serde(default = "default_host_api")]
    pub host_api: String,
    /// Path to the plugin program, relative to the manifest. Required for
    /// third-party plugins.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entrypoint: Option<String>,
    /// Sandboxed ABI the plugin speaks, e.g. `"wasm32-wasi+rpc/1"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub abi: Option<String>,
    /// Ops the plugin exposes through `PluginHandle::invoke`.
    #[serde(default)]
    pub ops: Vec<String>,
    /// Capabilities the plugin requests. Default-deny; granted per the proxy.
    #[serde(default)]
    pub capabilities: Vec<Capability>,
    /// Resource limits requested by the plugin (clamped to host maxima).
    #[serde(default)]
    pub limits: Budgets,
}

fn default_host_api() -> String {
    "1.x".to_string()
}

impl Manifest {
    /// Parse and validate a manifest from JSON.
    pub fn from_json(json: &str) -> PluginResult<Self> {
        let manifest: Manifest = serde_json::from_str(json)?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Read and validate a `plugin.json` manifest from disk.
    pub fn load(path: &str) -> PluginResult<Self> {
        let text = std::fs::read_to_string(path)
            .map_err(|error| PluginError::NotFound(format!("cannot read '{path}': {error}")))?;
        Self::from_json(&text)
    }

    /// Enforce the security invariants for a manifest *before* any code runs.
    ///
    /// Reserved names are host policy; use [`Manifest::validate_reserved`] to
    /// pass the host's first-party catalog.
    pub fn validate(&self) -> PluginResult<()> {
        self.validate_reserved(&[])
    }

    /// Like [`Manifest::validate`], but also rejects names in `reserved` (the
    /// host's first-party catalog).
    pub fn validate_reserved(&self, reserved: &[&str]) -> PluginResult<()> {
        if self.name.trim().is_empty() {
            return Err(PluginError::Unsupported(
                "plugin manifest: 'name' is required".to_string(),
            ));
        }
        if !self
            .name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
        {
            return Err(PluginError::Unsupported(format!(
                "plugin manifest: name '{}' may only contain [A-Za-z0-9._-]",
                self.name
            )));
        }
        if self.version.trim().is_empty() {
            return Err(PluginError::Unsupported(
                "plugin manifest: 'version' is required".to_string(),
            ));
        }
        if self.host_api.trim().is_empty() {
            return Err(PluginError::Unsupported(
                "plugin manifest: 'host_api' is required".to_string(),
            ));
        }

        if self.tier == Tier::ThirdParty {
            if reserved.contains(&self.name.as_str()) {
                return Err(PluginError::Unsupported(format!(
                    "plugin name '{}' is reserved for a first-party plugin",
                    self.name
                )));
            }
            if self.entrypoint.as_deref().unwrap_or("").trim().is_empty() {
                return Err(PluginError::Unsupported(
                    "third-party plugin manifest requires an 'entrypoint'".to_string(),
                ));
            }
            if self.ops.is_empty() {
                return Err(PluginError::Unsupported(
                    "third-party plugin manifest must declare at least one op".to_string(),
                ));
            }
            if let Some(capability) = self
                .capabilities
                .iter()
                .find(|capability| capability.is_first_party_only())
            {
                return Err(PluginError::Unsupported(format!(
                    "capability '{}' is first-party only and cannot be requested by a third-party plugin",
                    capability.as_str()
                )));
            }
            let hard = Budgets::default();
            if self.limits.max_invoke_millis == 0
                || self.limits.max_invoke_millis > hard.max_invoke_millis
            {
                return Err(PluginError::Unsupported(format!(
                    "third-party plugin 'max_invoke_millis' must be 1..={}",
                    hard.max_invoke_millis
                )));
            }
            if self.limits.max_response_bytes == 0
                || self.limits.max_response_bytes > hard.max_response_bytes
            {
                return Err(PluginError::Unsupported(format!(
                    "third-party plugin 'max_response_bytes' must be 1..={}",
                    hard.max_response_bytes
                )));
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Audit log (append-only, hash-chained)
// ---------------------------------------------------------------------------

/// One immutable audit record.
#[derive(Debug, Clone)]
pub struct AuditEvent {
    pub seq: u64,
    pub plugin: String,
    pub action: String,
    pub detail: String,
    pub prev_hash: u64,
    pub hash: u64,
}

static AUDIT: OnceLock<Mutex<Vec<AuditEvent>>> = OnceLock::new();

fn audit_log() -> &'static Mutex<Vec<AuditEvent>> {
    AUDIT.get_or_init(|| Mutex::new(Vec::new()))
}

/// FNV-1a, chained with the previous record's hash. Illustrative integrity
/// chain - values are redacted and the log is append-only.
fn chain_hash(prev: u64, seq: u64, plugin: &str, action: &str, detail: &str) -> u64 {
    let mut acc: u64 = 0xcbf2_9ce4_8422_2325 ^ prev;
    for byte in seq
        .to_le_bytes()
        .iter()
        .chain(plugin.as_bytes())
        .chain(action.as_bytes())
        .chain(detail.as_bytes())
    {
        acc ^= u64::from(*byte);
        acc = acc.wrapping_mul(0x0000_0100_0000_01b3);
    }
    acc
}

/// Append an event. `detail` must never contain secrets (cookies/credentials).
pub fn audit(plugin: &str, action: &str, detail: &str) {
    let mut log = audit_log().lock().unwrap();
    let seq = log.len() as u64 + 1;
    let prev_hash = log.last().map(|e| e.hash).unwrap_or(0);
    let hash = chain_hash(prev_hash, seq, plugin, action, detail);
    log.push(AuditEvent {
        seq,
        plugin: plugin.to_string(),
        action: action.to_string(),
        detail: detail.to_string(),
        prev_hash,
        hash,
    });
}

/// A snapshot of the audit log.
pub fn audit_entries() -> Vec<AuditEvent> {
    audit_log().lock().unwrap().clone()
}

/// Verify the hash chain is intact.
pub fn audit_verify() -> bool {
    let log = audit_log().lock().unwrap();
    let mut prev = 0u64;
    for (index, event) in log.iter().enumerate() {
        if event.seq != index as u64 + 1 || event.prev_hash != prev {
            return false;
        }
        if chain_hash(prev, event.seq, &event.plugin, &event.action, &event.detail) != event.hash {
            return false;
        }
        prev = event.hash;
    }
    true
}

// ---------------------------------------------------------------------------
// Supervisor budgets
// ---------------------------------------------------------------------------

/// Resource budgets enforced per plugin invocation.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Budgets {
    pub max_invoke_millis: u64,
    pub max_response_bytes: usize,
}

impl Default for Budgets {
    fn default() -> Self {
        Self {
            max_invoke_millis: 30_000,
            max_response_bytes: 1_048_576,
        }
    }
}

// ---------------------------------------------------------------------------
// Launch plan
// ---------------------------------------------------------------------------

/// Launch-time contributions collected from enabled plugins *before* the
/// browser process is spawned. Only first-party plugins may mutate this.
#[derive(Debug, Clone)]
pub struct LaunchPlan {
    pub executable: PathBuf,
    pub headless: bool,
    pub detached: bool,
    pub extra_args: Vec<String>,
    pub patched: bool,
}

impl LaunchPlan {
    pub fn new(executable: PathBuf, headless: bool, detached: bool) -> Self {
        Self {
            executable,
            headless,
            detached,
            extra_args: Vec::new(),
            patched: false,
        }
    }

    pub fn add_arg(&mut self, arg: impl Into<String>) {
        self.extra_args.push(arg.into());
    }
}

// ---------------------------------------------------------------------------
// Op registry
// ---------------------------------------------------------------------------

/// Context passed to a plugin op handler.
pub struct OpCall {
    pub plugin: String,
    pub op: String,
    pub args_json: String,
    pub page: Option<ArcPageHost>,
}

type OpHandler = Arc<dyn Fn(OpCall) -> BoxFut<PluginResult<String>> + Send + Sync>;

/// A plugin's op table, populated from [`Plugin::build`].
#[derive(Default)]
pub struct Registry {
    pub(crate) ops: BTreeMap<String, OpHandler>,
}

impl Registry {
    /// Register a named op with its handler.
    pub fn op<F>(&mut self, name: &str, handler: F)
    where
        F: Fn(OpCall) -> BoxFut<PluginResult<String>> + Send + Sync + 'static,
    {
        self.ops.insert(name.to_string(), Arc::new(handler));
    }
}

// ---------------------------------------------------------------------------
// Host interface
// ---------------------------------------------------------------------------

/// A capability-scoped view of a page, handed to plugins.
///
/// Plugins never receive the engine's `Page` or transport handle directly - only
/// this interface - so the host stays in control of exactly what a plugin can
/// touch.
pub trait PageHost: Send + Sync + 'static {
    /// Install a script that runs on every new document.
    fn add_init_script(&self, script: String) -> BoxFut<PluginResult<()>>;
    /// Dispatch a raw CDP method on this page's session.
    fn dispatch_cdp(&self, method: String, params_json: String) -> BoxFut<PluginResult<String>>;
    /// Move the mouse to `(x, y)`.
    fn move_mouse(&self, x: f64, y: f64) -> BoxFut<PluginResult<()>>;
    /// Move to `(x, y)` and click.
    fn click_mouse(&self, x: f64, y: f64) -> BoxFut<PluginResult<()>>;
    /// Type `text` into the focused element.
    fn keyboard_type(&self, text: String) -> BoxFut<PluginResult<()>>;
    /// The current mouse position.
    fn mouse_position(&self) -> (f64, f64);
}

/// A shared page host handle.
pub type ArcPageHost = Arc<dyn PageHost>;

// ---------------------------------------------------------------------------
// The Plugin trait
// ---------------------------------------------------------------------------

/// A plugin. First-party plugins implement this in Rust; third-party plugins
/// will be adapted onto it by the sandboxed runner.
pub trait Plugin: Send + Sync + 'static {
    /// Unique, reserved name (e.g. `"stealth"`).
    fn name(&self) -> &str;

    /// Trust tier (defaults to first-party).
    fn tier(&self) -> Tier {
        Tier::FirstParty
    }

    /// Whether the plugin must be enabled before the browser launches.
    fn requires_launch(&self) -> bool {
        false
    }

    /// Declarative manifest.
    fn manifest(&self) -> Manifest;

    /// Contribute to the launch plan (privileged; first-party only).
    fn configure_launch(&self, _plan: &mut LaunchPlan) -> PluginResult<()> {
        Ok(())
    }

    /// Register ops for `invoke`.
    fn build(&self, _reg: &mut Registry) {}

    /// Called after every page is created, before it navigates.
    fn on_page_created(&self, _page: ArcPageHost) -> BoxFut<PluginResult<()>> {
        Box::pin(async { Ok(()) })
    }
}

// ---------------------------------------------------------------------------
// Plugin manager
// ---------------------------------------------------------------------------

/// Resolves a plugin name to an implementation - the host's first-party catalog.
pub type Catalog = Arc<dyn Fn(&str) -> Option<Arc<dyn Plugin>> + Send + Sync>;

/// Owns the enabled plugins and dispatches hooks/ops. Held by `Browser`.
///
/// The plugin set can grow at runtime (see [`PluginManager::enable`]), so it is
/// guarded by an `RwLock`. Guards are never held across an `await`.
pub struct PluginManager {
    catalog: Catalog,
    plugins: RwLock<Vec<Arc<dyn Plugin>>>,
    tables: RwLock<BTreeMap<String, BTreeMap<String, OpHandler>>>,
    budgets: Budgets,
    launched: AtomicBool,
}

impl PluginManager {
    /// Build a manager from a list of plugin names resolved through `catalog`.
    /// Unknown / third-party names are refused (the sandboxed tier is not
    /// implemented yet).
    pub fn new(names: &[String], catalog: Catalog) -> PluginResult<Self> {
        let manager = Self {
            catalog,
            plugins: RwLock::new(Vec::new()),
            tables: RwLock::new(BTreeMap::new()),
            budgets: Budgets::default(),
            launched: AtomicBool::new(false),
        };
        for name in names {
            manager.enable(name)?;
        }
        Ok(manager)
    }

    /// Enable a compiled-in first-party plugin. Idempotent; unknown or
    /// third-party names are refused so untrusted code is never executed.
    ///
    /// Enabling after launch is recorded as a runtime enable: the plugin's
    /// launch-time contribution (if any) has already been skipped.
    pub fn enable(&self, name: &str) -> PluginResult<()> {
        if self.has(name) {
            return Ok(());
        }
        let plugin = (self.catalog)(name).ok_or_else(|| {
            PluginError::Unsupported(format!(
                "plugin '{name}' is not a known first-party plugin; \
                 third-party plugins are not supported in this phase"
            ))
        })?;

        let mut reg = Registry::default();
        plugin.build(&mut reg);
        self.tables
            .write()
            .unwrap()
            .insert(name.to_string(), reg.ops);
        self.plugins.write().unwrap().push(plugin);

        let detail = if self.launched.load(Ordering::SeqCst) {
            "runtime"
        } else {
            "launch"
        };
        audit(name, "enable", detail);
        Ok(())
    }

    /// Record that the launch phase has completed.
    pub fn mark_launched(&self) {
        self.launched.store(true, Ordering::SeqCst);
    }

    pub fn names(&self) -> Vec<String> {
        self.plugins
            .read()
            .unwrap()
            .iter()
            .map(|p| p.name().to_string())
            .collect()
    }

    pub fn has(&self, name: &str) -> bool {
        self.plugins
            .read()
            .unwrap()
            .iter()
            .any(|p| p.name() == name)
    }

    pub fn manifest(&self, name: &str) -> Option<Manifest> {
        self.plugins
            .read()
            .unwrap()
            .iter()
            .find(|p| p.name() == name)
            .map(|p| p.manifest())
    }

    pub fn ops(&self, name: &str) -> Vec<String> {
        self.tables
            .read()
            .unwrap()
            .get(name)
            .map(|table| table.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Clone the current plugin list so hooks can be awaited without the lock.
    fn snapshot(&self) -> Vec<Arc<dyn Plugin>> {
        self.plugins.read().unwrap().clone()
    }

    /// Run the launch phase for every enabled plugin.
    pub fn configure_launch(&self, plan: &mut LaunchPlan) -> PluginResult<()> {
        for plugin in self.snapshot() {
            plugin.configure_launch(plan)?;
            audit(plugin.name(), "configure_launch", "ok");
        }
        Ok(())
    }

    /// Run the page-created phase for every enabled plugin.
    pub async fn on_page_created(&self, page: ArcPageHost) -> PluginResult<()> {
        for plugin in self.snapshot() {
            plugin.on_page_created(Arc::clone(&page)).await?;
            audit(plugin.name(), "on_page_created", "ok");
        }
        Ok(())
    }

    /// Invoke a plugin op, enforcing budgets and recording an audit event.
    pub async fn invoke(
        &self,
        plugin: &str,
        op: &str,
        args_json: String,
        page: Option<ArcPageHost>,
    ) -> PluginResult<String> {
        let handler = {
            let tables = self.tables.read().unwrap();
            let table = tables.get(plugin).ok_or_else(|| {
                PluginError::NotFound(format!("plugin '{plugin}' is not enabled"))
            })?;
            table.get(op).cloned().ok_or_else(|| {
                PluginError::NotFound(format!("plugin '{plugin}' has no op '{op}'"))
            })?
        };

        let started = std::time::Instant::now();
        let outcome = handler(OpCall {
            plugin: plugin.to_string(),
            op: op.to_string(),
            args_json,
            page,
        })
        .await;
        let elapsed_ms = started.elapsed().as_millis() as u64;

        if elapsed_ms > self.budgets.max_invoke_millis {
            audit(plugin, op, "budget:time-exceeded");
            return Err(PluginError::Unsupported(format!(
                "plugin '{plugin}.{op}' exceeded the {} ms budget",
                self.budgets.max_invoke_millis
            )));
        }
        if let Ok(result) = &outcome
            && result.len() > self.budgets.max_response_bytes
        {
            audit(plugin, op, "budget:response-too-large");
            return Err(PluginError::Unsupported(format!(
                "plugin '{plugin}.{op}' response exceeds {} bytes",
                self.budgets.max_response_bytes
            )));
        }

        audit(plugin, op, if outcome.is_ok() { "ok" } else { "error" });
        outcome
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_MANIFEST: &str = r#"{
        "name": "example.echo",
        "version": "0.1.0",
        "tier": "third-party",
        "host_api": ">=1.0 <2.0",
        "entrypoint": "example-echo.wasm",
        "abi": "wasm32-wasi+rpc/1",
        "ops": ["echo"],
        "capabilities": ["query", "get_text"],
        "limits": { "max_invoke_millis": 5000, "max_response_bytes": 65536 }
    }"#;

    #[test]
    fn parses_valid_third_party_manifest() {
        let manifest = Manifest::from_json(VALID_MANIFEST).unwrap();
        assert_eq!(manifest.name, "example.echo");
        assert_eq!(manifest.tier, Tier::ThirdParty);
        assert_eq!(
            manifest.capabilities,
            vec![Capability::Query, Capability::GetText]
        );
        assert_eq!(manifest.limits.max_invoke_millis, 5000);
    }

    #[test]
    fn rejects_first_party_only_capability() {
        let json = VALID_MANIFEST.replace(r#""query", "get_text""#, r#""binary_patch""#);
        assert!(matches!(
            Manifest::from_json(&json).unwrap_err(),
            PluginError::Unsupported(_)
        ));
    }

    #[test]
    fn rejects_reserved_name_via_host_catalog() {
        let manifest = Manifest::from_json(VALID_MANIFEST).unwrap();
        assert!(manifest.validate_reserved(&["stealth"]).is_ok());
        assert!(matches!(
            manifest.validate_reserved(&["example.echo"]).unwrap_err(),
            PluginError::Unsupported(_)
        ));
    }

    #[test]
    fn rejects_missing_entrypoint() {
        let json = VALID_MANIFEST.replace(r#""entrypoint": "example-echo.wasm","#, "");
        assert!(matches!(
            Manifest::from_json(&json).unwrap_err(),
            PluginError::Unsupported(_)
        ));
    }

    #[test]
    fn rejects_limits_above_host_maxima() {
        let json = VALID_MANIFEST.replace("65536", "999999999");
        assert!(matches!(
            Manifest::from_json(&json).unwrap_err(),
            PluginError::Unsupported(_)
        ));
    }

    #[test]
    fn rejects_unknown_capability_and_unknown_field() {
        let bad_capability = VALID_MANIFEST.replace(r#""query", "get_text""#, r#""root_shell""#);
        assert!(matches!(
            Manifest::from_json(&bad_capability).unwrap_err(),
            PluginError::Message(_)
        ));

        let extra_field = VALID_MANIFEST.replace(
            r#""ops": ["echo"],"#,
            r#""ops": ["echo"], "backdoor": true,"#,
        );
        assert!(matches!(
            Manifest::from_json(&extra_field).unwrap_err(),
            PluginError::Message(_)
        ));
    }

    #[test]
    fn documented_example_manifest_validates() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../docs/plugins/examples/echo/plugin.json"
        );
        let manifest = Manifest::load(path).unwrap();
        assert_eq!(manifest.name, "example.echo");
        assert_eq!(manifest.tier, Tier::ThirdParty);
        assert!(manifest.ops.contains(&"echo".to_string()));
    }

    #[test]
    fn audit_log_chain_is_intact() {
        audit("test", "action", "never-a-secret");
        assert!(audit_verify());
        assert!(audit_entries().iter().any(|e| e.plugin == "test"));
    }
}
