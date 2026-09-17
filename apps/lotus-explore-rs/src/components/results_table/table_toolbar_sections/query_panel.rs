// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::components::copy_button::CopyButton;
use crate::features::explore::use_toolbar_result_snapshot;
use crate::i18n::{TextKey, t};
use crate::state::use_form_criteria_context;
use crate::state::use_results_context;
use crate::ui::classes;
use dioxus::prelude::*;

#[component]
pub fn QueryPanel() -> Element {
    let locale = crate::hooks::use_locale();
    let explore = use_results_context().explore;
    let form_ctx = use_form_criteria_context();
    let toolbar_snapshot = use_toolbar_result_snapshot(explore);

    let mut panel_visible = use_signal(|| toolbar_snapshot.read().sparql_query.is_some());
    let mut panel_open = use_signal(|| false);

    // Only hide panel when criteria actually changes to a new search (not just typing)
    let mut prev_criteria = use_signal(|| form_ctx.criteria.read().clone());
    use_effect(move || {
        let current = form_ctx.criteria.read().clone();
        let previous = prev_criteria.read().clone();
        if current != previous {
            prev_criteria.set(current);
            // Only hide if we had a query before and now it's a new search
            if toolbar_snapshot.read().sparql_query.is_some() {
                panel_visible.set(false);
                panel_open.set(false);
            }
        }
    });

    // Show panel when a new query is available
    use_effect(move || {
        let current_query = toolbar_snapshot.read();
        if !*panel_visible.peek() && current_query.sparql_query.is_some() {
            panel_visible.set(true);
        }
    });

    rsx! {
        if *panel_visible.read() {
            if let Some(q) = toolbar_snapshot.read().sparql_query.as_ref() {
                details {
                    class: "overflow-hidden",
                    open: *panel_open.read(),
                    ontoggle: move |_| {
                        let next = !*panel_open.peek();
                        panel_open.set(next);
                    },
                    summary {
                        class: "flex w-full min-w-0 cursor-pointer select-none items-center gap-2 bg-panel-soft px-3 py-2 text-ui font-semibold text-muted hover:bg-bg {classes::FOCUS_RING_BTN}",
                        span {
                            class: if *panel_open.read() {
                                "inline-block rotate-90 text-subtle {classes::TRANSITION_TRANSFORM}"
                            } else {
                                "inline-block text-subtle {classes::TRANSITION_TRANSFORM}"
                            },
                            "▶"
                        }
                        "{t(locale, TextKey::SparqlQuery)}"
                    }
                    div { class: "flex w-full min-w-0 flex-col gap-2 bg-panel-soft p-3 sm:p-4",
                        // Keep the SPARQL block constrained so it reads like an intentional utility panel.
                        CopyButton {
                            text: q.clone(),
                            title: t(locale, TextKey::CopySparqlQuery),
                            locale,
                        }
                        pre {
                            class: "m-0 max-h-96 overflow-y-auto whitespace-pre-wrap break-all {classes::RADIUS_CARD} border border-border bg-surface p-4 font-mono text-ui text-text",
                            "{q.as_ref()}"
                        }
                    }
                }
            }
        }
    }
}
