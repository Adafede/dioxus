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

#![expect(
    clippy::empty_enums,
    reason = "Dioxus Props derive macro generates an empty helper enum"
)]

use dioxus::prelude::*;
use std::fmt::Write;

/// An [EDAM ontology](https://edamontology.org/) term (`topic_*`,
/// `operation_*`, `data_*`, or `format_*`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EdamTerm {
    /// Short EDAM identifier (e.g. `"topic_2258"`, `"operation_0366"`,
    /// `"data_2301"`, `"format_1196"`).
    pub id: &'static str,
    /// Human-readable preferred label (e.g. `"Cheminformatics"`).
    pub label: &'static str,
}

impl EdamTerm {
    /// Const constructor for an [`EdamTerm`].
    #[must_use]
    pub const fn new(id: &'static str, label: &'static str) -> Self {
        Self { id, label }
    }

    /// Canonical EDAM URI (`http://edamontology.org/<id>`).
    #[must_use]
    pub fn uri(self) -> String {
        format!("http://edamontology.org/{}", self.id)
    }
}

/// EDAM bioinformatics and cheminformatics ontology annotations for a web tool.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EdamAnnotations {
    /// Scientific domain topics (`topic_*`).
    pub topics: &'static [EdamTerm],
    /// Computational operations performed (`operation_*`).
    pub operations: &'static [EdamTerm],
    /// Input data types (`data_*`).
    pub input_data: &'static [EdamTerm],
    /// Input file/string formats (`format_*`).
    pub input_formats: &'static [EdamTerm],
    /// Output data types (`data_*`).
    pub output_data: &'static [EdamTerm],
    /// Output file/string formats (`format_*`).
    pub output_formats: &'static [EdamTerm],
}

/// Metadata an app exposes as its read-only `WebMCP` `capabilities` tool
/// and `BioSchemas` / Schema.org `SoftwareApplication` JSON-LD.
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
    /// EDAM ontology terms describing topics, operations, data, and formats.
    #[props(default)]
    pub edam: EdamAnnotations,
}

/// Writes `s` as a JSON string literal (also valid as a JS string literal) into `out`.
fn write_json_string(out: &mut String, s: &str) {
    out.reserve(s.len() + 2);
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
}

/// Returns `s` as a JSON string literal (also valid as a JS string literal).
fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    write_json_string(&mut out, s);
    out
}

fn json_string_list(items: &[&str]) -> String {
    let mut out = String::new();
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        write_json_string(&mut out, item);
    }
    out
}

fn write_json_edam_uri(out: &mut String, term: EdamTerm) {
    out.push_str("\"http://edamontology.org/");
    for c in term.id.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c => out.push(c),
        }
    }
    out.push('"');
}

fn json_edam_terms(terms: &[EdamTerm]) -> String {
    let mut out = String::with_capacity(terms.len() * 80);
    for (i, t) in terms.iter().copied().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(r#"{"id": "#);
        write_json_string(&mut out, t.id);
        out.push_str(r#", "label": "#);
        write_json_string(&mut out, t.label);
        out.push_str(r#", "uri": "#);
        write_json_edam_uri(&mut out, t);
        out.push('}');
    }
    out
}

fn json_edam_uris(terms: &[EdamTerm]) -> String {
    let mut out = String::with_capacity(terms.len() * 40);
    for (i, t) in terms.iter().copied().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        write_json_edam_uri(&mut out, t);
    }
    out
}

/// Returns the inline `<script>` body that registers one read-only tool on
/// the page's `WebMCP` document context (guarded IIFE).
///
/// Crate-private: `DocumentHead` builds this from the
/// `webmcp` prop, and apps pass a [`WebMcpConfig`] rather than a script.
#[must_use]
pub(crate) fn capabilities_script(cfg: WebMcpConfig) -> String {
    let tool_name = format!("{}_capabilities", cfg.app_id);
    let name = json_string(&tool_name);
    let title = json_string(cfg.title);
    let description = json_string(cfg.description);
    let inputs = json_string_list(cfg.inputs);
    let outputs = json_string_list(cfg.outputs);
    let topics = json_edam_terms(cfg.edam.topics);
    let operations = json_edam_terms(cfg.edam.operations);
    let input_data = json_edam_terms(cfg.edam.input_data);
    let input_formats = json_edam_terms(cfg.edam.input_formats);
    let output_data = json_edam_terms(cfg.edam.output_data);
    let output_formats = json_edam_terms(cfg.edam.output_formats);

    format!(
        r#"(function() {{
  const mc = document.modelContext || (navigator && navigator.modelContext);
  if (!mc || typeof mc.registerTool !== "function") return;
  mc.registerTool({{
    name: {name},
    title: {title},
    description: {description},
    inputSchema: {{ "type": "object", "properties": {{}}, "additionalProperties": false }},
    execute: async () => ({{ name: {title}, description: {description}, inputs: [{inputs}], outputs: [{outputs}], edam: {{ topics: [{topics}], operations: [{operations}], inputs: {{ data: [{input_data}], formats: [{input_formats}] }}, outputs: {{ data: [{output_data}], formats: [{output_formats}] }} }} }}),
    annotations: {{ readOnlyHint: true }}
  }}).catch(() => {{}});
}})();"#
    )
}

/// Returns a Schema.org / `BioSchemas` `SoftwareApplication` JSON-LD document
/// enriched with EDAM ontology URIs and terms.
#[must_use]
pub(crate) fn bioschemas_json_ld(cfg: WebMcpConfig) -> String {
    let id = json_string(cfg.app_id);
    let name = json_string(cfg.title);
    let description = json_string(cfg.description);
    let topic_uris = json_edam_uris(cfg.edam.topics);
    let operation_uris = json_edam_uris(cfg.edam.operations);
    let topics = json_edam_terms(cfg.edam.topics);
    let operations = json_edam_terms(cfg.edam.operations);
    let input_data = json_edam_terms(cfg.edam.input_data);
    let input_formats = json_edam_terms(cfg.edam.input_formats);
    let output_data = json_edam_terms(cfg.edam.output_data);
    let output_formats = json_edam_terms(cfg.edam.output_formats);

    format!(
        r#"{{"@context":"https://schema.org","@type":"SoftwareApplication","identifier":{id},"name":{name},"description":{description},"applicationCategory":"ScientificApplication","operatingSystem":"WebAssembly","applicationSubCategory":[{topic_uris}],"featureList":[{operation_uris}],"edam":{{"topics":[{topics}],"operations":[{operations}],"inputData":[{input_data}],"inputFormats":[{input_formats}],"outputData":[{output_data}],"outputFormats":[{output_formats}]}}}}"#
    )
}

#[cfg(test)]
mod tests;
