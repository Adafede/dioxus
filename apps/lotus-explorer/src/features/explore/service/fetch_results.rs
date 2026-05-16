// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Full-results fetch service.
//!
//! This module is **Dioxus-free**: the phase-change callback (`on_fetching`)
//! is a plain `Fn()` closure so that tests can supply a no-op.

use crate::features::explore::search_state::SearchMetrics;
use crate::features::explore::types::{DomainError, ParseFault, QueryStage};
use crate::models::{CompoundEntry, DatasetStats};
use crate::perf;
#[cfg(target_arch = "wasm32")]
use crate::queries;
use crate::repositories::LotusRepository;
use crate::services::search_telemetry as telemetry;
use crate::sparql;
#[cfg(target_arch = "wasm32")]
use shared::sparql::ResponseBody;
#[cfg(not(target_arch = "wasm32"))]
use std::io::{BufReader, Seek};
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

/// Result of a successful full-results fetch.
pub struct FetchResult {
    pub rows: Vec<CompoundEntry>,
    pub total_stats: Option<DatasetStats>,
    pub total_matches: Option<usize>,
    pub display_capped_rows: bool,
}

struct PlannedResultsFetch<'a> {
    execution_query: &'a str,
    display_limit: usize,
}

#[cfg(not(target_arch = "wasm32"))]
struct FetchedResultsCsv {
    payload: FetchedResultsPayload,
    network_elapsed: Duration,
}

#[cfg(not(target_arch = "wasm32"))]
enum FetchedResultsPayload {
    TempFile(tempfile::NamedTempFile),
}

#[cfg(not(target_arch = "wasm32"))]
struct ProcessedResults {
    rows: Vec<CompoundEntry>,
    total_stats: DatasetStats,
    total_matches: usize,
    display_capped_rows: bool,
    parse_elapsed: Duration,
}

