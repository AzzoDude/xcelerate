//! Native, agent-friendly DOM snapshot.
//!
//! This is xcelerate's answer to the per-step "serialize the page for an LLM"
//! pipeline that agent libraries such as browser-use implement in Python. Doing
//! it here means:
//!
//! * one persistent CDP session instead of a fresh WebSocket per step;
//! * all hot loops (snapshot parsing, clickability, indexing) run in Rust, so
//!   the O(n^2) traps that bite per-node Python code cannot happen;
//! * every binding (Dart, C#, Go, Node, Python) and the CLI/MCP surfaces get
//!   the same output for free.
//!
//! The snapshot is built from two CDP calls issued in parallel -
//! [`Accessibility.getFullAXTree`] for semantics (roles, names, values) and
//! [`DOMSnapshot.captureSnapshot`] for layout (bounds, clickability, tags) -
//! then rendered to a compact, indented text where every interactive element is
//! tagged with a stable `[index]`. [`Page::click_index`] maps those indices back
//! to nodes without re-resolving CSS selectors.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use serde_json::{Value, json};

use crate::element::Element;
use crate::error::{XcelerateError, XcelerateResult};
use crate::page::Page;

/// Computed styles worth requesting from `DOMSnapshot.captureSnapshot`.
///
/// Deliberately small: requesting every computed style crashes heavy pages in
/// Chrome, and none of the omitted ones are needed for interactivity/visibility
/// detection.
pub(crate) const REQUIRED_COMPUTED_STYLES: &[&str] = &[
    "display",
    "visibility",
    "opacity",
    "overflow",
    "overflow-x",
    "overflow-y",
    "cursor",
    "pointer-events",
    "position",
    "background-color",
];

/// Max characters kept from a single accessible name/value.
const MAX_TEXT_LEN: usize = 160;

/// Longest indentation (in levels) rendered, so deep DOMs stay readable.
const MAX_INDENT: usize = 12;

/// Accessibility roles that are interactive and therefore receive an index.
const INTERACTIVE_AX_ROLES: &[&str] = &[
    "button",
    "link",
    "textbox",
    "searchbox",
    "checkbox",
    "radio",
    "combobox",
    "switch",
    "slider",
    "spinbutton",
    "option",
    "tab",
    "menuitem",
    "menuitemcheckbox",
    "menuitemradio",
    "listbox",
    "treeitem",
    "gridcell",
    "cell",
];

/// Roles that carry useful structure even when they have no accessible name.
const STRUCTURAL_AX_ROLES: &[&str] = &[
    "heading",
    "img",
    "table",
    "list",
    "listitem",
    "form",
    "navigation",
    "main",
    "dialog",
    "alert",
    "banner",
    "contentinfo",
    "region",
    "article",
    "paragraph",
];

/// Low-signal roles that merely restate an ancestor's content.
const SKIP_AX_ROLES: &[&str] = &[
    "none",
    "generic",
    "GenericContainer",
    "Ignored",
    "InlineTextBox",
    "LineBreak",
];

/// Container roles that are printed without an index but not treated as noise.
const CONTAINER_ROLES: &[&str] = &["RootWebArea", "WebArea", "Iframe"];

/// Appended when an indexed element extends past the viewport bottom, telling
/// the caller there is more to reveal by scrolling.
const SCROLL_HINT: &str = "... (more content below the viewport — scroll to reveal)";

/// A rectangle in CSS pixels, viewport-relative where applicable.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Layout/attribute facts about a backend node, distilled from a DOM snapshot.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct LayoutInfo {
    pub bounds: Option<Rect>,
    pub is_clickable: bool,
    pub tag: Option<String>,
    /// Stacking/paint order from `DOMSnapshot` (present when `includePaintOrder`
    /// is requested). Higher values are painted later, i.e. on top of lower ones.
    pub paint_order: Option<i64>,
    /// Index of the `DOMSnapshot` document this node belongs to. Bounds and paint
    /// order are per-frame, so only boxes from the same document may be compared
    /// (see [`is_occluded`]).
    pub document_index: usize,
}

/// Fraction of `inner`'s area that lies within `outer` (0.0..=1.0).
///
/// Degenerate rectangles return `0.0` so they can never be treated as covered.
fn containment_ratio(inner: Rect, outer: Rect) -> f64 {
    if inner.width <= 0.0 || inner.height <= 0.0 {
        return 0.0;
    }
    let overlap_x = (inner.x + inner.width).min(outer.x + outer.width) - inner.x.max(outer.x);
    let overlap_y = (inner.y + inner.height).min(outer.y + outer.height) - inner.y.max(outer.y);
    if overlap_x <= 0.0 || overlap_y <= 0.0 {
        return 0.0;
    }
    (overlap_x * overlap_y) / (inner.width * inner.height)
}

/// Best-effort occlusion test: is `rect` (a box plus its paint order) almost
/// entirely covered by a strictly larger box that is painted on top of it?
///
/// Deliberately conservative so genuinely clickable elements are never dropped:
/// it requires complete containment (>= 99% of the area), a strictly larger
/// cover, and a strictly higher cover paint order. Missing paint orders or
/// degenerate boxes are treated as *not* occluded.
pub(crate) fn is_occluded(
    rect: (Rect, Option<i64>),
    rects_with_order: &[(Rect, Option<i64>)],
) -> bool {
    let (target, target_order) = rect;
    let Some(target_order) = target_order else {
        return false;
    };
    let target_area = target.width * target.height;
    if target_area <= 0.0 {
        return false;
    }

    rects_with_order.iter().any(|&(cover, cover_order)| {
        let Some(cover_order) = cover_order else {
            return false;
        };
        cover_order > target_order
            && cover.width * cover.height > target_area
            && containment_ratio(target, cover) >= 0.99
    })
}

/// An interactive element surfaced by the snapshot, addressable by `index`.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SnapshotElement {
    pub index: u32,
    pub backend_node_id: i64,
    pub role: String,
    pub name: String,
    pub value: Option<String>,
    pub states: Vec<String>,
    pub tag: Option<String>,
    pub bounds: Option<Rect>,
}

impl SnapshotElement {
    fn to_json(&self) -> Value {
        let mut object = serde_json::Map::new();
        object.insert("index".into(), json!(self.index));
        object.insert("backendNodeId".into(), json!(self.backend_node_id));
        object.insert("role".into(), json!(self.role));
        if !self.name.is_empty() {
            object.insert("name".into(), json!(self.name));
        }
        if let Some(value) = &self.value {
            object.insert("value".into(), json!(value));
        }
        if !self.states.is_empty() {
            object.insert("states".into(), json!(self.states));
        }
        if let Some(tag) = &self.tag {
            object.insert("tag".into(), json!(tag));
        }
        if let Some(bounds) = self.bounds {
            object.insert(
                "bounds".into(),
                json!({ "x": bounds.x, "y": bounds.y, "width": bounds.width, "height": bounds.height }),
            );
        }
        Value::Object(object)
    }
}

