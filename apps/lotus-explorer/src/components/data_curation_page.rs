// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::curation::build_curation_share_url;
use crate::features::curation::state::page_controller::use_curation_page_controller;
use crate::features::explore::absolute_share_url;
use crate::i18n::{TextKey, t};
use dioxus::prelude::*;
use std::sync::Arc;

use super::copy_button::CopyButton;
use super::curation_results_table::CurationResultsTable;

mod sections;
use sections::{AddRowCard, QueueRowsCard, QuickStatementsCard, TsvImportCard};

#[component]
pub fn DataCurationPage() -> Element {
    let locale = crate::hooks::use_locale();
    let mut controller = use_curation_page_controller(locale);
    let shareable_url = use_memo(move || {
        build_curation_share_url(controller.rows.read().as_slice(), locale, true)
            .map(Arc::<str>::from)
    });
    let processing_value = *controller.processing.read();
    let awaiting_second_pass_value = *controller.awaiting_second_pass.read();
    let has_tsv_input = controller.has_tsv_input();

    use_effect(move || {
        controller.maybe_autorun();
    });

    let on_add_row = move |_: ()| controller.add_row();
    let on_load_examples = move |_: ()| controller.load_example_rows();
    let on_parse_tsv = move |_: ()| controller.parse_tsv();
    let on_process = move |_: ()| controller.process();
    let on_second_pass = move |_: ()| controller.run_second_pass();
    let on_import_uploaded_tsv = move |content: String| controller.import_uploaded_tsv(content);
    let on_import_error = move |message: String| controller.status_message.set(Some(message));
    let rows_for_table = controller.result_rows.read().clone();

    rsx! {
        section { class: "curation-wrap",
            div { class: "curation-grid",
                AddRowCard {
                    locale,
                    form: controller.form,
                    processing: processing_value,
                    on_add_row,
                    on_load_examples,
                }

                TsvImportCard {
                    locale,
                    tsv_input: controller.tsv_input,
                    processing: processing_value,
                    has_tsv_input,
                    on_parse_tsv,
                    on_import_uploaded_tsv,
                    on_import_error,
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

            if let Some(msg) = controller.status_message.read().as_deref() {
                div { class: "notice notice-info", role: "status",
                    span { class: "notice-label", "{t(locale, TextKey::Notice)}" }
                    span { class: "notice-value", "{msg}" }
                }
            }

            QueueRowsCard {
                locale,
                rows: controller.rows,
                processing: processing_value,
                on_process,
            }

            if !rows_for_table.is_empty() {
                CurationResultsTable { locale, rows: rows_for_table }
            }

            QuickStatementsCard {
                locale,
                quickstatements: controller.quickstatements,
                awaiting_second_pass: awaiting_second_pass_value,
                processing: processing_value,
                on_second_pass,
            }
        }
    }
}
