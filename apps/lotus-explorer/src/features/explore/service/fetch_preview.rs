// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Count + preview fetch service.
//!
//! This module is **Dioxus-free**: the phase-change callback (`on_fetching`)
//! is a plain `Fn()` closure so that tests can supply a no-op.

use crate::features::explore::search_state::SearchMetrics;
use crate::features::explore::types::{DomainError, ParseFault};
use crate::models::{CompoundEntry, DatasetStats};
use crate::perf;
use crate::queries;
use crate::repositories::LotusRepository;
use crate::sparql;
#[cfg(not(target_arch = "wasm32"))]
use crate::utils::logging::log_warn_evt;
use crate::utils::logging::{log_debug_evt, log_timing_evt};

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
                    return Err(DomainError::MemoryLimit {
                        stage: "count_and_preview",
                    });
                }
                return Err(err);
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                log_warn_evt(
                    "search",
                    "Fallback",
                    "entered",
                    Some(&format!("reason=two_phase_failed original={err:?}")),
                );
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
        log_debug_evt("search", "Counting", "sequential_fetch_wasm", None);
        let count_timer = perf::start_timer("LOTUS:count_query");
        let counts_csv =
            repo.sparql_bytes(count_query)
                .await
                .map_err(|source| DomainError::Transport {
                    stage: "count query",
                    source,
                })?;
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
        log_timing_evt(
            "search",
            "Counting",
            "done",
            count_parse_elapsed,
            Some(&format!(
                "entries={} compounds={} taxa={} refs={}",
                full_stats.n_entries,
                full_stats.n_compounds,
                full_stats.n_taxa,
                full_stats.n_references
            )),
        );

        on_fetching();

        let display_timer = perf::start_timer("LOTUS:display_query");
        let display_csv =
            repo.sparql_bytes(display_query)
                .await
                .map_err(|source| DomainError::Transport {
                    stage: "display query",
                    source,
                })?;
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
        log_timing_evt(
            "search",
            "FetchingPreview",
            "done",
            display_parse_elapsed,
            Some(&format!("rows={}", rows.len())),
        );

        let display_capped_rows = full_stats.n_entries > rows.len();
        Ok(FetchResult {
            rows,
            total_stats: Some(full_stats.clone()),
            total_matches: Some(full_stats.n_entries),
            display_capped_rows,
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        log_debug_evt("search", "Counting", "parallel_fetch_started", None);
        let count_timer = perf::start_timer("LOTUS:count_query");
        let display_timer = perf::start_timer("LOTUS:display_query");
        let (counts_csv, display_csv) = futures::try_join!(
            async {
                repo.sparql_bytes(count_query)
                    .await
                    .map_err(|source| DomainError::Transport {
                        stage: "count query",
                        source,
                    })
            },
            async {
                repo.sparql_bytes(display_query)
                    .await
                    .map_err(|source| DomainError::Transport {
                        stage: "display query",
                        source,
                    })
            },
        )?;
        let count_elapsed = perf::end_timer("LOTUS:count_query", count_timer);
        let display_elapsed = perf::end_timer("LOTUS:display_query", display_timer);
        metrics.add_network(count_elapsed);
        metrics.add_network(display_elapsed);

        let full_stats = sparql::parse_counts_csv_bytes(&counts_csv).map_err(|e| {
            DomainError::Parse(ParseFault::CountCsv {
                details: e.to_string(),
            })
        })?;
        log_timing_evt(
            "search",
            "Counting",
            "done",
            count_elapsed,
            Some(&format!(
                "entries={} compounds={} taxa={} refs={}",
                full_stats.n_entries,
                full_stats.n_compounds,
                full_stats.n_taxa,
                full_stats.n_references
            )),
        );

        on_fetching();

        let rows = sparql::parse_compounds_csv_display_bytes(&display_csv, display_limit).map_err(
            |e| {
                DomainError::Parse(ParseFault::DisplayCsv {
                    details: e.to_string(),
                })
            },
        )?;
        log_timing_evt(
            "search",
            "FetchingPreview",
            "done",
            display_elapsed,
            Some(&format!("rows={}", rows.len())),
        );
        let display_capped_rows = full_stats.n_entries > rows.len();
        Ok(FetchResult {
            rows,
            total_stats: Some(full_stats.clone()),
            total_matches: Some(full_stats.n_entries),
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
    let csv =
        repo.sparql_bytes(execution_query)
            .await
            .map_err(|source| DomainError::Transport {
                stage: "fallback query",
                source,
            })?;
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
    log_timing_evt(
        "search",
        "Fallback",
        "done",
        parse_elapsed,
        Some(&format!("rows={}", rows.len())),
    );
    let display_capped_rows = parse_capped || full_stats.n_entries > rows.len();
    Ok(FetchResult {
        rows,
        total_stats: Some(full_stats.clone()),
        total_matches: Some(full_stats.n_entries),
        display_capped_rows,
    })
}
