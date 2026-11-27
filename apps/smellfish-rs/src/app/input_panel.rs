// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! CSV dropzone, SMILES paste panel, and status/warning telemetry strip.

#![expect(
    clippy::empty_enums,
    reason = "Dioxus #[component] macro generates empty Props builder helper enums"
)]

use dioxus::events::{DragData, FormData};
use dioxus::html::HasFileData;
use dioxus::prelude::*;
use ui::{Button, ButtonVariant};

use super::browser::{ImportSignals, attempt_import, attempt_import_from_text};

#[component]
pub(super) fn InputPanel(
    mut signals: ImportSignals,
    mut pasted_smiles: Signal<String>,
    mut demo_cleared: Signal<bool>,
) -> Element {
    let busy = *signals.busy.read();
    let dragging = *signals.drag_active.read();
    let ep_list = signals.endpoints.read();

    let on_file_change = move |evt: Event<FormData>| {
        attempt_import(&evt.data().files(), &signals);
    };
    let on_drag_activate = move |evt: Event<DragData>| {
        evt.prevent_default();
        signals.drag_active.set(true);
    };
    let on_drag_leave = move |evt: Event<DragData>| {
        evt.prevent_default();
        signals.drag_active.set(false);
    };
    let on_drop = move |evt: Event<DragData>| {
        attempt_import(&evt.data().files(), &signals);
    };
    let submit_pasted = move |_| {
        attempt_import_from_text(pasted_smiles.read().trim(), &signals);
    };

    rsx! {
        section { class: "panel input-panel",
            div { class: "input-split",
                label {
                    class: if dragging { "input-card dropzone dragging" } else { "input-card dropzone" },
                    r#for: "smiles-csv",
                    ondragenter: on_drag_activate,
                    ondragover: on_drag_activate,
                    ondragleave: on_drag_leave,
                    ondrop: on_drop,
                    div { class: "input-card-body",
                        strong { "Drop a CSV or click to browse" }
                        div { class: "small muted", "CSV with a smiles column" }
                    }
                    input {
                        id: "smiles-csv",
                        r#type: "file",
                        accept: ".csv,text/csv",
                        disabled: busy,
                        onchange: on_file_change,
                    }
                }

                div { class: "input-card paste-card",
                    div { class: "input-card-body",
                        div { class: "paste-head",
                            strong { "Paste SMILES" }
                            span { class: "small muted", "One per line" }
                        }
                        label { r#for: "smiles-paste", class: "visually-hidden", "SMILES structures, one per line" }
                        textarea {
                            id: "smiles-paste",
                            class: "smiles-textarea",
                            placeholder: "CCO\nC1CCCCC1\nCOC1=CC=CC=C1",
                            disabled: busy,
                            value: "{pasted_smiles}",
                            onfocus: move |_| {
                                if !*demo_cleared.read() {
                                    demo_cleared.set(true);
                                    pasted_smiles.set(String::new());
                                }
                            },
                            oninput: move |evt| pasted_smiles.set(evt.value()),
                        }
                    }
                    div { class: "paste-actions",
                        Button {
                            label: "Analyze pasted SMILES",
                            variant: ButtonVariant::Primary,
                            disabled: busy,
                            onclick: Some(EventHandler::new(submit_pasted)),
                        }
                    }
                }
            }

            if !signals.status.read().is_empty() || busy {
                p { class: "status", role: "status", aria_live: "polite", aria_atomic: "true",
                    if busy { span { class: "spinner" } }
                    "{signals.status}"
                }
            }
            if !ep_list.is_empty() {
                div { class: "endpoint-status",
                    for ep in ep_list.iter() {
                        span { class: "endpoint-chip", class: if ep.reachable { "ok" } else { "down" }, "{ep.name}: {ep.detail} ({ep.endpoint})" }
                    }
                }
            }
            if !signals.file_name.read().is_empty() {
                p { class: "small muted", "Loaded: {signals.file_name}" }
            }
            if !signals.warnings.read().is_empty() {
                div { class: "alert", "{signals.warnings.read().join(\" • \")}" }
            }
        }
    }
}
