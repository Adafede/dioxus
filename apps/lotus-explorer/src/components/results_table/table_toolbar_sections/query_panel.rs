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

    let mut prev_criteria = use_signal(|| criteria.read().clone());
    let mut prev_query = use_signal(|| toolbar_snapshot.read().sparql_query.clone());
    let mut panel_visible = use_signal(|| toolbar_snapshot.read().sparql_query.is_some());

    // Parameter changes should remove the tab until a new query is generated.
    use_effect(move || {
        let current_criteria = criteria.read();
        if *current_criteria != *prev_criteria.read() {
            panel_visible.set(false);
            prev_criteria.set(current_criteria.clone());
        }
    });

    // Show the tab again when a new query value arrives for current parameters.
    use_effect(move || {
        let current_query = toolbar_snapshot.read().sparql_query.clone();
        if current_query != *prev_query.read() {
            panel_visible.set(current_query.is_some());
            prev_query.set(current_query);
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
