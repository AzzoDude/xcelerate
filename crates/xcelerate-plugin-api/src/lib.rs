//! Plugin API for `xcelerate`.
//!
//! This crate is the boundary between the engine and its plugins. It defines
//! the [`Plugin`] trait, the [`Manifest`] specification, the [`Capability`] /
//! trust model, the append-only [`audit`] log, and the [`PageHost`]
//! interface a plugin uses to touch a page.
//!
//! It deliberately depends only on `serde`/`serde_json` - never on the engine
//! facade - so plugin crates can implement plugins without a dependency cycle.
//!
//! # Security posture
//!
//! * **default-deny** - a plugin does nothing unless it is explicitly enabled;
//! * plugins never see a raw page or transport handle - only [`PageHost`];
//! * every privileged action is recorded in an append-only, hash-chained log;
//! * a manifest may not request a host-only [`Capability`].

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
// Capabilities
// ---------------------------------------------------------------------------

/// A capability a plugin may request. Predicates/scoping are enforced by the
/// capability proxy; the enum records the classification so sandbox-only
/// primitives can never be granted across a sandbox boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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
    // Host-only: never granted to loaded plugins.
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

    /// Privileged capabilities reserved for built-in (compiled-in) plugins.
    pub fn is_builtin_only(self) -> bool {
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
/// Built-in manifests are constructed in Rust by [`Plugin::manifest`]. Loaded
/// manifests are authored as `plugin.json` and parsed with [`Manifest::from_json`]
/// or [`Manifest::load`]. Every manifest is validated by [`Manifest::validate`]
/// **before** any plugin code is allowed to run.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    /// Unique plugin name, e.g. `"example.plugin"`.
    pub name: String,
    pub version: String,
    /// Host interface range this plugin targets, e.g. `">=1.0 <2.0"`.
    #[serde(default = "default_host_api")]
    pub host_api: String,
    /// Path to the plugin program, relative to the manifest. Required for
    /// sandboxed plugins.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entrypoint: Option<String>,
    /// Sandboxed ABI the plugin speaks, e.g. `"wasm32-wasip2/1"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub abi: Option<String>,
    /// Ops the plugin exposes through `PluginHandle::invoke`.
    #[serde(default)]
    pub ops: Vec<String>,
    /// Capabilities the plugin requests. Default-deny; granted per the proxy.
    #[serde(default)]
    pub capabilities: Vec<Capability>,
    /// Other plugins this one depends on: plugin name -> version range
    /// (e.g. `{ "acme.totp": "^1.0" }`). Resolved and loaded before this one.
    #[serde(default)]
    pub dependencies: std::collections::BTreeMap<String, String>,
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
    /// pass the host's built-in catalog.
    pub fn validate(&self) -> PluginResult<()> {
        self.validate_reserved(&[])
    }

    /// Like [`Manifest::validate`], but also rejects names in `reserved` (the
    /// host's built-in catalog).
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

        for (dependency, range) in &self.dependencies {
            if dependency.trim().is_empty() || range.trim().is_empty() {
                return Err(PluginError::Unsupported(
                    "plugin dependencies need a non-empty name and version range".to_string(),
                ));
            }
            if dependency == &self.name {
                return Err(PluginError::Unsupported(format!(
                    "plugin '{}' cannot depend on itself",
                    self.name
                )));
            }
        }

        if reserved.contains(&self.name.as_str()) {
            return Err(PluginError::Unsupported(format!(
                "plugin name '{}' is reserved for a built-in plugin",
                self.name
            )));
        }
        if self.entrypoint.as_deref().unwrap_or("").trim().is_empty() {
            return Err(PluginError::Unsupported(
                "sandboxed plugin manifest requires an 'entrypoint'".to_string(),
            ));
        }
        if self.ops.is_empty() {
            return Err(PluginError::Unsupported(
                "sandboxed plugin manifest must declare at least one op".to_string(),
            ));
        }
        if let Some(capability) = self
            .capabilities
            .iter()
            .find(|capability| capability.is_builtin_only())
        {
            return Err(PluginError::Unsupported(format!(
                "capability '{}' is host-only and cannot be requested by a loaded plugin",
                capability.as_str()
            )));
        }
        let hard = Budgets::default();
        if self.limits.max_invoke_millis == 0
            || self.limits.max_invoke_millis > hard.max_invoke_millis
        {
            return Err(PluginError::Unsupported(format!(
                "sandboxed plugin 'max_invoke_millis' must be 1..={}",
                hard.max_invoke_millis
            )));
        }
        if self.limits.max_response_bytes == 0
            || self.limits.max_response_bytes > hard.max_response_bytes
        {
            return Err(PluginError::Unsupported(format!(
                "sandboxed plugin 'max_response_bytes' must be 1..={}",
                hard.max_response_bytes
            )));
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
/// browser process is spawned. Only built-in plugins may mutate this.
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

/// A plugin. Built-in plugins implement this in Rust; loaded plugins are
/// adapted onto it by the runner.
pub trait Plugin: Send + Sync + 'static {
    /// Unique, reserved name (e.g. `"stealth"`).
    fn name(&self) -> &str;

    /// Whether the plugin must be enabled before the browser launches.
    fn requires_launch(&self) -> bool {
        false
    }

    /// Declarative manifest.
    fn manifest(&self) -> Manifest;

    /// Contribute to the launch plan (privileged; built-in only).
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

/// Resolves a plugin name to an implementation - the host's built-in catalog.
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
    /// Unknown names are refused.
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

    /// Enable a built-in plugin. Idempotent; unknown names are
    /// refused so unknown code is never executed.
    ///
    /// Enabling after launch is recorded as a runtime enable: the plugin's
    /// launch-time contribution (if any) has already been skipped.
    pub fn enable(&self, name: &str) -> PluginResult<()> {
        if self.has(name) {
            return Ok(());
        }
        let plugin = (self.catalog)(name).ok_or_else(|| {
            PluginError::Unsupported(format!("plugin '{name}' is not a known built-in plugin"))
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

    /// Install a trusted, in-process plugin directly.
    ///
    /// This is the compile-time extension point ("a plugin is a library"): a
    /// plugin crate added as a Cargo dependency is installed here by the
    /// embedder. Such a plugin runs in-process and is therefore trusted exactly
    /// like the built-in catalog - it may use the whole [`PageHost`] interface.
    /// Plugins loaded from disk run sandboxed and capability-gated instead.
    ///
    /// Names already owned by the host catalog (for example `stealth`) are
    /// refused, so a library plugin can never shadow a built-in one. Install
    /// before creating pages so the plugin's `on_page_created` hook sees them.
    pub fn install(&self, plugin: Arc<dyn Plugin>) -> PluginResult<()> {
        let name = plugin.name().to_string();
        if name.trim().is_empty() {
            return Err(PluginError::Unsupported(
                "cannot install a plugin with an empty name".to_string(),
            ));
        }
        if (self.catalog)(&name).is_some() {
            return Err(PluginError::Unsupported(format!(
                "plugin name '{name}' is reserved by the host catalog"
            )));
        }
        if self.has(&name) {
            return Err(PluginError::Unsupported(format!(
                "plugin '{name}' is already installed"
            )));
        }

        let mut reg = Registry::default();
        plugin.build(&mut reg);
        self.tables.write().unwrap().insert(name.clone(), reg.ops);
        self.plugins.write().unwrap().push(plugin);
        audit(&name, "install", "in-process");
        Ok(())
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
        "name": "example.plugin",
        "version": "0.1.0",
        "host_api": ">=1.0 <2.0",
        "entrypoint": "example.wasm",
        "abi": "wasm32-wasip2/1",
        "ops": ["run"],
        "capabilities": ["query", "get_text"],
        "limits": { "max_invoke_millis": 5000, "max_response_bytes": 65536 }
    }"#;

    #[test]
    fn parses_valid_sandboxed_manifest() {
        let manifest = Manifest::from_json(VALID_MANIFEST).unwrap();
        assert_eq!(manifest.name, "example.plugin");
        assert_eq!(
            manifest.capabilities,
            vec![Capability::Query, Capability::GetText]
        );
        assert_eq!(manifest.limits.max_invoke_millis, 5000);
    }

    #[test]
    fn rejects_host_only_capability() {
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
            manifest.validate_reserved(&["example.plugin"]).unwrap_err(),
            PluginError::Unsupported(_)
        ));
    }

    #[test]
    fn rejects_missing_entrypoint() {
        let json = VALID_MANIFEST.replace(r#""entrypoint": "example.wasm","#, "");
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

        let extra_field =
            VALID_MANIFEST.replace(r#""ops": ["run"],"#, r#""ops": ["run"], "backdoor": true,"#);
        assert!(matches!(
            Manifest::from_json(&extra_field).unwrap_err(),
            PluginError::Message(_)
        ));
    }

    #[test]
    fn parses_and_validates_dependencies() {
        let with_dep = VALID_MANIFEST.replace(
            r#""ops": ["run"],"#,
            r#""ops": ["run"], "dependencies": { "acme.totp": "^1.0" },"#,
        );
        let manifest = Manifest::from_json(&with_dep).unwrap();
        assert_eq!(
            manifest.dependencies.get("acme.totp").map(String::as_str),
            Some("^1.0")
        );

        let self_dep = VALID_MANIFEST.replace(
            r#""ops": ["run"],"#,
            r#""ops": ["run"], "dependencies": { "example.plugin": "^1.0" },"#,
        );
        assert!(matches!(
            Manifest::from_json(&self_dep).unwrap_err(),
            PluginError::Unsupported(_)
        ));
    }

    #[test]
    fn documented_example_manifest_validates() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../docs/plugins/examples/wasm-echo/plugin.json"
        );
        let manifest = Manifest::load(path).unwrap();
        assert_eq!(manifest.name, "example.wasm-echo");
        assert!(manifest.ops.contains(&"echo".to_string()));
    }

    #[test]
    fn dependency_example_manifests_validate() {
        let base = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/plugins/examples");

        let provider = Manifest::load(&format!("{base}/kv-store/plugin.json")).unwrap();
        assert_eq!(provider.name, "acme.kv");
        assert!(provider.dependencies.is_empty());

        let consumer = Manifest::load(&format!("{base}/notes/plugin.json")).unwrap();
        assert_eq!(consumer.name, "acme.notes");
        assert_eq!(
            consumer.dependencies.get("acme.kv").map(String::as_str),
            Some("^1.0")
        );
    }

    #[test]
    fn audit_log_chain_is_intact() {
        audit("test", "action", "never-a-secret");
        assert!(audit_verify());
        assert!(audit_entries().iter().any(|e| e.plugin == "test"));
    }

    /// A minimal in-process plugin, standing in for a library a user adds as a
    /// Cargo dependency.
    struct LibraryPlugin;

    impl Plugin for LibraryPlugin {
        fn name(&self) -> &str {
            "example.library"
        }
        fn manifest(&self) -> Manifest {
            Manifest {
                name: "example.library".to_string(),
                version: "0.1.0".to_string(),
                host_api: "1.x".to_string(),
                entrypoint: None,
                abi: None,
                ops: vec!["ping".to_string()],
                capabilities: vec![],
                dependencies: Default::default(),
                limits: Budgets::default(),
            }
        }
        fn build(&self, reg: &mut Registry) {
            reg.op("ping", |_call| {
                Box::pin(async { Ok("\"pong\"".to_string()) })
            });
        }
    }

    fn empty_catalog() -> Catalog {
        Arc::new(|_name: &str| -> Option<Arc<dyn Plugin>> { None })
    }

    #[test]
    fn installs_a_library_plugin() {
        let manager = PluginManager::new(&[], empty_catalog()).unwrap();
        manager.install(Arc::new(LibraryPlugin)).unwrap();
        assert_eq!(manager.names(), vec!["example.library".to_string()]);
        assert_eq!(manager.ops("example.library"), vec!["ping".to_string()]);
    }

    #[test]
    fn refuses_installing_a_catalog_name_or_duplicate() {
        let catalog: Catalog = Arc::new(|name: &str| {
            (name == "example.library").then(|| Arc::new(LibraryPlugin) as Arc<dyn Plugin>)
        });
        let manager = PluginManager::new(&[], catalog).unwrap();
        // A name the host catalog owns cannot be shadowed.
        assert!(matches!(
            manager.install(Arc::new(LibraryPlugin)).unwrap_err(),
            PluginError::Unsupported(_)
        ));
        // And a plugin cannot be installed twice.
        let manager = PluginManager::new(&[], empty_catalog()).unwrap();
        manager.install(Arc::new(LibraryPlugin)).unwrap();
        assert!(matches!(
            manager.install(Arc::new(LibraryPlugin)).unwrap_err(),
            PluginError::Unsupported(_)
        ));
    }
}
