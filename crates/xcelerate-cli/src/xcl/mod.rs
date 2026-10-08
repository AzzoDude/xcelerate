//! XCL — the Xcelerate Command Language.
//!
//! A line-oriented, imperative DSL for authoring browser-automation runs that is
//! equally comfortable for an AI agent to emit and a human to read. The design
//! contract is *one line = one action*, bounded control flow, and a small set of
//! explicit verbs — nothing that requires a call stack to reason about.
//!
//! This module holds the language proper (parse → program → execute); it is
//! deliberately free of the browser/transport so the same command table can back
//! both the interactive session REPL and a `.xcl` file runner.

// The language is integrated ahead of the surfaces that consume it (the
// interactive session REPL and the AI/CLI provenance tiers), so a few helpers and
// enum variants are defined but not yet wired up. Keep them rather than deleting a
// deliberate API; silence the dead-code lint for those modules only.
#[allow(dead_code)]
pub mod ast;
#[allow(dead_code)]
pub mod engine;
pub mod exec;
pub mod lex;
pub mod parse;
pub mod run;
#[allow(dead_code)]
pub mod runtime;
#[allow(dead_code)]
pub mod security;

pub use engine::Engine;
pub use run::run_file;
