//! Native always-on-top overlay for xcelerate runs.
//!
//! Built directly on `winit` + `egui` (via `egui-winit` and `egui_glow`), not
//! `eframe`. The overlay must track the browser window closely, and here that is
//! a direct `Window::set_outer_position` on the event thread: no repaint, no
//! viewport-command queue and no vsync stand between a move and the screen.

pub mod codegen;
mod codegen_ui;
mod fonts;
mod highlight;
pub mod icons;

use std::sync::{Arc, Mutex};

use egui::Vec2;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalPosition, PhysicalSize};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::window::{Window, WindowId, WindowLevel};

pub use codegen::{Action, Language};
pub use codegen_ui::CodegenUi;

/// The window content the shell drives: the codegen overlay.
pub(crate) trait View {
    /// The window size the view currently wants, in logical pixels.
    fn desired_size(&self) -> Vec2;
    /// Whether the view may appear yet.
    fn ready(&mut self, elapsed_secs: f32, has_bounds: bool) -> bool;
    /// Where the window should sit, in physical pixels.
    fn target_position(&self, scale: f32, monitor_px: Option<(f32, f32)>) -> Option<(f32, f32)>;
    /// Applies the view's style and fonts to the context.
    fn configure(&self, ctx: &egui::Context);
    /// Draws the whole view into the root `Ui`.
    fn draw(&mut self, ui: &mut egui::Ui);
    /// Whether the browser window's rectangle is known.
    fn has_bounds(&self) -> bool;
    /// Whether the window was asked to close.
    fn is_closed(&self) -> bool;
    /// Attaches the event-loop proxy the view's handle wakes.
    fn attach(&self, proxy: EventLoopProxy<UserEvent>);
    /// Whether the window should cover the whole browser window (OS-level gate).
    fn covers_browser(&self) -> bool {
        false
    }
    /// The browser window rectangle, when known.
    fn browser_bounds(&self) -> Option<Bounds> {
        None
    }
}

/// The browser window's screen rectangle, in logical (CSS) pixels.
#[derive(Clone, Copy, Debug)]
pub struct Bounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

pub(crate) struct State {
    pub(crate) closed: bool,
    pub(crate) bounds: Option<Bounds>,
    /// The codegen target language; `None` until the codegen overlay is enabled.
    pub(crate) codegen: Option<Language>,
    pub(crate) actions: Vec<Action>,
    pub(crate) recording: bool,
    /// OS-level gate: cover the whole browser window so the OS routes input to
    /// the overlay instead of the page behind it.
    pub(crate) cover: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            closed: false,
            bounds: None,
            codegen: None,
            actions: Vec::new(),
            recording: true,
            cover: false,
        }
    }
}

/// What the event loop reacts to.
#[derive(Debug, Clone)]
pub enum UserEvent {
    /// The browser window moved or resized: reposition only, no repaint.
    Reposition,
    /// Something the UI shows changed: repaint.
    Wake,
    /// egui asked for a repaint after `delay`.
    Redraw(std::time::Duration),
}

/// A cheap, cloneable handle to the overlay's shared state.
///
/// The host pushes state in; the window is driven by the event loop, which the
/// handle wakes through a [`winit::event_loop::EventLoopProxy`].
#[derive(Clone, Default)]
pub struct OverlayHandle {
    state: Arc<Mutex<State>>,
    proxy: Arc<Mutex<Option<EventLoopProxy<UserEvent>>>>,
}

impl OverlayHandle {
    /// Creates a detached handle. Call [`run`] to show the window.
    pub fn new() -> Self {
        Self::default()
    }

    /// Pins the overlay to the browser window; `None` falls back to the monitor.
    pub fn set_bounds(&self, bounds: Option<Bounds>) {
        {
            let mut state = self.lock();
            if !changed(state.bounds, bounds) {
                return;
            }
            state.bounds = bounds;
        }
        self.wake(UserEvent::Reposition);
    }

    /// Whether the browser window's rectangle is known.
    pub(crate) fn has_bounds(&self) -> bool {
        self.lock().bounds.is_some()
    }

    /// Asks the window to close; the session calls this when the run ends.
    pub fn request_close(&self) {
        self.lock().closed = true;
        self.wake(UserEvent::Wake);
    }

    /// Whether the window was asked to close.
    pub fn is_closed(&self) -> bool {
        self.lock().closed
    }

    /// Switches the overlay to the codegen view, rendering in `language`.
    pub fn enable_codegen(&self, language: Language) {
        self.lock().codegen = Some(language);
        self.wake(UserEvent::Wake);
    }

