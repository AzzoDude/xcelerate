//! The XCL executor: maps [`Command`]s onto the live browser, plugin, and HTTP
//! transports. This is the one place the language touches the engine facade.
//!
//! Dispatch is async, matching the rest of the CLI (which runs on a multi-thread
//! tokio runtime), so there are no thread bridges — a browser/plugin call is a
//! plain `.await`.

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use xcelerate::plugin::PluginManager;
use xcelerate::{Browser, Page};

use super::ast::Command;
use super::runtime::{Context, Outcome};

/// A **driver**: how xcelerate talks to one kind of "application".
///
/// The browser is not special - it is an application too, driven through CDP. The
/// desktop (any native Windows window) is driven through UI Automation. Both are
/// native processes; the driver is only the interface. A script brings one in
/// with `import browser` / `import desktop`, and the *active* driver decides what
/// the shared verbs (`click`, `fill`, `find`, `wait`, `scroll`) act on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Driver {
    /// The browser application, driven by CDP.
    Browser,
    /// Any native Windows application, driven by UI Automation.
    Desktop,
}

/// The future a [`BrowserFactory`] resolves to.
pub type BrowserFuture =
    Pin<Box<dyn Future<Output = Result<(Arc<Browser>, Arc<Page>), String>> + Send>>;

/// Launches the browser driver on demand, so a run that never touches the browser
/// never pays for one. Supplied by the CLI, which owns the launch configuration.
pub type BrowserFactory = Arc<dyn Fn() -> BrowserFuture + Send + Sync>;

/// Shared executor state: the imported drivers plus the tabs opened during a run.
pub struct Executor {
    /// The browser driver, once imported (launched on demand). `None` until then.
    browser: Mutex<Option<Arc<Browser>>>,
    /// How to launch the browser driver when it is first imported.
    factory: Option<BrowserFactory>,
    /// Tabs opened this run; index 0 is the page the run started on.
    tabs: Mutex<Vec<Arc<Page>>>,
    active: AtomicUsize,
    /// The driver the shared verbs currently act on.
    driver: Mutex<Driver>,
    /// The run's plugin host. It is independent of the browser, so a native-only
    /// run can load and drive the `xcelerate.app` plugin with no browser at all.
    plugins: Arc<PluginManager>,
    /// The native window selected by `window`/`launch`; Windows only.
    #[cfg(windows)]
    native_window: Mutex<Option<String>>,
}

/// A fresh, empty plugin host. Plugins arrive through [`Executor::load_plugin`].
fn empty_plugin_manager() -> Arc<PluginManager> {
    let catalog: xcelerate::plugin::Catalog = Arc::new(|_| None);
    Arc::new(PluginManager::new(&[], catalog).expect("an empty plugin manager is always valid"))
}

impl Executor {
    /// An executor with an already-launched browser driver and one active tab.
    /// The browser is the active driver.
    pub fn new(browser: Arc<Browser>, page: Arc<Page>) -> Self {
        Self {
            browser: Mutex::new(Some(browser)),
            factory: None,
            tabs: Mutex::new(vec![page]),
            active: AtomicUsize::new(0),
            driver: Mutex::new(Driver::Browser),
            plugins: empty_plugin_manager(),
            #[cfg(windows)]
            native_window: Mutex::new(None),
        }
    }

    /// A browser-capable executor that has **not** launched the browser yet: the
    /// first `import browser` (or browser verb) calls `factory`. The browser is the
    /// active driver, so a browser script needs no explicit import.
    pub fn lazy(factory: BrowserFactory) -> Self {
        Self {
            browser: Mutex::new(None),
            factory: Some(factory),
            tabs: Mutex::new(Vec::new()),
            active: AtomicUsize::new(0),
            driver: Mutex::new(Driver::Browser),
            plugins: empty_plugin_manager(),
            #[cfg(windows)]
            native_window: Mutex::new(None),
        }
    }

    /// A browserless executor attached to a native window (Windows only).
    #[cfg(windows)]
    pub fn native(window: String) -> Self {
        Self {
            browser: Mutex::new(None),
            factory: None,
            tabs: Mutex::new(Vec::new()),
            active: AtomicUsize::new(0),
            driver: Mutex::new(Driver::Desktop),
            plugins: empty_plugin_manager(),
            native_window: Mutex::new(Some(window)),
        }
    }

    /// A browserless executor with no window selected yet (Windows only): the
    /// desktop driver is active and `window`/`launch` pick the window.
    #[cfg(windows)]
    pub fn desktop() -> Self {
        Self {
            browser: Mutex::new(None),
            factory: None,
            tabs: Mutex::new(Vec::new()),
            active: AtomicUsize::new(0),
            driver: Mutex::new(Driver::Desktop),
            plugins: empty_plugin_manager(),
            #[cfg(windows)]
            native_window: Mutex::new(None),
        }
    }

    /// The driver the shared verbs currently act on.
    pub fn driver(&self) -> Driver {
        *self.driver.lock().unwrap()
    }

    /// Makes `driver` the active one; the shared verbs follow it.
    pub fn set_driver(&self, driver: Driver) {
        *self.driver.lock().unwrap() = driver;
    }

    /// Ensures the browser driver is launched, when a factory is available.
    /// Idempotent and cheap once the browser is up.
    pub async fn ensure_browser(&self) -> Result<(), String> {
        if self.browser.lock().unwrap().is_some() {
            return Ok(());
        }
        let factory = self.factory.clone().ok_or_else(|| NO_BROWSER.to_string())?;
        let (browser, page) = factory().await?;
        *self.browser.lock().unwrap() = Some(browser);
        *self.tabs.lock().unwrap() = vec![page];
        self.active.store(0, Ordering::Relaxed);
        Ok(())
    }

    /// Whether the browser driver is loaded.
    pub fn has_browser(&self) -> bool {
        self.browser.lock().unwrap().is_some()
    }

    /// Whether this run *can* load the browser driver (loaded, or a factory is
    /// available to launch it).
    pub fn browser_available(&self) -> bool {
        self.factory.is_some() || self.browser.lock().unwrap().is_some()
    }

    /// The browser driver, or an error if it is not loaded.
    pub fn browser(&self) -> Result<Arc<Browser>, String> {
        self.browser
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| NO_BROWSER.to_string())
    }

    /// Takes the browser handle out (for closing), if it was launched.
    pub fn take_browser(&self) -> Option<Arc<Browser>> {
        self.browser.lock().unwrap().take()
    }

    /// The currently active page (browser driver only).
    pub fn page(&self) -> Result<Arc<Page>, String> {
        let tabs = self.tabs.lock().unwrap();
        let index = self
            .active
            .load(Ordering::Relaxed)
            .min(tabs.len().saturating_sub(1));
        tabs.get(index)
            .cloned()
            .ok_or_else(|| NO_BROWSER.to_string())
    }

    /// The current native window title, if any.
    #[cfg(windows)]
    fn native_window(&self) -> Option<String> {
        self.native_window.lock().unwrap().clone()
    }

    /// Replaces the native target window.
    #[cfg(windows)]
    fn set_native_window(&self, window: String) {
        *self.native_window.lock().unwrap() = Some(window);
    }

    /// Loads a sandboxed plugin from a path (a directory or `plugin.json`),
    /// returning its name. The plugin is hosted by this executor, not the
    /// browser, so a native-only run can load one with no browser at all.
    pub fn load_plugin(&self, path: &str) -> Result<String, String> {
        let plugin = xcelerate::plugin::load_plugin(path).map_err(|error| error.to_string())?;
        let name = plugin.name().to_string();
        self.plugins
            .install(plugin)
            .map_err(|error| error.to_string())?;
        Ok(name)
    }

    /// The names of the plugins loaded into this run.
    pub fn plugin_names(&self) -> Vec<String> {
        self.plugins.names()
    }

    /// The run's plugin host (for listing ops and reading a plugin's config).
    pub fn plugins(&self) -> &Arc<PluginManager> {
        &self.plugins
    }

    /// Invokes a plugin op, optionally bound to the run's active page so the
    /// `browser` bridge is open.
    pub async fn invoke_plugin(
        &self,
        plugin: &str,
        op: &str,
        args_json: String,
        page: Option<Arc<Page>>,
    ) -> Result<String, String> {
        let page = page.map(xcelerate::plugin::page_host);
        self.plugins
            .invoke(plugin, op, args_json, page)
            .await
            .map_err(|error| error.to_string())
    }
}

