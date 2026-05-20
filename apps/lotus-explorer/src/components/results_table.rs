// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::features::explore::selectors::use_result_selector;
use crate::i18n::{TextKey, t};
use crate::state::use_results_context;
use crate::ui::a11y_contract::{RESULTS_SECTION_HEADING_ID, RESULTS_SECTION_ID};
use dioxus::prelude::*;

mod download_model;
mod row_cells;
mod scroll_runtime;
mod sort_helpers;
mod sort_model;
mod table_header;
mod table_toolbar_sections;
mod table_view_model;
mod toolbar;
mod virtualized_table;

use table_view_model::build_table_view_model;
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
/// isolated in `build_table_view_model`, which is memoized to avoid unnecessary
/// re-preparation. All query-panel / stats / download signals are delegated to
/// `ResultsToolbar`, which subscribes to them independently. Sort interactions
/// therefore only re-render `VirtualizedResultsTable`, not the toolbar or stats bar.
#[component]
pub fn ResultsTable() -> Element {
    let state = use_results_context();
    let explore = state.explore;
    let locale = crate::hooks::use_locale();
    let entries = use_result_selector(explore, |result| result.entries.clone());
    let sort_state = use_result_selector(explore, |result| result.sort);
    let entries_len = entries.read().len();

    // Single unified memo for all table preparation: entries + sort_state → prepared view model.
    let table_view_model = use_memo(move || {
        build_table_view_model(&entries.read(), *sort_state.read())
    });

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
                    p { "{t(locale, TextKey::NoResults)}" }
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
