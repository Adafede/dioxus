use super::{FetchResult, PlannedResultsFetch};
use crate::features::explore::search_metrics::SearchMetrics;
use crate::features::explore::types::{DomainError, ParseFault, QueryStage};
use crate::perf;
use crate::queries;
use crate::repositories::LotusRepository;
use crate::repositories::RepositoryError;
use crate::services::search_telemetry as telemetry;
use crate::sparql;

pub(super) async fn fetch_results<R: LotusRepository>(
    repo: &R,
    plan: &PlannedResultsFetch<'_>,
    metrics: &mut SearchMetrics,
    on_processing: &impl Fn(),
) -> Result<FetchResult, DomainError> {
    // A search issues exactly ONE Qlever POST: the display query. The total is
    // derived locally from the returned rows (`rows.len()`) — "kept the results
    // of the query and counted them", as it was before a separate COUNT query
    // was introduced.
    //
    // The previous COUNT (`query_counts_from_base`) re-ran the full base —
    // including REFERENCE_METADATA_OPTIONAL/PROPERTIES_OPTIONAL with NO LIMIT,
    // wrapped in COUNT(DISTINCT CONCAT(...)) over every matched triple. It was
    // the heavy request that drove Qlever's anonymous quota into a permanent
    // 429, and running it concurrently with the display query (`try_join!`)
    // created the burst that 429'd. Both the concurrent fan-out and the extra
    // POST are removed: there is now a single, sequential Qlever POST per search,
    // and the count is local (matching the native path, which derives totals
    // from the fetched CSV via `DatasetStats::from_entries(rows)`).
    let results_timer = perf::start_timer("LOTUS:results_page_query");
    let results_query = queries::query_with_limit(plan.execution_query, plan.display_limit);
    let results_csv = repo
        .sparql_body(&results_query)
        .await
        .map_err(DomainError::transport_at(QueryStage::ResultsQuery))?;
    let results_elapsed = perf::end_timer("LOTUS:results_page_query", results_timer);
    metrics.add_network(results_elapsed);

    on_processing();

    let results_parse_timer = perf::start_timer("LOTUS:results_page_parse");
    let rows = sparql::parse_compounds_csv_display_bytes(&results_csv, plan.display_limit)
        .map_err(results_csv_parse_error)?;
    let results_parse_elapsed = perf::end_timer("LOTUS:results_page_parse", results_parse_timer);
    metrics.add_parse(results_parse_elapsed);

    // Local count from rows already in hand — no second Qlever POST, so no extra
    // chance to trip the anonymous rate limit. This equals the true total for
    // taxa that fit in a single page (the common curation case).
    let total_matches = Some(rows.len());
    let display_capped_rows = rows.len() >= plan.display_limit;

    telemetry::results_fetch_done(
        results_elapsed.saturating_add(results_parse_elapsed),
        rows.len(),
        total_matches.unwrap_or(rows.len()),
    );

    Ok(FetchResult {
        rows,
        total_stats: None,
        total_matches,
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
        let csv = lotus::transport::ResponseBody::from_static(
            b"compound,compoundLabel,taxon,ref_qid\nQ1,One,Q10,Q20\nQ2,Two,Q11,Q21\n",
        );

        let rows = sparql::parse_compounds_csv_display_bytes(&csv, 1).expect("display parse");
        assert_eq!(rows.len(), 1);
    }
}
