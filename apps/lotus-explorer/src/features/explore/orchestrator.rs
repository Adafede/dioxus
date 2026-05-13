// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Search orchestration for the reducer-backed Explore store.
//!
//! `start_search` dispatches a `SearchRequested` action, snapshots the request
//! token, and spawns `do_search`.  `do_search` remains async and is responsible
//! for the I/O-heavy search pipeline, but all user-visible state changes go
//! through `dispatch_explore_action`.

use crate::api;
use crate::export;
use crate::features::explore::actions::ExploreAction;
use crate::features::explore::search_state::{
    dispatch_explore_action, emit_search_summary, set_signal_if_changed, ExploreState,
    SearchMetrics,
};
use crate::features::explore::search_utils::{compute_hashes, sanitize_taxon_input};
use crate::features::explore::taxon_cache;
use crate::features::explore::types::{AppError, ErrorKind, QueryPhase};
use crate::i18n::{
    Locale, err_invalid_search_input, err_query_stage_failed, err_taxon_not_found,
    err_taxon_parse_failed, err_taxon_resolution_failed, err_taxon_search_failed,
    warn_ambiguous_taxon, warn_input_standardized,
};
use crate::models::{
    CompoundEntry, DatasetStats, Rows, SearchCriteria, SmilesSearchType, TaxonMatch,
};
use crate::perf;
use crate::queries;
use crate::repositories::LotusRepository;
use crate::sparql;
use crate::utils::logging::{log_debug_evt, log_info_evt, log_timing_evt, log_warn_evt};
use dioxus::prelude::*;
use shared::lotus::models::runtime_table_row_limit;
use std::sync::Arc;

/// The result produced by a successful `do_search` call.
pub struct SearchOutcome {
    pub rows: Vec<CompoundEntry>,
    pub qid: Option<String>,
    pub warning: Option<String>,
    pub query: String,
    pub total_matches: Option<usize>,
    pub total_stats: Option<DatasetStats>,
    pub display_capped_rows: bool,
}

pub fn start_search<R: LotusRepository + Copy>(
    criteria: Signal<SearchCriteria>,
    locale: Signal<Locale>,
    direct_download_mode: bool,
    explore: Signal<ExploreState>,
    repo: R,
) {
    let crit = criteria.peek().clone();
    if !crit.is_valid() {
        dispatch_explore_action(
            explore,
            ExploreAction::SearchFailed {
                kind: ErrorKind::Validation,
                message: err_invalid_search_input(*locale.peek()),
            },
        );
        return;
    }

    dispatch_explore_action(
        explore,
        ExploreAction::SearchRequested {
            criteria_snapshot: crit.clone(),
            direct_download: direct_download_mode,
        },
    );
    let request_token = explore.peek().search_request_token;

    spawn(async move {
        match do_search(crit.clone(), *locale.peek(), explore, direct_download_mode, repo).await {
            Ok(outcome) => {
                if request_token != explore.peek().search_request_token {
                    log_debug_evt(
                        "search",
                        "finish",
                        "stale_result_ignored",
                        Some(&format!("request_token={request_token}")),
                    );
                    return;
                }

                let filtered_stats = if direct_download_mode {
                    None
                } else {
                    Some(
                        outcome
                            .total_stats
                            .clone()
                            .unwrap_or_else(|| DatasetStats::from_entries(&outcome.rows)),
                    )
                };
                let filtered_matches = if direct_download_mode {
                    None
                } else {
                    Some(outcome.total_matches.unwrap_or(outcome.rows.len()))
                };

                let (q_hash, r_hash) = compute_hashes(
                    outcome.qid.as_deref().unwrap_or(""),
                    &crit,
                    &outcome.rows,
                );
                let meta_str = export::build_metadata_json(export::MetadataInputs {
                    criteria: &crit,
                    qid: outcome.qid.as_deref(),
                    number_of_records_override: filtered_matches,
                    query_hash: &q_hash,
                    result_hash: &r_hash,
                });

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
                        total_matches: filtered_matches,
                        total_stats: filtered_stats,
                        display_capped_rows: outcome.display_capped_rows,
                        query_hash: q_hash,
                        result_hash: r_hash,
                        metadata_json: Arc::<str>::from(meta_str),
                    },
                );
            }
            Err(e) => {
                if request_token != explore.peek().search_request_token {
                    log_debug_evt(
                        "search",
                        "finish",
                        "stale_error_ignored",
                        Some(&format!("request_token={request_token}")),
                    );
                    return;
                }
                dispatch_explore_action(
                    explore,
                    ExploreAction::SearchFailed {
                        kind: e.kind,
                        message: e.message,
                    },
                );
            }
        }
    });
}

