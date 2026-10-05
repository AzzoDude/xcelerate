//! Clean, LLM-ready content extraction (capability #1).
//!
//! Implemented as inherent `impl Page` methods in this module so they stay out
//! of the `#[uniffi::export]` block and binding checksums remain stable.
//!
//! The pipeline is intentionally dependency-free and tolerant:
//!
//! 1. [`extract_main_html`] performs a "readability-lite" pass: it drops noisy
//!    regions (scripts, styles, navigation, ...) and, when present, keeps the
//!    largest `<main>`/`<article>` block; otherwise it falls back to `<body>`.
//! 2. [`html_to_markdown`] renders the remaining HTML into Markdown with a
//!    small hand-rolled renderer that handles the common block and inline
//!    elements needed for LLM ingestion.

use crate::error::XcelerateResult;
use crate::page::Page;

/// Elements dropped together with their contents during extraction.
const STRIP_TAGS: &[&str] = &[
    "script", "style", "svg", "noscript", "template", "nav", "header", "footer", "aside",
];

/// Elements whose contents are never rendered into Markdown.
const SKIP_RENDER: &[&str] = &[
    "script", "style", "svg", "noscript", "template", "title", "meta", "link", "base", "head",
];

/// Block-level containers that introduce blank-line separation.
const BLOCK_CONTAINERS: &[&str] = &[
    "div",
    "section",
    "article",
    "main",
    "header",
    "footer",
    "aside",
    "figure",
    "figcaption",
    "form",
    "fieldset",
    "address",
    "center",
    "details",
    "summary",
    "dl",
    "dt",
    "dd",
    "body",
    "html",
];

/// Extracts the main content of an HTML document.
///
/// This is a lightweight, regex-free approximation of readability:
///
/// 1. HTML comments and the [`STRIP_TAGS`] elements (with their contents) are
///    removed.
/// 2. If one or more `<main>`/`<article>` blocks remain, the one with the most
///    content is returned (its inner HTML).
/// 3. Otherwise the `<body>` content is returned, or the whole input when no
///    `<body>` is present.
pub(crate) fn extract_main_html(html: &str) -> String {
    let mut cleaned = strip_comments(html);
    for tag in STRIP_TAGS {
        cleaned = strip_element(&cleaned, tag);
    }

    let mut candidates: Vec<String> = Vec::new();
    candidates.extend(find_blocks(&cleaned, "main"));
    candidates.extend(find_blocks(&cleaned, "article"));
    if let Some(best) = candidates.into_iter().max_by_key(|c| c.len())
        && !best.trim().is_empty()
    {
        return best;
    }

    if let Some(body) = find_blocks(&cleaned, "body")
        .into_iter()
        .max_by_key(|b| b.len())
    {
        return body;
    }

    cleaned
}

/// Renders HTML into Markdown.
///
/// Supports headings (`h1`..`h6`), paragraphs, line breaks, horizontal rules,
/// links, emphasis, inline code, fenced code blocks, blockquotes, ordered and
/// unordered lists, images, and simple tables. It decodes the common HTML
/// entities (including numeric references), collapses runs of whitespace in
/// normal text, trims trailing whitespace on every line, and reduces runs of
/// three or more newlines to two.
pub(crate) fn html_to_markdown(html: &str) -> String {
    let nodes = parse_html(html);
    finalize(&render_nodes(&nodes, Ctx { in_pre: false }))
}

impl Page {
    /// Returns the page's main content converted to Markdown.
    ///
    /// The raw HTML is fetched with [`Page::content`], reduced to its main
    /// region by [`extract_main_html`], and then rendered to Markdown by
    /// [`html_to_markdown`]. This is the clean, LLM-ready representation of the
    /// current document.
    pub async fn markdown(&self) -> XcelerateResult<String> {
        let html = self.content().await?;
        let main = extract_main_html(&html);
        Ok(html_to_markdown(&main))
    }
}

// ---------------------------------------------------------------------------
// Minimal HTML tree
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum Node {
    Text(String),
    Element(Element),
}

#[derive(Debug, Clone)]
struct Element {
    name: String,
    attrs: Vec<(String, String)>,
    children: Vec<Node>,
}

/// Rendering context propagated through the tree walk.
#[derive(Clone, Copy)]
struct Ctx {
    in_pre: bool,
}