/// Error returned when the browser driver is used but not available in this run.
const NO_BROWSER: &str =
    "the browser driver is not loaded; run `import browser` (or use a browser verb) first";

/// Dispatches a single [`Command`] to the transport, returning its [`Outcome`].
pub async fn dispatch(ctx: &mut Context, cmd: &Command, exe: &Executor) -> Outcome {
    match cmd {
        Command::Let { name, value } => match ctx.resolve(value) {
            Ok(v) => {
                ctx.vars.insert(name.clone(), v);
                Outcome::ok(format!("let {name}"))
            }
            Err(e) => Outcome::fail(e),
        },
        Command::Set { name, value } => match ctx.resolve(value) {
            Ok(v) => {
                ctx.vars.insert(name.clone(), v);
                Outcome::ok(format!("set {name}"))
            }
            Err(e) => Outcome::fail(e),
        },
        Command::Param { name, default } => {
            if ctx.vars.contains_key(name) {
                Outcome::ok(format!("param {name}"))
            } else if let Some(default) = default {
                match ctx.resolve(default) {
                    Ok(v) => {
                        ctx.vars.insert(name.clone(), v);
                        Outcome::ok(format!("param {name} (default)"))
                    }
                    Err(e) => Outcome::fail(e),
                }
            } else {
                Outcome::fail(format!("missing required param `{name}`"))
            }
        }
        Command::Print { args } => {
            let mut parts = Vec::with_capacity(args.len());
            for a in args {
                match ctx.resolve(a) {
                    Ok(v) => parts.push(v),
                    Err(e) => return Outcome::fail(e),
                }
            }
            Outcome::ok(parts.join(" "))
        }
        Command::Import { name } => match name.as_str() {
            // Driver imports: bring in an application interface. `browser` launches
            // the CDP-driven browser on demand; `desktop` selects UI Automation.
            // (Plugins are imported below - a driver is not a plugin.)
            "browser" => match exe.ensure_browser().await {
                Ok(()) => {
                    exe.set_driver(Driver::Browser);
                    Outcome::ok("import browser".to_string())
                }
                Err(error) => Outcome::fail(error),
            },
            "desktop" => {
                #[cfg(windows)]
                {
                    exe.set_driver(Driver::Desktop);
                    Outcome::ok("import desktop".to_string())
                }
                #[cfg(not(windows))]
                {
                    Outcome::fail("the desktop driver is Windows only".to_string())
                }
            }
            _ => {
                if !ctx.permissions.plugin_allowed(name) {
                    return Outcome::fail(format!(
                        "plugin `{name}` is not allowed (use --allow-plugin)"
                    ));
                }
                // `import` is a load-time hint; actual loading happens through the
                // plugin manager when the browser is built. Here we only gated it.
                Outcome::ok(format!("import {name}"))
            }
        },
        Command::Plugins => {
            let names = exe.plugin_names();
            let text = if names.is_empty() {
                "none".into()
            } else {
                names.join(", ")
            };
            Outcome::ok(format!("loaded: {text}"))
        }
        Command::PluginConfig { name, op } => {
            // Reading a plugin's config/schema is still a plugin capability: it
            // must be allowlisted, exactly like `import`/`run`.
            if !ctx.permissions.plugin_allowed(name) {
                return Outcome::fail(format!(
                    "plugin `{name}` is not allowed (pass --allow-plugin {name})"
                ));
            }
            match exe.plugins().manifest(name) {
                Some(manifest) => {
                    let json = serde_json::to_string(&manifest.config)
                        .unwrap_or_else(|_| "{}".to_string());
                    let parsed: serde_json::Value =
                        serde_json::from_str(&json).unwrap_or(serde_json::Value::Null);
                    match op {
                        Some(op) => Outcome::ok_value(
                            format!("config for {name}.{op}"),
                            parsed
                                .get(op)
                                .cloned()
                                .unwrap_or(serde_json::Value::Null)
                                .to_string(),
                        ),
                        None => Outcome::ok_value(format!("config for {name}"), json),
                    }
                }
                None => Outcome::fail(format!("plugin `{name}` is not loaded")),
            }
        }
        Command::Run { plugin, op, json } => {
            // `lower_step` routes an unresolvable `call` here as a deliberate
            // failure, with the human-readable reason carried in `op`.
            if plugin == "__call__" {
                return Outcome::fail(op.clone());
            }
            if !ctx.permissions.plugin_allowed(plugin) {
                return Outcome::fail(format!(
                    "plugin `{plugin}` is not allowed (use --allow-plugin)"
                ));
            }
            let args = match json {
                Some(a) => ctx.resolve(a).unwrap_or_else(|e| e),
                None => "{}".to_string(),
            };
            // Bind the plugin to the run's active page, when there is one, so a
            // plugin granted the `browser` capability can drive it. Best-effort:
            // a native-only run has no browser, and its `app` bridge still works.
            let _ = exe.ensure_browser().await;
            let page = exe.page().ok();
            match exe.invoke_plugin(plugin, op, args, page).await {
                Ok(result) => Outcome::ok_value(format!("{plugin}.{op}"), result),
                Err(e) => Outcome::fail(format!("{plugin}.{op}: {e}")),
            }
        }
        Command::Request {
            method,
            url,
            headers,
            body,
        } => request(ctx, method, url, headers, body).await,
        Command::Raw { verb, args } => match dispatch_raw(ctx, verb, args, exe).await {
            Ok(msg) => Outcome::ok(msg),
            Err(e) => Outcome::fail(e),
        },
        Command::Assert { subject, op, value } => {
            let expected = match ctx.resolve(value) {
                Ok(v) => v,
                Err(e) => return Outcome::fail(e),
            };
            if let Err(error) = exe.ensure_browser().await {
                return Outcome::fail(error);
            }
            let page = match exe.page() {
                Ok(page) => page,
                Err(error) => return Outcome::fail(error),
            };
            let actual: String = match subject.as_str() {
                "url" => page.url().await.unwrap_or_default(),
                "title" => page.title().await.unwrap_or_default(),
                "content" => page.content().await.unwrap_or_default(),
                "text" => page
                    .evaluate_string("document.body ? document.body.innerText : ''".to_string())
                    .await
                    .unwrap_or_default(),
                "status" => ctx.vars.get("STATUS").cloned().unwrap_or_default(),
                "body" | "response" => ctx.vars.get("RESPONSE_BODY").cloned().unwrap_or_default(),
                other => ctx
                    .vars
                    .get(other.strip_prefix('$').unwrap_or(other))
                    .cloned()
                    .unwrap_or_default(),
            };
            let passed = match op.as_str() {
                "==" | "eq" => actual == expected,
                "!=" | "ne" => actual != expected,
                "contains" => actual.contains(&expected),
                "matches" => actual.contains(&expected),
                ">" => actual > expected,
                "<" => actual < expected,
                ">=" => actual >= expected,
                "<=" => actual <= expected,
                _ => false,
            };
            if passed {
                Outcome::ok(format!("assert {subject} {op} {expected:?}"))
            } else {
                Outcome::fail(format!(
                    "assert {subject} {op} {expected:?} failed (got {actual:?})"
                ))
            }
        }
        Command::Done => Outcome::done("done".to_string()),
        Command::Quit => Outcome::quit("quit".to_string()),
        Command::FuncStart(_)
        | Command::FuncEnd
        | Command::Label { .. }
        | Command::Goto { .. }
        | Command::Repeat { .. }
        | Command::Retry { .. }
        | Command::IfOk(_)
        | Command::IfFail(_)
        | Command::Call { .. } => Outcome::ok(String::new()),
    }
}

/// Which driver a verb runs on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Route {
    Browser,
    Desktop,
}