/// Line diff behind [`Page::agent_snapshot_diff`]: lines present only in
/// `previous` are prefixed `-`, lines new in `current` are prefixed `+`, in the
/// order they appear. Reports "no change" when the snapshots match, and returns
/// the whole `current` when the two share no lines at all (nothing useful to
/// diff, e.g. after a navigation).
fn diff_snapshots(previous: &str, current: &str) -> String {
    let previous_lines: HashSet<&str> = previous.lines().collect();
    let current_lines: HashSet<&str> = current.lines().collect();

    if previous_lines == current_lines {
        return "<no change since the previous snapshot>".to_string();
    }

    let removed: Vec<&str> = previous
        .lines()
        .filter(|line| !current_lines.contains(line))
        .collect();
    let added: Vec<&str> = current
        .lines()
        .filter(|line| !previous_lines.contains(line))
        .collect();

    // No line survives between the two: diffing would just restate both.
    if added.len() == current_lines.len() && removed.len() == previous_lines.len() {
        return current.to_string();
    }

    let mut out = String::new();
    for line in removed {
        out.push('-');
        out.push_str(line);
        out.push('\n');
    }
    for line in added {
        out.push('+');
        out.push_str(line);
        out.push('\n');
    }
    out.pop();
    out
}

/// Removes invisible noise from an accessible name, then collapses whitespace
/// and truncates to `max` characters (adding an ellipsis).
///
/// Icon fonts expose private-use codepoints (U+E000–U+F8FF) and some pages embed
/// zero-width characters; both are invisible to a human but add bytes and
/// confuse a model. Rust's `split_whitespace` already treats NBSP and the other
/// Unicode space separators as whitespace, so no explicit normalisation of those
/// is needed.
fn clean(text: &str, max: usize) -> String {
    let filtered: String = text
        .chars()
        .filter(|c| {
            !('\u{E000}'..='\u{F8FF}').contains(c)
                && !matches!(*c, '\u{200B}'..='\u{200D}' | '\u{FEFF}')
        })
        .collect();
    let collapsed = filtered.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.chars().count() <= max {
        collapsed
    } else {
        let mut truncated: String = collapsed.chars().take(max.saturating_sub(1)).collect();
        truncated.push('…');
        truncated
    }
}

fn as_string(value: Option<&Value>) -> Option<String> {
    value.and_then(Value::as_str).map(str::to_string)
}

/// Reads `field.value` from an AX node entry, which the protocol wraps in a
/// typed box (`{"type": ..., "value": ...}`).
fn ax_field(node: &Value, field: &str) -> Option<String> {
    as_string(node.get(field).and_then(|v| v.get("value")))
}

/// Ax properties surfaced as bare state flags, in the stable order they render.
///
/// Only states that change what an agent should *do* are listed: a checked box
/// and a disabled button must be distinguishable from their unchecked/enabled
/// twins. Purely descriptive properties (level, autocomplete, ...) are dropped
/// to keep lines short.
const STATE_AX_PROPERTIES: &[&str] = &[
    "checked", "selected", "pressed", "expanded", "disabled", "required", "readonly", "invalid",
];

/// Reads the boolean-ish AX `properties` array into a compact list of state
/// flags (`checked`, `disabled`, ...).
///
/// The protocol boxes each value as `{"type", "value"}`, and the same state can
/// arrive as a boolean, a `0`/`1` number, or a `"true"`/`"mixed"` string, so all
/// three encodings are normalised. A tristate that is neither on nor off renders
/// as `name=mixed` (for example `checked=mixed`).
fn ax_states(node: &Value) -> Vec<String> {
    let Some(properties) = node.get("properties").and_then(Value::as_array) else {
        return Vec::new();
    };

    let mut states = Vec::new();
    for name in STATE_AX_PROPERTIES {
        let Some(property) = properties
            .iter()
            .find(|p| p.get("name").and_then(Value::as_str) == Some(*name))
        else {
            continue;
        };
        let flag = match property.get("value").and_then(|v| v.get("value")) {
            Some(Value::Bool(true)) => Some((*name).to_string()),
            Some(Value::Number(n)) if n.as_f64() == Some(1.0) => Some((*name).to_string()),
            Some(Value::String(text)) if text == "true" => Some((*name).to_string()),
            Some(Value::String(text)) if text == "mixed" => Some(format!("{name}=mixed")),
            _ => None,
        };
        if let Some(flag) = flag {
            states.push(flag);
        }
    }
    states
}

/// Looks up `name` in a DOM snapshot node's flat `[nameIdx, valueIdx, ...]`
/// attribute array and returns the value from the string table.
fn attribute_value<'a>(strings: &[&'a str], attributes: &[Value], name: &str) -> Option<&'a str> {
    attributes.as_chunks::<2>().0.iter().find_map(|pair| {
        let key = pair[0]
            .as_u64()
            .and_then(|i| strings.get(i as usize).copied());
        if key == Some(name) {
            pair[1]
                .as_u64()
                .and_then(|i| strings.get(i as usize).copied())
        } else {
            None
        }
    })
}

