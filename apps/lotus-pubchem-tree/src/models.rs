// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::api::{
    DataStatsDto, DownloadArtifactDto, PreviewNodeDto, PreviewTreeDto, PubchemBuildResponse,
    TreeSummaryDto,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BusyState {
    Idle,
    Fetching,
    Building,
}

impl BusyState {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Idle => "Ready",
            Self::Fetching => "Fetching data from Wikidata…",
            Self::Building => "Building tree previews and downloads…",
        }
    }

    pub const fn is_busy(self) -> bool {
        !matches!(self, Self::Idle)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreviewTab {
    Biological,
    Chemical,
    Npclassifier,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DownloadArtifact {
    pub key: String,
    pub label: String,
    pub url: String,
    pub filename: String,
}

impl From<DownloadArtifactDto> for DownloadArtifact {
    fn from(value: DownloadArtifactDto) -> Self {
        Self {
            key: value.key,
            label: value.label,
            url: value.url,
            filename: value.filename,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreviewNode {
    pub label: String,
    pub children: Vec<PreviewNode>,
}

impl From<PreviewNodeDto> for PreviewNode {
    fn from(value: PreviewNodeDto) -> Self {
        Self {
            label: value.label,
            children: value.children.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreviewTree {
    pub shown_nodes: usize,
    pub total_nodes: usize,
    pub nodes: Vec<PreviewNode>,
}

impl From<PreviewTreeDto> for PreviewTree {
    fn from(value: PreviewTreeDto) -> Self {
        Self {
            shown_nodes: value.shown_nodes,
            total_nodes: value.total_nodes,
            nodes: value.nodes.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeSummary {
    pub root_nodes: usize,
    pub total_nodes: usize,
}

impl From<TreeSummaryDto> for TreeSummary {
    fn from(value: TreeSummaryDto) -> Self {
        Self {
            root_nodes: value.root_nodes,
            total_nodes: value.total_nodes,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataStats {
    pub n_compounds: usize,
    pub n_taxa: usize,
    pub n_compound_taxon_pairs: usize,
    pub n_taxa_with_ncbi: usize,
    pub n_taxon_parent_pairs: usize,
    pub n_taxa_with_names: usize,
    pub n_compound_parent_pairs: usize,
    pub n_compounds_with_labels: usize,
}

impl From<DataStatsDto> for DataStats {
    fn from(value: DataStatsDto) -> Self {
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildResult {
    pub generated_at: String,
    pub biological_summary: TreeSummary,
    pub chemical_summary: TreeSummary,
    pub npclassifier_summary: TreeSummary,
    pub biological_preview: PreviewTree,
    pub chemical_preview: PreviewTree,
    pub npclassifier_preview: PreviewTree,
    pub npclassifier_warning: Option<String>,
    pub downloads: Vec<DownloadArtifact>,
}

impl From<PubchemBuildResponse> for BuildResult {
    fn from(value: PubchemBuildResponse) -> Self {
        Self {
            generated_at: value.generated_at,
            biological_summary: value.biological_summary.into(),
            chemical_summary: value.chemical_summary.into(),
            npclassifier_summary: value.npclassifier_summary.into(),
            biological_preview: value.biological_preview.into(),
            chemical_preview: value.chemical_preview.into(),
            npclassifier_preview: value.npclassifier_preview.into(),
            npclassifier_warning: value.npclassifier_warning,
            downloads: value.downloads.into_iter().map(Into::into).collect(),
        }
    }
}
