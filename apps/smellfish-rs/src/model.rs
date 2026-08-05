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
    pub ring_family: String,
    pub evidence_notes: Vec<String>,
    pub verdict: String,
    pub error: Option<String>,
}

#[derive(Clone, Debug)]
pub struct MotifSummary {
    pub label: String,
    pub kind: String,
    pub smarts: String,
    pub count: usize,
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
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RdkitMotifHit {
    pub label: String,
    pub kind: String,
    pub smarts: String,
}
