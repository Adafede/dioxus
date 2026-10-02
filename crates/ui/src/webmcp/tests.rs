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
