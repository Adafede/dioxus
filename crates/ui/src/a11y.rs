// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Accessibility primitives shared across every workspace application.

use dioxus::prelude::*;

/// Inline style parking a skip link off-screen at `top: -100%` while keeping it in tab order.
pub(crate) const SKIP_LINK_STYLE: &str = "position:absolute;top:-100%;left:0.5rem;z-index:9999;padding:0.5rem 1rem;background:transparent;color:#0b5cab;font-size:0.875rem;font-weight:600;border-radius:0 0 4px 4px;text-decoration:underline;";

/// Focus CSS rule injected by [`DocumentHead`](crate::document::DocumentHead) to reveal [`skip_link`].
pub(crate) const SKIP_LINK_FOCUS_CSS: &str = ".skip-link:focus { top: 0 !important; left: 0.5rem !important; outline: 3px solid #0b5cab; outline-offset: 2px; }";

/// Skip navigation link targeting each workspace app's `#main-content` landmark.
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
