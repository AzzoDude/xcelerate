//! The codegen overlay: a JetBrains-style header over a code editor.
//!
//! Independent of the browser's focus: it stays put while the AI works and only
//! fades in once. It reads the recorded actions and the target language straight
//! off the shared [`OverlayHandle`], so the session and the window share one
//! handle.

use std::time::Duration;

use egui::{Align, Color32, CornerRadius, FontId, Frame, Layout, Margin, RichText, Sense, Vec2};
use winit::event_loop::EventLoopProxy;

use crate::codegen::Language;
use crate::{Bounds, OverlayHandle, UserEvent, View, fonts, highlight, icons};

/// Window width.
const WIN_W: f32 = 560.0;
/// Height of the header bar.
const NAV_H: f32 = 34.0;
/// Height of the separator under the header.
const BORDER_H: f32 = 1.0;
/// Height of the editor.
const CODE_H: f32 = 300.0;

// Darcula / New UI palette.
const CARD: Color32 = Color32::from_rgba_premultiplied(30, 31, 34, 250);
const HEADER: Color32 = Color32::from_rgba_premultiplied(42, 44, 47, 250);
const EDITOR: Color32 = Color32::from_rgba_premultiplied(26, 27, 30, 250);
const BORDER: Color32 = Color32::from_rgb(57, 59, 64);
const FG: Color32 = Color32::from_rgb(223, 225, 229);
const MUTED: Color32 = Color32::from_rgba_premultiplied(150, 153, 161, 170);
/// JetBrains blue (New UI accent).
const ACCENT: Color32 = Color32::from_rgb(53, 116, 240);
/// Darcula red, for the recording dot.
const RED: Color32 = Color32::from_rgb(219, 88, 96);
const HOVER: Color32 = Color32::from_rgba_premultiplied(255, 255, 255, 20);
const ACTIVE: Color32 = Color32::from_rgba_premultiplied(255, 255, 255, 34);
const SELECTION: Color32 = Color32::from_rgb(33, 66, 131);
/// Near-invisible fill painted across the whole window in OS-gate mode so the
/// OS treats the window as hit-testable and routes input to it, not the page.
const HIT_TEST_FILL: Color32 = Color32::from_rgba_premultiplied(0, 0, 0, 1);

/// The panel's height: header, separator, editor.
fn panel_height() -> f32 {
    NAV_H + BORDER_H + CODE_H
}

/// The codegen overlay, drawn each frame by the winit shell.
pub struct CodegenUi {
    handle: OverlayHandle,
    /// Set once the fade-in may begin.
    ready: bool,
    /// Current opacity, eased toward its target each frame.
    alpha: f32,
}

impl CodegenUi {
    /// Creates the UI for a handle. [`apply_style`] sets the look separately.
    pub fn new(handle: OverlayHandle) -> Self {
        Self {
            handle,
            ready: false,
            alpha: 0.0,
        }
    }

    /// The language currently selected, defaulting to Python.
    fn language(&self) -> Language {
        self.handle.codegen_language().unwrap_or(Language::Python)
    }

