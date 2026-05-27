// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! LOTUS PubChem tree generation logic shared by the API and web frontends.
//!
//! This module ports the LOTUS PubChem Tree Generator workflow from the Python
//! marimo prototype into typed Rust domain logic. It is intentionally pure at the
//! tree-building layer so the API can cache fetched sessions and the Dioxus app
//! can remain a thin UI over typed HTTP endpoints.

use crate::sparql::{
    FetchError, QLEVER_WIKIDATA, SparqlResponseFormat, extract_qid, field, non_empty,
};
use csv::{ReaderBuilder, StringRecord};
use serde::Serialize;
use serde_json::{Map, Value, json};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet};
use std::fmt;
use std::io::Cursor;

pub const APP_VERSION: &str = "0.1.0";
pub const APP_NAME: &str = "LOTUS PubChem Tree Generator";
pub const NPCLASSIFIER_CACHE_URL: &str = "https://media.githubusercontent.com/media/adafede/marimo/main/apps/public/npclassifier/npclassifier_cache.csv";
const NPCLASSIFIER_CACHE_FALLBACK_URL: &str = "https://media.githubusercontent.com/media/adafede/marimo/main/apps/public/npclassifier/npclassifier_cache.csv";
pub const PREVIEW_MAX_ROOT_NODES: usize = 10_000;
pub const PREVIEW_MAX_CHILDREN: usize = 10_000;
pub const PREVIEW_MAX_DEPTH: usize = 25;
pub const PREVIEW_LEAF_BUDGET: usize = 10_000;

const METADATA_PROJECT: &str = "LOTUS";
const METADATA_PROJECT_URL: &str = "https://lotus.nprod.net/";
const METADATA_WIKIDATA_ITEM: &str = "Q104225190";
const METADATA_LICENSE_DATA: &str = "CC0 1.0 Universal";
const METADATA_LICENSE_CODE: &str = "AGPL-3.0";

const QUERY_COMPOUND_INCHIKEY_TAXON: &str = r#"
PREFIX wd: <http://www.wikidata.org/entity/>
PREFIX wdt: <http://www.wikidata.org/prop/direct/>
SELECT DISTINCT ?compound ?compound_inchikey ?taxon WHERE {
  ?compound wdt:P235 ?compound_inchikey ;
        wdt:P703 ?taxon .
}
"#;

const QUERY_TAXON_NCBI: &str = r#"
PREFIX wd: <http://www.wikidata.org/entity/>
PREFIX wdt: <http://www.wikidata.org/prop/direct/>
SELECT DISTINCT ?taxon ?taxon_ncbi WHERE {
  ?taxon wdt:P685 ?taxon_ncbi .
}
"#;

const QUERY_TAXON_PARENT: &str = r#"
PREFIX wd: <http://www.wikidata.org/entity/>
PREFIX wdt: <http://www.wikidata.org/prop/direct/>
SELECT DISTINCT ?taxon ?taxon_parent WHERE {
  ?taxon wdt:P171 ?taxon_parent .
  ?taxon_parent wdt:P171* wd:Q2382443 .
}
"#;

const QUERY_TAXON_NAME: &str = r#"
PREFIX wd: <http://www.wikidata.org/entity/>
PREFIX wdt: <http://www.wikidata.org/prop/direct/>
SELECT DISTINCT ?taxon ?taxon_name WHERE {
  ?taxon wdt:P225 ?taxon_name .
}
"#;

const QUERY_COMPOUND_SMILES_CAN: &str = r#"
PREFIX wd: <http://www.wikidata.org/entity/>
PREFIX wdt: <http://www.wikidata.org/prop/direct/>
SELECT DISTINCT ?compound ?compound_smiles_can WHERE {
  ?compound wdt:P233 ?compound_smiles_can .
}
"#;

const QUERY_COMPOUND_SMILES_ISO: &str = r#"
PREFIX wd: <http://www.wikidata.org/entity/>
PREFIX wdt: <http://www.wikidata.org/prop/direct/>
SELECT DISTINCT ?compound ?compound_smiles_iso WHERE {
  ?compound wdt:P2017 ?compound_smiles_iso .
}
"#;

const QUERY_COMPOUND_SMARTS: &str = r#"
PREFIX wd: <http://www.wikidata.org/entity/>
PREFIX wdt: <http://www.wikidata.org/prop/direct/>
SELECT DISTINCT ?compound ?compound_smarts WHERE {
  ?compound wdt:P8533 ?compound_smarts .
}
"#;

const QUERY_COMPOUND_CXSMILES: &str = r#"
PREFIX wd: <http://www.wikidata.org/entity/>
PREFIX wdt: <http://www.wikidata.org/prop/direct/>
SELECT DISTINCT ?compound ?compound_cxsmiles WHERE {
  ?compound wdt:P10718 ?compound_cxsmiles .
}
"#;

const QUERY_COMPOUND_PARENT: &str = r#"
PREFIX wd: <http://www.wikidata.org/entity/>
PREFIX wdt: <http://www.wikidata.org/prop/direct/>
SELECT DISTINCT ?compound ?compound_parent WHERE {
  ?compound wdt:P279 ?compound_parent .
  ?compound_parent wdt:P279* wd:Q11173 .
}
"#;

const QUERY_COMPOUND_LABEL: &str = r#"
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
PREFIX wd: <http://www.wikidata.org/entity/>
PREFIX wdt: <http://www.wikidata.org/prop/direct/>
SELECT DISTINCT ?compound ?compound_label ?lang WHERE {
  ?compound wdt:P279* wd:Q11173 .
  ?compound rdfs:label ?compound_label .
  BIND(LANG(?compound_label) AS ?lang)
  FILTER (?lang IN ("en", "mul"))
}
"#;

const QUERY_REFERENCE_DOI: &str = r#"
PREFIX wikibase: <http://wikiba.se/ontology#>
PREFIX wdt: <http://www.wikidata.org/prop/direct/>
PREFIX p: <http://www.wikidata.org/prop/>
PREFIX ps: <http://www.wikidata.org/prop/statement/>
PREFIX prov: <http://www.w3.org/ns/prov#>
PREFIX pr: <http://www.wikidata.org/prop/reference/>
SELECT DISTINCT ?compound ?taxon ?reference ?doi WHERE {
  ?compound wdt:P235 ?inchikey .
  ?compound p:P703 ?statement .
  ?statement ps:P703 ?taxon .
  ?statement prov:wasDerivedFrom/pr:P248 ?reference .
  ?reference wdt:P356 ?doi .
}
"#;

const QUERY_REFERENCE_PMID: &str = r#"
PREFIX wikibase: <http://wikiba.se/ontology#>
PREFIX wdt: <http://www.wikidata.org/prop/direct/>
PREFIX p: <http://www.wikidata.org/prop/>
PREFIX ps: <http://www.wikidata.org/prop/statement/>
PREFIX prov: <http://www.w3.org/ns/prov#>
PREFIX pr: <http://www.wikidata.org/prop/reference/>
SELECT DISTINCT ?compound ?taxon ?reference ?pmid WHERE {
  ?compound wdt:P235 ?inchikey .
  ?compound p:P703 ?statement .
  ?statement ps:P703 ?taxon .
  ?statement prov:wasDerivedFrom/pr:P248 ?reference .
  ?reference wdt:P698 ?pmid .
}
"#;

#[derive(Debug, Clone)]
pub enum PubchemTreeError {
    Network(String),
    Http(u16, String),
    Parse(String),
    Invalid(String),
}

impl fmt::Display for PubchemTreeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network(msg) => write!(f, "network error: {msg}"),
            Self::Http(status, msg) => write!(f, "HTTP {status}: {msg}"),
            Self::Parse(msg) => write!(f, "parse error: {msg}"),
            Self::Invalid(msg) => write!(f, "invalid data: {msg}"),
        }
    }
}

impl std::error::Error for PubchemTreeError {}

impl From<FetchError> for PubchemTreeError {
    fn from(value: FetchError) -> Self {
        match value {
            FetchError::Network(msg) => Self::Network(msg),
            FetchError::Http(status, body) => Self::Http(status, body),
            FetchError::Parse(msg) => Self::Parse(msg),
            FetchError::Empty => Self::Parse("query returned no results".to_string()),
        }
    }
}

impl From<csv::Error> for PubchemTreeError {
    fn from(value: csv::Error) -> Self {
        Self::Parse(value.to_string())
    }
}

