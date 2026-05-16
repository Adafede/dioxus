// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! SPARQL-side results pipeline for Explore searches.
//!
//! Owns the non-API execution path after strategy selection:
//! taxon resolution, query construction, download-only short-circuit, and
//! full-results retrieval.

use super::{
    build_query::{apply_server_filters, build_sparql_query},
    fetch_results, resolve_taxon,
};
use crate::features::explore::request::SearchRequest;
use crate::features::explore::search_metrics::SearchMetrics;
use crate::features::explore::types::{DomainError, QueryPhase, TaxonWarning};
use crate::models::{CompoundEntry, DatasetStats};
use crate::repositories::LotusRepository;
use shared::lotus::models::runtime_table_row_limit;

#[derive(Debug)]
pub struct ResultsPipelineOutcome {
    pub rows: Vec<CompoundEntry>,
    pub qid: Option<String>,
    pub warning: Option<TaxonWarning>,
    pub query: String,
    pub total_matches: Option<usize>,
    pub total_stats: Option<DatasetStats>,
    pub display_capped_rows: bool,
}

pub async fn execute<R: LotusRepository>(
    request: &SearchRequest,
    normalized_smiles: &str,
    repo: &R,
    metrics: &mut SearchMetrics,
    on_phase: impl Fn(QueryPhase),
    direct_download_mode: bool,
) -> Result<ResultsPipelineOutcome, DomainError> {
    if resolve_taxon::requires_remote_lookup(request.criteria().taxon.trim()) {
        on_phase(QueryPhase::ResolvingTaxon);
    }

    let taxon_resolution =
        resolve_taxon::resolve(request.criteria().taxon.trim(), repo, metrics).await?;

    let sparql_query = build_sparql_query(
        normalized_smiles,
        request.criteria(),
        taxon_resolution.qid.as_deref(),
    );
    let execution_query = apply_server_filters(&sparql_query, request.criteria());

    if direct_download_mode {
        return Ok(ResultsPipelineOutcome {
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
    let fetch_result = fetch_results::fetch(
        &execution_query,
        display_limit,
        repo,
        metrics,
        || on_phase(QueryPhase::FetchingResults),
        || on_phase(QueryPhase::ProcessingResults),
    )
    .await?;

    Ok(ResultsPipelineOutcome {
        rows: fetch_result.rows,
        qid: taxon_resolution.qid,
        warning: taxon_resolution.warning,
        query: execution_query,
        total_matches: fetch_result.total_matches,
        total_stats: fetch_result.total_stats,
        display_capped_rows: fetch_result.display_capped_rows,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::explore::command::SearchCommand;
    use crate::features::explore::request::SearchRequest;
    use crate::models::SearchCriteria;
    use crate::repositories::mock::MockRepository;

    #[test]
    fn download_only_builds_query_without_fetching_results() {
        futures::executor::block_on(async {
            let request = SearchRequest::new(
                SearchCriteria {
                    taxon: String::new(),
                    smiles: String::new(),
                    ..SearchCriteria::default()
                },
                SearchCommand::StartupDownload,
            );
            let repo = MockRepository::sparql_error("should not fetch rows");
            let mut metrics = SearchMetrics::default();

            let outcome = execute(&request, "", &repo, &mut metrics, |_| {}, true)
                .await
                .expect("download-only should not hit results fetch");

            assert!(outcome.rows.is_empty());
            assert!(outcome.total_matches.is_none());
            assert!(outcome.query.contains("SELECT"));
        });
    }

    #[test]
    fn interactive_pipeline_fetches_rows_and_counts() {
        futures::executor::block_on(async {
            let request = SearchRequest::new(
                SearchCriteria {
                    taxon: String::new(),
                    smiles: String::new(),
                    ..SearchCriteria::default()
                },
                SearchCommand::Interactive,
            );
            let repo = MockRepository::sparql_only(
                b"compound,compoundLabel,taxon,ref_qid\nQ1,One,Q10,Q20\nQ2,Two,Q11,Q21\n".to_vec(),
            );
            let mut metrics = SearchMetrics::default();

            let outcome = execute(&request, "", &repo, &mut metrics, |_| {}, false)
                .await
                .expect("interactive pipeline should fetch results");

            assert_eq!(outcome.rows.len(), 2);
            assert_eq!(outcome.total_matches, Some(2));
            assert!(outcome.total_stats.is_some());
        });
    }
}
