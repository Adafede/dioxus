// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Curation-page UI sections: share-bar, status notice, add-row /
//! TSV-import / queue / quickstatements cards, plus dark-mode detection.

use crate::components::ui::{Button, ButtonSize, ButtonVariant};
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
use crate::ui::classes;
use dioxus::prelude::*;
use std::sync::Arc;
use upload::{extract_blob_from_file_data, read_blob_string};

use crate::components::copy_button::CopyButton;
use crate::features::explore::absolute_share_url;

#[component]
pub fn ShareBar(locale: Locale, share: Arc<str>) -> Element {
    rsx! {
        div { class: "{classes::SHARE_BAR}", role: "status",
            span { class: "{classes::SHARE_BAR_LABEL}", "{t(locale, TextKey::Share)}" }
            div { class: "flex flex-col gap-2",
                input {
                    aria_label: "{t(locale, TextKey::CopyShareableLink)}",
                    class: "w-full font-mono {classes::INPUT}",
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
}

#[cfg(test)]
mod tests {
    use crate::curation::CurationInputRow;

    #[test]
    fn queue_rows_removal_logic_preserves_other_rows() {
        let mut rows = vec![
            CurationInputRow {
                name: "A".to_string(),
                smiles: "CCO".to_string(),
                taxon: None,
                doi: None,
            },
            CurationInputRow {
                name: "B".to_string(),
                smiles: "CCN".to_string(),
                taxon: None,
                doi: None,
            },
            CurationInputRow {
                name: "C".to_string(),
                smiles: "CCC".to_string(),
                taxon: None,
                doi: None,
            },
        ];
        rows.remove(1);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].name, "A");
        assert_eq!(rows[1].name, "C");
    }
}

#[component]
pub fn StatusNotice(locale: Locale, message: Arc<str>) -> Element {
    rsx! {
        div {
            class: "{classes::NOTICE_BAR} {classes::NOTICE_WARNING}",
            role: "status",
            aria_live: "polite",
            span { class: "{classes::NOTICE_LABEL} {classes::NOTICE_LABEL_WARNING}", "{t(locale, TextKey::Notice)}" }
            span { class: "{classes::NOTICE_BODY} break-words leading-snug text-inherit", "{message}" }
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
    let schema = r#"{"type":"object","properties":{"name":{"type":"string","description":"Compound name"},"smiles":{"type":"string","description":"SMILES representation"},"taxon":{"type":"string","description":"Taxon name or identifier"},"doi":{"type":"string","description":"Optional DOI"}},"additionalProperties":true}"#;

    rsx! {
        form {
            id: "lotus-curation-add-row-form",
            "data-webmcp-id": "lotus-curation-add-row-form",
            "data-webmcp-type": "form",
            "data-webmcp-name": "LOTUS curation add-row form",
            "data-webmcp-description": "Add a single curated natural-product record with a name, SMILES, taxon, and DOI.",
            "data-webmcp-schema": "{schema}",
            "data-mcp-id": "lotus-curation-add-row-form",
            "data-mcp-type": "form",
            "data-mcp-name": "LOTUS curation add-row form",
            "data-mcp-description": "Add a single curated natural-product record with a name, SMILES, taxon, and DOI.",
            "data-mcp-schema": "{schema}",
            onsubmit: move |evt: Event<FormData>| {
                evt.prevent_default();
                on_add_row.call(());
            },
            class: "flex flex-col gap-4 rounded-xl bg-panel-soft p-4 shadow-xs",
            h3 { "{heading_add_one_row(locale)}" }
            div { class: "{classes::FORM_GRID}",
                label { class: "form-label", r#for: "curation-name-input",
                    "{placeholder_molecule_name(locale)}"
                }
                input {
                    id: "curation-name-input",
                    name: "name",
                    class: "form-input {classes::INPUT}",
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
                    name: "smiles",
                    class: "form-input {classes::INPUT}",
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
                    name: "taxon",
                    class: "form-input {classes::INPUT}",
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
                    name: "doi",
                    class: "form-input {classes::INPUT}",
                    r#type: "text",
                    placeholder: "{placeholder_doi_optional(locale)}",
                    value: "{form.doi}",
                    oninput: move |e| form.doi.set(e.value()),
                }
            }
            div { class: "{classes::ACTIONS}",
                Button {
                    label: button_add_row(locale).to_string(),
                    variant: ButtonVariant::Primary,
                    size: ButtonSize::Sm,
                    onclick: Some(EventHandler::new(move |_: Event<MouseData>| on_add_row.call(()))),
                }
                Button {
                    label: button_load_example_rows(locale).to_string(),
                    variant: ButtonVariant::Secondary,
                    size: ButtonSize::Sm,
                    disabled: processing,
                    onclick: Some(EventHandler::new(move |_: Event<MouseData>| on_load_examples.call(()))),
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
    let tsv_schema = r#"{"type":"object","properties":{"tsv":{"type":"string","description":"TSV rows with name, SMILES, taxon, and DOI columns"}},"additionalProperties":true}"#;

    rsx! {
        form {
            id: "lotus-curation-tsv-form",
            "data-webmcp-id": "lotus-curation-tsv-form",
            "data-webmcp-type": "form",
            "data-webmcp-name": "LOTUS TSV import form",
            "data-webmcp-description": "Paste or upload a TSV file of curated compound rows to import into the queue.",
            "data-webmcp-schema": "{tsv_schema}",
            "data-mcp-id": "lotus-curation-tsv-form",
            "data-mcp-type": "form",
            "data-mcp-name": "LOTUS TSV import form",
            "data-mcp-description": "Paste or upload a TSV file of curated compound rows to import into the queue.",
            "data-mcp-schema": "{tsv_schema}",
            onsubmit: move |evt: Event<FormData>| {
                evt.prevent_default();
                if has_tsv_input {
                    on_parse_tsv.call(());
                }
            },
            class: "flex flex-col gap-4 rounded-xl bg-panel-soft p-4 shadow-xs",
            h3 { "{heading_tsv_import(locale)}" }
            p { class: "text-ui text-subtle leading-snug", "{hint_expected_tsv_headers(locale)}" }
            label { class: "form-label", r#for: "curation-tsv-input", "TSV" }
            textarea {
                id: "curation-tsv-input",
                name: "tsv",
                class: "{classes::TEXTAREA_130}",
                aria_describedby: "curation-tsv-hint",
                value: "{tsv_input}",
                oninput: move |e| tsv_input.set(e.value()),
            }
            p { id: "curation-tsv-hint", class: "sr-only",
                "{hint_expected_tsv_headers(locale)}"
            }
            div { class: "{classes::ACTIONS}",
                Button {
                    label: button_append_tsv_rows(locale).to_string(),
                    variant: ButtonVariant::Secondary,
                    size: ButtonSize::Sm,
                    disabled: processing || !has_tsv_input,
                    onclick: Some(EventHandler::new(move |_: Event<MouseData>| on_parse_tsv.call(()))),
                }
                input {
                    class: "curation-file-input max-w-full text-ui text-muted",
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
                            match extract_blob_from_file_data(&[file]) {
                                Ok(Some(extracted)) => {
                                    match read_blob_string(&extracted.blob).await {
                                        Ok(content) => on_import_uploaded_tsv.call(content),
                                        Err(err) => on_import_error.call(err.to_string()),
                                    }
                                }
                                Ok(None) => {}
                                Err(err) => on_import_error.call(err),
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
    let rows_snapshot = rows.read().clone();

    rsx! {
        div { class: "flex flex-col gap-4 rounded-xl bg-panel-soft p-4 shadow-xs",
            div { class: "flex flex-wrap items-center justify-between gap-2.5",
                h3 { "{heading_queued_rows(locale)}" }
                Button {
                    label: if processing {
                        button_generating(locale).to_string()
                    } else {
                        button_generate_quickstatements(locale).to_string()
                    },
                    variant: ButtonVariant::Primary,
                    size: ButtonSize::Sm,
                    disabled: processing,
                    onclick: Some(EventHandler::new(move |_: Event<MouseData>| on_process.call(()))),
                }
            }
            div {
                class: "w-full overflow-x-auto {classes::RADIUS_CARD} {classes::BORDER_PANEL} focus-visible:outline-none {classes::FOCUS_RING_BTN}",
                role: "region",
                tabindex: "0",
                aria_label: "{heading_queued_rows(locale)}",
                table {
                    class: "w-full min-w-max table-auto border-collapse text-ui",
                    thead {
                        tr { class: "text-left",
                            th { scope: "col", class: "{classes::TABLE_TH} {classes::QUEUE_ACTION_COL}", "{col_action(locale)}" }
                            th { scope: "col", class: "{classes::TABLE_TH} {classes::QUEUE_INDEX_COL}", "#" }
                            th { scope: "col", class: "{classes::TABLE_TH} w-[140px] min-w-[140px]", "{col_name(locale)}" }
                            th { scope: "col", class: "{classes::TABLE_TH} min-w-[220px]", "SMILES" }
                            th { scope: "col", class: "{classes::TABLE_TH} w-[140px] min-w-[140px]", "{t(locale, TextKey::TaxonCol)}" }
                            th { scope: "col", class: "{classes::TABLE_TH} w-[140px] min-w-[140px]", "DOI" }
                        }
                    }
                    tbody {
                        if rows_snapshot.is_empty() {
                            tr {
                                td { class: "{classes::TD} {classes::QUEUE_ACTION_COL} font-mono text-micro", "-" }
                                td { class: "{classes::TD} {classes::QUEUE_INDEX_COL} font-mono text-micro", "-" }
                                td { class: "{classes::TD} font-mono text-micro", "-" }
                                td { class: "{classes::TD} min-w-[220px] font-mono text-micro", "-" }
                                td { class: "{classes::TD} font-mono text-micro", "-" }
                                td { class: "{classes::TD} font-mono text-micro", "-" }
                            }
                        } else {
                            for (idx, row) in rows_snapshot.iter().enumerate() {
                                tr { key: "{row.name}|{row.smiles}",
                                    class: "odd:bg-surface/30 hover:bg-surface/60 focus-visible:outline-none {classes::FOCUS_RING} focus-visible:ring-offset-[-2px]",
                                    tabindex: "0",
                                    td { class: "{classes::TD} {classes::QUEUE_ACTION_COL}",
                                        Button {
                                            label: button_remove(locale).to_string(),
                                            variant: ButtonVariant::Danger,
                                            size: ButtonSize::Sm,
                                            onclick: Some(EventHandler::new(move |_: Event<MouseData>| {
                                                let row_count = rows.read().len();
                                                if idx < row_count {
                                                    rows.write().remove(idx);
                                                }
                                            })),
                                        }
                                    }
                                    td { class: "{classes::TD} {classes::QUEUE_INDEX_COL} font-mono text-micro", "{idx + 1}" }
                                    td { class: "{classes::TD}", "{row.name}" }
                                    td { class: "{classes::TD} min-w-[220px]", "{row.smiles}" }
                                    td { class: "{classes::TD}", "{row.taxon.as_deref().unwrap_or(\"\")}" }
                                    td { class: "{classes::TD} font-mono text-micro", "{row.doi.as_deref().unwrap_or(\"\")}" }
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
    let qs_ref = quickstatements.read();
    if qs_ref.dependencies.is_empty() && qs_ref.main.is_empty() {
        return rsx! {};
    }

    let qs_dependency_link = build_qs_dev_link(&qs_ref.dependencies);
    let qs_main_link = build_qs_dev_link(&qs_ref.main);

    rsx! {
        div { class: "flex flex-col gap-4 rounded-xl bg-panel-soft p-4 shadow-xs",
            if !qs_ref.dependencies.is_empty() {
                p { class: "text-ui text-subtle leading-snug", "{msg_two_step_hint(locale)}" }
                p { class: "text-ui text-subtle leading-snug", "{msg_delay_advice(locale)}" }
                p { class: "text-ui text-subtle leading-snug",
                    a {
                        href: "{qs_dependency_link}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        "{curation_qs_dev_label(locale)}"
                    }
                    " - {curation_qs_dev_prereq_hint(locale)}"
                }
                div { class: "flex flex-wrap items-center justify-between gap-2.5",
                    h3 { "{heading_quickstatements_dependencies(locale)}" }
                    CopyButton {
                        text: qs_ref.dependencies.clone(),
                        locale,
                    }
                }
                textarea {
                    class: "{classes::TEXTAREA_220}",
                    aria_label: "{heading_quickstatements_dependencies(locale)}",
                    readonly: true,
                    value: "{qs_ref.dependencies}",
                }
                Button {
                    label: button_second_pass(locale).to_string(),
                    variant: ButtonVariant::Secondary,
                    size: ButtonSize::Sm,
                    disabled: processing,
                    onclick: Some(EventHandler::new(move |_: Event<MouseData>| on_second_pass.call(()))),
                }
            }

            if !awaiting_second_pass && !qs_ref.main.is_empty() {
                p { class: "text-ui text-subtle leading-snug",
                    a {
                        href: "{qs_main_link}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        "{curation_qs_dev_label(locale)}"
                    }
                    " - {curation_qs_dev_main_hint(locale)}"
                }
                div { class: "flex flex-wrap items-center justify-between gap-2.5",
                    h3 { "{heading_quickstatements(locale)}" }
                    CopyButton {
                        text: qs_ref.main.clone(),
                        locale,
                    }
                }
                textarea {
                    class: "{classes::TEXTAREA_220}",
                    aria_label: "{heading_quickstatements(locale)}",
                    readonly: true,
                    value: "{qs_ref.main}",
                }
            }
        }
    }
}
