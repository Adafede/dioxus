//! WebMCP tool registration for the W3C WebMCP proposal.
//!
//! Each app exposes a single read-only `<app_id>_capabilities` tool on its
//! page's WebMCP document context (`document.modelContext`), so AI agents can
//! discover what an app consumes and produces before invoking it. The emitted
//! script is a guarded no-op in browsers without a WebMCP context, so it is
//! safe to inject unconditionally via [`DocumentHead`](crate::DocumentHead).
//!
//! Spec: <https://webmachinelearning.github.io/webmcp/>

use dioxus::prelude::*;

/// Metadata an app exposes as its read-only WebMCP `capabilities` tool.
#[derive(Clone, Copy, Debug, Props, PartialEq)]
pub struct WebMcpConfig {
    /// Stable kebab-case identifier used to derive the tool name
    /// (`"<app_id>_capabilities"`).
    pub app_id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    /// Human-readable input surface (e.g. `"smiles_list"`).
    pub inputs: &'static [&'static str],
    /// Human-readable output surface (e.g. `"cx_smiles"`).
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
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Returns the inline `<script>` body that registers one read-only tool on the
/// page's WebMCP document context. The body is a guarded IIFE: it no-ops when
/// `document.modelContext`/`navigator.modelContext` is absent or does not
/// expose `registerTool`, and swallows registration rejections.
pub fn capabilities_script(cfg: WebMcpConfig) -> String {
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
mod tests {
    use super::*;

    #[test]
    fn script_is_guarded_iife_with_readonly_tool() {
        let cfg = WebMcpConfig {
            app_id: "test_app",
            title: "Test \"Quote\" App",
            description: "desc",
            inputs: &["in1", "in2"],
            outputs: &["out1"],
        };
        let script = capabilities_script(cfg);

        assert!(
            script.starts_with("(function() {"),
            "missing IIFE guard: {script}"
        );
        assert!(script.ends_with("})();"), "unterminated IIFE: {script}");
        assert!(script.contains("document.modelContext || (navigator && navigator.modelContext)"));
        assert!(script.contains("typeof mc.registerTool !== \"function\""));
        assert!(script.contains(".catch(() => {});"));
        assert!(script.contains("\"test_app_capabilities\""));
        assert!(script.contains("readOnlyHint: true"));
        assert!(script.contains("inputSchema: { \"type\": \"object\""));
        // JSON string escaping of a double quote inside the title.
        assert!(
            script.contains("\\\"Quote\\\""),
            "quote not escaped: {script}"
        );
    }

    #[test]
    fn empty_io_arrays_render_as_empty_js_arrays() {
        let cfg = WebMcpConfig {
            app_id: "landing",
            title: "Landing",
            description: "x",
            inputs: &[],
            outputs: &["links"],
        };
        let script = capabilities_script(cfg);
        assert!(script.contains("inputs: []"), "empty inputs: {script}");
        assert!(script.contains("outputs: [\"links\"]"), "outputs: {script}");
    }
}
