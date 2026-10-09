//! # xcelerate-interpreter
//!
//! The **XCL** language (the Xcelerate Command Language): a line-oriented,
//! imperative DSL for authoring browser-automation runs that is equally
//! comfortable for an AI agent to emit and a human to read. The design contract
//! is *one line = one action*, bounded control flow, and a small set of explicit
//! verbs — nothing that requires a call stack to reason about.
//!
//! The pipeline is `lex` → `parse` → [`Engine`] → `exec`:
//!
//! - [`lex`] turns a line into tokens (quotes group, `#` starts a comment).
//! - [`parse`] builds a [`parse::Program`] of steps, lowering control flow to
//!   bounded jumps.
//! - [`engine::Engine`] walks the program, enforces the hard step budget, and
//!   yields one [`ast::Command`] at a time.
//! - [`exec`] dispatches each command onto the browser, plugins, and HTTP.
//!
//! This crate is deliberately free of the CLI's surface (clap, launch, codegen):
//! it exposes the language and its execution over a live [`xcelerate::Page`], so
//! the same command table backs both the `.xcl` file runner and the interactive
//! session REPL.

pub mod ast;
pub mod engine;
pub mod exec;
pub mod lex;
#[cfg(feature = "http")]
pub mod net;
pub mod parse;
pub mod runtime;
pub mod security;

pub use engine::Engine;
