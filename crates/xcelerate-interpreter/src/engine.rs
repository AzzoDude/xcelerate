//! The XCL engine: walks a [`Program`], executing steps against shared state.
//!
//! Control flow is lowered to bounded program-counter jumps and a small set of
//! loop registers, so there is no call stack. The engine enforces the hard step
//! budget, inlines any `call` to a defined function (depth ≤ 1), and produces one
//! [`Outcome`] per step.

use std::collections::HashMap;

use super::ast::{Arg, Callable, Command, ParamDef, Step};
use super::parse::Program;
use super::runtime::{Context, Outcome, RuntimeLimits};

/// What the engine hands back to its driver on each poll.
///
/// Borrowing the action (`Action(&Command)`) keeps the interpreter's hot loop
/// allocation-free: a `Command` owns `String`s and `Vec<Arg>`s, so returning it
/// by value forced a deep clone on every step.
#[derive(Debug)]
pub enum Next<'a> {
    /// Execute this command, then feed the outcome back via [`Engine::observe`].
    Action(&'a Command),
    /// The run must stop before executing anything; surface this error (e.g. the
    /// step budget was exceeded).
    Halt(String),
    /// The program has finished.
    End,
}

/// The engine advances one op at a time via [`Engine::next_action`] (the driver
/// entry point) or [`Engine::step`] (synchronous helper) until it is
/// [`Engine::done`]. It is transport-agnostic: browser/plugin/HTTP glue is
/// injected through the driver.
pub struct Engine {
    /// Flattened, fully-lowered instruction stream.
    ops: Vec<Op>,
    pc: usize,
    limits: RuntimeLimits,
    /// Loop registers indexed by the `LoopStart` op index: remaining iterations
    /// (0 = inactive). A dense `Vec` avoids hashing on every loop iteration.
    loops: Vec<u32>,
    /// The outcome of the most recently executed *action* op (for `if-ok`/`if-fail`).
    last_ok: bool,
}

/// A lowered, executable op.
#[derive(Debug, Clone)]
enum Op {
    /// Execute a command (the common case).
    Command(Command),
    /// Jump unconditionally to `target`.
    Jump(usize),
    /// Begin a loop of `n` iterations; the matching `LoopEnd::target` points back
    /// to this op.
    LoopStart { n: u32 },
    /// Decrement the loop register; jump back to `target` while it remains > 0.
    LoopEnd { target: usize },
    /// Jump to `target` only if the previous action succeeded (`ok == true`).
    JumpIfOk { target: usize },
    /// Jump to `target` only if the previous action *failed* (`ok == false`).
    JumpIfNotOk { target: usize },
}

impl Engine {
    /// Lower a parsed program into an engine. Resolves labels, inlines `call`.
    pub fn new(program: Program, limits: RuntimeLimits) -> Self {
        let funcs = program.funcs.clone();

        // Lower top-level steps into ops + resolve `goto` labels afterwards.
        // Lowering expands each step into a handful of ops (`repeat`/`retry` emit
        // three, inlined `call`s emit their body), so pre-size to avoid regrowth.
        let mut ops: Vec<Op> = Vec::with_capacity(program.steps.len().saturating_mul(2));
        for step in &program.steps {
            lower_step(step, &mut ops, &funcs);
        }

        // Resolve goto targets against label op-indices.
        let mut label_index = HashMap::new();
        for (idx, op) in ops.iter().enumerate() {
            if let Op::Command(Command::Label { name }) = op {
                label_index.insert(name.clone(), idx);
            }
        }
        for op in &mut ops {
            if let Op::Command(Command::Goto { name }) = op {
                let target = label_index.get(name).copied().unwrap_or(usize::MAX);
                *op = Op::Jump(target);
            }
        }

        let loops = vec![0u32; ops.len()];
        Self {
            ops,
            pc: 0,
            limits,
            loops,
            last_ok: true,
        }
    }

    /// Whether the run has reached the end.
    pub fn done(&self) -> bool {
        self.pc >= self.ops.len()
    }

