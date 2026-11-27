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
    /// Short EDAM identifier (e.g. `"topic_2258"`, `"operation_3432"`,
    /// `"data_2301"`, `"format_1196"`).
    pub id: &'static str,
    /// Human-readable preferred label (e.g. `"Cheminformatics"`).
    pub label: &'static str,
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
fn write_json_escaped_body(out: &mut String, s: &str) {
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '<' => out.push_str("\\u003c"),
            '>' => out.push_str("\\u003e"),
            '&' => out.push_str("\\u0026"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
}

fn write_json_string(out: &mut String, s: &str) {
    out.reserve(s.len() + 2);
    out.push('"');
    write_json_escaped_body(out, s);
    out.push('"');
}

/// Returns `s` as a JSON string literal (also valid as a JS string literal).
pub(crate) fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    write_json_string(&mut out, s);
    out
}

fn write_json_list_into<T: Copy>(
    out: &mut String,
    items: &[T],
    mut write_item: impl FnMut(&mut String, T),
) {
    for (i, &item) in items.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        write_item(out, item);
    }
}

fn write_json_edam_uri(out: &mut String, term: EdamTerm) {
    out.push_str("\"http://edamontology.org/");
    write_json_escaped_body(out, term.id);
    out.push('"');
}

fn write_edam_term_prop(out: &mut String, key: &str, terms: &[EdamTerm], quote_key: bool) {
    if quote_key {
        out.push('"');
        out.push_str(key);
        out.push_str(r#"":["#);
    } else {
        out.push_str(key);
        out.push_str(": [");
    }
    write_json_list_into(out, terms, |buf, t| {
        buf.push_str(r#"{"id": "#);
        write_json_string(buf, t.id);
        buf.push_str(r#", "label": "#);
        write_json_string(buf, t.label);
        buf.push_str(r#", "uri": "#);
        write_json_edam_uri(buf, t);
        buf.push('}');
    });
    out.push(']');
}

/// Returns the inline `<script>` body that registers one read-only tool on
/// the page's `WebMCP` document context (guarded IIFE).
///
/// Crate-private: `DocumentHead` builds this from the
/// `webmcp` prop, and apps pass a [`WebMcpConfig`] rather than a script.
#[must_use]
pub(crate) fn capabilities_script(cfg: WebMcpConfig) -> String {
    let mut out = String::with_capacity(1024);
    out.push_str(
        "(function() {\n  const mc = document.modelContext || (navigator && navigator.modelContext);\n  if (!mc || typeof mc.registerTool !== \"function\") return;\n  mc.registerTool({\n    name: \"",
    );
    write_json_escaped_body(&mut out, cfg.app_id);
    out.push_str("_capabilities\",\n    title: ");
    write_json_string(&mut out, cfg.title);
    out.push_str(",\n    description: ");
    write_json_string(&mut out, cfg.description);
    out.push_str(
        ",\n    inputSchema: { \"type\": \"object\", \"properties\": {}, \"additionalProperties\": false },\n    execute: async () => ({ name: ",
    );
    write_json_string(&mut out, cfg.title);
    out.push_str(", description: ");
    write_json_string(&mut out, cfg.description);
    out.push_str(", inputs: [");
    write_json_list_into(&mut out, cfg.inputs, write_json_string);
    out.push_str("], outputs: [");
    write_json_list_into(&mut out, cfg.outputs, write_json_string);
    out.push_str("], edam: { ");
    write_edam_term_prop(&mut out, "topics", cfg.edam.topics, false);
    out.push_str(", ");
    write_edam_term_prop(&mut out, "operations", cfg.edam.operations, false);
    out.push_str(", inputs: { ");
    write_edam_term_prop(&mut out, "data", cfg.edam.input_data, false);
    out.push_str(", ");
    write_edam_term_prop(&mut out, "formats", cfg.edam.input_formats, false);
    out.push_str(" }, outputs: { ");
    write_edam_term_prop(&mut out, "data", cfg.edam.output_data, false);
    out.push_str(", ");
    write_edam_term_prop(&mut out, "formats", cfg.edam.output_formats, false);
    out.push_str(" } } }),\n    annotations: { readOnlyHint: true }\n  }).catch(() => {});\n})();");
    out
}

/// Returns a Schema.org / `BioSchemas` `SoftwareApplication` JSON-LD document
/// enriched with EDAM ontology URIs and terms.
#[must_use]
pub(crate) fn bioschemas_json_ld(cfg: WebMcpConfig) -> String {
    let mut out = String::with_capacity(1024);
    out.push_str(r#"{"@context":"https://schema.org","@type":"SoftwareApplication","identifier":"#);
    write_json_string(&mut out, cfg.app_id);
    out.push_str(r#","name":"#);
    write_json_string(&mut out, cfg.title);
    out.push_str(r#","description":"#);
    write_json_string(&mut out, cfg.description);
    out.push_str(r#","applicationCategory":"ScientificApplication","operatingSystem":"WebAssembly","applicationSubCategory":["#);
    write_json_list_into(&mut out, cfg.edam.topics, write_json_edam_uri);
    out.push_str(r#"],"featureList":["#);
    write_json_list_into(&mut out, cfg.edam.operations, write_json_edam_uri);
    out.push_str(r#"],"edam":{"#);
    write_edam_term_prop(&mut out, "topics", cfg.edam.topics, true);
    out.push(',');
    write_edam_term_prop(&mut out, "operations", cfg.edam.operations, true);
    out.push(',');
    write_edam_term_prop(&mut out, "inputData", cfg.edam.input_data, true);
    out.push(',');
    write_edam_term_prop(&mut out, "inputFormats", cfg.edam.input_formats, true);
    out.push(',');
    write_edam_term_prop(&mut out, "outputData", cfg.edam.output_data, true);
    out.push(',');
    write_edam_term_prop(&mut out, "outputFormats", cfg.edam.output_formats, true);
    out.push_str("}}");
    out
}

#[cfg(test)]
mod tests;
