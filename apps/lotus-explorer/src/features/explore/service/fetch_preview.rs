// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Count + preview fetch service.
//!
//! This module is **Dioxus-free**: the phase-change callback (`on_fetching`)
//! is a plain `Fn()` closure so that tests can supply a no-op.

use crate::features::explore::search_state::SearchMetrics;
use crate::features::explore::types::{DomainError, ParseFault, QueryStage};
use crate::models::{CompoundEntry, DatasetStats};
use crate::perf;
use crate::queries;
use crate::repositories::LotusRepository;
use crate::services::search_telemetry as telemetry;
use crate::sparql;

/// Result of a successful count + preview fetch.
pub struct FetchResult {
    pub rows: Vec<CompoundEntry>,
    pub total_stats: Option<DatasetStats>,
    pub total_matches: Option<usize>,
    pub display_capped_rows: bool,
}

/// Fetch the count and display-preview rows for a search.
///
/// `on_fetching` is called once the counting phase completes, before the
/// display fetch begins.  Use it to advance the UI phase indicator; in tests
/// pass `|| ()`.
///
/// On WASM, the two queries are serialised (memory-safe).  On native they run
/// concurrently with `futures::try_join!`.  If the two-phase approach fails,
/// a single fallback query is attempted (native only).
#[allow(clippy::too_many_arguments)]
pub async fn fetch<R: LotusRepository>(
    execution_query: &str,
    display_limit: usize,
    repo: &R,
    metrics: &mut SearchMetrics,
    on_fetching: impl Fn(),
) -> Result<FetchResult, DomainError> {
    let count_query = queries::query_counts_from_base(execution_query);
    let display_query = queries::query_with_limit(execution_query, display_limit);

    let result = fetch_two_phase(
        &count_query,
        &display_query,
        display_limit,
        repo,
        metrics,
        &on_fetching,
    )
    .await;

    match result {
        Ok(v) => Ok(v),
        Err(err) => {
            #[cfg(target_arch = "wasm32")]
            {
                // On WASM the fallback is disabled to avoid OOM, but only map to
                // MemoryLimit when the underlying error actually indicates memory pressure.
                if is_probable_wasm_memory_limit(&err) {
                    return Err(DomainError::memory_limit(QueryStage::CountAndPreview));
                }
                return Err(err);
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                telemetry::fallback_entered(&format!("{err:?}"));
                fetch_fallback(execution_query, display_limit, repo, metrics).await
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
        DomainError::Parse(ParseFault::DisplayCsv { details })
        | DomainError::Parse(ParseFault::FallbackCsv { details }) => has_memory_signature(details),
        _ => false,
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

async fn fetch_two_phase<R: LotusRepository>(
    count_query: &str,
    display_query: &str,
    display_limit: usize,
    repo: &R,
    metrics: &mut SearchMetrics,
    on_fetching: &impl Fn(),
) -> Result<FetchResult, DomainError> {
    #[cfg(target_arch = "wasm32")]
    {
        telemetry::counting_sequential_fetch_wasm();
        let count_timer = perf::start_timer("LOTUS:count_query");
        let counts_csv = repo
            .sparql_bytes(count_query)
            .await
            .map_err(DomainError::transport_at(QueryStage::CountQuery))?;
        let count_elapsed = perf::end_timer("LOTUS:count_query", count_timer);
        metrics.add_network(count_elapsed);

        let count_parse_timer = perf::start_timer("LOTUS:count_parse");
        let full_stats = sparql::parse_counts_csv_bytes(&counts_csv).map_err(|e| {
            DomainError::Parse(ParseFault::CountCsv {
                details: e.to_string(),
            })
        })?;
        let count_parse_elapsed = perf::end_timer("LOTUS:count_parse", count_parse_timer);
        metrics.add_parse(count_parse_elapsed);
        let count_total_elapsed = count_elapsed.saturating_add(count_parse_elapsed);
        telemetry::counting_done(
            count_total_elapsed,
            full_stats.n_entries,
            full_stats.n_compounds,
            full_stats.n_taxa,
            full_stats.n_references,
        );

        on_fetching();
        telemetry::preview_started(display_limit);

        let display_timer = perf::start_timer("LOTUS:display_query");
        let display_csv = repo
            .sparql_bytes(display_query)
            .await
            .map_err(DomainError::transport_at(QueryStage::DisplayQuery))?;
        let display_elapsed = perf::end_timer("LOTUS:display_query", display_timer);
        metrics.add_network(display_elapsed);

        let display_parse_timer = perf::start_timer("LOTUS:display_parse");
        let rows = sparql::parse_compounds_csv_display_bytes(&display_csv, display_limit).map_err(
            |e| {
                DomainError::Parse(ParseFault::DisplayCsv {
                    details: e.to_string(),
                })
            },
        )?;
        let display_parse_elapsed = perf::end_timer("LOTUS:display_parse", display_parse_timer);
        metrics.add_parse(display_parse_elapsed);
        let display_total_elapsed = display_elapsed.saturating_add(display_parse_elapsed);
        telemetry::preview_done(display_total_elapsed, rows.len());

        let total_matches = full_stats.n_entries;
        let display_capped_rows = total_matches > rows.len();
        Ok(FetchResult {
            rows,
            total_stats: Some(full_stats),
            total_matches: Some(total_matches),
            display_capped_rows,
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        telemetry::counting_parallel_fetch_started();
        let count_timer = perf::start_timer("LOTUS:count_query");
        let display_timer = perf::start_timer("LOTUS:display_query");
        let (counts_csv, display_csv) = futures::try_join!(
            async {
                repo.sparql_bytes(count_query)
                    .await
                    .map_err(DomainError::transport_at(QueryStage::CountQuery))
            },
            async {
                repo.sparql_bytes(display_query)
                    .await
                    .map_err(DomainError::transport_at(QueryStage::DisplayQuery))
            },
        )?;
        let count_elapsed = perf::end_timer("LOTUS:count_query", count_timer);
        let display_elapsed = perf::end_timer("LOTUS:display_query", display_timer);
        metrics.add_network(count_elapsed);
        metrics.add_network(display_elapsed);

        let count_parse_timer = perf::start_timer("LOTUS:count_parse");
        let full_stats = sparql::parse_counts_csv_bytes(&counts_csv).map_err(|e| {
            DomainError::Parse(ParseFault::CountCsv {
                details: e.to_string(),
            })
        })?;
        let count_parse_elapsed = perf::end_timer("LOTUS:count_parse", count_parse_timer);
        metrics.add_parse(count_parse_elapsed);
        let count_total_elapsed = count_elapsed
            .max(display_elapsed)
            .saturating_add(count_parse_elapsed);
        telemetry::counting_done(
            count_total_elapsed,
            full_stats.n_entries,
            full_stats.n_compounds,
            full_stats.n_taxa,
            full_stats.n_references,
        );

        on_fetching();
        telemetry::preview_started(display_limit);

        let display_parse_timer = perf::start_timer("LOTUS:display_parse");
        let rows = sparql::parse_compounds_csv_display_bytes(&display_csv, display_limit).map_err(
            |e| {
                DomainError::Parse(ParseFault::DisplayCsv {
                    details: e.to_string(),
                })
            },
        )?;
        let display_parse_elapsed = perf::end_timer("LOTUS:display_parse", display_parse_timer);
        metrics.add_parse(display_parse_elapsed);
        telemetry::preview_done(display_parse_elapsed, rows.len());
        let total_matches = full_stats.n_entries;
        let display_capped_rows = total_matches > rows.len();
        Ok(FetchResult {
            rows,
            total_stats: Some(full_stats),
            total_matches: Some(total_matches),
            display_capped_rows,
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
async fn fetch_fallback<R: LotusRepository>(
    execution_query: &str,
    display_limit: usize,
    repo: &R,
    metrics: &mut SearchMetrics,
) -> Result<FetchResult, DomainError> {
    let fallback_timer = perf::start_timer("LOTUS:fallback_query");
    let csv = repo
        .sparql_bytes(execution_query)
        .await
        .map_err(DomainError::transport_at(QueryStage::FallbackQuery))?;
    let fallback_elapsed = perf::end_timer("LOTUS:fallback_query", fallback_timer);
    metrics.add_network(fallback_elapsed);

    let parse_timer = perf::start_timer("LOTUS:fallback_parse");
    let (rows, full_stats, parse_capped) =
        sparql::parse_compounds_csv_capped_bytes(&csv, display_limit).map_err(|e| {
            DomainError::Parse(ParseFault::FallbackCsv {
                details: e.to_string(),
            })
        })?;
    let parse_elapsed = perf::end_timer("LOTUS:fallback_parse", parse_timer);
    metrics.add_parse(parse_elapsed);
    telemetry::fallback_done(parse_elapsed, rows.len());
    let total_matches = full_stats.n_entries;
    let display_capped_rows = parse_capped || total_matches > rows.len();
    Ok(FetchResult {
        rows,
        total_stats: Some(full_stats),
        total_matches: Some(total_matches),
        display_capped_rows,
    })
}