/// Verbs that always drive the desktop driver (no browser equivalent exists).
fn is_native_only(verb: &str) -> bool {
    matches!(verb, "launch" | "window" | "tree" | "key" | "wheel")
}

/// Verbs that exist on **both** drivers; the active [`Driver`] chooses one.
fn is_shared(verb: &str) -> bool {
    matches!(verb, "click" | "find" | "wait" | "scroll" | "fill")
}

/// Routes `verb` for the active driver: native-only verbs always go to the
/// desktop driver, shared verbs follow the driver, everything else is a browser
/// verb.
fn resolve_route(verb: &str, driver: Driver) -> Route {
    if is_native_only(verb) || (is_shared(verb) && driver == Driver::Desktop) {
        Route::Desktop
    } else {
        Route::Browser
    }
}

/// Dispatches a native verb against a window (Windows only).
///
/// Read-only verbs (`tree`/`find`/`wait`) run once a window is selected; every
/// verb that *acts* (`click`/`fill`/`key`/`wheel`/`scroll`) runs the `app_allowed`
/// gate, and a chord that escapes the app is refused. The window is chosen by
/// `window <title>`, by `launch <app>`, or by the run's `--app <title>`.
#[cfg(windows)]
async fn dispatch_native(
    ctx: &Context,
    verb: &str,
    args: &[String],
    exe: &Executor,
) -> Result<String, String> {
    // Launching spawns a process and selects the window it opens; it needs no
    // pre-selected window, so it is handled first. It also makes native active.
    if verb == "launch" {
        let target = args
            .first()
            .cloned()
            .filter(|t| !t.is_empty())
            .ok_or("usage: launch <target> [window-title] [wait-ms]")?;
        let title = args.get(1).cloned().filter(|t| !t.is_empty());
        let wait_ms: u64 = args.get(2).and_then(|m| m.parse().ok()).unwrap_or(15_000);
        // Default-deny: the human must have granted the target, or the title it
        // will appear under. An AI/`.xcl` run can never self-grant.
        let permitted = ctx.permissions.app_allowed(&target)
            || title
                .as_deref()
                .is_some_and(|t| ctx.permissions.app_allowed(t));
        if !permitted {
            return Err(format!(
                "launching `{target}` is not allowed (pass --allow-app \"{target}\")"
            ));
        }
        let mut payload = serde_json::json!({ "target": target, "wait_ms": wait_ms });
        if let Some(title) = &title {
            payload["title"] = serde_json::json!(title);
        }
        let result = invoke_app(exe, "launch", payload).await?;
        let window = result
            .get("window")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        if !window.is_empty() {
            exe.set_native_window(window);
        }
        exe.set_driver(Driver::Desktop);
        return Ok(result
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("launched")
            .to_string());
    }

    let window = if verb == "window" {
        let name = args.first().cloned().ok_or("usage: window <title>")?;
        if !ctx.permissions.app_allowed(&name) {
            return Err(format!(
                "driving `{name}` is not allowed (pass --allow-app \"{name}\")"
            ));
        }
        exe.set_native_window(name.clone());
        exe.set_driver(Driver::Desktop);
        return Ok(format!("window {name:?}"));
    } else {
        exe.native_window()
            .ok_or("no native window selected (use `window <title>` or `launch <app>`)")?
    };

    // Acting verbs re-check the grant (defense in depth).
    let acting = matches!(verb, "click" | "fill" | "key" | "wheel" | "scroll");
    if acting && !ctx.permissions.app_allowed(&window) {
        return Err(format!(
            "driving `{window}` is not allowed (pass --allow-app \"{window}\")"
        ));
    }

    // Every verb now runs in the `xcelerate.app` plugin over the host bridge, so
    // the native-app code lives in the plugin, not here.
    match verb {
        "tree" => {
            let result = invoke_app(exe, "tree", serde_json::json!({ "window": window })).await?;
            Ok(native_lines(&result))
        }
        "find" => {
            let needle = args
                .first()
                .cloned()
                .unwrap_or_default()
                .to_ascii_lowercase();
            let result = invoke_app(
                exe,
                "find",
                serde_json::json!({ "window": window, "text": needle }),
            )
            .await?;
            let lines = native_lines(&result);
            Ok(if lines.is_empty() {
                "(no match)".to_string()
            } else {
                lines
            })
        }
        "wait" => {
            let text = args.first().cloned().ok_or("usage: wait <text> [ms]")?;
            let ms = args.get(1).and_then(|m| m.parse().ok()).unwrap_or(10_000);
            let result = invoke_app(
                exe,
                "wait",
                serde_json::json!({ "window": window, "text": text, "timeout_ms": ms }),
            )
            .await?;
            Ok(format!(
                "found {:?}",
                result
                    .get("found")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
            ))
        }
        "click" => {
            // `click <index>` uses the last `tree`; `click "<text>"` acts by name.
            let arg = args
                .first()
                .cloned()
                .ok_or("usage: click <index> | click \"<text>\"")?;
            let payload = match arg.parse::<usize>() {
                Ok(index) => serde_json::json!({ "window": window, "index": index }),
                Err(_) => serde_json::json!({ "window": window, "name": arg }),
            };
            let result = invoke_app(exe, "click", payload).await?;
            Ok(format!(
                "clicked via {}",
                result
                    .get("method")
                    .and_then(|v| v.as_str())
                    .unwrap_or("invoke")
            ))
        }
        "fill" => {
            let index: usize = args
                .first()
                .ok_or("usage: fill <index> <text>")?
                .parse()
                .map_err(|_| "fill <index> <text>: index must be a number".to_string())?;
            let text = args.get(1).cloned().unwrap_or_default();
            invoke_app(
                exe,
                "set_value",
                serde_json::json!({ "window": window, "index": index, "text": text }),
            )
            .await?;
            Ok(format!("set [{index}]"))
        }
        "key" => {
            let key = args.first().cloned().ok_or("usage: key <key>")?;
            if crate::security::is_dangerous_key(&key) {
                return Err(format!("`{key}` escapes the app; refused"));
            }
            invoke_app(
                exe,
                "key",
                serde_json::json!({ "window": window, "key": key }),
            )
            .await?;
            Ok(format!("key {key:?}"))
        }
        "wheel" => {
            let notches: i32 = args
                .first()
                .ok_or("usage: wheel <notches>")?
                .parse()
                .map_err(|_| "wheel <notches>: notches must be a number".to_string())?;
            invoke_app(
                exe,
                "wheel",
                serde_json::json!({ "window": window, "notches": notches }),
            )
            .await?;
            Ok(format!("wheeled {notches}"))
        }
        "scroll" => {
            let notches: i32 = args
                .first()
                .ok_or("usage: scroll <notches>")?
                .parse()
                .map_err(|_| "scroll <notches>: notches must be a number".to_string())?;
            let result = invoke_app(
                exe,
                "scroll",
                serde_json::json!({ "window": window, "notches": notches }),
            )
            .await?;
            Ok(format!(
                "scrolled {notches} via {}",
                result
                    .get("method")
                    .and_then(|v| v.as_str())
                    .unwrap_or("pattern")
            ))
        }
        other => Err(format!("unknown native verb `{other}`")),
    }
}

/// Invokes the `xcelerate.app` plugin and decodes its JSON result, mapping a
/// missing plugin (not loaded) to a clear, actionable error.
#[cfg(windows)]
async fn invoke_app(
    exe: &Executor,
    op: &str,
    payload: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let result = exe
        .invoke_plugin("xcelerate.app", op, payload.to_string(), None)
        .await?;
    serde_json::from_str(&result)
        .map_err(|error| format!("xcelerate.app.{op} returned invalid JSON: {error}"))
}

