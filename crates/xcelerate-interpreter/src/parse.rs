//! The XCL parser: tokens → [`Program`] of [`Step`]s.
//!
//! [`parse_program`] is the single entry point. It folds the flat token stream
//! into a validated program: functions are collected (and their bodies kept as
//! steps), labels are resolved to line indices, control-flow verbs are checked
//! for the bounded-loop invariants, and each token is classified into an
//! [`Arg`] (literal / `$var` / `{builtin}`).

use std::collections::HashMap;

use super::ast::{Arg, Callable, Command, FuncDef, ParamDef, Step};
use super::lex::{Line, lex_line};
use super::security::Limits;

/// A parse error with the 1-based line it occurred on.
#[derive(Debug, Clone)]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

impl ParseError {
    fn new(line: usize, message: impl Into<String>) -> Self {
        Self {
            line,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for ParseError {}

/// A fully parsed, validated XCL program.
#[derive(Debug, Clone, Default)]
pub struct Program {
    /// The flattened top-level steps (function bodies are stored in `funcs`,
    /// not inlined here; each [`Command::FuncStart`]/[`Command::FuncEnd`] pair
    /// is removed and the body captured into `funcs` instead).
    pub steps: Vec<Step>,
    /// Callables by `(name, arity)`. Keying on the parameter count lets a name be
    /// overloaded (`human` with two params and `human` with four coexist) while an
    /// exact duplicate is rejected. Values are script `func`s or `import`-bound
    /// plugin ops.
    pub funcs: HashMap<(String, usize), Callable>,
}

/// Parses an entire `.xcl` source into a [`Program`] with the default limits.
pub fn parse_program(source: &str) -> Result<Program, ParseError> {
    Parser::default().run(source)
}

/// Parses an entire `.xcl` source with caller-supplied [`Limits`].
pub fn parse_program_with_limits(source: &str, limits: Limits) -> Result<Program, ParseError> {
    Parser::default().with_limits(limits).run(source)
}

/// Parses an `.xcl` file, resolving `import "*.xcl"` relatives against
/// `base_dir` (the directory of the file being parsed).
pub fn parse_program_file(
    source: &str,
    limits: Limits,
    base_dir: Option<std::path::PathBuf>,
) -> Result<Program, ParseError> {
    Parser::default()
        .with_limits(limits)
        .with_base_dir(base_dir)
        .run(source)
}

/// Maximum `import "*.xcl"` nesting depth.
const MAX_IMPORT_DEPTH: usize = 16;

#[derive(Default)]
struct Parser {
    steps: Vec<Step>,
    funcs: HashMap<(String, usize), Callable>,
    /// When inside a `func ... end`, the function under construction.
    active_func: Option<ActiveFunc>,
    /// Boundedness caps; defaults keep untrusted input safe, a human may raise.
    limits: Limits,
    /// Directory that relative `import "*.xcl"` paths resolve against.
    base_dir: Option<std::path::PathBuf>,
    /// Files currently being imported, for cycle detection.
    imported: Vec<std::path::PathBuf>,
}

struct ActiveFunc {
    def: FuncDef,
    body: Vec<Step>,
}

impl Parser {
    fn with_limits(mut self, limits: Limits) -> Self {
        self.limits = limits;
        self
    }

    fn with_base_dir(mut self, base_dir: Option<std::path::PathBuf>) -> Self {
        self.base_dir = base_dir;
        self
    }

    /// `import "path.xcl"`: parse another script and merge its `func`s, so a
    /// script can share helpers. The import is resolved relative to this file.
    fn import_xcl(&mut self, raw: &str, line: usize) -> Result<(), ParseError> {
        let path = self.resolve_import(raw);
        if self.imported.contains(&path) {
            return Err(ParseError::new(
                line,
                format!("circular import: {}", path.display()),
            ));
        }
        if self.imported.len() >= MAX_IMPORT_DEPTH {
            return Err(ParseError::new(
                line,
                format!("import nesting too deep (max {MAX_IMPORT_DEPTH})"),
            ));
        }
        let source = std::fs::read_to_string(&path).map_err(|error| {
            ParseError::new(
                line,
                format!("cannot read import {}: {error}", path.display()),
            )
        })?;
        let mut imported = self.imported.clone();
        imported.push(path.clone());
        let module = Parser {
            steps: Vec::new(),
            funcs: HashMap::new(),
            active_func: None,
            limits: self.limits,
            base_dir: path.parent().map(|parent| parent.to_path_buf()),
            imported,
        }
        .run(&source)?;
        for (key, callable) in module.funcs {
            if self.funcs.contains_key(&key) {
                return Err(ParseError::new(
                    line,
                    format!(
                        "`{}` is already defined; an imported function name must be unique",
                        key.0
                    ),
                ));
            }
            if self.funcs.len() >= self.limits.max_funcs {
                return Err(ParseError::new(
                    line,
                    format!("too many functions/imports (max {})", self.limits.max_funcs),
                ));
            }
            self.funcs.insert(key, callable);
        }
        Ok(())
    }

    /// Resolves an import path: absolute stays as-is, relative joins `base_dir`
    /// (or the current directory when the script had no known parent).
    fn resolve_import(&self, raw: &str) -> std::path::PathBuf {
        let path = std::path::PathBuf::from(raw);
        if path.is_absolute() {
            return path;
        }
        let base = self
            .base_dir
            .clone()
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        base.join(path)
    }

    fn run(mut self, source: &str) -> Result<Program, ParseError> {
        for (idx, raw) in source.lines().enumerate() {
            let line_no = idx + 1;
            match lex_line(raw).map_err(|e| ParseError::new(line_no, e.to_string()))? {
                Line::Blank | Line::Comment => continue,
                Line::Statement(tokens) => {
                    self.parse_statement(line_no, &tokens)?;
                }
            }
        }
        // Leaving a `func` open is an error.
        if self.active_func.is_some() {
            return Err(ParseError::new(
                source.lines().count(),
                "unterminated `func` (missing `end`)",
            ));
        }
        Ok(Program {
            steps: self.steps,
            funcs: self.funcs,
        })
    }

    fn parse_statement(&mut self, line: usize, tokens: &[String]) -> Result<(), ParseError> {
        let verb = tokens[0].to_ascii_lowercase();
        let rest = &tokens[1..];

        // `end` closes the current function no matter what.
        if verb == "end" {
            return self.close_func(line);
        }

        // While inside a function body, accumulate body steps (depth-1 rules).
        if self.active_func.is_some() {
            return self.parse_func_body(line, &verb, rest);
        }

        // `import browser[,] desktop` - one statement binding one or more drivers.
        // Tokens are split on commas and spaces, so `import browser, desktop` and
        // `import browser desktop` both work.
        if verb == "import" {
            let names: Vec<&str> = rest
                .iter()
                .flat_map(|token| token.split(','))
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .collect();
            if !names.is_empty()
                && names
                    .iter()
                    .all(|name| *name == "browser" || *name == "desktop")
            {
                for name in names {
                    self.push_top(
                        Command::Import {
                            name: name.to_string(),
                        },
                        line,
                    );
                }
                return Ok(());
            }
        }

        let command = match verb.as_str() {
            "let" => {
                let (name, value) = assignment(rest, line, "let <name> <value>")?;
                let value =
                    value.ok_or_else(|| ParseError::new(line, "usage: let <name> <value>"))?;
                Command::Let {
                    name: bare_name(name, line)?,
                    value: parse_arg(value, line)?,
                }
            }
            "set" => {
                let (name, value) = assignment(rest, line, "set <name> <value>")?;
                let value =
                    value.ok_or_else(|| ParseError::new(line, "usage: set <name> <value>"))?;
                Command::Set {
                    name: bare_name(name, line)?,
                    value: parse_arg(value, line)?,
                }
            }
            "param" => {
                let (name, default) = assignment(rest, line, "param <name> [default]")?;
                Command::Param {
                    name: bare_name(name, line)?,
                    default: default.map(|v| parse_arg(v, line)).transpose()?,
                }
            }
            "func" => {
                self.open_func(line, rest)?;
                return Ok(());
            }
            "call" => {
                let name = rest
                    .first()
                    .ok_or_else(|| ParseError::new(line, "usage: call <name> <arg>..."))?;
                let args = rest[1..]
                    .iter()
                    .map(|v| parse_arg(v, line))
                    .collect::<Result<Vec<_>, _>>()?;
                Command::Call {
                    name: name.clone(),
                    args,
                }
            }
            "import" => {
                let target = rest.first().ok_or_else(|| {
                    ParseError::new(line, "usage: import <plugin-id|path.xcl> [op...]")
                })?;
                // `import "path.xcl"` includes another script's functions.
                if target.ends_with(".xcl") {
                    self.import_xcl(target, line)?;
                    return Ok(());
                }
                let plugin = target;
                // `import <plugin> <op>...` binds each named op as a bare callable, so
                // it can be invoked as `<op> [json]` instead of `run <plugin> <op>
                // [json]`. An op is callable with 0 args (no payload) or 1 (a JSON
                // payload). With no ops listed, `import` only gates the plugin.
                for op in &rest[1..] {
                    let op = bare_name(op, line)?;
                    for arity in 0..=1usize {
                        if self.funcs.contains_key(&(op.clone(), arity)) {
                            return Err(ParseError::new(
                                line,
                                format!(
                                    "`{op}` is already defined; an imported op name must be unique"
                                ),
                            ));
                        }
                        if self.funcs.len() >= self.limits.max_funcs {
                            return Err(ParseError::new(
                                line,
                                format!(
                                    "too many functions/imports (max {})",
                                    self.limits.max_funcs
                                ),
                            ));
                        }
                        self.funcs.insert(
                            (op.clone(), arity),
                            Callable::PluginOp {
                                plugin: plugin.clone(),
                                op: op.clone(),
                            },
                        );
                    }
                }
                Command::Import {
                    name: plugin.clone(),
                }
            }
            "run" => {
                let plugin = rest
                    .first()
                    .ok_or_else(|| ParseError::new(line, "usage: run <plugin> <op> [json]"))?;
                let op = rest
                    .get(1)
                    .ok_or_else(|| ParseError::new(line, "usage: run <plugin> <op> [json]"))?;
                let json = rest.get(2).map(|v| parse_arg(v, line)).transpose()?;
                Command::Run {
                    plugin: plugin.clone(),
                    op: op.clone(),
                    json,
                }
            }
            "plugins" => Command::Plugins,
            "plugin-config" => {
                let name = rest
                    .first()
                    .ok_or_else(|| ParseError::new(line, "usage: plugin-config <name> [op]"))?;
                Command::PluginConfig {
                    name: name.clone(),
                    op: rest.get(1).cloned(),
                }
            }
            "request" => {
                let method = rest.first().ok_or_else(|| {
                    ParseError::new(line, "usage: request <METHOD> <url> [headers] [body]")
                })?;
                let url = rest.get(1).ok_or_else(|| {
                    ParseError::new(line, "usage: request <METHOD> <url> [headers] [body]")
                })?;
                let headers = rest.get(2).map(|v| parse_arg(v, line)).transpose()?;
                let body = rest.get(3).map(|v| parse_arg(v, line)).transpose()?;
                Command::Request {
                    method: method.clone(),
                    url: parse_arg(url, line)?,
                    headers,
                    body,
                }
            }
            "repeat" | "retry" => {
                let n = count_arg(rest, line, self.limits.max_iterations)?;
                let inner_tokens = &rest[1..];
                if inner_tokens.is_empty() {
                    return Err(ParseError::new(line, "usage: repeat <n> <verb> <args...>"));
                }
                let inner = self.parse_inner_step(line, inner_tokens)?;
                if verb == "repeat" {
                    Command::Repeat {
                        n,
                        inner: Box::new(inner),
                    }
                } else {
                    Command::Retry {
                        n,
                        inner: Box::new(inner),
                    }
                }
            }
            "if-ok" | "if-fail" => {
                if rest.is_empty() {
                    return Err(ParseError::new(line, "usage: if-ok <verb> <args...>"));
                }
                let inner = self.parse_inner_step(line, rest)?;
                if verb == "if-ok" {
                    Command::IfOk(Box::new(inner))
                } else {
                    Command::IfFail(Box::new(inner))
                }
            }
            "label" => {
                let name = rest
                    .first()
                    .ok_or_else(|| ParseError::new(line, "usage: label <name>"))?;
                Command::Label { name: name.clone() }
            }
            "goto" => {
                let name = rest
                    .first()
                    .ok_or_else(|| ParseError::new(line, "usage: goto <name>"))?;
                Command::Goto { name: name.clone() }
            }
            "assert" => {
                let subject = rest
                    .first()
                    .ok_or_else(|| ParseError::new(line, "usage: assert <subject> <op> <value>"))?;
                let op = rest
                    .get(1)
                    .ok_or_else(|| ParseError::new(line, "usage: assert <subject> <op> <value>"))?;
                let value = rest
                    .get(2)
                    .ok_or_else(|| ParseError::new(line, "usage: assert <subject> <op> <value>"))?;
                Command::Assert {
                    subject: subject.clone(),
                    op: op.clone(),
                    value: parse_arg(value, line)?,
                }
            }
            "done" => Command::Done,
            "quit" => Command::Quit,
            // Any other verb is a browser/session pass-through command — unless it
            // names a function defined above, in which case a bare `name args...`
            // is a call, so the `call` keyword is optional.
            _ => {
                let args = rest
                    .iter()
                    .map(|v| parse_arg(v, line))
                    .collect::<Result<Vec<_>, _>>()?;
                // A bare `name args...` is a call when *any* overload is defined for
                // `name` (a `func` or an imported op); otherwise it is a pass-through
                // browser verb. Arity is checked when the call is lowered.
                if self.has_callable(&verb) {
                    Command::Call { name: verb, args }
                } else {
                    Command::Raw { verb, args }
                }
            }
        };

        // Reject nesting of control-flow inside a `repeat`/`retry` inner step is
        // already handled by parse_inner_step (which refuses control-flow verbs).
        self.push_top(command, line);
        Ok(())
    }

    /// Parses a single inner statement for `repeat`/`retry`/`if-ok`/`if-fail`.
    /// Controlled statements may not themselves be control-flow (no nesting).
    fn parse_inner_step(&self, line: usize, tokens: &[String]) -> Result<Step, ParseError> {
        let verb = tokens[0].to_ascii_lowercase();
        if matches!(
            verb.as_str(),
            "repeat" | "retry" | "if-ok" | "if-fail" | "func" | "end" | "call" | "label" | "goto"
        ) {
            return Err(ParseError::new(
                line,
                format!("`{verb}` cannot be nested inside another control-flow statement"),
            ));
        }
        // Reuse the statement parser on a scratch parser, then take the step.
        let mut scratch = Parser::default();
        scratch.parse_statement(line, tokens)?;
        Ok(scratch.steps.pop().expect("just pushed one step"))
    }

    fn push_top(&mut self, command: Command, line: usize) {
        self.steps.push(Step { line, command });
    }

    /// Whether any callable (a `func` or an imported op) is bound to `name`,
    /// regardless of arity.
    fn has_callable(&self, name: &str) -> bool {
        self.funcs.keys().any(|(n, _)| n == name)
    }

    fn open_func(&mut self, line: usize, rest: &[String]) -> Result<(), ParseError> {
        // The signature spans the rest of the line: `func name(a, b)` may lex as
        // `["name(a,", "b)"]` when there is whitespace after the comma, so join
        // the header tokens back together before parsing.
        let header = rest.join(" ");
        if header.is_empty() {
            return Err(ParseError::new(line, "usage: func <name>(<params...>)"));
        }
        let (name, params) = parse_signature(&header, line, self.limits.max_func_params)?;
        // Overloading is by arity: a second `human(a, b, c, d)` is fine beside
        // `human(a, b)`, but an exact duplicate (same name, same count) is not.
        if self.funcs.contains_key(&(name.clone(), params.len())) {
            return Err(ParseError::new(
                line,
                format!(
                    "function `{name}` with {} parameter(s) already defined",
                    params.len()
                ),
            ));
        }
        if self.funcs.len() >= self.limits.max_funcs {
            return Err(ParseError::new(
                line,
                format!("too many functions/imports (max {})", self.limits.max_funcs),
            ));
        }
        self.active_func = Some(ActiveFunc {
            def: FuncDef {
                name: name.clone(),
                params,
                body: Vec::new(),
            },
            body: Vec::new(),
        });
        Ok(())
    }

    fn parse_func_body(
        &mut self,
        line: usize,
        verb: &str,
        rest: &[String],
    ) -> Result<(), ParseError> {
        // Depth-1: forbid nested function definitions / calls inside a body.
        if matches!(verb, "func" | "end" | "call") {
            return Err(ParseError::new(
                line,
                format!("`{verb}` is not allowed inside a function body (max depth 1)"),
            ));
        }
        if matches!(verb, "repeat" | "retry" | "if-ok" | "if-fail") {
            return Err(ParseError::new(
                line,
                "control-flow is not allowed inside a function body",
            ));
        }
        // Parse the body command via a scratch parser (reuses the grammar).
        let mut scratch = Parser::default();
        scratch.parse_statement(line, &{
            let mut t = vec![verb.to_string()];
            t.extend(rest.iter().cloned());
            t
        })?;
        let step = scratch.steps.pop().expect("one step");
        if let Some(active) = self.active_func.as_mut() {
            active.body.push(step);
        }
        Ok(())
    }

    fn close_func(&mut self, line: usize) -> Result<(), ParseError> {
        let active = self
            .active_func
            .take()
            .ok_or_else(|| ParseError::new(line, "`end` without a matching `func`"))?;
        let mut def = active.def;
        def.body = active.body;
        let key = (def.name.clone(), def.params.len());
        self.funcs.insert(key, Callable::Func(def));
        Ok(())
    }
}

/// Parses a `func` signature `name(a, b, c)`.
fn parse_signature(
    header: &str,
    line: usize,
    max_params: u32,
) -> Result<(String, Vec<ParamDef>), ParseError> {
    let open = header
        .find('(')
        .ok_or_else(|| ParseError::new(line, "usage: func <name>(<params...>)"))?;
    let close = header
        .rfind(')')
        .ok_or_else(|| ParseError::new(line, "function signature needs `)`"))?;
    if close < open {
        return Err(ParseError::new(line, "malformed function signature"));
    }
    let name = header[..open].trim().to_string();
    if name.is_empty() {
        return Err(ParseError::new(line, "function name is required"));
    }
    let params_raw = &header[open + 1..close];
    let mut params = Vec::new();
    if !params_raw.trim().is_empty() {
        for part in params_raw.split(',') {
            let part = part.trim();
            if part.is_empty() {
                return Err(ParseError::new(line, "empty function parameter"));
            }
            params.push(ParamDef {
                name: part.to_string(),
            });
        }
    }
    if params.len() > max_params as usize {
        return Err(ParseError::new(
            line,
            format!("function has {} params; max is {max_params}", params.len()),
        ));
    }
    Ok((name, params))
}

/// Classifies a token into an [`Arg`].
///
/// A token may embed more than one reference and literal text around them, so
/// `$base/account` and `${base}/account` both build a URL, not a lookup of a
/// variable literally named `base/account`. References are:
///
/// * `$name` — a variable, ending at the first character that is not a name
///   character (`[A-Za-z0-9_-]`);
/// * `${name}` — a variable, explicitly delimited (so `$`-adjacent text works);
/// * `{NAME}` — a builtin, when the body is upper-case/digits/underscore; a
///   brace group that is not a builtin (e.g. a JSON fragment) stays literal.
fn parse_arg(token: &str, line: usize) -> Result<Arg, ParseError> {
    let chars: Vec<char> = token.chars().collect();
    let mut pieces: Vec<Arg> = Vec::new();
    let mut literal = String::new();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            // `$$` is a literal `$` (so a value or regex can contain one, e.g. a
            // currency sign).
            '$' if chars.get(i + 1) == Some(&'$') => {
                literal.push('$');
                i += 2;
            }
            '$' if chars.get(i + 1) == Some(&'{') => {
                let close = chars[i + 2..].iter().position(|&c| c == '}');
                let Some(offset) = close else {
                    return Err(ParseError::new(
                        line,
                        "unterminated variable reference `${`",
                    ));
                };
                let name: String = chars[i + 2..i + 2 + offset].iter().collect();
                if name.is_empty() {
                    return Err(ParseError::new(line, "empty variable reference `${}`"));
                }
                flush(&mut pieces, &mut literal, Arg::Var(name));
                i += 2 + offset + 1;
            }
            '$' => {
                let start = i + 1;
                let end = chars[start..]
                    .iter()
                    .position(|&c| !is_name_char(c))
                    .map_or(chars.len(), |offset| start + offset);
                if end == start {
                    return Err(ParseError::new(line, "empty variable reference `$`"));
                }
                let name: String = chars[start..end].iter().collect();
                flush(&mut pieces, &mut literal, Arg::Var(name));
                i = end;
            }
            '{' => {
                let close = chars[i + 1..].iter().position(|&c| c == '}');
                let inner = close.map(|offset| {
                    (
                        offset,
                        chars[i + 1..i + 1 + offset].iter().collect::<String>(),
                    )
                });
                match inner {
                    Some((offset, name)) if is_builtin_name(&name) => {
                        flush(&mut pieces, &mut literal, Arg::Builtin(name));
                        i += 1 + offset + 1;
                    }
                    // Not a builtin (or no closing brace): a plain literal `{`.
                    _ => {
                        literal.push('{');
                        i += 1;
                    }
                }
            }
            c => {
                literal.push(c);
                i += 1;
            }
        }
    }
    if !literal.is_empty() {
        pieces.push(Arg::Literal(literal));
    }
    Ok(match pieces.len() {
        0 => Arg::Literal(String::new()),
        1 => pieces.pop().expect("one piece"),
        _ => Arg::Template(pieces),
    })
}

/// Characters allowed in a bare `$name` reference.
fn is_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

/// A `{NAME}` builtin body: upper-case letters, digits and underscores only, so
/// a lower-case or punctuated brace group (a JSON fragment, a regex) is literal.
fn is_builtin_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
}

