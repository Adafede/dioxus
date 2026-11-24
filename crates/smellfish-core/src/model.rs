// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! Core data models and string-matching helpers for natural-product screening.

use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet, HashMap};

mod classification;
mod types;

pub use classification::{MotifContext, RingFamily, Verdict, VerdictCategory};
pub(crate) use types::InChIKey14;
pub use types::{CheckStatus, LotusCompound, NpScore, SourceClass};

use crate::error::SmellfishError;

/// One parsed input row prior to structure inspection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawRow {
    /// 1-based row index from the input file.
    pub index: usize,
    /// Compound identifier (from name/label column or auto-generated).
    pub label: String,
    /// SMILES structure string.
    pub smiles: String,
}

/// A single chemist's visual audit check on a molecule.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChemistCheck {
    /// Short label, e.g. `"NP-likeness"`, `"Skeleton"`, `"Database"`.
    pub name: &'static str,
    /// Check outcome (`Pass`, `Warn`, or `Fail`).
    pub status: CheckStatus,
    /// Human-readable rationale.
    pub detail: String,
}

/// Assessed molecule row displayed in the UI and exported to CSV.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MoleculeRow {
    /// 1-based input index.
    pub index: usize,
    /// Compound display label.
    pub label: String,
    /// Input SMILES string.
    pub smiles: String,
    /// Full `InChIKey` returned by `RDKit`, if inspection succeeded.
    pub inchikey: String,
    /// Validated 2D depiction `<svg>` markup.
    pub svg: Option<String>,
    /// Matched motif labels.
    pub motifs: Vec<String>,
    /// Ertl substituent occurrence counts (`label -> count`).
    pub substituents_counts: BTreeMap<String, usize>,
    /// LOTUS 1%-prevalence scaffold SMILES matches.
    pub lotus_scaffolds: Vec<String>,
    /// Taxon names matched via LOTUS/Wikidata.
    pub lotus_taxa: Vec<String>,
    /// Related Wikidata compound items and taxon coverage flags.
    pub lotus_compounds: Vec<LotusCompound>,
    /// Matched `PubChem` CIDs.
    pub pubchem_cids: Vec<String>,
    /// Ertl NP-likeness score and coverage (`None` if model unavailable or invalid SMILES).
    pub np_score: Option<NpScore>,
    /// Human-readable NP-likeness bucket label.
    pub np_label: String,
    /// Classified scaffold ring family.
    pub ring_family: RingFamily,
    /// Detailed bullet list supporting the assessment.
    pub evidence_notes: Vec<String>,
    /// One-line structural motif summary.
    pub motif_context: MotifContext,
    /// Three quick-check audit rows.
    pub chemist_checks: Vec<ChemistCheck>,
    /// Overall verdict badge and category.
    pub verdict: Verdict,
    /// Inspection error, if SMILES parsing failed.
    pub error: Option<String>,
    /// Whole-molecule `RDKit` descriptors retained for Pass-2 evaluation.
    pub descriptors: RdkitDescriptors,
    /// Stereochemical centre tags detected in the 2D depiction.
    pub stereo_tags: Vec<String>,
    /// Heavy-atom count used for Ertl score normalization.
    pub num_atoms: usize,
    /// Detailed motif hits with source/kingdom attribution.
    pub motif_hits: Vec<RdkitMotifHit>,
}

/// Dataset-wide frequency summary for one structural motif.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MotifSummary {
    /// Motif label.
    pub label: String,
    /// Natural/synthetic/unknown classification.
    pub source_class: SourceClass,
    /// Primary kingdom name.
    pub kingdom: String,
    /// All attributed kingdoms.
    pub kingdoms: Vec<String>,
    /// Number of molecules in the dataset carrying this motif.
    pub count: usize,
}

/// Dataset-level motif prevalence shared across Pass-2 row assessments.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DatasetMotifContext {
    /// Number of molecules containing each motif label.
    pub motif_counts: HashMap<String, usize>,
    /// Minimum molecule count for a motif to be flagged as dataset-common.
    pub common_threshold: usize,
}

/// Whole-molecule `RDKit` descriptors used for ring-family classification.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct RdkitDescriptors {
    /// Fraction of sp3-hybridized carbons (`0.0..=1.0`).
    pub fraction_csp3: Option<f64>,
    /// Total ring count.
    pub ring_count: Option<f64>,
    /// Aromatic ring count.
    pub aromatic_ring_count: Option<f64>,
    /// Aliphatic ring count.
    pub aliphatic_ring_count: Option<f64>,
}