    /// The codegen target language, or `None` until the overlay is enabled.
    pub fn codegen_language(&self) -> Option<Language> {
        self.lock().codegen
    }

    /// Sets the language the generated script is rendered in.
    pub fn set_codegen_language(&self, language: Language) {
        self.lock().codegen = Some(language);
        self.wake(UserEvent::Wake);
    }

    /// A snapshot of the recorded codegen actions.
    pub fn actions(&self) -> Vec<Action> {
        self.lock().actions.clone()
    }

    /// Appends one recorded action.
    pub fn push_action(&self, action: Action) {
        self.lock().actions.push(action);
        self.wake(UserEvent::Wake);
    }

    /// Clears the recorded actions.
    pub fn clear_actions(&self) {
        self.lock().actions.clear();
        self.wake(UserEvent::Wake);
    }

    /// Marks the codegen recorder running (the indicator pulses).
    pub fn set_recording(&self, recording: bool) {
        self.lock().recording = recording;
        self.wake(UserEvent::Wake);
    }

    /// Whether the codegen recorder is running.
    pub fn is_recording(&self) -> bool {
        self.lock().recording
    }

    /// Covers the browser window with the overlay (OS-level input gate).
    pub fn set_cover(&self, cover: bool) {
        self.lock().cover = cover;
        self.wake(UserEvent::Reposition);
    }

    /// Whether the overlay should cover the whole browser window.
    pub fn covers(&self) -> bool {
        self.lock().cover
    }

    pub(crate) fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn attach(&self, proxy: EventLoopProxy<UserEvent>) {
        *self
            .proxy
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(proxy);
    }

    fn wake(&self, event: UserEvent) {
        if let Some(proxy) = self
            .proxy
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
        {
            let _ = proxy.send_event(event);
        }
    }
}

/// Whether two window rectangles differ enough to be worth acting on.
fn changed(a: Option<Bounds>, b: Option<Bounds>) -> bool {
    match (a, b) {
        (Some(old), Some(new)) => {
            (old.x - new.x).abs() > 0.5
                || (old.y - new.y).abs() > 0.5
                || (old.width - new.width).abs() > 0.5
                || (old.height - new.height).abs() > 0.5
        }
        (None, None) => false,
        _ => true,
    }
}

/// Runs the codegen overlay event loop. Blocks until the window is closed.
pub fn run(handle: OverlayHandle) -> Result<(), Box<dyn std::error::Error>> {
    run_view(Box::new(CodegenUi::new(handle)))
}

fn run_view(view: Box<dyn View>) -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;
    view.attach(event_loop.create_proxy());
    let mut shell = Shell::new(view);
    event_loop.run_app(&mut shell)?;
    Ok(())
}

/// The window, its GL context and surface.
struct GlWindow {
    window: Window,
    context: glutin::context::PossiblyCurrentContext,
    /// Kept alive for the lifetime of the context/surface; never read directly.
    _display: glutin::display::Display,
    surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
}

impl GlWindow {
    /// Creates the window, its GL context and its surface. `size` is the initial
    /// inner size in logical pixels.
    fn new(
        event_loop: &ActiveEventLoop,
        size: [f32; 2],
    ) -> Result<(Self, glow::Context), Box<dyn std::error::Error>> {
        use glutin::context::NotCurrentGlContext as _;
        use glutin::display::GetGlDisplay as _;
        use glutin::display::GlDisplay as _;
        use glutin::surface::GlSurface as _;
        use winit::raw_window_handle::HasWindowHandle as _;

        let [width, height] = size;
        let attributes = Window::default_attributes()
            .with_title("xcelerate")
            .with_decorations(false)
            .with_resizable(false)
            .with_transparent(true)
            .with_window_level(WindowLevel::AlwaysOnTop)
            // Hidden until the browser window exists (see `Shell::sync_window`),
            // so the bar does not flash into being before the browser.
            .with_visible(false)
            .with_inner_size(LogicalSize::new(width as f64, height as f64));
        #[cfg(windows)]
        let attributes = {
            use winit::platform::windows::WindowAttributesExtWindows as _;
            // No taskbar entry: a companion to the browser, not its own app.
            attributes.with_skip_taskbar(true)
        };

        let template = glutin::config::ConfigTemplateBuilder::new()
            .prefer_hardware_accelerated(None)
            .with_depth_size(0)
            .with_stencil_size(0)
            .with_transparency(true);

        let (window, config) = glutin_winit::DisplayBuilder::new()
            .with_preference(glutin_winit::ApiPreference::FallbackEgl)
            .with_window_attributes(Some(attributes.clone()))
            .build(event_loop, template, |mut configs| {
                configs
                    .next()
                    .expect("no matching GL configuration for the overlay")
            })?;
        let display = config.display();

        let raw = window
            .as_ref()
            .and_then(|window| window.window_handle().ok())
            .map(|handle| handle.as_raw());
        let context_attributes = glutin::context::ContextAttributesBuilder::new().build(raw);
        let fallback = glutin::context::ContextAttributesBuilder::new()
            .with_context_api(glutin::context::ContextApi::Gles(None))
            .build(raw);
        let not_current = unsafe { display.create_context(&config, &context_attributes) }
            .or_else(|_| unsafe { display.create_context(&config, &fallback) })?;

        let window = match window {
            Some(window) => window,
            None => glutin_winit::finalize_window(event_loop, attributes, &config)?,
        };

        let size = window.inner_size();
        let surface_attributes =
            glutin::surface::SurfaceAttributesBuilder::<glutin::surface::WindowSurface>::new()
                .build(
                    window.window_handle()?.as_raw(),
                    std::num::NonZeroU32::new(size.width.max(1)).unwrap(),
                    std::num::NonZeroU32::new(size.height.max(1)).unwrap(),
                );
        let surface = unsafe { display.create_window_surface(&config, &surface_attributes) }?;
        let context = not_current.make_current(&surface)?;
        // No vsync: waiting for a refresh would delay every reposition.
        let _ = surface.set_swap_interval(&context, glutin::surface::SwapInterval::DontWait);

        let gl = unsafe {
            glow::Context::from_loader_function(|symbol| match std::ffi::CString::new(symbol) {
                Ok(symbol) => display.get_proc_address(&symbol),
                Err(_) => std::ptr::null(),
            })
        };

        Ok((
            Self {
                window,
                context,
                _display: display,
                surface,
            },
            gl,
        ))
    }

