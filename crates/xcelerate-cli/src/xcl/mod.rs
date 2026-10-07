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

pub mod ast;
pub mod engine;
pub mod exec;
pub mod lex;
pub mod parse;
pub mod run;
pub mod runtime;
pub mod security;

pub use ast::{Arg, Command, FuncDef, Step};
pub use engine::Engine;
pub use parse::{ParseError, parse_program};
pub use runtime::{Context, Outcome, RuntimeLimits};

pub use run::run_file;
