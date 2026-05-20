// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::curation::{CurationInputRow, QuickStatementsBundle};
use crate::i18n::{
    Locale, TextKey, button_generate_quickstatements, button_generating, button_remove,
    button_second_pass, col_action, col_name, curation_qs_dev_label, curation_qs_dev_main_hint,
    curation_qs_dev_prereq_hint, heading_queued_rows, heading_quickstatements,
    heading_quickstatements_dependencies, msg_delay_advice, msg_two_step_hint, t,
};
use dioxus::prelude::*;

use crate::components::copy_button::CopyButton;

#[component]
pub fn QueueRowsCard(
    locale: Locale,
    rows: Signal<Vec<CurationInputRow>>,
    processing: bool,
    on_process: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "curation-card",
            div { class: "curation-actions curation-space-between",
                h3 { "{heading_queued_rows(locale)}" }
                button {
                    class: "btn btn-sm btn-primary",
                    r#type: "button",
                    disabled: processing,
                    onclick: move |_| on_process.call(()),
                    if processing {
                        "{button_generating(locale)}"
                    } else {
                        "{button_generate_quickstatements(locale)}"
                    }
                }
            }
            table {
                class: "curation-table",
                aria_label: "{heading_queued_rows(locale)}",
                thead {
                    tr {
                        th { "#" }
                        th { "{col_name(locale)}" }
                        th { "SMILES" }
                        th { "{t(locale, TextKey::TaxonCol)}" }
                        th { "DOI" }
                        th { "{col_action(locale)}" }
                    }
                }
                tbody {
                    for (idx, row) in rows.read().iter().enumerate() {
                        tr {
                            td { "{idx + 1}" }
                            td { "{row.name}" }
                            td { class: "mono curation-cell-wrap", "{row.smiles}" }
                            td { "{row.taxon.as_deref().unwrap_or(\"\")}" }
                            td { class: "mono", "{row.doi.as_deref().unwrap_or(\"\")}" }
                            td {
                                button {
                                    class: "btn btn-xs",
                                    r#type: "button",
                                    onclick: move |_| {
                                        if idx < rows.read().len() {
                                            rows.write().remove(idx);
                                        }
                                    },
                                    "{button_remove(locale)}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn QuickStatementsCard(
    locale: Locale,
    quickstatements: QuickStatementsBundle,
    awaiting_second_pass: bool,
    processing: bool,
    qs_dependency_link: String,
    qs_main_link: String,
    on_second_pass: EventHandler<()>,
) -> Element {
    if quickstatements.dependencies.is_empty() && quickstatements.main.is_empty() {
        return rsx! {};
    }

    rsx! {
        div { class: "curation-card",
            if !quickstatements.dependencies.is_empty() {
                p { class: "curation-hint", "{msg_two_step_hint(locale)}" }
                p { class: "curation-hint", "{msg_delay_advice(locale)}" }
                p { class: "curation-hint",
                    a {
                        href: "{qs_dependency_link}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        "{curation_qs_dev_label(locale)}"
                    }
                    " - {curation_qs_dev_prereq_hint(locale)}"
                }
                div { class: "curation-actions curation-space-between",
                    h3 { "{heading_quickstatements_dependencies(locale)}" }
                    CopyButton {
                        text: quickstatements.dependencies.clone(),
                        locale,
                    }
                }
                textarea {
                    class: "form-textarea curation-qs",
                    aria_label: "{heading_quickstatements_dependencies(locale)}",
                    readonly: true,
                    value: "{quickstatements.dependencies}",
                }
                button {
                    class: "btn btn-sm btn-primary btn-block",
                    r#type: "button",
                    disabled: processing,
                    onclick: move |_| on_second_pass.call(()),
                    "{button_second_pass(locale)}"
                }
            }

            if !awaiting_second_pass && !quickstatements.main.is_empty() {
                p { class: "curation-hint",
                    a {
                        href: "{qs_main_link}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        "{curation_qs_dev_label(locale)}"
                    }
                    " - {curation_qs_dev_main_hint(locale)}"
                }
                div { class: "curation-actions curation-space-between",
                    h3 { "{heading_quickstatements(locale)}" }
                    CopyButton {
                        text: quickstatements.main.clone(),
                        locale,
                    }
                }
                textarea {
                    class: "form-textarea curation-qs",
                    aria_label: "{heading_quickstatements(locale)}",
                    readonly: true,
                    value: "{quickstatements.main}",
                }
            }
        }
    }
}
