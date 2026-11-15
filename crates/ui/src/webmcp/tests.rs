// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

// The `tests` tests, extracted from `webmcp.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `webmcp` module, so
// `use super::*` below reaches exactly what it did before the move.
use super::*;

#[test]
fn script_is_guarded_iife_with_readonly_tool() {
    let cfg = WebMcpConfig {
        app_id: "test_app",
        title: "Test \"Quote\" App",
        description: "desc",
        inputs: &["in1", "in2"],
        outputs: &["out1"],
        edam: EdamAnnotations {
            topics: &[EdamTerm {
                id: "topic_2258",
                label: "Cheminformatics",
            }],
            operations: &[EdamTerm {
                id: "operation_0366",
                label: "Structure clustering",
            }],
            input_data: &[EdamTerm {
                id: "data_2301",
                label: "SMILES string",
            }],
            input_formats: &[EdamTerm {
                id: "format_1196",
                label: "SMILES",
            }],
            output_data: &[EdamTerm {
                id: "data_2301",
                label: "SMILES string",
            }],
            output_formats: &[EdamTerm {
                id: "format_1196",
                label: "SMILES",
            }],
        },
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
    assert!(script.contains("http://edamontology.org/topic_2258"));
    assert!(script.contains("http://edamontology.org/operation_0366"));
    assert!(script.contains("http://edamontology.org/format_1196"));
    // JSON string escaping of a double quote inside the title.
    assert!(
        script.contains("\\\"Quote\\\""),
        "quote not escaped: {script}"
    );

    let ld = bioschemas_json_ld(cfg);
    assert!(ld.contains("\"@type\":\"SoftwareApplication\""), "ld: {ld}");
    assert!(
        ld.contains("http://edamontology.org/topic_2258"),
        "ld topic: {ld}"
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
        edam: EdamAnnotations::default(),
    };
    let script = capabilities_script(cfg);
    assert!(script.contains("inputs: []"), "empty inputs: {script}");
    assert!(script.contains("outputs: [\"links\"]"), "outputs: {script}");
}

#[test]
fn script_and_json_ld_escape_script_breakout_and_line_separators() {
    const PAYLOAD: &str = "</script><script>alert(1)&\"\\\u{2028}\u{2029}";
    let cfg = WebMcpConfig {
        app_id: "xss_app",
        title: PAYLOAD,
        description: PAYLOAD,
        inputs: &[PAYLOAD],
        outputs: &[PAYLOAD],
        edam: EdamAnnotations {
            topics: &[EdamTerm {
                id: "topic_1</script>&\"\\",
                label: PAYLOAD,
            }],
            operations: &[],
            input_data: &[],
            input_formats: &[],
            output_data: &[],
            output_formats: &[],
        },
    };

    let script = capabilities_script(cfg);
    assert!(
        !script.contains("</script>"),
        "script must escape closing tag: {script}"
    );
    assert!(
        !script.contains('\u{2028}') && !script.contains('\u{2029}'),
        "script must escape JS line/paragraph separators: {script}"
    );
    assert!(
        script.contains(r"\u003c/script\u003e"),
        "script should unicode-escape angle brackets: {script}"
    );
    assert!(
        script.contains(r"topic_1\u003c/script\u003e\u0026"),
        "EDAM term URI must escape angle brackets and ampersands: {script}"
    );

    let ld = bioschemas_json_ld(cfg);
    assert!(
        !ld.contains("</script>"),
        "JSON-LD must escape closing script tags: {ld}"
    );
    assert!(
        !ld.contains('\u{2028}') && !ld.contains('\u{2029}'),
        "JSON-LD must escape line/paragraph separators: {ld}"
    );
}
