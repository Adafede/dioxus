// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use serde::{Deserialize, Serialize};
use shared::lotus::models::{CompoundEntry, DatasetStats, SmilesSearchType};
use utoipa::ToSchema;

use shared::lotus::pubchem_tree::{DataStats, PreviewNode, PreviewTree, TreeSummary};

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
            ApiSmilesSearchType::Substructure => SmilesSearchType::Substructure,
            ApiSmilesSearchType::Similarity => SmilesSearchType::Similarity,
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
            ApiElementState::Allowed => shared::lotus::models::ElementState::Allowed,
            ApiElementState::Required => shared::lotus::models::ElementState::Required,
            ApiElementState::Excluded => shared::lotus::models::ElementState::Excluded,
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
            compound_qid: value.compound_qid.as_ref().to_string(),
            name: value.name.as_ref().to_string(),
            inchikey: value.inchikey.map(|v| v.as_ref().to_string()),
            smiles: value.smiles.map(|v| v.as_ref().to_string()),
            mass: value.mass,
            formula: value.formula.map(|v| v.as_ref().to_string()),
            taxon_qid: value.taxon_qid.as_ref().to_string(),
            taxon_name: value.taxon_name.as_ref().to_string(),
            reference_qid: value.reference_qid.as_ref().to_string(),
            ref_title: value.ref_title.map(|v| v.as_ref().to_string()),
            ref_doi: value.ref_doi.map(|v| v.as_ref().to_string()),
            pub_year: value.pub_year,
            statement: value.statement.map(|v| v.as_ref().to_string()),
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

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct DataStatsDto {
    pub(crate) n_compounds: usize,
    pub(crate) n_taxa: usize,
    pub(crate) n_compound_taxon_pairs: usize,
    pub(crate) n_taxa_with_ncbi: usize,
    pub(crate) n_taxon_parent_pairs: usize,
    pub(crate) n_taxa_with_names: usize,
    pub(crate) n_compound_parent_pairs: usize,
    pub(crate) n_compounds_with_labels: usize,
}

impl From<DataStats> for DataStatsDto {
    fn from(value: DataStats) -> Self {
        Self {
            n_compounds: value.n_compounds,
            n_taxa: value.n_taxa,
            n_compound_taxon_pairs: value.n_compound_taxon_pairs,
            n_taxa_with_ncbi: value.n_taxa_with_ncbi,
            n_taxon_parent_pairs: value.n_taxon_parent_pairs,
            n_taxa_with_names: value.n_taxa_with_names,
            n_compound_parent_pairs: value.n_compound_parent_pairs,
            n_compounds_with_labels: value.n_compounds_with_labels,
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct TreeSummaryDto {
    pub(crate) root_nodes: usize,
    pub(crate) total_nodes: usize,
}

impl From<TreeSummary> for TreeSummaryDto {
    fn from(value: TreeSummary) -> Self {
        Self {
            root_nodes: value.root_nodes,
            total_nodes: value.total_nodes,
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct PreviewNodeDto {
    pub(crate) label: String,
    #[schema(no_recursion)]
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) children: Vec<PreviewNodeDto>,
}

impl From<PreviewNode> for PreviewNodeDto {
    fn from(value: PreviewNode) -> Self {
        Self {
            label: value.label,
            children: value.children.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct PreviewTreeDto {
    pub(crate) shown_nodes: usize,
    pub(crate) total_nodes: usize,
    pub(crate) nodes: Vec<PreviewNodeDto>,
}

impl From<PreviewTree> for PreviewTreeDto {
    fn from(value: PreviewTree) -> Self {
        Self {
            shown_nodes: value.shown_nodes,
            total_nodes: value.total_nodes,
            nodes: value.nodes.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct DownloadArtifactDto {
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) url: String,
    pub(crate) filename: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct PubchemFetchResponse {
    pub(crate) session_id: String,
    pub(crate) stats: DataStatsDto,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub(crate) struct PubchemBuildRequest {
    pub(crate) session_id: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct PubchemBuildResponse {
    pub(crate) session_id: String,
    pub(crate) generated_at: String,
    pub(crate) biological_summary: TreeSummaryDto,
    pub(crate) chemical_summary: TreeSummaryDto,
    pub(crate) npclassifier_summary: TreeSummaryDto,
    pub(crate) biological_preview: PreviewTreeDto,
    pub(crate) chemical_preview: PreviewTreeDto,
    pub(crate) npclassifier_preview: PreviewTreeDto,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) npclassifier_warning: Option<String>,
    pub(crate) downloads: Vec<DownloadArtifactDto>,
}
