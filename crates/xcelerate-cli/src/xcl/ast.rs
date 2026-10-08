//! The XCL abstract syntax: a flat, one-action-per-line command model.
//!
//! There is deliberately no expression tree, no nested blocks, and no value
//! stack. A program is an ordered list of [`Step`]s; control flow lowers to
//! bounded jumps ([`Command::Goto`] / [`Command::Label`]) and guarded single
//! statements, so execution never needs a call stack beyond one function frame.

/// A single parsed argument: either a literal token or a variable reference to
/// interpolate at run time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Arg {
    /// A literal string (already unquoted).
    Literal(String),
    /// `$name` — interpolate the variable at run time.
    Var(String),
    /// A builtin such as `{BASE_URL}`, `{TIMESTAMP}`, `{UUID}`.
    Builtin(String),
}

impl Arg {
    /// Renders the argument (interpolating vars/builtins) for a human/`--codegen`
    /// view. `resolve` is the authoritative interpolation; this is the lossless
    /// echo form used in transcripts.
    pub fn source(&self) -> String {
        match self {
            Arg::Literal(s) => quote_if_needed(s),
            Arg::Var(name) => format!("${name}"),
            Arg::Builtin(name) => format!("{{{name}}}"),
        }
    }
}

/// A function parameter: just a name (positional in the `func` signature).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamDef {
    pub name: String,
}

/// A named, bounded, non-recursive callable. Functions *perform* actions; they
/// never return a value. Depth is at most one (a function body may not contain
/// `func` or `call`), and they must be defined before use.
#[derive(Debug, Clone)]
pub struct FuncDef {
    pub name: String,
    pub params: Vec<ParamDef>,
    /// The parameterized body as source steps; the `call` site substitutes
    /// arguments into `Arg::Var` positions before execution. Stored as already-
    /// parsed steps for cheap re-instantiation.
    pub body: Vec<Step>,
}

/// A single executable command.
#[derive(Debug, Clone)]
pub enum Command {
    // --- definitions -------------------------------------------------------
    /// `let <name> <value>` — define a read-only variable.
    Let { name: String, value: Arg },
    /// `set <name> <value>` — (re)assign a variable.
    Set { name: String, value: Arg },
    /// `param <name> [default]` — declare a runtime parameter.
    Param { name: String, default: Option<Arg> },
    /// `print <arg>...` — write the resolved arguments to stdout (the explicit
    /// log channel; everything else is silent on success).
    Print { args: Vec<Arg> },
    /// `func <name>(a, b, ...)` — begin a function definition (implicit `end`).
    FuncStart(FuncDef),
    /// `end` — terminates the current `func` block.
    FuncEnd,
    /// `<name> <arg>...` (or `call <name> <arg>...`) — invoke a defined
    /// function (depth ≤ 1). The `call` keyword is optional.
    Call { name: String, args: Vec<Arg> },

    // --- plugins / workers ------------------------------------------------
    /// `import <plugin-id>` — load a plugin by id.
    Import { name: String },
    /// `run <plugin> <op> [json]` — invoke a plugin op (worker).
    Run {
        plugin: String,
        op: String,
        json: Option<Arg>,
    },
    /// `plugins` — list loaded workers.
    Plugins,
    /// `plugin-config <name> [op]` — dump op schemas/defaults.
    PluginConfig { name: String, op: Option<String> },

    // --- browserless HTTP -------------------------------------------------
    /// `request <METHOD> <url> [headers] [body]`.
    Request {
        method: String,
        url: Arg,
        headers: Option<Arg>,
        body: Option<Arg>,
    },

    // --- control flow -----------------------------------------------------
    /// `repeat <n> <verb> <args...>` — run a single statement `n` times.
    Repeat { n: u32, inner: Box<Step> },
    /// `retry <n> <verb> <args...>` — repeat a statement until success.
    Retry { n: u32, inner: Box<Step> },
    /// `if-ok <verb> <args...>` — run only if the previous statement succeeded.
    IfOk(Box<Step>),
    /// `if-fail <verb> <args...>` — run only if the previous statement failed.
    IfFail(Box<Step>),
    /// `label <name>` — a jump target.
    Label { name: String },
    /// `goto <name>` — jump to a label.
    Goto { name: String },
    /// `assert <subject> <op> <value>` — structured check.
    Assert {
        subject: String,
        op: String,
        value: Arg,
    },

    // --- browser pass-through --------------------------------------------
    /// Any browser/session verb not needing special lowering; holds the verb
    /// name and its raw arguments verbatim so the engine can dispatch to the
    /// shared command table.
    Raw { verb: String, args: Vec<Arg> },

    /// `done` — mark the task complete and end the run.
    Done,
    /// `quit` — end the run early.
    Quit,
}

/// A parsed line: a command plus its 1-based source line number.
#[derive(Debug, Clone)]
pub struct Step {
    pub line: usize,
    pub command: Command,
}

/// Quotes a value with spaces (or that is otherwise ambiguous) for echo output.
fn quote_if_needed(s: &str) -> String {
    if s.is_empty()
        || s.contains(char::is_whitespace)
        || s.contains('$')
        || s.contains('{')
        || s.contains('"')
        || s.contains('\'')
    {
        format!("{s:?}")
    } else {
        s.to_string()
    }
}
