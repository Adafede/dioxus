use serde::Deserialize;
use std::collections::{BTreeSet, HashMap};

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct RawRow {
    pub index: usize,
    pub label: String,
    pub smiles: String,
}

/// A single chemist's check on a molecule — the kind of quick visual
/// audit a natural-product chemist would do when eyeballing a structure.
#[derive(Clone, Debug)]
pub struct ChemistCheck {
    /// Short label, e.g. "NP-likeness", "Skeleton", "Oxygenation".
    pub name: &'static str,
    /// One of "pass", "warn", "fail".
    pub status: &'static str,
    /// Human-readable detail, e.g. "Ertl score +1.3, steroid-like scaffold".
    pub detail: String,
}

/// A single molecule's metadata and evidence assessment.
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct MoleculeRow {
    pub index: usize,
    pub label: String,
    pub smiles: String,
    pub canonical_smiles: String,
    pub inchikey: String,
    pub svg: Option<String>,
    pub motifs: Vec<String>,
    pub substituents: Vec<String>,
    pub lotus_taxa: Vec<String>,
    pub lotus_compounds: Vec<String>,
    pub lotus_compounds_with_taxa: BTreeSet<String>, // Track which LOTUS compounds have taxa
    pub pubchem_cids: Vec<String>,
    pub pubchem_names: Vec<String>,
    pub pubchem_taxa: Vec<String>,
    pub np_likeness: f64,
    pub np_label: String,
    pub np_confidence: f64,
    pub np_score_available: bool,
    pub ring_family: String,
    pub evidence_notes: Vec<String>,
    pub motif_context: String,
    pub chemist_checks: Vec<ChemistCheck>,
    pub verdict: String,
    pub error: Option<String>,
    /// Stored descriptors so the second-pass evidence assessment can
    /// re-evaluate structural context alongside dataset-level data.
    pub descriptors: RdkitDescriptors,
    /// Stored stereo tags for the same reason.
    pub stereo_tags: Vec<String>,
    /// Heavy-atom count (transparency for the Ertl normalisation).
    pub num_atoms: usize,
    /// Full motif hit metadata for UI grouping.
    pub motif_hits: Vec<RdkitMotifHit>,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct MotifSummary {
    pub label: String,
    pub kind: String,
    pub smarts: String,
    pub source_class: String,
    pub kingdom: String,
    pub kingdoms: Vec<String>,
    pub count: usize,
}

/// Dataset-level motif prevalence used to enrich individual molecule rows.
///
/// `motif_counts` maps each motif label to the number of molecules in the
/// uploaded set that contain it.  Motifs appearing in ≥ `common_threshold`
/// molecules are considered "dataset-common".
#[derive(Clone, Debug, Default)]
#[allow(dead_code)]
pub struct DatasetMotifContext {
    pub motif_counts: HashMap<String, usize>,
    pub total_molecules: usize,
    pub common_threshold: usize,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[allow(dead_code)]
pub struct RdkitDescriptors {
    pub amw: Option<f64>,
    pub exact_mw: Option<f64>,
    pub clogp: Option<f64>,
    pub tpsa: Option<f64>,
    pub fraction_csp3: Option<f64>,
    pub ring_count: Option<f64>,
    pub aromatic_ring_count: Option<f64>,
    pub aliphatic_ring_count: Option<f64>,
    pub rotatable_bonds: Option<f64>,
    pub hba: Option<f64>,
    pub hbd: Option<f64>,
    pub hetero_atoms: Option<f64>,
}

#[derive(Clone, Debug, Default)]
#[allow(dead_code)]
pub struct SourceSummary {
    pub taxa: BTreeSet<String>,
    pub compounds: BTreeSet<String>,
    pub names: BTreeSet<String>,
    pub cids: BTreeSet<String>,
    pub compounds_with_taxa: BTreeSet<String>, // Track which compounds have taxon info
}

#[derive(Clone, Debug, Default)]
#[allow(dead_code)]
pub struct Enrichment {
    pub lotus: HashMap<String, SourceSummary>,
    pub pubchem: HashMap<String, SourceSummary>,
}

#[derive(Clone, Debug, Default)]
#[allow(dead_code)]
pub struct EndpointStatus {
    pub name: String,
    pub endpoint: String,
    pub reachable: bool,
    pub detail: String,
}

#[derive(Clone, Debug, Default)]
#[allow(dead_code)]
pub struct EnrichmentOutcome {
    pub enrichment: Enrichment,
    pub endpoints: Vec<EndpointStatus>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct RdkitInspectResponse {
    pub canonicalsmiles: Option<String>,
    pub inchikey: Option<String>,
    pub svg: Option<String>,
    pub motifs: Option<Vec<RdkitMotifHit>>,
    pub descriptors: Option<RdkitDescriptors>,
    pub stereo_tags: Option<Vec<String>>,
    /// Ertl NP-likeness score (Ertl et al., J. Chem. Inf. Model. 2008),
    /// `null` when the fragment model was not loaded.
    #[serde(default)]
    pub np_score: Option<f64>,
    /// Confidence = fraction of Morgan bits found in the model (0–1).
    #[serde(default)]
    pub np_confidence: Option<f64>,
    /// Heavy‑atom count used for the score normalisation.
    #[serde(default)]
    pub num_atoms: Option<usize>,
    /// Ertl natural-product substituent matches (top-60 from Ertl 2022).
    #[serde(default)]
    pub substituents: Vec<String>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)]
pub struct RdkitMotifHit {
    pub label: String,
    pub kind: String,
    pub smarts: String,
    #[serde(default)]
    pub source_class: String,
    #[serde(default)]
    pub kingdom: String,
    #[serde(default)]
    pub kingdoms: Vec<String>,
}