/// Builds a `backendNodeId -> LayoutInfo` map from a `DOMSnapshot.captureSnapshot`
/// response.
///
/// The rare boolean data (`isClickable`) is converted to a `HashSet` and the
/// layout index is precomputed once, so lookups are O(1) per node. This is the
/// exact class of bug that made browser-use's Python version O(n^2).
pub(crate) fn build_layout_lookup(
    snapshot: &Value,
    device_pixel_ratio: f64,
) -> HashMap<i64, LayoutInfo> {
    let dpr = if device_pixel_ratio > 0.0 {
        device_pixel_ratio
    } else {
        1.0
    };

    let mut lookup = HashMap::new();
    let strings: Vec<&str> = snapshot
        .get("strings")
        .and_then(Value::as_array)
        .map(|values| values.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();

    let Some(documents) = snapshot.get("documents").and_then(Value::as_array) else {
        return lookup;
    };

    for (document_index, document) in documents.iter().enumerate() {
        let Some(nodes) = document.get("nodes") else {
            continue;
        };
        let Some(backend_ids) = nodes.get("backendNodeId").and_then(Value::as_array) else {
            continue;
        };

        // Snapshot node index -> tag name.
        let node_names: Vec<Option<&str>> = nodes
            .get("nodeName")
            .and_then(Value::as_array)
            .map(|names| {
                names
                    .iter()
                    .map(|name| name.as_u64().and_then(|i| strings.get(i as usize).copied()))
                    .collect()
            })
            .unwrap_or_default();

        // Rare boolean data: membership, not a list scan.
        let clickable_set: HashSet<u64> = nodes
            .get("isClickable")
            .and_then(|rare| rare.get("index"))
            .and_then(Value::as_array)
            .map(|indices| indices.iter().filter_map(Value::as_u64).collect())
            .unwrap_or_default();

        // Flat `[nameIdx, valueIdx, ...]` pairs, used only to enrich a tag with
        // its input type (`input[type=file]` vs `input[type=text]`), which the AX
        // role alone cannot distinguish.
        let attributes = nodes.get("attributes").and_then(Value::as_array);

        let layout = document.get("layout");

        // Precompute the first layout index for each snapshot node index.
        let mut layout_index_map: HashMap<u64, usize> = HashMap::new();
        if let Some(node_indices) = layout
            .and_then(|layout| layout.get("nodeIndex"))
            .and_then(Value::as_array)
        {
            for (layout_idx, node_index) in node_indices.iter().enumerate() {
                if let Some(node_index) = node_index.as_u64() {
                    layout_index_map.entry(node_index).or_insert(layout_idx);
                }
            }
        }

        let bounds = layout
            .and_then(|layout| layout.get("bounds"))
            .and_then(Value::as_array);

        // Parallel to `layout.nodeIndex`; only present when the capture requested
        // `includePaintOrder`.
        let paint_orders = layout
            .and_then(|layout| layout.get("paintOrders"))
            .and_then(Value::as_array);

        for (snapshot_index, backend_id) in backend_ids.iter().enumerate() {
            let Some(backend_id) = backend_id.as_i64() else {
                continue;
            };

            let mut tag = node_names
                .get(snapshot_index)
                .copied()
                .flatten()
                .filter(|name| !name.starts_with('#'))
                .map(|name| name.to_ascii_lowercase());

            // The AX role cannot tell a file input from a text one; the DOM
            // `type` attribute can, and agents act on it (upload vs type).
            if tag.as_deref() == Some("input")
                && let Some(kind) = attributes
                    .and_then(|attributes| attributes.get(snapshot_index))
                    .and_then(Value::as_array)
                    .and_then(|attributes| attribute_value(&strings, attributes, "type"))
                && !kind.is_empty()
            {
                tag = Some(format!("input[type={kind}]"));
            }

            let mut info = LayoutInfo {
                is_clickable: clickable_set.contains(&(snapshot_index as u64)),
                tag,
                bounds: None,
                paint_order: None,
                document_index,
            };

            if let Some(layout_idx) = layout_index_map.get(&(snapshot_index as u64)) {
                info.paint_order = paint_orders
                    .and_then(|orders| orders.get(*layout_idx))
                    .and_then(Value::as_i64);

                if let Some(entry) = bounds.and_then(|bounds| bounds.get(*layout_idx))
                    && let Some(quad) = entry.as_array()
                    && quad.len() >= 4
                {
                    info.bounds = Some(Rect {
                        x: quad[0].as_f64().unwrap_or(0.0) / dpr,
                        y: quad[1].as_f64().unwrap_or(0.0) / dpr,
                        width: quad[2].as_f64().unwrap_or(0.0) / dpr,
                        height: quad[3].as_f64().unwrap_or(0.0) / dpr,
                    });
                }
            }

            lookup.insert(backend_id, info);
        }
    }

    lookup
}

/// Internal view over one accessibility node.
struct AxNode {
    role: String,
    name: String,
    value: Option<String>,
    states: Vec<String>,
    backend_id: Option<i64>,
    ignored: bool,
    child_ids: Vec<String>,
    children: Vec<usize>,
    has_parent: bool,
}

/// True when any indexed element's box extends below `viewport_height`.
///
/// Used to decide whether to append [`SCROLL_HINT`]. A non-positive viewport
/// (for example, when the probe failed) counts as unknown and never triggers it.
pub(crate) fn has_content_below(elements: &[SnapshotElement], viewport_height: f64) -> bool {
    if viewport_height <= 0.0 {
        return false;
    }
    elements.iter().any(|element| {
        element
            .bounds
            .is_some_and(|bounds| bounds.y + bounds.height > viewport_height)
    })
}

/// Collects every descendant frame id from a `Page.getFrameTree` `childFrames`
/// array (recursively).
fn collect_child_frame_ids(children: &[Value], out: &mut Vec<String>) {
    for child in children {
        if let Some(id) = child.pointer("/frame/id").and_then(Value::as_str) {
            out.push(id.to_string());
        }
        if let Some(grandchildren) = child.get("childFrames").and_then(Value::as_array) {
            collect_child_frame_ids(grandchildren, out);
        }
    }
}

/// Frame-free form of [`assemble_frames`], used by the tests; production code
/// always has the frame map and calls [`assemble_frames`] directly.
#[cfg(test)]
pub(crate) fn assemble(
    ax_tree: &Value,
    layout: &HashMap<i64, LayoutInfo>,
    viewport_height: Option<f64>,
) -> (String, Vec<SnapshotElement>) {
    assemble_frames(ax_tree, layout, viewport_height, &HashMap::new())
}

/// Appends one AX tree's raw nodes to `nodes`, recording each `nodeId` in `by_id`
/// under `prefix` so ids from different frames cannot collide.
fn append_ax_nodes(
    raw_nodes: &[Value],
    prefix: &str,
    nodes: &mut Vec<AxNode>,
    by_id: &mut HashMap<String, usize>,
) {
    for raw in raw_nodes {
        let ignored = raw.get("ignored").and_then(Value::as_bool).unwrap_or(false);
        let role = as_string(raw.pointer("/role/value")).unwrap_or_default();
        let name = ax_field(raw, "name").unwrap_or_default();
        let value = ax_field(raw, "value").filter(|value| !value.is_empty());
        let backend_id = raw.get("backendDOMNodeId").and_then(Value::as_i64);
        let child_ids: Vec<String> = raw
            .get("childIds")
            .and_then(Value::as_array)
            .map(|ids| {
                ids.iter()
                    .filter_map(Value::as_str)
                    .map(|id| format!("{prefix}{id}"))
                    .collect()
            })
            .unwrap_or_default();
        let has_parent = raw.get("parentId").is_some();

        let index = nodes.len();
        if let Some(id) = raw.get("nodeId").and_then(Value::as_str) {
            by_id.insert(format!("{prefix}{id}"), index);
        }
        nodes.push(AxNode {
            role,
            name,
            value,
            states: ax_states(raw),
            backend_id,
            ignored,
            child_ids,
            children: Vec::new(),
            has_parent,
        });
    }
}

/// Assembles the renderable text and the indexable element list from an AX tree
/// and the layout lookup.
///
/// Nodes are visited in document order. Interactive nodes are assigned a stable
/// index; structural, named, or text-bearing nodes are printed for context.
/// Interactive candidates that are fully covered by a later-painted box
/// ([`is_occluded`]) are dropped, and cross-origin frames still render a
/// placeholder. When `viewport_height` is known and indexed content extends
/// below it, [`SCROLL_HINT`] is appended.
///
/// Same-origin subframe AX trees passed in `frames` (keyed by the backend node id
/// of the `<iframe>` that hosts each frame) are merged in: a subframe is a
/// separate document, so the main frame's AX tree omits it and it would otherwise
/// render as a bare `<iframe>` placeholder. Each frame's nodes are appended (with
/// their ids namespaced) and its root re-parented under the host iframe, so the
/// ordinary traversal renders the frame's content.
pub(crate) fn assemble_frames(
    ax_tree: &Value,
    layout: &HashMap<i64, LayoutInfo>,
    viewport_height: Option<f64>,
    frames: &HashMap<i64, Value>,
) -> (String, Vec<SnapshotElement>) {
    let empty = Vec::new();
    let raw_nodes = ax_tree
        .get("nodes")
        .and_then(Value::as_array)
        .unwrap_or(&empty);

    let mut nodes: Vec<AxNode> = Vec::with_capacity(raw_nodes.len());
    let mut by_id: HashMap<String, usize> = HashMap::with_capacity(raw_nodes.len());
    append_ax_nodes(raw_nodes, "", &mut nodes, &mut by_id);

    // Merge each same-origin subframe's tree under its host iframe node.
    let hosts: Vec<(usize, i64)> = nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| node.role.eq_ignore_ascii_case("iframe"))
        .filter_map(|(index, node)| node.backend_id.map(|backend| (index, backend)))
        .collect();
    for (host_index, backend_id) in hosts {
        let Some(sub_tree) = frames.get(&backend_id) else {
            continue;
        };
        let prefix = format!("f{backend_id}:");
        let start = nodes.len();
        let sub_nodes = sub_tree
            .get("nodes")
            .and_then(Value::as_array)
            .unwrap_or(&empty);
        append_ax_nodes(sub_nodes, &prefix, &mut nodes, &mut by_id);

        // Link the frame's root under the iframe so traversal reaches it, and stop
        // it being treated as a second root.
        if start < nodes.len() {
            nodes[start].has_parent = true;
            if let Some(root_id) = sub_nodes
                .first()
                .and_then(|node| node.get("nodeId"))
                .and_then(Value::as_str)
            {
                nodes[host_index]
                    .child_ids
                    .push(format!("{prefix}{root_id}"));
            }
        }
    }

    // Resolve child ids to indices (second pass, so parents always exist).
    for node in &mut nodes {
        node.children = node
            .child_ids
            .iter()
            .filter_map(|id| by_id.get(id).copied())
            .collect();
    }

    // A parent whose accessible name is exactly the concatenation of its text
    // children would otherwise print the same text twice: once as the parent's
    // name and once per child. Mark those children redundant so they are not
    // rendered. Conservative - a name that does not match (after whitespace
    // collapsing) leaves the children in place.
    let mut redundant = vec![false; nodes.len()];
    for parent in &nodes {
        if parent.name.is_empty() || parent.children.is_empty() {
            continue;
        }
        let mut text = String::new();
        let mut all_text = true;
        for &child in &parent.children {
            let child = &nodes[child];
            if !matches!(child.role.as_str(), "StaticText" | "InlineTextBox") {
                all_text = false;
                break;
            }
            text.push_str(&child.name);
        }
        let normalize = |value: &str| value.split_whitespace().collect::<Vec<_>>().join(" ");
        if all_text && !text.is_empty() && normalize(&text) == normalize(&parent.name) {
            for &child in &parent.children {
                redundant[child] = true;
            }
        }
    }

    // Roots are nodes without a parent (normally the RootWebArea), rendered in
    // the order they appear in the protocol response.
    let roots: Vec<usize> = nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| !node.has_parent)
        .map(|(index, _)| index)
        .collect();

    let mut lines: Vec<String> = Vec::new();
    let mut elements: Vec<SnapshotElement> = Vec::new();
    let mut visited = vec![false; nodes.len()];
    let mut next_index: u32 = 0;

    // When a layout snapshot is available, a node missing from it has no box and
    // is treated as invisible. If the snapshot failed entirely, we fall back to
    // indexing every interactive node rather than emitting nothing.
    let have_layout = !layout.is_empty();

    // Every laid-out box plus its paint order, grouped by document. Bounds and
    // paint order are per-frame, so an occlusion test must only ever compare boxes
    // from the *same* document (mixing an iframe's own box with the parent frame's
    // nodes yields false positives).
    let mut ordered_rects: HashMap<usize, Vec<(Rect, Option<i64>)>> = HashMap::new();
    for info in layout.values() {
        if let Some(bounds) = info.bounds {
            ordered_rects
                .entry(info.document_index)
                .or_default()
                .push((bounds, info.paint_order));
        }
    }

    let mut stack: Vec<(usize, usize)> = roots.iter().rev().map(|&root| (root, 0)).collect();
    while let Some((index, depth)) = stack.pop() {
        if std::mem::replace(&mut visited[index], true) {
            continue;
        }
        let node = &nodes[index];

        let is_container = CONTAINER_ROLES.contains(&node.role.as_str());
        let is_iframe = node.role.eq_ignore_ascii_case("iframe");
        let is_skipped_role = SKIP_AX_ROLES.contains(&node.role.as_str());
        let lookup = node.backend_id.and_then(|id| layout.get(&id));
        let is_clickable = lookup.map(|info| info.is_clickable).unwrap_or(false);
        let visible = if have_layout {
            lookup.and_then(|info| info.bounds).is_some()
        } else {
            true
        };
        let interactive = INTERACTIVE_AX_ROLES.contains(&node.role.as_str()) || is_clickable;

        let mut rendered = false;
        let mut child_depth = depth + 1;

        // A cross-origin frame has no layout entry (its document is not
        // captured), so it would otherwise vanish. Always render a placeholder
        // for it - role/title only, never an index. This also covers same-origin
        // frames whose node is present but unnamed.
        if is_iframe && !node.ignored {
            let mut line = String::new();
            line.push_str(&"  ".repeat(depth.min(MAX_INDENT)));
            line.push_str("<iframe>");
            let name = clean(&node.name, MAX_TEXT_LEN);
            if !name.is_empty() {
                line.push_str(&format!(" \"{name}\""));
            }
            lines.push(line);
            rendered = true;
        }

        if !node.ignored && !is_skipped_role && !is_container && !is_iframe {
            let tag = lookup
                .and_then(|info| info.tag.clone())
                .unwrap_or_else(|| node.role.clone());

            // Interactive node that is fully covered by a later-painted box is
            // not a real target: drop it so agents do not click through an
            // overlay. Conservative - see [`is_occluded`].
            let candidate = interactive && visible;
            let occluded = candidate
                && lookup
                    .and_then(|info| {
                        let bounds = info.bounds?;
                        let rects = ordered_rects.get(&info.document_index)?;
                        Some(is_occluded((bounds, info.paint_order), rects))
                    })
                    .unwrap_or(false);
            let index_it = candidate && !occluded;
            // Print anything interactive, structural, or that carries text.
            let show = !occluded
                && !redundant[index]
                && (index_it
                    || STRUCTURAL_AX_ROLES.contains(&node.role.as_str())
                    || !node.name.is_empty());

            if show {
                let assigned = if index_it {
                    let assigned = next_index;
                    next_index += 1;
                    Some(assigned)
                } else {
                    None
                };

                let name = clean(&node.name, MAX_TEXT_LEN);
                let mut line = String::new();
                line.push_str(&"  ".repeat(depth.min(MAX_INDENT)));
                if let Some(assigned) = assigned {
                    line.push_str(&format!("[{assigned}]"));
                }
                line.push('<');
                line.push_str(&tag);
                line.push('>');
                if !name.is_empty() {
                    line.push_str(&format!(" \"{name}\""));
                }
                if let Some(value) = &node.value {
                    line.push_str(&format!(" = \"{}\"", clean(value, MAX_TEXT_LEN)));
                }
                if !node.states.is_empty() {
                    line.push_str(&format!(" ({})", node.states.join(", ")));
                }
                lines.push(line);
                rendered = true;

                if let Some(assigned) = assigned
                    && let Some(backend_node_id) = node.backend_id
                {
                    elements.push(SnapshotElement {
                        index: assigned,
                        backend_node_id,
                        role: node.role.to_string(),
                        name,
                        value: node.value.clone(),
                        states: node.states.clone(),
                        tag: lookup.and_then(|info| info.tag.clone()),
                        bounds: lookup.and_then(|info| info.bounds),
                    });
                }
            }
        }

        if is_iframe {
            // A same-origin frame's content is merged under the placeholder, so
            // nest it one level deeper; other containers add no structure.
            child_depth = depth + 1;
        } else if is_container {
            child_depth = depth;
        }
        if !rendered && !is_skipped_role && !node.ignored && node.name.is_empty() {
            // An unnamed, unprinted wrapper should not inflate indentation.
            child_depth = depth;
        }

        for &child in node.children.iter().rev() {
            stack.push((child, child_depth));
        }
    }

    let mut text = lines.join("\n");
    if let Some(viewport_height) = viewport_height
        && has_content_below(&elements, viewport_height)
    {
        if !text.is_empty() {
            text.push('\n');
        }
        text.push_str(SCROLL_HINT);
    }

    (text, elements)
}