    fn resize(&self, size: PhysicalSize<u32>) {
        use glutin::surface::GlSurface as _;
        if let (Some(width), Some(height)) = (
            std::num::NonZeroU32::new(size.width),
            std::num::NonZeroU32::new(size.height),
        ) {
            self.surface.resize(&self.context, width, height);
        }
    }

    fn swap(&self) -> Result<(), Box<dyn std::error::Error>> {
        use glutin::surface::GlSurface as _;
        self.surface.swap_buffers(&self.context)?;
        Ok(())
    }
}

/// The winit application: owns the window, the GL context and the view.
struct Shell {
    view: Box<dyn View>,
    started: std::time::Instant,
    gl_window: Option<GlWindow>,
    gl: Option<Arc<glow::Context>>,
    egui_glow: Option<egui_glow::EguiGlow>,
    repaint_delay: std::time::Duration,
    shown: bool,
    last_top: std::time::Instant,
}

impl Shell {
    fn new(view: Box<dyn View>) -> Self {
        let now = std::time::Instant::now();
        Self {
            view,
            started: now,
            gl_window: None,
            gl: None,
            egui_glow: None,
            repaint_delay: std::time::Duration::MAX,
            shown: false,
            last_top: now,
        }
    }

    /// Sizes and positions the window: bottom-center of the browser window, or
    /// off-screen until the view is ready. With the OS-level gate on, the window
    /// instead covers the whole browser window.
    fn sync_window(&mut self) {
        let Some(gl_window) = self.gl_window.as_ref() else {
            return;
        };
        let window = &gl_window.window;
        let scale = window.scale_factor() as f32;

        if !self
            .view
            .ready(self.started.elapsed().as_secs_f32(), self.view.has_bounds())
        {
            window.set_outer_position(PhysicalPosition::new(-10_000, -10_000));
            return;
        }
        if !self.shown {
            self.shown = true;
            window.set_visible(true);
        }

        // OS-level gate: cover the browser window so the OS routes input here.
        if self.view.covers_browser()
            && let Some(bounds) = self.view.browser_bounds()
            && bounds.width > 1.0
            && bounds.height > 1.0
        {
            let size = PhysicalSize::new(
                ((bounds.width * scale as f64).round() as u32).max(1),
                ((bounds.height * scale as f64).round() as u32).max(1),
            );
            if window.inner_size() != size {
                let _ = window.request_inner_size(size);
            }
            window.set_outer_position(PhysicalPosition::new(
                (bounds.x * scale as f64).round(),
                (bounds.y * scale as f64).round(),
            ));
            return;
        }

        let want = self.view.desired_size();
        let size = PhysicalSize::new(
            ((want.x * scale).round() as u32).max(1),
            ((want.y * scale).round() as u32).max(1),
        );
        if window.inner_size() != size {
            let _ = window.request_inner_size(size);
        }
        let monitor = window
            .current_monitor()
            .map(|monitor| monitor.size())
            .map(|size| (size.width as f32, size.height as f32));
        if let Some((x, y)) = self.view.target_position(scale, monitor) {
            window.set_outer_position(PhysicalPosition::new(x.round(), y.round()));
        }
    }