/// Pushes accumulated literal text (if any) then the reference, keeping order.
fn flush(pieces: &mut Vec<Arg>, literal: &mut String, arg: Arg) {
    if !literal.is_empty() {
        pieces.push(Arg::Literal(std::mem::take(literal)));
    }
    pieces.push(arg);
}

/// Require a name token (no `$` prefix in the declaration position).
fn bare_name(token: &str, line: usize) -> Result<String, ParseError> {
    if token.starts_with('$') {
        return Err(ParseError::new(
            line,
            format!("expected a plain name, not a reference `{token}`"),
        ));
    }
    if token.is_empty()
        || !token
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(ParseError::new(line, format!("invalid name `{token}`")));
    }
    Ok(token.to_string())
}

/// Splits `<name> [=] [value]` for `let` / `set` / `param`, tolerating an
/// optional `=` between the name and the value so `let base = https://…` reads
/// the same as `let base https://…`.
fn assignment<'a>(
    rest: &'a [String],
    line: usize,
    usage: &str,
) -> Result<(&'a str, Option<&'a str>), ParseError> {
    let name = rest.first().ok_or_else(|| ParseError::new(line, usage))?;
    let skip = usize::from(rest.get(1).is_some_and(|t| t == "="));
    let value = rest.get(1 + skip).map(String::as_str);
    Ok((name.as_str(), value))
}

