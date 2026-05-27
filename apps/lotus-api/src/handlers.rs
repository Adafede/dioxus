// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use axum::{
    Json,
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::Response,
};
use shared::lotus::models;
use shared::lotus::pubchem_tree::{
    DownloadArtifactKind, NPCLASSIFIER_CACHE_URL, PubchemTreeError, build_download_json,
    build_trees, compute_stats, fetch_dataset,
};
use std::{
    sync::{Arc, atomic::Ordering},
    time::Instant,
};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use tokio::time::timeout;

use crate::{
    errors::{ApiError, ErrorResponse, SharedApiError},
    query_logic::{
        api_export_file_url, apply_request, build_execution_query, build_upstream_export_url,
        gzip_bytes, qlever_export_url, resolve_taxon_qid_cached, sanitize_download_filename,
    },
    services::build_search_response,
    state::{
        AppState, BuiltPubchemSession, build_export_cache_key, build_pubchem_session_id,
        build_search_cache_key, export_cache_get, export_cache_put, export_inflight_cell,
        export_inflight_remove, pubchem_session_get, pubchem_session_put_built,
        pubchem_session_put_fetched, search_cache_get, search_cache_put, search_inflight_cell,
        search_inflight_remove,
    },
    types::{
        DataStatsDto, DownloadArtifactDto, ExportArchiveFormat, ExportFileQuery, ExportUrlResponse,
        HealthResponse, PreviewTreeDto, PubchemBuildRequest, PubchemBuildResponse,
        PubchemFetchResponse, SearchRequest, SearchResponse, TreeSummaryDto,
    },
};

fn map_pubchem_error(err: PubchemTreeError) -> ApiError {
    match err {
        PubchemTreeError::Invalid(message) => ApiError::bad_request(message),
        PubchemTreeError::Http(_, message)
        | PubchemTreeError::Network(message)
        | PubchemTreeError::Parse(message) => ApiError::upstream(message),
    }
}

fn generated_at_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

fn date_stamp(generated_at: &str) -> String {
    generated_at
        .chars()
        .take(10)
        .filter(|ch| *ch != '-')
        .collect()
}

fn pubchem_download_url(session_id: &str, kind: DownloadArtifactKind) -> String {
    format!("/v1/pubchem-tree/download/{session_id}/{}", kind.key())
}

#[utoipa::path(
    get,
    path = "/health",
    responses((status = 200, description = "Service health", body = HealthResponse))
)]
pub(crate) async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(state.metrics.snapshot())
}

#[utoipa::path(
    get,
    path = "/metrics",
    responses((status = 200, description = "Prometheus-style runtime metrics", body = String))
)]
pub(crate) async fn metrics(State(state): State<AppState>) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-store")
        .body(axum::body::Body::from(state.metrics.render_prometheus()))
        .expect("metrics response")
}

