use serde::Deserialize;
use std::collections::{BTreeSet, HashMap};

#[derive(Clone, Debug)]
pub struct RawRow {
    pub index: usize,
    pub label: String,
    pub smiles: String,
}

#[derive(Clone, Debug)]
pub struct MoleculeRow {
    pub index: usize,
    pub label: String,
    pub smiles: String,
    pub canonical_smiles: String,
    pub inchikey: String,
    pub svg: Option<String>,
    pub motifs: Vec<String>,
    pub lotus_taxa: Vec<String>,
    pub lotus_compounds: Vec<String>,
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
    pub verdict: String,
    pub error: Option<String>,
    /// Stored descriptors so the second-pass evidence assessment can
    /// re-evaluate structural context alongside dataset-level data.
    pub descriptors: RdkitDescriptors,
    /// Stored stereo tags for the same reason.
    pub stereo_tags: Vec<String>,
    /// Heavy-atom count (transparency for the Ertl normalisation).
    pub num_atoms: usize,
}

#[derive(Clone, Debug)]
pub struct MotifSummary {
    pub label: String,
    pub kind: String,
    pub smarts: String,
    pub count: usize,
}

/// Dataset-level motif prevalence used to enrich individual molecule rows.
///
/// `motif_counts` maps each motif label to the number of molecules in the
/// uploaded set that contain it.  Motifs appearing in ≥ `common_threshold`
/// molecules are considered "dataset-common".
#[derive(Clone, Debug, Default)]
pub struct DatasetMotifContext {
    pub motif_counts: HashMap<String, usize>,
    pub total_molecules: usize,
    pub common_threshold: usize,
}

#[derive(Clone, Debug, Default, Deserialize)]
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
pub struct SourceSummary {
    pub taxa: BTreeSet<String>,
    pub compounds: BTreeSet<String>,
    pub names: BTreeSet<String>,
    pub cids: BTreeSet<String>,
}

#[derive(Clone, Debug, Default)]
pub struct Enrichment {
    pub lotus: HashMap<String, SourceSummary>,
    pub pubchem: HashMap<String, SourceSummary>,
}

#[derive(Clone, Debug, Default)]
pub struct EndpointStatus {
    pub name: String,
    pub endpoint: String,
    pub reachable: bool,
    pub detail: String,
}

#[derive(Clone, Debug, Default)]
pub struct EnrichmentOutcome {
    pub enrichment: Enrichment,
    pub endpoints: Vec<EndpointStatus>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct RdkitInspectResponse {
    pub canonicalsmiles: Option<String>,
    pub inchikey: Option<String>,
    pub svg: Option<String>,
    pub motifs: Option<Vec<RdkitMotifHit>>,
    pub descriptors: Option<RdkitDescriptors>,
    pub stereo_tags: Option<Vec<String>>,
    /// Real Ertl NP-likeness score (Ertl et al., J. Chem. Inf. Model. 2008),
    /// `null` when the fragment model was not loaded.
    #[serde(default)]
    pub np_score: Option<f64>,
    /// Confidence = fraction of Morgan bits found in the model (0–1).
    #[serde(default)]
    pub np_confidence: Option<f64>,
    /// Heavy‑atom count used for the score normalisation.
    #[serde(default)]
    pub num_atoms: Option<usize>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RdkitMotifHit {
    pub label: String,
    pub kind: String,
    pub smarts: String,
}
