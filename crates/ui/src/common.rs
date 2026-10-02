// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Common UI utilities shared across all apps.

use dioxus::prelude::*;

/// Parks a skip link off-screen at `top: -100%`.
///
/// Off-screen rather than `display: none` on purpose: a hidden or display-less
/// element is removed from the tab order, so the link could not be focused and
/// the rule that returns it could not apply.
///
/// The rule that returns it on focus is private to this crate and is injected
/// by [`DocumentHead`](crate::document::DocumentHead), so an app rendering a skip link
/// needs nothing else.
pub const SKIP_LINK_STYLE: &str = "position:absolute;top:-100%;left:0.5rem;z-index:9999;padding:0.5rem 1rem;background:transparent;color:#0b5cab;font-size:0.875rem;font-weight:600;border-radius:0 0 4px 4px;text-decoration:underline;";

/// Brings a link styled by [`SKIP_LINK_STYLE`] back on screen when focused.
///
/// `top` and `left` are overridden with `!important` because the inline style
/// they undo is itself inline, and an inline declaration beats any selector
/// that is not also `!important`. Without that, this rule parses and applies
/// and changes nothing.
///
/// Injected by [`DocumentHead`] rather than by each app. It used to be inlined
/// in `json-count-rs`'s rsx and nowhere else, which left the skip links in
/// `index` and `mgf-precursor-erro-rs` permanently off-screen: focusable and
/// announced by a screen reader, but never once visible. One app in three had
/// remembered the rule.
///
/// Crate-private on purpose: an app that reaches for this is re-introducing the
/// per-app copy that caused the bug.
pub(crate) const SKIP_LINK_FOCUS_CSS: &str = ".skip-link:focus { top: 0 !important; left: 0.5rem !important; outline: 3px solid #0b5cab; outline-offset: 2px; }";

/// Skip navigation link pointing at `#main-content`.
///
/// The `href` is fixed. `lipid-selecto-rs` and `mgf-precursor-erro-rs` give
/// their main landmark `id="main"` instead, and each hand-rolls its own skip
/// link rather than using this one.
#[component]
pub fn skip_link() -> Element {
    rsx! {
        a {
            href: "#main-content",
            class: "skip-link",
            style: SKIP_LINK_STYLE,
            "Skip to main content"
        }
    }
}

#[cfg(test)]
mod tests {
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
}
