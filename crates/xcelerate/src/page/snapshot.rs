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
const SKIP_AX_ROLES: &[&str] = &["none", "generic", "GenericContainer", "Ignored"];

/// Container roles that are printed without an index but not treated as noise.
const CONTAINER_ROLES: &[&str] = &["RootWebArea", "WebArea", "Iframe"];

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
}

/// An interactive element surfaced by the snapshot, addressable by `index`.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SnapshotElement {
    pub index: u32,
    pub backend_node_id: i64,
    pub role: String,
    pub name: String,
    pub value: Option<String>,
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

/// Collapses whitespace and truncates to `max` characters (adding an ellipsis).
fn clean(text: &str, max: usize) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
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

    for document in documents {
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

        for (snapshot_index, backend_id) in backend_ids.iter().enumerate() {
            let Some(backend_id) = backend_id.as_i64() else {
                continue;
            };

            let tag = node_names
                .get(snapshot_index)
                .copied()
                .flatten()
                .filter(|name| !name.starts_with('#'))
                .map(|name| name.to_ascii_lowercase());

            let mut info = LayoutInfo {
                is_clickable: clickable_set.contains(&(snapshot_index as u64)),
                tag,
                bounds: None,
            };

            if let Some(layout_idx) = layout_index_map.get(&(snapshot_index as u64))
                && let Some(entry) = bounds.and_then(|bounds| bounds.get(*layout_idx))
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
    backend_id: Option<i64>,
    ignored: bool,
    child_ids: Vec<String>,
    children: Vec<usize>,
    has_parent: bool,
}

/// Assembles the renderable text and the indexable element list from an AX tree
/// and the layout lookup.
///
/// Nodes are visited in document order. Interactive nodes are assigned a stable
/// index; structural, named, or text-bearing nodes are printed for context.
pub(crate) fn assemble(
    ax_tree: &Value,
    layout: &HashMap<i64, LayoutInfo>,
) -> (String, Vec<SnapshotElement>) {
    let empty = Vec::new();
    let raw_nodes = ax_tree
        .get("nodes")
        .and_then(Value::as_array)
        .unwrap_or(&empty);

    let mut nodes: Vec<AxNode> = Vec::with_capacity(raw_nodes.len());
    let mut by_id: HashMap<String, usize> = HashMap::with_capacity(raw_nodes.len());

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
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        let has_parent = raw.get("parentId").is_some();

        let index = nodes.len();
        if let Some(id) = raw.get("nodeId").and_then(Value::as_str) {
            by_id.insert(id.to_string(), index);
        }
        nodes.push(AxNode {
            role,
            name,
            value,
            backend_id,
            ignored,
            child_ids,
            children: Vec::new(),
            has_parent,
        });
    }

    // Resolve child ids to indices (second pass, so parents always exist).
    for node in &mut nodes {
        node.children = node
            .child_ids
            .iter()
            .filter_map(|id| by_id.get(id).copied())
            .collect();
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

    let mut stack: Vec<(usize, usize)> = roots.iter().rev().map(|&root| (root, 0)).collect();
    while let Some((index, depth)) = stack.pop() {
        if std::mem::replace(&mut visited[index], true) {
            continue;
        }
        let node = &nodes[index];

        let is_container = CONTAINER_ROLES.contains(&node.role.as_str());
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

        if !node.ignored && !is_skipped_role && !is_container {
            let tag = lookup
                .and_then(|info| info.tag.clone())
                .unwrap_or_else(|| node.role.clone());

            let index_it = interactive && visible;
            // Print anything interactive, structural, or that carries text.
            let show = index_it
                || STRUCTURAL_AX_ROLES.contains(&node.role.as_str())
                || !node.name.is_empty();

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
                        tag: lookup.and_then(|info| info.tag.clone()),
                        bounds: lookup.and_then(|info| info.bounds),
                    });
                }
            }
        }

        if is_container {
            // Containers keep their children at the same level: they add no
            // structure of their own to the rendered text.
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

    (lines.join("\n"), elements)
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
                "includePaintOrder": false,
            }),
        );
        let dpr_future = self.client.execute_raw_with_session(
            Some(&self.session_id),
            "Runtime.evaluate",
            json!({ "expression": "window.devicePixelRatio || 1", "returnByValue": true }),
        );

        let (ax_result, snapshot_result, dpr_result) =
            tokio::join!(ax_future, snapshot_future, dpr_future);

        let ax_tree = ax_result?;
        let snapshot = snapshot_result.unwrap_or(Value::Null);
        let device_pixel_ratio = dpr_result
            .ok()
            .and_then(|value| value.pointer("/result/value").and_then(Value::as_f64))
            .unwrap_or(1.0);

        let layout = build_layout_lookup(&snapshot, device_pixel_ratio);
        let (text, elements) = assemble(&ax_tree, &layout);

        {
            let mut cache = self.snapshot_index.lock().await;
            cache.clear();
            for element in &elements {
                cache.insert(element.index, element.backend_node_id);
            }
        }

        Ok((text, elements))
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
                    ]
                }
            }]
        });

        let lookup = build_layout_lookup(&snapshot, 2.0);
        let button = lookup.get(&200).expect("button layout");
        assert!(button.is_clickable);
        assert_eq!(button.tag.as_deref(), Some("button"));
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
        let (text, elements) = assemble(&sample_ax(), &HashMap::new());

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
            },
        );
        let (_, elements) = assemble(&sample_ax(), &lookup);

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
}
