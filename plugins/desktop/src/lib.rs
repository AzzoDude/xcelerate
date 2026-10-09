//! `desktop` - native window control as a WebAssembly plugin.
//!
//! The plugin owns the **verb logic**; the host owns the **protocol**. Ops that
//! map 1:1 onto a host primitive (`windows`, `launch`, `set_value`, `close`,
//! `key`, `wheel`, `scroll`) are forwarded; the composite verbs (`tree`, `find`,
//! `wait`, `click`) are implemented here by composing primitives (`snapshot`,
//! `click-index`, `sleep`). The guest never touches the OS: UI Automation stays
//! in the host, and the guest only asks for a snapshot or an act.
//!
//! Build: `xcelerate build --wasm-only` (run in this directory) stages
//! `desktop.wasm` next to this manifest. Then drop the directory in
//! `~/.xcl/plugins/` and load it by name (`desktop`). The bridge is Windows only.

wit_bindgen::generate!({
    path: "wit/plugin.wit",
    world: "plugin-world",
});

use xcelerate::plugin::host;
use xcelerate_plugin::guest::{decode, encode};
use xcelerate_plugin::plugin;

struct Desktop;

plugin! {
    guest = Desktop,
    name = "desktop",
    bridge = desktop,
    // Composite verbs: the logic lives in this crate (see below).
    local = { tree, find, wait, click },
    ops = {
        windows,
        launch,
        // `set_value` forwards to the host verb `set-value`.
        set_value => "set-value",
        close,
        key,
        wheel,
        scroll,
    },
}

export!(Desktop);

// ---------------------------------------------------------------------------
// Verb logic, composed over host primitives
// ---------------------------------------------------------------------------

/// How many elements a snapshot walks; matches the host default.
const DEFAULT_LIMIT: usize = 400;
/// How long `wait` sleeps between polls.
const POLL_MILLIS: u64 = 300;

/// One element of a host snapshot.
#[derive(serde::Deserialize)]
#[allow(dead_code)]
struct Element {
    index: usize,
    role: String,
    name: String,
    value: String,
    rect: (i64, i64, i64, i64),
}

/// A host snapshot result.
#[derive(serde::Deserialize)]
struct Snapshot {
    elements: Vec<Element>,
}

/// `tree` request: which window to walk.
#[derive(serde::Deserialize)]
struct WindowRequest {
    window: String,
}

/// `find` request: which window, and the needle.
#[derive(serde::Deserialize)]
struct FindRequest {
    window: String,
    text: String,
}

/// `wait` request: which window, the needle, and the timeout.
#[derive(serde::Deserialize)]
struct WaitRequest {
    window: String,
    text: String,
    timeout_ms: u64,
}

/// `click` request: an element index, or a name to resolve.
#[derive(serde::Deserialize)]
struct ClickRequest {
    window: String,
    index: Option<usize>,
    name: Option<String>,
}

/// A `{ lines: [...] }` reply, as the interpreter expects for `tree`/`find`.
#[derive(serde::Serialize)]
struct Lines {
    lines: Vec<String>,
}

/// A `{ found: "..." }` reply, as the interpreter expects for `wait`.
#[derive(serde::Serialize)]
struct Found {
    found: String,
}

/// A `{ method, index }` reply, as the interpreter expects for `click`.
#[derive(serde::Serialize)]
struct Clicked {
    method: String,
    index: Option<usize>,
}

/// The host's `click-index` result.
#[derive(serde::Deserialize)]
struct MethodResponse {
    method: String,
}

/// Take a snapshot of `window`'s elements through `host.desktop`.
fn snapshot(window: &str) -> Result<Vec<Element>, String> {
    #[derive(serde::Serialize)]
    struct Args<'a> {
        window: &'a str,
        limit: usize,
    }
    let reply = host::desktop(
        "snapshot",
        &encode(&Args {
            window,
            limit: DEFAULT_LIMIT,
        })?,
    )?;
    Ok(decode::<Snapshot>(&reply)?.elements)
}

/// Block the guest for `ms` via the host std sleep primitive (`host.core`): the
/// guest has no clock of its own, and the timer belongs to `core`, not `app`.
fn sleep(ms: u64) -> Result<(), String> {
    #[derive(serde::Serialize)]
    struct Args {
        ms: u64,
    }
    let _ = host::core("sleep", &encode(&Args { ms })?)?;
    Ok(())
}

/// One-line element rendering (`[index] <role> "name"  WxH@X,Y`).
fn format_element(element: &Element) -> String {
    let (x, y, w, h) = element.rect;
    format!(
        "[{}] <{}> {:?}  {}x{}@{},{}",
        element.index, element.role, element.name, w, h, x, y
    )
}

/// The index of the first element whose name contains `needle`, preferring a
/// `button` (stable across animated frames). This is the selection heuristic -
/// verb logic, so it lives here rather than in the host.
fn find_element(elements: &[Element], needle: &str) -> Option<usize> {
    let needle = needle.to_ascii_lowercase();
    elements
        .iter()
        .find(|e| e.role == "button" && e.name.to_ascii_lowercase().contains(&needle))
        .or_else(|| {
            elements
                .iter()
                .find(|e| e.name.to_ascii_lowercase().contains(&needle))
        })
        .map(|e| e.index)
}

/// `tree` - walk the window and render every element as a line.
fn tree(args: &[u8]) -> Result<Vec<u8>, String> {
    let request: WindowRequest = decode(args)?;
    let lines = snapshot(&request.window)?
        .iter()
        .map(format_element)
        .collect();
    encode(&Lines { lines })
}

/// `find` - the lines whose element name contains the needle.
fn find(args: &[u8]) -> Result<Vec<u8>, String> {
    let request: FindRequest = decode(args)?;
    let needle = request.text.to_ascii_lowercase();
    let lines = snapshot(&request.window)?
        .iter()
        .filter(|element| element.name.to_ascii_lowercase().contains(&needle))
        .map(format_element)
        .collect();
    encode(&Lines { lines })
}

/// `wait` - poll the snapshot until the needle appears or the timeout elapses.
fn wait(args: &[u8]) -> Result<Vec<u8>, String> {
    let request: WaitRequest = decode(args)?;
    let needle = request.text.to_ascii_lowercase();
    let start = std::time::Instant::now();
    loop {
        let elements = snapshot(&request.window)?;
        if let Some(hit) = elements
            .iter()
            .find(|element| element.name.to_ascii_lowercase().contains(&needle))
        {
            return encode(&Found {
                found: hit.name.clone(),
            });
        }
        if start.elapsed().as_millis() as u64 >= request.timeout_ms {
            return Err(format!("wait for {:?} timed out", request.text));
        }
        sleep(POLL_MILLIS)?;
    }
}

/// `click` - act on an element by index, or resolve a name first.
fn click(args: &[u8]) -> Result<Vec<u8>, String> {
    let request: ClickRequest = decode(args)?;
    let index = match (request.index, request.name.as_deref()) {
        (Some(index), _) => index,
        (None, Some(name)) => find_element(&snapshot(&request.window)?, name)
            .ok_or_else(|| format!("no element matching {name:?}"))?,
        (None, None) => return Err("click needs an index or a name".to_string()),
    };

    #[derive(serde::Serialize)]
    struct Args<'a> {
        window: &'a str,
        index: usize,
    }
    let reply = host::desktop(
        "click-index",
        &encode(&Args {
            window: &request.window,
            index,
        })?,
    )?;
    let outcome: MethodResponse = decode(&reply)?;
    encode(&Clicked {
        method: outcome.method,
        index: Some(index),
    })
}
