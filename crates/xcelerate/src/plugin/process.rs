//! Out-of-process (third-party) plugin host.
//!
//! A third-party plugin is **any program that speaks a line-delimited JSON-RPC
//! protocol over stdin/stdout**. That keeps it language-agnostic: the same
//! `plugin.json` + entrypoint works whether the entrypoint is a Python script, a
//! Node script, a Go binary, or anything else that can read and write JSON.
//!
//! The host never loads third-party code in-process. It spawns the entrypoint as
//! a child process, runs a `describe` handshake, and forwards `invoke` calls.
//! Every privileged thing the plugin can ask the host to do (`host.get_cookies`,
//! ...) is checked against a **default-deny** grant list and audited.
//!
//! ## Hardening (threat model: the plugin is untrusted)
//!
//! The child is a separate process, but that alone is *not* an OS sandbox - it
//! still runs as the host user. Until a platform sandbox (Job Objects / seccomp /
//! AppContainer) lands, this module closes the cheap vectors:
//!
//! * the interpreter is resolved to an **absolute, canonical path** (no `PATH`
//!   or current-directory executable planting) and the child's cwd is not moved
//!   into the plugin directory;
//! * the child environment is **cleared** and repopulated from a small allow-list
//!   (no host tokens/keys, no `XCELERATE_PLUGIN_ALLOW` leak);
//! * `invoke` and host callbacks are **bounded by timeouts**;
//! * every read frame is **size-capped**;
//! * the `entrypoint` must resolve **inside the plugin directory**;
//! * grants and denials are **audited**, and `host.log` is sanitized and capped.
//!
//! ## Protocol (ABI `rpc/1`)
//!
//! Host to plugin:
//!
//! ```json
//! {"jsonrpc":"2.0","id":1,"method":"describe","params":{"manifest":{...}}}
//! {"jsonrpc":"2.0","id":2,"method":"invoke","params":{"op":"echo","args":{...}}}
//! {"jsonrpc":"2.0","method":"shutdown"}
//! ```
//!
//! Plugin to host:
//!
//! ```json
//! {"jsonrpc":"2.0","id":1,"result":{"name":"example.echo","abi":"rpc/1","ops":["echo"]}}
//! {"jsonrpc":"2.0","id":9,"method":"host.get_cookies","params":{}}
//! ```

use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Sender as StdSender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};
use xcelerate_plugin_api::{
    BoxFut, Capability, Manifest, OpCall, Plugin, PluginError, PluginResult, Registry, Tier,
};

use crate::{CdpClient, XcelerateError, XcelerateResult};

/// The ABI string this host speaks.
pub(crate) const RPC_ABI: &str = "rpc/1";

/// How long `load_plugin` waits for the `describe` handshake.
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);

/// Ceiling for a host callback (a CDP round-trip) run from the reader thread.
const CALLBACK_TIMEOUT: Duration = Duration::from_secs(30);

/// The maximum size of a single protocol frame (a line).
const MAX_FRAME_BYTES: u64 = 8 * 1024 * 1024;

/// The only environment variables a plugin child inherits.
const ENV_ALLOW: &[&str] = &[
    "PATH",
    "HOME",
    "LANG",
    "LC_ALL",
    "TEMP",
    "TMP",
    "TZ",
    "USERPROFILE",
    "APPDATA",
    "LOCALAPPDATA",
    "SystemRoot",
    "windir",
    "PATHEXT",
    "COMSPEC",
];

/// Resolve a `load_plugin` path (a plugin directory or a `plugin.json`) to the
/// manifest path.
pub(crate) fn resolve_manifest_path(path: &str) -> XcelerateResult<PathBuf> {
    let candidate = PathBuf::from(path);
    if candidate.is_dir() {
        Ok(candidate.join("plugin.json"))
    } else {
        Ok(candidate)
    }
}

