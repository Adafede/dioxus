// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::i18n::{TextKey, t};
use crate::state::use_results_context;
use crate::state::use_form_criteria_context;
use dioxus::prelude::*;

#[component]
pub fn QueryPanel() -> Element {
    let locale = crate::hooks::use_locale();
    let explore = use_results_context().explore;
    let sparql_query =
        crate::features::explore::selectors::use_result_selector(explore, |result| {
            result.sparql_query.clone()
        });

    // Track search criteria to auto-close the panel when parameters change
    let form_ctx = use_form_criteria_context();
    let criteria = crate::features::explore::selectors::use_criteria_selector(
        form_ctx.criteria,
        |c| c.clone(),
    );

    // Close the query panel when search parameters change (new search incoming)
    use_effect(move || {
        // Reading criteria triggers dependency tracking - effect re-runs when criteria changes
        let _ = criteria.read();

        // This effect re-runs whenever criteria changes
        #[cfg(target_arch = "wasm32")]
        {
            if let Ok(window) = web_sys::window().ok_or(()) {
                if let Ok(document) = window.document().ok_or(()) {
                    if let Some(details) =
                        document.query_selector(".query-panel").ok().flatten()
                    {
                        let _ = details.remove_attribute("open");
                    }
                }
            }
        }
    });

    rsx! {
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
