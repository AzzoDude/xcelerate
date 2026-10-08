//! `xcelerate-uia` — drive any native window (Windows UI Automation).
//!
//! One-shot commands, a persistent `session`, and a warm `serve` daemon:
//!
//! ```text
//! xcelerate-uia apps                    # list windows
//! xcelerate-uia find "Store" "Get"      # matching elements only
//! xcelerate-uia click-name "Store" "Get"
//! xcelerate-uia session "Store" < cmds  # many actions, one process
//! xcelerate-uia serve                   # daemon on 127.0.0.1:9417 (warm UIA)
//! xcelerate-uia send < cmds             # run cmds on the daemon
//! ```

use std::error::Error;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::process::ExitCode;

use xcelerate_uia::{Element, Uia};

const USAGE: &str = "usage: xcelerate-uia <apps | tree <t> | find <t> <text> | wait <t> <text> [ms] \
    | click <t> <i> | click-name <t> <text> | set-value <t> <i> <text> \
    | key <t> <name> | wheel <t> <n> | scroll <t> <n> | session <t> | serve [port] | send [port]>";

const DEFAULT_PORT: u16 = 9417;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String]) -> Result<(), Box<dyn Error>> {
    let command = args.first().map(String::as_str).unwrap_or("");
    if command == "apps" {
        for window in Uia::new()?.windows()? {
            println!("{}", xcelerate_uia::format_window(&window));
        }
        return Ok(());
    }
    if command == "serve" {
        return serve(port_arg(args));
    }
    if command == "send" {
        return send(port_arg(args));
    }

    let title = args.get(1).ok_or(USAGE)?;
    let mut uia = Uia::new()?;
    if command == "session" {
        return session(&mut uia, title);
    }

    // One-shot: attach, then run the *same* command core the session and the
    // daemon use, so the two paths can never drift apart.
    let mut st = State::default();
    attach(&mut st, title);
    let line = std::iter::once(command)
        .chain(args[2..].iter().map(String::as_str))
        .collect::<Vec<_>>()
        .join(" ");
    if let Step::Reply(text) = exec(&mut uia, &mut st, &line)? {
        println!("{text}");
    }
    Ok(())
}

fn port_arg(args: &[String]) -> u16 {
    args.get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_PORT)
}

// ---------------------------------------------------------------------------
// Shared command core (used by `session` and the daemon).
// ---------------------------------------------------------------------------

/// Result of one command line.
enum Step {
    Reply(String),
    Quiet,
    Quit,
}

/// Mutable state carried across commands in a session or daemon connection.
#[derive(Default)]
struct State {
    title: String,
    infos: Vec<Element>,
    last_names: Vec<String>,
}

fn attach(st: &mut State, title: &str) {
    st.title = title.to_string();
    st.infos.clear();
    st.last_names.clear();
}

/// Snapshot the attached window **once**; every later command reuses it. Kept
/// lazy so commands that never touch the tree (`key`, `wheel`) cost no snapshot.
fn ensure_infos(uia: &mut Uia, st: &mut State) -> Result<(), Box<dyn Error>> {
    if st.infos.is_empty() {
        st.infos = uia.snapshot(&st.title, 400)?;
        st.last_names = names(&st.infos);
    }
    Ok(())
}

fn names(infos: &[Element]) -> Vec<String> {
    infos.iter().map(|e| e.name.clone()).collect()
}