/// Parses HTML into a tolerant, best-effort tree.
///
/// Unmatched close tags are ignored and unclosed elements are implicitly
/// closed at the end of input; the parser never fails.
fn parse_html(html: &str) -> Vec<Node> {
    let mut roots: Vec<Node> = Vec::new();
    let mut stack: Vec<Element> = Vec::new();
    let mut i = 0;

    while i < html.len() {
        if html.as_bytes()[i] == b'<' {
            if html[i..].starts_with("<!--") {
                if let Some(pos) = html[i + 4..].find("-->") {
                    i = i + 4 + pos + 3;
                } else {
                    break;
                }
                continue;
            }

            let Some(end) = find_tag_end(html, i) else {
                push_text(&mut stack, &mut roots, &html[i..]);
                break;
            };

            let raw = &html[i + 1..end - 1];
            if raw.starts_with('!') || raw.starts_with('?') {
                i = end;
                continue;
            }

            let (name, closing, self_closing) = parse_tag(raw);
            if name.is_empty() {
                i = end;
                continue;
            }

            if closing {
                if let Some(pos) = stack.iter().rposition(|e| e.name == name) {
                    while stack.len() > pos {
                        let elem = stack.pop().expect("stack length checked above");
                        push_node(&mut stack, &mut roots, Node::Element(elem));
                    }
                }
            } else if self_closing || is_void(&name) {
                let elem = Element {
                    attrs: parse_attrs(raw),
                    name,
                    children: Vec::new(),
                };
                push_node(&mut stack, &mut roots, Node::Element(elem));
            } else {
                stack.push(Element {
                    name,
                    attrs: parse_attrs(raw),
                    children: Vec::new(),
                });
            }
            i = end;
        } else {
            let next = html[i..].find('<').map(|p| i + p).unwrap_or(html.len());
            push_text(&mut stack, &mut roots, &html[i..next]);
            i = next;
        }
    }

    while let Some(elem) = stack.pop() {
        push_node(&mut stack, &mut roots, Node::Element(elem));
    }

    roots
}

fn push_node(stack: &mut [Element], roots: &mut Vec<Node>, node: Node) {
    match stack.last_mut() {
        Some(parent) => parent.children.push(node),
        None => roots.push(node),
    }
}

fn push_text(stack: &mut [Element], roots: &mut Vec<Node>, text: &str) {
    if text.is_empty() {
        return;
    }
    push_node(stack, roots, Node::Text(text.to_string()));
}