/// Resolve the entrypoint, requiring it to stay inside the plugin directory.
///
/// Both sides are canonicalized, so an absolute path, a `..` escape, a UNC path,
/// or a symlink pointing outside the plugin directory is rejected.
fn resolve_entrypoint(manifest_path: &Path, manifest: &Manifest) -> XcelerateResult<PathBuf> {
    let entrypoint = manifest.entrypoint.as_deref().ok_or_else(|| {
        XcelerateError::Unsupported(format!("plugin '{}' has no entrypoint", manifest.name))
    })?;
    let base = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let base = base.canonicalize().map_err(|error| {
        XcelerateError::NotFound(format!(
            "plugin '{}': cannot resolve plugin directory: {error}",
            manifest.name
        ))
    })?;
    let resolved = base.join(entrypoint).canonicalize().map_err(|error| {
        XcelerateError::NotFound(format!(
            "plugin '{}' entrypoint not found: {error}",
            manifest.name
        ))
    })?;
    if !resolved.starts_with(&base) {
        return Err(XcelerateError::Unsupported(format!(
            "plugin '{}' entrypoint escapes the plugin directory",
            manifest.name
        )));
    }
    if !resolved.is_file() {
        return Err(XcelerateError::Unsupported(format!(
            "plugin '{}' entrypoint is not a regular file",
            manifest.name
        )));
    }
    Ok(resolved)
}

/// The capabilities the host grants this plugin.
///
/// First-party-only capabilities are never granted. Dangerous capabilities are
/// denied unless the host opted in, either per plugin (`<name>:<capability>`) or
/// broadly (`<capability>`), via `XCELERATE_PLUGIN_ALLOW`.
pub(crate) fn granted_capabilities(
    manifest: &Manifest,
    allow: &HashSet<String>,
) -> HashSet<Capability> {
    manifest
        .capabilities
        .iter()
        .copied()
        .filter(|capability| !capability.is_first_party_only())
        .filter(|capability| {
            !capability.is_dangerous()
                || allow.contains(capability.as_str())
                || allow.contains(&format!("{}:{}", manifest.name, capability.as_str()))
        })
        .collect()
}

/// Parse the `XCELERATE_PLUGIN_ALLOW` opt-in list (comma separated).
fn allow_list() -> HashSet<String> {
    std::env::var("XCELERATE_PLUGIN_ALLOW")
        .unwrap_or_default()
        .split(',')
        .map(|entry| entry.trim().to_string())
        .filter(|entry| !entry.is_empty())
        .collect()
}

/// Find `program` on `PATH`, returning a canonical absolute path. On Windows the
/// `PATHEXT` extensions are tried. The current directory is deliberately never
/// searched, so a planted executable next to a plugin cannot be picked up.
fn resolve_interpreter(program: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    let mut names = vec![program.to_string()];
    if cfg!(windows) {
        let pathext = std::env::var("PATHEXT").unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".into());
        for extension in pathext.split(';') {
            let extension = extension.trim();
            if !extension.is_empty() {
                names.push(format!("{program}{}", extension.to_ascii_lowercase()));
            }
        }
    }
    for directory in std::env::split_paths(&path) {
        if directory.as_os_str().is_empty() {
            continue;
        }
        for name in &names {
            let candidate = directory.join(name);
            if candidate.is_file() {
                return Some(candidate.canonicalize().unwrap_or(candidate));
            }
        }
    }
    None
}

/// How to launch an entrypoint, inferred from its file extension.
fn command_for(entrypoint: &Path) -> XcelerateResult<(PathBuf, Vec<String>)> {
    let interpreter = match entrypoint
        .extension()
        .and_then(|extension| extension.to_str())
    {
        Some("py") => Some("python"),
        Some("js") | Some("mjs") => Some("node"),
        Some("rb") => Some("ruby"),
        Some("sh") => Some("sh"),
        _ => None,
    };
    match interpreter {
        Some(program) => {
            let resolved = resolve_interpreter(program).ok_or_else(|| {
                XcelerateError::NotFound(format!(
                    "interpreter '{program}' was not found as an absolute path on PATH"
                ))
            })?;
            Ok((resolved, vec![entrypoint.to_string_lossy().into_owned()]))
        }
        None => Ok((entrypoint.to_path_buf(), Vec::new())),
    }
}

