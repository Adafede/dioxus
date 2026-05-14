// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Sidebar: mobile-filter toggle button, search panel, and branding logo.
//!
//! Reads mobile-filter state from `ResultsContext.explore` and dispatches
//! `MobileFiltersToggled` directly — the only prop is `on_search`.

use crate::components::search_panel::SearchPanel;
use crate::features::explore::actions::ExploreAction;
use crate::features::explore::search_state::dispatch_explore_action;
use crate::hooks::use_locale;
use crate::i18n::{TextKey, t};
use crate::state::use_results_context;
use crate::ui::a11y_contract::{SEARCH_PANEL_BODY_ID, SEARCH_PANEL_HEADING_ID};
use dioxus::prelude::*;

/// Sidebar: filter toggle + `SearchPanel` + logo.
///
/// The only prop is `on_search` because the search action captures `criteria`,
/// `explore`, and `repo` from `App` scope and cannot come from context.
/// All other concerns (mobile state, locale) are read from context.
#[component]
pub fn Sidebar(on_search: EventHandler<()>) -> Element {
    let locale = use_locale();
    let explore = use_results_context().explore;
    let mobile_filters_open = explore.read().ui.mobile_filters_open;

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
                onclick: move |_| dispatch_explore_action(explore, ExploreAction::MobileFiltersToggled),
                if mobile_filters_open {
                    "{t(locale, TextKey::FiltersHide)}"
                } else {
                    "{t(locale, TextKey::FiltersShow)}"
                }
            }
            SearchPanel { on_search }
            div { class: "sidebar-logo-wrap",
                img {
                    class: "sidebar-logo",
                    src: "assets/lotus_ferris.svg",
                    alt: "{t(locale, TextKey::PageTitle)}",
                }
            }
        }
    }
}