#[utoipa::path(
    post,
    path = "/v1/search",
    request_body = SearchRequest,
    responses(
        (status = 200, description = "Search results", body = SearchResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 502, description = "Upstream SPARQL failure", body = ErrorResponse),
        (status = 503, description = "Server overloaded", body = ErrorResponse),
        (status = 504, description = "Search timeout", body = ErrorResponse)
    )
)]
pub(crate) async fn search(
    State(state): State<AppState>,
    Json(req): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, ApiError> {
    let req_started = Instant::now();
    let _permit = state.request_permits.try_acquire().map_err(|_| {
        state
            .metrics
            .overload_rejections
            .fetch_add(1, Ordering::Relaxed);
        log::warn!("event=search state=rejected reason=overloaded");
        ApiError::overloaded("Server is busy, retry shortly")
    })?;

    let mut criteria = apply_request(&req)?;
    if !criteria.is_valid() {
        return Err(ApiError::bad_request(
            "Either taxon or smiles/structure must be provided",
        ));
    }

    let (resolved_taxon_qid, warning) = timeout(
        state.request_timeout,
        resolve_taxon_qid_cached(&state, criteria.taxon.clone()),
    )
    .await
    .map_err(|_| {
        state
            .metrics
            .request_timeouts
            .fetch_add(1, Ordering::Relaxed);
        log::warn!("event=search state=timeout phase=taxon");
        ApiError::upstream("taxon resolution timed out")
    })??;
    if let Some(qid) = resolved_taxon_qid.as_deref()
        && qid != "*"
    {
        criteria.taxon = qid.to_string();
    }
    let execution_query = build_execution_query(&criteria, resolved_taxon_qid.as_deref());
    let limit = req
        .limit
        .unwrap_or(state.default_limit)
        .clamp(1, models::TABLE_ROW_LIMIT);
    let include_counts = req.include_counts.unwrap_or(true);
    log::info!(
        "event=search state=start include_counts={} limit={} has_smiles={}",
        include_counts,
        limit,
        !criteria.smiles.trim().is_empty(),
    );
    let cache_key = build_search_cache_key(&execution_query, limit, include_counts);
    if let Some(cached) = search_cache_get(&state, &cache_key) {
        state
            .metrics
            .search_cache_hits
            .fetch_add(1, Ordering::Relaxed);
        log::debug!("event=search state=cache_hit");
        return Ok(Json(cached));
    }
    state
        .metrics
        .search_cache_misses
        .fetch_add(1, Ordering::Relaxed);
    log::debug!("event=search state=cache_miss");

    let (cell, is_leader) = search_inflight_cell(&state, &cache_key);
    if !is_leader {
        state
            .metrics
            .search_inflight_waits
            .fetch_add(1, Ordering::Relaxed);
        log::debug!("event=search state=coalesced_wait");
    } else {
        state
            .metrics
            .search_upstream_hits
            .fetch_add(1, Ordering::Relaxed);
        log::debug!("event=search state=upstream_hit");
    }
    let metrics = state.metrics.clone();
    let request_timeout = state.request_timeout;
    let response = cell
        .get_or_init(|| async {
            timeout(
                request_timeout,
                build_search_response(
                    &execution_query,
                    limit,
                    include_counts,
                    resolved_taxon_qid.clone(),
                    warning.clone(),
                ),
            )
            .await
            .map_err(|_| {
                metrics.request_timeouts.fetch_add(1, Ordering::Relaxed);
                log::warn!("event=search state=timeout phase=execution");
                SharedApiError {
                    status: StatusCode::GATEWAY_TIMEOUT,
                    message: "search execution timed out".to_string(),
                }
            })?
            .map_err(SharedApiError::from)
        })
        .await
        .clone();
    search_inflight_remove(&state, &cache_key, &cell, is_leader);

    match response {
        Ok(response) => {
            search_cache_put(&state, cache_key, response.clone());
            log::info!(
                "event=search state=success elapsed_ms={:.1} rows={} total_matches={}",
                req_started.elapsed().as_secs_f64() * 1000.0,
                response.rows.len(),
                response.total_matches,
            );
            Ok(Json(response))
        }
        Err(err) => {
            log::warn!(
                "event=search state=error elapsed_ms={:.1} status={} message={}",
                req_started.elapsed().as_secs_f64() * 1000.0,
                err.status,
                err.message,
            );
            Err(err.into())
        }
    }
}

#[utoipa::path(
    post,
    path = "/v1/export-url",
    request_body = SearchRequest,
    responses(
        (status = 200, description = "Direct export URLs", body = ExportUrlResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 502, description = "Upstream SPARQL failure", body = ErrorResponse),
        (status = 503, description = "Server overloaded", body = ErrorResponse),
        (status = 504, description = "Taxon-resolution timeout", body = ErrorResponse)
    )
)]
pub(crate) async fn export_urls(
    State(state): State<AppState>,
    Json(req): Json<SearchRequest>,
) -> Result<Json<ExportUrlResponse>, ApiError> {
    let req_started = Instant::now();
    let _permit = state.request_permits.try_acquire().map_err(|_| {
        state
            .metrics
            .overload_rejections
            .fetch_add(1, Ordering::Relaxed);
        log::warn!("event=export state=rejected reason=overloaded");
        ApiError::overloaded("Server is busy, retry shortly")
    })?;

    let mut criteria = apply_request(&req)?;
    if !criteria.is_valid() {
        return Err(ApiError::bad_request(
            "Either taxon or smiles/structure must be provided",
        ));
    }

    let (resolved_taxon_qid, _warning) = timeout(
        state.request_timeout,
        resolve_taxon_qid_cached(&state, criteria.taxon.clone()),
    )
    .await
    .map_err(|_| {
        state
            .metrics
            .request_timeouts
            .fetch_add(1, Ordering::Relaxed);
        log::warn!("event=export state=timeout phase=taxon");
        ApiError::upstream("taxon resolution timed out")
    })??;
    if let Some(qid) = resolved_taxon_qid.as_deref()
        && qid != "*"
    {
        criteria.taxon = qid.to_string();
    }
    let query = build_execution_query(&criteria, resolved_taxon_qid.as_deref());
    let cache_key = build_export_cache_key(&query);
    log::info!("event=export state=start");

    if let Some(cached) = export_cache_get(&state, &cache_key) {
        state
            .metrics
            .export_cache_hits
            .fetch_add(1, Ordering::Relaxed);
        log::debug!("event=export state=cache_hit");
        return Ok(Json(cached));
    }
    state
        .metrics
        .export_cache_misses
        .fetch_add(1, Ordering::Relaxed);
    log::debug!("event=export state=cache_miss");

    let (cell, is_leader) = export_inflight_cell(&state, &cache_key);
    if !is_leader {
        state
            .metrics
            .export_inflight_waits
            .fetch_add(1, Ordering::Relaxed);
        log::debug!("event=export state=coalesced_wait");
    } else {
        state
            .metrics
            .export_upstream_hits
            .fetch_add(1, Ordering::Relaxed);
        log::debug!("event=export state=upstream_hit");
    }
    let response = cell
        .get_or_init(|| async {
            Ok::<_, SharedApiError>(ExportUrlResponse {
                csv_url: qlever_export_url(&query, "csv_export"),
                json_url: qlever_export_url(&query, "qlever_json_export"),
                rdf_url: qlever_export_url(
                    &shared::lotus::queries::query_construct_from_select(&query),
                    "turtle_export",
                ),
                csv_gz_url: api_export_file_url(&cache_key, ExportArchiveFormat::Csv),
                json_gz_url: api_export_file_url(&cache_key, ExportArchiveFormat::Json),
                rdf_gz_url: api_export_file_url(&cache_key, ExportArchiveFormat::Rdf),
                query,
            })
        })
        .await
        .clone();
    export_inflight_remove(&state, &cache_key, &cell, is_leader);

    match response {
        Ok(response) => {
            export_cache_put(&state, cache_key, response.clone());
            log::info!(
                "event=export state=success elapsed_ms={:.1}",
                req_started.elapsed().as_secs_f64() * 1000.0,
            );
            Ok(Json(response))
        }
        Err(err) => {
            log::warn!(
                "event=export state=error elapsed_ms={:.1} status={} message={}",
                req_started.elapsed().as_secs_f64() * 1000.0,
                err.status,
                err.message,
            );
            Err(err.into())
        }
    }
}

