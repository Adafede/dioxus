// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Toolbar assembly for the results table: query panel, stats bar, downloads,
//! and the capped-rows notice.

use super::table_toolbar_sections::{CappedRowsNotice, DownloadActionsGroup, QueryPanel, StatBar};
use crate::features::explore::selectors::{use_result_selector, use_ui_selector};
use crate::models::DatasetStats;
use crate::state::use_results_context;
use dioxus::prelude::*;

/// Toolbar: query panel + stats bar + download actions + capped-rows notice.
///
/// Reads sparql_query, metadata_json, query_hash, result_hash, executed_criteria,
/// total_stats, total_matches, display_capped_rows, and entries (for fallback
/// stats) from context. Intentionally separate from `ResultsTable` so that
/// sort changes never cause toolbar re-renders.
#[component]
pub(super) fn ResultsToolbar() -> Element {
    let state = use_results_context();
    let explore = state.explore;
    let entries = use_result_selector(explore, |result| result.entries.clone());
    let sparql_query = use_result_selector(explore, |result| result.sparql_query.clone());
    let metadata_json = use_result_selector(explore, |result| result.metadata_json.clone());
    let query_hash = use_result_selector(explore, |result| result.query_hash.clone());
    let result_hash = use_result_selector(explore, |result| result.result_hash.clone());
    let total_stats = use_result_selector(explore, |result| result.total_stats.clone());
    let total_matches = use_result_selector(explore, |result| result.total_matches);
    let display_capped_rows = use_result_selector(explore, |result| result.display_capped_rows);
    let criteria = use_ui_selector(explore, |ui| ui.executed_criteria.clone());

    let fallback_stats: Memo<DatasetStats> =
        use_memo(move || DatasetStats::from_entries(entries.read().as_ref()));
    let display_stats = total_stats
        .read()
        .as_ref()
        .cloned()
        .unwrap_or_else(|| fallback_stats.read().clone());
    let stats_partial = false;

    rsx! {
        QueryPanel { sparql_query: sparql_query.read().clone() }

        div { class: "results-toolbar",
            StatBar {
                stats: display_stats,
                total_matches: *total_matches.read(),
                stats_partial,
            }
            DownloadActionsGroup {
                criteria: criteria.read().clone(),
                sparql_query: sparql_query.read().clone(),
                metadata_json: metadata_json.read().clone(),
                query_hash: query_hash.read().clone(),
                result_hash: result_hash.read().clone(),
            }
        }
        if *display_capped_rows.read() {
            CappedRowsNotice {}
        }
    }
}