    /// Returns the next [`Next`] for the driver, advancing the PC past any
    /// control-flow ops (jumps/loops) synchronously.
    ///
    /// This is the async-friendly entry point: the caller `.await`s the returned
    /// [`Next::Action`] command, then calls [`Engine::observe`] with the resulting
    /// [`Outcome`] so the `if-ok`/`if-fail`/`retry` register stays accurate, then
    /// polls again. The command is returned by reference (no clone).
    pub fn next_action(&mut self, ctx: &mut Context) -> Next<'_> {
        loop {
            if self.done() {
                return Next::End;
            }
            if ctx.steps_executed >= self.limits.max_steps {
                self.pc = self.ops.len();
                return Next::Halt(format!("step budget ({}) exceeded", self.limits.max_steps));
            }
            match &self.ops[self.pc] {
                Op::Command(cmd) => {
                    // A label is a no-op control marker.
                    if matches!(cmd, Command::Label { .. }) {
                        self.pc += 1;
                        continue;
                    }
                    if matches!(cmd, Command::Quit) {
                        self.pc = self.ops.len();
                    } else {
                        self.pc += 1;
                    }
                    ctx.steps_executed += 1;
                    return Next::Action(cmd);
                }
                Op::Jump(target) => {
                    if *target == usize::MAX {
                        self.pc = self.ops.len();
                        return Next::Halt("unresolved `goto` label".into());
                    }
                    // Count the jump itself: a `label`/`goto` cycle touches no
                    // command, so without this the step budget would never trip
                    // and the run would spin forever.
                    ctx.steps_executed += 1;
                    self.pc = *target;
                }
                Op::LoopStart { n } => {
                    self.loops[self.pc] = *n;
                    self.pc += 1;
                }
                Op::LoopEnd { target } => {
                    ctx.steps_executed += 1;
                    let remaining = self.loops[*target].saturating_sub(1);
                    if remaining > 0 {
                        self.loops[*target] = remaining;
                        self.pc = *target + 1;
                    } else {
                        self.loops[*target] = 0;
                        self.pc += 1;
                    }
                }
                Op::JumpIfOk { target } => {
                    ctx.steps_executed += 1;
                    if self.last_ok {
                        self.pc = *target;
                    } else {
                        self.pc += 1;
                    }
                }
                Op::JumpIfNotOk { target } => {
                    ctx.steps_executed += 1;
                    if self.last_ok {
                        self.pc += 1;
                    } else {
                        self.pc = *target;
                    }
                }
            }
        }
    }

    /// Records the outcome of the previously returned action, updating the
    /// `if-ok`/`if-fail` register.
    pub fn observe(&mut self, ok: bool) {
        self.last_ok = ok;
    }

    /// Executes a single op via `dispatch`, mutating `ctx`, and advances the PC.
    /// Returns the step outcome, or `None` at end of program.
    pub fn step<F>(&mut self, ctx: &mut Context, disp: &mut F) -> Option<Outcome>
    where
        F: FnMut(&mut Context, &Command) -> Outcome,
    {
        if self.done() {
            return None;
        }
        if ctx.steps_executed >= self.limits.max_steps {
            self.pc = self.ops.len();
            return Some(Outcome::fail(format!(
                "step budget ({}) exceeded",
                self.limits.max_steps
            )));
        }

        // Borrow the op instead of cloning it: a `Command` owns `String`s and
        // `Vec<Arg>`s, so cloning it every step was the interpreter's hottest
        // allocation (felt in tight `repeat` loops). `self.ops` is a disjoint
        // field from `pc`/`last_ok`/`loops`, so the borrow is fine.
        let outcome = match &self.ops[self.pc] {
            Op::Command(cmd) => {
                ctx.steps_executed += 1;
                let out = disp(ctx, cmd);
                if is_action(cmd) {
                    self.last_ok = out.ok;
                }
                if matches!(cmd, Command::Quit) {
                    self.pc = self.ops.len();
                } else {
                    self.pc += 1;
                }
                out
            }
            Op::Jump(target) => {
                if *target == usize::MAX {
                    self.pc = self.ops.len();
                    Outcome::fail("unresolved `goto` label".to_string())
                } else {
                    self.pc = *target;
                    Outcome::ok("goto".to_string())
                }
            }
            Op::JumpIfOk { target } => {
                if self.last_ok {
                    self.pc = *target;
                    Outcome::ok("if-ok taken".to_string())
                } else {
                    self.pc += 1;
                    Outcome::ok("if-ok skipped".to_string())
                }
            }
            Op::JumpIfNotOk { target } => {
                if self.last_ok {
                    self.pc += 1;
                    Outcome::ok("if-not-ok skipped".to_string())
                } else {
                    self.pc = *target;
                    Outcome::ok("if-not-ok taken".to_string())
                }
            }
            Op::LoopStart { n } => {
                self.loops[self.pc] = *n;
                self.pc += 1;
                Outcome::ok(format!("repeat {n}"))
            }
            Op::LoopEnd { target } => {
                let remaining = self.loops[*target].saturating_sub(1);
                if remaining > 0 {
                    self.loops[*target] = remaining;
                    self.pc = *target + 1; // step past the LoopStart into the body
                    Outcome::ok("loop".to_string())
                } else {
                    self.loops[*target] = 0;
                    self.pc += 1;
                    Outcome::ok("loop done".to_string())
                }
            }
        };

        if outcome.should_quit {
            self.pc = self.ops.len();
        }
        Some(outcome)
    }
}