/// Read one newline-delimited frame, refusing frames larger than `max` bytes so
/// a hostile plugin cannot exhaust host memory with an endless line.
fn read_frame<R: BufRead>(reader: &mut R, max: u64, line: &mut String) -> std::io::Result<usize> {
    line.clear();
    let read = reader.by_ref().take(max).read_line(line)?;
    if read as u64 == max && !line.ends_with('\n') {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "plugin frame exceeded the maximum size",
        ));
    }
    Ok(read)
}

/// Map a CDP method to the specific capability it needs, so `cdp_proxy` cannot
/// be used to bypass `read_cookies` / `write_cookies` / `evaluate`.
fn capability_for_cdp(method: &str) -> Option<Capability> {
    match method {
        "Storage.getCookies" | "Network.getCookies" | "Storage.getCookiesForFrame" => {
            Some(Capability::ReadCookies)
        }
        "Storage.setCookies"
        | "Network.setCookie"
        | "Network.setCookies"
        | "Network.deleteCookies"
        | "Network.clearBrowserCookies"
        | "Storage.clearCookies" => Some(Capability::WriteCookies),
        "Runtime.evaluate" | "Runtime.callFunctionOn" | "Runtime.compileScript" => {
            Some(Capability::Evaluate)
        }
        _ => None,
    }
}

type PendingMap = Mutex<HashMap<u64, tokio::sync::oneshot::Sender<Result<Value, String>>>>;

/// The runtime side of a plugin process: request/response plumbing plus the
/// capability-gated host callbacks.
pub(crate) struct ProcessChannel {
    name: String,
    writer: Mutex<StdSender<String>>,
    pending: PendingMap,
    next_id: AtomicU64,
    client: Option<Arc<CdpClient>>,
    runtime: Mutex<Option<tokio::runtime::Handle>>,
    granted: HashSet<Capability>,
    budget: Duration,
    max_frame: u64,
}

impl ProcessChannel {
    /// Send an `invoke` and await the plugin's response, bounded by the budget.
    async fn call(&self, op: &str, args: Value) -> Result<Value, String> {
        // Record the runtime so the reader thread can drive host callbacks.
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            *self.runtime.lock().unwrap() = Some(handle);
        }
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.pending.lock().unwrap().insert(id, tx);