/// Returns whether `name` is an HTML void element.
fn is_void(name: &str) -> bool {
    matches!(
        name,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

fn render_nodes(nodes: &[Node], ctx: Ctx) -> String {
    let mut out = String::new();
    for node in nodes {
        match node {
            Node::Text(text) => out.push_str(&normalize_text(text, ctx.in_pre)),
            Node::Element(elem) => out.push_str(&render_element(elem, ctx)),
        }
    }
    out
}

fn render_element(elem: &Element, ctx: Ctx) -> String {
    let name = elem.name.as_str();

    if SKIP_RENDER.contains(&name) {
        return String::new();
    }

    match name {
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            let level = name[1..].parse::<usize>().unwrap_or(1).clamp(1, 6);
            let inner = render_nodes(&elem.children, ctx);
            format!("\n\n{} {}\n\n", "#".repeat(level), inner.trim())
        }
        "p" => {
            let inner = render_nodes(&elem.children, ctx);
            format!("\n\n{}\n\n", inner.trim())
        }
        "br" => "\n".to_string(),
        "hr" => "\n\n---\n\n".to_string(),
        "a" => {
            let inner = render_nodes(&elem.children, ctx);
            match attr(elem, "href") {
                Some(href) if !href.is_empty() => format!("[{inner}]({href})"),
                _ => inner,
            }
        }
        "strong" | "b" => {
            let inner = render_nodes(&elem.children, ctx);
            format!("**{}**", inner.trim())
        }
        "em" | "i" => {
            let inner = render_nodes(&elem.children, ctx);
            format!("*{}*", inner.trim())
        }
        "code" => {
            if ctx.in_pre {
                render_nodes(&elem.children, ctx)
            } else {
                let inner = render_nodes(&elem.children, Ctx { in_pre: true });
                format!("`{}`", inner.trim())
            }
        }
        "pre" => {
            let inner = render_nodes(&elem.children, Ctx { in_pre: true });
            let body = inner.trim_matches('\n');
            format!("\n\n```\n{body}\n```\n\n")
        }
        "blockquote" => {
            let inner = render_nodes(&elem.children, ctx);
            let mut out = String::from("\n\n");
            for line in inner.trim().lines() {
                if line.trim().is_empty() {
                    out.push_str(">\n");
                } else {
                    out.push_str("> ");
                    out.push_str(line);
                    out.push('\n');
                }
            }
            out.push('\n');
            out
        }
        "ul" | "ol" => render_list(elem, ctx),
        "table" => render_table(elem, ctx),
        "img" => {
            let alt = attr(elem, "alt").unwrap_or_default();
            let src = attr(elem, "src").unwrap_or_default();
            if src.is_empty() && alt.is_empty() {
                String::new()
            } else {
                format!("![{alt}]({src})")
            }
        }
        _ if BLOCK_CONTAINERS.contains(&name) => {
            let inner = render_nodes(&elem.children, ctx);
            format!("\n\n{}\n\n", inner.trim())
        }
        // Transparent elements (li/tr/td/th/unknown tags): render children.
        _ => render_nodes(&elem.children, ctx),
    }
}

fn render_list(elem: &Element, ctx: Ctx) -> String {
    let ordered = elem.name == "ol";
    let mut out = String::from("\n\n");
    let mut index = 1usize;

    for child in &elem.children {
        let Node::Element(li) = child else { continue };
        if li.name != "li" {
            continue;
        }

        let marker = if ordered {
            format!("{index}. ")
        } else {
            "- ".to_string()
        };
        index += 1;
        let pad = " ".repeat(marker.chars().count());

        let inner = render_nodes(&li.children, ctx);
        let inner = inner.trim();
        if inner.is_empty() {
            out.push_str(&marker);
            out.push('\n');
            continue;
        }

        for (line_no, line) in inner.lines().enumerate() {
            if line_no == 0 {
                out.push_str(&marker);
            } else {
                out.push_str(&pad);
            }
            out.push_str(line);
            out.push('\n');
        }
    }

    out.push('\n');
    out
}

fn render_table(table: &Element, ctx: Ctx) -> String {
    let mut rows: Vec<Vec<String>> = Vec::new();
    collect_rows(table, &mut rows, ctx);
    if rows.is_empty() {
        return String::new();
    }

    let cols = rows.iter().map(Vec::len).max().unwrap_or(0);
    if cols == 0 {
        return String::new();
    }

    let mut out = String::from("\n\n");
    for (row_index, row) in rows.iter().enumerate() {
        out.push('|');
        for col in 0..cols {
            let cell = row.get(col).map(String::as_str).unwrap_or("");
            out.push(' ');
            out.push_str(&cell.replace('|', "\\|"));
            out.push_str(" |");
        }
        out.push('\n');

        if row_index == 0 {
            out.push('|');
            for _ in 0..cols {
                out.push_str(" --- |");
            }
            out.push('\n');
        }
    }
    out.push('\n');
    out
}

fn collect_rows(elem: &Element, rows: &mut Vec<Vec<String>>, ctx: Ctx) {
    for child in &elem.children {
        let Node::Element(node) = child else { continue };
        match node.name.as_str() {
            "tr" => {
                let mut cells = Vec::new();
                for cell in &node.children {
                    let Node::Element(cell) = cell else { continue };
                    if cell.name == "td" || cell.name == "th" {
                        let text = render_nodes(&cell.children, ctx);
                        let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
                        cells.push(text);
                    }
                }
                if !cells.is_empty() {
                    rows.push(cells);
                }
            }
            "thead" | "tbody" | "tfoot" => collect_rows(node, rows, ctx),
            _ => {}
        }
    }
}

fn attr<'a>(elem: &'a Element, name: &str) -> Option<&'a str> {
    elem.attrs
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.as_str())
}

// ---------------------------------------------------------------------------
// Text, entities, and final cleanup
// ---------------------------------------------------------------------------

/// Decodes entities and, outside `<pre>`, collapses whitespace runs to a
/// single space.
fn normalize_text(text: &str, in_pre: bool) -> String {
    let decoded = decode_entities(text);
    if in_pre {
        return decoded;
    }

    let mut out = String::with_capacity(decoded.len());
    let mut in_whitespace = false;
    for ch in decoded.chars() {
        if ch.is_whitespace() {
            if !in_whitespace {
                out.push(' ');
                in_whitespace = true;
            }
        } else {
            out.push(ch);
            in_whitespace = false;
        }
    }
    out
}

