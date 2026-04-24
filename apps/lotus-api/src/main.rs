use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
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

#[derive(Debug, Serialize, ToSchema)]
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

#[derive(Debug, Serialize, ToSchema)]
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

#[derive(Debug, Serialize, ToSchema)]
struct SearchResponse {
    resolved_taxon_qid: Option<String>,
    warning: Option<String>,
    query: String,
    rows: Vec<RowDto>,
    total_matches: usize,
    stats: SearchStats,
}

#[derive(Debug, Serialize, ToSchema)]
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
async fn main() {
    env_logger::init();

    let state = AppState { default_limit: 500 };

    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/search", post(search))
        .route("/v1/export-url", post(export_urls))
        .merge(SwaggerUi::new("/docs").url("/openapi.json", ApiDoc::openapi()))
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8787);
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .expect("valid bind address");
    log::info!("lotus-api listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind listener");
    axum::serve(listener, app).await.expect("serve api");
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

    let (resolved_taxon_qid, warning) = resolve_taxon_qid(criteria.taxon.clone()).await?;
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
    let display_query = queries::query_with_limit(&execution_query, limit);

    let rows_bytes = sparql::execute_sparql_bytes(&display_query)
        .await
        .map_err(|e| ApiError::upstream(format!("display query failed: {e}")))?;
    let rows = sparql::parse_compounds_csv_display_bytes(&rows_bytes, limit)
        .map_err(|e| ApiError::upstream(format!("display parse failed: {e}")))?;

    let include_counts = req.include_counts.unwrap_or(true);
    let stats = if include_counts {
        let count_query = queries::query_counts_from_base(&execution_query);
        let count_bytes = sparql::execute_sparql_bytes(&count_query)
            .await
            .map_err(|e| ApiError::upstream(format!("count query failed: {e}")))?;
        sparql::parse_counts_csv_bytes(&count_bytes)
            .map_err(|e| ApiError::upstream(format!("count parse failed: {e}")))?
    } else {
        DatasetStats::from_entries(&rows)
    };

    let total_matches = stats.n_entries;
    let rows = rows.into_iter().map(RowDto::from).collect();

    Ok(Json(SearchResponse {
        resolved_taxon_qid,
        warning,
        query: execution_query,
        rows,
        total_matches,
        stats: stats.into(),
    }))
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
async fn export_urls(Json(req): Json<SearchRequest>) -> Result<Json<ExportUrlResponse>, ApiError> {
    let mut criteria = apply_request(&req)?;
    if !criteria.is_valid() {
        return Err(ApiError::bad_request(
            "Either taxon or smiles/structure must be provided",
        ));
    }

    let (resolved_taxon_qid, _warning) = resolve_taxon_qid(criteria.taxon.clone()).await?;
    if let Some(qid) = resolved_taxon_qid.as_deref()
        && qid != "*"
    {
        criteria.taxon = qid.to_string();
    }
    let query = build_execution_query(&criteria, resolved_taxon_qid.as_deref());

    Ok(Json(ExportUrlResponse {
        csv_url: qlever_export_url(&query, "csv_export"),
        json_url: qlever_export_url(&query, "sparql_json_export"),
        rdf_url: qlever_export_url(
            &queries::query_construct_from_select(&query),
            "turtle_export",
        ),
        query,
    }))
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
        c.formula_exact = v.to_string();
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
    let smiles = criteria.smiles.trim();
    let base_query = if !smiles.is_empty() {
        let taxon_for_sachem = match resolved_taxon_qid {
            Some("*") => Some("Q2382443"),
            Some(qid) => Some(qid),
            None => None,
        };
        queries::query_sachem(
            smiles,
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
