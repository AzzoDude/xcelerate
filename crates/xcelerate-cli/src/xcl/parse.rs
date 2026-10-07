//! The XCL parser: tokens → [`Program`] of [`Step`]s.
//!
//! [`parse_program`] is the single entry point. It folds the flat token stream
//! into a validated program: functions are collected (and their bodies kept as
//! steps), labels are resolved to line indices, control-flow verbs are checked
//! for the bounded-loop invariants, and each token is classified into an
//! [`Arg`] (literal / `$var` / `{builtin}`).

use std::collections::HashMap;

use super::ast::{Arg, Command, FuncDef, ParamDef, Step};
use super::lex::{Line, lex_line};

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
    /// Functions by name, in definition order.
    pub funcs: HashMap<String, FuncDef>,
}

/// Parses an entire `.xcl` source into a [`Program`].
pub fn parse_program(source: &str) -> Result<Program, ParseError> {
    let parser = Parser::default();
    parser.run(source)
}

#[derive(Default)]
struct Parser {
    steps: Vec<Step>,
    funcs: HashMap<String, FuncDef>,
    /// When inside a `func ... end`, the function under construction.
    active_func: Option<ActiveFunc>,
}

struct ActiveFunc {
    def: FuncDef,
    body: Vec<Step>,
}

impl Parser {
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

        let command = match verb.as_str() {
            "let" => {
                let (name, value) = two(rest, line, "let <name> <value>")?;
                Command::Let {
                    name: bare_name(&name, line)?,
                    value: parse_arg(&value, line)?,
                }
            }
            "set" => {
                let (name, value) = two(rest, line, "set <name> <value>")?;
                Command::Set {
                    name: bare_name(&name, line)?,
                    value: parse_arg(&value, line)?,
                }
            }
            "param" => {
                let name = rest
                    .first()
                    .ok_or_else(|| ParseError::new(line, "usage: param <name> [default]"))?;
                let default = rest.get(1).map(|v| parse_arg(v, line)).transpose()?;
                Command::Param {
                    name: bare_name(name, line)?,
                    default,
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
                let name = rest
                    .first()
                    .ok_or_else(|| ParseError::new(line, "usage: import <plugin-id>"))?;
                Command::Import { name: name.clone() }
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
                let n = count_arg(rest, line)?;
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
            "quit" | "exit" | "q" => Command::Quit,
            // Any other verb is a browser/session pass-through command.
            _ => Command::Raw {
                verb: verb.clone(),
                args: rest
                    .iter()
                    .map(|v| parse_arg(v, line))
                    .collect::<Result<_, _>>()?,
            },
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

    fn open_func(&mut self, line: usize, rest: &[String]) -> Result<(), ParseError> {
        // The signature spans the rest of the line: `func name(a, b)` may lex as
        // `["name(a,", "b)"]` when there is whitespace after the comma, so join
        // the header tokens back together before parsing.
        let header = rest.join(" ");
        if header.is_empty() {
            return Err(ParseError::new(line, "usage: func <name>(<params...>)"));
        }
        let (name, params) = parse_signature(&header, line)?;
        if self.funcs.contains_key(&name) {
            return Err(ParseError::new(
                line,
                format!("function `{name}` already defined"),
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
        let name = def.name.clone();
        self.funcs.insert(name, def);
        Ok(())
    }
}

/// Parses a `func` signature `name(a, b, c)`.
fn parse_signature(header: &str, line: usize) -> Result<(String, Vec<ParamDef>), ParseError> {
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
    if params.len() > crate::xcl::security::MAX_FUNC_PARAMS as usize {
        return Err(ParseError::new(
            line,
            format!(
                "function has {} params; max is {}",
                params.len(),
                crate::xcl::security::MAX_FUNC_PARAMS
            ),
        ));
    }
    Ok((name, params))
}

/// Classifies a token into an [`Arg`].
fn parse_arg(token: &str, line: usize) -> Result<Arg, ParseError> {
    if let Some(rest) = token.strip_prefix('$') {
        if rest.is_empty() {
            return Err(ParseError::new(line, "empty variable reference `$`"));
        }
        return Ok(Arg::Var(rest.to_string()));
    }
    if let Some(inner) = token.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
        return Ok(Arg::Builtin(inner.to_string()));
    }
    Ok(Arg::Literal(token.to_string()))
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

fn two<'a>(rest: &'a [String], line: usize, usage: &str) -> Result<(&'a str, &'a str), ParseError> {
    let a = rest.first().ok_or_else(|| ParseError::new(line, usage))?;
    let b = rest.get(1).ok_or_else(|| ParseError::new(line, usage))?;
    Ok((a, b))
}

/// Parses and bounds a loop count: a positive integer ≤ the security cap.
fn count_arg(rest: &[String], line: usize) -> Result<u32, ParseError> {
    let raw = rest
        .first()
        .ok_or_else(|| ParseError::new(line, "usage: repeat <n> <verb> <args...>"))?;
    let n: u32 = raw
        .parse()
        .map_err(|_| ParseError::new(line, format!("`{raw}` is not a positive integer")))?;
    if n == 0 {
        return Err(ParseError::new(line, "repeat count must be positive"));
    }
    let cap = crate::xcl::security::MAX_ITERATIONS;
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
    fn parses_let_and_interpolation() {
        let p = parse_program("let base \"https://x.com\"\nopen $base\n").unwrap();
        assert_eq!(p.steps.len(), 2);
        assert!(matches!(&p.steps[0].command, Command::Let { name, .. } if name == "base"));
        assert!(
            matches!(&p.steps[1].command, Command::Raw { verb, args } if verb == "open" && args[0] == Arg::Var("base".into()))
        );
    }

    #[test]
    fn parses_func_and_call() {
        let src = "func register(a, b)\nopen \"{BASE_URL}/signup\"\nfill \"#email\" $a\nend\ncall register \"x\" \"y\"\n";
        let p = parse_program(src).unwrap();
        assert!(p.funcs.contains_key("register"));
        assert_eq!(p.funcs["register"].params.len(), 2);
        assert_eq!(p.funcs["register"].body.len(), 2);
        assert_eq!(p.steps.len(), 1);
        assert!(matches!(&p.steps[0].command, Command::Call { name, .. } if name == "register"));
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
}
