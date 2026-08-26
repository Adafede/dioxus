// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Curation-page UI sections: share-bar, status notice, add-row /
//! TSV-import / queue / quickstatements cards, plus dark-mode detection.

// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::curation::{CurationInputRow, QuickStatementsBundle};
use crate::components::ui::{Button, ButtonSize, ButtonVariant};
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
use std::sync::Arc;
use ui::prelude::*;

use crate::components::copy_button::CopyButton;
use crate::features::explore::absolute_share_url;
use crate::state::use_app_state_context;

mod styles;

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
    let dark_mode = use_app_state_context().state.read().dark_mode;
    rsx! {
        NoticeBar {
            label: t(locale, TextKey::Notice).to_string(),
            tone: NoticeTone::Warning,
            role: "status",
            aria_live: "polite",
            dark: dark_mode,
            margin: "0",
            span { class: "{styles::NOTICE_VALUE}", "{message}" }
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
            class: "{styles::CARD}",
            h3 { "{heading_add_one_row(locale)}" }
            div { class: "{styles::FORM_GRID}",
                label { class: "form-label", r#for: "curation-name-input",
                    "{placeholder_molecule_name(locale)}"
                }
                input {
                    id: "curation-name-input",
                    name: "name",
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
                    name: "smiles",
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
                    name: "taxon",
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
                    name: "doi",
                    class: "form-input",
                    r#type: "text",
                    placeholder: "{placeholder_doi_optional(locale)}",
                    value: "{form.doi}",
                    oninput: move |e| form.doi.set(e.value()),
                }
            }
            div { class: "{styles::actions(false)}",
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
            class: "{styles::CARD}",
            h3 { "{heading_tsv_import(locale)}" }
            p { class: "{styles::HINT}", "{hint_expected_tsv_headers(locale)}" }
            label { class: "form-label", r#for: "curation-tsv-input", "TSV" }
            textarea {
                id: "curation-tsv-input",
                name: "tsv",
                class: "{styles::TEXTAREA_130}",
                aria_describedby: "curation-tsv-hint",
                value: "{tsv_input}",
                oninput: move |e| tsv_input.set(e.value()),
            }
            p { id: "curation-tsv-hint", class: "sr-only",
                "{hint_expected_tsv_headers(locale)}"
            }
            div { class: "{styles::actions(false)}",
                Button {
                    label: button_append_tsv_rows(locale).to_string(),
                    variant: ButtonVariant::Secondary,
                    size: ButtonSize::Sm,
                    disabled: processing || !has_tsv_input,
                    onclick: Some(EventHandler::new(move |_: Event<MouseData>| on_parse_tsv.call(()))),
                }
                input {
                    class: "curation-file-input {styles::FILE_INPUT}",
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
    let rows_snapshot = rows.read().clone();

    rsx! {
        div { class: "{styles::CARD}",
            div { class: "{styles::actions(true)}",
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
                class: "{styles::TABLE_SCROLL}",
                role: "region",
                tabindex: "0",
                aria_label: "{heading_queued_rows(locale)}",
                table {
                    class: "{styles::QUEUE_TABLE}",
                    thead {
                        tr { class: "text-left",
                            th { scope: "col", class: "{styles::TH} {styles::QUEUE_ACTION_COL}", "{col_action(locale)}" }
                            th { scope: "col", class: "{styles::TH} {styles::QUEUE_INDEX_COL}", "#" }
                            th { scope: "col", class: "{styles::TH}", "{col_name(locale)}" }
                            th { scope: "col", class: "{styles::TH} {styles::QUEUE_SMILES_COL}", "SMILES" }
                            th { scope: "col", class: "{styles::TH}", "{t(locale, TextKey::TaxonCol)}" }
                            th { scope: "col", class: "{styles::TH}", "DOI" }
                        }
                    }
                    tbody {
                        if rows_snapshot.is_empty() {
                            tr {
                                td { class: "{styles::TD} {styles::QUEUE_ACTION_COL} font-mono text-xs", "-" }
                                td { class: "{styles::TD} {styles::QUEUE_INDEX_COL} font-mono text-xs", "-" }
                                td { class: "{styles::TD} font-mono text-xs", "-" }
                                td { class: "{styles::TD} {styles::QUEUE_SMILES_COL} font-mono text-xs", "-" }
                                td { class: "{styles::TD} font-mono text-xs", "-" }
                                td { class: "{styles::TD} font-mono text-xs", "-" }
                            }
                        } else {
                            for (idx, row) in rows_snapshot.iter().enumerate() {
                                tr { key: "{row.name}|{row.smiles}",
                                    class: "odd:bg-surface/30 hover:bg-surface/60",
                                    td { class: "{styles::TD} {styles::QUEUE_ACTION_COL}",
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
                                    td { class: "{styles::TD} {styles::QUEUE_INDEX_COL} font-mono text-xs", "{idx + 1}" }
                                    td { class: "{styles::TD}", "{row.name}" }
                                    td { class: "{styles::TD} {styles::QUEUE_SMILES_COL}", "{row.smiles}" }
                                    td { class: "{styles::TD}", "{row.taxon.as_deref().unwrap_or(\"\")}" }
                                    td { class: "{styles::TD} font-mono text-xs", "{row.doi.as_deref().unwrap_or(\"\")}" }
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
        div { class: "{styles::CARD}",
            if !qs_ref.dependencies.is_empty() {
                p { class: "{styles::HINT}", "{msg_two_step_hint(locale)}" }
                p { class: "{styles::HINT}", "{msg_delay_advice(locale)}" }
                p { class: "{styles::HINT}",
                    a {
                        href: "{qs_dependency_link}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        "{curation_qs_dev_label(locale)}"
                    }
                    " - {curation_qs_dev_prereq_hint(locale)}"
                }
                div { class: "{styles::actions(true)}",
                    h3 { "{heading_quickstatements_dependencies(locale)}" }
                    CopyButton {
                        text: qs_ref.dependencies.clone(),
                        locale,
                    }
                }
                textarea {
                    class: "{styles::TEXTAREA_220}",
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
                p { class: "{styles::HINT}",
                    a {
                        href: "{qs_main_link}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        "{curation_qs_dev_label(locale)}"
                    }
                    " - {curation_qs_dev_main_hint(locale)}"
                }
                div { class: "{styles::actions(true)}",
                    h3 { "{heading_quickstatements(locale)}" }
                    CopyButton {
                        text: qs_ref.main.clone(),
                        locale,
                    }
                }
                textarea {
                    class: "{styles::TEXTAREA_220}",
                    aria_label: "{heading_quickstatements(locale)}",
                    readonly: true,
                    value: "{qs_ref.main}",
                }
            }
        }
    }
}