impl Page {
    /// Builds the native agent snapshot: the rendered text plus the indexed
    /// interactive elements from which it was produced.
    async fn build_agent_snapshot(&self) -> XcelerateResult<(String, Vec<SnapshotElement>)> {
        // Accessibility has to be enabled before the tree is meaningful. A
        // failure here is non-fatal (the snapshot degrades to layout only), so
        // the error is intentionally ignored.
        let _ = self
            .client
            .execute_raw_with_session(Some(&self.session_id), "Accessibility.enable", json!({}))
            .await;

        let ax_future = self.client.execute_raw_with_session(
            Some(&self.session_id),
            "Accessibility.getFullAXTree",
            json!({}),
        );
        let snapshot_future = self.client.execute_raw_with_session(
            Some(&self.session_id),
            "DOMSnapshot.captureSnapshot",
            json!({
                "computedStyles": REQUIRED_COMPUTED_STYLES,
                "includeDOMRects": true,
                "includePaintOrder": true,
            }),
        );
        // One probe returns both the scale factor (bounding boxes are in device
        // pixels) and the viewport height (for the scroll hint).
        let probe_future = self.client.execute_raw_with_session(
            Some(&self.session_id),
            "Runtime.evaluate",
            json!({
                "expression": "({ dpr: window.devicePixelRatio || 1, innerHeight: window.innerHeight || 0 })",
                "returnByValue": true,
            }),
        );

        let (ax_result, snapshot_result, probe_result) =
            tokio::join!(ax_future, snapshot_future, probe_future);

        let ax_tree = ax_result?;
        let snapshot = snapshot_result.unwrap_or(Value::Null);
        let probe = probe_result
            .ok()
            .and_then(|value| value.pointer("/result/value").cloned());
        let device_pixel_ratio = probe
            .as_ref()
            .and_then(|value| value.get("dpr"))
            .and_then(Value::as_f64)
            .unwrap_or(1.0);
        let viewport_height = probe
            .as_ref()
            .and_then(|value| value.get("innerHeight"))
            .and_then(Value::as_f64)
            .filter(|height| *height > 0.0);

        let layout = build_layout_lookup(&snapshot, device_pixel_ratio);
        // Same-origin subframes are separate documents, so their content is not in
        // the main frame's AX tree; fetch each and merge it in.
        let frames = self.collect_frame_ax_trees().await;
        let (text, elements) = assemble_frames(&ax_tree, &layout, viewport_height, &frames);

        {
            let mut cache = self.snapshot_index.lock().await;
            cache.clear();
            for element in &elements {
                cache.insert(element.index, element.backend_node_id);
            }
        }

        Ok((text, elements))
    }