pub async fn do_search<R: LotusRepository + Copy>(
    crit: SearchCriteria,
    locale: Locale,
    explore: Signal<ExploreState>,
    direct_download_mode: bool,
    repo: R,
) -> Result<SearchOutcome, AppError> {
    let search_timer = perf::start_timer("LOTUS:search_total");
    let mut metrics = SearchMetrics::default();
    log_info_evt("search", "start", "begin", None);

    let taxon = crit.taxon.trim().to_string();
    let smiles = {
        let normalized = crit.smiles.replace("\r\n", "\n").replace('\r', "\n");
        let kind = queries::classify_structure(&normalized);
        if matches!(
            kind,
            queries::StructureKind::MolfileV2000 | queries::StructureKind::MolfileV3000
        ) {
            normalized
        } else {
            normalized.trim().to_string()
        }
    };

    if !direct_download_mode {
        let mut api_crit = crit.clone();
        api_crit.smiles = smiles.clone();
        let display_limit = runtime_table_row_limit();
        let include_counts = true;
        let api_timer = perf::start_timer("LOTUS:api_search");
        match repo.api_search(&api_crit, display_limit, include_counts).await {
            None => {
                log_info_evt("search", "api", "path_not_available", Some("reason=not_configured"));
            }
            Some(Ok(response)) => {
                let api_elapsed = perf::end_timer("LOTUS:api_search", api_timer);
                metrics.add_network(api_elapsed);
                log_timing_evt(
                    "search",
                    "api",
                    "success",
                    api_elapsed,
                    Some(&format!("rows={} total_matches={}", response.rows.len(), response.total_matches)),
                );
                let display_capped_rows = if include_counts {
                    response.total_matches > response.rows.len()
                } else {
                    response.rows.len() >= display_limit
                };
                let rows = response.rows.into_iter().map(CompoundEntry::from).collect::<Vec<_>>();
                return Ok(SearchOutcome {
                    rows,
                    qid: response.resolved_taxon_qid,
                    warning: response.warning,
                    query: response.query,
                    total_matches: Some(response.total_matches),
                    total_stats: Some(response.stats.into()),
                    display_capped_rows,
                });
            }
            Some(Err(err)) => {
                let api_elapsed = perf::end_timer("LOTUS:api_search", api_timer);
                log_timing_evt(
                    "search",
                    "api",
                    "fallback_direct",
                    api_elapsed,
                    Some(&format!("reason={err}")),
                );
            }
        }
    } else {
        log_info_evt("search", "api", "path_not_available", Some("reason=direct_download_mode"));
    }

    let mut warning: Option<String> = None;
    let taxon_qid = resolve_taxon_qid(&taxon, locale, explore, &mut metrics, &mut warning, &repo).await?;

    let sparql_query = build_sparql_query(&smiles, &crit, taxon_qid.as_deref());
    let execution_query = queries::query_with_server_filters(&sparql_query, &crit);
    log_debug_evt(
        "search",
        "query_build",
        "after_server_filters",
        Some(&format!(
            "has_SERVICE={} has_FILTER={}",
            execution_query.contains("SERVICE"),
            execution_query.contains("FILTER")
        )),
    );

    if direct_download_mode {
        let total_elapsed = perf::end_timer("LOTUS:search_total", search_timer);
        log_timing_evt("search", "direct_download", "ready", total_elapsed, Some("skipped=count_and_preview"));
        emit_search_summary(total_elapsed, metrics);
        return Ok(SearchOutcome {
            rows: Vec::new(),
            qid: taxon_qid,
            warning,
            query: execution_query,
            total_matches: None,
            total_stats: None,
            display_capped_rows: false,
        });
    }

    let display_limit = runtime_table_row_limit();
    dispatch_explore_action(explore, ExploreAction::SearchPhaseChanged(QueryPhase::Counting));
    let count_query = queries::query_counts_from_base(&execution_query);
    let display_query = queries::query_with_limit(&execution_query, display_limit);

    let (rows, total_stats_out, total_matches, display_capped_rows) = fetch_count_and_preview(
        &count_query,
        &display_query,
        &execution_query,
        display_limit,
        locale,
        explore,
        &mut metrics,
        &repo,
    )
    .await?;

    let outcome = SearchOutcome {
        rows,
        qid: taxon_qid,
        warning,
        query: execution_query,
        total_matches,
        total_stats: total_stats_out,
        display_capped_rows,
    };
    let total_elapsed = perf::end_timer("LOTUS:search_total", search_timer);
    perf::log_timing(
        "SearchComplete",
        &format!(
            "Search completed (display_rows={}, total_matches={})",
            outcome.rows.len(),
            outcome.total_matches.unwrap_or(outcome.rows.len())
        ),
        Some(total_elapsed),
    );
    emit_search_summary(total_elapsed, metrics);
    Ok(outcome)
}

