// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Sidebar: mobile-filter toggle button, search panel, and branding logo.

use crate::components::search_panel::SearchPanel;
use crate::components::ui::{Button, ButtonSize, ButtonVariant};
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
                "sidebar w-[360px] min-w-[250px] min-h-[400px] overflow-hidden rounded-2xl bg-panel shadow-xs flex flex-col mobile-open flex-shrink-0 min-[1440px]:mr-2 pl-4 sm:pl-6 lg:pl-8"
            } else {
                "sidebar w-[360px] min-w-[250px] min-h-[400px] overflow-hidden rounded-2xl bg-panel shadow-xs flex flex-col mobile-closed flex-shrink-0 min-[1440px]:mr-2 pl-4 sm:pl-6 lg:pl-8"
            },
            aria_labelledby: SEARCH_PANEL_HEADING_ID,
            div {
                class: "w-full flex justify-center pr-4 pt-4 pb-2",
                "aria-hidden": "true",
                div {
                    class: "w-20 min-[1440px]:w-24 aspect-[1115/981]",
                    dangerous_inner_html: LOTUS_LOGO_SVG,
                }
            }
            div {
                class: "flex justify-center pr-4 pb-4",
                Button {
                    r#type: "button",
                    variant: ButtonVariant::Primary,
                    size: ButtonSize::Md,
                    class: Some("filters-toggle".to_string()),
                    aria_controls: SEARCH_PANEL_BODY_ID,
                    aria_expanded: if mobile_filters_open { "true" } else { "false" },
                    aria_pressed: if mobile_filters_open { "true" } else { "false" },
                    onclick: move |_| interactions.toggle_mobile_filters(),
                    label: if mobile_filters_open {
                        t(locale, TextKey::FiltersHide).to_string()
                    } else {
                        t(locale, TextKey::FiltersShow).to_string()
                    },
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
            class: "sidebar w-[360px] min-w-[250px] min-h-[400px] overflow-hidden rounded-2xl bg-panel shadow-xs flex flex-col mobile-closed flex-shrink-0 pl-4 sm:pl-6 lg:pl-8",
            }
        }
    }
}