    /// AX trees of the page's same-origin subframes, keyed by the backend node id
    /// of the `<iframe>` that hosts each frame.
    ///
    /// Best effort: a frame whose owner or tree cannot be resolved (for example a
    /// cross-origin frame) is skipped, and the snapshot falls back to the bare
    /// `<iframe>` placeholder.
    async fn collect_frame_ax_trees(&self) -> HashMap<i64, Value> {
        let mut trees = HashMap::new();
        // `DOM.getFrameOwner` needs the DOM domain enabled.
        let _ = self
            .client
            .execute_raw_with_session(Some(&self.session_id), "DOM.enable", json!({}))
            .await;
        let Ok(frame_tree) = self
            .client
            .execute_raw_with_session(Some(&self.session_id), "Page.getFrameTree", json!({}))
            .await
        else {
            return trees;
        };

        let mut frame_ids = Vec::new();
        if let Some(children) = frame_tree
            .pointer("/frameTree/childFrames")
            .and_then(Value::as_array)
        {
            collect_child_frame_ids(children, &mut frame_ids);
        }

        for frame_id in frame_ids {
            let Ok(owner) = self
                .client
                .execute_raw_with_session(
                    Some(&self.session_id),
                    "DOM.getFrameOwner",
                    json!({ "frameId": frame_id }),
                )
                .await
            else {
                continue;
            };
            let Some(backend_id) = owner.get("backendNodeId").and_then(Value::as_i64) else {
                continue;
            };
            if let Ok(ax) = self
                .client
                .execute_raw_with_session(
                    Some(&self.session_id),
                    "Accessibility.getFullAXTree",
                    json!({ "frameId": frame_id }),
                )
                .await
            {
                trees.insert(backend_id, ax);
            }
        }
        trees
    }

