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
            style: SKIP_LINK_STYLE,
            "Skip to main content"
        }
    }
}
