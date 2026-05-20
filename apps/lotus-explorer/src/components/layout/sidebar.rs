// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Sidebar: mobile-filter toggle button, search panel, and branding logo.
//!
//! Reads mobile-filter state from selectors and invokes explore interactions via
//! context — zero props required.

use crate::components::search_panel::SearchPanel;
use crate::features::explore::interactions::use_explore_interactions;
use crate::features::explore::selectors::use_ui_selector;
use crate::hooks::use_locale;
use crate::i18n::{TextKey, t};
use crate::state::use_results_context;
use crate::ui::a11y_contract::{SEARCH_PANEL_BODY_ID, SEARCH_PANEL_HEADING_ID};
use dioxus::prelude::*;

/// Sidebar: filter toggle + `SearchPanel` + logo.
///
/// All concerns (mobile state, locale, search actions) are read from context.
#[component]
pub fn Sidebar() -> Element {
    let locale = use_locale();
    let explore = use_results_context().explore;
    let interactions = use_explore_interactions();
    let mobile_filters_open = *use_ui_selector(explore, |ui| ui.mobile_filters_open).read();

    rsx! {
        aside {
            class: if mobile_filters_open { "sidebar mobile-open" } else { "sidebar mobile-closed" },
            aria_labelledby: SEARCH_PANEL_HEADING_ID,
            button {
                class: "filters-toggle",
                r#type: "button",
                aria_controls: SEARCH_PANEL_BODY_ID,
                aria_expanded: if mobile_filters_open { "true" } else { "false" },
                aria_pressed: if mobile_filters_open { "true" } else { "false" },
                onclick: move |_| interactions.toggle_mobile_filters(),
                if mobile_filters_open {
                    "{t(locale, TextKey::FiltersHide)}"
                } else {
                    "{t(locale, TextKey::FiltersShow)}"
                }
            }
            SearchPanel {}
            div { class: "sidebar-logo-wrap",
                div {
                    class: "sidebar-logo-link",
                    img {
                        class: "sidebar-logo",
                        src: "assets/lotus_ferris.svg",
                        alt: "",
                        "aria-hidden": "true",
                    }
                }
            }
        }
    }
}