/// Decodes the common named entities plus numeric (`&#NN;`/`&#xHH;`) references.
///
/// Unknown or malformed entities are left untouched.
fn decode_entities(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < input.len() {
        if bytes[i] == b'&'
            && let Some(semi) = input[i..].find(';')
        {
            let entity = &input[i + 1..i + semi];
            if is_entity_body(entity)
                && let Some(decoded) = decode_entity(entity)
            {
                out.push_str(&decoded);
                i = i + semi + 1;
                continue;
            }
        }
        let ch = input[i..].chars().next().expect("index within bounds");
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

fn is_entity_body(entity: &str) -> bool {
    !entity.is_empty()
        && entity.len() <= 12
        && entity
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '#')
}

fn decode_entity(entity: &str) -> Option<String> {
    match entity {
        "amp" => Some("&".to_string()),
        "lt" => Some("<".to_string()),
        "gt" => Some(">".to_string()),
        "quot" => Some("\"".to_string()),
        "apos" => Some("'".to_string()),
        "nbsp" => Some(" ".to_string()),
        _ => {
            let digits = entity.strip_prefix('#')?;
            let code = if let Some(hex) = digits.strip_prefix(['x', 'X']) {
                u32::from_str_radix(hex, 16).ok()?
            } else {
                digits.parse::<u32>().ok()?
            };
            char::from_u32(code).map(|c| c.to_string())
        }
    }
}

/// Trims trailing whitespace per line and collapses runs of three or more
/// newlines down to two.
fn finalize(input: &str) -> String {
    let mut trimmed = String::with_capacity(input.len());
    for line in input.lines() {
        trimmed.push_str(line.trim_end());
        trimmed.push('\n');
    }

    let mut out = String::with_capacity(trimmed.len());
    let mut newlines = 0usize;
    for ch in trimmed.chars() {
        if ch == '\n' {
            newlines += 1;
        } else {
            for _ in 0..newlines.min(2) {
                out.push('\n');
            }
            newlines = 0;
            out.push(ch);
        }
    }
    for _ in 0..newlines.min(2) {
        out.push('\n');
    }

    out.trim().to_string()
}

// ---------------------------------------------------------------------------
// Low-level HTML scanning helpers
// ---------------------------------------------------------------------------

/// Finds the index just past the `>` that closes the tag starting at `start`.
///
/// Quoted attribute values are respected so a `>` inside an attribute does not
/// terminate the tag early. Returns `None` when the tag is never closed.
fn find_tag_end(html: &str, start: usize) -> Option<usize> {
    let bytes = html.as_bytes();
    let mut i = start + 1;
    let mut quote: Option<u8> = None;
    while i < bytes.len() {
        let b = bytes[i];
        match quote {
            Some(q) => {
                if b == q {
                    quote = None;
                }
            }
            None => {
                if b == b'"' || b == b'\'' {
                    quote = Some(b);
                } else if b == b'>' {
                    return Some(i + 1);
                }
            }
        }
        i += 1;
    }
    None
}

/// Splits a raw tag body (between `<` and `>`) into its lowercase name, whether
/// it is a closing tag, and whether it is self-closing.
fn parse_tag(raw: &str) -> (String, bool, bool) {
    let mut body = raw.trim();
    let mut closing = false;
    if let Some(rest) = body.strip_prefix('/') {
        closing = true;
        body = rest.trim_start();
    }

    let self_closing = body.ends_with('/');
    if self_closing {
        body = body[..body.len() - 1].trim_end();
    }

    let end = body
        .find(|c: char| c.is_whitespace() || c == '/')
        .unwrap_or(body.len());
    (body[..end].to_ascii_lowercase(), closing, self_closing)
}

/// Parses a raw tag body into `(lowercase_name, value)` attribute pairs.
fn parse_attrs(raw: &str) -> Vec<(String, String)> {
    let mut attrs = Vec::new();
    let bytes = raw.as_bytes();
    let mut i = 0;

    // Skip the tag name (and a leading `/` for closing tags).
    while i < bytes.len() && !bytes[i].is_ascii_whitespace() {
        i += 1;
    }

    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() || bytes[i] == b'/' {
            break;
        }

        let name_start = i;
        while i < bytes.len()
            && bytes[i] != b'='
            && !bytes[i].is_ascii_whitespace()
            && bytes[i] != b'/'
        {
            i += 1;
        }
        let name = raw[name_start..i].to_ascii_lowercase();

        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }

        if i < bytes.len() && bytes[i] == b'=' {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i < bytes.len() && (bytes[i] == b'"' || bytes[i] == b'\'') {
                let quote = bytes[i];
                i += 1;
                let value_start = i;
                while i < bytes.len() && bytes[i] != quote {
                    i += 1;
                }
                let value = &raw[value_start..i];
                if i < bytes.len() {
                    i += 1;
                }
                if !name.is_empty() {
                    attrs.push((name, decode_entities(value)));
                }
            } else {
                let value_start = i;
                while i < bytes.len() && !bytes[i].is_ascii_whitespace() {
                    i += 1;
                }
                if !name.is_empty() {
                    attrs.push((name, decode_entities(&raw[value_start..i])));
                }
            }
        } else if !name.is_empty() {
            attrs.push((name, String::new()));
        }
    }

    attrs
}

