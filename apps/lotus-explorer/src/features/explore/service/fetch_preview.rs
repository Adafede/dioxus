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
use crate::repositories::LotusRepository;
use crate::services::search_telemetry as telemetry;
use crate::sparql;

/// Result of a successful full-results fetch.
pub struct FetchResult {
    pub rows: Vec<CompoundEntry>,
    pub total_stats: Option<DatasetStats>,
    pub total_matches: Option<usize>,
    pub display_capped_rows: bool,
}

/// Fetch full results with a single query and cap rendered rows locally.
///
/// `on_fetching` is called before the results fetch begins. Use it to advance
/// the UI phase indicator; in tests pass `|| ()`.
#[allow(clippy::too_many_arguments)]
pub async fn fetch<R: LotusRepository>(
    execution_query: &str,
    display_limit: usize,
    repo: &R,
    metrics: &mut SearchMetrics,
    on_fetching: impl Fn(),
) -> Result<FetchResult, DomainError> {
    let result =
        fetch_full_table_once(execution_query, display_limit, repo, metrics, &on_fetching).await;

    match result {
        Ok(v) => Ok(v),
        Err(err) => {
            #[cfg(target_arch = "wasm32")]
            {
                // Keep previous wasm classification semantics for memory pressure errors.
                if is_probable_wasm_memory_limit(&err) {
                    return Err(DomainError::memory_limit(QueryStage::CountAndPreview));
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
        DomainError::Parse(ParseFault::DisplayCsv { details })
        | DomainError::Parse(ParseFault::FallbackCsv { details }) => has_memory_signature(details),
        _ => false,
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

async fn fetch_full_table_once<R: LotusRepository>(
    execution_query: &str,
    display_limit: usize,
    repo: &R,
    metrics: &mut SearchMetrics,
    on_fetching: &impl Fn(),
) -> Result<FetchResult, DomainError> {
    on_fetching();
    telemetry::results_fetch_started(display_limit);

    let preview_timer = perf::start_timer("LOTUS:results_query");
    let preview_csv = repo
        .sparql_bytes(execution_query)
        .await
        .map_err(DomainError::transport_at(QueryStage::DisplayQuery))?;
    let preview_elapsed = perf::end_timer("LOTUS:results_query", preview_timer);
    metrics.add_network(preview_elapsed);

    let parse_timer = perf::start_timer("LOTUS:results_parse");
    let (rows, full_stats, parse_capped) =
        sparql::parse_compounds_csv_capped_bytes(&preview_csv, display_limit).map_err(|e| {
            DomainError::Parse(ParseFault::DisplayCsv {
                details: e.to_string(),
            })
        })?;
    let parse_elapsed = perf::end_timer("LOTUS:results_parse", parse_timer);
    metrics.add_parse(parse_elapsed);
    let total_matches = full_stats.n_entries;
    let display_capped_rows = parse_capped || total_matches > rows.len();
    telemetry::results_fetch_done(
        preview_elapsed.saturating_add(parse_elapsed),
        rows.len(),
        total_matches,
    );

    Ok(FetchResult {
        rows,
        total_stats: Some(full_stats),
        total_matches: Some(total_matches),
        display_capped_rows,
    })
}
