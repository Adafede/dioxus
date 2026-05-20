// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::curation::{
    CurationInputRow, CurationResultRow, QuickStatementsBundle, build_curation_share_url,
    curate_rows, example_rows, initial_curation_autorun_from_url, initial_curation_rows_from_url,
};
use crate::features::curation::queue::{append_unique_rows, non_empty_trimmed};
use crate::features::curation::services::quickstatements::build_qs_dev_link;
use crate::features::curation::state::page_controller::{
    import_tsv_rows, rows_to_tsv, start_curation_run,
};
use crate::features::curation::workflow;
use crate::features::explore::url_state::absolute_share_url;
use crate::i18n::{
    TextKey, button_add_row, button_append_tsv_rows, button_load_example_rows, heading_add_one_row,
    heading_tsv_import, hint_expected_tsv_headers, msg_add_row_before_generate,
    msg_duplicate_row_skipped, msg_examples_loaded, msg_name_smiles_required,
    msg_second_pass_running, placeholder_doi_optional, placeholder_molecule_name,
    placeholder_taxon_optional, t,
};
use dioxus::prelude::*;
use std::sync::Arc;

use super::copy_button::CopyButton;
use super::curation_results_table::CurationResultsTable;

mod sections;
use sections::{QueueRowsCard, QuickStatementsCard};

