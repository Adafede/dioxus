// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Explore feature state, reducer, and dispatch helpers.
//!
//! [`ExploreState`] is split into three focused sub-stores so that
//! `use_memo`-based selectors (see [`super::selectors`]) can provide
//! narrow reactive subscriptions.  Only the coarsest component
//! subscriptions need to observe the whole state.

use crate::features::explore::actions::ExploreAction;
use crate::features::explore::types::{DomainError, QueryPhase, TaxonWarning};
use crate::models::{CompoundEntry, DatasetStats, Rows, SearchCriteria, SortState};
use dioxus::prelude::*;
use std::sync::Arc;

// ── Sub-store A: Search lifecycle ─────────────────────────────────────────────

/// Lifecycle-related fields: loading flag, current error, phase indicator,
/// and bookkeeping tokens.  Changes here should re-render loading overlays
/// and error notices.
#[derive(Clone, PartialEq)]
pub struct SearchLifecycleState {
    pub loading: bool,
    pub error: Option<DomainError>,
    pub query_phase: QueryPhase,
    pub searched_once: bool,
    pub download_only_mode: bool,
    pub download_dispatching: bool,
    pub search_request_token: u64,
}

impl Default for SearchLifecycleState {
    fn default() -> Self {
        Self {
            loading: false,
            error: None,
            query_phase: QueryPhase::Idle,
            searched_once: false,
            download_only_mode: false,
            download_dispatching: false,
            search_request_token: 0,
        }
    }
}

// ── Sub-store B: Result data ──────────────────────────────────────────────────

/// Result payload and presentation state.  Changes here re-render the results
/// table, toolbar, header-meta row, and taxon notice.
#[derive(Clone, PartialEq)]
pub struct ResultDataState {
    pub entries: Rows,
    pub taxon_notice: Option<TaxonWarning>,
    pub resolved_qid: Option<String>,
    pub query_hash: Option<String>,
    pub result_hash: Option<String>,
    pub sparql_query: Option<Arc<str>>,
    pub metadata_json: Option<Arc<str>>,
    pub total_matches: Option<usize>,
    pub total_stats: Option<DatasetStats>,
    pub display_capped_rows: bool,
    pub sort: SortState,
}

impl Default for ResultDataState {
    fn default() -> Self {
        Self {
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
            sort: SortState::default(),
        }
    }
}

// ── Sub-store C: UI chrome ────────────────────────────────────────────────────

/// UI chrome and the last-executed criteria snapshot.  Changes here re-render
/// the sidebar / mobile-filter overlay and the query toolbar.
#[derive(Clone, PartialEq, Default)]
pub struct UiChromeState {
    pub executed_criteria: SearchCriteria,
    pub mobile_filters_open: bool,
}

// ── Composite state ───────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Default)]
pub struct ExploreState {
    pub lifecycle: SearchLifecycleState,
    pub result: ResultDataState,
    pub ui: UiChromeState,
}

// ── Reducer ───────────────────────────────────────────────────────────────────

