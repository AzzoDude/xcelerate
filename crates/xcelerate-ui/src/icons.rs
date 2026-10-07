//! Embedded Phosphor icons (regular weight, 256x256, `fill="currentColor"`).
//!
//! egui has no notion of `currentColor`, so each icon is tinted at draw time via
//! [`egui::Image::tint`].

use egui::{Color32, Image};

/// `(name, svg)` for every bundled Phosphor icon.
pub const ALL: &[(&str, &str)] = &[
    ("stop", include_str!("../assets/icons/stop.svg")),
    ("pause", include_str!("../assets/icons/pause.svg")),
    ("play", include_str!("../assets/icons/play.svg")),
    ("camera", include_str!("../assets/icons/camera.svg")),
    ("caret-down", include_str!("../assets/icons/caret-down.svg")),
    ("caret-up", include_str!("../assets/icons/caret-up.svg")),
    ("x", include_str!("../assets/icons/x.svg")),
    ("robot", include_str!("../assets/icons/robot.svg")),
    ("copy", include_str!("../assets/icons/copy.svg")),
    ("trash", include_str!("../assets/icons/trash.svg")),
];

fn svg(name: &str) -> &'static str {
    ALL.iter().find(|(n, _)| *n == name).map_or("", |(_, s)| *s)
}

/// An [`egui::Image`] for a Phosphor icon, drawn at `size` points and tinted.
///
/// The embedded SVGs are `fill="currentColor"`, which resvg cannot resolve, so
/// it would rasterise them black - and a black texture multiplied by any tint
/// stays black. Painting them white lets the tint recolour them properly.
pub fn image(name: &str, size: f32, tint: Color32) -> Image<'static> {
    let svg = svg(name).replace("currentColor", "#ffffff");
    Image::from_bytes(format!("bytes://xcelerate/{name}.svg"), svg.into_bytes())
        .fit_to_exact_size(egui::vec2(size, size))
        .tint(tint)
}
