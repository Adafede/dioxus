// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::models::PreviewNode;
use dioxus::prelude::*;

#[component]
pub fn PreviewTreeView(nodes: Vec<PreviewNode>) -> Element {
    rsx! {
        ul { class: "tree-root",
            for node in nodes {
                PreviewTreeNode { node }
            }
        }
    }
}

#[component]
fn PreviewTreeNode(node: PreviewNode) -> Element {
    rsx! {
        li { class: "tree-node",
            details {
                open: true,
                summary { class: "tree-label", "{node.label}" }
                if !node.children.is_empty() {
                    ul {
                        for child in node.children {
                            PreviewTreeNode { node: child }
                        }
                    }
                }
            }
        }
    }
}
