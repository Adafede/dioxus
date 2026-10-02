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

/// Skip navigation link to the app's main landmark.
///
/// The target is fixed, and every app in this workspace gives its `<main>`
/// element `id="main-content"` to match. `every_app_pairs_its_skip_link_with_a_
/// main_landmark` in `crates/upload/tests/gate_consistency.rs` is what keeps
/// that true; it is not checkable from here, because the `main` element is in
/// a different crate.
///
/// This took a `target` prop while two ids were in use — `#main` in
/// `lipid-selecto-rs` and `mgf-precursor-erro-rs`, `#main-content` in the
/// other four, which had produced a second copy of this component differing in
/// one string. Standardising on one id left the prop with no callers, and a
/// `pub` prop with no callers in a `publish = false` crate is the same
/// unreachable surface as a `pub` fn with no callers.
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
mod tests;
