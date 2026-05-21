// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::features::explore::use_table_result_snapshot;
use crate::i18n::{TextKey, t};
use crate::state::use_results_context;
use crate::ui::a11y_contract::{RESULTS_SECTION_HEADING_ID, RESULTS_SECTION_ID};
use dioxus::prelude::*;

mod download_model;
mod header_model;
mod render_model;
mod row_cells;
mod scroll_runtime;
mod sort_helpers;
mod sort_model;
mod table_header;
mod table_toolbar_sections;
mod table_view_model;
mod toolbar;
mod virtualization_controller;
mod virtualized_table;

use table_view_model::{apply_sort, prepare_table_state};
use toolbar::ResultsToolbar;
use virtualized_table::VirtualizedResultsTable;

const TABLE_SCROLL_ID: &str = "results-table-scroll";
const VIRTUAL_OVERSCAN_ROWS: usize = 12;
const ROW_HEIGHT_PX_COMFORTABLE: usize = 114;
const TABLE_VIEWPORT_FALLBACK_PX: usize = 640;

/// Renders the full results section.
///
/// Reactive surface is deliberately narrow: this component subscribes only to
/// `entries` (for the empty-state check) and `locale`. Table preparation is
/// split into two memos:
/// 1. `prepared_state` — re-runs only when entries change (expensive: row prep + sort cache)
/// 2. `table_view_model` — re-runs when entries OR sort changes (cheap: index selection only)
/// Sort interactions therefore never re-run row preparation, and never re-render the toolbar.
#[component]
pub fn ResultsTable() -> Element {
    let state = use_results_context();
    let explore = state.explore;
    let locale = crate::hooks::use_locale();
    let snapshot = use_table_result_snapshot(explore);
    let entries: Memo<crate::models::Rows> = use_memo(move || snapshot.read().entries.clone());
    let entries_len = entries.read().len();

    // Expensive step: row text derivation + sort-index cache. Only re-runs when entries change.
    let prepared_state = use_memo(move || prepare_table_state(&entries.read()));

    // Cheap step: pick the right sort indices. Re-runs when entries or sort changes.
    let table_view_model =
        use_memo(move || apply_sort(&prepared_state.read(), snapshot.read().sort));

    let total = entries_len;

    rsx! {
        section {
            id: RESULTS_SECTION_ID,
            class: "results-wrap",
            aria_label: "{t(locale, TextKey::TableTriplesAria)}",
            aria_labelledby: RESULTS_SECTION_HEADING_ID,
            h2 { id: RESULTS_SECTION_HEADING_ID, class: "sr-only", "{t(locale, TextKey::TableTriplesAria)}" }
            ResultsToolbar {}

            if total == 0 {
                div { class: "empty-state",
                    p { class: "form-hint", "{t(locale, TextKey::NoResults)}" }
                }
            } else {
                VirtualizedResultsTable {
                    entries,
                    table_view_model,
                }
            }
        }
    }
}
