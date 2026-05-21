// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::i18n::{TextKey, t};
use crate::state::use_form_criteria_context;
use crate::state::use_results_context;
use dioxus::prelude::*;

#[component]
pub fn QueryPanel() -> Element {
    let locale = crate::hooks::use_locale();
    let explore = use_results_context().explore;
    let form_ctx = use_form_criteria_context();
    let sparql_query =
        crate::features::explore::selectors::use_result_selector(explore, |result| {
            result.sparql_query.clone()
        });
    let criteria =
        crate::features::explore::selectors::use_criteria_selector(form_ctx.criteria, |c| {
            c.clone()
        });

    let mut prev_criteria = use_signal(|| criteria.read().clone());
    let mut prev_query = use_signal(|| sparql_query.read().clone());
    let mut panel_visible = use_signal(|| sparql_query.read().is_some());

    // Parameter changes should remove the tab until a new query is generated.
    use_effect(move || {
        let current_criteria = criteria.read().clone();
        if current_criteria != *prev_criteria.read() {
            panel_visible.set(false);
            prev_criteria.set(current_criteria);
        }
    });

    // Show the tab again when a new query value arrives for current parameters.
    use_effect(move || {
        let current_query = sparql_query.read().clone();
        if current_query != *prev_query.read() {
            panel_visible.set(current_query.is_some());
            prev_query.set(current_query);
        }
    });

    rsx! {
        if *panel_visible.read() {
            if let Some(q) = sparql_query.read().as_ref() {
                details { class: "query-panel",
                    summary { "{t(locale, TextKey::SparqlQuery)}" }
                    div { class: "query-body",
                        pre { class: "query-text", "{q.as_ref()}" }
                        crate::components::copy_button::CopyButton {
                            text: q.clone(),
                            title: t(locale, TextKey::CopySparqlQuery),
                            locale,
                            class: "btn btn-xs copy-btn query-copy-btn",
                        }
                    }
                }
            }
        }
    }
}
