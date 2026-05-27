// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct DataStatsDto {
    pub n_compounds: usize,
    pub n_taxa: usize,
    pub n_compound_taxon_pairs: usize,
    pub n_taxa_with_ncbi: usize,
    pub n_taxon_parent_pairs: usize,
    pub n_taxa_with_names: usize,
    pub n_compound_parent_pairs: usize,
    pub n_compounds_with_labels: usize,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct TreeSummaryDto {
    pub root_nodes: usize,
    pub total_nodes: usize,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct PreviewNodeDto {
    pub label: String,
    #[serde(default)]
    pub children: Vec<PreviewNodeDto>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct PreviewTreeDto {
    pub shown_nodes: usize,
    pub total_nodes: usize,
    pub nodes: Vec<PreviewNodeDto>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct DownloadArtifactDto {
    pub key: String,
    pub label: String,
    pub url: String,
    pub filename: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct PubchemFetchResponse {
    pub session_id: String,
    pub stats: DataStatsDto,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PubchemBuildRequest {
    pub session_id: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct PubchemBuildResponse {
    pub session_id: String,
    pub generated_at: String,
    pub biological_summary: TreeSummaryDto,
    pub chemical_summary: TreeSummaryDto,
    pub npclassifier_summary: TreeSummaryDto,
    pub biological_preview: PreviewTreeDto,
    pub chemical_preview: PreviewTreeDto,
    pub npclassifier_preview: PreviewTreeDto,
    pub npclassifier_warning: Option<String>,
    pub downloads: Vec<DownloadArtifactDto>,
}