pub fn reduce(mut state: ExploreState, action: ExploreAction) -> ExploreState {
    match action {
        ExploreAction::SearchRequested {
            criteria_snapshot,
            direct_download,
        } => {
            state.lifecycle.loading = true;
            state.lifecycle.error = None;
            state.lifecycle.query_phase = QueryPhase::ResolvingTaxon;
            state.lifecycle.searched_once = true;
            state.lifecycle.download_only_mode = direct_download;
            state.lifecycle.download_dispatching = false;
            state.lifecycle.search_request_token =
                state.lifecycle.search_request_token.saturating_add(1);
            state.result = ResultDataState::default();
            state.ui.executed_criteria = criteria_snapshot;
            state.ui.mobile_filters_open = false;
        }
        ExploreAction::SearchPhaseChanged(phase) => {
            state.lifecycle.query_phase = phase;
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
            state.lifecycle.loading = false;
            state.lifecycle.error = None;
            state.lifecycle.query_phase = QueryPhase::Idle;
            state.lifecycle.download_dispatching = false;
            state.result.entries = Arc::from(rows.into_boxed_slice());
            state.result.taxon_notice = warning;
            state.result.resolved_qid = qid;
            state.result.query_hash = Some(query_hash);
            state.result.result_hash = Some(result_hash);
            state.result.sparql_query = Some(Arc::<str>::from(query));
            state.result.metadata_json = Some(metadata_json);
            state.result.total_matches = total_matches;
            state.result.total_stats = total_stats;
            state.result.display_capped_rows = display_capped_rows;
        }
        ExploreAction::SearchFailed { error } => {
            state.lifecycle.loading = false;
            state.lifecycle.error = Some(error);
            state.lifecycle.query_phase = QueryPhase::Idle;
            state.lifecycle.download_dispatching = false;
        }
        ExploreAction::ErrorDismissed => {
            state.lifecycle.error = None;
        }
        ExploreAction::MobileFiltersToggled => {
            state.ui.mobile_filters_open = !state.ui.mobile_filters_open;
        }
        ExploreAction::DownloadDispatchStarted => {
            state.lifecycle.download_dispatching = true;
        }
        ExploreAction::DownloadDispatchFinished => {
            state.lifecycle.download_dispatching = false;
        }
        ExploreAction::SortToggled(column) => {
            if state.result.sort.col == column {
                state.result.sort.dir = if state.result.sort.dir == crate::models::SortDir::Asc {
                    crate::models::SortDir::Desc
                } else {
                    crate::models::SortDir::Asc
                };
            } else {
                state.result.sort.col = column;
                state.result.sort.dir = crate::models::SortDir::Asc;
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

// ── Metrics ───────────────────────────────────────────────────────────────────

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

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::explore::types::{DomainError, ValidationFault};
    use crate::models::{SearchCriteria, SortColumn, SortDir};
    use std::sync::Arc;

    fn default_state() -> ExploreState {
        ExploreState::default()
    }

    // ── SearchRequested ───────────────────────────────────────────────────────

    #[test]
    fn search_requested_sets_loading_and_clears_result() {
        let state = default_state();
        let next = reduce(
            state,
            ExploreAction::SearchRequested {
                criteria_snapshot: SearchCriteria::default(),
                direct_download: false,
            },
        );
        assert!(next.lifecycle.loading);
        assert!(next.lifecycle.error.is_none());
        assert_eq!(next.lifecycle.query_phase, QueryPhase::ResolvingTaxon);
        assert!(next.lifecycle.searched_once);
        assert!(!next.lifecycle.download_only_mode);
        assert_eq!(next.lifecycle.search_request_token, 1);
        assert!(next.result.entries.is_empty());
        assert!(next.result.sparql_query.is_none());
    }

    #[test]
    fn search_requested_direct_download_flag_propagates() {
        let state = default_state();
        let next = reduce(
            state,
            ExploreAction::SearchRequested {
                criteria_snapshot: SearchCriteria::default(),
                direct_download: true,
            },
        );
        assert!(next.lifecycle.download_only_mode);
    }

    #[test]
    fn search_requested_increments_request_token() {
        let mut state = default_state();
        for expected in 1u64..=3 {
            state = reduce(
                state,
                ExploreAction::SearchRequested {
                    criteria_snapshot: SearchCriteria::default(),
                    direct_download: false,
                },
            );
            assert_eq!(state.lifecycle.search_request_token, expected);
        }
    }

    // ── SearchPhaseChanged ────────────────────────────────────────────────────

    #[test]
    fn phase_changed_updates_only_phase() {
        let mut state = default_state();
        state.lifecycle.loading = true;
        let next = reduce(
            state,
            ExploreAction::SearchPhaseChanged(QueryPhase::Counting),
        );
        assert_eq!(next.lifecycle.query_phase, QueryPhase::Counting);
        assert!(next.lifecycle.loading, "loading must be untouched");
    }

    // ── SearchSucceeded ───────────────────────────────────────────────────────

    #[test]
    fn search_succeeded_clears_loading_and_stores_result() {
        let mut state = default_state();
        state.lifecycle.loading = true;
        let rows: Vec<CompoundEntry> = vec![];
        let next = reduce(
            state,
            ExploreAction::SearchSucceeded {
                rows,
                qid: Some("Q123".into()),
                warning: None,
                query: "SELECT ?x WHERE {}".into(),
                total_matches: Some(42),
                total_stats: None,
                display_capped_rows: true,
                query_hash: "qh".into(),
                result_hash: "rh".into(),
                metadata_json: Arc::<str>::from("{}"),
            },
        );
        assert!(!next.lifecycle.loading);
        assert_eq!(next.result.resolved_qid.as_deref(), Some("Q123"));
        assert_eq!(next.result.total_matches, Some(42));
        assert!(next.result.display_capped_rows);
        assert_eq!(next.result.query_hash.as_deref(), Some("qh"));
        assert_eq!(next.result.result_hash.as_deref(), Some("rh"));
    }

    // ── SearchFailed ─────────────────────────────────────────────────────────

    #[test]
    fn search_failed_stores_domain_error_and_clears_loading() {
        let mut state = default_state();
        state.lifecycle.loading = true;
        let err = DomainError::Validation(ValidationFault::EmptyInput);
        let next = reduce(state, ExploreAction::SearchFailed { error: err.clone() });
        assert!(!next.lifecycle.loading);
        assert_eq!(next.lifecycle.error, Some(err));
        assert_eq!(next.lifecycle.query_phase, QueryPhase::Idle);
    }

    // ── ErrorDismissed ────────────────────────────────────────────────────────

    #[test]
    fn error_dismissed_clears_error_only() {
        let mut state = default_state();
        state.lifecycle.error = Some(DomainError::Validation(ValidationFault::EmptyInput));
        state.lifecycle.loading = true; // should not be touched
        let next = reduce(state, ExploreAction::ErrorDismissed);
        assert!(next.lifecycle.error.is_none());
        assert!(next.lifecycle.loading, "loading must be untouched");
    }

    // ── MobileFiltersToggled ──────────────────────────────────────────────────

    #[test]
    fn mobile_filters_toggled_flips_flag() {
        let state = default_state();
        assert!(!state.ui.mobile_filters_open);
        let next = reduce(state, ExploreAction::MobileFiltersToggled);
        assert!(next.ui.mobile_filters_open);
        let next2 = reduce(next, ExploreAction::MobileFiltersToggled);
        assert!(!next2.ui.mobile_filters_open);
    }

    // ── DownloadDispatch ──────────────────────────────────────────────────────

    #[test]
    fn download_dispatch_start_stop_round_trip() {
        let state = default_state();
        let next = reduce(state, ExploreAction::DownloadDispatchStarted);
        assert!(next.lifecycle.download_dispatching);
        let next2 = reduce(next, ExploreAction::DownloadDispatchFinished);
        assert!(!next2.lifecycle.download_dispatching);
    }

    // ── SortToggled ───────────────────────────────────────────────────────────

    #[test]
    fn sort_toggled_same_column_reverses_direction() {
        let mut state = default_state();
        state.result.sort.col = SortColumn::Name;
        state.result.sort.dir = SortDir::Asc;
        let next = reduce(state, ExploreAction::SortToggled(SortColumn::Name));
        assert_eq!(next.result.sort.dir, SortDir::Desc);
        let next2 = reduce(next, ExploreAction::SortToggled(SortColumn::Name));
        assert_eq!(next2.result.sort.dir, SortDir::Asc);
    }

    #[test]
    fn sort_toggled_new_column_resets_to_asc() {
        let mut state = default_state();
        state.result.sort.col = SortColumn::Name;
        state.result.sort.dir = SortDir::Desc;
        let next = reduce(state, ExploreAction::SortToggled(SortColumn::Mass));
        assert_eq!(next.result.sort.col, SortColumn::Mass);
        assert_eq!(next.result.sort.dir, SortDir::Asc);
    }

    // ── dispatch_explore_action deduplicates writes ───────────────────────────

    #[test]
    fn dispatch_no_op_does_not_change_state() {
        // SearchPhaseChanged(Idle) on default state → phase is already Idle
        let state = default_state();
        assert_eq!(state.lifecycle.query_phase, QueryPhase::Idle);
        // reduce returns identical value → dispatch would skip the write
        let next = reduce(
            state.clone(),
            ExploreAction::SearchPhaseChanged(QueryPhase::Idle),
        );
        assert_eq!(next.lifecycle.query_phase, state.lifecycle.query_phase);
    }
}