/// The `lines` array of an app-plugin result, joined for display.
#[cfg(windows)]
fn native_lines(value: &serde_json::Value) -> String {
    value
        .get("lines")
        .and_then(|lines| lines.as_array())
        .map(|lines| {
            lines
                .iter()
                .filter_map(|line| line.as_str())
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default()
}

/// Native verbs are Windows only.
#[cfg(not(windows))]
async fn dispatch_native(
    _ctx: &Context,
    verb: &str,
    _args: &[String],
    _exe: &Executor,
) -> Result<String, String> {
    Err(format!("`{verb}` (native window control) is Windows only"))
}

/// Dispatches a browser/session `Raw` verb against the live page.
/// Returns `Result<String, String>` so the `?` operator resolves cleanly; the
/// caller (`dispatch`) maps it onto [`Outcome`].
async fn dispatch_raw(
    ctx: &mut Context,
    verb: &str,
    args: &[super::ast::Arg],
    exe: &Executor,
) -> Result<String, String> {
    let mut resolved = Vec::new();
    for a in args {
        match ctx.resolve(a) {
            Ok(v) => resolved.push(v),
            Err(e) => return Err(e),
        }
    }
    // `drivers` is driver-agnostic: report which drivers this run exposes and
    // which is active. (The counterpart of `plugins`.)
    if verb == "drivers" {
        let active = exe.driver();
        let browser_state = if exe.has_browser() {
            "loaded"
        } else if exe.browser_available() {
            "available"
        } else {
            "unavailable"
        };
        let desktop_state = if cfg!(windows) {
            "available"
        } else {
            "unavailable"
        };
        let mark = |driver: Driver| if driver == active { "*" } else { " " };
        return Ok(format!(
            "{} browser  {browser_state}\n{} desktop  {desktop_state}",
            mark(Driver::Browser),
            mark(Driver::Desktop),
        ));
    }

    // Context selection: `open`/`goto` are browser verbs, so they make the browser
    // the active driver too (you would not navigate then keep clicking a window).
    // Switch drivers explicitly with `import browser` / `import desktop`.
    if matches!(verb, "open" | "goto") {
        exe.set_driver(Driver::Browser);
    }
    // Native-only verbs, and shared verbs while the desktop driver is active, go
    // to UI Automation; they need no browser.
    if resolve_route(verb, exe.driver()) == Route::Desktop {
        return dispatch_native(ctx, verb, &resolved, exe).await;
    }
    // Otherwise this is a browser verb: make sure the browser driver is up (a lazy
    // run launches it here on first use), then take the page.
    exe.ensure_browser().await?;

    let page = exe.page()?;

    match verb {
        "open" | "goto" => {
            let url = super::runtime::normalize_url(&resolved.first().cloned().unwrap_or_default());
            page.navigate(url.clone())
                .await
                .map_err(|e| e.to_string())?;
            let _ = page.wait_for_navigation().await;
            Ok(format!("open {url}"))
        }
        "title" => Ok(page.title().await.unwrap_or_default()),
        "url" => Ok(page.url().await.unwrap_or_default()),
        "text" => Ok(page
            .evaluate_string("document.body ? document.body.innerText : ''".to_string())
            .await
            .unwrap_or_default()),
        "markdown" | "md" => Ok(page.markdown().await.unwrap_or_default()),
        "content" | "html" => Ok(page.content().await.unwrap_or_default()),
        "hover" => {
            let sel = resolved.first().cloned().unwrap_or_default();
            let el = Arc::clone(&page)
                .wait_for_selector(sel.clone())
                .await
                .map_err(|e| e.to_string())?;
            el.hover_mouse().await.map_err(|e| e.to_string())?;
            Ok(format!("hover {sel}"))
        }
        "mouse" => {
            // `mouse [click|move] <x> <y>|<index>|<selector>|<text>`. A leading
            // `click` moves the cursor to the target and clicks it.
            let mut args: &[String] = &resolved;
            let mut click = false;
            if let Some(first) = args.first() {
                if first.eq_ignore_ascii_case("click") {
                    click = true;
                    args = &args[1..];
                } else if first.eq_ignore_ascii_case("move") {
                    args = &args[1..];
                }
            }
            if args.len() == 2
                && let (Ok(x), Ok(y)) = (args[0].parse::<f64>(), args[1].parse::<f64>())
            {
                if click {
                    Arc::clone(&page)
                        .click_mouse(x, y)
                        .await
                        .map_err(|e| e.to_string())?;
                    return Ok(format!("click {x} {y}"));
                }
                Arc::clone(&page)
                    .move_mouse(x, y)
                    .await
                    .map_err(|e| e.to_string())?;
                return Ok(format!("mouse {x} {y}"));
            }
            let target = args.first().cloned().unwrap_or_default();
            if target.is_empty() {
                return Err(
                    "usage: mouse [click] <index|selector|text> | mouse <x> <y>".to_string()
                );
            }
            if let Ok(index) = target.parse::<u32>() {
                if click {
                    Arc::clone(&page)
                        .click_index(index)
                        .await
                        .map_err(|e| e.to_string())?;
                    Ok(format!("click {index}"))
                } else {
                    Arc::clone(&page)
                        .move_to_index(index)
                        .await
                        .map_err(|e| e.to_string())?;
                    Ok(format!("mouse {index}"))
                }
            } else if crate::interact::looks_like_a_selector(&target) {
                let el = Arc::clone(&page)
                    .wait_for_selector(target.clone())
                    .await
                    .map_err(|e| e.to_string())?;
                if click {
                    el.click_mouse().await.map_err(|e| e.to_string())?;
                    Ok(format!("click {target}"))
                } else {
                    el.hover_mouse().await.map_err(|e| e.to_string())?;
                    Ok(format!("mouse {target}"))
                }
            } else {
                match crate::interact::control_by_text(&page, &target)
                    .await
                    .map_err(|e| e.to_string())?
                {
                    Some(el) => {
                        if click {
                            el.click_mouse().await.map_err(|e| e.to_string())?;
                            Ok(format!("click {target:?}"))
                        } else {
                            el.hover_mouse().await.map_err(|e| e.to_string())?;
                            Ok(format!("mouse {target:?}"))
                        }
                    }
                    None => Err(format!("no visible element contains {target:?}")),
                }
            }
        }
        "scroll" => {
            let arg = resolved.first().cloned().unwrap_or_default();
            let js = match arg.as_str() {
                "" | "down" => "window.scrollBy(0, window.innerHeight * 0.9)".to_string(),
                "up" => "window.scrollBy(0, -window.innerHeight * 0.9)".to_string(),
                "top" => "window.scrollTo(0, 0)".to_string(),
                "bottom" => "window.scrollTo(0, document.body.scrollHeight)".to_string(),
                other => match other.parse::<i64>() {
                    Ok(pixels) => format!("window.scrollBy(0, {pixels})"),
                    Err(_) => return Err("usage: scroll <pixels|up|down|top|bottom>".to_string()),
                },
            };
            let _ = page.evaluate_string(js).await;
            Ok(format!(
                "scroll {}",
                if arg.is_empty() { "down" } else { arg.as_str() }
            ))
        }
        "find" => {
            let text = resolved.first().cloned().unwrap_or_default();
            let count = page
                .find_text(text.clone())
                .await
                .map_err(|e| e.to_string())?;
            Ok(format!("{count} match(es) for {text:?}"))
        }
        "challenge" | "detect" => {
            let report = page.detect_challenge().await.map_err(|e| e.to_string())?;
            Ok(report.to_json().to_string())
        }
        "shot" | "screenshot" => {
            let raw = resolved
                .first()
                .cloned()
                .unwrap_or_else(|| "screenshot.png".to_string());
            let path = ctx.resolve_path(&raw)?;
            let png = page.screenshot().await.map_err(|e| e.to_string())?;
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            std::fs::write(&path, &png).map_err(|e| e.to_string())?;
            Ok(format!("wrote {} ({} bytes)", path.display(), png.len()))
        }
        "shot-full" | "screenshot-full" => {
            let raw = resolved
                .first()
                .cloned()
                .unwrap_or_else(|| "screenshot.png".to_string());
            let path = ctx.resolve_path(&raw)?;
            let png = page.screenshot_full().await.map_err(|e| e.to_string())?;
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            std::fs::write(&path, &png).map_err(|e| e.to_string())?;
            Ok(format!("wrote {} ({} bytes)", path.display(), png.len()))
        }
        "media" => page.media_json().await.map_err(|e| e.to_string()),
        "download" => {
            // `download <url> <path>`: fetch through the browser. A direct file is
            // streamed; an HLS (`.m3u8`) stream is assembled from its segments.
            // The path is confined to the workspace root (see `resolve_path`).
            let url = resolved.first().cloned().unwrap_or_default();
            let raw = resolved.get(1).cloned().unwrap_or_default();
            if url.is_empty() || raw.is_empty() {
                return Err("usage: download <url> <path>".to_string());
            }
            let path = ctx.resolve_path(&raw)?;
            let target = path.to_string_lossy().into_owned();
            page.grab(url.clone(), target)
                .await
                .map_err(|e| e.to_string())
        }
        "capture" => {
            // `capture <url> <path> [seconds]`: open a page, let it play, and
            // reassemble the media it fetches itself (Media Source Extensions —
            // YouTube, Facebook). Capturing starts before navigating so the init
            // segment is seen. The path is confined to the workspace root.
            let url = resolved.first().cloned().unwrap_or_default();
            let raw = resolved.get(1).cloned().unwrap_or_default();
            if url.is_empty() || raw.is_empty() {
                return Err("usage: capture <url> <path> [seconds]".to_string());
            }
            let path = ctx.resolve_path(&raw)?;
            let seconds = resolved
                .get(2)
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(15);
            page.start_media_capture()
                .await
                .map_err(|e| e.to_string())?;
            page.navigate(super::runtime::normalize_url(&url))
                .await
                .map_err(|e| e.to_string())?;
            let _ = page.wait_for_navigation().await;
            let _ = page
                .evaluate_string(
                    "(function(){ const v=document.querySelector('video'); \
                     if (v) { v.muted = true; if (v.play) v.play().catch(function(){}); } \
                     return true; })()"
                        .to_string(),
                )
                .await;
            tokio::time::sleep(std::time::Duration::from_secs(seconds)).await;
            page.save_capture(path.to_string_lossy().into_owned())
                .await
                .map_err(|e| e.to_string())
        }
        "upload" | "set-input-files" => {
            // `<input type="file">` cannot be set from page JS; this uses CDP
            // `DOM.setFileInputFiles` under the hood. The source is confined to
            // the workspace root so a script cannot exfiltrate files by path.
            let selector = resolved.first().cloned().unwrap_or_default();
            let raw = resolved.get(1).cloned().unwrap_or_default();
            if selector.is_empty() || raw.is_empty() {
                return Err("usage: upload <selector> <path>".to_string());
            }
            let path = ctx.resolve_path(&raw)?;
            let files = serde_json::json!([path.to_string_lossy()]).to_string();
            Arc::clone(&page)
                .set_input_files(selector.clone(), files)
                .await
                .map_err(|e| e.to_string())?;
            Ok(format!("upload {selector} <- {}", path.display()))
        }
        "cookie" | "cookies" => {
            // `cookie [get [name]]` / `cookie set <name> <value> [domain] [path]`
            // / `cookie add <json>` / `cookie delete <name>` / `cookie clear`.
            // Cookies are set through CDP (`Network.setCookie`) so `HttpOnly`
            // session cookies - which page JS cannot write - can be restored.
            let sub = resolved
                .first()
                .map(|s| s.to_ascii_lowercase())
                .unwrap_or_default();
            match sub.as_str() {
                // No subcommand, or `get`/`list`: dump cookies as JSON. A name
                // argument narrows it to that one cookie.
                "" | "get" | "list" => match resolved.get(1) {
                    Some(name) if !name.is_empty() => {
                        page.cookie(name.clone()).await.map_err(|e| e.to_string())
                    }
                    _ => page.cookies().await.map_err(|e| e.to_string()),
                },
                "set" => {
                    let name = resolved.get(1).cloned().unwrap_or_default();
                    let value = resolved.get(2).cloned().unwrap_or_default();
                    if name.is_empty() {
                        return Err("usage: cookie set <name> <value> [domain] [path]".to_string());
                    }
                    let mut cookie =
                        serde_json::json!({ "name": name, "value": value, "path": "/" });
                    match resolved.get(3).filter(|d| !d.is_empty()) {
                        Some(domain) => {
                            cookie["domain"] = serde_json::Value::String(domain.clone());
                            if let Some(path) = resolved.get(4).filter(|p| !p.is_empty()) {
                                cookie["path"] = serde_json::Value::String(path.clone());
                            }
                        }
                        // `Network.setCookie` needs a `url` or a `domain`; fall
                        // back to the page's own URL so `cookie set` works on the
                        // page that is open right now.
                        None => {
                            cookie["url"] =
                                serde_json::Value::String(page.url().await.unwrap_or_default());
                        }
                    }
                    page.execute_cdp_cmd("Network.setCookie".to_string(), cookie.to_string())
                        .await
                        .map_err(|e| e.to_string())?;
                    Ok(format!("cookie set {name}"))
                }
                // `cookie add <json>` - full control (object or array) so
                // `httpOnly`/`secure`/`sameSite`/expiry survive a round-trip.
                "add" | "import" => {
                    let json = resolved.get(1).cloned().unwrap_or_default();
                    if json.is_empty() {
                        return Err("usage: cookie add <json>".to_string());
                    }
                    let parsed: serde_json::Value = serde_json::from_str(&json)
                        .map_err(|e| format!("cookie add: invalid JSON: {e}"))?;
                    let list = match parsed {
                        serde_json::Value::Array(list) => list,
                        other => vec![other],
                    };
                    for cookie in &list {
                        page.execute_cdp_cmd("Network.setCookie".to_string(), cookie.to_string())
                            .await
                            .map_err(|e| e.to_string())?;
                    }
                    Ok(format!("added {} cookie(s)", list.len()))
                }
                "delete" | "remove" | "rm" => {
                    let name = resolved.get(1).cloned().unwrap_or_default();
                    if name.is_empty() {
                        return Err("usage: cookie delete <name>".to_string());
                    }
                    page.execute_cdp_cmd(
                        "Network.deleteCookies".to_string(),
                        serde_json::json!({ "name": name }).to_string(),
                    )
                    .await
                    .map_err(|e| e.to_string())?;
                    Ok(format!("cookie delete {name}"))
                }
                "clear" | "clear-all" => {
                    page.execute_cdp_cmd(
                        "Network.clearBrowserCookies".to_string(),
                        "{}".to_string(),
                    )
                    .await
                    .map_err(|e| e.to_string())?;
                    Ok("cookies cleared".to_string())
                }
                other => Err(format!(
                    "usage: cookie [get [name]|set <name> <value> [domain] [path]|add <json>|delete <name>|clear] (got `{other}`)"
                )),
            }
        }
        "route" => {
            // `route <pattern>` intercepts and lets through (recorded);
            // `route abort <pattern>` blocks; `route fulfill <pattern> <status>
            // <body> [content-type]` serves a response; `route har <path>` loads
            // fulfill rules from a HAR file.
            let sub = resolved
                .first()
                .map(|s| s.to_ascii_lowercase())
                .unwrap_or_default();
            match sub.as_str() {
                "" => Err("usage: route <pattern> | route abort <pattern> | \
                     route fulfill <pattern> <status> <body> [content-type] | route har <path>"
                    .to_string()),
                "abort" => {
                    let pattern = resolved.get(1).cloned().unwrap_or_default();
                    if pattern.is_empty() {
                        return Err("usage: route abort <pattern>".to_string());
                    }
                    page.route_abort(pattern.clone())
                        .await
                        .map_err(|e| e.to_string())?;
                    Ok(format!("route abort {pattern}"))
                }
                "fulfill" => {
                    let pattern = resolved.get(1).cloned().unwrap_or_default();
                    let Some(body) = resolved.get(3).cloned() else {
                        return Err(
                            "usage: route fulfill <pattern> <status> <body> [content-type]"
                                .to_string(),
                        );
                    };
                    if pattern.is_empty() {
                        return Err(
                            "usage: route fulfill <pattern> <status> <body> [content-type]"
                                .to_string(),
                        );
                    }
                    let status = resolved
                        .get(2)
                        .and_then(|s| s.parse::<u16>().ok())
                        .unwrap_or(200);
                    // The core interception pump answers every fulfill with 200.
                    if status != 200 {
                        println!("note: fulfill is served with 200 regardless of {status}");
                    }
                    let content_type = resolved.get(4).cloned();
                    page.route_fulfill(pattern.clone(), body, content_type)
                        .await
                        .map_err(|e| e.to_string())?;
                    Ok(format!("route fulfill {pattern}"))
                }
                "har" => {
                    let raw = resolved.get(1).cloned().unwrap_or_default();
                    if raw.is_empty() {
                        return Err("usage: route har <path>".to_string());
                    }
                    let path = ctx.resolve_path(&raw)?;
                    page.route_from_har(path.to_string_lossy().into_owned())
                        .await
                        .map_err(|e| e.to_string())?;
                    Ok(format!("route har {}", path.display()))
                }
                pattern => {
                    page.route(pattern.to_string(), "continue".to_string(), None, None)
                        .await
                        .map_err(|e| e.to_string())?;
                    Ok(format!("route {pattern}"))
                }
            }
        }
        "unroute" => match resolved.first().filter(|p| !p.is_empty()) {
            Some(pattern) => {
                page.unroute(pattern.clone())
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(format!("route removed for {pattern}"))
            }
            None => {
                page.unroute_all().await.map_err(|e| e.to_string())?;
                Ok("routes cleared".to_string())
            }
        },
        "auth" => {
            let username = resolved.first().cloned().unwrap_or_default();
            let password = resolved.get(1).cloned().unwrap_or_default();
            if username.is_empty() {
                return Err("usage: auth <username> <password>".to_string());
            }
            page.authenticate(username.clone(), password)
                .await
                .map_err(|e| e.to_string())?;
            Ok(format!("http auth credentials set for {username}"))
        }
        "permissions" => {
            let origin = resolved.first().cloned().unwrap_or_default();
            let permissions: Vec<String> = resolved.iter().skip(1).cloned().collect();
            if origin.is_empty() || permissions.is_empty() {
                return Err("usage: permissions <origin> <permission> [permission...]".to_string());
            }
            let json = serde_json::to_string(&permissions).map_err(|e| e.to_string())?;
            exe.browser()?
                .grant_permissions(origin.clone(), json)
                .await
                .map_err(|e| e.to_string())?;
            Ok(format!(
                "granted {} permission(s) to {origin}",
                permissions.len()
            ))
        }
        "geolocation" => {
            // The core exposes no `Page` geolocation helper; use the CDP
            // `Emulation.setGeolocationOverride` command through the page.
            let first = resolved
                .first()
                .map(|s| s.to_ascii_lowercase())
                .unwrap_or_default();
            if first == "clear" {
                page.execute_cdp_cmd(
                    "Emulation.clearGeolocationOverride".to_string(),
                    "{}".to_string(),
                )
                .await
                .map_err(|e| e.to_string())?;
                return Ok("geolocation cleared".to_string());
            }
            let latitude = resolved.first().and_then(|s| s.parse::<f64>().ok());
            let longitude = resolved.get(1).and_then(|s| s.parse::<f64>().ok());
            match (latitude, longitude) {
                (Some(latitude), Some(longitude)) => {
                    let accuracy = resolved
                        .get(2)
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    page.execute_cdp_cmd(
                        "Emulation.setGeolocationOverride".to_string(),
                        serde_json::json!({
                            "latitude": latitude,
                            "longitude": longitude,
                            "accuracy": accuracy
                        })
                        .to_string(),
                    )
                    .await
                    .map_err(|e| e.to_string())?;
                    Ok(format!(
                        "geolocation {latitude},{longitude} (+/-{accuracy}m)"
                    ))
                }
                _ => Err(
                    "usage: geolocation <latitude> <longitude> [accuracy] | geolocation clear"
                        .to_string(),
                ),
            }
        }
        "storage" => {
            let sub = resolved
                .first()
                .map(|s| s.to_ascii_lowercase())
                .unwrap_or_default();
            match sub.as_str() {
                "local" | "session" => {
                    let store = if sub == "local" {
                        "localStorage"
                    } else {
                        "sessionStorage"
                    };
                    let action = resolved
                        .get(1)
                        .map(|s| s.to_ascii_lowercase())
                        .unwrap_or_default();
                    match action.as_str() {
                        "" | "get" | "list" => match resolved.get(2) {
                            Some(key) if !key.is_empty() => {
                                let raw = page
                                    .evaluate_json(format!("{store}.getItem({})", js_literal(key)))
                                    .await
                                    .map_err(|e| e.to_string())?;
                                Ok(unwrap_json_string(raw))
                            }
                            _ => page
                                .evaluate_json(format!(
                                    "Object.fromEntries(Object.entries({store}))"
                                ))
                                .await
                                .map_err(|e| e.to_string()),
                        },
                        "set" => {
                            let key = resolved.get(2).cloned().unwrap_or_default();
                            let value = resolved.get(3).cloned().unwrap_or_default();
                            if key.is_empty() {
                                return Err(format!("usage: storage {sub} set <key> <value>"));
                            }
                            page.evaluate_json(format!(
                                "{store}.setItem({}, {})",
                                js_literal(&key),
                                js_literal(&value)
                            ))
                            .await
                            .map_err(|e| e.to_string())?;
                            Ok(format!("storage {sub} set {key}"))
                        }
                        "clear" => {
                            page.evaluate_json(format!("{store}.clear()"))
                                .await
                                .map_err(|e| e.to_string())?;
                            Ok(format!("storage {sub} cleared"))
                        }
                        other => Err(format!(
                            "usage: storage {sub} [get [key]|set <key> <value>|clear] (got `{other}`)"
                        )),
                    }
                }
                "save" => {
                    let raw = resolved.get(1).cloned().unwrap_or_default();
                    if raw.is_empty() {
                        return Err("usage: storage save <path>".to_string());
                    }
                    let path = ctx.resolve_path(&raw)?;
                    let state = page.storage_state().await.map_err(|e| e.to_string())?;
                    if let Some(parent) = path.parent() {
                        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                    }
                    std::fs::write(&path, &state).map_err(|e| e.to_string())?;
                    Ok(format!(
                        "wrote storage state to {} ({} bytes)",
                        path.display(),
                        state.len()
                    ))
                }
                "restore" => {
                    let raw = resolved.get(1).cloned().unwrap_or_default();
                    if raw.is_empty() {
                        return Err("usage: storage restore <path>".to_string());
                    }
                    let path = ctx.resolve_path(&raw)?;
                    let json = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
                    page.set_storage_state(json)
                        .await
                        .map_err(|e| e.to_string())?;
                    Ok(format!("restored storage state from {}", path.display()))
                }
                _ => Err(
                    "usage: storage <local|session> [get [key]|set <key> <value>|clear] | \
                     storage save <path> | storage restore <path>"
                        .to_string(),
                ),
            }
        }
        "click" | "tap" => {
            let target = resolved.first().cloned().unwrap_or_default();
            if target.is_empty() {
                return Err("usage: click <index|selector|text>".to_string());
            }
            // A bare integer is a snapshot index, a selector-looking string is a
            // CSS selector, anything else is matched against visible text - the
            // same rule the interactive session uses.
            if let Ok(index) = target.parse::<u32>() {
                Arc::clone(&page)
                    .click_index(index)
                    .await
                    .map_err(|e| e.to_string())?;
                tokio::time::sleep(std::time::Duration::from_millis(600)).await;
                Ok(format!("click {index}"))
            } else if crate::interact::looks_like_a_selector(&target) {
                let el = Arc::clone(&page)
                    .wait_for_selector(target.clone())
                    .await
                    .map_err(|e| e.to_string())?;
                // Move the real mouse to the element and click, so the cursor
                // travels to the target instead of firing a synthetic DOM click.
                el.click_mouse().await.map_err(|e| e.to_string())?;
                Ok(format!("click {target}"))
            } else {
                match crate::interact::control_by_text(&page, &target)
                    .await
                    .map_err(|e| e.to_string())?
                {
                    Some(el) => {
                        el.click_mouse().await.map_err(|e| e.to_string())?;
                        Ok(format!("click {target:?}"))
                    }
                    None => Err(format!("no visible element contains {target:?}")),
                }
            }
        }
        "fill" => {
            let sel = resolved.first().cloned().unwrap_or_default();
            let text = resolved.get(1).cloned().unwrap_or_default();
            // A bare integer is a snapshot index (framework-rendered fields often
            // expose no stable selector) - the same rule `click` already uses.
            let el = if let Ok(index) = sel.parse::<u32>() {
                if Arc::clone(&page).click_index(index).await.is_err() {
                    let _ = page.agent_snapshot().await;
                    Arc::clone(&page)
                        .click_index(index)
                        .await
                        .map_err(|e| e.to_string())?;
                }
                page.evaluate_handle("document.activeElement".to_string())
                    .await
                    .map_err(|e| e.to_string())?
            } else {
                Arc::clone(&page)
                    .wait_for_selector(sel.clone())
                    .await
                    .map_err(|e| e.to_string())?
            };
            el.type_text(text.clone())
                .await
                .map_err(|e| e.to_string())?;
            Ok(format!("fill {sel}"))
        }
        "select" => {
            let selector = resolved.first().cloned().unwrap_or_default();
            let value = resolved.get(1).cloned().unwrap_or_default();
            if selector.is_empty() || value.is_empty() {
                return Err("usage: select <selector> <value>".to_string());
            }
            // Native `<select>` only (matches by value or label). A custom
            // listbox is driven with `click` (open it, then click the option).
            let values = serde_json::json!([value]).to_string();
            Arc::clone(&page)
                .select_option(selector.clone(), values)
                .await
                .map_err(|e| e.to_string())?;
            Ok(format!("select {selector} = {value}"))
        }
        "type" => {
            let text = resolved.first().cloned().unwrap_or_default();
            let el = page
                .evaluate_handle("document.activeElement".to_string())
                .await
                .map_err(|e| e.to_string())?;
            el.type_text(text.clone())
                .await
                .map_err(|e| e.to_string())?;
            Ok(format!("type {text}"))
        }
        "press" | "submit" | "send" => {
            let key = if verb == "press" {
                resolved.first().cloned().unwrap_or_default()
            } else {
                "Enter".to_string()
            };
            let el = page
                .evaluate_handle("document.activeElement".to_string())
                .await
                .map_err(|e| e.to_string())?;
            el.press(key.clone()).await.map_err(|e| e.to_string())?;
            Ok(format!("press {key}"))
        }
        // Unit-suffixed sleeps: the unit lives in the verb, the value is a plain
        // number (there is no `2s`/`500ms` literal in the language).
        "wait-ms" => {
            super::runtime::wait_scaled(&resolved.first().cloned().unwrap_or_default(), 1).await
        }
        "wait-sec" => {
            super::runtime::wait_scaled(&resolved.first().cloned().unwrap_or_default(), 1_000).await
        }
        "wait-min" => {
            super::runtime::wait_scaled(&resolved.first().cloned().unwrap_or_default(), 60_000)
                .await
        }
        "wait-hr" => {
            super::runtime::wait_scaled(&resolved.first().cloned().unwrap_or_default(), 3_600_000)
                .await
        }
        // Randomized sleep: two millisecond bounds (inclusive).
        "wait-random" => {
            let min = super::runtime::parse_ms(&resolved.first().cloned().unwrap_or_default())?;
            let max = super::runtime::parse_ms(&resolved.get(1).cloned().unwrap_or_default())?;
            let ms = super::runtime::random_ms(min, max);
            tokio::time::sleep(std::time::Duration::from_millis(ms as u64)).await;
            Ok(format!("wait-random {ms}ms"))
        }
        "wait" | "sleep" => {
            let arg = resolved.first().cloned().unwrap_or_default();
            if let Some(ms) = super::runtime::parse_duration_ms(&arg) {
                tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
                Ok(format!("wait {ms}ms"))
            } else if !arg.is_empty() {
                Arc::clone(&page)
                    .wait_for_selector(arg.clone())
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(format!("wait for {arg}"))
            } else {
                Ok("wait".to_string())
            }
        }
        "wait-idle" | "idle" => {
            page.wait_for_network_idle(500, 30_000)
                .await
                .map_err(|e| e.to_string())?;
            Ok("network idle".to_string())
        }
        "wait-stable" | "stable" => {
            page.wait_for_dom_stable(500, 30_000)
                .await
                .map_err(|e| e.to_string())?;
            Ok("dom stable".to_string())
        }
        "back" => {
            page.go_back().await.map_err(|e| e.to_string())?;
            Ok("back".to_string())
        }
        "reload" => {
            page.reload().await.map_err(|e| e.to_string())?;
            Ok("reload".to_string())
        }
        "eval" | "js" => {
            if !ctx.permissions.allow_eval {
                Err("`eval` is disabled (pass --allow-unsafe)".to_string())
            } else {
                Ok(page
                    .evaluate_json(resolved.first().cloned().unwrap_or_default())
                    .await
                    .unwrap_or_default())
            }
        }
        "await" => {
            // Seconds, decimal allowed: `await 20`, `await 0.5`. One verb instead
            // of the `wait-sec/min/hr/ms` zoo. Capped so a script cannot park for
            // a nonsense duration.
            let seconds: f64 = resolved
                .first()
                .ok_or("usage: await <seconds>")?
                .parse()
                .map_err(|_| "await <seconds>: seconds must be a number".to_string())?;
            let seconds = seconds.clamp(0.0, 3600.0);
            tokio::time::sleep(std::time::Duration::from_secs_f64(seconds)).await;
            Ok(format!("awaited {seconds}s"))
        }
        "tabs" => {
            let tabs = exe.tabs.lock().unwrap().clone();
            let active = exe.active.load(Ordering::Relaxed);
            for (i, tab) in tabs.iter().enumerate() {
                let url = tab.url().await.unwrap_or_default();
                let mark = if i == active { "*" } else { " " };
                println!("{mark} [{i}] {url}");
            }
            Ok(format!("{} tab(s)", tabs.len()))
        }
        "new-tab" | "newtab" | "tab-new" => {
            let url = super::runtime::normalize_url(
                &resolved
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "about:blank".to_string()),
            );
            let opened = exe
                .browser()?
                .new_page(url.clone())
                .await
                .map_err(|e| e.to_string())?;
            let index = {
                let mut tabs = exe.tabs.lock().unwrap();
                tabs.push(Arc::clone(&opened));
                tabs.len() - 1
            };
            exe.active.store(index, Ordering::Relaxed);
            Ok(format!("new-tab [{index}] {url}"))
        }
        "switch" | "tab" | "use" => {
            let count = exe.tabs.lock().unwrap().len();
            let index = match resolved.first().map(String::as_str) {
                // An index into the tabs this run opened.
                Some(arg) => match arg.parse::<usize>() {
                    Ok(index) if index < count => index,
                    Ok(index) => return Err(format!("no tab {index}; {count} open")),
                    // Otherwise a target id: a tab this run already knows about,
                    // or a popup it did not open (attached on demand). Matches the
                    // session's `switch <n|targetId>`.
                    Err(_) => {
                        let known = exe
                            .tabs
                            .lock()
                            .unwrap()
                            .iter()
                            .position(|tab| tab.target_id() == arg);
                        match known {
                            Some(index) => index,
                            None => {
                                let attached = exe
                                    .browser()?
                                    .attach_page(arg.to_string())
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let mut tabs = exe.tabs.lock().unwrap();
                                tabs.push(Arc::clone(&attached));
                                tabs.len() - 1
                            }
                        }
                    }
                },
                None => (exe.active.load(Ordering::Relaxed) + 1) % count.max(1),
            };
            exe.active.store(index, Ordering::Relaxed);
            let url = exe.page()?.url().await.unwrap_or_default();
            Ok(format!("switch [{index}] {url}"))
        }
        "close-tab" => {
            let count = exe.tabs.lock().unwrap().len();
            let arg = resolved
                .first()
                .ok_or_else(|| "usage: close-tab <index|targetId>".to_string())?;
            let target_id: String = match arg.parse::<usize>() {
                Ok(index) if index < count => {
                    exe.tabs.lock().unwrap()[index].target_id().to_string()
                }
                Ok(index) => return Err(format!("no tab {index}; {count} open")),
                Err(_) => arg.clone(),
            };
            // Never close the last tab: nothing would be left to drive.
            if count <= 1 {
                return Err("cannot close the last tab".to_string());
            }
            let params = serde_json::json!({ "targetId": target_id.as_str() }).to_string();
            exe.page()?
                .execute_cdp_cmd("Target.closeTarget".to_string(), params)
                .await
                .map_err(|e| e.to_string())?;
            let remaining = {
                let mut tabs = exe.tabs.lock().unwrap();
                if let Some(position) = tabs
                    .iter()
                    .position(|tab| tab.target_id() == target_id.as_str())
                {
                    tabs.remove(position);
                }
                tabs.len()
            };
            let active = exe
                .active
                .load(Ordering::Relaxed)
                .min(remaining.saturating_sub(1));
            exe.active.store(active, Ordering::Relaxed);
            Ok(format!("closed tab {target_id}; {remaining} left"))
        }
        "dialog" => {
            let arg = resolved.first().map(String::as_str).unwrap_or("");
            let policy = match arg.to_ascii_lowercase().as_str() {
                "" | "dismiss" => xcelerate::page::DialogPolicy::Dismiss,
                "accept" => xcelerate::page::DialogPolicy::Accept,
                other => {
                    return Err(format!("usage: dialog <dismiss|accept> (got {other:?})"));
                }
            };
            page.set_dialog_policy(policy)
                .await
                .map_err(|e| e.to_string())?;
            Ok(format!(
                "dialogs: {}",
                if arg.is_empty() { "dismiss" } else { arg }
            ))
        }
        "drag" => {
            let from = resolved
                .first()
                .cloned()
                .ok_or_else(|| "usage: drag <from> <to>".to_string())?;
            let to = resolved
                .get(1)
                .cloned()
                .ok_or_else(|| "usage: drag <from> <to>".to_string())?;
            Arc::clone(&page)
                .drag(&from, &to)
                .await
                .map_err(|e| e.to_string())?;
            Ok(format!("dragged {from} -> {to}"))
        }
        other => Err(format!("unknown verb `{other}`")),
    }
}

/// Renders a Rust string as a JavaScript string literal, so a storage key or
/// value containing quotes, backslashes or newlines cannot break out of the
/// evaluated expression.
fn js_literal(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string())
}