    fn paint(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        // Fade in once; this window does not track the browser's focus.
        let dt = ctx.input(|input| input.stable_dt).clamp(0.001, 0.1);
        let target = if self.ready { 1.0 } else { 0.0 };
        self.alpha += (target - self.alpha) * (8.0 * dt).min(1.0);
        if (target - self.alpha).abs() > 0.002 {
            ctx.request_repaint();
        }
        ui.set_opacity(self.alpha);
        // Keep the pulse and any recorder-side changes ticking without input.
        ctx.request_repaint_after(Duration::from_millis(500));

        let language = self.language();
        let actions = self.handle.actions();
        let code = language.generate(&actions);
        let count = actions.len();

        if self.handle.covers() {
            // OS-level gate: fill the whole (browser-sized) window so the OS
            // delivers input here, then pin the panel to the bottom-center.
            ui.painter().rect_filled(ui.max_rect(), 0.0, HIT_TEST_FILL);
            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                let push = (ui.available_height() - panel_height()).max(0.0);
                ui.add_space(push);
                self.panel(ui, &code, language, count);
            });
        } else {
            self.panel(ui, &code, language, count);
        }
    }

    /// The header over the editor, as one rounded card.
    fn panel(&self, ui: &mut egui::Ui, code: &str, language: Language, count: usize) {
        Frame::NONE
            .fill(CARD)
            .corner_radius(CornerRadius::same(6))
            .inner_margin(Margin::ZERO)
            .show(ui, |ui| {
                ui.set_width(WIN_W);
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                ui.vertical(|ui| {
                    self.navbar(ui, code, language, count);
                    Frame::NONE.fill(BORDER).show(ui, |ui| {
                        ui.set_width(WIN_W);
                        ui.set_height(BORDER_H);
                    });
                    self.code_panel(ui, code, language);
                });
            });
    }

    /// The header bar: brand, recording status, language menu and buttons.
    fn navbar(&self, ui: &mut egui::Ui, code: &str, language: Language, count: usize) {
        Frame::NONE
            .fill(HEADER)
            .corner_radius(CornerRadius {
                nw: 6,
                ne: 6,
                sw: 0,
                se: 0,
            })
            .inner_margin(Margin::symmetric(10, 0))
            .show(ui, |ui| {
                ui.set_width(WIN_W - 20.0);
                ui.allocate_ui_with_layout(
                    Vec2::new(WIN_W - 20.0, NAV_H),
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        ui.add(icons::image("robot", 16.0, ACCENT));
                        ui.add_space(7.0);
                        ui.label(RichText::new("xcelerate").font(brand(13.0)).color(FG));
                        ui.add_space(3.0);
                        ui.label(RichText::new("codegen").font(brand(13.0)).color(ACCENT));

                        ui.add_space(14.0);
                        self.status(ui, count);

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let mut chosen = language;
                            egui::ComboBox::from_id_salt("xcelerate-codegen-language")
                                .selected_text(RichText::new(chosen.label()).size(12.0).color(FG))
                                .width(140.0)
                                .show_ui(ui, |ui| {
                                    for option in Language::ALL {
                                        ui.selectable_value(&mut chosen, option, option.label());
                                    }
                                });
                            if chosen != self.language() {
                                self.handle.set_codegen_language(chosen);
                            }
                            ui.add_space(6.0);
                            if icon_button(ui, "x", "Close").clicked() {
                                self.handle.request_close();
                            }
                            if icon_button(ui, "trash", "Clear the recording").clicked() {
                                self.handle.clear_actions();
                            }
                            if icon_button(ui, "copy", "Copy the generated script").clicked() {
                                ui.ctx().copy_text(code.to_owned());
                            }
                        });
                    },
                );
            });
    }

    /// The recording indicator, status word and action count.
    fn status(&self, ui: &mut egui::Ui, count: usize) {
        let recording = self.handle.is_recording();
        let time = ui.input(|input| input.time) as f32;
        let pulse = if recording {
            0.35 + 0.65 * (0.5 + 0.5 * (time * 3.0).sin())
        } else {
            1.0
        };
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(9.0), Sense::hover());
        let color = if recording { RED } else { MUTED };
        ui.painter()
            .circle_filled(rect.center(), 3.1, color.gamma_multiply(pulse));
        ui.add_space(6.0);
        ui.label(
            RichText::new(if recording { "recording" } else { "paused" })
                .size(11.0)
                .color(MUTED),
        );
        ui.add_space(6.0);
        ui.label(
            RichText::new(format!(
                "{count} action{}",
                if count == 1 { "" } else { "s" }
            ))
            .size(11.0)
            .color(MUTED),
        );
    }

    /// The editor: a scrollable, line-numbered, highlighted script.
    fn code_panel(&self, ui: &mut egui::Ui, code: &str, language: Language) {
        Frame::NONE
            .fill(EDITOR)
            .corner_radius(CornerRadius {
                nw: 0,
                ne: 0,
                sw: 6,
                se: 6,
            })
            .inner_margin(Margin::same(8))
            .show(ui, |ui| {
                let height = CODE_H - 16.0;
                ui.set_width(WIN_W - 16.0);
                ui.set_height(height);
                egui::ScrollArea::both()
                    .max_height(height)
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        let job = highlight::job(code, language);
                        ui.add(
                            egui::Label::new(job)
                                .selectable(true)
                                .wrap_mode(egui::TextWrapMode::Extend),
                        );
                    });
            });
    }
}

