// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::i18n::{TextKey, t};
use crate::state::use_results_context;
use dioxus::prelude::*;

#[component]
pub fn QueryPanel() -> Element {
    let locale = crate::hooks::use_locale();
    let explore = use_results_context().explore;
    let sparql_query =
        crate::features::explore::selectors::use_result_selector(explore, |result| {
            result.sparql_query.clone()
        });

    let mut prev_query = use_signal(|| None::<String>);

    // Close the query panel when the SPARQL query itself changes.
    use_effect(move || {
        let current_query = sparql_query
            .read()
            .as_ref()
            .map(|query| query.as_ref().to_string());

        if current_query != *prev_query.read() {
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

            prev_query.set(current_query);
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