/// Aggregated database annotations keyed by `InChIKey` connectivity layer.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SourceSummary {
    /// Source organism names.
    pub taxa: BTreeSet<String>,
    /// Wikidata QIDs.
    pub compounds: BTreeSet<String>,
    /// `PubChem` CIDs.
    pub cids: BTreeSet<String>,
    /// Wikidata QIDs that also carry taxon records.
    pub compounds_with_taxa: BTreeSet<String>,
}

/// Database lookup maps across LOTUS and `PubChem`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Enrichment {
    /// `InChIKey14 -> SourceSummary` from LOTUS / Wikidata.
    pub lotus: HashMap<String, SourceSummary>,
    /// `InChIKey14 -> SourceSummary` from `PubChem`.
    pub pubchem: HashMap<String, SourceSummary>,
}

/// Reachability telemetry for a single remote SPARQL endpoint.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EndpointStatus {
    /// Human-readable service name.
    pub name: String,
    /// Queried SPARQL endpoint URL.
    pub endpoint: String,
    /// Whether the probe or query succeeded.
    pub reachable: bool,
    /// Short summary or error line.
    pub detail: String,
}

/// Complete enrichment payload plus endpoint reachability and warnings.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EnrichmentOutcome {
    /// Per-database hits.
    pub enrichment: Enrichment,
    /// Endpoint status badges.
    pub endpoints: Vec<EndpointStatus>,
    /// Non-fatal warnings surfaced to the UI banner.
    pub warnings: Vec<String>,
}

/// Raw JSON payload produced by `window.__smilesRdkit.inspect(smiles)`.
#[derive(Debug, Default, Deserialize)]
pub struct RdkitInspectResponse {
    /// Computed `InChIKey`.
    pub inchikey: Option<String>,
    /// Generated 2D SVG markup.
    pub svg: Option<String>,
    /// Matched motif records.
    pub motifs: Option<Vec<RdkitMotifHit>>,
    /// Computed descriptors.
    pub descriptors: Option<RdkitDescriptors>,
    /// Stereocenter tags.
    pub stereo_tags: Option<Vec<String>>,
    /// Ertl NP-likeness score (`None` when the model failed to load).
    #[serde(default)]
    pub np_score: Option<f64>,
    /// Fraction of fingerprint bits found in the Ertl model.
    #[serde(default)]
    pub np_confidence: Option<f64>,
    /// Heavy-atom count used for normalization.
    #[serde(default)]
    pub num_atoms: Option<usize>,
    /// Ertl 2022 natural-product substituent matches.
    #[serde(default)]
    pub substituents: Vec<String>,
    /// LOTUS 1%-prevalent scaffold SMILES matches.
    #[serde(default)]
    pub lotus_scaffolds: Vec<String>,
    /// Structure parsing error from `RDKit.js`, if any.
    pub error: Option<String>,
}

/// Deserializes a JSON string from `window.__smilesRdkit.inspect`.
///
/// # Errors
///
/// Returns [`SmellfishError::InvalidRdkitJson`] if the JSON does not match [`RdkitInspectResponse`].
pub fn parse_rdkit_inspect_json(json: &str) -> Result<RdkitInspectResponse, SmellfishError> {
    Ok(serde_json::from_str(json)?)
}

/// Single structural motif match returned by `RDKit.js`.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct RdkitMotifHit {
    /// Human-readable motif label.
    pub label: String,
    /// Source class (`natural`, `synthetic`, or `unknown`).
    #[serde(default)]
    pub source_class: SourceClass,
    /// Primary kingdom association.
    #[serde(default)]
    pub kingdom: String,
    /// All kingdoms where the motif appears.
    #[serde(default)]
    pub kingdoms: Vec<String>,
}

/// Returns `true` if `haystack` contains `needle` (ASCII case-insensitive).
#[must_use]
pub(crate) fn contains_ignore_ascii_case(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    haystack
        .as_bytes()
        .windows(needle.len())
        .any(|window| window.eq_ignore_ascii_case(needle.as_bytes()))
}

/// Returns `true` if `label` contains any keyword in `motifs` (ASCII case-insensitive).
#[must_use]
pub(crate) fn matches_any(label: &str, motifs: &[&str]) -> bool {
    motifs.iter().any(|m| contains_ignore_ascii_case(label, m))
}
