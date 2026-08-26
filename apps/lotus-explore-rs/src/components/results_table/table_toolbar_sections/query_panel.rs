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

    let mut criteria_effect_ready = use_signal(|| false);
    let mut panel_visible = use_signal(|| toolbar_snapshot.read().sparql_query.is_some());
    let mut panel_open = use_signal(|| false);

    use_effect(move || {
        let _ = form_ctx.criteria.read();
        if *criteria_effect_ready.peek() {
            panel_visible.set(false);
            panel_open.set(false);
        } else {
            criteria_effect_ready.set(true);
        }
    });

    use_effect(move || {
        let current_query = toolbar_snapshot.read();
        if !*panel_visible.peek() {
            panel_visible.set(current_query.sparql_query.is_some());
        }
    });

    rsx! {
        if *panel_visible.read() {
            if let Some(q) = toolbar_snapshot.read().sparql_query.as_ref() {
                details {
                    class: "overflow-hidden rounded-xl border border-panel-border bg-panel-soft shadow-xs",
                    open: *panel_open.read(),
                    onchange: move |evt: FormEvent| {
                        panel_open.set(evt.value() == "true");
                    },
                    summary {
                        class: "flex cursor-pointer select-none items-center gap-2 px-3.5 py-2.5 text-ui font-semibold text-muted hover:bg-bg focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/40",
                        span {
                            class: if *panel_open.read() {
                                "inline-block rotate-90 text-subtle transition-transform"
                            } else {
                                "inline-block text-subtle transition-transform"
                            },
                            "▶"
                        }
                        "{t(locale, TextKey::SparqlQuery)}"
                    }
                    div { class: "flex flex-col gap-2 border-t border-border p-3",
                        CopyButton {
                            text: q.clone(),
                            title: t(locale, TextKey::CopySparqlQuery),
                            locale,
                        }
                        pre {
                            class: "m-0 max-h-80 overflow-y-auto whitespace-pre-wrap break-all rounded-lg border border-border bg-surface p-3 font-mono text-ui text-text",
                            "{q.as_ref()}"
                        }
                    }
                }
            }
        }
    }
}
