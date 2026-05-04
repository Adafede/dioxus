// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tower_http::cors::{Any, CorsLayer};
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

#[allow(dead_code)]
#[path = "../../lotus-explorer/src/models.rs"]
mod models;
#[allow(dead_code)]
#[path = "../../lotus-explorer/src/queries.rs"]
mod queries;
#[allow(dead_code)]
#[path = "../../lotus-explorer/src/sparql.rs"]
mod sparql;

use models::{CompoundEntry, DatasetStats, SearchCriteria, SmilesSearchType, TaxonMatch};

#[derive(Clone)]
struct AppState {
    default_limit: usize,
    taxon_cache: Arc<Mutex<HashMap<String, CachedTaxonResolution>>>,
    search_cache: Arc<Mutex<HashMap<String, CachedSearchResponse>>>,
    export_cache: Arc<Mutex<HashMap<String, CachedExportResponse>>>,
}

const TAXON_CACHE_TTL: Duration = Duration::from_secs(60 * 60 * 24);
const SEARCH_CACHE_TTL: Duration = Duration::from_secs(60 * 3);
const EXPORT_CACHE_TTL: Duration = Duration::from_secs(60 * 10);
const MAX_TAXON_CACHE_ENTRIES: usize = 512;
const MAX_SEARCH_CACHE_ENTRIES: usize = 128;
const MAX_EXPORT_CACHE_ENTRIES: usize = 256;

#[derive(Clone)]
struct CachedTaxonResolution {
    inserted_at: Instant,
    value: (Option<String>, Option<String>),
}

#[derive(Clone)]
struct CachedSearchResponse {
    inserted_at: Instant,
    value: SearchResponse,
}

#[derive(Clone)]
struct CachedExportResponse {
    inserted_at: Instant,
    value: ExportUrlResponse,
}

#[derive(Debug, Clone)]
struct AppConfig {
    host: String,
    port: u16,
    default_limit: usize,
    cors_allowed_origins: Option<Vec<axum::http::HeaderValue>>,
}

impl AppConfig {
    fn from_env() -> Result<Self, String> {
        Self::from_provider(|name| std::env::var(name).ok())
    }

    fn from_provider<F>(mut get: F) -> Result<Self, String>
    where
        F: FnMut(&str) -> Option<String>,
    {
        let host = get("HOST")
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "127.0.0.1".to_string());

        let port = parse_u16_env(get("PORT"), "PORT", 8787)?;
        let default_limit = parse_usize_env(get("DEFAULT_LIMIT"), "DEFAULT_LIMIT", 500)?
            .clamp(1, models::TABLE_ROW_LIMIT);

        let app_env = get("APP_ENV")
            .map(|value| value.trim().to_ascii_lowercase())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "development".to_string());

        let cors_allowed_origins = parse_allowed_origins(get("CORS_ALLOWED_ORIGINS"))?;
        if app_env == "production" && cors_allowed_origins.is_none() {
            return Err(
                "APP_ENV=production requires CORS_ALLOWED_ORIGINS to be configured".to_string(),
            );
        }

        Ok(Self {
            host,
            port,
            default_limit,
            cors_allowed_origins,
        })
    }

    fn bind_addr(&self) -> Result<SocketAddr, String> {
        format!("{}:{}", self.host, self.port)
            .parse::<SocketAddr>()
            .map_err(|e| format!("invalid bind address '{}:{}': {e}", self.host, self.port))
    }
}

fn parse_u16_env(value: Option<String>, name: &str, default_value: u16) -> Result<u16, String> {
    match value {
        Some(raw) => raw
            .trim()
            .parse::<u16>()
            .map_err(|e| format!("{name} must be a valid u16: {e}")),
        None => Ok(default_value),
    }
}

fn parse_usize_env(
    value: Option<String>,
    name: &str,
    default_value: usize,
) -> Result<usize, String> {
    match value {
        Some(raw) => raw
            .trim()
            .parse::<usize>()
            .map_err(|e| format!("{name} must be a valid non-negative integer: {e}")),
        None => Ok(default_value),
    }
}

