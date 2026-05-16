// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Search orchestration — thin dispatcher that sequences service calls.
//!
//! `start_search` validates input, dispatches `SearchRequested`, then spawns
//! `do_search`.  `do_search` delegates all I/O and business logic to the
//! `service::` modules so that those remain independently testable.

use crate::features::explore::actions::ExploreAction;
use crate::features::explore::command::SearchCommand;
use crate::features::explore::request::SearchRequest;
use crate::features::explore::search_state::{
    ExploreState, SearchMetrics, dispatch_explore_action, emit_search_summary,
};
use crate::features::explore::service::{
    build_query::{apply_server_filters, build_sparql_query, normalize_smiles},
    fetch_preview, finalize, resolve_taxon,
    strategy::ExecutionStrategy,
};
use crate::features::explore::types::{DomainError, QueryPhase, ValidationFault};
use crate::models::{CompoundEntry, DatasetStats, SearchCriteria};
use crate::perf;
use crate::repositories::LotusRepository;
use crate::services::search_telemetry as telemetry;
use dioxus::core::Task;
use dioxus::prelude::*;
use shared::lotus::models::runtime_table_row_limit;
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

/// The raw outcome from a completed `do_search`.
pub struct SearchOutcome {
    pub rows: Vec<CompoundEntry>,
    pub qid: Option<String>,
    pub warning: Option<crate::features::explore::types::TaxonWarning>,
    pub query: String,
    pub total_matches: Option<usize>,
    pub total_stats: Option<DatasetStats>,
    pub display_capped_rows: bool,
}

fn validate_search_criteria(criteria: &SearchCriteria) -> Result<(), DomainError> {
    if criteria.taxon.trim().is_empty()
        && criteria.smiles.trim().is_empty()
        && !criteria.formula_enabled
    {
        return Err(DomainError::Validation(ValidationFault::EmptyInput));
    }
    Ok(())
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

    let task = spawn(async move {
        match do_search(&request, explore, repo.clone()).await {
            Ok(outcome) => {
                if request.request_token() != explore.peek().lifecycle.search_request_token {
                    telemetry::stale_result_ignored(request.request_token());
                    return;
                }

                let meta = finalize::finalize(
                    request.criteria(),
                    outcome.qid.as_deref(),
                    &outcome.rows,
                    outcome.total_matches,
                    outcome.total_stats.clone(),
                    request.direct_download(),
                );

                dispatch_explore_action(
                    explore,
                    ExploreAction::SearchPhaseChanged(QueryPhase::Rendering),
                );
                dispatch_explore_action(
                    explore,
                    ExploreAction::SearchSucceeded {
                        rows: outcome.rows,
                        qid: outcome.qid,
                        warning: outcome.warning,
                        query: outcome.query,
                        total_matches: meta.filtered_matches,
                        total_stats: meta.filtered_stats,
                        display_capped_rows: outcome.display_capped_rows,
                        query_hash: meta.query_hash,
                        result_hash: meta.result_hash,
                        metadata_json: meta.metadata_json,
                    },
                );
            }
            Err(e) => {
                if request.request_token() != explore.peek().lifecycle.search_request_token {
                    telemetry::stale_error_ignored(request.request_token());
                    return;
                }
                dispatch_explore_action(explore, ExploreAction::SearchFailed { error: e });
            }
        }
    });
    task_controller.replace_in_flight(task);
}

