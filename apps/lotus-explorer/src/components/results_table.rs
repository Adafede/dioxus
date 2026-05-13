// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::i18n::{TextKey, t};
use crate::state::use_results_context;
use dioxus::prelude::*;
use std::sync::Arc;

#[path = "results_table/sort_helpers.rs"]
mod sort_helpers;

#[path = "results_table/sort_model.rs"]
mod sort_model;

#[path = "results_table/table_toolbar_sections.rs"]
mod table_toolbar_sections;

#[path = "results_table/toolbar.rs"]
mod toolbar;
use toolbar::ResultsToolbar;

#[path = "results_table/row_cells.rs"]
mod row_cells;

#[path = "results_table/scroll_runtime.rs"]
mod scroll_runtime;

#[path = "results_table/table_header.rs"]
mod table_header;

#[path = "results_table/virtualized_table.rs"]
mod virtualized_table;
use virtualized_table::VirtualizedResultsTable;

const TABLE_SCROLL_ID: &str = "results-table-scroll";
const VIRTUAL_OVERSCAN_ROWS: usize = 12;
const ROW_HEIGHT_PX_COMFORTABLE: usize = 114;
const TABLE_VIEWPORT_FALLBACK_PX: usize = 640;

/// Renders the full results section.
///
/// Reactive surface is deliberately narrow: this component subscribes only to
/// `entries` (for the empty-state check and sort index) and `locale`. All
/// query-panel / stats / download signals are delegated to `ResultsToolbar`,
/// which subscribes to them independently. Sort interactions therefore only
/// re-render `VirtualizedResultsTable`, not the toolbar or stats bar.
#[component]
pub fn ResultsTable() -> Element {
    let state = use_results_context();
    let explore = state.explore;
    let locale = crate::hooks::use_locale();
    let entries_len = explore.read().result.entries.len();

    let sorted_indices: Memo<Arc<[u32]>> = use_memo(move || {
        let snapshot = explore.read();
        sort_model::build_sorted_indices(&snapshot.result.entries, snapshot.result.sort)
    });

    let total = entries_len;

    rsx! {
        div { id: "results-section", class: "results-wrap",
            ResultsToolbar {}

            if total == 0 {
                div { class: "empty-state",
                    p { "{t(locale, TextKey::NoResults)}" }
                }
            } else {
                VirtualizedResultsTable { explore, sorted_indices }
            }
        }
    }
}
