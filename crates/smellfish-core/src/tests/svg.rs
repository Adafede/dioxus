// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

use crate::svg::{is_safe_svg, sanitize_svg};

#[test]
fn accepts_plain_and_xml_prefixed_svg_and_rejects_active_payloads() {
    assert!(
        is_safe_svg(
            r#"<svg xmlns="http://www.w3.org/2000/svg"><path d="M0 0" opacity="0.5"/></svg>"#
        ),
        "plain SVG markup must pass"
    );
    assert!(
        is_safe_svg(
            r#"<?xml version="1.0" encoding="UTF-8"?><svg xmlns="http://www.w3.org/2000/svg"></svg>"#
        ),
        "XML declaration before <svg> must pass"
    );
    assert!(
        !is_safe_svg(r"<svg><script>alert(1)</script></svg>"),
        "script blocked"
    );
    assert!(
        !is_safe_svg(r#"<svg ONLOAD="alert(1)"></svg>"#),
        "onload blocked"
    );
    assert!(
        !is_safe_svg(r#"<svg onload = "alert(1)"></svg>"#),
        "spaced onload blocked"
    );
    assert!(
        !is_safe_svg(r#"<svg><a href="javascript:alert(1)">x</a></svg>"#),
        "javascript: blocked"
    );
    assert!(
        !is_safe_svg(r#"<svg><a href="data:text/html,hi">x</a></svg>"#),
        "data:text/html blocked"
    );
    assert!(
        !is_safe_svg(r"<svg><foreignObject><body/></foreignObject></svg>"),
        "foreignObject blocked"
    );
    assert!(!is_safe_svg(r"<svg><path d='M0 0'"), "unclosed tag blocked");
    assert!(!is_safe_svg(r"<svgfoo></svg>"), "svg tag boundary enforced");
    assert!(
        !is_safe_svg(r#"<svg><animate onbegin="alert(1)"/></svg>"#),
        "animate blocked"
    );
    assert!(
        !is_safe_svg("<svg><a href=\"java\nscript:alert(1)\">x</a></svg>"),
        "control-char obfuscated script blocked"
    );

    assert!(
        sanitize_svg(Some("<svg></svg>".to_string())).is_some(),
        "safe SVG kept"
    );
    assert!(
        sanitize_svg(Some("<svg><script/></svg>".to_string())).is_none(),
        "unsafe SVG dropped"
    );
}
