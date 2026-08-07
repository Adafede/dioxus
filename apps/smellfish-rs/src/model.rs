use serde::Deserialize;
use std::collections::BTreeSet;

#[cfg(any(test, target_arch = "wasm32"))]
#[derive(Clone, Debug)]
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

/// A molecule row displayed in the UI.  Fields marked `#[cfg(...)]` are
/// only needed during the wasm evidence pipeline; they exist so the struct
/// can be fully populated and assessed in the browser.
#[derive(Clone, Debug)]
pub struct MoleculeRow {
    pub index: usize,
    pub label: String,
    pub smiles: String,
    /// Canonical SMILES — used for InChIKey→row mapping during enrichment.
    #[cfg(target_arch = "wasm32")]
    pub canonical_smiles: String,
    /// InChIKey — used for database lookups during enrichment.
    #[cfg(target_arch = "wasm32")]
    pub inchikey: String,
    pub svg: Option<String>,
    pub motifs: Vec<String>,
    /// Substituent pattern -> occurrence count (for multiplicity-aware display)
    pub substituents_counts: std::collections::HashMap<String, usize>,
    /// LOTUS 1-percent scaffold matches (Rutz et al.) — displayed as chips.
    pub lotus_scaffolds: Vec<String>,
    #[cfg(target_arch = "wasm32")]
    pub lotus_taxa: Vec<String>,
    pub lotus_compounds: Vec<String>,
    pub lotus_compounds_with_taxa: BTreeSet<String>,
    pub pubchem_cids: Vec<String>,
    /// Ertl NP-likeness score.
    pub np_likeness: f64,
    /// Human-readable NP-likeness label.
    pub np_label: String,
    /// Confidence in the NP-likeness score.
    #[cfg(target_arch = "wasm32")]
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
    #[cfg(target_arch = "wasm32")]
    pub descriptors: RdkitDescriptors,
    /// Stored stereo tags for the same reason.
    #[cfg(target_arch = "wasm32")]
    pub stereo_tags: Vec<String>,
    /// Heavy-atom count (transparency for the Ertl normalisation).
    pub num_atoms: usize,
    /// Full motif hit metadata for UI grouping.
    pub motif_hits: Vec<RdkitMotifHit>,
}

#[derive(Clone, Debug)]
pub struct MotifSummary {
    pub label: String,
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
#[cfg(any(test, target_arch = "wasm32"))]
pub struct DatasetMotifContext {
    pub motif_counts: std::collections::HashMap<String, usize>,
    pub common_threshold: usize,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[cfg(any(test, target_arch = "wasm32"))]
pub struct RdkitDescriptors {
    pub fraction_csp3: Option<f64>,
    pub ring_count: Option<f64>,
    pub aromatic_ring_count: Option<f64>,
    pub aliphatic_ring_count: Option<f64>,
}

#[derive(Clone, Debug, Default)]
#[cfg(target_arch = "wasm32")]
pub struct SourceSummary {
    pub taxa: BTreeSet<String>,
    pub compounds: BTreeSet<String>,
    pub cids: BTreeSet<String>,
    /// Track which compounds have taxon info.
    pub compounds_with_taxa: BTreeSet<String>,
}

#[derive(Clone, Debug, Default)]
#[cfg(target_arch = "wasm32")]
pub struct Enrichment {
    pub lotus: std::collections::HashMap<String, SourceSummary>,
    pub pubchem: std::collections::HashMap<String, SourceSummary>,
}

#[derive(Clone, Debug, Default)]
pub struct EndpointStatus {
    pub name: String,
    pub endpoint: String,
    pub reachable: bool,
    pub detail: String,
}

#[derive(Clone, Debug, Default)]
#[cfg(target_arch = "wasm32")]
pub struct EnrichmentOutcome {
    pub enrichment: Enrichment,
    pub endpoints: Vec<EndpointStatus>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[cfg(target_arch = "wasm32")]
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
    /// Ertl natural-product substituent matches (top-2000 from Ertl 2022).
    #[serde(default)]
    pub substituents: Vec<String>,
    /// LOTUS 1-percent scaffolds (Rutz et al. mortar fragmentation) —
    /// scaffold SMILES that this molecule contains as a substructure,
    /// filtered to those appearing in > 1 % of LOTUS molecules.
    #[serde(default)]
    pub lotus_scaffolds: Vec<String>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RdkitMotifHit {
    pub label: String,
    #[serde(default)]
    pub source_class: String,
    #[serde(default)]
    pub kingdom: String,
    #[serde(default)]
    pub kingdoms: Vec<String>,
}

/// Normalize a source-class string to one of the three canonical values;
/// anything else becomes `"unknown"`.  Duplicated in `app.rs` historically;
/// consolidated here so all modules share the same logic.
pub fn normalized_source_class(source_class: &str) -> &str {
    match source_class {
        "natural" | "synthetic" | "unknown" => source_class,
        _ => "unknown",
    }
}
