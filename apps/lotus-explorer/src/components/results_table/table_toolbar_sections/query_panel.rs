// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::components::copy_button::CopyButton;
use crate::features::explore::use_toolbar_result_snapshot;
use crate::i18n::{TextKey, t};
use crate::state::use_form_criteria_context;
use crate::state::use_results_context;
use dioxus::prelude::*;

#[component]
pub fn QueryPanel() -> Element {
    let locale = crate::hooks::use_locale();
    let explore = use_results_context().explore;
    let form_ctx = use_form_criteria_context();
    let toolbar_snapshot = use_toolbar_result_snapshot(explore);
    let criteria =
        crate::features::explore::selectors::use_criteria_selector(form_ctx.criteria, |c| {
            c.clone()
        });

    let mut criteria_effect_ready = use_signal(|| false);
    let mut panel_visible = use_signal(|| toolbar_snapshot.read().sparql_query.is_some());

    // Parameter changes should remove the tab until a new query is generated.
    // peek() for the guard so this effect only subscribes to `criteria`, not to itself.
    use_effect(move || {
        let _ = criteria.read();
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
                details { class: "query-panel",
                    summary { "{t(locale, TextKey::SparqlQuery)}" }
                    div { class: "query-body",
                        CopyButton {
                            text: q.clone(),
                            title: t(locale, TextKey::CopySparqlQuery),
                            locale,
                            class: "btn btn-xs copy-btn query-copy-btn",
                        }
                        pre { class: "query-text", "{q.as_ref()}" }
                    }
                }
            }
        }
    }
}