/// Execute the full search pipeline; returns a [`SearchOutcome`] or a
/// [`DomainError`].  No locale strings are produced here — formatting happens
/// at the UI boundary in `components::layout::notices`.
pub async fn do_search<R: LotusRepository>(
    request: &SearchRequest,
    explore: Signal<ExploreState>,
    repo: R,
) -> Result<SearchOutcome, DomainError> {
    let search_timer = perf::start_timer("LOTUS:search_total");
    let mut metrics = SearchMetrics::default();
    telemetry::search_start();

    let strategy = ExecutionStrategy::resolve(request.direct_download());
    let smiles = normalize_smiles(&request.criteria().smiles);

    // ── API fast path ─────────────────────────────────────────────────────────
    if strategy == ExecutionStrategy::ApiFirst {
        let mut api_crit = request.criteria().clone();
        api_crit.smiles = smiles.clone();
        let display_limit = runtime_table_row_limit();
        let include_counts = true;
        let api_timer = perf::start_timer("LOTUS:api_search");
        match repo
            .api_search(&api_crit, display_limit, include_counts)
            .await
        {
            None => {
                telemetry::api_path_not_available("reason=not_configured");
            }
            Some(Ok(response)) => {
                let api_elapsed = perf::end_timer("LOTUS:api_search", api_timer);
                metrics.add_network(api_elapsed);
                telemetry::api_success(api_elapsed, response.rows.len(), response.total_matches);
                let display_capped_rows = if include_counts {
                    response.total_matches > response.rows.len()
                } else {
                    response.rows.len() >= display_limit
                };
                let rows = response
                    .rows
                    .into_iter()
                    .map(CompoundEntry::from)
                    .collect::<Vec<_>>();
                let warning = response
                    .warning
                    .map(crate::features::explore::types::TaxonWarning::ApiMessage);
                let total_elapsed = perf::end_timer("LOTUS:search_total", search_timer);
                emit_search_summary(total_elapsed, metrics);
                return Ok(SearchOutcome {
                    rows,
                    qid: response.resolved_taxon_qid,
                    warning,
                    query: response.query,
                    total_matches: Some(response.total_matches),
                    total_stats: Some(response.stats.into()),
                    display_capped_rows,
                });
            }
            Some(Err(err)) => {
                let api_elapsed = perf::end_timer("LOTUS:api_search", api_timer);
                telemetry::api_fallback_direct(api_elapsed, &err.to_string());
            }
        }
    } else {
        telemetry::api_path_not_available("reason=download_only_mode");
    }

    // ── SPARQL pipeline ───────────────────────────────────────────────────────

    let taxon_resolution =
        resolve_taxon::resolve(request.criteria().taxon.trim(), &repo, &mut metrics).await?;

    let sparql_query =
        build_sparql_query(&smiles, request.criteria(), taxon_resolution.qid.as_deref());
    let execution_query = apply_server_filters(&sparql_query, request.criteria());

    if strategy.is_download_only() {
        let total_elapsed = perf::end_timer("LOTUS:search_total", search_timer);
        telemetry::direct_download_ready(total_elapsed);
        emit_search_summary(total_elapsed, metrics);
        return Ok(SearchOutcome {
            rows: Vec::new(),
            qid: taxon_resolution.qid,
            warning: taxon_resolution.warning,
            query: execution_query,
            total_matches: None,
            total_stats: None,
            display_capped_rows: false,
        });
    }

    let display_limit = runtime_table_row_limit();
    dispatch_explore_action(
        explore,
        ExploreAction::SearchPhaseChanged(QueryPhase::Counting),
    );

    let fetch_result =
        fetch_preview::fetch(&execution_query, display_limit, &repo, &mut metrics, || {
            dispatch_explore_action(
                explore,
                ExploreAction::SearchPhaseChanged(QueryPhase::FetchingPreview),
            );
        })
        .await?;

    let outcome = SearchOutcome {
        rows: fetch_result.rows,
        qid: taxon_resolution.qid,
        warning: taxon_resolution.warning,
        query: execution_query,
        total_matches: fetch_result.total_matches,
        total_stats: fetch_result.total_stats,
        display_capped_rows: fetch_result.display_capped_rows,
    };

    let total_elapsed = perf::end_timer("LOTUS:search_total", search_timer);
    telemetry::search_complete(
        total_elapsed,
        outcome.rows.len(),
        outcome.total_matches.unwrap_or(outcome.rows.len()),
    );
    emit_search_summary(total_elapsed, metrics);
    Ok(outcome)
}
