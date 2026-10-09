//! `core` - the language's **standard library**, as a WebAssembly plugin.
//!
//! The interpreter understands only the language; the std verbs are ordinary
//! functions that live here and compose the host's raw primitives
//! (`host.core`: `stdout`, `now`, `env`, `sleep`, `random`). This is the same
//! split as `browser` and `app`: the host keeps the primitive, the plugin keeps
//! the verb logic.
//!
//! Build: `xcelerate build --wasm-only` (run in this directory) stages `core.wasm`
//! next to this manifest. Then drop the directory in `~/.xcl/plugins/` and load
//! it by name (`core`).

wit_bindgen::generate!({
    path: "wit/plugin.wit",
    world: "plugin-world",
});

use serde::{Deserialize, Serialize};
use xcelerate::plugin::host;
use xcelerate_plugin::guest::{decode, encode};

/// The std ops this plugin answers (the interpreter relays these by name).
const OPS: &[&str] = &[
    "print",
    "now",
    "env",
    "sleep",
    "await",
    "wait-ms",
    "wait-sec",
    "wait-min",
    "wait-hr",
    "wait-random",
];

struct Core;

impl exports::xcelerate::plugin::plugin::Guest for Core {
    fn describe() -> Vec<u8> {
        #[derive(Serialize)]
        struct Describe<'a> {
            name: &'a str,
            version: &'a str,
            ops: Vec<&'a str>,
        }
        let describe = Describe {
            name: "core",
            version: env!("CARGO_PKG_VERSION"),
            ops: OPS.to_vec(),
        };
        rmp_serde::to_vec_named(&describe).unwrap_or_default()
    }

    fn invoke(op: String, args: Vec<u8>) -> Result<Vec<u8>, String> {
        let request: Request = decode(&args)?;
        encode(&run(&op, &request)?)
    }
}

export!(Core);

// ---------------------------------------------------------------------------
// Std functions
// ---------------------------------------------------------------------------

/// What the interpreter sends: the resolved positional arguments.
#[derive(Deserialize)]
struct Request {
    #[serde(default)]
    args: Vec<String>,
}

fn arg(request: &Request, index: usize) -> String {
    request.args.get(index).cloned().unwrap_or_default()
}

/// `host.core("stdout")` - write one line.
fn stdout(message: &str) -> Result<(), String> {
    #[derive(Serialize)]
    struct Args<'a> {
        message: &'a str,
    }
    let _ = host::core("stdout", &encode(&Args { message })?)?;
    Ok(())
}

/// `host.core("sleep")` - the host blocks the guest thread for `ms`.
fn sleep_ms(ms: u64) -> Result<(), String> {
    #[derive(Serialize)]
    struct Args {
        ms: u64,
    }
    let _ = host::core("sleep", &encode(&Args { ms })?)?;
    Ok(())
}

/// The whole verb table.
fn run(op: &str, request: &Request) -> Result<String, String> {
    match op {
        "print" => {
            let message = request.args.join(" ");
            stdout(&message)?;
            Ok(message)
        }
        "now" => {
            #[derive(Serialize)]
            struct Empty {}
            #[derive(Deserialize)]
            struct Out {
                seconds: u64,
            }
            let reply = host::core("now", &encode(&Empty {})?)?;
            let out: Out = decode(&reply)?;
            Ok(out.seconds.to_string())
        }
        "env" => {
            #[derive(Serialize)]
            struct Args {
                name: String,
            }
            #[derive(Deserialize)]
            struct Out {
                name: String,
                value: Option<String>,
            }
            let reply = host::core(
                "env",
                &encode(&Args {
                    name: arg(request, 0),
                })?,
            )?;
            let out: Out = decode(&reply)?;
            let _ = out.name;
            Ok(out.value.unwrap_or_default())
        }

        // Time. `sleep`/`await` take a plain number; the unit verbs carry the
        // unit in the name (there is no `2s`/`500ms` literal in the language).
        "sleep" => {
            let ms = number(request, "milliseconds")?;
            sleep_ms(ms)?;
            Ok(format!("waited {ms}ms"))
        }
        "await" => {
            let seconds: f64 = arg(request, 0)
                .trim()
                .parse()
                .map_err(|_| "await <seconds>: seconds must be a number".to_string())?;
            let seconds = seconds.clamp(0.0, 3600.0);
            sleep_ms((seconds * 1000.0) as u64)?;
            Ok(format!("awaited {seconds}s"))
        }
        "wait-ms" => scaled(request, 1),
        "wait-sec" => scaled(request, 1_000),
        "wait-min" => scaled(request, 60_000),
        "wait-hr" => scaled(request, 3_600_000),
        "wait-random" => {
            let min = signed(request, 0)?;
            let max = signed(request, 1)?;
            #[derive(Serialize)]
            struct Args {
                min: i64,
                max: i64,
            }
            #[derive(Deserialize)]
            struct Out {
                ms: i64,
            }
            let reply = host::core("random", &encode(&Args { min, max })?)?;
            let out: Out = decode(&reply)?;
            let ms = out.ms.max(0) as u64;
            sleep_ms(ms)?;
            Ok(format!("wait-random {ms}ms"))
        }

        other => Err(format!("core: unknown op '{other}'")),
    }
}

fn number(request: &Request, what: &str) -> Result<u64, String> {
    arg(request, 0)
        .trim()
        .parse()
        .map_err(|_| format!("expected a number of {what}"))
}

fn signed(request: &Request, index: usize) -> Result<i64, String> {
    arg(request, index)
        .trim()
        .parse()
        .map_err(|_| "expected a number of milliseconds".to_string())
}

/// `wait-<unit> <n>` - sleep `n * unit_ms`, capped at one hour.
fn scaled(request: &Request, unit_ms: u64) -> Result<String, String> {
    let count = number(request, "units")?;
    let ms = count.saturating_mul(unit_ms).min(3_600_000);
    sleep_ms(ms)?;
    Ok(format!("waited {ms}ms"))
}
