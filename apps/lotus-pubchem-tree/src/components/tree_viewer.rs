// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Interactive tree explorer inspired by OTT-style navigation.
//!
//! Features:
//! - Stable node identity by traversal path
//! - Expand/collapse hierarchy with focus + reveal
//! - Priority-queue ranked search hits
//! - Selected node info panel
//! - Scrollable viewport for large trees

use crate::models::PreviewNode;
use dioxus::prelude::*;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};

#[derive(Clone, PartialEq)]
pub struct TreeViewerConfig {
    /// Enable search functionality
    pub search_enabled: bool,
    /// Maximum display height before scrolling (in pixels)
    pub max_height: Option<usize>,
    /// Show node counts
    pub show_counts: bool,
    /// Initial expanded state depth (0 = all collapsed, 1 = only roots expanded)
    pub initial_expansion_depth: usize,
}

impl Default for TreeViewerConfig {
    fn default() -> Self {
        Self {
            search_enabled: true,
            max_height: Some(600),
            show_counts: false,
            initial_expansion_depth: 0,
        }
    }
}

#[derive(Clone, Debug)]
struct IndexedNode {
    key: String,
    label: String,
    label_lower: String,
    breadcrumbs: String,
    path: Vec<usize>,
    depth: usize,
    child_count: usize,
}

#[derive(Clone, Debug)]
struct SearchHit {
    key: String,
    label: String,
    breadcrumbs: String,
    path: Vec<usize>,
    depth: usize,
}