async fn resolve_taxon_qid<R: LotusRepository + Copy>(
    taxon: &str,
    locale: Locale,
    explore: Signal<ExploreState>,
    metrics: &mut SearchMetrics,
    warning: &mut Option<String>,
    repo: &R,
) -> Result<Option<String>, AppError> {
    if taxon.is_empty() {
        return Ok(None);
    }
    if taxon == "*" {
        return Ok(Some("*".to_string()));
    }
    if taxon.to_uppercase().starts_with('Q') && taxon[1..].chars().all(|c| c.is_ascii_digit()) {
        return Ok(Some(taxon.to_uppercase()));
    }

    dispatch_explore_action(explore, ExploreAction::SearchPhaseChanged(QueryPhase::ResolvingTaxon));
    let taxon_timer = perf::start_timer("LOTUS:taxon_resolution");

    let sanitized = sanitize_taxon_input(taxon);
    if sanitized != taxon {
        *warning = Some(warn_input_standardized(locale, taxon, &sanitized));
    }

    if let Some(cached_qid) = taxon_cache::lookup(&sanitized) {
        let taxon_elapsed = perf::end_timer("LOTUS:taxon_resolution", taxon_timer);
        log_timing_evt(
            "search",
            "ResolvingTaxon",
            "cache_hit",
            taxon_elapsed,
            Some(&format!("taxon_input={} qid={}", sanitized, cached_qid)),
        );
        return Ok(Some(cached_qid));
    }

    let query = queries::query_taxon_search(&sanitized);
    let csv = repo.sparql_bytes(&query).await.map_err(|e| AppError {
        kind: ErrorKind::Network,
        message: err_taxon_search_failed(locale, &e),
    })?;
    let taxon_elapsed = perf::end_timer("LOTUS:taxon_resolution", taxon_timer);
    metrics.add_network(taxon_elapsed);
    perf::log_timing("ResolvingTaxon", "Taxon query completed", Some(taxon_elapsed));

    let matches = sparql::parse_taxon_csv_bytes(&csv).map_err(|e| AppError {
        kind: ErrorKind::Parse,
        message: err_taxon_parse_failed(locale, &e.to_string()),
    })?;
    if matches.is_empty() {
        return Err(AppError {
            kind: ErrorKind::Validation,
            message: err_taxon_not_found(locale, taxon),
        });
    }

    let lower = sanitized.to_lowercase();
    let exact: Vec<&TaxonMatch> = matches.iter().filter(|m| m.name.to_lowercase() == lower).collect();
    let best = exact
        .first()
        .copied()
        .or_else(|| matches.first())
        .ok_or_else(|| AppError {
            kind: ErrorKind::Parse,
            message: err_taxon_resolution_failed(locale),
        })?;

    if exact.len() > 1 || (exact.is_empty() && matches.len() > 1) {
        let names = matches
            .iter()
            .take(4)
            .map(|m| format!("{} ({})", m.name, m.qid))
            .collect::<Vec<_>>()
            .join(", ");
        *warning = Some(warn_ambiguous_taxon(locale, &best.name, &best.qid, &names));
    }

    taxon_cache::store(&sanitized, &best.qid);
    Ok(Some(best.qid.clone()))
}

