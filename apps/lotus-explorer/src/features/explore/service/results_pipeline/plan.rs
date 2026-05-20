use super::ResultsPipelineOutcome;
use crate::features::explore::request::SearchRequest;
use crate::features::explore::search_metrics::SearchMetrics;
use crate::features::explore::service::{
    build_query::{apply_server_filters, build_sparql_query},
    fetch_results::FetchResult,
    resolve_taxon::{self, TaxonResolution},
};
use crate::features::explore::types::{DomainError, QueryPhase};
use crate::repositories::LotusRepository;

pub(super) struct ResultsExecutionPlan {
    taxon_resolution: TaxonResolution,
    execution_query: String,
}

impl ResultsExecutionPlan {
    pub(super) fn execution_query(&self) -> &str {
        &self.execution_query
    }

    pub(super) fn into_download_only_outcome(self) -> ResultsPipelineOutcome {
        ResultsPipelineOutcome {
            rows: Vec::new(),
            qid: self.taxon_resolution.qid,
            warning: self.taxon_resolution.warning,
            query: self.execution_query,
            total_matches: None,
            total_stats: None,
            display_capped_rows: false,
        }
    }

    pub(super) fn into_interactive_outcome(
        self,
        fetch_result: FetchResult,
    ) -> ResultsPipelineOutcome {
        ResultsPipelineOutcome {
            rows: fetch_result.rows,
            qid: self.taxon_resolution.qid,
            warning: self.taxon_resolution.warning,
            query: self.execution_query,
            total_matches: fetch_result.total_matches,
            total_stats: fetch_result.total_stats,
            display_capped_rows: fetch_result.display_capped_rows,
        }
    }
}

pub(super) async fn build_execution_plan<R: LotusRepository>(
    request: &SearchRequest,
    normalized_smiles: &str,
    repo: &R,
    metrics: &mut SearchMetrics,
    on_phase: &impl Fn(QueryPhase),
) -> Result<ResultsExecutionPlan, DomainError> {
    let taxon = request.criteria().taxon.trim();
    if resolve_taxon::requires_remote_lookup(taxon) {
        on_phase(QueryPhase::ResolvingTaxon);
    }

    let taxon_resolution = resolve_taxon::resolve(taxon, repo, metrics).await?;
    let sparql_query = build_sparql_query(
        normalized_smiles,
        request.criteria(),
        taxon_resolution.qid.as_deref(),
    );
    let execution_query = apply_server_filters(&sparql_query, request.criteria());

    Ok(ResultsExecutionPlan {
        taxon_resolution,
        execution_query,
    })
}
