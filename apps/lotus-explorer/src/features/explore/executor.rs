// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Search execution pipeline for the Explore feature.
//!
//! This module runs the API/SPARQL workflow and emits phase callbacks, but does
//! not mutate Dioxus state directly.

use crate::features::explore::request::SearchRequest;
use crate::features::explore::search_state::{SearchMetrics, emit_search_summary};
use crate::features::explore::service::{
    build_query::{apply_server_filters, build_sparql_query, normalize_smiles},
    fetch_preview, resolve_taxon,
    strategy::ExecutionStrategy,
};
use crate::features::explore::types::{DomainError, QueryPhase};
use crate::models::{CompoundEntry, DatasetStats};
use crate::perf;
use crate::repositories::LotusRepository;
use crate::services::search_telemetry as telemetry;
use shared::lotus::models::runtime_table_row_limit;

/// The raw outcome from a completed search execution.
pub struct SearchOutcome {
    pub rows: Vec<CompoundEntry>,
    pub qid: Option<String>,
    pub warning: Option<crate::features::explore::types::TaxonWarning>,
    pub query: String,
    pub total_matches: Option<usize>,
    pub total_stats: Option<DatasetStats>,
    pub display_capped_rows: bool,
}

pub struct SearchExecutor<R, P>
where
    R: LotusRepository,
    P: Fn(QueryPhase),
{
    repo: R,
    on_phase: P,
}

impl<R, P> SearchExecutor<R, P>
where
    R: LotusRepository,
    P: Fn(QueryPhase),
{
    #[must_use]
    pub fn new(repo: R, on_phase: P) -> Self {
        Self { repo, on_phase }
    }

    pub async fn execute(&self, request: &SearchRequest) -> Result<SearchOutcome, DomainError> {
        let search_timer = perf::start_timer("LOTUS:search_total");
        let mut metrics = SearchMetrics::default();
        telemetry::search_start();

        let strategy = ExecutionStrategy::resolve(request.direct_download());
        let smiles = normalize_smiles(&request.criteria().smiles);

        // API fast path.
        if strategy == ExecutionStrategy::ApiFirst {
            let mut api_crit = request.criteria().clone();
            api_crit.smiles = smiles.clone();
            let display_limit = runtime_table_row_limit();
            let include_counts = true;
            let api_timer = perf::start_timer("LOTUS:api_search");
            match self
                .repo
                .api_search(&api_crit, display_limit, include_counts)
                .await
            {
                None => {
                    telemetry::api_path_not_available("reason=not_configured");
                }
                Some(Ok(response)) => {
                    let api_elapsed = perf::end_timer("LOTUS:api_search", api_timer);
                    metrics.add_network(api_elapsed);
                    telemetry::api_success(
                        api_elapsed,
                        response.rows.len(),
                        response.total_matches,
                    );
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

        // SPARQL pipeline.
        let taxon_resolution =
            resolve_taxon::resolve(request.criteria().taxon.trim(), &self.repo, &mut metrics)
                .await?;

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
        (self.on_phase)(QueryPhase::Counting);

        let fetch_result = fetch_preview::fetch(
            &execution_query,
            display_limit,
            &self.repo,
            &mut metrics,
            || (self.on_phase)(QueryPhase::FetchingPreview),
        )
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
}

/// Execute the full search pipeline; returns a [`SearchOutcome`] or a
/// [`DomainError`]. No locale strings are produced here.
pub async fn do_search<R: LotusRepository>(
    request: &SearchRequest,
    repo: R,
    on_phase: impl Fn(QueryPhase),
) -> Result<SearchOutcome, DomainError> {
    SearchExecutor::new(repo, on_phase).execute(request).await
}
