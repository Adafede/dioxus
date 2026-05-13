// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::curation::{
    CurationInputRow, CurationResultRow, CurationStatus, QuickStatementsBundle,
    build_curation_share_url, build_quickstatements_bundle, curate_rows, example_rows,
    initial_curation_autorun_from_url, initial_curation_rows_from_url, parse_tsv_rows,
    row_uniqueness_key,
};
use crate::i18n::{
    Locale, TextKey, button_add_row, button_append_tsv_rows, button_generate_quickstatements,
    button_generating, button_load_example_rows, button_remove, button_second_pass, col_action,
    col_name, curation_qs_dev_label, curation_qs_dev_main_hint, curation_qs_dev_prereq_hint,
    heading_add_one_row, heading_queued_rows, heading_quickstatements,
    heading_quickstatements_dependencies, heading_tsv_import, hint_expected_tsv_headers,
    msg_add_row_before_generate, msg_curation_failed, msg_curation_rate_limited, msg_delay_advice,
    msg_done_review_copy, msg_duplicate_row_skipped, msg_examples_loaded, msg_name_smiles_required,
    msg_no_valid_tsv_rows, msg_prerequisites_pending, msg_running_checks, msg_second_pass_done,
    msg_second_pass_running, msg_second_pass_still_pending_count, msg_tsv_import_complete,
    msg_two_step_hint, placeholder_doi_optional, placeholder_molecule_name,
    placeholder_taxon_optional, subtitle_curation_explorer, t, title_curation_explorer,
};
use dioxus::prelude::*;
use std::collections::HashMap;
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
            taxon: non_empty_opt(&taxon_input.read()),
            doi: non_empty_opt(&doi_input.read()),
        };
        let key = row_uniqueness_key(&row);
        if rows
            .read()
            .iter()
            .any(|existing| row_uniqueness_key(existing) == key)
        {
            *status_message.write() = Some(msg_duplicate_row_skipped(locale));
            return;
        }
        rows.write().push(row);
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
                let (added, skipped) = append_unique_rows(&mut rows.write(), parsed);
                *status_message.write() = Some(msg_tsv_import_complete(locale, added, skipped));
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
        let pending_inputs = result_rows
            .read()
            .iter()
            .filter(|row| !row.dependency_blocks.is_empty())
            .map(|row| row.input.clone())
            .collect::<Vec<_>>();
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
                    let mut by_key = HashMap::new();
                    for row in updated_rows {
                        by_key.insert(row_uniqueness_key(&row.input), row);
                    }

                    let merged_rows = previous_rows
                        .iter()
                        .cloned()
                        .map(|row| {
                            let key = row_uniqueness_key(&row.input);
                            by_key.remove(&key).unwrap_or(row)
                        })
                        .collect::<Vec<_>>();

                                    let bundle = build_quickstatements_bundle(&merged_rows);
                    let still_pending = !bundle.dependencies.is_empty();
                    let pending_count = merged_rows
                        .iter()
                        .filter(|row| !row.dependency_blocks.is_empty())
                        .count();
                    result_rows.set(Arc::from(merged_rows.into_boxed_slice()));
                    quickstatements.set(bundle);
                    awaiting_second_pass.set(still_pending);
                    processing.set(false);
                    status_message.set(Some(if still_pending {
                        msg_second_pass_still_pending_count(locale, pending_count)
                    } else {
                        msg_second_pass_done(locale).to_string()
                    }));
                }
                Err(err) => {
                    processing.set(false);
                    status_message.set(Some(format_curation_error(locale, &err.to_string())));
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
                                let (added, skipped) = append_unique_rows(&mut rows.write(), example_rows());
                                *status_message.write() = Some(msg_examples_loaded(locale, added, skipped));
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

            if let Some(msg) = status_message.read().as_deref() {
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
                        disabled: *processing.read(),
                        onclick: on_process,
                        if *processing.read() {
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

            if !result_rows.read().is_empty() {
                CurationResultsTable { locale, rows: result_rows.read().clone() }
            }

            if !quickstatements.read().dependencies.is_empty()
                || !quickstatements.read().main.is_empty()
            {
                div { class: "curation-card",
                    if !quickstatements.read().dependencies.is_empty() {
                        p { class: "curation-hint", "{msg_two_step_hint(locale)}" }
                        p { class: "curation-hint", "{msg_delay_advice(locale)}" }
                        p { class: "curation-hint",
                            a {
                                href: "https://qs-dev.toolforge.org/",
                                target: "_blank",
                                rel: "noopener noreferrer",
                                "{curation_qs_dev_label(locale)}"
                            }
                            " — {curation_qs_dev_prereq_hint(locale)}"
                        }
                        div { class: "curation-actions curation-space-between",
                            h3 { "{heading_quickstatements_dependencies(locale)}" }
                            CopyButton {
                                text: quickstatements.read().dependencies.clone(),
                                locale,
                            }
                        }
                        textarea {
                            class: "form-textarea curation-qs",
                            aria_label: "{heading_quickstatements_dependencies(locale)}",
                            readonly: true,
                            value: "{quickstatements.read().dependencies}",
                        }
                        button {
                            class: "btn btn-sm btn-primary",
                            r#type: "button",
                            disabled: *processing.read(),
                            onclick: on_second_pass,
                            "{button_second_pass(locale)}"
                        }
                    }

                    if !*awaiting_second_pass.read() && !quickstatements.read().main.is_empty() {
                        p { class: "curation-hint",
                            a {
                                href: "https://qs-dev.toolforge.org/",
                                target: "_blank",
                                rel: "noopener noreferrer",
                                "{curation_qs_dev_label(locale)}"
                            }
                            " — {curation_qs_dev_main_hint(locale)}"
                        }
                        div { class: "curation-actions curation-space-between",
                            h3 { "{heading_quickstatements(locale)}" }
                            CopyButton {
                                text: quickstatements.read().main.clone(),
                                locale,
                            }
                        }
                        textarea {
                            class: "form-textarea curation-qs",
                            aria_label: "{heading_quickstatements(locale)}",
                            readonly: true,
                            value: "{quickstatements.read().main}",
                        }
                    }
                }
            }
        }
    }
}

