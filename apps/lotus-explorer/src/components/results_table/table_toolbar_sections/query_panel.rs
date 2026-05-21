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
