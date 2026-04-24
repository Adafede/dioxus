use crate::models::{CompoundEntry, DatasetStats, ElementState, SearchCriteria, SmilesSearchType};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;

#[derive(Debug)]
pub enum ApiClientError {
    NotConfigured,
    Network(String),
    Http(u16, String),
    Parse(String),
}

impl fmt::Display for ApiClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotConfigured => write!(f, "LOTUS API not configured"),
            Self::Network(e) => write!(f, "Network error: {e}"),
            Self::Http(code, body) => write!(f, "HTTP {code}: {body}"),
            Self::Parse(e) => write!(f, "Parse error: {e}"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
enum ApiSmilesSearchType {
    Substructure,
    Similarity,
}

impl From<SmilesSearchType> for ApiSmilesSearchType {
    fn from(value: SmilesSearchType) -> Self {
        match value {
            SmilesSearchType::Substructure => Self::Substructure,
            SmilesSearchType::Similarity => Self::Similarity,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
enum ApiElementState {
    Allowed,
    Required,
    Excluded,
}

impl From<ElementState> for ApiElementState {
    fn from(value: ElementState) -> Self {
        match value {
            ElementState::Allowed => Self::Allowed,
            ElementState::Required => Self::Required,
            ElementState::Excluded => Self::Excluded,
        }
    }
}

#[derive(Debug, Serialize)]
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

impl SearchRequest {
    fn from_criteria(criteria: &SearchCriteria, limit: usize, include_counts: bool) -> Self {
        let taxon = criteria.taxon.trim();
        let smiles = criteria.smiles.trim();
        let formula_exact = criteria.formula_exact.trim();

        Self {
            taxon: (!taxon.is_empty()).then(|| taxon.to_string()),
            smiles: (!smiles.is_empty()).then(|| smiles.to_string()),
            smiles_search_type: (!smiles.is_empty()).then_some(criteria.smiles_search_type.into()),
            smiles_threshold: (criteria.smiles_search_type == SmilesSearchType::Similarity
                && !smiles.is_empty())
            .then_some(criteria.smiles_threshold),
            mass_min: criteria.has_mass_filter().then_some(criteria.mass_min),
            mass_max: criteria.has_mass_filter().then_some(criteria.mass_max),
            year_min: criteria.has_year_filter().then_some(criteria.year_min),
            year_max: criteria.has_year_filter().then_some(criteria.year_max),
            formula_exact: (!formula_exact.is_empty()).then(|| formula_exact.to_string()),
            c_min: criteria.formula_enabled.then_some(criteria.c_min),
            c_max: criteria.formula_enabled.then_some(criteria.c_max),
            h_min: criteria.formula_enabled.then_some(criteria.h_min),
            h_max: criteria.formula_enabled.then_some(criteria.h_max),
            n_min: criteria.formula_enabled.then_some(criteria.n_min),
            n_max: criteria.formula_enabled.then_some(criteria.n_max),
            o_min: criteria.formula_enabled.then_some(criteria.o_min),
            o_max: criteria.formula_enabled.then_some(criteria.o_max),
            p_min: criteria.formula_enabled.then_some(criteria.p_min),
            p_max: criteria.formula_enabled.then_some(criteria.p_max),
            s_min: criteria.formula_enabled.then_some(criteria.s_min),
            s_max: criteria.formula_enabled.then_some(criteria.s_max),
            f_state: criteria.formula_enabled.then_some(criteria.f_state.into()),
            cl_state: criteria.formula_enabled.then_some(criteria.cl_state.into()),
            br_state: criteria.formula_enabled.then_some(criteria.br_state.into()),
            i_state: criteria.formula_enabled.then_some(criteria.i_state.into()),
            limit: Some(limit),
            include_counts: Some(include_counts),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    pub resolved_taxon_qid: Option<String>,
    pub warning: Option<String>,
    pub query: String,
    pub rows: Vec<RowDto>,
    pub total_matches: usize,
    pub stats: SearchStats,
}

#[derive(Debug, Deserialize)]
pub struct SearchStats {
    pub n_compounds: usize,
    pub n_taxa: usize,
    pub n_references: usize,
    pub n_entries: usize,
}

impl From<SearchStats> for DatasetStats {
    fn from(value: SearchStats) -> Self {
        Self {
            n_compounds: value.n_compounds,
            n_taxa: value.n_taxa,
            n_references: value.n_references,
            n_entries: value.n_entries,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RowDto {
    pub compound_qid: String,
    pub name: String,
    pub inchikey: Option<String>,
    pub smiles: Option<String>,
    pub mass: Option<f64>,
    pub formula: Option<String>,
    pub taxon_qid: String,
    pub taxon_name: String,
    pub reference_qid: String,
    pub ref_title: Option<String>,
    pub ref_doi: Option<String>,
    pub pub_year: Option<i16>,
    pub statement: Option<String>,
}

impl From<RowDto> for CompoundEntry {
    fn from(value: RowDto) -> Self {
        Self {
            compound_qid: value.compound_qid,
            name: Arc::<str>::from(value.name),
            inchikey: value.inchikey,
            smiles: value.smiles,
            mass: value.mass,
            formula: value.formula.map(Arc::<str>::from),
            taxon_qid: value.taxon_qid,
            taxon_name: Arc::<str>::from(value.taxon_name),
            reference_qid: value.reference_qid,
            ref_title: value.ref_title.map(Arc::<str>::from),
            ref_doi: value.ref_doi,
            pub_year: value.pub_year,
            statement: value.statement,
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Deserialize)]
pub struct ExportUrlResponse {
    pub csv_url: String,
    pub json_url: String,
    pub rdf_url: String,
}

pub async fn search(
    criteria: &SearchCriteria,
    limit: usize,
    include_counts: bool,
) -> Result<SearchResponse, ApiClientError> {
    let base = api_base_url().ok_or(ApiClientError::NotConfigured)?;
    let request = SearchRequest::from_criteria(criteria, limit, include_counts);
    post_json(&base, "/v1/search", &request).await
}

#[cfg(target_arch = "wasm32")]
pub async fn export_urls(criteria: &SearchCriteria) -> Result<ExportUrlResponse, ApiClientError> {
    let base = api_base_url().ok_or(ApiClientError::NotConfigured)?;
    let request = SearchRequest::from_criteria(criteria, 1, false);
    post_json(&base, "/v1/export-url", &request).await
}

pub fn api_base_url() -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(base) = runtime_query_param("api_base") {
            if let Some(normalized) = normalize_api_base(&base) {
                return Some(normalized);
            }
        }
    }

    if let Some(base) = option_env!("LOTUS_API_BASE")
        && let Some(normalized) = normalize_api_base(base)
    {
        return Some(normalized);
    }

    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(hostname) = window.location().hostname() {
                let hostname = hostname.to_ascii_lowercase();
                if hostname == "localhost" || hostname == "127.0.0.1" {
                    return Some("http://127.0.0.1:8787".to_string());
                }
            }
        }
    }

    None
}

fn normalize_api_base(value: &str) -> Option<String> {
    let trimmed = value.trim().trim_end_matches('/');
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

#[cfg(target_arch = "wasm32")]
fn runtime_query_param(name: &str) -> Option<String> {
    let window = web_sys::window()?;
    let search = window.location().search().ok()?;
    let query = search.trim_start_matches('?');
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let mut parts = pair.splitn(2, '=');
        let key = parts.next().unwrap_or_default();
        let value = parts.next().unwrap_or_default();
        let decoded_key = urlencoding::decode(key).ok()?;
        if decoded_key == name {
            return urlencoding::decode(value).ok().map(|v| v.into_owned());
        }
    }
    None
}

async fn post_json<Req, Res>(base: &str, path: &str, body: &Req) -> Result<Res, ApiClientError>
where
    Req: Serialize + ?Sized,
    Res: for<'de> Deserialize<'de>,
{
    let url = format!("{}{}", base.trim_end_matches('/'), path);
    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .json(body)
        .send()
        .await
        .map_err(|e| ApiClientError::Network(e.to_string()))?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(ApiClientError::Http(status.as_u16(), body));
    }

    response
        .json::<Res>()
        .await
        .map_err(|e| ApiClientError::Parse(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_builder_keeps_large_formula_ranges() {
        let mut criteria = SearchCriteria {
            taxon: "*".into(),
            ..SearchCriteria::default()
        };
        criteria.formula_enabled = true;
        criteria.c_max = 300;
        criteria.h_max = 900;

        let request = SearchRequest::from_criteria(&criteria, 123, true);
        assert_eq!(request.c_max, Some(300));
        assert_eq!(request.h_max, Some(900));
        assert_eq!(request.limit, Some(123));
        assert_eq!(request.include_counts, Some(true));
    }

    #[test]
    fn normalize_base_trims_trailing_slash() {
        assert_eq!(
            normalize_api_base("https://api.example.org/"),
            Some("https://api.example.org".to_string())
        );
    }
}
