// The `tests` tests, extracted from `common.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `common` module, so
// `use super::*` below reaches exactly what it did before the move.
use super::{SKIP_LINK_FOCUS_CSS, SKIP_LINK_STYLE};

/// These two constants only work together. If the inline style's `top` or
/// the focus rule's override of it is renamed or edited, both still compile,
/// still parse as CSS, and the skip link is invisible again. Nothing else in
/// the workspace tests this, because the link only misbehaves in a browser.
#[test]
fn the_focus_rule_overrides_the_offset_the_inline_style_parks_it_with() {
    assert!(
        SKIP_LINK_STYLE.contains("top:-100%"),
        "SKIP_LINK_STYLE no longer parks the link off-screen; the focus rule \
         was written to undo exactly that"
    );
    assert!(
        SKIP_LINK_FOCUS_CSS.contains("top: 0 !important"),
        "SKIP_LINK_FOCUS_CSS must beat the inline top, which needs !important \
         because an inline declaration outranks a plain selector"
    );
}

/// The inline style has to leave the link in the tab order, or focusing it
/// is impossible and the focus rule above has nothing to act on.
#[test]
fn the_inline_style_keeps_the_link_in_the_tab_order() {
    for hiding in ["display:none", "display: none", "visibility:hidden"] {
        assert!(
            !SKIP_LINK_STYLE.contains(hiding),
            "SKIP_LINK_STYLE contains `{hiding}`, which removes the link from \
             the tab order so it can never be focused"
        );
    }
}