/// Unwraps a top-level JSON string result (as `evaluate_json` returns for a JS
/// string) so `storage local get` prints the raw value, not a quoted one.
fn unwrap_json_string(raw: String) -> String {
    match serde_json::from_str::<serde_json::Value>(&raw) {
        Ok(serde_json::Value::String(text)) => text,
        _ => raw,
    }
}

/// Executes a `request` command: enforces HTTP permission + SSRF private-range
/// guard, then performs the HTTP call (when the `http` feature is enabled).
async fn request(
    ctx: &mut Context,
    method: &str,
    url: &super::ast::Arg,
    headers: &Option<super::ast::Arg>,
    body: &Option<super::ast::Arg>,
) -> Outcome {
    if !ctx.permissions.allow_http {
        return Outcome::fail("`request` is disabled (pass --allow-http or --allow-unsafe)");
    }
    let url = match ctx.resolve(url) {
        Ok(u) => u,
        Err(e) => return Outcome::fail(e),
    };
    if let Some(host) = host_of(&url)
        && crate::security::is_private_host(&host)
        && !ctx.permissions.allow_private
    {
        return Outcome::fail(format!(
            "blocked: `{host}` is a private/metadata host (pass --allow-private)"
        ));
    }
    // The literal check above cannot catch a public name that *resolves* to a
    // private address (DNS rebinding, `*.nip.io`, `*.sslip.io`). Resolve and
    // re-check unless the human explicitly allowed private hosts.
    if !ctx.permissions.allow_private
        && let Some(host) = host_of(&url)
    {
        let port = url
            .split("://")
            .nth(1)
            .and_then(|rest| rest.split(['/', '?', '#']).next())
            .and_then(|authority| authority.rsplit_once(':'))
            .and_then(|(_, p)| p.parse::<u16>().ok())
            .unwrap_or(if url.starts_with("https") { 443 } else { 80 });
        let clean = host.trim_matches(['[', ']']).to_string();
        if let Ok(addrs) = tokio::net::lookup_host((clean.as_str(), port)).await {
            for addr in addrs {
                if crate::security::is_private_ip(addr.ip()) {
                    return Outcome::fail(format!(
                        "blocked: `{host}` resolves to a private address ({})",
                        addr.ip()
                    ));
                }
            }
        }
    }

    #[cfg(not(feature = "http"))]
    {
        let _ = (method, url, headers, body);
        Outcome::fail("`request` requires a build with the `http` feature")
    }

    #[cfg(feature = "http")]
    {
        let headers_json = headers.as_ref().map(|h| ctx.resolve(h)).transpose();
        let body_val = body.as_ref().map(|b| ctx.resolve(b)).transpose();
        let header_map = match headers_json {
            Ok(Some(h)) => serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&h)
                .map(|m| {
                    m.into_iter()
                        .map(|(k, v)| (k, v.as_str().unwrap_or_default().to_string()))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
            _ => Vec::new(),
        };
        let body_str = match body_val {
            Ok(Some(b)) => b,
            _ => String::new(),
        };
        match crate::net::request(method, &url, header_map, body_str).await {
            Ok((status, response)) => {
                ctx.vars.insert("STATUS".into(), status.to_string());
                ctx.vars.insert("RESPONSE_BODY".into(), response);
                Outcome::ok(format!("http {status}"))
            }
            Err(e) => Outcome::fail(e.to_string()),
        }
    }
}

/// Extracts a lowercased host from a URL (mirrors `policy::host_of`).
fn host_of(url: &str) -> Option<String> {
    let after = url
        .find("://")
        .map(|i| &url[i + 3..])
        .or_else(|| url.strip_prefix("//"))?;
    let authority = after.split(['/', '?', '#']).next().unwrap_or("");
    let host_port = authority
        .rsplit_once('@')
        .map(|(_, h)| h)
        .unwrap_or(authority);
    // A bracketed IPv6 literal keeps its brackets; a trailing `:port` is stripped
    // only for the non-bracketed (host or IPv4) case.
    let host = if host_port.starts_with('[') {
        let end = host_port.find(']')?;
        &host_port[..=end]
    } else {
        host_port
            .rsplit_once(':')
            .map(|(h, _)| h)
            .unwrap_or(host_port)
    };
    let host = host.trim();
    if host.is_empty() {
        None
    } else {
        Some(host.to_ascii_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::{Driver, Route, is_native_only, is_shared, resolve_route};

    #[test]
    fn native_only_verbs_are_unprefixed() {
        for verb in ["launch", "window", "tree", "key", "wheel"] {
            assert!(is_native_only(verb), "{verb} should be native-only");
        }
        assert!(!is_native_only("open"));
        assert!(!is_native_only("click"));
    }

    #[test]
    fn shared_verbs_follow_the_active_driver() {
        for verb in ["click", "find", "wait", "scroll", "fill"] {
            assert!(is_shared(verb), "{verb} should be shared");
            assert_eq!(resolve_route(verb, Driver::Browser), Route::Browser);
            assert_eq!(resolve_route(verb, Driver::Desktop), Route::Desktop);
        }
    }

    #[test]
    fn native_only_ignores_the_driver() {
        assert_eq!(resolve_route("launch", Driver::Browser), Route::Desktop);
        assert_eq!(resolve_route("key", Driver::Browser), Route::Desktop);
    }

    #[test]
    fn browser_verbs_stay_on_the_browser() {
        assert_eq!(resolve_route("open", Driver::Desktop), Route::Browser);
        assert_eq!(resolve_route("title", Driver::Desktop), Route::Browser);
    }
}
