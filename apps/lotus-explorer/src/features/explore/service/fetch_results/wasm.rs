use super::{FetchResult, PlannedResultsFetch};
use crate::features::explore::search_metrics::SearchMetrics;
use crate::features::explore::types::{DomainError, ParseFault, QueryStage};
use crate::perf;
use crate::queries;
use crate::repositories::RepositoryError;
use crate::repositories::LotusRepository;
use crate::services::search_telemetry as telemetry;
use crate::sparql;
use futures::try_join;
use shared::sparql::ResponseBody;

pub(super) async fn fetch_results<R: LotusRepository>(
    repo: &R,
    plan: &PlannedResultsFetch<'_>,
    metrics: &mut SearchMetrics,
    on_processing: &impl Fn(),
) -> Result<FetchResult, DomainError> {
    let count_query = queries::query_counts_from_base(plan.execution_query);
    let results_query = queries::query_with_limit(plan.execution_query, plan.display_limit);

    let count_timer = perf::start_timer("LOTUS:results_count_query");
    let results_timer = perf::start_timer("LOTUS:results_page_query");
    let (counts_csv, results_csv): (ResponseBody, ResponseBody) = try_join!(
        async {
            repo.sparql_body(&count_query)
                .await
                .map_err(DomainError::transport_at(QueryStage::ResultsQuery))
        },
        async {
            repo.sparql_body(&results_query)
                .await
                .map_err(DomainError::transport_at(QueryStage::ResultsQuery))
        },
    )?;
    let count_elapsed = perf::end_timer("LOTUS:results_count_query", count_timer);
    let results_elapsed = perf::end_timer("LOTUS:results_page_query", results_timer);
    let overlapped_network_elapsed = count_elapsed.max(results_elapsed);
    metrics.add_parallel_network(overlapped_network_elapsed, 2);

    on_processing();

    let count_parse_timer = perf::start_timer("LOTUS:results_count_parse");
    let total_stats =
        sparql::parse_counts_csv_bytes(&counts_csv).map_err(results_csv_parse_error)?;
    let count_parse_elapsed = perf::end_timer("LOTUS:results_count_parse", count_parse_timer);
    metrics.add_parse(count_parse_elapsed);

    let results_parse_timer = perf::start_timer("LOTUS:results_page_parse");
    let rows = sparql::parse_compounds_csv_display_bytes(&results_csv, plan.display_limit)
        .map_err(results_csv_parse_error)?;
    let results_parse_elapsed = perf::end_timer("LOTUS:results_page_parse", results_parse_timer);
    metrics.add_parse(results_parse_elapsed);

    let total_matches = total_stats.n_entries;
    let display_capped_rows = total_matches > rows.len();
    telemetry::results_fetch_done(
        overlapped_network_elapsed
            .saturating_add(count_parse_elapsed)
            .saturating_add(results_parse_elapsed),
        rows.len(),
        total_matches,
    );

    Ok(FetchResult {
        rows,
        total_stats: Some(total_stats),
        total_matches: Some(total_matches),
        display_capped_rows,
    })
}

pub(super) fn is_probable_memory_limit(err: &DomainError) -> bool {
    fn has_memory_signature(msg: &str) -> bool {
        let m = msg.to_ascii_lowercase();
        m.contains("out of memory")
            || m.contains("memory")
            || m.contains("too large")
            || m.contains("allocation")
            || m.contains("capacity")
    }

    match err {
        DomainError::Transport { source, .. } => match source {
            RepositoryError::NotConfigured => false,
            RepositoryError::Network(detail) => has_memory_signature(detail.as_str()),
            RepositoryError::Http { body, .. } => has_memory_signature(body),
            RepositoryError::Parse(detail) => has_memory_signature(detail.as_str()),
        },
        DomainError::Parse(ParseFault::ResultsCsv { details }) => has_memory_signature(details),
        _ => false,
    }
}

fn results_csv_parse_error(err: impl std::fmt::Display) -> DomainError {
    DomainError::Parse(ParseFault::ResultsCsv {
        details: err.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wasm_preview_rows_are_bounded() {
        let csv = shared::sparql::ResponseBody::from_static(
            b"compound,compoundLabel,taxon,ref_qid\nQ1,One,Q10,Q20\nQ2,Two,Q11,Q21\n",
        );

        let rows = sparql::parse_compounds_csv_display_bytes(&csv, 1).expect("display parse");
        assert_eq!(rows.len(), 1);
    }
}
