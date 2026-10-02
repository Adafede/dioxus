// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the index project

//! Accessibility components for the landing page.

use dioxus::prelude::*;
use ui::prelude::*;

/// Skip link pointing at `#main-content`.
///
/// The inline style parks the link at `top: -100%`; the rule that brings it
/// back on focus comes from `ui::document::DocumentHead`, which every app
/// rendering a `DocumentHead` gets. This component used to inline its own copy
/// of nothing at all and stayed invisible, because the rule was only ever
/// written into `json-count-rs`'s rsx.
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
