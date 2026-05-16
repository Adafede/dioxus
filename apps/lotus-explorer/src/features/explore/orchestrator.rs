// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Search orchestration — thin dispatcher that sequences service calls.
//!
//! `start_search` validates input, dispatches `SearchRequested`, then spawns
//! `do_search`.  `do_search` delegates all I/O and business logic to the
//! `service::` modules so that those remain independently testable.

use crate::features::explore::actions::ExploreAction;
use crate::features::explore::command::SearchCommand;
use crate::features::explore::executor::{SearchOutcome, do_search};
use crate::features::explore::lifecycle::SearchLifecycleCoordinator;
use crate::features::explore::request::SearchRequest;
use crate::features::explore::search_state::{ExploreState, dispatch_explore_action};
use crate::features::explore::service::finalize;
use crate::features::explore::types::DomainError;
use crate::models::SearchCriteria;
use crate::repositories::LotusRepository;
use crate::services::search_telemetry as telemetry;
use dioxus::core::Task;
use dioxus::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Default)]
pub struct SearchTaskController {
    in_flight: Rc<RefCell<Option<Task>>>,
}

impl SearchTaskController {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn replace_in_flight(&self, next: Task) {
        if let Some(prev) = self.in_flight.borrow_mut().replace(next) {
            prev.cancel();
            telemetry::search_inflight_cancelled();
        }
    }
}

fn validate_search_criteria(criteria: &SearchCriteria) -> Result<(), DomainError> {
    crate::features::explore::form_validation::validate_dispatch_criteria(criteria)
        .map_err(DomainError::Validation)
}

fn build_search_succeeded_action(request: &SearchRequest, outcome: SearchOutcome) -> ExploreAction {
    let SearchOutcome {
        rows,
        qid,
        warning,
        query,
        total_matches,
        total_stats,
        display_capped_rows,
    } = outcome;

    let meta = finalize::finalize(
        request.criteria(),
        qid.as_deref(),
        &rows,
        total_matches,
        total_stats,
        request.direct_download(),
    );

    ExploreAction::SearchSucceeded {
        rows,
        qid,
        warning,
        query,
        total_matches: meta.filtered_matches,
        total_stats: meta.filtered_stats,
        display_capped_rows,
        query_hash: meta.query_hash,
        result_hash: meta.result_hash,
        metadata_json: meta.metadata_json,
    }
}

/// Validate input, dispatch `SearchRequested`, then spawn `do_search`.
pub fn start_search<R: LotusRepository>(
    criteria: Signal<SearchCriteria>,
    command: SearchCommand,
    explore: Signal<ExploreState>,
    task_controller: SearchTaskController,
    repo: R,
) {
    let request = SearchRequest::new(criteria.peek().clone(), command);
    if let Err(error) = validate_search_criteria(request.criteria()) {
        dispatch_explore_action(explore, ExploreAction::SearchFailed { error });
        return;
    }

    dispatch_explore_action(explore, request.as_action());
    let request = request.with_request_token(explore.peek().lifecycle.search_request_token);
    let coordinator = SearchLifecycleCoordinator::new(explore);

    let task = spawn(async move {
        match do_search(&request, repo.clone(), |phase| coordinator.on_phase(phase)).await {
            Ok(outcome) => {
                coordinator.on_success(&request, build_search_succeeded_action(&request, outcome))
            }
            Err(e) => coordinator.on_error(&request, e),
        }
    });
    task_controller.replace_in_flight(task);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::explore::types::ValidationFault;

    #[test]
    fn build_search_succeeded_action_applies_finalized_counts() {
        let request = SearchRequest::new(SearchCriteria::default(), SearchCommand::Interactive);
        let outcome = SearchOutcome {
            rows: Vec::new(),
            qid: Some("Q42".to_string()),
            warning: None,
            query: "SELECT * WHERE {}".to_string(),
            total_matches: Some(7),
            total_stats: None,
            display_capped_rows: true,
        };

        let action = build_search_succeeded_action(&request, outcome);
        match action {
            ExploreAction::SearchSucceeded {
                total_matches,
                total_stats,
                display_capped_rows,
                ..
            } => {
                assert_eq!(total_matches, Some(7));
                assert!(total_stats.is_some());
                assert!(display_capped_rows);
            }
            _ => panic!("expected SearchSucceeded action"),
        }
    }

    #[test]
    fn validate_search_criteria_rejects_empty_input() {
        let criteria = SearchCriteria {
            taxon: " ".into(),
            smiles: "".into(),
            formula_enabled: false,
            ..SearchCriteria::default()
        };

        let result = validate_search_criteria(&criteria);
        assert_eq!(
            result,
            Err(DomainError::Validation(ValidationFault::EmptyInput))
        );
    }

    #[test]
    fn validate_search_criteria_accepts_formula_only_input() {
        let criteria = SearchCriteria {
            taxon: "".into(),
            smiles: "".into(),
            formula_enabled: true,
            ..SearchCriteria::default()
        };

        assert_eq!(validate_search_criteria(&criteria), Ok(()));
    }

    #[test]
    fn validate_search_criteria_maps_shared_mass_validation_fault() {
        let criteria = SearchCriteria {
            taxon: "Rosa".into(),
            mass_min: -1.0,
            ..SearchCriteria::default()
        };

        assert_eq!(
            validate_search_criteria(&criteria),
            Err(DomainError::Validation(ValidationFault::MassOutOfRange))
        );
    }
}
