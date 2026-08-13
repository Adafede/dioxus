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
use std::sync::Arc;
use ui::prelude::*;

use crate::ui::style_constants::primary_buttons;

use crate::components::copy_button::CopyButton;
use crate::features::explore::absolute_share_url;

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
    rsx! {
        NoticeBar {
            label: t(locale, TextKey::Notice).to_string(),
            tone: NoticeTone::Warning,
            role: "status",
            aria_live: "polite",
            dark: is_dark_mode(),
            margin: "0",
            span { style: curation_notice_value_style(), "{message}" }
        }
    }
}

/// Detect if the system is in dark mode.
fn is_dark_mode() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(media) = window.match_media("(prefers-color-scheme: dark)") {
                if let Some(media_query) = media {
                    return media_query.matches();
                }
            }
        }
    }
    false
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
        div { style: curation_card_style(),
            h3 { "{heading_add_one_row(locale)}" }
            div { style: curation_form_grid_style(),
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
            div { style: curation_actions_style(false),
                button {
                    style: button_primary_sm_style(),
                    r#type: "button",
                    onclick: move |_| on_add_row.call(()),
                    "{button_add_row(locale)}"
                }
                button {
                    style: button_primary_sm_style(),
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
        div { style: curation_card_style(),
            h3 { "{heading_tsv_import(locale)}" }
            p { style: curation_hint_style(), "{hint_expected_tsv_headers(locale)}" }
            label { class: "form-label", r#for: "curation-tsv-input", "TSV" }
            textarea {
                id: "curation-tsv-input",
                class: "form-textarea mono",
                style: curation_textarea_style("130px"),
                aria_describedby: "curation-tsv-hint",
                value: "{tsv_input}",
                oninput: move |e| tsv_input.set(e.value()),
            }
            p { id: "curation-tsv-hint", class: "sr-only",
                "{hint_expected_tsv_headers(locale)}"
            }
            div { style: curation_actions_style(false),
                button {
                    r#type: "button",
                    disabled: processing || !has_tsv_input,
                    onclick: move |_| on_parse_tsv.call(()),
                    style: button_sm_style(),
                    "{button_append_tsv_rows(locale)}"
                }
                input {
                    class: "curation-file-input",
                    style: curation_file_input_style(),
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
        div { style: curation_card_style(),
            div { style: curation_actions_style(true),
                h3 { "{heading_queued_rows(locale)}" }
                button {
                style: button_primary_sm_style(),
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
            div {
                class: "curation-table-scroll",
                style: curation_table_scroll_style(),
                role: "region",
                tabindex: "0",
                aria_label: "{heading_queued_rows(locale)}",
                table {
                    class: "curation-table curation-queue-table",
                    style: queue_table_style(),
                    thead {
                        tr {
                            th { style: queue_action_col_style(), "{col_action(locale)}" }
                            th { style: queue_index_col_style(), "#" }
                            th { "{col_name(locale)}" }
                            th { style: queue_smiles_col_style(), "SMILES" }
                            th { "{t(locale, TextKey::TaxonCol)}" }
                            th { "DOI" }
                        }
                    }
                    tbody {
                        for (idx, row) in rows_snapshot.iter().enumerate() {
                            tr { style: row_stripe_style(idx),
                                td { style: queue_action_col_style(),
                                    button {
                                        r#type: "button",
                                        style: button_xs_style(),
                                        onclick: move |_| {
                                            if idx < rows.read().len() {
                                                rows.write().remove(idx);
                                            }
                                        },
                                        "{button_remove(locale)}"
                                    }
                                }
                                td { style: queue_index_col_style(), "{idx + 1}" }
                                td { "{row.name}" }
                                td { style: queue_smiles_col_style(), class: "mono", "{row.smiles}" }
                                td { "{row.taxon.as_deref().unwrap_or(\"\")}" }
                                td { class: "mono", "{row.doi.as_deref().unwrap_or(\"\")}" }
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
        div { style: curation_card_style(),
            if !qs_ref.dependencies.is_empty() {
                p { style: curation_hint_style(), "{msg_two_step_hint(locale)}" }
                p { style: curation_hint_style(), "{msg_delay_advice(locale)}" }
                p { style: curation_hint_style(),
                    a {
                        href: "{qs_dependency_link}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        "{curation_qs_dev_label(locale)}"
                    }
                    " - {curation_qs_dev_prereq_hint(locale)}"
                }
                div { style: curation_actions_style(true),
                    h3 { "{heading_quickstatements_dependencies(locale)}" }
                    CopyButton {
                        text: qs_ref.dependencies.clone(),
                        locale,
                    }
                }
                textarea {
                    class: "form-textarea mono",
                    style: curation_textarea_style("220px"),
                    aria_label: "{heading_quickstatements_dependencies(locale)}",
                    readonly: true,
                    value: "{qs_ref.dependencies}",
                }
                button {
                    style: button_primary_block_style(),
                    r#type: "button",
                    disabled: processing,
                    onclick: move |_| on_second_pass.call(()),
                    "{button_second_pass(locale)}"
                }
            }

            if !awaiting_second_pass && !qs_ref.main.is_empty() {
                p { style: curation_hint_style(),
                    a {
                        href: "{qs_main_link}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        "{curation_qs_dev_label(locale)}"
                    }
                    " - {curation_qs_dev_main_hint(locale)}"
                }
                div { style: curation_actions_style(true),
                    h3 { "{heading_quickstatements(locale)}" }
                    CopyButton {
                        text: qs_ref.main.clone(),
                        locale,
                    }
                }
                textarea {
                    class: "form-textarea mono",
                    style: curation_textarea_style("220px"),
                    aria_label: "{heading_quickstatements(locale)}",
                    readonly: true,
                    value: "{qs_ref.main}",
                }
            }
        }
    }
}

fn curation_card_style() -> String {
    StyleBuilder::new()
        .property("display", "flex")
        .property("flex-direction", "column")
        .property("gap", "10px")
        .property("padding", "12px")
        .property("border", "1px solid var(--panel-border)")
        .property("border-radius", "var(--radius)")
        .property("background", "var(--panel-bg-soft)")
        .property("box-shadow", "var(--panel-shadow)")
        .build()
}

fn curation_form_grid_style() -> String {
    StyleBuilder::new()
        .property("display", "grid")
        .property("grid-template-columns", "1fr")
        .property("gap", "8px")
        .build()
}

fn curation_actions_style(space_between: bool) -> String {
    let mut style = StyleBuilder::new()
        .property("display", "flex")
        .property("flex-wrap", "wrap")
        .property("gap", "8px")
        .property("align-items", "center");
    if space_between {
        style = style.property("justify-content", "space-between");
    }
    style.build()
}

fn curation_hint_style() -> String {
    StyleBuilder::new()
        .property("font-size", "var(--fs-0)")
        .property("color", "var(--text)")
        .build()
}

fn curation_textarea_style(min_height: &str) -> String {
    StyleBuilder::new()
        .property("min-height", min_height)
        .property("font-family", "var(--mono)")
        .property("border-radius", "8px")
        .property("resize", "none")
        .build()
}

fn curation_file_input_style() -> String {
    StyleBuilder::new()
        .property("color", "var(--text2)")
        .property("max-width", "100%")
        .property("font-size", "var(--fs-0)")
        .build()
}

fn curation_notice_value_style() -> String {
    StyleBuilder::new()
        .color("inherit")
        .property("word-break", "break-word")
        .property("line-height", "1.4")
        .build()
}

fn curation_table_scroll_style() -> String {
    StyleBuilder::new()
        .property("width", "100%")
        .property("min-width", "0")
        .property("overflow-x", "auto")
        .property("overflow-y", "visible")
        .property("border", "1px solid var(--panel-border)")
        .property("background", "var(--panel-bg-soft)")
        .property("box-shadow", "var(--panel-shadow)")
        .property(
            "transition",
            "background .15s ease, border-color .15s ease, box-shadow .15s ease",
        )
        .build()
}

fn queue_table_style() -> String {
    StyleBuilder::new()
        .property("width", "100%")
        .property("border-collapse", "collapse")
        .property("font-size", "var(--fs-ui)")
        .property("table-layout", "auto")
        .property("word-break", "break-word")
        .build()
}

fn queue_action_col_style() -> String {
    StyleBuilder::new()
        .property("width", "110px")
        .property("min-width", "110px")
        .build()
}

fn queue_index_col_style() -> String {
    StyleBuilder::new().property("min-width", "3ch").build()
}

fn queue_smiles_col_style() -> String {
    StyleBuilder::new()
        .property("min-width", "220px")
        .property("max-width", "320px")
        .build()
}

fn row_stripe_style(idx: usize) -> String {
    let background = if idx.is_multiple_of(2) {
        "color-mix(in srgb, var(--surface) 94%, transparent)"
    } else {
        "color-mix(in srgb, var(--surface) 88%, transparent)"
    };

    StyleBuilder::new()
        .property("transition", "background .14s ease")
        .property("--row-bg", background)
        .build()
}

fn button_primary_sm_style() -> String {
    primary_buttons::button_primary_sm_style()
}

fn button_primary_block_style() -> String {
    primary_buttons::button_primary_block_style()
}

fn button_sm_style() -> String {
    primary_buttons::button_sm_style()
}

fn button_xs_style() -> String {
    primary_buttons::button_xs_style()
}