        let request = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "invoke",
            "params": { "op": op, "args": args },
        });
        if let Err(message) = self.send(&request.to_string()) {
            self.pending.lock().unwrap().remove(&id);
            return Err(message);
        }

        match tokio::time::timeout(self.budget, rx).await {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => Err("plugin closed before responding".to_string()),
            Err(_) => {
                // Do not leak the waiter if the plugin never answers.
                self.pending.lock().unwrap().remove(&id);
                Err(format!(
                    "plugin '{}' exceeded its {} ms invoke budget",
                    self.name,
                    self.budget.as_millis()
                ))
            }
        }
    }

    fn send(&self, line: &str) -> Result<(), String> {
        self.writer
            .lock()
            .unwrap()
            .send(format!("{line}\n"))
            .map_err(|_| "plugin stdin is closed".to_string())
    }

    /// Enforce a capability, auditing both the grant and the denial.
    fn gate(&self, capability: Capability, action: &str) -> Result<(), String> {
        if self.granted.contains(&capability) {
            xcelerate_plugin_api::audit(&self.name, action, "granted");
            Ok(())
        } else {
            xcelerate_plugin_api::audit(&self.name, action, "denied");
            Err(format!(
                "capability '{}' was not granted to this plugin",
                capability.as_str()
            ))
        }
    }

    /// Serve a plugin-to-host call, enforcing the capability grant.
    fn serve(&self, method: &str, params: &Value) -> Result<Value, String> {
        match method {
            "host.log" => {
                const MAX_LOG_CHARS: usize = 512;
                let message: String = params
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .chars()
                    .filter(|c| !c.is_control())
                    .take(MAX_LOG_CHARS)
                    .collect();
                xcelerate_plugin_api::audit(&self.name, "log", &message);
                Ok(json!({ "ok": true }))
            }
            "host.get_cookies" => {
                self.gate(Capability::ReadCookies, "host.get_cookies")?;
                // `Storage.getCookies` is a browser-level command (the target-level
                // `Network.getCookies` needs a page session the host may not have).
                self.cdp("Storage.getCookies", json!({}))
            }
            "host.set_cookie" => {
                self.gate(Capability::WriteCookies, "host.set_cookie")?;
                let cookie = params.get("cookie").cloned().unwrap_or(json!({}));
                self.cdp("Storage.setCookies", json!({ "cookies": [cookie] }))
            }
            "host.cdp" => {
                self.gate(Capability::CdpProxy, "host.cdp")?;
                let cdp_method = params
                    .get("method")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "host.cdp requires a 'method'".to_string())?;
                // `cdp_proxy` must not silently subsume the narrower grants.
                if let Some(needed) = capability_for_cdp(cdp_method) {
                    self.gate(needed, "host.cdp")?;
                }
                xcelerate_plugin_api::audit(&self.name, "host.cdp", cdp_method);
                let cdp_params = params.get("params").cloned().unwrap_or(json!({}));
                self.cdp(cdp_method, cdp_params)
            }
            other => Err(format!("unsupported host method '{other}'")),
        }
    }

    /// Run a root-session CDP command off the reader thread (bounded by a timeout).
    fn cdp(&self, method: &str, params: Value) -> Result<Value, String> {
        let client = self
            .client
            .clone()
            .ok_or_else(|| "this plugin has no browser attached".to_string())?;
        let handle = self
            .runtime
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| "host runtime is not available yet".to_string())?;
        let method = method.to_string();
        let (tx, rx) = mpsc::channel();
        handle.spawn(async move {
            let result = client
                .execute_raw(&method, params)
                .await
                .map_err(|error| error.to_string());
            let _ = tx.send(result);
        });
        rx.recv_timeout(CALLBACK_TIMEOUT)
            .map_err(|_| "host call timed out".to_string())?
    }
}

/// A loaded third-party plugin, driven over its child process.
pub(crate) struct OutOfProcessPlugin {
    manifest: Manifest,
    ops: Vec<String>,
    channel: Arc<ProcessChannel>,
    child: Mutex<Child>,
}

impl Plugin for OutOfProcessPlugin {
    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn tier(&self) -> Tier {
        Tier::ThirdParty
    }

    fn manifest(&self) -> Manifest {
        self.manifest.clone()
    }

    fn build(&self, reg: &mut Registry) {
        for op in &self.ops {
            let channel = Arc::clone(&self.channel);
            let name = op.clone();
            let op = op.clone();
            reg.op(&name, move |call: OpCall| -> BoxFut<PluginResult<String>> {
                let channel = Arc::clone(&channel);
                let op = op.clone();
                Box::pin(async move {
                    let args: Value = serde_json::from_str(&call.args_json).unwrap_or(Value::Null);
                    channel
                        .call(&op, args)
                        .await
                        .map(|result| result.to_string())
                        .map_err(PluginError::Message)
                })
            });
        }
    }
}

impl Drop for OutOfProcessPlugin {
    fn drop(&mut self) {
        let _ = self
            .channel
            .send(&json!({ "jsonrpc": "2.0", "method": "shutdown" }).to_string());
        if let Ok(mut child) = self.child.lock() {
            kill_and_wait(&mut child);
        }
    }
}