fn non_empty_opt(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn append_unique_rows(
    queue: &mut Vec<CurationInputRow>,
    candidates: Vec<CurationInputRow>,
) -> (usize, usize) {
    let mut added = 0usize;
    let mut skipped = 0usize;
    for row in candidates {
        let key = row_uniqueness_key(&row);
        if queue
            .iter()
            .any(|existing| row_uniqueness_key(existing) == key)
        {
            skipped += 1;
            continue;
        }
        queue.push(row);
        added += 1;
    }
    (added, skipped)
}

fn format_curation_error(locale: Locale, detail: &str) -> String {
    if detail.contains("HTTP 429")
        || detail.contains("rate limited")
        || detail.contains("10 per 1 minute")
    {
        return msg_curation_rate_limited(locale).to_string();
    }

    msg_curation_failed(locale, detail)
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
        match curate_rows(locale, snapshot).await {
            Ok((curated_rows, qs)) => {
                let pending_count = curated_rows
                    .iter()
                    .filter(|row| matches!(row.status, CurationStatus::PendingDependencies))
                    .count();
                awaiting_second_pass.set(!qs.dependencies.is_empty());
                result_rows.set(Arc::from(curated_rows.into_boxed_slice()));
                quickstatements.set(qs);
                processing.set(false);
                status_message.set(Some(if pending_count > 0 {
                    msg_prerequisites_pending(locale, pending_count)
                } else {
                    msg_done_review_copy(locale)
                }));
            }
            Err(err) => {
                processing.set(false);
                status_message.set(Some(format_curation_error(locale, &err.to_string())));
            }
        }
    });
}

fn absolute_share_url(share: &str) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(win) = web_sys::window() {
            let loc = win.location();
            if let (Ok(origin), Ok(pathname)) = (loc.origin(), loc.pathname()) {
                return format!("{origin}{pathname}{share}");
            }
        }
    }
    share.to_string()
}