impl View for CodegenUi {
    fn desired_size(&self) -> Vec2 {
        Vec2::new(WIN_W, panel_height())
    }

    fn ready(&mut self, elapsed_secs: f32, _has_bounds: bool) -> bool {
        // This window shows on its own; a short grace period covers startup.
        if !self.ready && elapsed_secs > 0.35 {
            self.ready = true;
        }
        self.ready
    }

    fn target_position(&self, scale: f32, monitor_px: Option<(f32, f32)>) -> Option<(f32, f32)> {
        let want = self.desired_size();
        let browser = self.handle.lock().bounds.map(|bounds| {
            (
                bounds.x as f32,
                bounds.y as f32,
                bounds.width as f32,
                bounds.height as f32,
            )
        });
        let (x, y, width, height) = match browser {
            Some(area) if area.2 > 1.0 && area.3 > 1.0 => area,
            _ => {
                let (monitor_w, monitor_h) = monitor_px?;
                (0.0, 0.0, monitor_w / scale, monitor_h / scale)
            }
        };
        let px = x + (width - want.x) / 2.0;
        let py = y + height - want.y - 24.0;
        Some((px * scale, py * scale))
    }

    fn configure(&self, ctx: &egui::Context) {
        apply_style(ctx);
    }

    fn draw(&mut self, ui: &mut egui::Ui) {
        self.paint(ui);
    }

    fn has_bounds(&self) -> bool {
        self.handle.has_bounds()
    }

    fn is_closed(&self) -> bool {
        self.handle.is_closed()
    }

    fn attach(&self, proxy: EventLoopProxy<UserEvent>) {
        self.handle.attach(proxy);
    }

    fn covers_browser(&self) -> bool {
        self.handle.covers()
    }

    fn browser_bounds(&self) -> Option<Bounds> {
        self.handle.lock().bounds
    }
}

/// Applies the codegen overlay's Darcula styling and font to an egui context.
fn apply_style(ctx: &egui::Context) {
    ctx.set_theme(egui::ThemePreference::Dark);
    fonts::install(ctx);
    ctx.style_mut_of(egui::Theme::Dark, |style| {
        style.visuals = egui::Visuals::dark();
        style.visuals.panel_fill = Color32::TRANSPARENT;
        style.visuals.window_fill = HEADER;
        style.visuals.window_stroke = egui::Stroke::new(1.0, BORDER);
        let widgets = &mut style.visuals.widgets;
        widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
        widgets.inactive.fg_stroke.color = MUTED;
        widgets.hovered.weak_bg_fill = HOVER;
        widgets.hovered.bg_fill = HOVER;
        widgets.hovered.fg_stroke.color = FG;
        widgets.active.weak_bg_fill = ACTIVE;
        widgets.active.fg_stroke.color = FG;
        style.visuals.selection.bg_fill = SELECTION;
        style.spacing.item_spacing = Vec2::new(6.0, 4.0);
        style.spacing.button_padding = Vec2::new(8.0, 4.0);
    });
}

/// The brand font: JetBrains Mono Bold.
fn brand(size: f32) -> FontId {
    FontId::new(size, egui::FontFamily::Name(fonts::BOLD_FAMILY.into()))
}

/// A flat icon button, brightening on hover.
fn icon_button(ui: &mut egui::Ui, name: &str, tip: &str) -> egui::Response {
    let image = icons::image(name, 15.0, Color32::WHITE);
    let button = egui::Button::image(image)
        .frame(true)
        .min_size(Vec2::splat(24.0))
        .image_tint_follows_text_color(true);
    ui.add(button).on_hover_text(tip)
}