/// Kill and reap a child so failed loads cannot leak zombies.
fn kill_and_wait(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn id_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos() as u64)
        .unwrap_or(1)
        | 1
}

/// Spawn a third-party plugin, run the handshake, and return it ready to install.
pub(crate) fn spawn(
    manifest: &Manifest,
    manifest_path: &Path,
    client: Option<Arc<CdpClient>>,
) -> XcelerateResult<Arc<OutOfProcessPlugin>> {
    let entrypoint = resolve_entrypoint(manifest_path, manifest)?;
    let (program, args) = command_for(&entrypoint)?;

    let mut command = Command::new(&program);
    command.args(&args);
    // Never hand host secrets (tokens, keys, XCELERATE_PLUGIN_ALLOW) to the child.
    command.env_clear();
    for key in ENV_ALLOW {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        // Do not let the plugin forge host log lines or inject terminal escapes.
        .stderr(Stdio::null());

    let mut child = command.spawn().map_err(|error| {
        XcelerateError::NotFound(format!(
            "could not start plugin '{}': {error}",
            program.display()
        ))
    })?;

    let stdin = match child.stdin.take() {
        Some(stdin) => stdin,
        None => {
            kill_and_wait(&mut child);
            return Err(XcelerateError::InternalError);
        }
    };
    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            kill_and_wait(&mut child);
            return Err(XcelerateError::InternalError);
        }
    };

    let (writer_tx, writer_rx) = mpsc::channel::<String>();
    std::thread::spawn(move || writer_loop(stdin, writer_rx));

    let max_frame = (manifest.limits.max_response_bytes.max(1) as u64).min(MAX_FRAME_BYTES);
    let channel = Arc::new(ProcessChannel {
        name: manifest.name.clone(),
        writer: Mutex::new(writer_tx),
        pending: Mutex::new(HashMap::new()),
        next_id: AtomicU64::new(id_seed()),
        client,
        runtime: Mutex::new(None),
        granted: granted_capabilities(manifest, &allow_list()),
        budget: Duration::from_millis(manifest.limits.max_invoke_millis.max(1)),
        max_frame,
    });

    let describe = json!({
        "jsonrpc": "2.0",
        "id": 0,
        "method": "describe",
        "params": { "manifest": manifest },
    });
    if let Err(message) = channel.send(&describe.to_string()) {
        kill_and_wait(&mut child);
        return Err(XcelerateError::Unsupported(message));
    }

    let reader = match handshake(stdout, manifest, max_frame) {
        Ok(reader) => reader,
        Err(message) => {
            kill_and_wait(&mut child);
            return Err(XcelerateError::Unsupported(message));
        }
    };

    let reader_channel = Arc::clone(&channel);
    std::thread::spawn(move || reader_loop(reader, reader_channel));

    let child = Mutex::new(child);

    Ok(Arc::new(OutOfProcessPlugin {
        manifest: manifest.clone(),
        ops: manifest.ops.clone(),
        channel,
        child,
    }))
}

/// Write every queued line to the plugin's stdin.
fn writer_loop(mut stdin: ChildStdin, rx: mpsc::Receiver<String>) {
    for line in rx {
        if stdin.write_all(line.as_bytes()).is_err() {
            break;
        }
        let _ = stdin.flush();
    }
}

