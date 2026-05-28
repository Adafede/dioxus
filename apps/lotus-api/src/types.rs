// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use serde::{Deserialize, Serialize};
use shared::lotus::models::{CompoundEntry, DatasetStats, SmilesSearchType};
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct HealthResponse {
    pub(crate) status: &'static str,
    pub(crate) uptime_secs: u64,
    pub(crate) search_cache_hits: u64,
    pub(crate) search_cache_misses: u64,
    pub(crate) search_inflight_waits: u64,
    pub(crate) search_upstream_hits: u64,
    pub(crate) export_cache_hits: u64,
    pub(crate) export_cache_misses: u64,
    pub(crate) export_inflight_waits: u64,
    pub(crate) export_upstream_hits: u64,
    pub(crate) overload_rejections: u64,
    pub(crate) request_timeouts: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ApiSmilesSearchType {
    Substructure,
    Similarity,
}

impl From<ApiSmilesSearchType> for SmilesSearchType {
    fn from(value: ApiSmilesSearchType) -> Self {
        match value {
            ApiSmilesSearchType::Substructure => Self::Substructure,
            ApiSmilesSearchType::Similarity => Self::Similarity,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ApiElementState {
    Allowed,
    Required,
    Excluded,
}

impl From<ApiElementState> for shared::lotus::models::ElementState {
    fn from(value: ApiElementState) -> Self {
        match value {
            ApiElementState::Allowed => Self::Allowed,
            ApiElementState::Required => Self::Required,
            ApiElementState::Excluded => Self::Excluded,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct SearchRequest {
    pub(crate) taxon: Option<String>,
    pub(crate) smiles: Option<String>,
    pub(crate) smiles_search_type: Option<ApiSmilesSearchType>,
    pub(crate) smiles_threshold: Option<f64>,
    pub(crate) mass_min: Option<f64>,
    pub(crate) mass_max: Option<f64>,
    pub(crate) year_min: Option<u16>,
    pub(crate) year_max: Option<u16>,
    pub(crate) formula_exact: Option<String>,
    pub(crate) c_min: Option<u16>,
    pub(crate) c_max: Option<u16>,
    pub(crate) h_min: Option<u16>,
    pub(crate) h_max: Option<u16>,
    pub(crate) n_min: Option<u16>,
    pub(crate) n_max: Option<u16>,
    pub(crate) o_min: Option<u16>,
    pub(crate) o_max: Option<u16>,
    pub(crate) p_min: Option<u16>,
    pub(crate) p_max: Option<u16>,
    pub(crate) s_min: Option<u16>,
    pub(crate) s_max: Option<u16>,
    pub(crate) f_state: Option<ApiElementState>,
    pub(crate) cl_state: Option<ApiElementState>,
    pub(crate) br_state: Option<ApiElementState>,
    pub(crate) i_state: Option<ApiElementState>,
    pub(crate) limit: Option<usize>,
    pub(crate) include_counts: Option<bool>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct SearchStats {
    pub(crate) n_compounds: usize,
    pub(crate) n_taxa: usize,
    pub(crate) n_references: usize,
    pub(crate) n_entries: usize,
    pub(crate) n_entries_unique: usize,
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
pub(crate) struct RowDto {
    pub(crate) compound_qid: String,
    pub(crate) name: String,
    pub(crate) inchikey: Option<String>,
    pub(crate) smiles: Option<String>,
    pub(crate) mass: Option<f64>,
    pub(crate) formula: Option<String>,
    pub(crate) taxon_qid: String,
    pub(crate) taxon_name: String,
    pub(crate) reference_qid: String,
    pub(crate) ref_title: Option<String>,
    pub(crate) ref_doi: Option<String>,
    pub(crate) pub_year: Option<i16>,
    pub(crate) statement: Option<String>,
}

impl From<CompoundEntry> for RowDto {
    fn from(value: CompoundEntry) -> Self {
        Self {
            compound_qid: value.compound_qid.to_string(),
            name: value.name.to_string(),
            inchikey: value.inchikey.map(|v| v.to_string()),
            smiles: value.smiles.map(|v| v.to_string()),
            mass: value.mass,
            formula: value.formula.map(|v| v.to_string()),
            taxon_qid: value.taxon_qid.to_string(),
            taxon_name: value.taxon_name.to_string(),
            reference_qid: value.reference_qid.to_string(),
            ref_title: value.ref_title.map(|v| v.to_string()),
            ref_doi: value.ref_doi.map(|v| v.to_string()),
            pub_year: value.pub_year,
            statement: value.statement.map(|v| v.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct SearchResponse {
    pub(crate) resolved_taxon_qid: Option<String>,
    pub(crate) warning: Option<String>,
    pub(crate) query: String,
    pub(crate) rows: Vec<RowDto>,
    pub(crate) total_matches: usize,
    pub(crate) stats: SearchStats,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct ExportUrlResponse {
    pub(crate) query: String,
    pub(crate) csv_url: String,
    pub(crate) json_url: String,
    pub(crate) rdf_url: String,
    pub(crate) csv_gz_url: String,
    pub(crate) json_gz_url: String,
    pub(crate) rdf_gz_url: String,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct ExportFileQuery {
    pub(crate) filename: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ExportArchiveFormat {
    Csv,
    Json,
    Rdf,
}

impl ExportArchiveFormat {
    pub(crate) fn parse(raw: &str) -> Option<Self> {
        match raw {
            "csv" => Some(Self::Csv),
            "json" => Some(Self::Json),
            "rdf" => Some(Self::Rdf),
            _ => None,
        }
    }

    pub(crate) fn extension(self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Json => "json",
            Self::Rdf => "rdf",
        }
    }

    pub(crate) fn content_type(self) -> &'static str {
        match self {
            Self::Csv => "text/csv; charset=utf-8",
            Self::Json => "application/sparql-results+json; charset=utf-8",
            Self::Rdf => "text/turtle; charset=utf-8",
        }
    }
}