#[utoipa::path(
    post,
    path = "/v1/pubchem-tree/fetch",
    responses(
        (status = 200, description = "Fetched LOTUS dataset statistics and session id", body = PubchemFetchResponse),
        (status = 502, description = "Upstream SPARQL failure", body = ErrorResponse),
        (status = 503, description = "Server overloaded", body = ErrorResponse),
        (status = 504, description = "Fetch timeout", body = ErrorResponse)
    )
)]
pub(crate) async fn pubchem_fetch(
    State(state): State<AppState>,
) -> Result<Json<PubchemFetchResponse>, ApiError> {
    let _permit = state.request_permits.try_acquire().map_err(|_| {
        state
            .metrics
            .overload_rejections
            .fetch_add(1, Ordering::Relaxed);
        ApiError::overloaded("Server is busy, retry shortly")
    })?;

    let fetched = timeout(
        state.request_timeout,
        fetch_dataset(shared::sparql::QLEVER_WIKIDATA),
    )
    .await
    .map_err(|_| {
        state
            .metrics
            .request_timeouts
            .fetch_add(1, Ordering::Relaxed);
        ApiError::upstream("PubChem tree fetch timed out")
    })?
    .map_err(map_pubchem_error)?;

    let stats = compute_stats(&fetched);
    let session_id = build_pubchem_session_id(&state);
    pubchem_session_put_fetched(&state, session_id.clone(), Arc::new(fetched));

    Ok(Json(PubchemFetchResponse {
        session_id,
        stats: DataStatsDto::from(stats),
    }))
}

