// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::features::explore::actions::ExploreAction;
use crate::features::explore::error_recovery_coordinator::should_clear_state_on_error;
use crate::features::explore::types::QueryPhase;
use crate::models::SortDir;
use std::sync::Arc;

use super::{ExploreState, ResultDataState};

pub fn reduce(mut state: ExploreState, action: ExploreAction) -> ExploreState {
    match action {
        ExploreAction::SearchRequested {
            criteria_snapshot,
            command,
        } => {
            state.lifecycle.loading = true;
            state.lifecycle.error = None;
            state.lifecycle.query_phase = QueryPhase::PreparingQuery;
            state.lifecycle.searched_once = true;
            state.lifecycle.download_only_mode = command.direct_download();
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
            if should_clear_state_on_error(error.query_stage()) {
                state.result = ResultDataState::default();
            }
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
                state.result.sort.dir = if state.result.sort.dir == SortDir::Asc {
                    SortDir::Desc
                } else {
                    SortDir::Asc
                };
            } else {
                state.result.sort.col = column;
                state.result.sort.dir = SortDir::Asc;
            }
        }
    }
    state
}
