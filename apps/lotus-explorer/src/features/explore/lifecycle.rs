// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Lifecycle coordination for explore search execution.
//!
//! Keeps dispatch policy (phase updates, stale-token suppression, success/error
//! transitions) in one module so orchestration stays focused on request flow.

use crate::features::explore::actions::ExploreAction;
use crate::features::explore::request::SearchRequest;
use crate::features::explore::search_state::{ExploreState, dispatch_explore_action};
use crate::features::explore::types::{DomainError, QueryPhase};
use crate::services::search_telemetry as telemetry;
use dioxus::prelude::*;

#[derive(Clone, Copy)]
pub struct SearchLifecycleCoordinator {
    explore: Signal<ExploreState>,
}

impl SearchLifecycleCoordinator {
    #[must_use]
    pub fn new(explore: Signal<ExploreState>) -> Self {
        Self { explore }
    }

    pub fn on_phase(&self, phase: QueryPhase) {
        dispatch_explore_action(self.explore, ExploreAction::SearchPhaseChanged(phase));
    }

    pub fn on_success(&self, request: &SearchRequest, success_action: ExploreAction) {
        if is_stale_token(request.request_token(), self.current_token()) {
            telemetry::stale_result_ignored(request.request_token());
            return;
        }

        for action in success_transition_actions(success_action) {
            dispatch_explore_action(self.explore, action);
        }
    }

    pub fn on_error(&self, request: &SearchRequest, error: DomainError) {
        if is_stale_token(request.request_token(), self.current_token()) {
            telemetry::stale_error_ignored(request.request_token());
            return;
        }
        dispatch_explore_action(self.explore, ExploreAction::SearchFailed { error });
    }

    fn current_token(&self) -> u64 {
        self.explore.peek().lifecycle.search_request_token
    }
}

#[must_use]
pub fn is_stale_token(request_token: u64, current_token: u64) -> bool {
    request_token != current_token
}

#[must_use]
pub fn success_transition_actions(success_action: ExploreAction) -> [ExploreAction; 2] {
    [
        ExploreAction::SearchPhaseChanged(QueryPhase::Rendering),
        success_action,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_token_detection_requires_exact_match() {
        assert!(!is_stale_token(4, 4));
        assert!(is_stale_token(4, 5));
    }

    #[test]
    fn success_actions_emit_rendering_before_result_commit() {
        let actions = success_transition_actions(ExploreAction::ErrorDismissed);
        assert!(matches!(
            actions[0],
            ExploreAction::SearchPhaseChanged(QueryPhase::Rendering)
        ));
        assert!(matches!(actions[1], ExploreAction::ErrorDismissed));
    }
}