    /// Returns an agent-friendly, indexed snapshot of the page.
    ///
    /// Every interactive element is prefixed with a stable `[index]` (valid for
    /// the most recent snapshot on this page) that can be passed to
    /// [`Page::click_index`]. Semantic, non-interactive nodes are shown for
    /// context, indented by their nesting depth. Ideal for driving a page from
    /// an LLM or script without brittle CSS selectors.
    pub async fn agent_snapshot(&self) -> XcelerateResult<String> {
        let (text, _) = self.build_agent_snapshot().await?;
        Ok(text)
    }

    /// Returns only what changed since the previous call, as a compact line diff.
    ///
    /// A multi-step agent loop usually re-reads the page after every action;
    /// resending the whole snapshot each time is wasteful. This returns a `-`/`+`
    /// line diff against the previous call - or the full snapshot on the first
    /// call, and whenever the two snapshots share no lines (for example after a
    /// navigation, where a diff would be meaningless).
    pub async fn agent_snapshot_diff(&self) -> XcelerateResult<String> {
        let (text, _) = self.build_agent_snapshot().await?;
        let mut last = self.last_snapshot.lock().await;
        let rendered = match last.as_deref() {
            Some(previous) if !previous.is_empty() => diff_snapshots(previous, &text),
            _ => text.clone(),
        };
        *last = Some(text);
        Ok(rendered)
    }

    /// Returns the indexed interactive elements of the page as a JSON array.
    ///
    /// Each entry is `{ index, backendNodeId, role, name?, value?, tag?, bounds? }`.
    /// Also refreshes the index map used by [`Page::click_index`].
    pub async fn snapshot_json(&self) -> XcelerateResult<String> {
        let (_, elements) = self.build_agent_snapshot().await?;
        let values: Vec<Value> = elements.iter().map(SnapshotElement::to_json).collect();
        Ok(Value::Array(values).to_string())
    }

