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
use ui::prelude::*;

use crate::ui::style_constants::primary_buttons;

const LOTUS_LOGO_SVG: &str = include_str!("../../../public/favicon.svg");

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
            style: sidebar_style(),
            class: if mobile_filters_open { "sidebar mobile-open" } else { "sidebar mobile-closed" },
            aria_labelledby: SEARCH_PANEL_HEADING_ID,
            div {
                class: "sidebar-logo",
                style: StyleBuilder::new()
                    .display("flex")
                    .justify_content("center")
                    .build(),
                "aria-hidden": "true",
                dangerous_inner_html: LOTUS_LOGO_SVG,
            }
            div {
                style: button_wrapper_style(),
                button {
                    r#type: "button",
                    class: "filters-toggle",
                    style: primary_buttons::button_filters_toggle_style(),
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

/// Splits the sidebar's first render off the boot's critical main-thread path.
///
/// The dioxus 0.7 boot renders the whole shell — logo, filter toggle and the full
/// `SearchPanel` — in a single synchronous pass; that pass is the landing's "long
/// task". dioxus 0.7 predates `LazyComponent` (shipped in 1.0), so we gate the
/// sidebar behind `use_effect` + signal: first paint shows a skeleton `<aside>`,
/// and the real sidebar mounts on the next tick (~1ms after paint). On mobile the
/// sidebar is off-canvas (`mobile-closed`) until toggled, so the defer is invisible
/// there; on desktop it re-mounts within ~1ms of first paint.
#[component]
pub fn LazySidebar() -> Element {
    let mut ready = use_signal(|| false);
    // `use_effect` (dioxus-hooks 0.7) runs once after the first commit and only
    // re-runs on a tracked read change. We only *write* the signal here, so this
    // fires exactly once — no render loop — and schedules the real sidebar for the
    // next tick (i.e. after first paint).
    use_effect(move || {
        ready.set(true);
    });
    rsx! {
        if *ready.read() {
            Sidebar {}
        } else {
            aside {
                class: "sidebar mobile-closed",
                style: StyleBuilder::new()
                    .display("flex")
                    .flex_direction("column")
                    .property("height", "100%")
                    .build(),
            }
        }
    }
}

fn sidebar_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .flex_direction("column")
        .build()
}

fn button_wrapper_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .justify_content("center")
        .build()
}
