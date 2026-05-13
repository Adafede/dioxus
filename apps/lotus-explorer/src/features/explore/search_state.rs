// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::features::explore::actions::ExploreAction;
use crate::features::explore::types::{ErrorKind, QueryPhase};
use crate::models::{CompoundEntry, DatasetStats, Rows, SearchCriteria, SortState};
use dioxus::prelude::*;
use std::sync::Arc;

#[derive(Clone, PartialEq)]
pub struct ExploreState {
    pub executed_criteria: SearchCriteria,
    pub loading: bool,
    pub error: Option<String>,
    pub error_kind: ErrorKind,
    pub query_phase: QueryPhase,
    pub searched_once: bool,
    pub download_only_mode: bool,
    pub download_dispatching: bool,
    pub entries: Rows,
    pub taxon_notice: Option<String>,
    pub resolved_qid: Option<String>,
    pub query_hash: Option<String>,
    pub result_hash: Option<String>,
    pub sparql_query: Option<Arc<str>>,
    pub metadata_json: Option<Arc<str>>,
    pub total_matches: Option<usize>,
    pub total_stats: Option<DatasetStats>,
    pub display_capped_rows: bool,
    pub mobile_filters_open: bool,
    pub search_request_token: u64,
    pub sort: SortState,
}

impl Default for ExploreState {
    fn default() -> Self {
        Self {
            executed_criteria: SearchCriteria::default(),
            loading: false,
            error: None,
            error_kind: ErrorKind::Unknown,
            query_phase: QueryPhase::Idle,
            searched_once: false,
            download_only_mode: false,
            download_dispatching: false,
            entries: Arc::<[CompoundEntry]>::from([]),
            taxon_notice: None,
            resolved_qid: None,
            query_hash: None,
            result_hash: None,
            sparql_query: None,
            metadata_json: None,
            total_matches: None,
            total_stats: None,
            display_capped_rows: false,
            mobile_filters_open: false,
            search_request_token: 0,
            sort: SortState::default(),
        }
    }
}

pub fn reduce(mut state: ExploreState, action: ExploreAction) -> ExploreState {
    match action {
        ExploreAction::SearchRequested {
            criteria_snapshot,
            direct_download,
        } => {
            state.executed_criteria = criteria_snapshot;
            state.loading = true;
            state.error = None;
            state.error_kind = ErrorKind::Unknown;
            state.query_phase = QueryPhase::ResolvingTaxon;
            state.searched_once = true;
            state.download_only_mode = direct_download;
            state.download_dispatching = false;
            state.entries = Arc::<[CompoundEntry]>::from([]);
            state.taxon_notice = None;
            state.resolved_qid = None;
            state.query_hash = None;
            state.result_hash = None;
            state.sparql_query = None;
            state.metadata_json = None;
            state.total_matches = None;
            state.total_stats = None;
            state.display_capped_rows = false;
            state.mobile_filters_open = false;
            state.search_request_token = state.search_request_token.saturating_add(1);
        }
        ExploreAction::SearchPhaseChanged(phase) => {
            state.query_phase = phase;
        }
        ExploreAction::SearchSucceeded {
            rows,
            qid,
            warning,
            query,
            total_matches,
            total_stats,
            display_capped_rows,
            query_hash,
            result_hash,
            metadata_json,
        } => {
            state.loading = false;
            state.error = None;
            state.error_kind = ErrorKind::Unknown;
            state.query_phase = QueryPhase::Idle;
            state.download_dispatching = false;
            state.entries = Arc::from(rows.into_boxed_slice());
            state.taxon_notice = warning;
            state.resolved_qid = qid;
            state.query_hash = Some(query_hash);
            state.result_hash = Some(result_hash);
            state.sparql_query = Some(Arc::<str>::from(query));
            state.metadata_json = Some(metadata_json);
            state.total_matches = total_matches;
            state.total_stats = total_stats;
            state.display_capped_rows = display_capped_rows;
        }
        ExploreAction::SearchFailed { kind, message } => {
            state.loading = false;
            state.error_kind = kind;
            state.error = Some(message);
            state.query_phase = QueryPhase::Idle;
            state.download_dispatching = false;
        }
        ExploreAction::ErrorDismissed => {
            state.error = None;
            state.error_kind = ErrorKind::Unknown;
        }
        ExploreAction::MobileFiltersToggled => {
            state.mobile_filters_open = !state.mobile_filters_open;
        }
        ExploreAction::DownloadDispatchStarted => {
            state.download_dispatching = true;
        }
        ExploreAction::DownloadDispatchFinished => {
            state.download_dispatching = false;
        }
        ExploreAction::SortToggled(column) => {
            if state.sort.col == column {
                state.sort.dir = if state.sort.dir == crate::models::SortDir::Asc {
                    crate::models::SortDir::Desc
                } else {
                    crate::models::SortDir::Asc
                };
            } else {
                state.sort.col = column;
                state.sort.dir = crate::models::SortDir::Asc;
            }
        }
    }
    state
}

pub fn dispatch_explore_action(mut state: Signal<ExploreState>, action: ExploreAction) {
    let next = reduce(state.peek().clone(), action);
    if *state.peek() != next {
        *state.write() = next;
    }
}

#[derive(Default, Clone, Copy)]
pub struct SearchMetrics {
    pub network_ms: f64,
    pub parse_ms: f64,
    pub sparql_calls: usize,
}

impl SearchMetrics {
    pub fn add_network(&mut self, elapsed: std::time::Duration) {
        self.network_ms += elapsed.as_secs_f64() * 1000.0;
        self.sparql_calls += 1;
    }

    pub fn add_parse(&mut self, elapsed: std::time::Duration) {
        self.parse_ms += elapsed.as_secs_f64() * 1000.0;
    }
}

pub fn emit_search_summary(total_elapsed: std::time::Duration, metrics: SearchMetrics) {
    let total_ms = total_elapsed.as_secs_f64() * 1000.0;
    let details = format!(
        "total_ms={total_ms:.1} network_ms={:.1} parse_ms={:.1} sparql_calls={}",
        metrics.network_ms, metrics.parse_ms, metrics.sparql_calls
    );
    crate::utils::logging::log_info_evt("search", "summary", "done", Some(&details));

    if total_ms >= 5000.0 {
        crate::utils::logging::log_warn_evt("search", "summary", "slow_query", Some(&details));
    }
}

pub fn set_signal_if_changed<T: PartialEq + 'static>(mut signal: Signal<T>, next: T) {
    if *signal.peek() != next {
        *signal.write() = next;
    }
}