    /// Clicks the element that carried `index` in the most recent snapshot.
    ///
    /// Resolves the node's `backendNodeId` with `DOM.resolveNode` and then
    /// reuses the stealth click path (scroll into view + real mouse events), so
    /// no CSS selector is re-evaluated and click targets cannot drift between
    /// the snapshot and the click.
    pub async fn click_index(self: Arc<Self>, index: u32) -> XcelerateResult<Arc<Self>> {
        let backend_node_id = {
            let cache = self.snapshot_index.lock().await;
            cache.get(&index).copied()
        }
        .ok_or_else(|| {
            XcelerateError::NotFound(format!(
                "snapshot index {index} is unknown; call agent_snapshot first"
            ))
        })?;

        let resolved = self
            .client
            .execute_raw_with_session(
                Some(&self.session_id),
                "DOM.resolveNode",
                json!({ "backendNodeId": backend_node_id, "objectGroup": "xcelerate.snapshot" }),
            )
            .await?;
        let object_id = resolved
            .pointer("/object/objectId")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                XcelerateError::NotFound(format!(
                    "could not resolve a live node for snapshot index {index}"
                ))
            })?;

        let element = Arc::new(Element {
            page: Arc::clone(&self),
            object_id: object_id.to_string(),
        });
        element.click_stealth().await?;
        Ok(self)
    }

    /// Releases the object group held by the last [`Page::click_index`] call.
    pub async fn release_snapshot(&self) -> XcelerateResult<()> {
        let _ = self
            .client
            .execute_raw_with_session(
                Some(&self.session_id),
                "Runtime.releaseObjectGroup",
                json!({ "objectGroup": "xcelerate.snapshot" }),
            )
            .await;
        self.snapshot_index.lock().await.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_ax() -> Value {
        json!({
            "nodes": [
                {
                    "nodeId": "1",
                    "ignored": false,
                    "role": { "value": "RootWebArea" },
                    "name": { "value": "Example" },
                    "childIds": ["2", "3", "4"]
                },
                {
                    "nodeId": "2",
                    "ignored": false,
                    "parentId": "1",
                    "backendDOMNodeId": 10,
                    "role": { "value": "link" },
                    "name": { "value": "Home" },
                    "childIds": []
                },
                {
                    "nodeId": "3",
                    "ignored": false,
                    "parentId": "1",
                    "backendDOMNodeId": 11,
                    "role": { "value": "textbox" },
                    "name": { "value": "Email" },
                    "value": { "value": "a@b.com" },
                    "childIds": []
                },
                {
                    "nodeId": "4",
                    "ignored": false,
                    "parentId": "1",
                    "backendDOMNodeId": 12,
                    "role": { "value": "generic" },
                    "name": { "value": "" },
                    "childIds": ["5"]
                },
                {
                    "nodeId": "5",
                    "ignored": false,
                    "parentId": "4",
                    "backendDOMNodeId": 13,
                    "role": { "value": "button" },
                    "name": { "value": "Sign in" },
                    "childIds": []
                }
            ]
        })
    }

    #[test]
    fn layout_lookup_uses_first_layout_index_and_converts_dpr() {
        let snapshot = json!({
            "strings": ["DIV", "BUTTON"],
            "documents": [{
                "nodes": {
                    "backendNodeId": [100, 200],
                    "nodeName": [0, 1],
                    "isClickable": { "index": [1] }
                },
                "layout": {
                    "nodeIndex": [0, 1],
                    "bounds": [
                        [0.0, 0.0, 20.0, 10.0],
                        [10.0, 10.0, 40.0, 20.0]
                    ],
                    "paintOrders": [3, 7]
                }
            }]
        });

        let lookup = build_layout_lookup(&snapshot, 2.0);
        let button = lookup.get(&200).expect("button layout");
        assert!(button.is_clickable);
        assert_eq!(button.tag.as_deref(), Some("button"));
        assert_eq!(button.paint_order, Some(7));
        assert_eq!(lookup.get(&100).expect("div layout").paint_order, Some(3));
        assert_eq!(
            button.bounds,
            Some(Rect {
                x: 5.0,
                y: 5.0,
                width: 20.0,
                height: 10.0
            })
        );
        assert!(!lookup.get(&100).expect("div layout").is_clickable);
    }

    #[test]
    fn assemble_indexes_interactive_nodes_in_document_order() {
        let (text, elements) = assemble(&sample_ax(), &HashMap::new(), None);

        assert_eq!(elements.len(), 3);
        assert_eq!(elements[0].index, 0);
        assert_eq!(elements[0].role, "link");
        assert_eq!(elements[0].name, "Home");
        assert_eq!(elements[1].index, 1);
        assert_eq!(elements[1].name, "Email");
        assert_eq!(elements[1].value.as_deref(), Some("a@b.com"));
        assert_eq!(elements[2].index, 2);
        assert_eq!(elements[2].name, "Sign in");

        // The container is skipped, so the button is not over-indented.
        assert!(text.contains("[0]<link> \"Home\""));
        assert!(text.contains("[1]<textbox> \"Email\" = \"a@b.com\""));
        assert!(text.contains("[2]<button> \"Sign in\""));
        assert!(!text.contains("RootWebArea"));
    }

    #[test]
    fn assemble_skips_hidden_interactive_nodes() {
        let mut lookup = HashMap::new();
        // Node 10 (Home link) has no bounds -> invisible, so it is not indexed.
        lookup.insert(
            11,
            LayoutInfo {
                bounds: Some(Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 1.0,
                    height: 1.0,
                }),
                is_clickable: true,
                tag: Some("input".into()),
                paint_order: None,
                document_index: 0,
            },
        );
        let (_, elements) = assemble(&sample_ax(), &lookup, None);

        // Only the visible textbox (backend 11) is indexed.
        assert_eq!(elements.len(), 1);
        assert_eq!(elements[0].backend_node_id, 11);
        assert_eq!(elements[0].index, 0);
    }

    #[test]
    fn clean_collapses_and_truncates() {
        assert_eq!(clean("  a   b\n c ", 20), "a b c");
        assert_eq!(clean("abcdef", 4), "abc…");
    }

    #[test]
    fn clean_strips_private_use_and_zero_width_characters() {
        // Icon-font glyphs (U+E000..U+F8FF) and a zero-width space must not leak
        // into the rendered name.
        assert_eq!(
            clean("\u{E000}Add\u{200B} to cart\u{E001}", 40),
            "Add to cart"
        );
    }

    #[test]
    fn diff_reports_no_change() {
        assert_eq!(
            diff_snapshots("a\nb", "a\nb"),
            "<no change since the previous snapshot>"
        );
    }

    #[test]
    fn diff_reports_added_and_removed_lines() {
        assert_eq!(diff_snapshots("a\nb", "a\nc"), "-b\n+c");
    }

    #[test]
    fn diff_returns_current_when_nothing_is_shared() {
        assert_eq!(diff_snapshots("a\nb", "x\ny"), "x\ny");
    }

    #[test]
    fn assemble_surfaces_ax_state_flags() {
        let ax = json!({
            "nodes": [
                { "nodeId": "1", "ignored": false, "role": { "value": "RootWebArea" }, "name": { "value": "T" }, "childIds": ["2"] },
                { "nodeId": "2", "ignored": false, "parentId": "1", "backendDOMNodeId": 20,
                  "role": { "value": "checkbox" }, "name": { "value": "Accept" },
                  "properties": [
                      { "name": "checked", "value": { "type": "tristate", "value": "true" } },
                      { "name": "disabled", "value": { "type": "boolean", "value": true } },
                      { "name": "expanded", "value": { "type": "boolean", "value": false } }
                  ],
                  "childIds": [] }
            ]
        });
        let (text, elements) = assemble(&ax, &HashMap::new(), None);
        assert_eq!(elements.len(), 1);
        assert_eq!(elements[0].states, vec!["checked", "disabled"]);
        assert!(
            text.contains("[0]<checkbox> \"Accept\" (checked, disabled)"),
            "{text}"
        );
    }

    #[test]
    fn assemble_marks_a_mixed_tristate() {
        let ax = json!({
            "nodes": [
                { "nodeId": "1", "ignored": false, "role": { "value": "RootWebArea" }, "name": { "value": "T" }, "childIds": ["2"] },
                { "nodeId": "2", "ignored": false, "parentId": "1", "backendDOMNodeId": 20,
                  "role": { "value": "checkbox" }, "name": { "value": "All" },
                  "properties": [ { "name": "checked", "value": { "type": "tristate", "value": "mixed" } } ],
                  "childIds": [] }
            ]
        });
        let (_, elements) = assemble(&ax, &HashMap::new(), None);
        assert_eq!(elements[0].states, vec!["checked=mixed"]);
    }

    #[test]
    fn assemble_drops_redundant_static_text_children() {
        let ax = json!({
            "nodes": [
                { "nodeId": "1", "ignored": false, "role": { "value": "RootWebArea" }, "name": { "value": "T" }, "childIds": ["2"] },
                { "nodeId": "2", "ignored": false, "parentId": "1", "backendDOMNodeId": 30,
                  "role": { "value": "button" }, "name": { "value": "Sign in" }, "childIds": ["3"] },
                { "nodeId": "3", "ignored": false, "parentId": "2", "backendDOMNodeId": 31,
                  "role": { "value": "StaticText" }, "name": { "value": "Sign in" }, "childIds": [] }
            ]
        });
        let (text, _) = assemble(&ax, &HashMap::new(), None);
        assert_eq!(text.matches("Sign in").count(), 1, "{text}");
    }

    #[test]
    fn layout_lookup_enriches_an_input_with_its_type() {
        let snapshot = json!({
            "strings": ["INPUT", "type", "file"],
            "documents": [{
                "nodes": {
                    "backendNodeId": [100],
                    "nodeName": [0],
                    "attributes": [[1, 2]]
                }
            }]
        });
        let lookup = build_layout_lookup(&snapshot, 1.0);
        assert_eq!(
            lookup.get(&100).expect("input layout").tag.as_deref(),
            Some("input[type=file]")
        );
    }

    /// Layout lookup for `sample_ax`'s backend ids, each with a paint order.
    fn layout_with(entries: &[(i64, f64, f64, f64, f64)]) -> HashMap<i64, LayoutInfo> {
        entries
            .iter()
            .map(|&(id, x, y, width, height)| {
                (
                    id,
                    LayoutInfo {
                        bounds: Some(Rect {
                            x,
                            y,
                            width,
                            height,
                        }),
                        is_clickable: false,
                        tag: None,
                        paint_order: Some(0),
                        document_index: 0,
                    },
                )
            })
            .collect()
    }

    #[test]
    fn is_occluded_requires_containment_size_and_higher_paint_order() {
        let target = (
            Rect {
                x: 10.0,
                y: 10.0,
                width: 20.0,
                height: 20.0,
            },
            Some(1),
        );
        let cover = (
            Rect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
            },
            Some(2),
        );

        // Fully contained in a strictly larger, later-painted box.
        assert!(is_occluded(target, &[cover]));
        // The cover is painted first, so the target sits on top of it.
        assert!(!is_occluded(target, &[(cover.0, Some(0))]));
        // A same-sized box is not strictly larger.
        assert!(!is_occluded(target, &[(target.0, Some(2))]));
        // Overlap below the 99% containment threshold.
        let partial = (
            Rect {
                x: 25.0,
                y: 25.0,
                width: 100.0,
                height: 100.0,
            },
            Some(2),
        );
        assert!(!is_occluded(target, &[partial]));
        // Unknown paint orders are never treated as occluding/occluded.
        assert!(!is_occluded((target.0, None), &[cover]));
        assert!(!is_occluded(target, &[(cover.0, None)]));
    }

    #[test]
    fn assemble_drops_interactive_node_covered_by_later_painted_box() {
        let mut layout = HashMap::new();
        // Backend 10 (Home link) is painted under an overlay that covers it.
        layout.insert(
            10,
            LayoutInfo {
                bounds: Some(Rect {
                    x: 10.0,
                    y: 10.0,
                    width: 20.0,
                    height: 20.0,
                }),
                is_clickable: true,
                tag: Some("a".into()),
                paint_order: Some(1),
                document_index: 0,
            },
        );
        layout.insert(
            99,
            LayoutInfo {
                bounds: Some(Rect {
                    x: 5.0,
                    y: 5.0,
                    width: 50.0,
                    height: 50.0,
                }),
                is_clickable: false,
                tag: Some("div".into()),
                paint_order: Some(2),
                document_index: 0,
            },
        );
        // The remaining interactive nodes are laid out clear of the overlay.
        layout.insert(
            11,
            LayoutInfo {
                bounds: Some(Rect {
                    x: 0.0,
                    y: 100.0,
                    width: 60.0,
                    height: 20.0,
                }),
                is_clickable: false,
                tag: Some("input".into()),
                paint_order: Some(0),
                document_index: 0,
            },
        );
        layout.insert(
            13,
            LayoutInfo {
                bounds: Some(Rect {
                    x: 0.0,
                    y: 200.0,
                    width: 60.0,
                    height: 20.0,
                }),
                is_clickable: false,
                tag: Some("button".into()),
                paint_order: Some(0),
                document_index: 0,
            },
        );

        let (text, elements) = assemble(&sample_ax(), &layout, None);

        // The covered link is dropped; the other interactive nodes remain.
        assert_eq!(elements.len(), 2);
        assert!(elements.iter().all(|element| element.backend_node_id != 10));
        assert!(!text.contains("\"Home\""));
        assert!(text.contains("[0]<input> \"Email\""));
    }

    #[test]
    fn assemble_ignores_occluders_from_another_document() {
        // A button in the main document (document 0) would look "covered" by an
        // iframe's own document box (document 1), which sits at (0,0) 300x150 in
        // its own frame's coordinate space. Cross-frame boxes must be ignored.
        let mut lookup = HashMap::new();
        lookup.insert(
            10,
            LayoutInfo {
                bounds: Some(Rect {
                    x: 8.0,
                    y: 80.0,
                    width: 85.0,
                    height: 21.0,
                }),
                is_clickable: false,
                tag: Some("button".into()),
                paint_order: Some(1),
                document_index: 0,
            },
        );
        lookup.insert(
            99,
            LayoutInfo {
                bounds: Some(Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 300.0,
                    height: 150.0,
                }),
                is_clickable: false,
                tag: Some("iframe".into()),
                paint_order: Some(3),
                document_index: 1,
            },
        );

        let ax = json!({ "nodes": [
            { "nodeId": "1", "ignored": false, "role": { "value": "RootWebArea" }, "name": { "value": "T" }, "childIds": ["2"] },
            { "nodeId": "2", "ignored": false, "parentId": "1", "backendDOMNodeId": 10,
              "role": { "value": "button" }, "name": { "value": "Go" }, "childIds": [] }
        ]});

        let (text, elements) = assemble(&ax, &lookup, None);
        assert_eq!(elements.len(), 1, "button dropped: {text}");
        assert_eq!(elements[0].name, "Go");
    }

    #[test]
    fn assemble_merges_a_subframe_tree_under_its_iframe() {
        let main = json!({ "nodes": [
            { "nodeId": "1", "ignored": false, "role": { "value": "RootWebArea" }, "name": { "value": "T" }, "childIds": ["2"] },
            { "nodeId": "2", "ignored": false, "parentId": "1", "backendDOMNodeId": 50,
              "role": { "value": "Iframe" }, "name": { "value": "" }, "childIds": [] }
        ]});
        // The subframe reuses node ids ("1"/"2"); the merge must namespace them.
        let sub = json!({ "nodes": [
            { "nodeId": "1", "ignored": false, "role": { "value": "RootWebArea" }, "name": { "value": "frame" }, "childIds": ["2"] },
            { "nodeId": "2", "ignored": false, "parentId": "1", "backendDOMNodeId": 900,
              "role": { "value": "button" }, "name": { "value": "Frame button" }, "childIds": [] }
        ]});
        let mut frames = HashMap::new();
        frames.insert(50, sub);

        let (text, elements) = assemble_frames(&main, &HashMap::new(), None, &frames);

        assert_eq!(elements.len(), 1, "{text}");
        assert_eq!(elements[0].index, 0);
        assert_eq!(elements[0].name, "Frame button");
        assert!(text.contains("<iframe>"), "{text}");
        assert!(text.contains("[0]<button> \"Frame button\""), "{text}");
    }

    #[test]
    fn assemble_renders_iframe_placeholder_without_index() {
        let ax = json!({
            "nodes": [
                {
                    "nodeId": "1",
                    "ignored": false,
                    "role": { "value": "RootWebArea" },
                    "name": { "value": "Page" },
                    "childIds": ["2"]
                },
                {
                    "nodeId": "2",
                    "ignored": false,
                    "parentId": "1",
                    "role": { "value": "Iframe" },
                    "name": { "value": "https://example.com/embed" },
                    "childIds": []
                }
            ]
        });
        // No layout entry for the cross-origin frame.
        let (text, elements) = assemble(&ax, &HashMap::new(), None);

        assert!(elements.is_empty());
        assert!(text.contains("<iframe> \"https://example.com/embed\""));
        assert!(!text.contains('['));
    }

    #[test]
    fn assemble_appends_scroll_hint_when_content_is_below_viewport() {
        let layout = layout_with(&[
            (10, 0.0, 0.0, 10.0, 10.0),
            (11, 0.0, 250.0, 10.0, 10.0),
            (13, 0.0, 20.0, 10.0, 10.0),
        ]);

        let (text, elements) = assemble(&sample_ax(), &layout, Some(200.0));
        assert_eq!(elements.len(), 3);
        assert!(has_content_below(&elements, 200.0));
        assert!(text.ends_with(SCROLL_HINT));

        let (fits, _) = assemble(&sample_ax(), &layout, Some(1000.0));
        assert!(!has_content_below(&elements, 1000.0));
        assert!(!fits.contains("more content below"));

        // An unknown viewport never adds the hint.
        let (unknown, _) = assemble(&sample_ax(), &layout, None);
        assert!(!unknown.contains("more content below"));
    }
}
