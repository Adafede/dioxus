// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

mod lifecycle;
mod result_data;
mod ui;

use super::ExploreState;
use crate::features::explore::actions::ExploreAction;

#[cfg(test)]
pub fn reduce(mut state: ExploreState, action: ExploreAction) -> ExploreState {
    reduce_mut(&mut state, action);
    state
}

pub fn reduce_mut(state: &mut ExploreState, action: ExploreAction) {
    match action {
        ExploreAction::SearchRequested {
            criteria_snapshot,
            command,
        } => {
            lifecycle::search_requested(&mut state.lifecycle, command);
            result_data::reset_for_new_search(&mut state.result);
            ui::search_requested(&mut state.ui, criteria_snapshot);
        }
        ExploreAction::SearchPhaseChanged(phase) => {
            lifecycle::phase_changed(&mut state.lifecycle, phase);
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
            lifecycle::search_finished(&mut state.lifecycle);
            result_data::search_succeeded(
                &mut state.result,
                result_data::SearchSuccessPayload {
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
                },
            );
        }
        ExploreAction::SearchFailed { error } => {
            if lifecycle::search_failed(&mut state.lifecycle, &error) {
                result_data::clear(&mut state.result);
            }
            lifecycle::record_error(&mut state.lifecycle, error);
        }
        ExploreAction::ErrorDismissed => {
            lifecycle::dismiss_error(&mut state.lifecycle);
        }
        ExploreAction::MobileFiltersToggled => {
            ui::toggle_mobile_filters(&mut state.ui);
        }
        ExploreAction::DownloadDispatchStarted => {
            lifecycle::download_dispatch_started(&mut state.lifecycle);
        }
        ExploreAction::DownloadDispatchFinished => {
            lifecycle::download_dispatch_finished(&mut state.lifecycle);
        }
        ExploreAction::SortToggled(column) => {
            result_data::sort_toggled(&mut state.result, column);
        }
    }
}