#[utoipa::path(
    post,
    path = "/v1/pubchem-tree/build",
    request_body = PubchemBuildRequest,
    responses(
        (status = 200, description = "Built PubChem tree previews and download links", body = PubchemBuildResponse),
        (status = 400, description = "Unknown or expired session", body = ErrorResponse),
        (status = 502, description = "Build failure", body = ErrorResponse),
        (status = 503, description = "Server overloaded", body = ErrorResponse),
        (status = 504, description = "Build timeout", body = ErrorResponse)
    )
)]
pub(crate) async fn pubchem_build(
    State(state): State<AppState>,
    Json(req): Json<PubchemBuildRequest>,
) -> Result<Json<PubchemBuildResponse>, ApiError> {
    let _permit = state.request_permits.try_acquire().map_err(|_| {
        state
            .metrics
            .overload_rejections
            .fetch_add(1, Ordering::Relaxed);
        ApiError::overloaded("Server is busy, retry shortly")
    })?;

    let session = pubchem_session_get(&state, &req.session_id)
        .ok_or_else(|| ApiError::bad_request("PubChem tree session expired or is unknown"))?;

    let built_session = match session.built {
        Some(built) => built,
        None => {
            let fetched = session.fetched.clone();
            let bundle = timeout(
                state.request_timeout,
                build_trees(&fetched, NPCLASSIFIER_CACHE_URL),
            )
            .await
            .map_err(|_| {
                state
                    .metrics
                    .request_timeouts
                    .fetch_add(1, Ordering::Relaxed);
                ApiError::upstream("PubChem tree build timed out")
            })?
            .map_err(map_pubchem_error)?;
            let built = Arc::new(BuiltPubchemSession {
                generated_at: generated_at_rfc3339(),
                bundle: Arc::new(bundle),
            });
            let _ = pubchem_session_put_built(&state, &req.session_id, built.clone());
            built
        }
    };

    let generated_at = built_session.generated_at.clone();
    let date_stamp = date_stamp(&generated_at);
    let bundle = built_session.bundle.as_ref();
    let downloads = [
        DownloadArtifactKind::BiologicalPubchem,
        DownloadArtifactKind::ChemicalWikidataPubchem,
        DownloadArtifactKind::ChemicalNpclassifierPubchem,
        DownloadArtifactKind::BiologicalFull,
        DownloadArtifactKind::ChemicalWikidataFull,
        DownloadArtifactKind::ChemicalNpclassifierFull,
    ]
    .into_iter()
    .filter(|kind| kind.available(bundle))
    .map(|kind| DownloadArtifactDto {
        key: kind.key().to_string(),
        label: kind.label().to_string(),
        url: pubchem_download_url(&req.session_id, kind),
        filename: kind.filename(&date_stamp),
    })
    .collect();

    Ok(Json(PubchemBuildResponse {
        session_id: req.session_id,
        generated_at,
        biological_summary: TreeSummaryDto::from(bundle.biological_summary),
        chemical_summary: TreeSummaryDto::from(bundle.chemical_summary),
        npclassifier_summary: TreeSummaryDto::from(bundle.npclassifier_summary),
        biological_preview: PreviewTreeDto::from(bundle.biological_preview.clone()),
        chemical_preview: PreviewTreeDto::from(bundle.chemical_preview.clone()),
        npclassifier_preview: PreviewTreeDto::from(bundle.npclassifier_preview.clone()),
        npclassifier_warning: bundle.npclassifier_warning.clone(),
        downloads,
    }))
}

