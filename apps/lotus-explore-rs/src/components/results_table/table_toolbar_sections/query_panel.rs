// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::components::copy_button::CopyButton;
use crate::features::explore::use_toolbar_result_snapshot;
use crate::i18n::{TextKey, t};
use crate::state::use_form_criteria_context;
use crate::state::use_results_context;
use dioxus::prelude::*;
use ui::prelude::*;

#[component]
pub fn QueryPanel() -> Element {
    let locale = crate::hooks::use_locale();
    let explore = use_results_context().explore;
    let form_ctx = use_form_criteria_context();
    let toolbar_snapshot = use_toolbar_result_snapshot(explore);

    let mut criteria_effect_ready = use_signal(|| false);
    let mut panel_visible = use_signal(|| toolbar_snapshot.read().sparql_query.is_some());

    // Parameter changes should remove the tab until a new query is generated.
    // peek() for the guard so this effect only subscribes to `form_ctx.criteria`, not to itself.
    use_effect(move || {
        let _ = form_ctx.criteria.read();
        if *criteria_effect_ready.peek() {
            panel_visible.set(false);
        } else {
            criteria_effect_ready.set(true);
        }
    });

    // Show the tab again when a new query value arrives for current parameters.
    // peek() for panel_visible so this only subscribes to `toolbar_snapshot`.
    use_effect(move || {
        let current_query = toolbar_snapshot.read();
        if !*panel_visible.peek() {
            panel_visible.set(current_query.sparql_query.is_some());
        }
    });

    rsx! {
        if *panel_visible.read() {
            if let Some(q) = toolbar_snapshot.read().sparql_query.as_ref() {
                details { style: query_panel_style(),
                    summary { style: query_summary_style(), "{t(locale, TextKey::SparqlQuery)}" }
                    div { style: query_body_style(),
                        CopyButton {
                            text: q.clone(),
                            title: t(locale, TextKey::CopySparqlQuery),
                            locale,
                            class: "copy-btn",
                        }
                        pre { style: query_text_style(), "{q.as_ref()}" }
                    }
                }
            }
        }
    }
}

fn query_summary_style() -> String {
    StyleBuilder::new()
        .cursor("pointer")
        .padding("8px 14px")
        .font_size("var(--fs-0)")
        .color("var(--text2)")
        .property("user-select", "none")
        .property("letter-spacing", "0.04em")
        .font_weight("600")
        .property("list-style", "none")
        .build()
}

fn query_panel_style() -> String {
    StyleBuilder::new()
        .background_color("var(--panel-bg-soft)")
        .border("1px solid var(--panel-border)")
        .border_radius("var(--radius)")
        .box_shadow("var(--panel-shadow)")
        .property(
            "transition",
            "background .15s ease, border-color .15s ease, box-shadow .15s ease",
        )
        .build()
}

fn query_body_style() -> String {
    StyleBuilder::new()
        .property("position", "relative")
        .border_radius("0 0 var(--radius) var(--radius)")
        .property("overflow", "hidden")
        .build()
}

fn query_text_style() -> String {
    StyleBuilder::new()
        .padding("12px 16px")
        .property("margin", "0")
        .font_family("var(--mono)")
        .font_size("var(--fs-0)")
        .color("var(--text)")
        .background_color("var(--bg2)")
        .property("border-left", "3px solid var(--wd-entries)")
        .property("white-space", "pre-wrap")
        .property("word-break", "break-word")
        .property("max-height", "320px")
        .property("overflow", "auto")
        .build()
}
