//! The shared mutable state one session keeps between steps.
//!
//! The REPL loop and every verb arm read and write these fields, so they live in
//! one place instead of being threaded through the dispatch as locals.

use std::path::PathBuf;
use std::sync::Arc;

use xcelerate::{Browser, Page};
use xcelerate_codegen::{Action, Selector};

use crate::cli::BrowserArgs;

/// Everything the session owns: the browser, the pages it drives, and the
/// bookkeeping `--codegen` needs.
pub(crate) struct Session {
    /// The parsed command line, owned so the arms can read flags (`ai`,
    /// `linear`, `codegen`, `codegen_out`, ...) without a lifetime.
    pub(crate) args: BrowserArgs,
    /// The browser process the session owns.
    pub(crate) browser: Arc<Browser>,
    /// The active page; `new-tab` / `switch` / `close-tab` and the CDP self-heal
    /// can all replace it.
    pub(crate) page: Arc<Page>,
    /// The tabs this session opened, in the order `switch` / `close-tab` index
    /// them.
    pub(crate) tabs: Vec<Arc<Page>>,
    /// Index into `tabs` of `page`.
    pub(crate) active_tab: usize,
    /// The recorded run, rendered as code when `--codegen` is set.
    pub(crate) recording: Vec<Action>,
    /// The last selector a step acted on, so focus-scoped steps (`press`,
    /// `submit`) can still be recorded.
    pub(crate) last_target: Option<Selector>,
    /// Video recording state: `record` writes to this path, `stop-record`
    /// finalizes it. Only meaningful while `--codegen` is active.
    pub(crate) video_path: Option<String>,
    /// Whether the AI has declared the task complete with `done`.
    pub(crate) job_done: bool,
    /// Whether stdin is a terminal (a prompt) or a pipe (an echoed transcript).
    pub(crate) interactive: bool,
    /// The workspace root every file verb is confined to (`--output-dir`, or the
    /// current directory). A command may not name a path outside it.
    pub(crate) root: PathBuf,
    /// Set by a step that ran but changed nothing (a by-text target that matched,
    /// or a selector wait that timed out). Cleared before each step so `--codegen`
    /// never records an action that did not actually happen.
    pub(crate) step_no_op: bool,
}