impl From<serde_json::Error> for PubchemTreeError {
    fn from(value: serde_json::Error) -> Self {
        Self::Parse(value.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompoundTaxonRow {
    pub compound: String,
    pub compound_inchikey: String,
    pub taxon: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaxonNcbiRow {
    pub taxon: String,
    pub taxon_ncbi: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaxonParentRow {
    pub taxon: String,
    pub taxon_parent: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaxonNameRow {
    pub taxon: String,
    pub taxon_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompoundValueRow {
    pub compound: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompoundParentRow {
    pub compound: String,
    pub compound_parent: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompoundLabelRow {
    pub compound: String,
    pub compound_label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceDoiRow {
    pub compound: String,
    pub taxon: String,
    pub reference: String,
    pub doi: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferencePmidRow {
    pub compound: String,
    pub taxon: String,
    pub reference: String,
    pub pmid: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpClassifierRow {
    pub smiles: String,
    pub pathway: String,
    pub superclass: String,
    pub class_name: String,
    pub error: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchedDataset {
    pub compound_taxon: Vec<CompoundTaxonRow>,
    pub taxon_ncbi: Vec<TaxonNcbiRow>,
    pub taxon_parent: Vec<TaxonParentRow>,
    pub taxon_name: Vec<TaxonNameRow>,
    pub compound_smiles_can: Vec<CompoundValueRow>,
    pub compound_smiles_iso: Vec<CompoundValueRow>,
    pub compound_smarts: Vec<CompoundValueRow>,
    pub compound_cxsmiles: Vec<CompoundValueRow>,
    pub compound_parent: Vec<CompoundParentRow>,
    pub compound_label: Vec<CompoundLabelRow>,
    pub reference_doi: Vec<ReferenceDoiRow>,
    pub reference_pmid: Vec<ReferencePmidRow>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum OneOrMany {
    One(String),
    Many(Vec<String>),
}

impl OneOrMany {
    fn from_vec(mut values: Vec<String>) -> Option<Self> {
        values.sort();
        values.dedup();
        match values.len() {
            0 => None,
            1 => values.pop().map(Self::One),
            _ => Some(Self::Many(values)),
        }
    }

    fn as_vec(&self) -> Vec<String> {
        match self {
            Self::One(value) => vec![value.clone()],
            Self::Many(values) => values.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct DescriptorSet {
    #[serde(rename = "InChIKey", skip_serializing_if = "Option::is_none")]
    pub inchikey: Option<OneOrMany>,
    #[serde(rename = "SMILES", skip_serializing_if = "Option::is_none")]
    pub smiles: Option<OneOrMany>,
    #[serde(rename = "SMARTS", skip_serializing_if = "Option::is_none")]
    pub smarts: Option<OneOrMany>,
    #[serde(rename = "CXSMILES", skip_serializing_if = "Option::is_none")]
    pub cxsmiles: Option<OneOrMany>,
}

impl DescriptorSet {
    fn is_empty(&self) -> bool {
        self.inchikey.is_none()
            && self.smiles.is_none()
            && self.smarts.is_none()
            && self.cxsmiles.is_none()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReferenceEntry {
    #[serde(rename = "QID")]
    pub qid: String,
    #[serde(rename = "DOI", skip_serializing_if = "Option::is_none")]
    pub doi: Option<String>,
    #[serde(rename = "PMID", skip_serializing_if = "Option::is_none")]
    pub pmid: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CompoundOccurrence {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Identifiers")]
    pub identifiers: BTreeMap<String, String>,
    #[serde(rename = "Descriptors", skip_serializing_if = "Option::is_none")]
    pub descriptors: Option<DescriptorSet>,
    #[serde(rename = "References", skip_serializing_if = "Vec::is_empty", default)]
    pub references: Vec<ReferenceEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TreeNode {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Identifiers")]
    pub identifiers: BTreeMap<String, String>,
    #[serde(rename = "Compounds", skip_serializing_if = "Vec::is_empty", default)]
    pub compounds: Vec<CompoundOccurrence>,
    #[serde(rename = "Descriptors", skip_serializing_if = "Option::is_none")]
    pub descriptors: Option<DescriptorSet>,
    #[serde(rename = "Children", skip_serializing_if = "Vec::is_empty", default)]
    pub children: Vec<TreeNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PreviewNode {
    pub label: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub children: Vec<PreviewNode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TreeSummary {
    pub root_nodes: usize,
    pub total_nodes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PreviewTree {
    pub shown_nodes: usize,
    pub total_nodes: usize,
    pub nodes: Vec<PreviewNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PubchemTreeBundle {
    pub biological_tree: Vec<TreeNode>,
    pub chemical_tree: Vec<TreeNode>,
    pub npclassifier_tree: Vec<TreeNode>,
    pub biological_summary: TreeSummary,
    pub chemical_summary: TreeSummary,
    pub npclassifier_summary: TreeSummary,
    pub biological_preview: PreviewTree,
    pub chemical_preview: PreviewTree,
    pub npclassifier_preview: PreviewTree,
    pub npclassifier_warning: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadArtifactKind {
    BiologicalPubchem,
    ChemicalWikidataPubchem,
    ChemicalNpclassifierPubchem,
    BiologicalFull,
    ChemicalWikidataFull,
    ChemicalNpclassifierFull,
}

impl DownloadArtifactKind {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "biological" => Some(Self::BiologicalPubchem),
            "chemical-wikidata" => Some(Self::ChemicalWikidataPubchem),
            "chemical-npclassifier" => Some(Self::ChemicalNpclassifierPubchem),
            "biological-full" => Some(Self::BiologicalFull),
            "chemical-wikidata-full" => Some(Self::ChemicalWikidataFull),
            "chemical-npclassifier-full" => Some(Self::ChemicalNpclassifierFull),
            _ => None,
        }
    }

    pub const fn key(self) -> &'static str {
        match self {
            Self::BiologicalPubchem => "biological",
            Self::ChemicalWikidataPubchem => "chemical-wikidata",
            Self::ChemicalNpclassifierPubchem => "chemical-npclassifier",
            Self::BiologicalFull => "biological-full",
            Self::ChemicalWikidataFull => "chemical-wikidata-full",
            Self::ChemicalNpclassifierFull => "chemical-npclassifier-full",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::BiologicalPubchem => "Biological Tree JSON",
            Self::ChemicalWikidataPubchem => "Chemical Tree (Wikidata) JSON",
            Self::ChemicalNpclassifierPubchem => "Chemical Tree JSON",
            Self::BiologicalFull => "Biological Tree (Full)",
            Self::ChemicalWikidataFull => "Chemical Tree Wikidata (Full)",
            Self::ChemicalNpclassifierFull => "Chemical Tree (Full)",
        }
    }

    pub fn filename(self, date_stamp: &str) -> String {
        match self {
            Self::BiologicalPubchem => format!("{date_stamp}_lotus_biological_tree.json"),
            Self::ChemicalWikidataPubchem => {
                format!("{date_stamp}_lotus_chemical_tree_wikidata.json")
            }
            Self::ChemicalNpclassifierPubchem => format!("{date_stamp}_lotus_chemical_tree.json"),
            Self::BiologicalFull => format!("{date_stamp}_lotus_biological_tree_full.json"),
            Self::ChemicalWikidataFull => {
                format!("{date_stamp}_lotus_chemical_tree_wikidata_full.json")
            }
            Self::ChemicalNpclassifierFull => {
                format!("{date_stamp}_lotus_chemical_tree_full.json")
            }
        }
    }

    pub fn available(self, bundle: &PubchemTreeBundle) -> bool {
        match self {
            Self::ChemicalNpclassifierPubchem | Self::ChemicalNpclassifierFull => {
                !bundle.npclassifier_tree.is_empty()
            }
            _ => true,
        }
    }
}

pub async fn fetch_dataset(endpoint: &str) -> Result<FetchedDataset, PubchemTreeError> {
    let compound_taxon = execute_csv_query_or_empty(
        "compound_taxon",
        QUERY_COMPOUND_INCHIKEY_TAXON,
        endpoint,
        &["compound", "compound_inchikey", "taxon"],
    )
    .await;
    let taxon_ncbi = execute_csv_query_or_empty(
        "taxon_ncbi",
        QUERY_TAXON_NCBI,
        endpoint,
        &["taxon", "taxon_ncbi"],
    )
    .await;
    let taxon_parent = execute_csv_query_or_empty(
        "taxon_parent",
        QUERY_TAXON_PARENT,
        endpoint,
        &["taxon", "taxon_parent"],
    )
    .await;
    let taxon_name = execute_csv_query_or_empty(
        "taxon_name",
        QUERY_TAXON_NAME,
        endpoint,
        &["taxon", "taxon_name"],
    )
    .await;
    let compound_smiles_can = execute_csv_query_or_empty(
        "compound_smiles_can",
        QUERY_COMPOUND_SMILES_CAN,
        endpoint,
        &["compound", "compound_smiles_can"],
    )
    .await;
    let compound_smiles_iso = execute_csv_query_or_empty(
        "compound_smiles_iso",
        QUERY_COMPOUND_SMILES_ISO,
        endpoint,
        &["compound", "compound_smiles_iso"],
    )
    .await;
    let compound_smarts = execute_csv_query_or_empty(
        "compound_smarts",
        QUERY_COMPOUND_SMARTS,
        endpoint,
        &["compound", "compound_smarts"],
    )
    .await;
    let compound_cxsmiles = execute_csv_query_or_empty(
        "compound_cxsmiles",
        QUERY_COMPOUND_CXSMILES,
        endpoint,
        &["compound", "compound_cxsmiles"],
    )
    .await;
    let compound_parent = execute_csv_query_or_empty(
        "compound_parent",
        QUERY_COMPOUND_PARENT,
        endpoint,
        &["compound", "compound_parent"],
    )
    .await;
    let compound_label = execute_csv_query_or_empty(
        "compound_label",
        QUERY_COMPOUND_LABEL,
        endpoint,
        &["compound", "compound_label", "lang"],
    )
    .await;
    let reference_doi = execute_csv_query_or_empty(
        "reference_doi",
        QUERY_REFERENCE_DOI,
        endpoint,
        &["compound", "taxon", "reference", "doi"],
    )
    .await;
    let reference_pmid = execute_csv_query_or_empty(
        "reference_pmid",
        QUERY_REFERENCE_PMID,
        endpoint,
        &["compound", "taxon", "reference", "pmid"],
    )
    .await;

    Ok(FetchedDataset {
        compound_taxon: parse_compound_taxon(&compound_taxon)?,
        taxon_ncbi: parse_taxon_ncbi(&taxon_ncbi)?,
        taxon_parent: parse_taxon_parent(&taxon_parent)?,
        taxon_name: parse_taxon_name(&taxon_name)?,
        compound_smiles_can: parse_compound_value(&compound_smiles_can, "compound_smiles_can")?,
        compound_smiles_iso: parse_compound_value(&compound_smiles_iso, "compound_smiles_iso")?,
        compound_smarts: parse_compound_value(&compound_smarts, "compound_smarts")?,
        compound_cxsmiles: parse_compound_value(&compound_cxsmiles, "compound_cxsmiles")?,
        compound_parent: parse_compound_parent(&compound_parent)?,
        compound_label: parse_compound_label(&compound_label)?,
        reference_doi: parse_reference_doi(&reference_doi)?,
        reference_pmid: parse_reference_pmid(&reference_pmid)?,
    })
}

pub fn compute_stats(data: &FetchedDataset) -> DataStats {
    let n_compounds = data
        .compound_taxon
        .iter()
        .map(|row| row.compound.as_str())
        .collect::<HashSet<_>>()
        .len();
    let n_taxa = data
        .compound_taxon
        .iter()
        .map(|row| row.taxon.as_str())
        .collect::<HashSet<_>>()
        .len();

    DataStats {
        n_compounds,
        n_taxa,
        n_compound_taxon_pairs: data.compound_taxon.len(),
        n_taxa_with_ncbi: data.taxon_ncbi.len(),
        n_taxon_parent_pairs: data.taxon_parent.len(),
        n_taxa_with_names: data.taxon_name.len(),
        n_compound_parent_pairs: data.compound_parent.len(),
        n_compounds_with_labels: data.compound_label.len(),
    }
}

pub async fn build_trees(
    data: &FetchedDataset,
    npclassifier_cache_url: &str,
) -> Result<PubchemTreeBundle, PubchemTreeError> {
    let label_map = data
        .compound_label
        .iter()
        .map(|row| (row.compound.clone(), row.compound_label.clone()))
        .collect::<HashMap<_, _>>();

    let (compounds_with_taxa, inchikey_map) = build_compounds_with_taxa(&data.compound_taxon);
    let smiles_map = build_smiles_map(&data.compound_smiles_iso, &data.compound_smiles_can);
    let smarts_map = build_value_map(&data.compound_smarts);
    let cxsmiles_map = build_value_map(&data.compound_cxsmiles);
    let descriptor_map =
        build_descriptor_map(&smiles_map, &smarts_map, &cxsmiles_map, &inchikey_map);
    let reference_map = build_reference_map(&data.reference_doi, &data.reference_pmid);

    let biological_tree = build_biological_tree(
        &data.compound_taxon,
        &data.taxon_ncbi,
        &data.taxon_parent,
        &data.taxon_name,
        &compounds_with_taxa,
        &descriptor_map,
        &label_map,
        &reference_map,
    );
    let chemical_tree = build_compound_tree(
        &data.compound_parent,
        &label_map,
        &compounds_with_taxa,
        &descriptor_map,
    );

    let (npclassifier_tree, npclassifier_warning) = match fetch_npclassifier_cache(
        npclassifier_cache_url,
    )
    .await
    {
        Ok(rows) if !rows.is_empty() => {
            let (smiles_to_inchikey, smiles_to_qid) =
                build_smiles_to_inchikey_map(&smiles_map, &inchikey_map);
            (
                build_npclassifier_tree(&rows, &smiles_to_inchikey, &smiles_to_qid, &label_map),
                None,
            )
        }
        Ok(_) => (
            Vec::new(),
            Some(
                "Could not fetch NPClassifier cache. NPClassifier tree will not be available."
                    .to_string(),
            ),
        ),
        Err(err) => (
            Vec::new(),
            Some(format!(
                "Could not fetch NPClassifier cache. NPClassifier tree will not be available. ({err})"
            )),
        ),
    };

    let biological_summary = TreeSummary {
        root_nodes: biological_tree.len(),
        total_nodes: count_tree_nodes(&biological_tree),
    };
    let chemical_summary = TreeSummary {
        root_nodes: chemical_tree.len(),
        total_nodes: count_tree_nodes(&chemical_tree),
    };
    let npclassifier_summary = TreeSummary {
        root_nodes: npclassifier_tree.len(),
        total_nodes: count_tree_nodes(&npclassifier_tree),
    };

    Ok(PubchemTreeBundle {
        biological_preview: build_preview_tree(&biological_tree),
        chemical_preview: build_preview_tree(&chemical_tree),
        npclassifier_preview: build_preview_tree(&npclassifier_tree),
        biological_summary,
        chemical_summary,
        npclassifier_summary,
        biological_tree,
        chemical_tree,
        npclassifier_tree,
        npclassifier_warning,
    })
}

pub fn build_download_json(
    kind: DownloadArtifactKind,
    bundle: &PubchemTreeBundle,
    generated_at: &str,
) -> Result<String, PubchemTreeError> {
    let value = match kind {
        DownloadArtifactKind::BiologicalPubchem => {
            tree_to_pubchem_format(&bundle.biological_tree, true)
        }
        DownloadArtifactKind::ChemicalWikidataPubchem => {
            tree_to_pubchem_format(&bundle.chemical_tree, false)
        }
        DownloadArtifactKind::ChemicalNpclassifierPubchem => {
            npclassifier_tree_to_pubchem(&bundle.npclassifier_tree)
        }
        DownloadArtifactKind::BiologicalFull => build_tree_output(
            "biological",
            &bundle.biological_tree,
            "wikidata",
            generated_at,
        ),
        DownloadArtifactKind::ChemicalWikidataFull => {
            build_tree_output("chemical", &bundle.chemical_tree, "wikidata", generated_at)
        }
        DownloadArtifactKind::ChemicalNpclassifierFull => build_tree_output(
            "chemical",
            &bundle.npclassifier_tree,
            "npclassifier",
            generated_at,
        ),
    };

    serde_json::to_string_pretty(&value).map_err(Into::into)
}

pub fn count_tree_nodes(tree: &[TreeNode]) -> usize {
    tree.iter()
        .map(|node| 1 + count_tree_nodes(&node.children))
        .sum()
}

async fn execute_csv_query(query: &str, endpoint: &str) -> Result<Vec<u8>, PubchemTreeError> {
    crate::sparql::execute_sparql_with_format_bytes(query, endpoint, SparqlResponseFormat::Csv)
        .await
        .map_err(Into::into)
}

async fn execute_csv_query_or_empty(
    query_name: &str,
    query: &str,
    endpoint: &str,
    header: &[&str],
) -> Vec<u8> {
    match execute_csv_query(query, endpoint).await {
        Ok(bytes) => bytes,
        Err(err) => {
            log::warn!(
                "pubchem fetch query failed; continuing with empty result: {query_name}: {err}"
            );
            empty_csv_bytes(header)
        }
    }
}

fn empty_csv_bytes(header: &[&str]) -> Vec<u8> {
    let mut out = header.join(",").into_bytes();
    out.push(b'\n');
    out
}

async fn fetch_npclassifier_cache(url: &str) -> Result<Vec<NpClassifierRow>, PubchemTreeError> {
    let bytes = crate::sparql::fetch_url_bytes(url).await?;
    if looks_like_git_lfs_pointer(&bytes) && url != NPCLASSIFIER_CACHE_FALLBACK_URL {
        let fallback_bytes =
            crate::sparql::fetch_url_bytes(NPCLASSIFIER_CACHE_FALLBACK_URL).await?;
        return parse_npclassifier_cache(&fallback_bytes);
    }
    parse_npclassifier_cache(&bytes)
}

fn looks_like_git_lfs_pointer(bytes: &[u8]) -> bool {
    let preview_len = bytes.len().min(256);
    let preview = String::from_utf8_lossy(&bytes[..preview_len]);
    preview.contains("version https://git-lfs.github.com/spec/v1")
}

fn csv_reader(bytes: &[u8]) -> csv::Reader<Cursor<&[u8]>> {
    ReaderBuilder::new()
        .has_headers(true)
        .from_reader(Cursor::new(bytes))
}

fn header_indexes<'a>(
    headers: &StringRecord,
    names: &'a [&'a str],
) -> HashMap<&'a str, Option<usize>> {
    names
        .iter()
        .map(|name| (*name, headers.iter().position(|header| header == *name)))
        .collect()
}

fn parse_compound_taxon(bytes: &[u8]) -> Result<Vec<CompoundTaxonRow>, PubchemTreeError> {
    let mut rdr = csv_reader(bytes);
    let headers = rdr.headers()?.clone();
    let idx = header_indexes(&headers, &["compound", "compound_inchikey", "taxon"]);
    let mut rows = Vec::new();
    for record in rdr.records() {
        let record = record?;
        let compound = extract_qid(field(&record, idx["compound"]));
        let taxon = extract_qid(field(&record, idx["taxon"]));
        let compound_inchikey = field(&record, idx["compound_inchikey"]).to_string();
        if compound.is_empty() || taxon.is_empty() {
            continue;
        }
        rows.push(CompoundTaxonRow {
            compound,
            compound_inchikey,
            taxon,
        });
    }
    Ok(rows)
}

fn parse_taxon_ncbi(bytes: &[u8]) -> Result<Vec<TaxonNcbiRow>, PubchemTreeError> {
    let mut rdr = csv_reader(bytes);
    let headers = rdr.headers()?.clone();
    let idx = header_indexes(&headers, &["taxon", "taxon_ncbi"]);
    let mut rows = Vec::new();
    for record in rdr.records() {
        let record = record?;
        let taxon = extract_qid(field(&record, idx["taxon"]));
        let Some(taxon_ncbi) = non_empty(field(&record, idx["taxon_ncbi"])) else {
            continue;
        };
        if taxon.is_empty() {
            continue;
        }
        rows.push(TaxonNcbiRow { taxon, taxon_ncbi });
    }
    Ok(rows)
}

fn parse_taxon_parent(bytes: &[u8]) -> Result<Vec<TaxonParentRow>, PubchemTreeError> {
    let mut rdr = csv_reader(bytes);
    let headers = rdr.headers()?.clone();
    let idx = header_indexes(&headers, &["taxon", "taxon_parent"]);
    let mut rows = Vec::new();
    for record in rdr.records() {
        let record = record?;
        let taxon = extract_qid(field(&record, idx["taxon"]));
        let taxon_parent = extract_qid(field(&record, idx["taxon_parent"]));
        if taxon.is_empty() || taxon_parent.is_empty() {
            continue;
        }
        rows.push(TaxonParentRow {
            taxon,
            taxon_parent,
        });
    }
    Ok(rows)
}

fn parse_taxon_name(bytes: &[u8]) -> Result<Vec<TaxonNameRow>, PubchemTreeError> {
    let mut rdr = csv_reader(bytes);
    let headers = rdr.headers()?.clone();
    let idx = header_indexes(&headers, &["taxon", "taxon_name"]);
    let mut rows = Vec::new();
    for record in rdr.records() {
        let record = record?;
        let taxon = extract_qid(field(&record, idx["taxon"]));
        let Some(taxon_name) = non_empty(field(&record, idx["taxon_name"])) else {
            continue;
        };
        if taxon.is_empty() {
            continue;
        }
        rows.push(TaxonNameRow { taxon, taxon_name });
    }
    Ok(rows)
}

fn parse_compound_value(
    bytes: &[u8],
    value_column: &str,
) -> Result<Vec<CompoundValueRow>, PubchemTreeError> {
    let mut rdr = csv_reader(bytes);
    let headers = rdr.headers()?.clone();
    let columns = ["compound", value_column];
    let idx = header_indexes(&headers, &columns);
    let mut rows = Vec::new();
    for record in rdr.records() {
        let record = record?;
        let compound = extract_qid(field(&record, idx["compound"]));
        let Some(value) = non_empty(field(&record, idx[value_column])) else {
            continue;
        };
        if compound.is_empty() {
            continue;
        }
        rows.push(CompoundValueRow { compound, value });
    }
    Ok(rows)
}

fn parse_compound_parent(bytes: &[u8]) -> Result<Vec<CompoundParentRow>, PubchemTreeError> {
    let mut rdr = csv_reader(bytes);
    let headers = rdr.headers()?.clone();
    let idx = header_indexes(&headers, &["compound", "compound_parent"]);
    let mut rows = Vec::new();
    for record in rdr.records() {
        let record = record?;
        let compound = extract_qid(field(&record, idx["compound"]));
        let compound_parent = extract_qid(field(&record, idx["compound_parent"]));
        if compound.is_empty() || compound_parent.is_empty() {
            continue;
        }
        rows.push(CompoundParentRow {
            compound,
            compound_parent,
        });
    }
    Ok(rows)
}

fn parse_compound_label(bytes: &[u8]) -> Result<Vec<CompoundLabelRow>, PubchemTreeError> {
    let mut rdr = csv_reader(bytes);
    let headers = rdr.headers()?.clone();
    let idx = header_indexes(&headers, &["compound", "compound_label", "lang"]);
    let mut labels = HashMap::<String, (u8, String)>::new();

    for record in rdr.records() {
        let record = record?;
        let compound = extract_qid(field(&record, idx["compound"]));
        let Some(compound_label) = non_empty(field(&record, idx["compound_label"])) else {
            continue;
        };
        let lang = field(&record, idx["lang"]);
        let priority = match lang {
            "mul" => 2,
            "en" => 1,
            _ => 0,
        };
        if compound.is_empty() || priority == 0 {
            continue;
        }
        match labels.get(&compound) {
            Some((existing_priority, _)) if *existing_priority >= priority => {}
            _ => {
                labels.insert(compound, (priority, compound_label));
            }
        }
    }

    let mut rows = labels
        .into_iter()
        .map(|(compound, (_, compound_label))| CompoundLabelRow {
            compound,
            compound_label,
        })
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| a.compound.cmp(&b.compound));
    Ok(rows)
}

fn parse_reference_doi(bytes: &[u8]) -> Result<Vec<ReferenceDoiRow>, PubchemTreeError> {
    let mut rdr = csv_reader(bytes);
    let headers = rdr.headers()?.clone();
    let idx = header_indexes(&headers, &["compound", "taxon", "reference", "doi"]);
    let mut rows = Vec::new();
    for record in rdr.records() {
        let record = record?;
        let compound = extract_qid(field(&record, idx["compound"]));
        let taxon = extract_qid(field(&record, idx["taxon"]));
        let reference = extract_qid(field(&record, idx["reference"]));
        let Some(doi) = non_empty(field(&record, idx["doi"])) else {
            continue;
        };
        if compound.is_empty() || taxon.is_empty() || reference.is_empty() {
            continue;
        }
        rows.push(ReferenceDoiRow {
            compound,
            taxon,
            reference,
            doi,
        });
    }
    Ok(rows)
}

fn parse_reference_pmid(bytes: &[u8]) -> Result<Vec<ReferencePmidRow>, PubchemTreeError> {
    let mut rdr = csv_reader(bytes);
    let headers = rdr.headers()?.clone();
    let idx = header_indexes(&headers, &["compound", "taxon", "reference", "pmid"]);
    let mut rows = Vec::new();
    for record in rdr.records() {
        let record = record?;
        let compound = extract_qid(field(&record, idx["compound"]));
        let taxon = extract_qid(field(&record, idx["taxon"]));
        let reference = extract_qid(field(&record, idx["reference"]));
        let Some(pmid) = non_empty(field(&record, idx["pmid"])) else {
            continue;
        };
        if compound.is_empty() || taxon.is_empty() || reference.is_empty() {
            continue;
        }
        rows.push(ReferencePmidRow {
            compound,
            taxon,
            reference,
            pmid,
        });
    }
    Ok(rows)
}

fn parse_npclassifier_cache(bytes: &[u8]) -> Result<Vec<NpClassifierRow>, PubchemTreeError> {
    let mut rdr = csv_reader(bytes);
    let headers = rdr.headers()?.clone();
    let idx = header_indexes(
        &headers,
        &["smiles", "pathway", "superclass", "class", "error"],
    );
    let mut rows = Vec::new();
    for record in rdr.records() {
        let record = record?;
        let Some(smiles) = non_empty(field(&record, idx["smiles"])) else {
            continue;
        };
        rows.push(NpClassifierRow {
            smiles,
            pathway: field(&record, idx["pathway"]).to_string(),
            superclass: field(&record, idx["superclass"]).to_string(),
            class_name: field(&record, idx["class"]).to_string(),
            error: field(&record, idx["error"]).to_string(),
        });
    }
    Ok(rows)
}

fn unique_sorted(values: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut set = BTreeSet::new();
    for value in values.into_iter().map(|value| value.trim().to_string()) {
        if !value.is_empty() {
            set.insert(value);
        }
    }
    set.into_iter().collect()
}

fn build_value_map(rows: &[CompoundValueRow]) -> HashMap<String, Vec<String>> {
    let mut grouped: HashMap<String, Vec<String>> = HashMap::new();
    for row in rows {
        grouped
            .entry(row.compound.clone())
            .or_default()
            .push(row.value.clone());
    }
    grouped
        .into_iter()
        .map(|(compound, values)| (compound, unique_sorted(values)))
        .filter(|(_, values)| !values.is_empty())
        .collect()
}

fn build_smiles_map(
    smiles_iso: &[CompoundValueRow],
    smiles_can: &[CompoundValueRow],
) -> HashMap<String, Vec<String>> {
    let iso_map = build_value_map(smiles_iso);
    let can_map = build_value_map(smiles_can);
    let all_compounds = iso_map
        .keys()
        .chain(can_map.keys())
        .cloned()
        .collect::<HashSet<_>>();

    all_compounds
        .into_iter()
        .filter_map(|compound| {
            let values = iso_map
                .get(&compound)
                .cloned()
                .or_else(|| can_map.get(&compound).cloned())?;
            Some((compound, values))
        })
        .collect()
}

fn build_compounds_with_taxa(
    compound_taxon: &[CompoundTaxonRow],
) -> (HashSet<String>, HashMap<String, Vec<String>>) {
    let mut grouped: HashMap<String, Vec<String>> = HashMap::new();
    for row in compound_taxon {
        grouped
            .entry(row.compound.clone())
            .or_default()
            .push(row.compound_inchikey.clone());
    }

    let mut compounds_with_taxa = HashSet::new();
    let mut inchikey_map = HashMap::new();
    for (compound, inchikeys) in grouped {
        let values = unique_sorted(inchikeys);
        if !values.is_empty() {
            compounds_with_taxa.insert(compound.clone());
            inchikey_map.insert(compound, values);
        }
    }
    (compounds_with_taxa, inchikey_map)
}

fn build_descriptor_map(
    smiles_map: &HashMap<String, Vec<String>>,
    smarts_map: &HashMap<String, Vec<String>>,
    cxsmiles_map: &HashMap<String, Vec<String>>,
    inchikey_map: &HashMap<String, Vec<String>>,
) -> HashMap<String, DescriptorSet> {
    let all_qids = smiles_map
        .keys()
        .chain(smarts_map.keys())
        .chain(cxsmiles_map.keys())
        .chain(inchikey_map.keys())
        .cloned()
        .collect::<HashSet<_>>();

    all_qids
        .into_iter()
        .filter_map(|qid| {
            let descriptors = DescriptorSet {
                inchikey: inchikey_map
                    .get(&qid)
                    .cloned()
                    .and_then(OneOrMany::from_vec),
                smiles: smiles_map.get(&qid).cloned().and_then(OneOrMany::from_vec),
                smarts: smarts_map.get(&qid).cloned().and_then(OneOrMany::from_vec),
                cxsmiles: cxsmiles_map
                    .get(&qid)
                    .cloned()
                    .and_then(OneOrMany::from_vec),
            };
            (!descriptors.is_empty()).then_some((qid, descriptors))
        })
        .collect()
}

#[derive(Default)]
struct RefAccumulator {
    doi: Option<String>,
    pmid: Option<String>,
}

type ReferenceMap = HashMap<String, HashMap<String, HashMap<String, RefAccumulator>>>;

fn build_reference_map(
    reference_doi: &[ReferenceDoiRow],
    reference_pmid: &[ReferencePmidRow],
) -> ReferenceMap {
    let mut reference_map: ReferenceMap = HashMap::new();

    for row in reference_doi {
        reference_map
            .entry(row.compound.clone())
            .or_default()
            .entry(row.taxon.clone())
            .or_default()
            .entry(row.reference.clone())
            .or_default()
            .doi = Some(row.doi.clone());
    }
    for row in reference_pmid {
        reference_map
            .entry(row.compound.clone())
            .or_default()
            .entry(row.taxon.clone())
            .or_default()
            .entry(row.reference.clone())
            .or_default()
            .pmid = Some(row.pmid.clone());
    }

    reference_map
}

#[allow(clippy::too_many_arguments)]
fn build_biological_tree(
    compound_taxon: &[CompoundTaxonRow],
    taxon_ncbi: &[TaxonNcbiRow],
    taxon_parent: &[TaxonParentRow],
    taxon_name: &[TaxonNameRow],
    compounds_with_taxa: &HashSet<String>,
    descriptor_map: &HashMap<String, DescriptorSet>,
    label_map: &HashMap<String, String>,
    reference_map: &ReferenceMap,
) -> Vec<TreeNode> {
    let taxa_with_compounds = compound_taxon
        .iter()
        .map(|row| row.taxon.clone())
        .collect::<HashSet<_>>();

    let child_to_parent = taxon_parent
        .iter()
        .map(|row| (row.taxon.clone(), row.taxon_parent.clone()))
        .collect::<HashMap<_, _>>();

    let mut relevant_taxa = taxa_with_compounds.clone();
    for taxon in &taxa_with_compounds {
        let mut current = taxon.as_str();
        while let Some(parent) = child_to_parent.get(current) {
            if !relevant_taxa.insert(parent.clone()) {
                break;
            }
            current = parent;
        }
    }

    let mut parent_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut all_children = HashSet::new();
    let mut all_parents = HashSet::new();
    for row in taxon_parent {
        if relevant_taxa.contains(&row.taxon) && relevant_taxa.contains(&row.taxon_parent) {
            all_children.insert(row.taxon.clone());
            all_parents.insert(row.taxon_parent.clone());
            parent_map
                .entry(row.taxon_parent.clone())
                .or_default()
                .push(row.taxon.clone());
        }
    }
    for children in parent_map.values_mut() {
        children.sort();
        children.dedup();
    }

    let name_map = taxon_name
        .iter()
        .map(|row| (row.taxon.clone(), row.taxon_name.clone()))
        .collect::<HashMap<_, _>>();
    let ncbi_map = taxon_ncbi
        .iter()
        .map(|row| (row.taxon.clone(), row.taxon_ncbi.clone()))
        .collect::<HashMap<_, _>>();

    let mut taxon_to_compounds: HashMap<String, Vec<String>> = HashMap::new();
    for row in compound_taxon {
        taxon_to_compounds
            .entry(row.taxon.clone())
            .or_default()
            .push(row.compound.clone());
    }
    for compounds in taxon_to_compounds.values_mut() {
        compounds.sort();
        compounds.dedup();
    }

    let mut roots = all_parents
        .difference(&all_children)
        .filter(|taxon| relevant_taxa.contains(*taxon))
        .cloned()
        .collect::<Vec<_>>();
    if roots.is_empty() {
        roots = relevant_taxa
            .iter()
            .filter(|taxon| !child_to_parent.contains_key(*taxon))
            .cloned()
            .collect();
    }
    if roots.is_empty() {
        roots = taxa_with_compounds.into_iter().collect();
    }
    roots.sort();
    roots.dedup();

    #[allow(clippy::too_many_arguments)]
    fn build_node(
        taxon_qid: &str,
        visited: &mut HashSet<String>,
        parent_map: &HashMap<String, Vec<String>>,
        name_map: &HashMap<String, String>,
        ncbi_map: &HashMap<String, String>,
        taxon_to_compounds: &HashMap<String, Vec<String>>,
        compounds_with_taxa: &HashSet<String>,
        descriptor_map: &HashMap<String, DescriptorSet>,
        label_map: &HashMap<String, String>,
        reference_map: &ReferenceMap,
    ) -> Option<TreeNode> {
        if !visited.insert(taxon_qid.to_string()) {
            return None;
        }

        let mut compounds = taxon_to_compounds
            .get(taxon_qid)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter(|qid| compounds_with_taxa.contains(qid))
            .map(|qid| {
                let mut identifiers = BTreeMap::new();
                identifiers.insert("QID".to_string(), qid.clone());
                let mut references = reference_map
                    .get(&qid)
                    .and_then(|by_taxon| by_taxon.get(taxon_qid))
                    .map(|refs| {
                        refs.iter()
                            .map(|(ref_qid, values)| ReferenceEntry {
                                qid: ref_qid.clone(),
                                doi: values.doi.clone(),
                                pmid: values.pmid.clone(),
                            })
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                references.sort_by(|a, b| {
                    a.doi
                        .cmp(&b.doi)
                        .then_with(|| a.pmid.cmp(&b.pmid))
                        .then_with(|| a.qid.cmp(&b.qid))
                });
                CompoundOccurrence {
                    name: label_map.get(&qid).cloned().unwrap_or(qid.clone()),
                    identifiers,
                    descriptors: descriptor_map.get(&qid).cloned(),
                    references,
                }
            })
            .collect::<Vec<_>>();
        compounds.sort_by(|a, b| {
            a.name
                .cmp(&b.name)
                .then_with(|| a.identifiers.cmp(&b.identifiers))
        });

        let mut children = parent_map
            .get(taxon_qid)
            .into_iter()
            .flat_map(|children| children.iter())
            .filter_map(|child| {
                build_node(
                    child,
                    visited,
                    parent_map,
                    name_map,
                    ncbi_map,
                    taxon_to_compounds,
                    compounds_with_taxa,
                    descriptor_map,
                    label_map,
                    reference_map,
                )
            })
            .collect::<Vec<_>>();
        children.sort_by(|a, b| a.name.cmp(&b.name));

        if compounds.is_empty() && children.is_empty() {
            return None;
        }

        let mut identifiers = BTreeMap::new();
        identifiers.insert("QID".to_string(), taxon_qid.to_string());
        if let Some(ncbi_taxid) = ncbi_map.get(taxon_qid) {
            identifiers.insert("NCBI_TaxID".to_string(), ncbi_taxid.clone());
        }

        Some(TreeNode {
            name: name_map
                .get(taxon_qid)
                .cloned()
                .unwrap_or_else(|| taxon_qid.to_string()),
            identifiers,
            compounds,
            descriptors: None,
            children,
        })
    }

    let mut tree = Vec::new();
    let mut visited = HashSet::new();
    for root in roots {
        if let Some(node) = build_node(
            &root,
            &mut visited,
            &parent_map,
            &name_map,
            &ncbi_map,
            &taxon_to_compounds,
            compounds_with_taxa,
            descriptor_map,
            label_map,
            reference_map,
        ) {
            tree.push(node);
        }
    }
    tree.sort_by(|a, b| a.name.cmp(&b.name));
    tree
}

fn build_compound_tree(
    compound_parent: &[CompoundParentRow],
    label_map: &HashMap<String, String>,
    compounds_with_taxa: &HashSet<String>,
    descriptor_map: &HashMap<String, DescriptorSet>,
) -> Vec<TreeNode> {
    let mut parent_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut all_compounds = HashSet::new();
    let mut all_parents = HashSet::new();
    for row in compound_parent {
        all_compounds.insert(row.compound.clone());
        all_parents.insert(row.compound_parent.clone());
        parent_map
            .entry(row.compound_parent.clone())
            .or_default()
            .push(row.compound.clone());
    }
    for children in parent_map.values_mut() {
        children.sort();
        children.dedup();
    }

    let mut roots = all_parents
        .difference(&all_compounds)
        .cloned()
        .collect::<Vec<_>>();
    if roots.is_empty() {
        roots = compounds_with_taxa.iter().cloned().collect();
    }
    roots.sort();
    roots.dedup();

    fn build_node(
        compound_qid: &str,
        visited: &mut HashSet<String>,
        parent_map: &HashMap<String, Vec<String>>,
        label_map: &HashMap<String, String>,
        compounds_with_taxa: &HashSet<String>,
        descriptor_map: &HashMap<String, DescriptorSet>,
    ) -> Option<TreeNode> {
        if !visited.insert(compound_qid.to_string()) {
            return None;
        }

        let mut children = parent_map
            .get(compound_qid)
            .into_iter()
            .flat_map(|children| children.iter())
            .filter_map(|child| {
                build_node(
                    child,
                    visited,
                    parent_map,
                    label_map,
                    compounds_with_taxa,
                    descriptor_map,
                )
            })
            .collect::<Vec<_>>();
        children.sort_by(|a, b| a.name.cmp(&b.name));

        let has_taxa = compounds_with_taxa.contains(compound_qid);
        if !has_taxa && children.is_empty() {
            return None;
        }

        let mut identifiers = BTreeMap::new();
        identifiers.insert("QID".to_string(), compound_qid.to_string());
        Some(TreeNode {
            name: label_map
                .get(compound_qid)
                .cloned()
                .unwrap_or_else(|| compound_qid.to_string()),
            identifiers,
            compounds: Vec::new(),
            descriptors: descriptor_map.get(compound_qid).cloned(),
            children,
        })
    }

    let mut tree = Vec::new();
    let mut visited = HashSet::new();
    for root in roots {
        if let Some(node) = build_node(
            &root,
            &mut visited,
            &parent_map,
            label_map,
            compounds_with_taxa,
            descriptor_map,
        ) {
            tree.push(node);
        }
    }
    tree.sort_by(|a, b| a.name.cmp(&b.name));
    tree
}

fn build_smiles_to_inchikey_map(
    smiles_map: &HashMap<String, Vec<String>>,
    inchikey_map: &HashMap<String, Vec<String>>,
) -> (HashMap<String, Vec<String>>, HashMap<String, String>) {
    let mut smiles_to_inchikey = HashMap::<String, Vec<String>>::new();
    let mut smiles_to_qid = HashMap::<String, String>::new();
    for (qid, smiles_list) in smiles_map {
        let Some(inchikeys) = inchikey_map.get(qid) else {
            continue;
        };
        for smiles in smiles_list {
            let entry = smiles_to_inchikey.entry(smiles.clone()).or_default();
            for inchikey in inchikeys {
                if !entry.contains(inchikey) {
                    entry.push(inchikey.clone());
                }
            }
            smiles_to_qid
                .entry(smiles.clone())
                .or_insert_with(|| qid.clone());
        }
    }
    for values in smiles_to_inchikey.values_mut() {
        values.sort();
        values.dedup();
    }
    (smiles_to_inchikey, smiles_to_qid)
}

fn build_npclassifier_tree(
    rows: &[NpClassifierRow],
    smiles_to_inchikey: &HashMap<String, Vec<String>>,
    smiles_to_qid: &HashMap<String, String>,
    label_map: &HashMap<String, String>,
) -> Vec<TreeNode> {
    let mut tree_data: BTreeMap<String, BTreeMap<String, BTreeMap<String, BTreeSet<String>>>> =
        BTreeMap::new();

    for row in rows {
        if !row.error.trim().is_empty() || row.pathway.trim().is_empty() {
            continue;
        }

        let pathways = split_multi_value(&row.pathway);
        let superclasses = split_multi_value(&row.superclass);
        let classes = split_multi_value(&row.class_name);

        for pathway in pathways {
            let pathway_entry = tree_data.entry(pathway).or_default();
            for (index, superclass) in superclasses.iter().enumerate() {
                let superclass_entry = pathway_entry.entry(superclass.clone()).or_default();
                let class_key = classes
                    .get(index)
                    .cloned()
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| "_unclassified".to_string());
                superclass_entry
                    .entry(class_key)
                    .or_default()
                    .insert(row.smiles.clone());
            }
        }
    }

    let mut tree = Vec::new();
    for (pathway, superclasses) in tree_data {
        let mut pathway_node = TreeNode {
            name: pathway.clone(),
            identifiers: btreemap_single("NPClassifier_Pathway".to_string(), pathway),
            compounds: Vec::new(),
            descriptors: None,
            children: Vec::new(),
        };

        for (superclass, classes) in superclasses {
            let mut superclass_node = TreeNode {
                name: superclass.clone(),
                identifiers: btreemap_single("NPClassifier_Superclass".to_string(), superclass),
                compounds: Vec::new(),
                descriptors: None,
                children: Vec::new(),
            };

            for (class_name, smiles_set) in classes {
                if class_name == "_unclassified" {
                    superclass_node
                        .children
                        .extend(smiles_set.into_iter().flat_map(|smiles| {
                            build_npclassifier_leaves(
                                &smiles,
                                smiles_to_inchikey,
                                smiles_to_qid,
                                label_map,
                            )
                        }));
                    continue;
                }

                let mut class_node = TreeNode {
                    name: class_name.clone(),
                    identifiers: btreemap_single("NPClassifier_Class".to_string(), class_name),
                    compounds: Vec::new(),
                    descriptors: None,
                    children: smiles_set
                        .into_iter()
                        .flat_map(|smiles| {
                            build_npclassifier_leaves(
                                &smiles,
                                smiles_to_inchikey,
                                smiles_to_qid,
                                label_map,
                            )
                        })
                        .collect(),
                };
                class_node.children.sort_by(|a, b| a.name.cmp(&b.name));
                if !class_node.children.is_empty() {
                    superclass_node.children.push(class_node);
                }
            }

            superclass_node.children.sort_by(|a, b| a.name.cmp(&b.name));
            if !superclass_node.children.is_empty() {
                pathway_node.children.push(superclass_node);
            }
        }

        pathway_node.children.sort_by(|a, b| a.name.cmp(&b.name));
        if !pathway_node.children.is_empty() {
            tree.push(pathway_node);
        }
    }

    tree.sort_by(|a, b| a.name.cmp(&b.name));
    tree
}

fn build_npclassifier_leaves(
    smiles: &str,
    smiles_to_inchikey: &HashMap<String, Vec<String>>,
    smiles_to_qid: &HashMap<String, String>,
    label_map: &HashMap<String, String>,
) -> Vec<TreeNode> {
    let Some(inchikeys) = smiles_to_inchikey.get(smiles) else {
        return Vec::new();
    };
    let qid = smiles_to_qid.get(smiles).cloned();
    inchikeys
        .iter()
        .map(|inchikey| {
            let mut identifiers = BTreeMap::new();
            if let Some(qid) = qid.as_ref() {
                identifiers.insert("QID".to_string(), qid.clone());
            }
            TreeNode {
                name: qid
                    .as_ref()
                    .and_then(|qid| label_map.get(qid))
                    .cloned()
                    .unwrap_or_else(|| inchikey.clone()),
                identifiers,
                compounds: Vec::new(),
                descriptors: Some(DescriptorSet {
                    inchikey: Some(OneOrMany::One(inchikey.clone())),
                    smiles: Some(OneOrMany::One(smiles.to_string())),
                    smarts: None,
                    cxsmiles: None,
                }),
                children: Vec::new(),
            }
        })
        .collect()
}

fn split_multi_value(value: &str) -> Vec<String> {
    value
        .split(" $")
        .flat_map(|part| part.split("$"))
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(ToString::to_string)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct ExpandCandidate {
    priority: usize,
    depth: usize,
    path: Vec<usize>,
}

impl Ord for ExpandCandidate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority
            .cmp(&other.priority)
            .then_with(|| other.depth.cmp(&self.depth))
            .then_with(|| self.path.cmp(&other.path))
    }
}

impl PartialOrd for ExpandCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn build_preview_tree(tree: &[TreeNode]) -> PreviewTree {
    let total_nodes = count_tree_nodes(tree);
    if tree.is_empty() {
        return PreviewTree {
            shown_nodes: 0,
            total_nodes,
            nodes: Vec::new(),
        };
    }

    let leaf_budget = PREVIEW_LEAF_BUDGET.max(1);
    let tip_counts = compute_tip_counts(tree);

    let mut root_indexes = (0..tree.len()).collect::<Vec<_>>();
    root_indexes.sort_by(|a, b| {
        tree[*b]
            .children
            .len()
            .cmp(&tree[*a].children.len())
            .then_with(|| tree[*a].name.cmp(&tree[*b].name))
    });

    let mut root_overflow = 0usize;
    if root_indexes.len() > PREVIEW_MAX_ROOT_NODES {
        root_overflow += root_indexes.len() - PREVIEW_MAX_ROOT_NODES;
        root_indexes.truncate(PREVIEW_MAX_ROOT_NODES);
    }

    let mut visible_leaves = root_indexes.len();
    if visible_leaves > leaf_budget {
        if leaf_budget == 1 {
            root_overflow += root_indexes.len().saturating_sub(1);
            root_indexes.truncate(1);
            visible_leaves = 1;
        } else {
            let keep = leaf_budget - 1;
            root_overflow += root_indexes.len().saturating_sub(keep);
            root_indexes.truncate(keep);
            visible_leaves = keep + 1;
        }
    }

    let mut expansion_plan: HashMap<Vec<usize>, usize> = HashMap::new();
    let mut frontier = BinaryHeap::<ExpandCandidate>::new();
    for root_index in &root_indexes {
        let path = vec![*root_index];
        if !tree[*root_index].children.is_empty() {
            frontier.push(ExpandCandidate {
                priority: *tip_counts.get(&path).unwrap_or(&1),
                depth: 0,
                path,
            });
        }
    }

    while visible_leaves < leaf_budget {
        let Some(candidate) = frontier.pop() else {
            break;
        };
        if expansion_plan.contains_key(&candidate.path) || candidate.depth >= PREVIEW_MAX_DEPTH {
            continue;
        }

        let Some(node) = node_at_path(tree, &candidate.path) else {
            continue;
        };
        if node.children.is_empty() {
            continue;
        }

        let sorted_children = sorted_child_indexes(node);
        let total_children = sorted_children.len().min(PREVIEW_MAX_CHILDREN);
        if total_children == 0 {
            continue;
        }

        let remaining_budget = leaf_budget - visible_leaves;
        let max_new_leaf_count = remaining_budget + 1;

        let (shown_real_children, includes_other_bucket) = if total_children <= max_new_leaf_count {
            (total_children, false)
        } else if max_new_leaf_count > 1 {
            // Reserve one slot for an aggregate "more children" bucket.
            (max_new_leaf_count - 1, true)
        } else {
            (0, false)
        };

        if shown_real_children == 0 {
            continue;
        }

        let replacement_leaf_count = shown_real_children + usize::from(includes_other_bucket);
        let delta = replacement_leaf_count.saturating_sub(1);

        expansion_plan.insert(candidate.path.clone(), shown_real_children);
        visible_leaves += delta;

        for child_index in sorted_children.into_iter().take(shown_real_children) {
            let mut child_path = candidate.path.clone();
            child_path.push(child_index);
            if let Some(child_node) = node_at_path(tree, &child_path)
                && !child_node.children.is_empty()
            {
                frontier.push(ExpandCandidate {
                    priority: *tip_counts.get(&child_path).unwrap_or(&1),
                    depth: candidate.depth + 1,
                    path: child_path,
                });
            }
        }
    }

    let mut shown_nodes = 0usize;
    let mut nodes = Vec::new();
    for root_index in root_indexes {
        let path = vec![root_index];
        nodes.push(render_preview_node(
            &tree[root_index],
            &path,
            0,
            &expansion_plan,
            &mut shown_nodes,
        ));
    }

    if root_overflow > 0 {
        shown_nodes += 1;
        nodes.push(PreviewNode {
            label: format!("⋯ {root_overflow} more root nodes"),
            children: Vec::new(),
        });
    }

    PreviewTree {
        shown_nodes,
        total_nodes,
        nodes,
    }
}

fn render_preview_node(
    node: &TreeNode,
    path: &[usize],
    depth: usize,
    expansion_plan: &HashMap<Vec<usize>, usize>,
    shown_nodes: &mut usize,
) -> PreviewNode {
    *shown_nodes += 1;
    let label = node.name.clone();

    let mut children = Vec::new();
    if depth < PREVIEW_MAX_DEPTH
        && let Some(shown_real_children) = expansion_plan.get(path)
    {
        let sorted_children = sorted_child_indexes(node);
        let shown_real_children = (*shown_real_children).min(sorted_children.len());

        for child_index in sorted_children.iter().take(shown_real_children).copied() {
            let child = &node.children[child_index];
            let mut child_path = path.to_vec();
            child_path.push(child_index);
            children.push(render_preview_node(
                child,
                &child_path,
                depth + 1,
                expansion_plan,
                shown_nodes,
            ));
        }

        let hidden_children = sorted_children.len().saturating_sub(shown_real_children);
        if hidden_children > 0 {
            *shown_nodes += 1;
            children.push(PreviewNode {
                label: format!("⋯ {hidden_children} more children"),
                children: Vec::new(),
            });
        }
    }

    PreviewNode { label, children }
}

fn sorted_child_indexes(node: &TreeNode) -> Vec<usize> {
    let mut indexes = (0..node.children.len()).collect::<Vec<_>>();
    indexes.sort_by(|a, b| {
        node.children[*b]
            .children
            .len()
            .cmp(&node.children[*a].children.len())
            .then_with(|| node.children[*a].name.cmp(&node.children[*b].name))
    });
    indexes
}

fn node_at_path<'a>(tree: &'a [TreeNode], path: &[usize]) -> Option<&'a TreeNode> {
    let (first, rest) = path.split_first()?;
    let mut node = tree.get(*first)?;
    for index in rest {
        node = node.children.get(*index)?;
    }
    Some(node)
}

fn compute_tip_counts(tree: &[TreeNode]) -> HashMap<Vec<usize>, usize> {
    let mut tip_counts = HashMap::<Vec<usize>, usize>::new();
    for (root_index, root) in tree.iter().enumerate() {
        let mut path = vec![root_index];
        compute_tip_counts_node(root, &mut path, &mut tip_counts);
    }
    tip_counts
}

fn compute_tip_counts_node(
    node: &TreeNode,
    path: &mut Vec<usize>,
    tip_counts: &mut HashMap<Vec<usize>, usize>,
) -> usize {
    let count = if node.children.is_empty() {
        1
    } else {
        let mut sum = 0usize;
        for (child_index, child) in node.children.iter().enumerate() {
            path.push(child_index);
            sum += compute_tip_counts_node(child, path, tip_counts);
            path.pop();
        }
        sum.max(1)
    };
    tip_counts.insert(path.clone(), count);
    count
}

fn tree_to_pubchem_format(tree: &[TreeNode], biological: bool) -> Value {
    if biological {
        convert_bio_node_to_pubchem(tree)
    } else {
        convert_chem_node_to_pubchem(tree)
    }
}

fn convert_bio_node_to_pubchem(nodes: &[TreeNode]) -> Value {
    if nodes.is_empty() {
        return Value::Object(Map::new());
    }

    let mut children_map = Map::new();
    for node in nodes {
        let mut node_content = Map::new();
        if let Some(qid) = node.identifiers.get("QID") {
            node_content.insert("QID".to_string(), Value::String(qid.clone()));
        }

        if !node.children.is_empty()
            && let Value::Object(mut child_result) = convert_bio_node_to_pubchem(&node.children)
            && let Some(children) = child_result.remove("children")
        {
            node_content.insert("children".to_string(), children);
        }

        if let Some(ncbi_id) = node.identifiers.get("NCBI_TaxID") {
            let mut ncbi_entry = Map::new();
            ncbi_entry.insert(
                "organism_name".to_string(),
                Value::Array(vec![Value::String(node.name.clone())]),
            );

            let mut compounds_map = Map::new();
            for compound in &node.compounds {
                if let Some(descriptors) = compound.descriptors.as_ref() {
                    for inchikey in descriptor_inchikeys(descriptors) {
                        let entry = compounds_map
                            .entry(inchikey)
                            .or_insert_with(|| Value::Object(Map::new()));
                        let Value::Object(entry_map) = entry else {
                            continue;
                        };
                        entry_map
                            .entry("Name".to_string())
                            .or_insert_with(|| Value::String(compound.name.clone()));
                        if let Some(qid) = compound.identifiers.get("QID") {
                            entry_map
                                .entry("QID".to_string())
                                .or_insert_with(|| Value::String(qid.clone()));
                        }
                        if !compound.references.is_empty() {
                            let refs = entry_map
                                .entry("references".to_string())
                                .or_insert_with(|| Value::Array(Vec::new()));
                            let Value::Array(refs_array) = refs else {
                                continue;
                            };
                            for reference in &compound.references {
                                let mut ref_obj = Map::new();
                                if let Some(doi) = reference.doi.as_ref() {
                                    ref_obj.insert("DOI".to_string(), Value::String(doi.clone()));
                                }
                                if let Some(pmid) = reference.pmid.as_ref() {
                                    ref_obj.insert("PMID".to_string(), Value::String(pmid.clone()));
                                }
                                if !ref_obj.is_empty() {
                                    let value = Value::Object(ref_obj);
                                    if !refs_array.contains(&value) {
                                        refs_array.push(value);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if !compounds_map.is_empty() {
                ncbi_entry.insert("compounds".to_string(), Value::Object(compounds_map));
            }

            let children_entry = node_content
                .entry("children".to_string())
                .or_insert_with(|| Value::Object(Map::new()));
            if let Value::Object(children_obj) = children_entry {
                children_obj.insert(ncbi_id.clone(), Value::Object(ncbi_entry));
            }
        }

        children_map.insert(
            node.name.clone(),
            if node_content.is_empty() {
                Value::Object(Map::new())
            } else {
                Value::Object(node_content)
            },
        );
    }

    json!({ "children": children_map })
}

fn convert_chem_node_to_pubchem(nodes: &[TreeNode]) -> Value {
    if nodes.is_empty() {
        return Value::Object(Map::new());
    }

    let mut children_map = Map::new();
    for node in nodes {
        let mut node_content = Map::new();
        if let Some(qid) = node.identifiers.get("QID") {
            node_content.insert("QID".to_string(), Value::String(qid.clone()));
        }

        if !node.children.is_empty()
            && let Value::Object(mut child_result) = convert_chem_node_to_pubchem(&node.children)
            && let Some(children) = child_result.remove("children")
        {
            node_content.insert("children".to_string(), children);
        }

        if let Some(descriptors) = node.descriptors.as_ref() {
            for inchikey in descriptor_inchikeys(descriptors) {
                let leaf = json!({
                    "Name": node.name,
                    "QID": node.identifiers.get("QID"),
                });
                let children_entry = node_content
                    .entry("children".to_string())
                    .or_insert_with(|| Value::Object(Map::new()));
                if let Value::Object(children_obj) = children_entry {
                    children_obj.insert(inchikey, leaf);
                }
            }
        }

        children_map.insert(
            node.name.clone(),
            if node_content.is_empty() {
                Value::Object(Map::new())
            } else {
                Value::Object(node_content)
            },
        );
    }

    json!({ "children": children_map })
}

fn npclassifier_tree_to_pubchem(tree: &[TreeNode]) -> Value {
    if tree.is_empty() {
        return Value::Object(Map::new());
    }

    fn convert_node(node: &TreeNode) -> Value {
        if node.children.is_empty() {
            let mut leaf = Map::new();
            leaf.insert("Name".to_string(), Value::String(node.name.clone()));
            if let Some(qid) = node.identifiers.get("QID") {
                leaf.insert("QID".to_string(), Value::String(qid.clone()));
            }
            return Value::Object(leaf);
        }

        let mut children = Map::new();
        for child in &node.children {
            let key = child
                .descriptors
                .as_ref()
                .and_then(|descriptors| descriptor_inchikeys(descriptors).into_iter().next())
                .unwrap_or_else(|| child.name.clone());
            children.insert(key, convert_node(child));
        }
        json!({ "children": children })
    }

    let mut roots = Map::new();
    for node in tree {
        roots.insert(node.name.clone(), convert_node(node));
    }
    json!({ "children": roots })
}

fn descriptor_inchikeys(descriptors: &DescriptorSet) -> Vec<String> {
    descriptors
        .inchikey
        .as_ref()
        .map(OneOrMany::as_vec)
        .unwrap_or_default()
}

fn build_tree_output(
    tree_type: &str,
    tree: &[TreeNode],
    source: &str,
    generated_at: &str,
) -> Value {
    let is_biological = tree_type == "biological";
    let is_npclassifier = source == "npclassifier";

    let (overview, description, root_info, notes) = if is_npclassifier {
        (
            "This JSON contains a hierarchical tree of chemical compounds classified using NPClassifier (pathway → superclass → class).",
            "NPClassifier-based hierarchical classification of natural products",
            "NPClassifier pathways",
            vec![
                "NPClassifier provides a comprehensive classification for natural products.",
                "Hierarchy: pathway → superclass → class → InChIKey",
                "Multiple classifications are possible for a single compound.",
                "See https://npclassifier.gnps2.org/ for more information.",
            ],
        )
    } else if is_biological {
        (
            "This JSON contains a hierarchical tree of biological taxa with their associated natural product compounds.",
            "Hierarchical taxonomy of biological organisms with associated natural product compounds",
            "Biota (Q2382443)",
            vec![
                "Descriptor values are strings when single, arrays when multiple values exist (very rare).",
                "All nodes are sorted alphabetically by Name.",
                "Only nodes with InChIKey associations (directly or via descendants) are included.",
                "Data queried from Wikidata via QLever SPARQL endpoint.",
            ],
        )
    } else {
        (
            "This JSON contains a hierarchical tree of chemical compound classes with structural descriptors.",
            "Hierarchical classification of chemical compounds with structural descriptors",
            "chemical compound (Q11173)",
            vec![
                "Note: Wikidata P279 (subclass of) relationships are sparse for natural products.",
                "Consider using the NPClassifier-based tree for better coverage.",
                "Descriptor values are strings when single, arrays when multiple values exist (very rare).",
                "All nodes are sorted alphabetically by Name.",
            ],
        )
    };

    json!({
        "_documentation": {
            "overview": overview,
            "structure": {
                "tree": "Array of root nodes. Each node is an object with Name, Identifiers, and optional Children.",
                "node_fields": {
                    "Name": "Human-readable name (taxon name or compound label)",
                    "Identifiers": "External database identifiers (Wikidata QID, NCBI TaxID for taxa, NPClassifier levels)",
                    "Compounds": "(Biological tree only) Array of compounds found in this taxon",
                    "Descriptors": "Chemical structure representations (InChIKey, SMILES, etc.)",
                    "References": "(Biological tree compounds only) Literature references for compound-taxon association",
                    "Children": "Array of child nodes (same structure, recursive)"
                },
                "descriptors_fields": {
                    "InChIKey": "IUPAC International Chemical Identifier Key",
                    "SMILES": "Simplified Molecular Input Line Entry System (isomeric preferred over canonical)",
                    "SMARTS": "SMILES Arbitrary Target Specification (substructure patterns)",
                    "CXSMILES": "ChemAxon Extended SMILES"
                },
                "reference_fields": {
                    "QID": "QID of the reference article in Wikidata",
                    "DOI": "Digital Object Identifier of the reference",
                    "PMID": "PubMed ID of the reference"
                }
            },
            "notes": notes,
        },
        "metadata": {
            "name": format!(
                "LOTUS {} Tree{}",
                capitalize(tree_type),
                if is_npclassifier { " (NPClassifier)" } else { "" }
            ),
            "description": description,
            "version": APP_VERSION,
            "generated": generated_at,
            "generator": APP_NAME,
            "source": {
                "name": METADATA_PROJECT,
                "url": METADATA_PROJECT_URL,
                "wikidata_item": METADATA_WIKIDATA_ITEM,
                "endpoint": if is_npclassifier { NPCLASSIFIER_CACHE_URL } else { QLEVER_WIKIDATA },
                "classification": if is_npclassifier { "NPClassifier" } else { "Wikidata P279" },
            },
            "license": {
                "data": METADATA_LICENSE_DATA,
                "code": METADATA_LICENSE_CODE,
            },
            "constraints": {
                "root": root_info,
            },
            "statistics": {
                "root_nodes": tree.len(),
                "total_nodes": count_tree_nodes(tree),
            },
        },
        "tree": tree,
    })
}

fn capitalize(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

fn btreemap_single(key: String, value: String) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    map.insert(key, value);
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tree() -> Vec<TreeNode> {
        vec![TreeNode {
            name: "Plants".to_string(),
            identifiers: btreemap_single("QID".to_string(), "Q756".to_string()),
            compounds: vec![CompoundOccurrence {
                name: "Limonene".to_string(),
                identifiers: btreemap_single("QID".to_string(), "Q111".to_string()),
                descriptors: Some(DescriptorSet {
                    inchikey: Some(OneOrMany::One("ABCDEFGHIJKL".to_string())),
                    smiles: Some(OneOrMany::One("CCCCC".to_string())),
                    smarts: None,
                    cxsmiles: None,
                }),
                references: vec![ReferenceEntry {
                    qid: "Q999".to_string(),
                    doi: Some("10.1000/test".to_string()),
                    pmid: None,
                }],
            }],
            descriptors: None,
            children: vec![TreeNode {
                name: "Rosa".to_string(),
                identifiers: {
                    let mut ids = BTreeMap::new();
                    ids.insert("QID".to_string(), "Q123".to_string());
                    ids.insert("NCBI_TaxID".to_string(), "74636".to_string());
                    ids
                },
                compounds: Vec::new(),
                descriptors: None,
                children: Vec::new(),
            }],
        }]
    }

    #[test]
    fn one_or_many_collapses_singletons() {
        assert_eq!(
            OneOrMany::from_vec(vec!["a".to_string()]),
            Some(OneOrMany::One("a".to_string()))
        );
        assert_eq!(
            OneOrMany::from_vec(vec!["b".to_string(), "a".to_string()]),
            Some(OneOrMany::Many(vec!["a".to_string(), "b".to_string()]))
        );
    }

    #[test]
    fn preview_tree_counts_and_truncation_are_stable() {
        let preview = build_preview_tree(&sample_tree());
        assert_eq!(preview.total_nodes, 2);
        assert_eq!(preview.shown_nodes, 2);
        assert_eq!(preview.nodes.len(), 1);
        assert_eq!(preview.nodes[0].label, "Plants");
        assert_eq!(preview.nodes[0].children[0].label, "Rosa");
    }

    #[test]
    fn pubchem_biological_format_contains_children_root() {
        let value = tree_to_pubchem_format(&sample_tree(), true);
        assert!(value.get("children").is_some());
    }

    #[test]
    fn full_output_contains_metadata_and_tree() {
        let value = build_tree_output(
            "biological",
            &sample_tree(),
            "wikidata",
            "2026-01-01T00:00:00Z",
        );
        assert_eq!(value["metadata"]["generated"], "2026-01-01T00:00:00Z");
        assert!(value["tree"].is_array());
    }

    #[test]
    fn download_artifact_keys_parse_round_trip() {
        for key in [
            "biological",
            "chemical-wikidata",
            "chemical-npclassifier",
            "biological-full",
            "chemical-wikidata-full",
            "chemical-npclassifier-full",
        ] {
            let kind = DownloadArtifactKind::parse(key).expect("artifact kind");
            assert_eq!(kind.key(), key);
        }
    }

    #[test]
    fn detects_git_lfs_pointer_payload() {
        let pointer = b"version https://git-lfs.github.com/spec/v1\noid sha256:abc\nsize 42\n";
        assert!(looks_like_git_lfs_pointer(pointer));
    }

    #[test]
    fn ignores_non_pointer_csv_payload() {
        let csv = b"smiles,pathway,superclass,class,isglycoside,error\nCCO,,,,False,\n";
        assert!(!looks_like_git_lfs_pointer(csv));
    }
}
