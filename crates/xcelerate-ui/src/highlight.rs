//! A small, language-agnostic syntax highlighter for the codegen preview.
//!
//! It is deliberately lexical, not a parser: it colours comments, string
//! literals, numbers, keywords, called functions and types. A union of the
//! keywords across the eleven targets is used, which is plenty for reading a
//! short generated script. The palette stays on the overlay's monochrome +
//! red look, with a single warm hue reserved for literals.

use egui::text::LayoutJob;
use egui::{Color32, FontFamily, FontId, TextFormat};

use crate::codegen::Language;

/// Default identifier / plain text colour (Darcula default).
const FG: Color32 = Color32::from_rgb(169, 183, 198);
/// Comments and other non-code text.
const COMMENT: Color32 = Color32::from_rgb(128, 128, 128);
/// Keywords (`async`, `fn`, `val`, ...) — Darcula orange.
const KEYWORD: Color32 = Color32::from_rgb(204, 120, 50);
/// String literals — Darcula green.
const STRING: Color32 = Color32::from_rgb(106, 135, 89);
/// Numeric literals — Darcula blue.
const NUMBER: Color32 = Color32::from_rgb(104, 151, 187);
/// The name of a function or method being called — Darcula yellow.
const FUNCTION: Color32 = Color32::from_rgb(255, 198, 109);
/// Capitalised identifiers, taken to be types (Darcula keeps them plain).
const TYPE: Color32 = Color32::from_rgb(169, 183, 198);
/// Braces, dots, operators.
const PUNCT: Color32 = Color32::from_rgb(169, 183, 198);
/// The line-number gutter.
const GUTTER: Color32 = Color32::from_rgb(96, 99, 102);

/// The size of the code text, in points.
pub(crate) const CODE_SIZE: f32 = 12.5;

/// The union of keywords across the targets; enough to make a script readable.
const KEYWORDS: &[&str] = &[
    "abstract",
    "and",
    "any",
    "as",
    "async",
    "await",
    "bool",
    "break",
    "case",
    "catch",
    "chan",
    "class",
    "const",
    "continue",
    "def",
    "defer",
    "do",
    "double",
    "else",
    "end",
    "enum",
    "except",
    "export",
    "extends",
    "false",
    "final",
    "finally",
    "float",
    "fn",
    "for",
    "foreach",
    "from",
    "func",
    "fun",
    "function",
    "go",
    "if",
    "implements",
    "import",
    "in",
    "instanceof",
    "int",
    "interface",
    "is",
    "let",
    "loop",
    "map",
    "match",
    "mod",
    "module",
    "mut",
    "namespace",
    "new",
    "nil",
    "none",
    "not",
    "null",
    "of",
    "or",
    "override",
    "package",
    "println",
    "print",
    "private",
    "protected",
    "pub",
    "public",
    "raise",
    "range",
    "require",
    "return",
    "self",
    "select",
    "static",
    "struct",
    "super",
    "suspend",
    "switch",
    "this",
    "throw",
    "throws",
    "trait",
    "true",
    "try",
    "type",
    "typeof",
    "undefined",
    "union",
    "unless",
    "unsafe",
    "use",
    "using",
    "val",
    "var",
    "void",
    "when",
    "where",
    "while",
    "with",
];

/// Builds a no-wrap [`LayoutJob`] for `code`, with a line-number gutter.
pub(crate) fn job(code: &str, language: Language) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.wrap.max_width = f32::INFINITY;
    let font = FontId::new(CODE_SIZE, FontFamily::Monospace);
    let lines: Vec<&str> = code.lines().collect();
    let width = lines.len().max(1).to_string().len();
    for (index, line) in lines.iter().enumerate() {
        job.append(
            &format!("{:>width$}  ", index + 1),
            0.0,
            format(&font, GUTTER),
        );
        for (text, color) in tokens(line, language) {
            job.append(&text, 0.0, format(&font, color));
        }
        job.append("\n", 0.0, format(&font, FG));
    }
    job
}

fn format(font: &FontId, color: Color32) -> TextFormat {
    TextFormat {
        font_id: font.clone(),
        color,
        ..Default::default()
    }
}

/// The comment marker for a language.
fn comment_prefix(language: Language) -> &'static str {
    match language {
        Language::Python | Language::Ruby | Language::PowerShell => "#",
        _ => "//",
    }
}

/// Splits a single line into coloured spans.
fn tokens(line: &str, language: Language) -> Vec<(String, Color32)> {
    let comment = comment_prefix(language);
    let bytes = line.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let rest = &line[i..];
        let c = bytes[i] as char;

        // Runs of spaces and tabs: keep them, plain.
        if c == ' ' || c == '\t' {
            let start = i;
            while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
                i += 1;
            }
            out.push((line[start..i].to_string(), FG));
            continue;
        }

        // Comments swallow the rest of the line.
        if rest.starts_with(comment) || rest.starts_with("/*") {
            out.push((rest.to_string(), COMMENT));
            break;
        }

        // String literals, with backslash escapes (no multi-line strings).
        if c == '"' || c == '\'' || c == '`' {
            let quote = bytes[i];
            let start = i;
            i += 1;
            while i < bytes.len() {
                if bytes[i] == b'\\' {
                    i += 2;
                    continue;
                }
                if bytes[i] == quote {
                    i += 1;
                    break;
                }
                i += 1;
            }
            let end = i.min(bytes.len());
            out.push((line[start..end].to_string(), STRING));
            i = end;
            continue;
        }

        // Numeric literals.
        if c.is_ascii_digit() {
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'.') {
                i += 1;
            }
            out.push((line[start..i].to_string(), NUMBER));
            continue;
        }

        // Identifiers and keywords.
        if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let word = &line[start..i];
            let mut look = i;
            while look < bytes.len() && bytes[look] == b' ' {
                look += 1;
            }
            let called = look < bytes.len() && bytes[look] == b'(';
            let color = if KEYWORDS.contains(&word) {
                KEYWORD
            } else if called {
                FUNCTION
            } else if word.starts_with(|ch: char| ch.is_ascii_uppercase()) {
                TYPE
            } else {
                FG
            };
            out.push((word.to_string(), color));
            continue;
        }

        // Everything else is punctuation; one byte at a time is fine.
        out.push((c.to_string(), PUNCT));
        i += 1;
    }
    out
}
