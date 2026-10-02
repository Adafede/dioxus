// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! `WebMCP` tool registration for the W3C `WebMCP` proposal.
//!
//! Each app exposes a single read-only `<app_id>_capabilities` tool on its
//! page's [`WebMcp`](https://webmachinelearning.github.io/webmcp/) document
//! context (`document.modelContext`), so AI agents can discover what an app
//! consumes and produces before invoking it. The emitted script is a guarded
//! no-op in browsers without a `WebMCP` context, so it is safe to inject
//! unconditionally via [`DocumentHead`](crate::document::DocumentHead).

use dioxus::prelude::*;
use std::fmt::Write;

/// Metadata an app exposes as its read-only `WebMCP` `capabilities` tool.
#[derive(Clone, Copy, Debug, Eq, Props, PartialEq)]
pub struct WebMcpConfig {
    /// Stable kebab-case identifier used to derive the tool name
    /// (`"<app_id>_capabilities"`).
    pub app_id: &'static str,
    /// Human-readable title for the tool.
    pub title: &'static str,
    /// Human-readable description of the tool's purpose.
    pub description: &'static str,
    /// Human-readable input surface names (e.g. `"smiles_list"`).
    pub inputs: &'static [&'static str],
    /// Human-readable output surface names (e.g. `"cx_smiles"`).
    pub outputs: &'static [&'static str],
}

/// Returns `s` as a JSON string literal (also valid as a JS string literal).
fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Returns the inline `<script>` body that registers one read-only tool on
/// the page's `WebMCP` document context (guarded IIFE).
///
/// Crate-private: [`DocumentHead`](crate::DocumentHead) builds this from the
/// `webmcp` prop, and apps pass a [`WebMcpConfig`] rather than a script.
#[must_use]
pub(crate) fn capabilities_script(cfg: WebMcpConfig) -> String {
    let tool_name = format!("{}_capabilities", cfg.app_id);
    let name = json_string(&tool_name);
    let title = json_string(cfg.title);
    let description = json_string(cfg.description);
    let inputs: String = cfg
        .inputs
        .iter()
        .map(|s| json_string(s))
        .collect::<Vec<_>>()
        .join(", ");
    let outputs: String = cfg
        .outputs
        .iter()
        .map(|s| json_string(s))
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        r#"(function() {{
  const mc = document.modelContext || (navigator && navigator.modelContext);
  if (!mc || typeof mc.registerTool !== "function") return;
  mc.registerTool({{
    name: {name},
    title: {title},
    description: {description},
    inputSchema: {{ "type": "object", "properties": {{}}, "additionalProperties": false }},
    execute: async () => ({{ name: {title}, description: {description}, inputs: [{inputs}], outputs: [{outputs}] }}),
    annotations: {{ readOnlyHint: true }}
  }}).catch(() => {{}});
}})();"#
    )
}

#[cfg(test)]
mod tests;