fn build_sparql_query(smiles: &str, crit: &SearchCriteria, taxon_qid: Option<&str>) -> String {
    if !smiles.is_empty() {
        let effective_type = if (smiles.contains('\n') || smiles.contains('\r'))
            && crit.smiles_search_type == SmilesSearchType::Similarity
        {
            SmilesSearchType::Substructure
        } else {
            crit.smiles_search_type
        };
        let taxon_for_sachem = match taxon_qid {
            Some("*") => Some("Q2382443"),
            Some(qid) => Some(qid),
            None => None,
        };
        let q = queries::query_sachem(smiles, effective_type, crit.smiles_threshold, taxon_for_sachem);
        log_debug_evt(
            "search",
            "query_build",
            "sachem_query_created",
            Some(&format!("has_SERVICE={}", q.contains("SERVICE"))),
        );
        q
    } else {
        match taxon_qid {
            Some(qid) if qid != "*" => queries::query_compounds_by_taxon(qid),
            _ => queries::query_all_compounds(),
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn fetch_count_and_preview<R: LotusRepository + Copy>(
    count_query: &str,
    display_query: &str,
    execution_query: &str,
    display_limit: usize,
    locale: Locale,
    explore: Signal<ExploreState>,
    metrics: &mut SearchMetrics,
    repo: &R,
) -> Result<(Vec<CompoundEntry>, Option<DatasetStats>, Option<usize>, bool), AppError> {
    let result = async {
        #[cfg(target_arch = "wasm32")]
        {
            log_debug_evt("search", "Counting", "sequential_fetch_wasm", None);
            let count_timer = perf::start_timer("LOTUS:count_query");
            let counts_csv = repo.sparql_bytes(count_query).await.map_err(|e| AppError {
                kind: ErrorKind::Network,
                message: err_query_stage_failed(locale, "count query", &e),
            })?;
            let count_elapsed = perf::end_timer("LOTUS:count_query", count_timer);
            metrics.add_network(count_elapsed);
            perf::log_timing("Counting", "Count query completed", Some(count_elapsed));

            let count_parse_timer = perf::start_timer("LOTUS:count_parse");
            let full_stats = sparql::parse_counts_csv_bytes(&counts_csv).map_err(|e| AppError {
                kind: ErrorKind::Parse,
                message: err_query_stage_failed(locale, "count parse", &e.to_string()),
            })?;
            let count_parse_elapsed = perf::end_timer("LOTUS:count_parse", count_parse_timer);
            metrics.add_parse(count_parse_elapsed);
            perf::log_timing(
                "Counting",
                &format!(
                    "Count parse completed (entries={}, compounds={}, taxa={}, refs={})",
                    full_stats.n_entries, full_stats.n_compounds, full_stats.n_taxa, full_stats.n_references
                ),
                Some(count_parse_elapsed),
            );

            dispatch_explore_action(explore, ExploreAction::SearchPhaseChanged(QueryPhase::FetchingPreview));
            let display_timer = perf::start_timer("LOTUS:display_query");
            let display_csv = repo.sparql_bytes(display_query).await.map_err(|e| AppError {
                kind: ErrorKind::Network,
                message: err_query_stage_failed(locale, "display query", &e),
            })?;
            let display_elapsed = perf::end_timer("LOTUS:display_query", display_timer);
            metrics.add_network(display_elapsed);
            perf::log_timing("FetchingPreview", "Display query completed", Some(display_elapsed));

            let display_parse_timer = perf::start_timer("LOTUS:display_parse");
            let rows = sparql::parse_compounds_csv_display_bytes(&display_csv, display_limit)
                .map_err(|e| AppError {
                    kind: ErrorKind::Parse,
                    message: err_query_stage_failed(locale, "display parse", &e.to_string()),
                })?;
            let display_parse_elapsed = perf::end_timer("LOTUS:display_parse", display_parse_timer);
            metrics.add_parse(display_parse_elapsed);
            perf::log_timing(
                "FetchingPreview",
                &format!("Display parse completed (rows={})", rows.len()),
                Some(display_parse_elapsed),
            );
            let display_capped_rows = full_stats.n_entries > rows.len();
            Ok::<_, AppError>((rows, Some(full_stats.clone()), Some(full_stats.n_entries), display_capped_rows))
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            log_debug_evt("search", "Counting", "parallel_fetch_started", None);
            let count_fut = repo.sparql_bytes(count_query);
            let display_fut = repo.sparql_bytes(display_query);
            let count_timer = perf::start_timer("LOTUS:count_query");
            let display_timer = perf::start_timer("LOTUS:display_query");
            let (counts_csv, display_csv) = futures::try_join!(
                async { count_fut.await.map_err(|e| AppError { kind: ErrorKind::Network, message: err_query_stage_failed(locale, "count query", &e) }) },
                async { display_fut.await.map_err(|e| AppError { kind: ErrorKind::Network, message: err_query_stage_failed(locale, "display query", &e) }) },
            )?;
            let count_elapsed = perf::end_timer("LOTUS:count_query", count_timer);
            let display_elapsed = perf::end_timer("LOTUS:display_query", display_timer);
            metrics.add_network(count_elapsed);
            metrics.add_network(display_elapsed);
            perf::log_timing("Counting", "Count query completed", Some(count_elapsed));
            perf::log_timing("FetchingPreview", "Display query completed", Some(display_elapsed));
            let count_parse_timer = perf::start_timer("LOTUS:count_parse");
            let full_stats = sparql::parse_counts_csv_bytes(&counts_csv).map_err(|e| AppError {
                kind: ErrorKind::Parse,
                message: err_query_stage_failed(locale, "count parse", &e.to_string()),
            })?;
            let count_parse_elapsed = perf::end_timer("LOTUS:count_parse", count_parse_timer);
            metrics.add_parse(count_parse_elapsed);
            perf::log_timing(
                "Counting",
                &format!(
                    "Count parse completed (entries={}, compounds={}, taxa={}, refs={})",
                    full_stats.n_entries, full_stats.n_compounds, full_stats.n_taxa, full_stats.n_references
                ),
                Some(count_parse_elapsed),
            );
            dispatch_explore_action(explore, ExploreAction::SearchPhaseChanged(QueryPhase::FetchingPreview));
            let display_parse_timer = perf::start_timer("LOTUS:display_parse");
            let rows = sparql::parse_compounds_csv_display_bytes(&display_csv, display_limit)
                .map_err(|e| AppError {
                    kind: ErrorKind::Parse,
                    message: err_query_stage_failed(locale, "display parse", &e.to_string()),
                })?;
            let display_parse_elapsed = perf::end_timer("LOTUS:display_parse", display_parse_timer);
            metrics.add_parse(display_parse_elapsed);
            perf::log_timing(
                "FetchingPreview",
                &format!("Display parse completed (rows={})", rows.len()),
                Some(display_parse_elapsed),
            );
            let display_capped_rows = full_stats.n_entries > rows.len();
            Ok::<_, AppError>((rows, Some(full_stats.clone()), Some(full_stats.n_entries), display_capped_rows))
        }
    }
    .await;

    match result {
        Ok(v) => Ok(v),
        Err(err_msg) => {
            #[cfg(target_arch = "wasm32")]
            {
                return Err(AppError {
                    kind: ErrorKind::Memory,
                    message: crate::i18n::err_wasm_large_query_fallback(locale, &err_msg.message),
                });
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                log_warn_evt("search", "Fallback", "entered", Some("reason=two_phase_failed"));
                let _ = err_msg;
                let fallback_query_timer = perf::start_timer("LOTUS:fallback_query");
                let csv = repo.sparql_bytes(execution_query).await.map_err(|e| AppError {
                    kind: ErrorKind::Network,
                    message: err_query_stage_failed(locale, "query", &e),
                })?;
                let fallback_query_elapsed = perf::end_timer("LOTUS:fallback_query", fallback_query_timer);
                metrics.add_network(fallback_query_elapsed);
                perf::log_timing("Fallback", "Fallback query completed", Some(fallback_query_elapsed));
                let fallback_parse_timer = perf::start_timer("LOTUS:fallback_parse");
                let (rows, full_stats, parse_capped) =
                    sparql::parse_compounds_csv_capped_bytes(&csv, display_limit).map_err(|e| AppError {
                        kind: ErrorKind::Parse,
                        message: err_query_stage_failed(locale, "parse", &e.to_string()),
                    })?;
                let fallback_parse_elapsed = perf::end_timer("LOTUS:fallback_parse", fallback_parse_timer);
                metrics.add_parse(fallback_parse_elapsed);
                perf::log_timing(
                    "Fallback",
                    &format!("Fallback parse completed (rows={})", rows.len()),
                    Some(fallback_parse_elapsed),
                );
                let display_capped_rows = parse_capped || full_stats.n_entries > rows.len();
                Ok((rows, Some(full_stats.clone()), Some(full_stats.n_entries), display_capped_rows))
            }
        }
    }
}