/// Whether a command is an "action" whose success/failure updates the `if-ok`/
/// `if-fail` register.
fn is_action(cmd: &Command) -> bool {
    !matches!(
        cmd,
        Command::Label { .. }
            | Command::Quit
            | Command::Done
            | Command::FuncStart(_)
            | Command::FuncEnd
    )
}

/// Lowers a top-level [`Step`] into ops, inlining any `call`.
fn lower_step(step: &Step, ops: &mut Vec<Op>, funcs: &HashMap<(String, usize), Callable>) {
    match &step.command {
        Command::FuncStart(_) | Command::FuncEnd => {}
        Command::Call { name, args } => {
            // Overload resolution is by arity (the argument count).
            match funcs.get(&(name.clone(), args.len())) {
                Some(Callable::Func(func)) => {
                    for body_step in &func.body {
                        let mut cmd = body_step.command.clone();
                        substitute_args(&mut cmd, &func.params, args);
                        ops.push(Op::Command(cmd));
                    }
                }
                Some(Callable::PluginOp { plugin, op }) => {
                    // An `import`-bound op: `<op> [json]` becomes `run <plugin> <op> [json]`.
                    ops.push(Op::Command(Command::Run {
                        plugin: plugin.clone(),
                        op: op.clone(),
                        json: args.first().cloned(),
                    }));
                }
                None => {
                    // No overload takes this many arguments. Emit a command that
                    // fails clearly at run time, naming the arities that do exist.
                    let mut arities: Vec<usize> = funcs
                        .keys()
                        .filter(|(n, _)| n == name)
                        .map(|(_, arity)| *arity)
                        .collect();
                    arities.sort_unstable();
                    let message = if arities.is_empty() {
                        format!("unknown function `{name}`")
                    } else {
                        format!(
                            "`{name}` has no overload taking {} argument(s); defined for {:?}",
                            args.len(),
                            arities
                        )
                    };
                    ops.push(Op::Command(Command::Run {
                        plugin: "__call__".into(),
                        op: message,
                        json: Some(Arg::Literal("{}".into())),
                    }));
                }
            }
        }
        Command::Label { .. } | Command::Goto { .. } => {
            ops.push(Op::Command(step.command.clone()));
        }
        Command::Repeat { n, inner } => {
            ops.push(Op::LoopStart { n: *n });
            ops.push(Op::Command(inner.command.clone()));
            ops.push(Op::LoopEnd {
                target: ops.len() - 2,
            });
        }
        Command::Retry { n, inner } => {
            // LoopStart/End nest with a body that, on success, must break out.
            // Represent as: LoopStart n; body; (JumpIfOk -> after end); LoopEnd.
            ops.push(Op::LoopStart { n: *n });
            ops.push(Op::Command(inner.command.clone()));
            // The "break on success" jump target is set after we append LoopEnd.
            let end_target = ops.len() + 2; // points just past the LoopEnd below
            ops.push(Op::JumpIfOk { target: end_target });
            ops.push(Op::LoopEnd {
                target: ops.len() - 3,
            });
        }
        Command::IfOk(inner) => {
            // Skip the body when the previous step did not succeed. Guards a
            // *single* statement (`if-ok <verb> ...`), so the target is the op
            // right after it.
            let target = ops.len() + 2;
            ops.push(Op::JumpIfNotOk { target });
            ops.push(Op::Command(inner.command.clone()));
        }
        Command::IfFail(inner) => {
            // Skip the body when the previous step succeeded.
            let target = ops.len() + 2;
            ops.push(Op::JumpIfOk { target });
            ops.push(Op::Command(inner.command.clone()));
        }
        other => ops.push(Op::Command(other.clone())),
    }
}

