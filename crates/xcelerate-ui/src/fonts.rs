//! JetBrains Mono, embedded and installed as egui's monospace family.
//!
//! The code preview and any other monospaced text render in JetBrains Mono;
//! proportional UI text keeps egui's default face. JetBrains Mono is licensed
//! under the SIL Open Font License 1.1 (see `assets/fonts/OFL.txt`).

use std::sync::Arc;

use egui::{FontData, FontDefinitions, FontFamily};

const REGULAR: &[u8] = include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf");
const BOLD: &[u8] = include_bytes!("../assets/fonts/JetBrainsMono-Bold.ttf");

/// The named family for bold monospaced text (the brand mark).
pub(crate) const BOLD_FAMILY: &str = "jetbrains-mono-bold";

/// Installs JetBrains Mono on the context. Replaces any previously set fonts.
pub(crate) fn install(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "jetbrains-mono".to_owned(),
        Arc::new(FontData::from_static(REGULAR)),
    );
    fonts.font_data.insert(
        BOLD_FAMILY.to_owned(),
        Arc::new(FontData::from_static(BOLD)),
    );
    // Monospaced text (the code preview, timestamps) uses JetBrains Mono;
    // proportional UI text keeps egui's default face.
    if let Some(list) = fonts.families.get_mut(&FontFamily::Monospace) {
        list.insert(0, "jetbrains-mono".to_owned());
    }
    fonts.families.insert(
        FontFamily::Name(BOLD_FAMILY.into()),
        vec![BOLD_FAMILY.to_owned(), "jetbrains-mono".to_owned()],
    );
    ctx.set_fonts(fonts);
}
