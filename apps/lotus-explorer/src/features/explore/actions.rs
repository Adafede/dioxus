// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Action catalog for the Explore feature reducer.

use crate::features::explore::types::{ErrorKind, QueryPhase};
use crate::models::{CompoundEntry, DatasetStats, SearchCriteria, SortColumn};
use std::sync::Arc;

/// All state transitions that can occur in the Explore feature.
#[derive(Clone, PartialEq)]
pub enum ExploreAction {
    /// Start a new search lifecycle.
    SearchRequested {
        criteria_snapshot: SearchCriteria,
        direct_download: bool,
    },

    /// Update the spinner / lifecycle phase.
    SearchPhaseChanged(QueryPhase),

    /// Commit a successful search result set.
    SearchSucceeded {
        rows: Vec<CompoundEntry>,
        qid: Option<String>,
        warning: Option<String>,
        query: String,
        total_matches: Option<usize>,
        total_stats: Option<DatasetStats>,
        display_capped_rows: bool,
        query_hash: String,
        result_hash: String,
        metadata_json: Arc<str>,
    },

    /// Commit a typed search error.
    SearchFailed {
        kind: ErrorKind,
        message: String,
    },

    /// Dismiss the current error notice.
    ErrorDismissed,

    /// Toggle the mobile filter drawer.
    MobileFiltersToggled,

    /// Start/stop download dispatching.
    DownloadDispatchStarted,
    DownloadDispatchFinished,

    /// Toggle a results-table sort column.
    SortToggled(SortColumn),
}