    fn redraw(&mut self, event_loop: &ActiveEventLoop) {
        let (Some(gl_window), Some(egui_glow), Some(gl)) = (
            self.gl_window.as_ref(),
            self.egui_glow.as_mut(),
            self.gl.as_ref(),
        ) else {
            return;
        };

        let view = self.view.as_mut();
        egui_glow.run(&gl_window.window, |root| view.draw(root));

        if self.view.is_closed() {
            event_loop.exit();
            return;
        }

        // The framebuffer is cleared to fully transparent so the window shows only
        // the panel, over whatever is behind it.
        unsafe {
            use glow::HasContext as _;
            gl.clear_color(0.0, 0.0, 0.0, 0.0);
            gl.clear(glow::COLOR_BUFFER_BIT);
        }
        egui_glow.paint(&gl_window.window);
        let _ = gl_window.swap();

        let delay = self.repaint_delay;
        event_loop.set_control_flow(if delay.is_zero() {
            gl_window.window.request_redraw();
            ControlFlow::Poll
        } else if let Some(instant) = std::time::Instant::now().checked_add(delay) {
            ControlFlow::WaitUntil(instant)
        } else {
            ControlFlow::Wait
        });
    }
}

impl ApplicationHandler<UserEvent> for Shell {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gl_window.is_some() {
            return;
        }
        let initial = self.view.desired_size();
        let (gl_window, gl) = match GlWindow::new(event_loop, [initial.x, initial.y]) {
            Ok(pair) => pair,
            Err(error) => {
                eprintln!("xcelerate overlay: {error}");
                event_loop.exit();
                return;
            }
        };
        let gl = Arc::new(gl);
        let egui_glow = egui_glow::EguiGlow::new(event_loop, Arc::clone(&gl), None, None, true);
        egui_extras::install_image_loaders(&egui_glow.egui_ctx);
        self.view.configure(&egui_glow.egui_ctx);
        self.gl_window = Some(gl_window);
        self.gl = Some(gl);
        self.egui_glow = Some(egui_glow);
        self.sync_window();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let (Some(gl_window), Some(egui_glow)) = (self.gl_window.as_ref(), self.egui_glow.as_mut())
        else {
            return;
        };

        match &event {
            WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                event_loop.exit();
                return;
            }
            WindowEvent::RedrawRequested => {
                self.redraw(event_loop);
                return;
            }
            WindowEvent::Resized(size) => {
                gl_window.resize(*size);
                return;
            }
            _ => {}
        }

        let response = egui_glow.on_window_event(&gl_window.window, &event);
        if response.repaint {
            gl_window.window.request_redraw();
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::Redraw(delay) => {
                self.repaint_delay = delay;
                // A zero delay means "paint as soon as possible": asking for the
                // redraw here keeps animations from stalling between events.
                if delay.is_zero()
                    && let Some(gl_window) = &self.gl_window
                {
                    gl_window.window.request_redraw();
                }
            }
            // A move is applied straight away, painting nothing.
            UserEvent::Reposition => self.sync_window(),
            UserEvent::Wake => {
                self.sync_window();
                if self.view.is_closed() {
                    event_loop.exit();
                    return;
                }
                if let Some(gl_window) = &self.gl_window {
                    gl_window.window.request_redraw();
                }
            }
        }
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: winit::event::StartCause) {
        // The repaint timer fired: without this the loop would simply sleep on,
        // leaving animations (like the fade) frozen mid-way.
        if let winit::event::StartCause::ResumeTimeReached { .. } = cause
            && let Some(gl_window) = &self.gl_window
        {
            gl_window.window.request_redraw();
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Windows can drop a topmost window behind another topmost one; re-assert
        // the level so the overlay stays visible while the browser is not focused.
        if self.last_top.elapsed() > std::time::Duration::from_millis(500) {
            self.last_top = std::time::Instant::now();
            if let Some(gl_window) = &self.gl_window {
                gl_window.window.set_window_level(WindowLevel::AlwaysOnTop);
            }
        }
        // Keep waking often enough to notice focus/bounds changes even when the
        // UI itself is idle.
        if !self.repaint_delay.is_zero() {
            event_loop.set_control_flow(ControlFlow::WaitUntil(
                std::time::Instant::now() + std::time::Duration::from_millis(100),
            ));
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(egui_glow) = &mut self.egui_glow {
            egui_glow.destroy();
        }
    }
}