/// Substitutes `call` arguments into a function body's `Arg::Var` by name.
fn substitute_args(command: &mut Command, params: &[ParamDef], args: &[Arg]) {
    let mut map = HashMap::new();
    for (i, param) in params.iter().enumerate() {
        if let Some(arg) = args.get(i) {
            map.insert(param.name.clone(), arg.clone());
        }
    }
    substitute(command, &map);
}

fn substitute(command: &mut Command, map: &HashMap<String, Arg>) {
    fn repl(a: &Arg, map: &HashMap<String, Arg>) -> Arg {
        match a {
            Arg::Var(name) => map.get(name).cloned().unwrap_or_else(|| a.clone()),
            // Substitute inside a template's pieces, so `$base/account` picks up a
            // `base` function parameter instead of resolving it at run time.
            Arg::Template(pieces) => Arg::Template(pieces.iter().map(|p| repl(p, map)).collect()),
            other => other.clone(),
        }
    }
    let repl = |a: &Arg| repl(a, map);
    match command {
        Command::Let { value, .. }
        | Command::Set { value, .. }
        | Command::Param {
            default: Some(value),
            ..
        } => *value = repl(value),
        Command::Call { args, .. } => {
            for a in args.iter_mut() {
                *a = repl(a);
            }
        }
        Command::Run { json: Some(j), .. } => *j = repl(j),
        Command::Request {
            url, headers, body, ..
        } => {
            *url = repl(url);
            if let Some(h) = headers {
                *h = repl(h);
            }
            if let Some(b) = body {
                *b = repl(b);
            }
        }
        Command::Assert { value, .. } => *value = repl(value),
        Command::Raw { args, .. } => {
            for a in args.iter_mut() {
                *a = repl(a);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::super::parse::parse_program;
    use super::*;

    /// `if-ok` / `if-fail` must branch on the *previous step's* outcome, not run
    /// their body unconditionally (the bug this guarded against).
    #[test]
    fn conditionals_branch_on_the_previous_outcome() {
        fn run_with(src: &str, fail_assert: bool) -> Vec<String> {
            let program = parse_program(src).unwrap();
            let mut engine = Engine::new(program, RuntimeLimits::default());
            let mut ctx = Context::new(
                super::super::security::Permissions::default(),
                super::super::security::Source::Repl,
                "https://x",
            );
            let mut printed = Vec::new();
            while let Some(outcome) = engine.step(&mut ctx, &mut |_ctx, cmd| match cmd {
                Command::Assert { .. } if fail_assert => Outcome::fail("no"),
                Command::Raw { verb, args } if verb == "print" => {
                    printed.push(
                        args.iter()
                            .map(|a| a.source())
                            .collect::<Vec<_>>()
                            .join(" "),
                    );
                    Outcome::ok("print")
                }
                Command::Done => Outcome::done("done"),
                _ => Outcome::ok("ok"),
            }) {
                if outcome.should_quit {
                    break;
                }
            }
            printed
        }
        let src = "assert x == y\nif-ok print yes\nif-fail print no\ndone\n";
        assert_eq!(run_with(src, false), vec!["yes".to_string()]);
        assert_eq!(run_with(src, true), vec!["no".to_string()]);
    }

    fn run(src: &str) -> Vec<Outcome> {
        let program = parse_program(src).unwrap();
        let mut engine = Engine::new(program, RuntimeLimits::default());
        let mut ctx = Context::new(
            super::super::security::Permissions::default(),
            super::super::security::Source::Repl,
            "https://x",
        );
        let mut out = Vec::new();
        while let Some(o) = engine.step(&mut ctx, &mut |_ctx, cmd| match cmd {
            Command::Raw { verb, .. } => Outcome::ok(verb.to_string()),
            Command::Let { name, .. } | Command::Set { name, .. } => {
                Outcome::ok(format!("set {name}"))
            }
            Command::Call { name, .. } => Outcome::ok(format!("call {name}")),
            Command::Run { plugin, op, .. } => {
                if plugin == "__call__" {
                    Outcome::fail(op.clone())
                } else {
                    Outcome::ok(format!("run {plugin}.{op}"))
                }
            }
            Command::Repeat { n, .. } | Command::Retry { n, .. } => Outcome::ok(format!("r{n}")),
            Command::Done => Outcome::done("done"),
            Command::Quit => Outcome::quit("quit"),
            _ => Outcome::ok("ok"),
        }) {
            out.push(o);
        }
        out
    }

    #[test]
    fn runs_basic_sequence() {
        let out = run("open https://x\nclick 3\ndone\n");
        assert_eq!(out.len(), 3);
        assert!(out[0].message.contains("open"));
    }

    #[test]
    fn repeat_runs_n_times() {
        let out = run("repeat 3 click 1\n");
        // LoopStart + 3×(body) + LoopEnd settle = the body runs 3 times.
        let clicks = out.iter().filter(|o| o.message == "click").count();
        assert_eq!(clicks, 3, "expected 3 clicks, got {clicks}");
    }

    #[test]
    fn call_inlines_function() {
        let src = "func go()\nclick 1\nend\ncall go\n";
        let out = run(src);
        assert!(out.iter().any(|o| o.message == "click"));
    }

    #[test]
    fn imported_op_call_lowers_to_run() {
        let out = run("import acme.mod echo\necho {\"message\":\"hi\"}\n");
        assert!(
            out.iter().any(|o| o.message == "run acme.mod.echo"),
            "expected the bound op to lower to a run, got {out:?}"
        );
    }

    #[test]
    fn arity_mismatch_fails_with_a_hint() {
        let out = run("func human(a, b)\nprint $a\nend\nhuman 1 2 3\n");
        assert!(
            out.iter()
                .any(|o| !o.ok && o.message.contains("no overload")),
            "expected an arity error, got {out:?}"
        );
    }

    #[test]
    fn done_terminates_the_run() {
        // `done` marks the task complete, so anything after it must not run.
        let out = run("done\nopen https://x\n");
        assert_eq!(out.len(), 1, "expected only the `done` step, got {out:?}");
        assert!(out[0].should_quit);
    }

    #[test]
    fn step_budget_is_enforced() {
        let program = parse_program("repeat 5 click 1\n").unwrap();
        let mut engine = Engine::new(
            program,
            RuntimeLimits {
                max_steps: 3,
                max_output_chars: 1024,
            },
        );
        let mut ctx = Context::new(
            super::super::security::Permissions::default(),
            super::super::security::Source::Repl,
            "https://x",
        );
        let mut count = 0;
        while engine
            .step(&mut ctx, &mut |_, _| Outcome::ok("x"))
            .is_some()
        {
            count += 1;
            if count > 100 {
                break;
            }
        }
        assert!(ctx.steps_executed <= 3);
    }
}