#[component]
pub fn DataCurationPage() -> Element {
    let locale = crate::hooks::use_locale();
    let mut name_input = use_signal(String::new);
    let mut smiles_input = use_signal(String::new);
    let mut taxon_input = use_signal(String::new);
    let mut doi_input = use_signal(String::new);
    let mut tsv_input = use_signal(String::new);
    let mut rows = use_signal(initial_curation_rows_from_url);
    let mut processing = use_signal(|| false);
    let mut status_message = use_signal(|| Option::<String>::None);
    let mut result_rows = use_signal(|| Arc::<[CurationResultRow]>::from([]));
    let mut quickstatements = use_signal(QuickStatementsBundle::default);
    let mut awaiting_second_pass = use_signal(|| false);
    let mut autorun_pending = use_signal(initial_curation_autorun_from_url);
    let shareable_url = use_memo(move || {
        build_curation_share_url(rows.read().as_slice(), locale, true).map(Arc::<str>::from)
    });
    let status_message_value = status_message.read().clone();
    let processing_value = *processing.read();
    let awaiting_second_pass_value = *awaiting_second_pass.read();
    let result_rows_value = result_rows.read().clone();
    let quickstatements_value = quickstatements.read().clone();
    let has_tsv_input = !tsv_input.read().trim().is_empty();
    let qs_dependency_link = build_qs_dev_link(&quickstatements_value.dependencies);
    let qs_main_link = build_qs_dev_link(&quickstatements_value.main);

    let on_add_row = move |_| {
        let name = name_input.read().trim().to_string();
        let smiles = smiles_input.read().trim().to_string();
        if name.is_empty() || smiles.is_empty() {
            *status_message.write() = Some(msg_name_smiles_required(locale));
            return;
        }
        let row = CurationInputRow {
            name,
            smiles,
            taxon: non_empty_trimmed(&taxon_input.read()),
            doi: non_empty_trimmed(&doi_input.read()),
        };
        if append_unique_rows(&mut rows.write(), vec![row]).skipped > 0 {
            *status_message.write() = Some(msg_duplicate_row_skipped(locale));
            return;
        }
        *status_message.write() = None;
        name_input.set(String::new());
        smiles_input.set(String::new());
        taxon_input.set(String::new());
        doi_input.set(String::new());
    };

    let on_parse_tsv = move |_| {
        let content = tsv_input.read().clone();
        import_tsv_rows(locale, &content, rows, status_message);
    };

    let on_process = move |_: ()| {
        let snapshot = rows.read().clone();
        if snapshot.is_empty() {
            *status_message.write() = Some(msg_add_row_before_generate(locale));
            return;
        }
        start_curation_run(
            locale,
            snapshot,
            processing,
            status_message,
            result_rows,
            quickstatements,
            awaiting_second_pass,
        );
    };

    use_effect(move || {
        let should_autorun = *autorun_pending.read();
        let snapshot = rows.read().clone();
        let already_has_results = !result_rows.read().is_empty();
        if should_autorun && !snapshot.is_empty() && !*processing.read() && !already_has_results {
            autorun_pending.set(false);
            start_curation_run(
                locale,
                snapshot,
                processing,
                status_message,
                result_rows,
                quickstatements,
                awaiting_second_pass,
            );
        }
    });

    let on_second_pass = move |_: ()| {
        let pending_inputs = workflow::second_pass_inputs(result_rows.read().as_ref());
        if pending_inputs.is_empty() {
            awaiting_second_pass.set(false);
            return;
        }

        let previous_rows = result_rows.read().clone();
        processing.set(true);
        status_message.set(Some(msg_second_pass_running(locale).to_string()));

        spawn(async move {
            match curate_rows(locale, pending_inputs).await {
                Ok((updated_rows, _)) => {
                    let outcome = workflow::apply_second_pass(locale, &previous_rows, updated_rows);
                    result_rows.set(outcome.result_rows);
                    quickstatements.set(outcome.quickstatements);
                    awaiting_second_pass.set(outcome.awaiting_second_pass);
                    processing.set(false);
                    status_message.set(Some(outcome.status_message));
                }
                Err(err) => {
                    processing.set(false);
                    status_message.set(Some(workflow::format_curation_error_typed(locale, &err)));
                }
            }
        });
    };

    rsx! {
        section { class: "curation-wrap",

            div { class: "curation-grid",
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
                            value: "{name_input}",
                            oninput: move |e| name_input.set(e.value()),
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
                            value: "{smiles_input}",
                            oninput: move |e| smiles_input.set(e.value()),
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
                            value: "{taxon_input}",
                            oninput: move |e| taxon_input.set(e.value()),
                        }
                        label { class: "form-label", r#for: "curation-doi-input",
                            "{placeholder_doi_optional(locale)}"
                        }
                        input {
                            id: "curation-doi-input",
                            class: "form-input",
                            r#type: "text",
                            placeholder: "{placeholder_doi_optional(locale)}",
                            value: "{doi_input}",
                            oninput: move |e| doi_input.set(e.value()),
                        }
                    }
                    div { class: "curation-actions",
                        button {
                            class: "btn btn-sm btn-primary",
                            r#type: "button",
                            onclick: on_add_row,
                            "{button_add_row(locale)}"
                        }
                        button {
                            class: "btn btn-sm btn-soft-accent",
                            r#type: "button",
                            disabled: processing_value,
                            onclick: move |_| {
                                let samples = example_rows();
                                tsv_input.set(rows_to_tsv(&samples));
                                let outcome = append_unique_rows(&mut rows.write(), samples);
                                *status_message.write() = Some(msg_examples_loaded(
                                    locale,
                                    outcome.added,
                                    outcome.skipped,
                                ));
                            },
                            "{button_load_example_rows(locale)}"
                        }
                    }
                }

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
                            disabled: processing_value || !has_tsv_input,
                            onclick: on_parse_tsv,
                            "{button_append_tsv_rows(locale)}"
                        }
                        input {
                            class: "curation-file-input",
                            aria_label: "TSV file upload",
                            r#type: "file",
                            accept: ".tsv,text/tab-separated-values,text/plain",
                            disabled: processing_value,
                            onchange: move |evt| {
                                let files = evt.files();
                                let Some(file) = files.first().cloned() else {
                                    return;
                                };
                                spawn(async move {
                                    match file.read_string().await {
                                        Ok(content) => {
                                            tsv_input.set(content.clone());
                                            import_tsv_rows(
                                                locale,
                                                &content,
                                                rows,
                                                status_message,
                                            );
                                        }
                                        Err(err) => {
                                            status_message.set(Some(err.to_string()));
                                        }
                                    }
                                });
                            },
                        }
                    }
                }
            }

            if let Some(share) = shareable_url.read().as_deref() {
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
                        text: Arc::<str>::from(absolute_share_url(share)),
                        title: t(locale, TextKey::CopyShareableLink),
                        locale,
                    }
                }
            }

            if let Some(msg) = status_message_value.as_deref() {
                div { class: "notice notice-info", role: "status",
                    span { class: "notice-label", "{t(locale, TextKey::Notice)}" }
                    span { class: "notice-value", "{msg}" }
                }
            }

            QueueRowsCard {
                locale,
                rows,
                processing: processing_value,
                on_process,
            }

            if !result_rows_value.is_empty() {
                CurationResultsTable { locale, rows: result_rows_value.clone() }
            }

            QuickStatementsCard {
                locale,
                quickstatements: quickstatements_value.clone(),
                awaiting_second_pass: awaiting_second_pass_value,
                processing: processing_value,
                qs_dependency_link,
                qs_main_link,
                on_second_pass,
            }
        }
    }
}

// helpers moved to `features::curation::state::page_controller`
