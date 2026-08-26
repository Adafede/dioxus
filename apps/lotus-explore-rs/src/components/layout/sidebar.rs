// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Sidebar: mobile-filter toggle button, search panel, and branding logo.

use crate::components::search_panel::SearchPanel;
use crate::features::explore::interactions::use_explore_interactions;
use crate::features::explore::selectors::use_ui_selector;
use crate::hooks::use_locale;
use crate::i18n::{TextKey, t};
use crate::state::use_results_context;
use crate::ui::a11y_contract::{SEARCH_PANEL_BODY_ID, SEARCH_PANEL_HEADING_ID};
use dioxus::prelude::*;

const LOTUS_LOGO_SVG: &str = include_str!("../../../public/favicon.svg");

#[component]
pub fn Sidebar() -> Element {
    let locale = use_locale();
    let explore = use_results_context().explore;
    let interactions = use_explore_interactions();
    let mobile_filters_open = *use_ui_selector(explore, |ui| ui.mobile_filters_open).read();

    rsx! {
        aside {
            class: if mobile_filters_open {
                "sidebar flex flex-col mobile-open"
            } else {
                "sidebar flex flex-col mobile-closed"
            },
            aria_labelledby: SEARCH_PANEL_HEADING_ID,
            div {
                class: "sidebar-logo flex justify-center",
                "aria-hidden": "true",
                dangerous_inner_html: LOTUS_LOGO_SVG,
            }
            div {
                class: "flex justify-center px-3 pb-2",
                button {
                    r#type: "button",
                    class: "filters-toggle inline-flex w-full min-h-10 cursor-pointer items-center justify-center rounded-lotus-sm border border-border bg-primary px-3.5 py-2 text-ui font-semibold text-white shadow-xs transition-[background] duration-150 hover:bg-primary-hover focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/40",
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
            }
            SearchPanel {}
        }
    }
}

#[component]
pub fn LazySidebar() -> Element {
    let mut ready = use_signal(|| false);
    use_effect(move || {
        ready.set(true);
    });
    rsx! {
        if *ready.read() {
            Sidebar {}
        } else {
            aside {
                class: "sidebar mobile-closed flex h-full flex-col",
            }
        }
    }
}
