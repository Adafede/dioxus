// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Common UI utilities shared across all apps.

use dioxus::prelude::*;

/// Parks a skip link off-screen at `top: -100%`.
///
/// Returning it to view on focus needs a matching
/// `.skip-link:focus { top: 0 !important; ... }` rule, which a consuming app
/// must supply; this constant cannot do it on its own.
pub const SKIP_LINK_STYLE: &str = "position:absolute;top:-100%;left:0.5rem;z-index:9999;padding:0.5rem 1rem;background:transparent;color:#0b5cab;font-size:0.875rem;font-weight:600;border-radius:0 0 4px 4px;text-decoration:underline;";

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