/// Read the `describe` response, returning the reader for the ongoing loop.
///
/// The read runs on a short-lived thread so a stuck plugin cannot hang the host.
fn handshake(
    stdout: ChildStdout,
    manifest: &Manifest,
    max_frame: u64,
) -> Result<BufReader<ChildStdout>, String> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        let outcome = read_frame(&mut reader, max_frame, &mut line);
        let _ = tx.send((outcome, line, reader));
    });

    let (outcome, line, reader) = rx
        .recv_timeout(HANDSHAKE_TIMEOUT)
        .map_err(|_| "plugin handshake timed out".to_string())?;
    let read = outcome.map_err(|error| format!("plugin handshake failed: {error}"))?;
    if read == 0 {
        return Err("plugin exited before the handshake".to_string());
    }

    let response: Value = serde_json::from_str(line.trim())
        .map_err(|error| format!("bad handshake JSON: {error}"))?;
    if let Some(error) = response.get("error") {
        return Err(format!(
            "plugin rejected the handshake: {}",
            error
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("unknown error")
        ));
    }
    let result = response
        .get("result")
        .ok_or_else(|| "handshake response had no result".to_string())?;

    let abi = result.get("abi").and_then(Value::as_str).unwrap_or("");
    if abi != RPC_ABI {
        return Err(format!("plugin speaks abi '{abi}', expected '{RPC_ABI}'"));
    }
    // The plugin must match the manifest it was loaded from - otherwise the
    // audited name/ops could diverge from what actually runs.
    let name = result.get("name").and_then(Value::as_str).unwrap_or("");
    if name != manifest.name {
        return Err(format!(
            "plugin reported name '{name}', but the manifest declares '{}'",
            manifest.name
        ));
    }
    let mut described: Vec<String> = result
        .get("ops")
        .and_then(Value::as_array)
        .map(|ops| {
            ops.iter()
                .filter_map(|op| op.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    described.sort();
    let mut declared = manifest.ops.clone();
    declared.sort();
    if described != declared {
        return Err("plugin ops do not match the manifest".to_string());
    }

    Ok(reader)
}

/// Route plugin output: responses go to their waiter, host calls are served.
fn reader_loop(mut reader: BufReader<ChildStdout>, channel: Arc<ProcessChannel>) {
    let mut line = String::new();
    loop {
        match read_frame(&mut reader, channel.max_frame, &mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let value: Value = match serde_json::from_str(trimmed) {
            Ok(value) => value,
            Err(_) => continue,
        };

        if value.get("method").is_some() {
            // A plugin-to-host call.
            let id = value.get("id").and_then(Value::as_u64);
            let method = value
                .get("method")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let params = value.get("params").cloned().unwrap_or(json!({}));
            let response = match channel.serve(method, &params) {
                Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
                Err(message) => json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": { "code": -32000, "message": message },
                }),
            };
            let _ = channel.send(&response.to_string());
        } else if let Some(id) = value.get("id").and_then(Value::as_u64) {
            // A response to an `invoke` (ids are non-negative integers).
            if let Some(waiter) = channel.pending.lock().unwrap().remove(&id) {
                let result = match value.get("error") {
                    Some(error) => Err(error
                        .get("message")
                        .and_then(Value::as_str)
                        .unwrap_or("plugin error")
                        .to_string()),
                    None => Ok(value.get("result").cloned().unwrap_or(Value::Null)),
                };
                let _ = waiter.send(result);
            }
        }
    }
    // Unblock anything still waiting.
    let mut pending = channel.pending.lock().unwrap();
    for (_, waiter) in pending.drain() {
        let _ = waiter.send(Err("plugin exited".to_string()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU32;

    static TEMP_SEQ: AtomicU32 = AtomicU32::new(0);

    fn python_available() -> bool {
        resolve_interpreter("python").is_some()
    }

    fn example_plugin() -> PathBuf {
        PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../docs/plugins/examples/echo/plugin.json"
        ))
    }

    fn temp_dir() -> PathBuf {
        let seq = TEMP_SEQ.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "xcelerate-plugin-test-{}-{seq}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn manifest_with(entrypoint: &str, capabilities: Vec<Capability>) -> Manifest {
        Manifest {
            name: "example.test".to_string(),
            version: "0.1.0".to_string(),
            tier: Tier::ThirdParty,
            host_api: "1.x".to_string(),
            entrypoint: Some(entrypoint.to_string()),
            abi: Some(RPC_ABI.to_string()),
            ops: vec!["ping".to_string()],
            capabilities,
            limits: Default::default(),
        }
    }

    #[test]
    fn resolves_manifest_and_entrypoint() {
        let manifest_path = example_plugin();
        assert_eq!(
            resolve_manifest_path(manifest_path.to_str().unwrap()).unwrap(),
            manifest_path
        );

        let manifest = Manifest::load(manifest_path.to_str().unwrap()).unwrap();
        let entrypoint = resolve_entrypoint(&manifest_path, &manifest).unwrap();
        assert!(
            entrypoint.exists(),
            "entrypoint should resolve to a real file"
        );
    }

    #[test]
    fn rejects_entrypoint_that_escapes_the_plugin_directory() {
        let base = temp_dir();
        let plugin_dir = base.join("plugin");
        std::fs::create_dir_all(&plugin_dir).unwrap();
        std::fs::write(base.join("escape.py"), "print('escaped')").unwrap();
        let manifest_path = plugin_dir.join("plugin.json");
        std::fs::write(&manifest_path, "{}").unwrap();

        let manifest = manifest_with("../escape.py", vec![]);
        let error = resolve_entrypoint(&manifest_path, &manifest).unwrap_err();
        assert!(
            error.to_string().contains("escapes the plugin directory"),
            "unexpected error: {error}"
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn rejects_absolute_entrypoint_outside_the_plugin_directory() {
        let base = temp_dir();
        let plugin_dir = base.join("plugin");
        std::fs::create_dir_all(&plugin_dir).unwrap();
        let outside = base.join("outside.py");
        std::fs::write(&outside, "print('outside')").unwrap();
        let manifest_path = plugin_dir.join("plugin.json");
        std::fs::write(&manifest_path, "{}").unwrap();

        let manifest = manifest_with(outside.to_str().unwrap(), vec![]);
        assert!(resolve_entrypoint(&manifest_path, &manifest).is_err());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn first_party_only_capabilities_are_never_granted() {
        let manifest = manifest_with(
            "x.py",
            vec![Capability::LaunchControl, Capability::BinaryPatch],
        );
        let granted = granted_capabilities(&manifest, &HashSet::new());
        assert!(
            granted.is_empty(),
            "first-party-only caps must never be granted"
        );
    }

    #[test]
    fn dangerous_capabilities_need_an_explicit_grant() {
        let manifest = manifest_with("x.py", vec![Capability::ReadCookies]);
        assert!(granted_capabilities(&manifest, &HashSet::new()).is_empty());

        let mut allow = HashSet::new();
        allow.insert("example.test:read_cookies".to_string());
        assert!(granted_capabilities(&manifest, &allow).contains(&Capability::ReadCookies));
    }

    #[test]
    fn cdp_proxy_does_not_substitute_for_narrower_capabilities() {
        assert_eq!(
            capability_for_cdp("Storage.getCookies"),
            Some(Capability::ReadCookies)
        );
        assert_eq!(
            capability_for_cdp("Runtime.evaluate"),
            Some(Capability::Evaluate)
        );
        assert_eq!(capability_for_cdp("Page.navigate"), None);
    }

    #[tokio::test]
    async fn invokes_a_python_plugin_out_of_process() {
        if !python_available() {
            eprintln!("skipping: python is not available");
            return;
        }
        let manifest_path = example_plugin();
        let manifest = Manifest::load(manifest_path.to_str().unwrap()).unwrap();
        let plugin = spawn(&manifest, &manifest_path, None).unwrap();

        let manager =
            xcelerate_plugin_api::PluginManager::new(&[], Arc::new(|_name: &str| None)).unwrap();
        manager.install(plugin).unwrap();

        let output = manager
            .invoke("example.echo", "echo", r#"{"value":42}"#.to_string(), None)
            .await
            .unwrap();
        assert!(output.contains("42"), "unexpected output: {output}");
    }
}
