// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the index project

//! Accessibility components for the landing page.

use dioxus::prelude::*;
use ui::prelude::*;

/// Skip link pointing at `#main-content`.
///
/// The inline style parks the link at `top: -100%`. Bringing it back on focus
/// relies on a `.skip-link:focus { top: 0 !important; ... }` rule, which this
/// crate does not currently emit — see `apps/json-count-rs/src/main.rs` for the
/// only copy in the workspace that does.
#[component]
pub(crate) fn SkipLink() -> Element {
    rsx! {
        a {
            href: "#main-content",
            class: "skip-link",
            style: StyleBuilder::new().property("position", "absolute").property("top", "-100%").property("left", "0.5rem").property("z-index", "9999").padding("0.5rem 1rem").property("background", "transparent").color("#0b5cab").font_size("0.875rem").font_weight("600").border_radius("0 0 4px 4px").text_decoration("underline").build(),
            "Skip to main content"
        }
    }
}