/// Removes all HTML comments from `html`.
fn strip_comments(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut i = 0;
    while i < html.len() {
        if html[i..].starts_with("<!--") {
            match html[i + 4..].find("-->") {
                Some(pos) => {
                    i = i + 4 + pos + 3;
                    continue;
                }
                None => break,
            }
        }
        let ch = html[i..].chars().next().expect("index within bounds");
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// Removes every `<tag>...</tag>` element (including nested occurrences) along
/// with its contents. A lone closing/void tag is dropped as well.
fn strip_element(html: &str, tag: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut i = 0;
    while i < html.len() {
        if html.as_bytes()[i] == b'<' {
            let Some(end) = find_tag_end(html, i) else {
                out.push_str(&html[i..]);
                break;
            };
            let raw = &html[i + 1..end - 1];
            let (name, closing, self_closing) = parse_tag(raw);
            if name == tag {
                i = if closing || self_closing {
                    end
                } else {
                    skip_element(html, end, tag)
                };
                continue;
            }
            out.push_str(&html[i..end]);
            i = end;
        } else {
            let ch = html[i..].chars().next().expect("index within bounds");
            out.push(ch);
            i += ch.len_utf8();
        }
    }
    out
}

/// Skips past the element opened before `from`, returning the index just after
/// its matching close tag (or the end of input when unclosed).
fn skip_element(html: &str, from: usize, tag: &str) -> usize {
    let close_start = find_matching_close(html, from, tag);
    if close_start >= html.len() {
        return html.len();
    }
    find_tag_end(html, close_start).unwrap_or(html.len())
}

/// Returns the index of the `<` that begins the close tag matching the element
/// opened before `from`, honouring nested occurrences of `tag`. Returns the
/// input length when no matching close tag exists.
fn find_matching_close(html: &str, from: usize, tag: &str) -> usize {
    let bytes = html.as_bytes();
    let mut i = from;
    let mut depth = 1usize;
    while i < html.len() {
        if bytes[i] == b'<' {
            let Some(end) = find_tag_end(html, i) else {
                return html.len();
            };
            let raw = &html[i + 1..end - 1];
            let (name, closing, self_closing) = parse_tag(raw);
            if name == tag {
                if closing {
                    depth -= 1;
                    if depth == 0 {
                        return i;
                    }
                } else if !self_closing {
                    depth += 1;
                }
            }
            i = end;
        } else {
            let ch = html[i..].chars().next().expect("index within bounds");
            i += ch.len_utf8();
        }
    }
    html.len()
}

/// Collects the inner HTML of every top-level `<tag>...</tag>` block.
fn find_blocks(html: &str, tag: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let bytes = html.as_bytes();
    let mut i = 0;
    while i < html.len() {
        if bytes[i] == b'<' {
            let Some(end) = find_tag_end(html, i) else {
                break;
            };
            let raw = &html[i + 1..end - 1];
            let (name, closing, self_closing) = parse_tag(raw);
            if name == tag && !closing && !self_closing {
                let close_start = find_matching_close(html, end, tag);
                blocks.push(html[end..close_start].to_string());
                i = if close_start >= html.len() {
                    html.len()
                } else {
                    find_tag_end(html, close_start).unwrap_or(html.len())
                };
                continue;
            }
            i = end;
        } else {
            let ch = html[i..].chars().next().expect("index within bounds");
            i += ch.len_utf8();
        }
    }
    blocks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heading_and_link_are_rendered() {
        let md =
            html_to_markdown("<h1>Hi</h1><p>Hello <a href=\"https://x/\" title=\"x\">link</a></p>");
        assert!(md.contains("# Hi"), "missing heading in: {md:?}");
        assert!(md.contains("[link](https://x/)"), "missing link in: {md:?}");
    }

    #[test]
    fn inline_formatting_is_rendered() {
        let md =
            html_to_markdown("<p><strong>bold</strong> and <em>it</em> and <code>x = 1</code></p>");
        assert!(md.contains("**bold**"), "{md:?}");
        assert!(md.contains("*it*"), "{md:?}");
        assert!(md.contains("`x = 1`"), "{md:?}");
    }

    #[test]
    fn extract_main_html_prefers_article() {
        let html = "<html><body><nav>menu</nav><div>junk</div>\
                    <article><h1>Title</h1><p>Body</p></article></body></html>";
        let main = extract_main_html(html);
        assert!(main.contains("Title"), "{main:?}");
        assert!(main.contains("Body"), "{main:?}");
        assert!(!main.contains("menu"), "nav leaked: {main:?}");
        assert!(!main.contains("junk"), "sibling leaked: {main:?}");
    }

    #[test]
    fn extract_main_html_picks_largest_block() {
        let html = "<body>\
                    <main><p>small</p></main>\
                    <article><p>a much longer piece of content here</p></article>\
                    </body>";
        let main = extract_main_html(html);
        assert!(main.contains("much longer"), "{main:?}");
        assert!(!main.contains("small"), "{main:?}");
    }

    #[test]
    fn extract_main_html_drops_scripts_and_comments() {
        let html = "<body><!-- secret --><script>alert(1)</script>\
                    <p>keep me</p></body>";
        let main = extract_main_html(html);
        assert!(main.contains("keep me"), "{main:?}");
        assert!(!main.contains("alert"), "script leaked: {main:?}");
        assert!(!main.contains("secret"), "comment leaked: {main:?}");
    }

    #[test]
    fn extract_main_html_falls_back_to_body() {
        let html = "<html><head><title>t</title></head><body><p>only</p></body></html>";
        let main = extract_main_html(html);
        assert!(main.contains("only"), "{main:?}");
    }

    #[test]
    fn entities_are_decoded() {
        assert_eq!(decode_entities("&amp;&lt;&#39;&gt;&quot;"), "&<'>\"");
        assert_eq!(decode_entities("&#65;&#x42;"), "AB");

        let md = html_to_markdown("<p>&amp; &lt; &gt; &quot; &#39; &nbsp; &#65;</p>");
        assert!(md.contains("& < > \" ' A"), "{md:?}");
    }

    #[test]
    fn blank_line_runs_are_collapsed() {
        let md = html_to_markdown("<div><br><br><br><br><p>a</p></div><div></div><div></div>");
        assert!(!md.contains("\n\n\n"), "unexpected blank run: {md:?}");
    }

    #[test]
    fn trailing_whitespace_is_trimmed() {
        let md = html_to_markdown("<p>hello   </p><p>world\t</p>");
        for line in md.lines() {
            assert_eq!(line, line.trim_end(), "trailing ws in {line:?}");
        }
    }

    #[test]
    fn lists_are_rendered() {
        let md = html_to_markdown("<ul><li>One</li><li>Two</li></ul>");
        assert!(md.contains("- One"), "{md:?}");
        assert!(md.contains("- Two"), "{md:?}");

        let md = html_to_markdown("<ol><li>First</li><li>Second</li></ol>");
        assert!(md.contains("1. First"), "{md:?}");
        assert!(md.contains("2. Second"), "{md:?}");
    }

    #[test]
    fn blockquotes_and_code_blocks_are_rendered() {
        let md = html_to_markdown("<blockquote><p>Quote</p></blockquote>");
        assert!(md.contains("> Quote"), "{md:?}");

        let md = html_to_markdown("<pre><code>let x = 1;</code></pre>");
        assert!(md.contains("```"), "{md:?}");
        assert!(md.contains("let x = 1;"), "{md:?}");
    }

    #[test]
    fn images_and_tables_are_rendered() {
        let md = html_to_markdown("<img src=\"a.png\" alt=\"Alt\">");
        assert!(md.contains("![Alt](a.png)"), "{md:?}");

        let md = html_to_markdown(
            "<table><tr><th>A</th><th>B</th></tr><tr><td>1</td><td>2</td></tr></table>",
        );
        assert!(md.contains("| A | B |"), "{md:?}");
        assert!(md.contains("| 1 | 2 |"), "{md:?}");
        assert!(md.contains("| --- | --- |"), "{md:?}");
    }

    #[test]
    fn parse_tag_handles_closing_and_self_closing() {
        assert_eq!(parse_tag("div class=\"x\"").0, "div");
        assert!(parse_tag("/div").1);
        assert!(parse_tag("br/").2);
        assert_eq!(parse_tag("h2").0, "h2");
    }
}
