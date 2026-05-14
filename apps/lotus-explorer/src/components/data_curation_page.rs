// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::curation::{
    CurationInputRow, CurationResultRow, QuickStatementsBundle, build_curation_share_url,
    curate_rows, example_rows, initial_curation_autorun_from_url, initial_curation_rows_from_url,
    parse_tsv_rows,
};
use crate::features::curation::queue::{append_unique_rows, non_empty_trimmed};
use crate::features::curation::services::quickstatements::build_qs_dev_link;
use crate::features::curation::workflow;
use crate::features::explore::url_state::absolute_share_url;
use crate::i18n::{
    Locale, TextKey, button_add_row, button_append_tsv_rows, button_generate_quickstatements,
    button_generating, button_load_example_rows, button_remove, button_second_pass, col_action,
    col_name, curation_qs_dev_label, curation_qs_dev_main_hint, curation_qs_dev_prereq_hint,
    heading_add_one_row, heading_queued_rows, heading_quickstatements,
    heading_quickstatements_dependencies, heading_tsv_import, hint_expected_tsv_headers,
    msg_add_row_before_generate, msg_delay_advice, msg_duplicate_row_skipped, msg_examples_loaded,
    msg_name_smiles_required, msg_no_valid_tsv_rows, msg_running_checks, msg_second_pass_running,
    msg_tsv_import_complete, msg_two_step_hint, placeholder_doi_optional,
    placeholder_molecule_name, placeholder_taxon_optional, subtitle_curation_explorer, t,
    title_curation_explorer,
};
use dioxus::prelude::*;
use std::sync::Arc;

use super::copy_button::CopyButton;
use super::curation_results_table::CurationResultsTable;

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

    let on_parse_tsv = move |_| match parse_tsv_rows(&tsv_input.read()) {
        Ok(parsed) => {
            if parsed.is_empty() {
                *status_message.write() = Some(msg_no_valid_tsv_rows(locale));
            } else {
                let outcome = append_unique_rows(&mut rows.write(), parsed);
                *status_message.write() = Some(msg_tsv_import_complete(
                    locale,
                    outcome.added,
                    outcome.skipped,
                ));
            }
        }
        Err(err) => {
            *status_message.write() = Some(err.to_string());
        }
    };

    let on_process = move |_| {
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

    let on_second_pass = move |_| {
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
                    status_message.set(Some(workflow::format_curation_error(
                        locale,
                        &err.to_string(),
                    )));
                }
            }
        });
    };

    rsx! {
        section { class: "curation-wrap",
            h2 { class: "curation-title", "{title_curation_explorer(locale)}" }
            p { class: "curation-subtitle", "{subtitle_curation_explorer(locale)}" }

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
                            class: "btn btn-sm",
                            r#type: "button",
                            onclick: move |_| {
                                let outcome = append_unique_rows(&mut rows.write(), example_rows());
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
                            onclick: on_parse_tsv,
                            "{button_append_tsv_rows(locale)}"
                        }
                        input {
                            aria_label: "TSV file upload",
                            r#type: "file",
                            accept: ".tsv,text/tab-separated-values,text/plain",
                            onchange: move |evt| {
                                let files = evt.files();
                                let Some(file) = files.first().cloned() else {
                                    return;
                                };
                                spawn(async move {
                                    if let Ok(content) = file.read_string().await {
                                        tsv_input.set(content);
                                    }
                                });
                            },
                        }
                    }
                }
            }

            if let Some(share) = shareable_url.read().as_deref() {
                div { class: "notice notice-info", role: "status",
                    span { class: "notice-label", "{t(locale, TextKey::Share)}" }
                    input {
                        aria_label: "{t(locale, TextKey::CopyShareableLink)}",
                        class: "notice-value notice-copy-field mono",
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

            div { class: "curation-card",
                div { class: "curation-actions curation-space-between",
                    h3 { "{heading_queued_rows(locale)}" }
                    button {
                        class: "btn btn-sm btn-primary",
                        r#type: "button",
                        disabled: processing_value,
                        onclick: on_process,
                        if processing_value {
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

            if !result_rows_value.is_empty() {
                CurationResultsTable { locale, rows: result_rows_value.clone() }
            }

            if !quickstatements_value.dependencies.is_empty()
                || !quickstatements_value.main.is_empty()
            {
                div { class: "curation-card",
                    if !quickstatements_value.dependencies.is_empty() {
                        p { class: "curation-hint", "{msg_two_step_hint(locale)}" }
                        p { class: "curation-hint", "{msg_delay_advice(locale)}" }
                        p { class: "curation-hint",
                            a {
                                href: "{qs_dependency_link}",
                                target: "_blank",
                                rel: "noopener noreferrer",
                                "{curation_qs_dev_label(locale)}"
                            }
                            " — {curation_qs_dev_prereq_hint(locale)}"
                        }
                        div { class: "curation-actions curation-space-between",
                            h3 { "{heading_quickstatements_dependencies(locale)}" }
                            CopyButton {
                                text: quickstatements_value.dependencies.clone(),
                                locale,
                            }
                        }
                        textarea {
                            class: "form-textarea curation-qs",
                            aria_label: "{heading_quickstatements_dependencies(locale)}",
                            readonly: true,
                            value: "{quickstatements_value.dependencies}",
                        }
                        button {
                            class: "btn btn-sm btn-primary",
                            r#type: "button",
                            disabled: processing_value,
                            onclick: on_second_pass,
                            "{button_second_pass(locale)}"
                        }
                    }

                    if !awaiting_second_pass_value && !quickstatements_value.main.is_empty() {
                        p { class: "curation-hint",
                            a {
                                href: "{qs_main_link}",
                                target: "_blank",
                                rel: "noopener noreferrer",
                                "{curation_qs_dev_label(locale)}"
                            }
                            " — {curation_qs_dev_main_hint(locale)}"
                        }
                        div { class: "curation-actions curation-space-between",
                            h3 { "{heading_quickstatements(locale)}" }
                            CopyButton {
                                text: quickstatements_value.main.clone(),
                                locale,
                            }
                        }
                        textarea {
                            class: "form-textarea curation-qs",
                            aria_label: "{heading_quickstatements(locale)}",
                            readonly: true,
                            value: "{quickstatements_value.main}",
                        }
                    }
                }
            }
        }
    }
}

fn start_curation_run(
    locale: Locale,
    snapshot: Vec<CurationInputRow>,
    mut processing: Signal<bool>,
    mut status_message: Signal<Option<String>>,
    mut result_rows: Signal<Arc<[CurationResultRow]>>,
    mut quickstatements: Signal<QuickStatementsBundle>,
    mut awaiting_second_pass: Signal<bool>,
) {
    processing.set(true);
    status_message.set(Some(msg_running_checks(locale)));
    spawn(async move {
        match workflow::run_curation(locale, snapshot).await {
            Ok(outcome) => {
                awaiting_second_pass.set(outcome.awaiting_second_pass);
                result_rows.set(outcome.result_rows);
                quickstatements.set(outcome.quickstatements);
                processing.set(false);
                status_message.set(Some(outcome.status_message));
            }
            Err(err) => {
                processing.set(false);
                status_message.set(Some(workflow::format_curation_error(
                    locale,
                    &err.to_string(),
                )));
            }
        }
    });
}