#[component]
pub fn InteractiveTreeViewer(
    nodes: Vec<PreviewNode>,
    #[props(default)] config: TreeViewerConfig,
) -> Element {
    let mut search_query = use_signal(String::new);
    let mut expanded_nodes = use_signal(HashSet::<String>::new);
    let mut selected_node = use_signal(|| None::<String>);
    let sorted_nodes = sort_tree_by_children(nodes);

    // Recompute baseline expansion when the tree changes.
    use_effect({
        let nodes = sorted_nodes.clone();
        move || {
            expanded_nodes.set(initial_expanded_keys(
                &nodes,
                config.initial_expansion_depth,
            ));
            selected_node.set(None);
        }
    });

    let indexed_nodes = flatten_tree(&sorted_nodes);
    let indexed_by_key: HashMap<String, IndexedNode> = indexed_nodes
        .iter()
        .cloned()
        .map(|node| (node.key.clone(), node))
        .collect();
    let total_nodes = indexed_nodes.len();

    let query_trimmed = search_query.read().trim().to_string();
    let query_lower = if query_trimmed.is_empty() {
        None
    } else {
        Some(query_trimmed.to_lowercase())
    };

    let ranked_hits = rank_matches(&indexed_nodes, &query_trimmed, 25);
    let top_hit = ranked_hits.first().cloned();

    let selected_meta = selected_node
        .read()
        .as_ref()
        .and_then(|key| indexed_by_key.get(key))
        .cloned();

    let style = match config.max_height {
        Some(height) => format!(
            "max-height: {}px; overflow-y: auto; overflow-x: hidden; padding: 0.5rem;",
            height
        ),
        None => "padding: 0.5rem;".to_string(),
    };

    rsx! {
        div { class: "tree-viewer-container",
            if config.search_enabled {
                div { class: "tree-search",
                    input {
                        r#type: "text",
                        class: "tree-search-input",
                        placeholder: "Search and press Enter for best hit...",
                        value: "{search_query}",
                        oninput: move |evt| search_query.set(evt.value()),
                        onkeydown: move |evt| {
                            if evt.key() == Key::Enter && let Some(hit) = top_hit.clone() {
                                reveal_search_hit(&hit, expanded_nodes, selected_node);
                            }
                        },
                    }
                }
            }

            if let Some(query) = query_lower.as_ref() {
                div { class: "tree-search-info",
                    "{total_nodes} total nodes, {ranked_hits.len()} top matches for \"{query}\""
                }

                div { class: "tree-toolbar",
                    button {
                        class: "tree-toolbar-btn",
                        onclick: {
                            let indexed_nodes = indexed_nodes.clone();
                            move |_| {
                                let keys = indexed_nodes
                                    .iter()
                                    .filter(|node| node.child_count > 0)
                                    .map(|node| node.key.clone())
                                    .collect();
                                expanded_nodes.set(keys);
                            }
                        },
                        "Expand all"
                    }
                    button {
                        class: "tree-toolbar-btn",
                        onclick: move |_| expanded_nodes.set(HashSet::new()),
                        "Collapse all"
                    }
                }

                if ranked_hits.is_empty() {
                    div { class: "tree-search-results empty", "No matching nodes." }
                } else {
                    div { class: "tree-search-results",
                        ul { class: "tree-hit-list",
                            for hit in ranked_hits.iter().cloned() {
                                li { class: "tree-hit-item",
                                    button {
                                        class: "tree-hit-btn",
                                        onclick: move |_| reveal_search_hit(&hit, expanded_nodes, selected_node),
                                        span { class: "tree-hit-label", "{hit.label}" }
                                        span { class: "tree-hit-meta", "{hit.breadcrumbs} · depth {hit.depth}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let Some(meta) = selected_meta {
                div { class: "tree-node-info",
                    strong { "Selected:" }
                    " "
                    span { "{meta.label}" }
                    span { class: "tree-node-info-meta",
                        "Depth {meta.depth} · {meta.child_count} children"
                    }
                    span { class: "tree-node-info-meta", "{meta.breadcrumbs}" }
                }
            }

            div { class: "tree-viewer", style: style,
                ul { class: "tree-root",
                    for (idx, node) in sorted_nodes.into_iter().enumerate() {
                        TreeNode {
                            node,
                            depth: 0,
                            path: vec![idx],
                            expanded_nodes,
                            selected_node,
                            search_query_lower: query_lower.clone(),
                            show_counts: config.show_counts,
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn TreeNode(
    node: PreviewNode,
    depth: usize,
    path: Vec<usize>,
    expanded_nodes: Signal<HashSet<String>>,
    selected_node: Signal<Option<String>>,
    search_query_lower: Option<String>,
    show_counts: bool,
) -> Element {
    let key = path_key(&path);
    let is_expanded = expanded_nodes.read().contains(&key);
    let has_children = !node.children.is_empty();

    let highlight = search_query_lower
        .as_ref()
        .map(|query| node.label.to_lowercase().contains(query))
        .unwrap_or(false);
    let is_selected = selected_node
        .read()
        .as_ref()
        .map(|selected| selected == &key)
        .unwrap_or(false);

    let expanded_icon = if has_children {
        if is_expanded { "▼" } else { "▶" }
    } else {
        "•"
    };

    let toggle_key = key.clone();
    let select_key = key.clone();
    let on_select = move |_| {
        selected_node.set(Some(select_key.clone()));
    };

    let on_toggle = move |_| {
        if has_children {
            let mut exp = expanded_nodes.read().clone();
            if exp.contains(&toggle_key) {
                exp.remove(&toggle_key);
            } else {
                exp.insert(toggle_key.clone());
            }
            expanded_nodes.set(exp);
        }
    };

    let class_str = if highlight {
        "tree-node highlighted"
    } else {
        "tree-node"
    };

    let item_class = if is_selected {
        "tree-item selected-item"
    } else if highlight {
        "tree-item highlighted-item"
    } else {
        "tree-item"
    };

    rsx! {
        li { class: class_str,
            div {
                class: item_class,
                style: format!("padding-left: {}px;", depth * 20),
                button {
                    class: "tree-toggle",
                    onclick: on_toggle,
                    disabled: !has_children,
                    "{expanded_icon}"
                }
                button {
                    class: "tree-node-btn",
                    onclick: on_select,
                    span { class: "tree-label", "{node.label}" }
                }
                if show_counts && has_children {
                    span { class: "tree-count", "({node.children.len()})" }
                }
            }

            if is_expanded && has_children {
                ul { class: "tree-children",
                    for (child_idx, child) in node.children.into_iter().enumerate() {
                        TreeNode {
                            node: child,
                            depth: depth + 1,
                            path: child_path(&path, child_idx),
                            expanded_nodes,
                            selected_node,
                            search_query_lower: search_query_lower.clone(),
                            show_counts,
                        }
                    }
                }
            }
        }
    }
}

fn reveal_search_hit(
    hit: &SearchHit,
    mut expanded_nodes: Signal<HashSet<String>>,
    mut selected_node: Signal<Option<String>>,
) {
    let mut expanded = expanded_nodes.read().clone();
    for key in ancestor_keys(&hit.path) {
        expanded.insert(key);
    }
    expanded_nodes.set(expanded);
    selected_node.set(Some(hit.key.clone()));
}

fn path_key(path: &[usize]) -> String {
    let mut key = String::from("n");
    for index in path {
        key.push('.');
        key.push_str(&index.to_string());
    }
    key
}

fn ancestor_keys(path: &[usize]) -> Vec<String> {
    let mut keys = Vec::new();
    for end in 1..path.len() {
        keys.push(path_key(&path[..end]));
    }
    keys
}

fn child_path(parent_path: &[usize], child_index: usize) -> Vec<usize> {
    let mut path = parent_path.to_vec();
    path.push(child_index);
    path
}

fn initial_expanded_keys(nodes: &[PreviewNode], max_depth: usize) -> HashSet<String> {
    let mut expanded = HashSet::new();
    if max_depth == 0 {
        return expanded;
    }

    for (root_index, node) in nodes.iter().enumerate() {
        let mut path = vec![root_index];
        collect_expanded_keys(node, &mut path, 1, max_depth, &mut expanded);
    }

    expanded
}

fn collect_expanded_keys(
    node: &PreviewNode,
    path: &mut Vec<usize>,
    current_depth: usize,
    max_depth: usize,
    expanded: &mut HashSet<String>,
) {
    if current_depth <= max_depth {
        expanded.insert(path_key(path));
    }

    if current_depth >= max_depth {
        return;
    }

    for (child_index, child) in node.children.iter().enumerate() {
        path.push(child_index);
        collect_expanded_keys(child, path, current_depth + 1, max_depth, expanded);
        path.pop();
    }
}

fn flatten_tree(nodes: &[PreviewNode]) -> Vec<IndexedNode> {
    let mut indexed = Vec::new();
    for (root_index, root) in nodes.iter().enumerate() {
        let mut path = vec![root_index];
        let mut labels = Vec::new();
        flatten_node(root, &mut path, &mut labels, &mut indexed);
    }
    indexed
}

fn flatten_node(
    node: &PreviewNode,
    path: &mut Vec<usize>,
    labels: &mut Vec<String>,
    indexed: &mut Vec<IndexedNode>,
) {
    labels.push(node.label.clone());
    indexed.push(IndexedNode {
        key: path_key(path),
        label: node.label.clone(),
        label_lower: node.label.to_lowercase(),
        breadcrumbs: labels.join(" > "),
        path: path.clone(),
        depth: path.len() - 1,
        child_count: node.children.len(),
    });

    for (child_index, child) in node.children.iter().enumerate() {
        path.push(child_index);
        flatten_node(child, path, labels, indexed);
        path.pop();
    }

    labels.pop();
}

fn rank_matches(indexed_nodes: &[IndexedNode], query: &str, limit: usize) -> Vec<SearchHit> {
    if limit == 0 {
        return Vec::new();
    }

    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return Vec::new();
    }

    // Use a max-heap priority queue so best hits are popped first.
    let mut heap = BinaryHeap::<(i32, Reverse<usize>, Reverse<usize>, Reverse<usize>)>::new();

    for (index, node) in indexed_nodes.iter().enumerate() {
        if let Some(base_score) = score_match(&node.label_lower, &query) {
            let depth_penalty = (node.depth as i32) * 12;
            let label_penalty = node.label.len().min(120) as i32;
            let final_score = base_score - depth_penalty - label_penalty;
            heap.push((
                final_score,
                Reverse(node.depth),
                Reverse(node.label.len()),
                Reverse(index),
            ));
        }
    }

    let mut hits = Vec::new();
    while hits.len() < limit {
        let Some((_, _, _, Reverse(index))) = heap.pop() else {
            break;
        };
        let node = &indexed_nodes[index];
        hits.push(SearchHit {
            key: node.key.clone(),
            label: node.label.clone(),
            breadcrumbs: node.breadcrumbs.clone(),
            path: node.path.clone(),
            depth: node.depth,
        });
    }

    hits
}

fn sort_tree_by_children(mut nodes: Vec<PreviewNode>) -> Vec<PreviewNode> {
    for node in &mut nodes {
        node.children = sort_tree_by_children(std::mem::take(&mut node.children));
    }
    // Stable sort keeps input order for ties while prioritizing broader branches.
    nodes.sort_by_key(|node| Reverse(node.children.len()));
    nodes
}

fn score_match(label_lower: &str, query: &str) -> Option<i32> {
    if label_lower == query {
        return Some(8_000);
    }

    if label_lower.starts_with(query) {
        return Some(6_200);
    }

    if label_lower
        .split(|ch: char| !ch.is_alphanumeric())
        .any(|token| token == query)
    {
        return Some(5_600);
    }

    if let Some(index) = label_lower.find(query) {
        let boundary_bonus = if index == 0 {
            250
        } else {
            label_lower[..index]
                .chars()
                .last()
                .map(|ch| if ch.is_alphanumeric() { 0 } else { 250 })
                .unwrap_or(0)
        };
        return Some(4_200 - (index as i32 * 8) + boundary_bonus);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tree() -> Vec<PreviewNode> {
        vec![PreviewNode {
            label: "Life".to_string(),
            children: vec![
                PreviewNode {
                    label: "Bacteria".to_string(),
                    children: vec![],
                },
                PreviewNode {
                    label: "Plants".to_string(),
                    children: vec![PreviewNode {
                        label: "Plantae exact".to_string(),
                        children: vec![],
                    }],
                },
            ],
        }]
    }

    #[test]
    fn path_key_is_stable_and_unique() {
        assert_eq!(path_key(&[0]), "n.0");
        assert_eq!(path_key(&[0, 1, 2]), "n.0.1.2");
        assert_ne!(path_key(&[0, 1]), path_key(&[1, 0]));
    }

    #[test]
    fn priority_queue_returns_best_hits_first() {
        let indexed = flatten_tree(&sample_tree());
        let hits = rank_matches(&indexed, "plantae exact", 5);
        assert!(!hits.is_empty());
        assert_eq!(hits[0].label, "Plantae exact");
        assert_eq!(hits[0].depth, 2);
    }

    #[test]
    fn sort_tree_prioritizes_nodes_with_more_children() {
        let sorted = sort_tree_by_children(vec![
            PreviewNode {
                label: "One child".to_string(),
                children: vec![PreviewNode {
                    label: "child".to_string(),
                    children: vec![],
                }],
            },
            PreviewNode {
                label: "Three children".to_string(),
                children: vec![
                    PreviewNode {
                        label: "a".to_string(),
                        children: vec![],
                    },
                    PreviewNode {
                        label: "b".to_string(),
                        children: vec![],
                    },
                    PreviewNode {
                        label: "c".to_string(),
                        children: vec![],
                    },
                ],
            },
        ]);
        assert_eq!(sorted[0].label, "Three children");
    }
}