/// Parses and bounds a loop count: a positive integer ≤ the security cap.
fn count_arg(rest: &[String], line: usize, max_iterations: u32) -> Result<u32, ParseError> {
    let raw = rest
        .first()
        .ok_or_else(|| ParseError::new(line, "usage: repeat <n> <verb> <args...>"))?;
    let n: u32 = raw
        .parse()
        .map_err(|_| ParseError::new(line, format!("`{raw}` is not a positive integer")))?;
    if n == 0 {
        return Err(ParseError::new(line, "repeat count must be positive"));
    }
    let cap = max_iterations;
    if n > cap {
        return Err(ParseError::new(
            line,
            format!("repeat count {n} exceeds the hard cap {cap}"),
        ));
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imports_another_xcl_file() {
        let dir = std::env::temp_dir().join(format!("xcl-import-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("lib.xcl"),
            "func greet(name)\nprint hello $name\nend\n",
        )
        .unwrap();
        let program = parse_program_file(
            "import \"./lib.xcl\"\ngreet world\n",
            Limits::default(),
            Some(dir.clone()),
        )
        .unwrap();
        assert!(program.funcs.contains_key(&("greet".to_string(), 1)));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn parses_let_and_interpolation() {
        let p = parse_program("let base \"https://x.com\"\nopen $base\n").unwrap();
        assert_eq!(p.steps.len(), 2);
        assert!(matches!(&p.steps[0].command, Command::Let { name, .. } if name == "base"));
        assert!(
            matches!(&p.steps[1].command, Command::Raw { verb, args } if verb == "open" && args[0] == Arg::Var("base".into()))
        );
    }

    #[test]
    fn interpolates_suffix_and_braces() {
        // A reference next to literal text builds a template, so `$base/account`
        // is a URL, not a lookup of a variable literally named `base/account`.
        assert_eq!(
            parse_arg("$base/account", 1).unwrap(),
            Arg::Template(vec![
                Arg::Var("base".into()),
                Arg::Literal("/account".into())
            ])
        );
        assert_eq!(
            parse_arg("${base}/account", 1).unwrap(),
            Arg::Template(vec![
                Arg::Var("base".into()),
                Arg::Literal("/account".into())
            ])
        );
        assert_eq!(
            parse_arg("{BASE_URL}/signup", 1).unwrap(),
            Arg::Template(vec![
                Arg::Builtin("BASE_URL".into()),
                Arg::Literal("/signup".into())
            ])
        );
        // A whole-token reference stays the plain variant, so existing scripts
        // and equality tests are unaffected.
        assert_eq!(parse_arg("$base", 1).unwrap(), Arg::Var("base".into()));
        assert_eq!(parse_arg("{UUID}", 1).unwrap(), Arg::Builtin("UUID".into()));
    }

    #[test]
    fn non_builtin_braces_stay_literal() {
        // Lower-case/punctuated brace groups are data (JSON, regex), not builtins.
        assert_eq!(
            parse_arg("{\"a\":1}", 1).unwrap(),
            Arg::Literal("{\"a\":1}".into())
        );
        assert_eq!(
            parse_arg("{name}", 1).unwrap(),
            Arg::Literal("{name}".into())
        );
    }

    #[test]
    fn parses_let_with_optional_equals() {
        // `let x = value` is accepted as `let x value`.
        let p = parse_program("let base = https://example.com\n").unwrap();
        assert!(matches!(&p.steps[0].command, Command::Let { name, value }
                if name == "base" && value == &Arg::Literal("https://example.com".into())));
    }

    #[test]
    fn print_is_an_ordinary_verb() {
        // `print` is a `core` std function, not a language keyword: it parses as a
        // pass-through verb (with its arguments resolved verbatim).
        let p = parse_program("let who \"world\"\nprint hello $who\n").unwrap();
        assert!(matches!(&p.steps[1].command, Command::Raw { verb, args }
                if verb == "print" && args == &vec![Arg::Literal("hello".into()), Arg::Var("who".into())]));
    }

    #[test]
    fn parses_func_and_call() {
        let src = "func register(a, b)\nopen \"{BASE_URL}/signup\"\nfill \"#email\" $a\nend\ncall register \"x\" \"y\"\n";
        let p = parse_program(src).unwrap();
        match p.funcs.get(&("register".to_string(), 2)) {
            Some(Callable::Func(def)) => {
                assert_eq!(def.params.len(), 2);
                assert_eq!(def.body.len(), 2);
            }
            other => panic!("expected func `register`, got {other:?}"),
        }
        assert_eq!(p.steps.len(), 1);
        assert!(matches!(&p.steps[0].command, Command::Call { name, .. } if name == "register"));
    }

    #[test]
    fn allows_overloads_by_arity() {
        // Same name, different parameter counts: both are fine.
        let src = "func human(a, b)\nprint $a\nend\nfunc human(a, b, c, d)\nprint $a\nend\nhuman 1 2\nhuman 1 2 3 4\n";
        let p = parse_program(src).unwrap();
        assert!(p.funcs.contains_key(&("human".to_string(), 2)));
        assert!(p.funcs.contains_key(&("human".to_string(), 4)));
        assert_eq!(p.steps.len(), 2);
        assert!(
            matches!(&p.steps[0].command, Command::Call { name, args } if name == "human" && args.len() == 2)
        );
        assert!(
            matches!(&p.steps[1].command, Command::Call { name, args } if name == "human" && args.len() == 4)
        );
    }

    #[test]
    fn rejects_duplicate_function_same_arity() {
        // Same name *and* same parameter count is a duplicate.
        let src = "func human(a, b)\nprint $a\nend\nfunc human(x, y)\nprint $x\nend\n";
        assert!(parse_program(src).is_err());
    }

    #[test]
    fn import_binds_op_as_callable() {
        let src = "import acme.mod echo\necho {\"message\":\"hi\"}\n";
        let p = parse_program(src).unwrap();
        // An op is callable with no payload or with one JSON payload.
        assert!(p.funcs.contains_key(&("echo".to_string(), 0)));
        assert!(p.funcs.contains_key(&("echo".to_string(), 1)));
        assert!(matches!(&p.steps[0].command, Command::Import { name } if name == "acme.mod"));
        assert!(
            matches!(&p.steps[1].command, Command::Call { name, args } if name == "echo" && args.len() == 1)
        );
    }

    #[test]
    fn import_without_ops_only_gates() {
        let p = parse_program("import acme.mod\n").unwrap();
        assert!(p.funcs.is_empty());
        assert!(matches!(&p.steps[0].command, Command::Import { name } if name == "acme.mod"));
    }

    #[test]
    fn duplicate_import_name_is_rejected() {
        // Two plugins both exporting `echo` collide on the same bare name.
        assert!(parse_program("import a.mod echo\nimport b.mod echo\n").is_err());
    }

    #[test]
    fn func_and_import_name_collide() {
        let src = "import acme.mod echo\nfunc echo(x)\nprint $x\nend\n";
        assert!(parse_program(src).is_err());
    }

    #[test]
    fn bare_function_name_is_a_call() {
        // The `call` keyword is optional: a bare function name invokes it.
        let src = "func fill_field(id, value)\nfill $id $value\nend\nfill_field \"#email\" \"x\"\n";
        let p = parse_program(src).unwrap();
        assert_eq!(p.steps.len(), 1);
        assert!(matches!(&p.steps[0].command, Command::Call { name, .. } if name == "fill_field"));

        // A name that is not a function stays a pass-through session command.
        let p = parse_program("open https://example.com\n").unwrap();
        assert!(matches!(&p.steps[0].command, Command::Raw { verb, .. } if verb == "open"));
    }

    #[test]
    fn rejects_recursive_definitions() {
        assert!(parse_program("func a()\nfunc b()\nend\nend\n").is_err());
    }

    #[test]
    fn rejects_nested_control_flow() {
        assert!(parse_program("repeat 3 repeat 2 click 1\n").is_err());
    }

    #[test]
    fn rejects_unterminated_func() {
        assert!(parse_program("func a()\nopen x\n").is_err());
    }

    #[test]
    fn parses_request() {
        let p = parse_program("request POST https://api/x {\"A\":\"B\"} {\"k\":\"v\"}\n").unwrap();
        assert!(matches!(&p.steps[0].command, Command::Request { method, .. } if method == "POST"));
    }

    #[test]
    fn parses_cookie_set_as_raw() {
        let p = parse_program("cookie set sessionid abc example.com\n").unwrap();
        assert!(matches!(&p.steps[0].command, Command::Raw { verb, args }
                if verb == "cookie" && args.len() == 4));
    }

    #[test]
    fn parses_cookie_add_with_json_literal() {
        // A single-quoted JSON object stays one token, so it reaches the verb whole.
        let src = "cookie add '[{\"name\":\"sessionid\",\"value\":\"x\"}]'\n";
        let p = parse_program(src).unwrap();
        let json = Arg::Literal("[{\"name\":\"sessionid\",\"value\":\"x\"}]".into());
        assert!(matches!(&p.steps[0].command, Command::Raw { verb, args }
                if verb == "cookie" && args.get(1) == Some(&json)));
    }
}