fn parse_allowed_origins(
    value: Option<String>,
) -> Result<Option<Vec<axum::http::HeaderValue>>, String> {
    let Some(raw) = value else {
        return Ok(None);
    };

    let mut origins = Vec::new();
    for origin in raw
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
    {
        if !origin.starts_with("http://") && !origin.starts_with("https://") {
            return Err(format!(
                "CORS_ALLOWED_ORIGINS entry '{origin}' must start with http:// or https://"
            ));
        }
        let header = axum::http::HeaderValue::from_str(origin)
            .map_err(|_| format!("CORS_ALLOWED_ORIGINS contains invalid origin '{origin}'"))?;
        origins.push(header);
    }

    if origins.is_empty() {
        Ok(None)
    } else {
        Ok(Some(origins))
    }
}

fn build_cors_layer(config: &AppConfig) -> CorsLayer {
    let layer = CorsLayer::new().allow_methods(Any).allow_headers(Any);
    match &config.cors_allowed_origins {
        Some(origins) => layer.allow_origin(origins.clone()),
        None => layer.allow_origin(Any),
    }
}

#[derive(Debug, Serialize, ToSchema)]
struct ErrorResponse {
    error: String,
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    fn upstream(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_GATEWAY,
            message: message.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = Json(ErrorResponse {
            error: self.message,
        });
        (self.status, body).into_response()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
enum ApiSmilesSearchType {
    Substructure,
    Similarity,
}

impl From<ApiSmilesSearchType> for SmilesSearchType {
    fn from(value: ApiSmilesSearchType) -> Self {
        match value {
            ApiSmilesSearchType::Substructure => SmilesSearchType::Substructure,
            ApiSmilesSearchType::Similarity => SmilesSearchType::Similarity,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
enum ApiElementState {
    Allowed,
    Required,
    Excluded,
}

impl From<ApiElementState> for models::ElementState {
    fn from(value: ApiElementState) -> Self {
        match value {
            ApiElementState::Allowed => models::ElementState::Allowed,
            ApiElementState::Required => models::ElementState::Required,
            ApiElementState::Excluded => models::ElementState::Excluded,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
struct SearchRequest {
    taxon: Option<String>,
    smiles: Option<String>,
    smiles_search_type: Option<ApiSmilesSearchType>,
    smiles_threshold: Option<f64>,
    mass_min: Option<f64>,
    mass_max: Option<f64>,
    year_min: Option<u16>,
    year_max: Option<u16>,
    formula_exact: Option<String>,
    c_min: Option<u16>,
    c_max: Option<u16>,
    h_min: Option<u16>,
    h_max: Option<u16>,
    n_min: Option<u16>,
    n_max: Option<u16>,
    o_min: Option<u16>,
    o_max: Option<u16>,
    p_min: Option<u16>,
    p_max: Option<u16>,
    s_min: Option<u16>,
    s_max: Option<u16>,
    f_state: Option<ApiElementState>,
    cl_state: Option<ApiElementState>,
    br_state: Option<ApiElementState>,
    i_state: Option<ApiElementState>,
    limit: Option<usize>,
    include_counts: Option<bool>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
struct SearchStats {
    n_compounds: usize,
    n_taxa: usize,
    n_references: usize,
    n_entries: usize,
    n_entries_unique: usize,
}

impl From<DatasetStats> for SearchStats {
    fn from(value: DatasetStats) -> Self {
        Self {
            n_compounds: value.n_compounds,
            n_taxa: value.n_taxa,
            n_references: value.n_references,
            n_entries: value.n_entries,
            n_entries_unique: value.n_entries_unique,
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
struct RowDto {
    compound_qid: String,
    name: String,
    inchikey: Option<String>,
    smiles: Option<String>,
    mass: Option<f64>,
    formula: Option<String>,
    taxon_qid: String,
    taxon_name: String,
    reference_qid: String,
    ref_title: Option<String>,
    ref_doi: Option<String>,
    pub_year: Option<i16>,
    statement: Option<String>,
}

impl From<CompoundEntry> for RowDto {
    fn from(value: CompoundEntry) -> Self {
        Self {
            compound_qid: value.compound_qid,
            name: value.name.as_ref().to_string(),
            inchikey: value.inchikey,
            smiles: value.smiles,
            mass: value.mass,
            formula: value.formula.map(|v| v.as_ref().to_string()),
            taxon_qid: value.taxon_qid,
            taxon_name: value.taxon_name.as_ref().to_string(),
            reference_qid: value.reference_qid,
            ref_title: value.ref_title.map(|v| v.as_ref().to_string()),
            ref_doi: value.ref_doi,
            pub_year: value.pub_year,
            statement: value.statement,
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
struct SearchResponse {
    resolved_taxon_qid: Option<String>,
    warning: Option<String>,
    query: String,
    rows: Vec<RowDto>,
    total_matches: usize,
    stats: SearchStats,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
struct ExportUrlResponse {
    query: String,
    csv_url: String,
    json_url: String,
    rdf_url: String,
}

#[derive(OpenApi)]
#[openapi(
    paths(health, search, export_urls),
    components(
        schemas(
            SearchRequest,
            SearchResponse,
            SearchStats,
            RowDto,
            ExportUrlResponse,
            ErrorResponse,
            ApiSmilesSearchType,
            ApiElementState
        )
    ),
    tags((name = "lotus-api", description = "LOTUS explorer programmatic API"))
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let config = AppConfig::from_env().map_err(std::io::Error::other)?;

    let state = AppState {
        default_limit: config.default_limit,
        taxon_cache: Arc::new(Mutex::new(HashMap::new())),
        search_cache: Arc::new(Mutex::new(HashMap::new())),
        export_cache: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/search", post(search))
        .route("/v1/export-url", post(export_urls))
        .merge(SwaggerUi::new("/docs").url("/openapi.json", ApiDoc::openapi()))
        .with_state(state)
        .layer(build_cors_layer(&config));

    let addr = config.bind_addr().map_err(std::io::Error::other)?;
    log::info!("lotus-api listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

#[utoipa::path(get, path = "/health", responses((status = 200, description = "Service health")))]
async fn health() -> &'static str {
    "ok"
}

#[utoipa::path(
    post,
    path = "/v1/search",
    request_body = SearchRequest,
    responses(
        (status = 200, description = "Search results", body = SearchResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 502, description = "Upstream SPARQL failure", body = ErrorResponse)
    )
)]
async fn search(
    State(state): State<AppState>,
    Json(req): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, ApiError> {
    let mut criteria = apply_request(&req)?;
    if !criteria.is_valid() {
        return Err(ApiError::bad_request(
            "Either taxon or smiles/structure must be provided",
        ));
    }

    let (resolved_taxon_qid, warning) = resolve_taxon_qid_cached(&state, criteria.taxon.clone()).await?;
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
    let cache_key = build_search_cache_key(&execution_query, limit, include_counts);
    if let Some(cached) = search_cache_get(&state, &cache_key) {
        return Ok(Json(cached));
    }

    let response = build_search_response(&execution_query, limit, include_counts, resolved_taxon_qid.clone(), warning.clone()).await?;
    search_cache_put(&state, cache_key, response.clone());

    Ok(Json(response))
}

async fn build_search_response(
    execution_query: &str,
    limit: usize,
    include_counts: bool,
    resolved_taxon_qid: Option<String>,
    warning: Option<String>,
) -> Result<SearchResponse, ApiError> {
    let display_query = queries::query_with_limit(execution_query, limit);
    let count_query = queries::query_counts_from_base(execution_query);

    let rows = if include_counts {
        let rows_future = async {
            let rows_bytes = sparql::execute_sparql_bytes(&display_query)
                .await
                .map_err(|e| ApiError::upstream(format!("display query failed: {e}")))?;
            sparql::parse_compounds_csv_display_bytes(&rows_bytes, limit)
                .map_err(|e| ApiError::upstream(format!("display parse failed: {e}")))
        };
        let stats_future = async {
            let count_bytes = sparql::execute_sparql_bytes(&count_query)
                .await
                .map_err(|e| ApiError::upstream(format!("count query failed: {e}")))?;
            sparql::parse_counts_csv_bytes(&count_bytes)
                .map_err(|e| ApiError::upstream(format!("count parse failed: {e}")))
        };

        let (rows, stats) = tokio::try_join!(rows_future, stats_future)?;
        return Ok(SearchResponse {
            resolved_taxon_qid,
            warning,
            query: execution_query.to_string(),
            total_matches: stats.n_entries,
            stats: SearchStats::from(stats),
            rows: rows.into_iter().map(RowDto::from).collect(),
        });
    } else {
        let rows_bytes = sparql::execute_sparql_bytes(&display_query)
            .await
            .map_err(|e| ApiError::upstream(format!("display query failed: {e}")))?;
        sparql::parse_compounds_csv_display_bytes(&rows_bytes, limit)
            .map_err(|e| ApiError::upstream(format!("display parse failed: {e}")))?
    };

    let stats = DatasetStats::from_entries(&rows);

    Ok(SearchResponse {
        resolved_taxon_qid,
        warning,
        query: execution_query.to_string(),
        rows: rows.into_iter().map(RowDto::from).collect(),
        total_matches: stats.n_entries,
        stats: stats.into(),
    })
}

#[utoipa::path(
    post,
    path = "/v1/export-url",
    request_body = SearchRequest,
    responses(
        (status = 200, description = "Direct export URLs", body = ExportUrlResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 502, description = "Upstream SPARQL failure", body = ErrorResponse)
    )
)]
async fn export_urls(
    State(state): State<AppState>,
    Json(req): Json<SearchRequest>,
) -> Result<Json<ExportUrlResponse>, ApiError> {
    let mut criteria = apply_request(&req)?;
    if !criteria.is_valid() {
        return Err(ApiError::bad_request(
            "Either taxon or smiles/structure must be provided",
        ));
    }

    let (resolved_taxon_qid, _warning) = resolve_taxon_qid_cached(&state, criteria.taxon.clone()).await?;
    if let Some(qid) = resolved_taxon_qid.as_deref()
        && qid != "*"
    {
        criteria.taxon = qid.to_string();
    }
    let query = build_execution_query(&criteria, resolved_taxon_qid.as_deref());

    if let Some(cached) = export_cache_get(&state, &query) {
        return Ok(Json(cached));
    }

    let response = ExportUrlResponse {
        csv_url: qlever_export_url(&query, "csv_export"),
        json_url: qlever_export_url(&query, "sparql_json_export"),
        rdf_url: qlever_export_url(
            &queries::query_construct_from_select(&query),
            "turtle_export",
        ),
        query,
    };
    export_cache_put(&state, response.query.clone(), response.clone());

    Ok(Json(response))
}

fn apply_request(req: &SearchRequest) -> Result<SearchCriteria, ApiError> {
    let mut c = SearchCriteria {
        taxon: req.taxon.clone().unwrap_or_default(),
        smiles: req.smiles.clone().unwrap_or_default(),
        ..Default::default()
    };

    if let Some(v) = req.smiles_search_type {
        c.smiles_search_type = v.into();
    }
    if let Some(v) = req.smiles_threshold {
        c.smiles_threshold = v.clamp(0.05, 1.0);
    }
    if let Some(v) = req.mass_min {
        c.mass_min = v.max(0.0);
    }
    if let Some(v) = req.mass_max {
        c.mass_max = v.max(c.mass_min);
    }
    if let Some(v) = req.year_min {
        c.year_min = v;
    }
    if let Some(v) = req.year_max {
        c.year_max = v.max(c.year_min);
    }

    let has_formula_input = req
        .formula_exact
        .as_deref()
        .is_some_and(|v| !v.trim().is_empty())
        || req.c_min.is_some()
        || req.c_max.is_some()
        || req.h_min.is_some()
        || req.h_max.is_some()
        || req.n_min.is_some()
        || req.n_max.is_some()
        || req.o_min.is_some()
        || req.o_max.is_some()
        || req.p_min.is_some()
        || req.p_max.is_some()
        || req.s_min.is_some()
        || req.s_max.is_some()
        || req.f_state.is_some()
        || req.cl_state.is_some()
        || req.br_state.is_some()
        || req.i_state.is_some();

    c.formula_enabled = has_formula_input;
    if let Some(v) = req.formula_exact.as_deref() {
        c.formula_exact = v.trim().to_string();
    }

    c.c_min = req.c_min.unwrap_or(c.c_min);
    c.c_max = req.c_max.unwrap_or(c.c_max);
    c.h_min = req.h_min.unwrap_or(c.h_min);
    c.h_max = req.h_max.unwrap_or(c.h_max);
    c.n_min = req.n_min.unwrap_or(c.n_min);
    c.n_max = req.n_max.unwrap_or(c.n_max);
    c.o_min = req.o_min.unwrap_or(c.o_min);
    c.o_max = req.o_max.unwrap_or(c.o_max);
    c.p_min = req.p_min.unwrap_or(c.p_min);
    c.p_max = req.p_max.unwrap_or(c.p_max);
    c.s_min = req.s_min.unwrap_or(c.s_min);
    c.s_max = req.s_max.unwrap_or(c.s_max);

    if c.c_min > c.c_max
        || c.h_min > c.h_max
        || c.n_min > c.n_max
        || c.o_min > c.o_max
        || c.p_min > c.p_max
        || c.s_min > c.s_max
    {
        return Err(ApiError::bad_request("Element min must be <= max"));
    }

    if let Some(v) = req.f_state {
        c.f_state = v.into();
    }
    if let Some(v) = req.cl_state {
        c.cl_state = v.into();
    }
    if let Some(v) = req.br_state {
        c.br_state = v.into();
    }
    if let Some(v) = req.i_state {
        c.i_state = v.into();
    }

    Ok(c)
}

fn build_execution_query(criteria: &SearchCriteria, resolved_taxon_qid: Option<&str>) -> String {
    let smiles = normalized_structure_input(&criteria.smiles);
    let base_query = if !smiles.is_empty() {
        let taxon_for_sachem = match resolved_taxon_qid {
            Some("*") => Some("Q2382443"),
            Some(qid) => Some(qid),
            None => None,
        };
        queries::query_sachem(
            &smiles,
            criteria.smiles_search_type,
            criteria.smiles_threshold,
            taxon_for_sachem,
        )
    } else {
        match resolved_taxon_qid {
            Some("*") | None => queries::query_all_compounds(),
            Some(qid) => queries::query_compounds_by_taxon(qid),
        }
    };

    queries::query_with_server_filters(&base_query, criteria)
}

fn normalized_structure_input(value: &str) -> String {
    let normalized = value.replace("\r\n", "\n").replace('\r', "\n");
    match queries::classify_structure(&normalized) {
        queries::StructureKind::MolfileV2000 | queries::StructureKind::MolfileV3000 => normalized,
        _ => normalized.trim().to_string(),
    }
}

async fn resolve_taxon_qid_cached(
    state: &AppState,
    taxon_input: String,
) -> Result<(Option<String>, Option<String>), ApiError> {
    let key = taxon_input.trim().to_lowercase();
    if !key.is_empty()
        && let Some(cached) = taxon_cache_get(state, &key)
    {
        return Ok(cached);
    }

    let resolved = resolve_taxon_qid(taxon_input).await?;
    if !key.is_empty() {
        taxon_cache_put(state, key, resolved.clone());
    }
    Ok(resolved)
}

async fn resolve_taxon_qid(
    taxon_input: String,
) -> Result<(Option<String>, Option<String>), ApiError> {
    let taxon = taxon_input.trim().to_string();
    if taxon.is_empty() {
        return Ok((None, None));
    }
    if taxon == "*" {
        return Ok((Some("*".to_string()), None));
    }
    if is_qid(&taxon) {
        return Ok((Some(taxon.to_ascii_uppercase()), None));
    }

    let sanitized = sanitize_taxon_input(&taxon);
    let query = queries::query_taxon_search(&sanitized);
    let csv = sparql::execute_sparql_bytes(&query)
        .await
        .map_err(|e| ApiError::upstream(format!("taxon lookup failed: {e}")))?;
    let matches = sparql::parse_taxon_csv_bytes(&csv)
        .map_err(|e| ApiError::upstream(format!("taxon parse failed: {e}")))?;

    if matches.is_empty() {
        return Err(ApiError::bad_request(format!("Taxon not found: {taxon}")));
    }

    let lower = sanitized.to_lowercase();
    let exact: Vec<&TaxonMatch> = matches
        .iter()
        .filter(|m| m.name.to_lowercase() == lower)
        .collect();
    let best = exact
        .first()
        .copied()
        .or_else(|| matches.first())
        .ok_or_else(|| ApiError::bad_request("Could not resolve taxon"))?;

    let warning = if sanitized != taxon {
        Some(format!(
            "Taxon normalized from '{taxon}' to '{}' ({}).",
            best.name, best.qid
        ))
    } else if exact.len() > 1 || (exact.is_empty() && matches.len() > 1) {
        Some(format!(
            "Ambiguous taxon input. Using '{}' ({})",
            best.name, best.qid
        ))
    } else {
        None
    };

    Ok((Some(best.qid.clone()), warning))
}

fn sanitize_taxon_input(taxon: &str) -> String {
    let replaced = taxon.replace('_', " ");
    let parts: Vec<&str> = replaced.split_whitespace().collect();
    if parts.len() <= 1 {
        return replaced;
    }

    let first = parts[0];
    if first.is_empty() {
        return replaced;
    }

    let mut first_cap = String::with_capacity(first.len());
    let mut chars = first.chars();
    if let Some(c) = chars.next() {
        for uc in c.to_uppercase() {
            first_cap.push(uc);
        }
    }
    for c in chars {
        for lc in c.to_lowercase() {
            first_cap.push(lc);
        }
    }

    let mut out = first_cap;
    out.push(' ');
    out.push_str(&parts[1..].join(" "));
    out
}

fn is_qid(value: &str) -> bool {
    let v = value.trim();
    if v.len() < 2 {
        return false;
    }
    let mut chars = v.chars();
    let first = chars.next().unwrap_or_default();
    (first == 'Q' || first == 'q') && chars.all(|c| c.is_ascii_digit())
}

fn qlever_export_url(query: &str, action: &str) -> String {
    format!(
        "{}?query={}&action={action}",
        shared::sparql::QLEVER_WIKIDATA,
        urlencoding::encode(query)
    )
}

fn build_search_cache_key(query: &str, limit: usize, include_counts: bool) -> String {
    format!("limit={limit}|include_counts={include_counts}|query={query}")
}

fn search_cache_get(state: &AppState, key: &str) -> Option<SearchResponse> {
    let mut cache = state.search_cache.lock().ok()?;
    prune_cache(&mut cache, SEARCH_CACHE_TTL, MAX_SEARCH_CACHE_ENTRIES, |entry| entry.inserted_at);
    cache.get(key).map(|entry| entry.value.clone())
}

fn search_cache_put(state: &AppState, key: String, value: SearchResponse) {
    if let Ok(mut cache) = state.search_cache.lock() {
        prune_cache(&mut cache, SEARCH_CACHE_TTL, MAX_SEARCH_CACHE_ENTRIES, |entry| entry.inserted_at);
        cache.insert(
            key,
            CachedSearchResponse {
                inserted_at: Instant::now(),
                value,
            },
        );
    }
}

fn export_cache_get(state: &AppState, key: &str) -> Option<ExportUrlResponse> {
    let mut cache = state.export_cache.lock().ok()?;
    prune_cache(&mut cache, EXPORT_CACHE_TTL, MAX_EXPORT_CACHE_ENTRIES, |entry| entry.inserted_at);
    cache.get(key).map(|entry| entry.value.clone())
}

fn export_cache_put(state: &AppState, key: String, value: ExportUrlResponse) {
    if let Ok(mut cache) = state.export_cache.lock() {
        prune_cache(&mut cache, EXPORT_CACHE_TTL, MAX_EXPORT_CACHE_ENTRIES, |entry| entry.inserted_at);
        cache.insert(
            key,
            CachedExportResponse {
                inserted_at: Instant::now(),
                value,
            },
        );
    }
}

fn taxon_cache_get(state: &AppState, key: &str) -> Option<(Option<String>, Option<String>)> {
    let mut cache = state.taxon_cache.lock().ok()?;
    prune_cache(&mut cache, TAXON_CACHE_TTL, MAX_TAXON_CACHE_ENTRIES, |entry| entry.inserted_at);
    cache.get(key).map(|entry| entry.value.clone())
}

fn taxon_cache_put(state: &AppState, key: String, value: (Option<String>, Option<String>)) {
    if let Ok(mut cache) = state.taxon_cache.lock() {
        prune_cache(&mut cache, TAXON_CACHE_TTL, MAX_TAXON_CACHE_ENTRIES, |entry| entry.inserted_at);
        cache.insert(
            key,
            CachedTaxonResolution {
                inserted_at: Instant::now(),
                value,
            },
        );
    }
}

fn prune_cache<V, F>(cache: &mut HashMap<String, V>, ttl: Duration, max_entries: usize, inserted_at: F)
where
    F: Fn(&V) -> Instant,
{
    let now = Instant::now();
    cache.retain(|_, value| now.duration_since(inserted_at(value)) <= ttl);
    while cache.len() > max_entries {
        let Some(oldest_key) = cache
            .iter()
            .min_by_key(|(_, value)| inserted_at(value))
            .map(|(key, _)| key.clone())
        else {
            break;
        };
        cache.remove(&oldest_key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn map_provider(values: &[(&str, &str)]) -> HashMap<String, String> {
        values
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn supports_u16_formula_ranges() {
        let req = SearchRequest {
            taxon: Some("*".to_string()),
            smiles: None,
            smiles_search_type: None,
            smiles_threshold: None,
            mass_min: None,
            mass_max: None,
            year_min: None,
            year_max: None,
            formula_exact: None,
            c_min: Some(1),
            c_max: Some(300),
            h_min: None,
            h_max: None,
            n_min: None,
            n_max: None,
            o_min: None,
            o_max: None,
            p_min: None,
            p_max: None,
            s_min: None,
            s_max: None,
            f_state: None,
            cl_state: None,
            br_state: None,
            i_state: None,
            limit: None,
            include_counts: None,
        };

        let c = apply_request(&req).expect("valid criteria");
        assert_eq!(c.c_max, 300);
    }

    #[test]
    fn config_uses_safe_defaults() {
        let env = HashMap::<String, String>::new();
        let cfg = AppConfig::from_provider(|name| env.get(name).cloned()).expect("valid config");
        assert_eq!(cfg.host, "127.0.0.1");
        assert_eq!(cfg.port, 8787);
        assert_eq!(cfg.default_limit, 500);
        assert!(cfg.cors_allowed_origins.is_none());
    }

    #[test]
    fn config_rejects_invalid_port() {
        let env = map_provider(&[("PORT", "abc")]);
        let err = AppConfig::from_provider(|name| env.get(name).cloned())
            .expect_err("invalid PORT should fail");
        assert!(err.contains("PORT"));
    }

    #[test]
    fn production_requires_explicit_cors_allowlist() {
        let env = map_provider(&[("APP_ENV", "production")]);
        let err = AppConfig::from_provider(|name| env.get(name).cloned())
            .expect_err("production without CORS origins should fail");
        assert!(err.contains("CORS_ALLOWED_ORIGINS"));
    }

    #[test]
    fn parses_comma_separated_cors_origins() {
        let env = map_provider(&[(
            "CORS_ALLOWED_ORIGINS",
            "https://api.example.org, http://localhost:5173",
        )]);
        let cfg = AppConfig::from_provider(|name| env.get(name).cloned()).expect("valid config");
        assert_eq!(cfg.cors_allowed_origins.as_ref().map(Vec::len), Some(2));
    }

    #[test]
    fn normalized_structure_preserves_multiline_molfile() {
        let molfile = "\n  Mrv\n\n  0  0  0  0  0  0            999 V3000\nM  END\n";
        let normalized = normalized_structure_input(molfile);
        assert!(normalized.starts_with('\n'));
        assert!(normalized.contains("V3000"));
    }

    #[test]
    fn prune_cache_removes_oldest_when_over_capacity() {
        let mut cache = HashMap::from([
            (
                "a".to_string(),
                CachedExportResponse {
                    inserted_at: Instant::now() - Duration::from_secs(30),
                    value: ExportUrlResponse {
                        query: "a".into(),
                        csv_url: "a".into(),
                        json_url: "a".into(),
                        rdf_url: "a".into(),
                    },
                },
            ),
            (
                "b".to_string(),
                CachedExportResponse {
                    inserted_at: Instant::now(),
                    value: ExportUrlResponse {
                        query: "b".into(),
                        csv_url: "b".into(),
                        json_url: "b".into(),
                        rdf_url: "b".into(),
                    },
                },
            ),
        ]);

        prune_cache(&mut cache, Duration::from_secs(60), 1, |entry| entry.inserted_at);
        assert!(cache.contains_key("b"));
        assert!(!cache.contains_key("a"));
    }
}
