// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Toolbar assembly for the results table: query panel, stats bar, downloads,
//! and the capped-rows notice.

use super::table_toolbar_sections::{
    CappedRowsNotice, DownloadActionsGroup, QueryPanel, StatBar,
};
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
    let explore = state.explore.read().clone();
    let entries = explore.result.entries.clone();
    let sparql_query = explore.result.sparql_query.clone();
    let metadata_json = explore.result.metadata_json.clone();
    let query_hash = explore.result.query_hash.clone();
    let result_hash = explore.result.result_hash.clone();
    let criteria = explore.ui.executed_criteria.clone();
    let total_stats = explore.result.total_stats.clone();
    let total_matches = explore.result.total_matches;
    let display_capped_rows = explore.result.display_capped_rows;

    let fallback_stats: Memo<DatasetStats> = use_memo(move || DatasetStats::from_entries(&entries));
    let display_stats = total_stats
        .as_ref()
        .cloned()
        .unwrap_or_else(|| fallback_stats.read().clone());
    let stats_partial = false;

    rsx! {
        QueryPanel { sparql_query: sparql_query.clone() }

        div { class: "results-toolbar",
            StatBar {
                stats: display_stats,
                total_matches,
                stats_partial,
            }
            DownloadActionsGroup {
                criteria: criteria.clone(),
                sparql_query: sparql_query.clone(),
                metadata_json: metadata_json.clone(),
                query_hash,
                result_hash,
            }
        }
        if display_capped_rows {
            CappedRowsNotice {}
        }
    }
}

