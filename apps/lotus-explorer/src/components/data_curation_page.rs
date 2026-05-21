// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::curation::build_curation_share_url;
use crate::features::curation::state::page_controller::CurationUiState;
use crate::features::curation::use_curation_page_controller;
use dioxus::prelude::*;
use std::sync::Arc;

use super::curation_results_table::CurationResultsTable;

mod sections;
use sections::{
    AddRowCard, QueueRowsCard, QuickStatementsCard, ShareBar, StatusNotice, TsvImportCard,
};

#[component]
pub fn DataCurationPage() -> Element {
    let locale = crate::hooks::use_locale();
    let mut controller = use_curation_page_controller(locale);
    let shareable_url = use_memo(move || {
        build_curation_share_url(controller.rows.read().as_slice(), locale, true)
            .map(Arc::<str>::from)
    });
    let ui_state = CurationUiState::from_controller(controller);
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

    rsx! {
        section { class: "curation-wrap",
            div { class: "curation-grid",
                AddRowCard {
                    locale,
                    form: controller.form,
                    processing: ui_state.processing,
                    on_add_row,
                    on_load_examples,
                }

                TsvImportCard {
                    locale,
                    tsv_input: controller.tsv_input,
                    processing: ui_state.processing,
                    has_tsv_input,
                    on_parse_tsv,
                    on_import_uploaded_tsv,
                    on_import_error,
                }
            }

            if let Some(share) = shareable_url.read().as_ref() {
                ShareBar { locale, share: share.clone() }
            }

            if let Some(status) = controller.status_message.read().as_ref() {
                StatusNotice { locale, message: Arc::<str>::from(status.as_str()) }
            }

            QueueRowsCard {
                locale,
                rows: controller.rows,
                processing: ui_state.processing,
                on_process,
            }

            if !controller.result_rows.read().is_empty() {
                CurationResultsTable { locale, rows: controller.result_rows.read().clone() }
            }

            QuickStatementsCard {
                locale,
                quickstatements: controller.quickstatements,
                awaiting_second_pass: ui_state.awaiting_second_pass,
                processing: ui_state.processing,
                on_second_pass,
            }
        }
    }
}
