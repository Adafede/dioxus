// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::curation::{CurationInputRow, QuickStatementsBundle};
use crate::features::curation::services::quickstatements::build_qs_dev_link;
use crate::hooks::use_add_row_form::AddRowForm;
use crate::i18n::{
    Locale, TextKey, button_add_row, button_append_tsv_rows, button_generate_quickstatements,
    button_generating, button_load_example_rows, button_remove, button_second_pass, col_action,
    col_name, curation_qs_dev_label, curation_qs_dev_main_hint, curation_qs_dev_prereq_hint,
    heading_add_one_row, heading_queued_rows, heading_quickstatements,
    heading_quickstatements_dependencies, heading_tsv_import, hint_expected_tsv_headers,
    msg_delay_advice, msg_two_step_hint, placeholder_doi_optional, placeholder_molecule_name,
    placeholder_taxon_optional, t,
};
use dioxus::prelude::*;

use crate::components::copy_button::CopyButton;
use crate::features::explore::absolute_share_url;
use std::sync::Arc;

#[component]
pub fn ShareBar(locale: Locale, share: Arc<str>) -> Element {
    rsx! {
        div { class: "share-bar", role: "status",
            span { class: "share-bar-label", "{t(locale, TextKey::Share)}" }
            input {
                aria_label: "{t(locale, TextKey::CopyShareableLink)}",
                class: "share-bar-input mono",
                r#type: "text",
                readonly: true,
                value: "{share}",
            }
            CopyButton {
                text: Arc::<str>::from(absolute_share_url(&share)),
                title: t(locale, TextKey::CopyShareableLink),
                locale,
            }
        }
    }
}

#[component]
pub fn StatusNotice(locale: Locale, message: Arc<str>) -> Element {
    rsx! {
        div { class: "notice notice-info", role: "status",
            span { class: "notice-label", "{t(locale, TextKey::Notice)}" }
            span { class: "notice-value", "{message}" }
        }
    }
}

#[component]
pub fn AddRowCard(
    locale: Locale,
    form: AddRowForm,
    processing: bool,
    on_add_row: EventHandler<()>,
    on_load_examples: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "curation-card",
            h3 { "{heading_add_one_row(locale)}" }
            div { class: "curation-form-grid",
                label { class: "form-label", r#for: "curation-name-input",
                    "{placeholder_molecule_name(locale)}"
                }
                input {
                    id: "curation-name-input",
                    class: "form-input",
                    r#type: "text",
                    placeholder: "{placeholder_molecule_name(locale)}",
                    value: "{form.name}",
                    oninput: move |e| form.name.set(e.value()),
                }
                label {
                    class: "form-label",
                    r#for: "curation-smiles-input",
                    "SMILES"
                }
                input {
                    id: "curation-smiles-input",
                    class: "form-input",
                    r#type: "text",
                    placeholder: "SMILES",
                    value: "{form.smiles}",
                    oninput: move |e| form.smiles.set(e.value()),
                }
                label {
                    class: "form-label",
                    r#for: "curation-taxon-input",
                    "{placeholder_taxon_optional(locale)}"
                }
                input {
                    id: "curation-taxon-input",
                    class: "form-input",
                    r#type: "text",
                    placeholder: "{placeholder_taxon_optional(locale)}",
                    value: "{form.taxon}",
                    oninput: move |e| form.taxon.set(e.value()),
                }
                label { class: "form-label", r#for: "curation-doi-input",
                    "{placeholder_doi_optional(locale)}"
                }
                input {
                    id: "curation-doi-input",
                    class: "form-input",
                    r#type: "text",
                    placeholder: "{placeholder_doi_optional(locale)}",
                    value: "{form.doi}",
                    oninput: move |e| form.doi.set(e.value()),
                }
            }
            div { class: "curation-actions",
                button {
                    class: "btn btn-sm btn-primary",
                    r#type: "button",
                    onclick: move |_| on_add_row.call(()),
                    "{button_add_row(locale)}"
                }
                button {
                    class: "btn btn-sm btn-soft-accent",
                    r#type: "button",
                    disabled: processing,
                    onclick: move |_| on_load_examples.call(()),
                    "{button_load_example_rows(locale)}"
                }
            }
        }
    }
}

#[component]
pub fn TsvImportCard(
    locale: Locale,
    tsv_input: Signal<String>,
    processing: bool,
    has_tsv_input: bool,
    on_parse_tsv: EventHandler<()>,
    on_import_uploaded_tsv: EventHandler<String>,
    on_import_error: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "curation-card",
            h3 { "{heading_tsv_import(locale)}" }
            p { class: "curation-hint", "{hint_expected_tsv_headers(locale)}" }
            label { class: "form-label", r#for: "curation-tsv-input", "TSV" }
            textarea {
                id: "curation-tsv-input",
                class: "form-textarea curation-tsv",
                aria_describedby: "curation-tsv-hint",
                value: "{tsv_input}",
                oninput: move |e| tsv_input.set(e.value()),
            }
            p { id: "curation-tsv-hint", class: "sr-only",
                "{hint_expected_tsv_headers(locale)}"
            }
            div { class: "curation-actions",
                button {
                    class: "btn btn-sm",
                    r#type: "button",
                    disabled: processing || !has_tsv_input,
                    onclick: move |_| on_parse_tsv.call(()),
                    "{button_append_tsv_rows(locale)}"
                }
                input {
                    class: "curation-file-input",
                    aria_label: "TSV file upload",
                    r#type: "file",
                    accept: ".tsv,text/tab-separated-values,text/plain",
                    disabled: processing,
                    onchange: move |evt| {
                        let files = evt.files();
                        let Some(file) = files.first().cloned() else {
                            return;
                        };
                        spawn(async move {
                            match file.read_string().await {
                                Ok(content) => on_import_uploaded_tsv.call(content),
                                Err(err) => on_import_error.call(err.to_string()),
                            }
                        });
                    },
                }
            }
        }
    }
}

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
    quickstatements: Signal<QuickStatementsBundle>,
    awaiting_second_pass: bool,
    processing: bool,
    on_second_pass: EventHandler<()>,
) -> Element {
    let quickstatements = quickstatements.read().clone();
    if quickstatements.dependencies.is_empty() && quickstatements.main.is_empty() {
        return rsx! {};
    }

    let qs_dependency_link = build_qs_dev_link(&quickstatements.dependencies);
    let qs_main_link = build_qs_dev_link(&quickstatements.main);

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