/// Fetch full results with a single query and cap rendered rows locally.
///
/// `on_fetching` is called before the network fetch begins and `on_processing`
/// before CSV parsing/stat aggregation; in tests pass `|| ()`.
#[allow(clippy::too_many_arguments)]
pub async fn fetch<R: LotusRepository>(
    execution_query: &str,
    display_limit: usize,
    repo: &R,
    metrics: &mut SearchMetrics,
    on_fetching: impl Fn(),
    on_processing: impl Fn(),
) -> Result<FetchResult, DomainError> {
    let plan = plan_full_results_fetch(execution_query, display_limit);
    on_fetching();
    telemetry::results_fetch_started(plan.display_limit);

    #[cfg(target_arch = "wasm32")]
    let result = fetch_results_wasm(repo, &plan, metrics, &on_processing).await;

    #[cfg(not(target_arch = "wasm32"))]
    let result = fetch_results_native(repo, &plan, metrics, &on_processing).await;

    match result {
        Ok(v) => Ok(v),
        Err(err) => {
            #[cfg(target_arch = "wasm32")]
            {
                // Keep previous wasm classification semantics for memory pressure errors.
                if is_probable_wasm_memory_limit(&err) {
                    return Err(DomainError::memory_limit(QueryStage::ResultsQuery));
                }
                Err(err)
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                Err(err)
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn is_probable_wasm_memory_limit(err: &DomainError) -> bool {
    fn has_memory_signature(msg: &str) -> bool {
        let m = msg.to_ascii_lowercase();
        m.contains("out of memory")
            || m.contains("memory")
            || m.contains("too large")
            || m.contains("allocation")
            || m.contains("capacity")
    }

    match err {
        DomainError::Transport { source, .. } => {
            let source_text = source.to_string();
            has_memory_signature(&source_text)
        }
        DomainError::Parse(ParseFault::ResultsCsv { details }) => has_memory_signature(details),
        _ => false,
    }
}

// ── Internal pipeline helpers ─────────────────────────────────────────────────

fn plan_full_results_fetch(execution_query: &str, display_limit: usize) -> PlannedResultsFetch<'_> {
    PlannedResultsFetch {
        execution_query,
        display_limit,
    }
}

#[cfg(not(target_arch = "wasm32"))]
async fn fetch_results_csv<R: LotusRepository>(
    repo: &R,
    plan: &PlannedResultsFetch<'_>,
) -> Result<FetchedResultsCsv, DomainError> {
    let results_timer = perf::start_timer("LOTUS:results_query");
    #[cfg(not(target_arch = "wasm32"))]
    let payload = FetchedResultsPayload::TempFile(
        repo.sparql_tempfile(plan.execution_query)
            .await
            .map_err(DomainError::transport_at(QueryStage::ResultsQuery))?,
    );
    #[cfg(target_arch = "wasm32")]
    let payload = FetchedResultsPayload::Body(
        repo.sparql_body(plan.execution_query)
            .await
            .map_err(DomainError::transport_at(QueryStage::ResultsQuery))?,
    );
    let network_elapsed = perf::end_timer("LOTUS:results_query", results_timer);
    Ok(FetchedResultsCsv {
        payload,
        network_elapsed,
    })
}

#[cfg(not(target_arch = "wasm32"))]
async fn fetch_results_native<R: LotusRepository>(
    repo: &R,
    plan: &PlannedResultsFetch<'_>,
    metrics: &mut SearchMetrics,
    on_processing: &impl Fn(),
) -> Result<FetchResult, DomainError> {
    let fetched = fetch_results_csv(repo, plan).await?;
    metrics.add_network(fetched.network_elapsed);
    on_processing();

    let processed = process_full_results_csv(fetched.payload, plan.display_limit)?;
    metrics.add_parse(processed.parse_elapsed);
    telemetry::results_fetch_done(
        fetched
            .network_elapsed
            .saturating_add(processed.parse_elapsed),
        processed.rows.len(),
        processed.total_matches,
    );

    Ok(FetchResult {
        rows: processed.rows,
        total_stats: Some(processed.total_stats),
        total_matches: Some(processed.total_matches),
        display_capped_rows: processed.display_capped_rows,
    })
}

#[cfg(target_arch = "wasm32")]
async fn fetch_results_wasm<R: LotusRepository>(
    repo: &R,
    plan: &PlannedResultsFetch<'_>,
    metrics: &mut SearchMetrics,
    on_processing: &impl Fn(),
) -> Result<FetchResult, DomainError> {
    let count_query = queries::query_counts_from_base(plan.execution_query);
    let results_query = queries::query_with_limit(plan.execution_query, plan.display_limit);

    let count_timer = perf::start_timer("LOTUS:results_count_query");
    let results_timer = perf::start_timer("LOTUS:results_page_query");
    let (counts_csv, results_csv): (ResponseBody, ResponseBody) = futures::try_join!(
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
    let total_stats = sparql::parse_counts_csv_bytes(&counts_csv).map_err(|e| {
        DomainError::Parse(ParseFault::ResultsCsv {
            details: e.to_string(),
        })
    })?;
    let count_parse_elapsed = perf::end_timer("LOTUS:results_count_parse", count_parse_timer);
    metrics.add_parse(count_parse_elapsed);

    let results_parse_timer = perf::start_timer("LOTUS:results_page_parse");
    let rows = sparql::parse_compounds_csv_display_bytes(&results_csv, plan.display_limit)
        .map_err(|e| {
            DomainError::Parse(ParseFault::ResultsCsv {
                details: e.to_string(),
            })
        })?;
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

#[cfg(not(target_arch = "wasm32"))]
fn process_full_results_csv(
    payload: FetchedResultsPayload,
    display_limit: usize,
) -> Result<ProcessedResults, DomainError> {
    let parse_timer = perf::start_timer("LOTUS:results_parse");
    let (rows, total_stats, parse_capped) = match payload {
        FetchedResultsPayload::TempFile(mut file) => {
            file.as_file_mut().rewind().map_err(|e| {
                DomainError::Parse(ParseFault::ResultsCsv {
                    details: format!("tempfile rewind failed: {e}"),
                })
            })?;
            sparql::parse_compounds_csv_capped_reader(
                BufReader::new(file.as_file_mut()),
                display_limit,
            )
        }
    }
    .map_err(|e| {
        DomainError::Parse(ParseFault::ResultsCsv {
            details: e.to_string(),
        })
    })?;
    let parse_elapsed = perf::end_timer("LOTUS:results_parse", parse_timer);

    let total_matches = total_stats.n_entries;
    let display_capped_rows = parse_capped || total_matches > rows.len();
    Ok(ProcessedResults {
        rows,
        total_stats,
        total_matches,
        display_capped_rows,
        parse_elapsed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn process_stage_marks_capped_and_preserves_full_counts() {
        let payload = {
            use std::io::Write;

            let mut file = tempfile::NamedTempFile::new().expect("tempfile create");
            file.write_all(
                b"compound,compoundLabel,taxon,ref_qid\nhttp://www.wikidata.org/entity/Q1,One,http://www.wikidata.org/entity/Q10,http://www.wikidata.org/entity/Q20\nhttp://www.wikidata.org/entity/Q2,Two,http://www.wikidata.org/entity/Q11,http://www.wikidata.org/entity/Q21\n",
            )
            .expect("tempfile write");
            FetchedResultsPayload::TempFile(file)
        };

        let processed = process_full_results_csv(payload, 1).expect("csv should parse");
        assert_eq!(processed.rows.len(), 1);
        assert_eq!(processed.total_matches, 2);
        assert_eq!(processed.total_stats.n_entries, 2);
        assert!(processed.display_capped_rows);
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn wasm_preview_rows_are_bounded() {
        let csv = shared::sparql::ResponseBody::from_static(
            b"compound,compoundLabel,taxon,ref_qid\nQ1,One,Q10,Q20\nQ2,Two,Q11,Q21\n",
        );

        let rows = sparql::parse_compounds_csv_display_bytes(&csv, 1).expect("display parse");
        assert_eq!(rows.len(), 1);
    }
}