/// Runs one command line against `st`. `title` must already be attached.
fn exec(uia: &mut Uia, st: &mut State, line: &str) -> Result<Step, Box<dyn Error>> {
    let (command, rest) = line
        .trim()
        .split_once(' ')
        .map_or((line.trim(), ""), |(c, r)| (c.trim(), r.trim()));
    let title = st.title.clone();
    let step = match command {
        "" => Step::Quiet,
        "attach" => {
            attach(st, rest);
            ensure_infos(uia, st)?;
            Step::Reply(format!("attached to {rest:?}: {} elements", st.infos.len()))
        }
        "tree" => {
            st.infos = uia.snapshot(&title, 400)?;
            st.last_names = names(&st.infos);
            Step::Reply(
                st.infos
                    .iter()
                    .map(xcelerate_uia::format_element)
                    .collect::<Vec<_>>()
                    .join("\n"),
            )
        }
        "find" => {
            ensure_infos(uia, st)?;
            let needle = rest.to_ascii_lowercase();
            let hits: Vec<String> = st
                .infos
                .iter()
                .filter(|e| e.name.to_ascii_lowercase().contains(&needle))
                .map(xcelerate_uia::format_element)
                .collect();
            Step::Reply(if hits.is_empty() {
                "(no match)".into()
            } else {
                hits.join("\n")
            })
        }
        "wait" => {
            let (text, ms) = match rest.rsplit_once(' ') {
                Some((t, m)) if m.parse::<u64>().is_ok() => (t.trim(), m.parse().unwrap()),
                _ => (rest, 10_000),
            };
            let loaded = !st.infos.is_empty();
            let hit = uia.wait_for(&title, text, ms)?;
            if loaded {
                st.infos = uia.snapshot(&title, 400)?;
                st.last_names = names(&st.infos);
            }
            Step::Reply(format!("found {hit:?}"))
        }
        "diff" => {
            st.infos = uia.snapshot(&title, 400)?;
            let now = names(&st.infos);
            let mut out = Vec::new();
            for n in now
                .iter()
                .filter(|n| !n.is_empty() && !st.last_names.contains(n))
            {
                out.push(format!("+ {n:?}"));
            }
            for n in st
                .last_names
                .iter()
                .filter(|n| !n.is_empty() && !now.contains(n))
            {
                out.push(format!("- {n:?}"));
            }
            st.last_names = now;
            Step::Reply(if out.is_empty() {
                "(no change)".into()
            } else {
                out.join("\n")
            })
        }
        "click" => {
            ensure_infos(uia, st)?;
            let index: usize = rest.parse()?;
            Step::Reply(format!("clicked [{index}] via {}", uia.click(index)?))
        }
        "click-name" => {
            ensure_infos(uia, st)?;
            let index = xcelerate_uia::find_element(&st.infos, rest)
                .ok_or_else(|| format!("no element {rest:?}"))?;
            Step::Reply(format!(
                "clicked [{index}] via {} {rest:?}",
                uia.click(index)?
            ))
        }
        "set-value" => {
            let (index, text) = rest
                .split_once(' ')
                .ok_or("usage: set-value <index> <text>")?;
            let index: usize = index.trim().parse()?;
            ensure_infos(uia, st)?;
            uia.set_value(index, text)?;
            Step::Reply(format!("set [{index}]"))
        }
        "scroll" => {
            let notches: i32 = rest.parse()?;
            Step::Reply(format!(
                "scrolled {notches} via {}",
                uia.scroll(&title, notches)?
            ))
        }
        "key" => {
            uia.key_name(&title, rest)?;
            Step::Reply(format!("key {rest:?}"))
        }
        "wheel" => {
            let notches: i32 = rest.parse()?;
            uia.wheel(&title, notches)?;
            Step::Reply(format!("wheeled {notches}"))
        }
        "quit" => Step::Quit,
        other => Step::Reply(format!("unknown command `{other}`")),
    };
    Ok(step)
}

// ---------------------------------------------------------------------------
// session / daemon / client
// ---------------------------------------------------------------------------

fn session(uia: &mut Uia, title: &str) -> Result<(), Box<dyn Error>> {
    let mut st = State::default();
    attach(&mut st, title);
    ensure_infos(uia, &mut st)?;
    println!(
        "attached to {title:?}: {} elements (one process; `find`, `click-name`, `set-value`, \
         `wait`, `diff`, `quit`)",
        st.infos.len()
    );
    for line in std::io::stdin().lock().lines() {
        match exec(uia, &mut st, &line?)? {
            Step::Reply(text) => println!("{text}"),
            Step::Quiet => {}
            Step::Quit => break,
        }
    }
    Ok(())
}

/// Warm daemon: one `Uia` for every connection.
fn serve(port: u16) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    let mut uia = Uia::new()?;
    eprintln!("xcelerate-uia daemon on 127.0.0.1:{port}");
    for stream in listener.incoming() {
        let Ok(stream) = stream else { continue };
        let _ = handle_client(&mut uia, stream);
    }
    Ok(())
}

fn handle_client(uia: &mut Uia, stream: TcpStream) -> std::io::Result<()> {
    let mut writer = stream.try_clone()?;
    let mut st = State::default();
    for line in BufReader::new(stream).lines() {
        let reply = match exec(uia, &mut st, &line?) {
            Ok(Step::Reply(text)) => text,
            Ok(Step::Quiet) => continue,
            Ok(Step::Quit) => break,
            Err(error) => format!("error: {error}"),
        };
        writeln!(writer, "{reply}")?;
        writer.flush()?;
    }
    Ok(())
}

/// Client: forward stdin to the daemon and print its replies.
fn send(port: u16) -> Result<(), Box<dyn Error>> {
    let mut stream = TcpStream::connect(("127.0.0.1", port))?;
    for line in std::io::stdin().lock().lines() {
        writeln!(stream, "{}", line?)?;
    }
    stream.flush()?;
    for line in BufReader::new(stream).lines() {
        println!("{}", line?);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------