#[utoipa::path(
    get,
    path = "/v1/pubchem-tree/download/{session_id}/{artifact}",
    params(
        ("session_id" = String, Path, description = "PubChem tree session id"),
        ("artifact" = String, Path, description = "Artifact key: biological|chemical-wikidata|chemical-npclassifier|biological-full|chemical-wikidata-full|chemical-npclassifier-full")
    ),
    responses(
        (status = 200, description = "JSON download artifact"),
        (status = 400, description = "Unknown session or artifact", body = ErrorResponse)
    )
)]
pub(crate) async fn pubchem_download(
    State(state): State<AppState>,
    Path((session_id, artifact_raw)): Path<(String, String)>,
) -> Result<Response, ApiError> {
    let kind = DownloadArtifactKind::parse(&artifact_raw)
        .ok_or_else(|| ApiError::bad_request("Unsupported PubChem tree artifact"))?;
    let session = pubchem_session_get(&state, &session_id)
        .ok_or_else(|| ApiError::bad_request("PubChem tree session expired or is unknown"))?;
    let built = session
        .built
        .ok_or_else(|| ApiError::bad_request("PubChem tree has not been built for this session"))?;

    if !kind.available(&built.bundle) {
        return Err(ApiError::bad_request(
            "Requested PubChem tree artifact is not available for this session",
        ));
    }

    let payload =
        build_download_json(kind, &built.bundle, &built.generated_at).map_err(map_pubchem_error)?;
    let filename = kind.filename(&date_stamp(&built.generated_at));

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json; charset=utf-8")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{filename}\""),
        )
        .header(header::CACHE_CONTROL, "private, max-age=600")
        .body(axum::body::Body::from(payload))
        .map_err(|e| ApiError::upstream(format!("response build failed: {e}")))
}

#[utoipa::path(
    get,
    path = "/v1/export-file/{cache_key}/{format}",
    params(
        ("cache_key" = String, Path, description = "Export cache key returned by /v1/export-url"),
        ("format" = String, Path, description = "Export format: csv|json|rdf"),
        ("filename" = Option<String>, Query, description = "Optional direct filename (disables gzip wrapping)")
    ),
    responses(
        (status = 200, description = "Export file bytes"),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 502, description = "Upstream export failure", body = ErrorResponse),
        (status = 503, description = "Server overloaded", body = ErrorResponse),
        (status = 504, description = "Export fetch timeout", body = ErrorResponse)
    )
)]
pub(crate) async fn export_file(
    State(state): State<AppState>,
    Path((cache_key, format_raw)): Path<(String, String)>,
    Query(params): Query<ExportFileQuery>,
) -> Result<Response, ApiError> {
    let req_started = Instant::now();
    let _permit = state.request_permits.try_acquire().map_err(|_| {
        state
            .metrics
            .overload_rejections
            .fetch_add(1, Ordering::Relaxed);
        log::warn!("event=export_file state=rejected reason=overloaded");
        ApiError::overloaded("Server is busy, retry shortly")
    })?;

    let format = ExportArchiveFormat::parse(&format_raw)
        .ok_or_else(|| ApiError::bad_request("Unsupported export format"))?;
    let cached = export_cache_get(&state, &cache_key).ok_or_else(|| {
        ApiError::bad_request("Export link expired or is unknown. Regenerate the export URL.")
    })?;

    let upstream_url = build_upstream_export_url(&cached.query, format);
    let raw_bytes = timeout(
        state.request_timeout,
        shared::sparql::fetch_url_bytes(&upstream_url),
    )
    .await
    .map_err(|_| {
        state
            .metrics
            .request_timeouts
            .fetch_add(1, Ordering::Relaxed);
        log::warn!("event=export_file state=timeout phase=fetch format={format_raw}");
        ApiError::upstream("export fetch timed out")
    })?
    .map_err(|e| ApiError::upstream(format!("export fetch failed: {e}")))?;
    let raw_len = raw_bytes.len();

    let requested_filename = params
        .filename
        .as_deref()
        .map(sanitize_download_filename)
        .filter(|name| !name.is_empty());

    let (body_bytes, content_type, attachment_name) = if let Some(filename) = requested_filename {
        (raw_bytes, format.content_type(), filename)
    } else {
        let gz_bytes = gzip_bytes(&raw_bytes)
            .map_err(|e| ApiError::upstream(format!("gzip encoding failed: {e}")))?;
        (
            gz_bytes,
            "application/gzip",
            format!("{cache_key}.{}.gz", format.extension()),
        )
    };
    log::info!(
        "event=export_file state=success elapsed_ms={:.1} format={} raw_bytes={} out_bytes={} named={}",
        req_started.elapsed().as_secs_f64() * 1000.0,
        format.extension(),
        raw_len,
        body_bytes.len(),
        params.filename.is_some(),
    );

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{attachment_name}\""),
        )
        .header(header::CACHE_CONTROL, "private, max-age=600")
        .body(axum::body::Body::from(body_bytes))
        .map_err(|e| ApiError::upstream(format!("response build failed: {e}")))
}
