use crate::csv::parse_csv_rows;
use crate::evidence::{EvidenceInputs, assess_np_evidence, row_verdict, run_checks};
use crate::model::{
    DatasetMotifContext, EndpointStatus, EnrichmentOutcome, MoleculeRow, MotifSummary,
    RdkitDescriptors,
};
#[cfg(target_arch = "wasm32")]
use crate::qlever::enrich_sources;
#[cfg(target_arch = "wasm32")]
use crate::rdkit::{rdkit_inspect, read_file_text};
use dioxus::prelude::{Signal, WritableExt, spawn};
use std::collections::{BTreeSet, HashMap, HashSet};

/// A molecule that has been processed by RDKit but not yet assessed for NP
/// evidence.  This intermediate representation lets us compute dataset-level
/// motif prevalence *before* evaluating each row.
struct RawInspectRow {
    index: usize,
    motif_labels: Vec<String>,
    motif_hits: Vec<crate::model::RdkitMotifHit>,
    substituents_counts: HashMap<String, usize>,
    lotus_scaffolds: Vec<String>,
    descriptors: RdkitDescriptors,
    stereo_tags: Vec<String>,
    np_score: Option<f64>,
    np_confidence: Option<f64>,
    num_atoms: usize,
}

#[cfg(target_arch = "wasm32")]
pub async fn import_csv(
    file: web_sys::File,
    mut status: Signal<String>,
) -> Result<ImportOutcome, String> {
    status.set("Reading CSV file…".to_string());
    let text = read_file_text(&file).await?;
    import_csv_text(&text, status).await
}

#[cfg(target_arch = "wasm32")]
async fn import_csv_text(text: &str, mut status: Signal<String>) -> Result<ImportOutcome, String> {
    status.set("Parsing CSV…".to_string());
    let raw_rows = parse_csv_rows(text)?;
    let total = raw_rows.len();

    let mut inspect_rows = Vec::with_capacity(total);
    let mut motif_counts: HashMap<
        String,
        (
            String,
            String,
            std::collections::BTreeSet<String>,
            HashSet<usize>,
        ),
    > = HashMap::new();
    let mut inchikeys = BTreeSet::new();
    let mut smiles_list = Vec::new();
    let mut rows = Vec::with_capacity(total);

    // ═══════════════════════════════════════════════════════════════════
    // PASS 1 — RDKit inspection + raw data collection
    // ═══════════════════════════════════════════════════════════════════
    status.set("Loading RDKit module…".to_string());
    for (i, raw) in raw_rows.into_iter().enumerate() {
        status.set(format!("Inspecting structure {i}/{total}…"));
        match rdkit_inspect(&raw.smiles).await {
            Ok(inspect) => {
                if let Some(err) = inspect.error {
                    rows.push(error_row(raw.index, raw.label, raw.smiles, err));
                    continue;
                }

                let inspect = inspect; // consume
                let motifs_list = inspect.motifs.unwrap_or_default();
                let motif_labels = motifs_list
                    .iter()
                    .map(|hit| hit.label.clone())
                    .collect::<Vec<_>>();

                // Record per-motif molecule membership for dataset prevalence.
                for hit in &motifs_list {
                    let entry = motif_counts.entry(hit.label.clone()).or_insert_with(|| {
                        (
                            hit.source_class.clone(),
                            hit.kingdom.clone(),
                            hit.kingdoms.iter().cloned().collect(),
                            HashSet::new(),
                        )
                    });
                    entry.2.extend(hit.kingdoms.iter().cloned());
                    entry.3.insert(raw.index);
                }

                let inchikey = inspect.inchikey.unwrap_or_default();
                if !inchikey.is_empty() {
                    inchikeys.insert(inchikey.clone());
                }
                smiles_list.push(raw.smiles.clone());

                let canonical = inspect.canonicalsmiles.unwrap_or_default();
                let descriptors = inspect.descriptors.unwrap_or_default();
                let stereo_tags = inspect.stereo_tags.unwrap_or_default();
                // Count substituent occurrences for proper multiplicity display
                let substituents_counts: std::collections::HashMap<String, usize> = inspect
                    .substituents
                    .iter()
                    .fold(std::collections::HashMap::new(), |mut acc, s| {
                        *acc.entry(s.clone()).or_insert(0) += 1;
                        acc
                    });
                let lotus_scaffolds = inspect.lotus_scaffolds;

                inspect_rows.push(RawInspectRow {
                    index: raw.index,
                    motif_labels: motif_labels.clone(),
                    motif_hits: motifs_list.clone(),
                    substituents_counts,
                    lotus_scaffolds,
                    descriptors,
                    stereo_tags,
                    np_score: inspect.np_score,
                    np_confidence: inspect.np_confidence,
                    num_atoms: inspect.num_atoms.unwrap_or(0),
                });

                // Placeholder row — evidence is filled in pass 2.
                rows.push(MoleculeRow {
                    index: raw.index,
                    label: raw.label,
                    smiles: raw.smiles,
                    canonical_smiles: canonical,
                    inchikey,
                    svg: inspect.svg,
                    motifs: motif_labels,
                    motif_hits: motifs_list,
                    substituents_counts: std::collections::HashMap::new(),
                    lotus_scaffolds: Vec::new(),
                    lotus_taxa: Vec::new(),
                    lotus_compounds: Vec::new(),
                    lotus_compounds_with_taxa: BTreeSet::new(),
                    pubchem_cids: Vec::new(),
                    np_likeness: 0.0,
                    np_label: String::new(),
                    np_confidence: 0.0,
                    np_score_available: false,
                    ring_family: String::new(),
                    evidence_notes: Vec::new(),
                    motif_context: String::new(),
                    chemist_checks: Vec::new(),
                    verdict: String::new(),
                    error: None,
                    descriptors: RdkitDescriptors::default(),
                    stereo_tags: Vec::new(),
                    num_atoms: 0,
                });
            }
            Err(err) => rows.push(error_row(raw.index, raw.label, raw.smiles, err)),
        }
    }

    let motif_summary = sorted_motifs(
        motif_counts
            .into_iter()
            .map(
                |(label, (source_class, kingdom, kingdoms, rows_set))| MotifSummary {
                    label,
                    source_class,
                    kingdom,
                    kingdoms: kingdoms.into_iter().collect(),
                    count: rows_set.len(),
                },
            )
            .collect(),
    );

    // ═══════════════════════════════════════════════════════════════════
    // Dataset-level context — computed once, used by every row
    // ═══════════════════════════════════════════════════════════════════
    let dataset_context = compute_dataset_context(&motif_summary, rows.len());

    // ═══════════════════════════════════════════════════════════════════
    // Database enrichment (LOTUS / PubChem) — batched for all rows
    // ═══════════════════════════════════════════════════════════════════
    status.set("Probing LOTUS and PubChem…".to_string());
    let unique_keys = inchikeys.into_iter().collect::<Vec<_>>();

    // Build mapping from InChIKey 14-char connectivity layer to row indices for matching results
    let inchikey_to_indices: std::collections::HashMap<String, Vec<usize>> = {
        let mut map = std::collections::HashMap::new();
        for (idx, row) in rows.iter().enumerate() {
            if !row.inchikey.is_empty() {
                let key = row
                    .inchikey
                    .split('-')
                    .next()
                    .unwrap_or(&row.inchikey)
                    .to_string();
                map.entry(key).or_insert_with(Vec::new).push(idx);
            }
        }
        map
    };

    let enrichment_outcome =
        enrich_sources(&unique_keys, &smiles_list, |msg| status.set(msg)).await;
    let mut rows = merge_enrichment(rows, &enrichment_outcome, &inchikey_to_indices);

    // ═══════════════════════════════════════════════════════════════════
    // PASS 2 — Evidence assessment using dataset context + Ertl score
    // ═══════════════════════════════════════════════════════════════════
    status.set("Assessing NP evidence…".to_string());

    // Count unique Ertl substituents found across the dataset.
    let unique_substituents: usize = inspect_rows
        .iter()
        .flat_map(|r| r.substituents_counts.keys())
        .collect::<std::collections::HashSet<_>>()
        .len();

    // Merge the stored raw inspect data back onto the rows.
    let mut raw_by_index: HashMap<usize, RawInspectRow> =
        inspect_rows.into_iter().map(|r| (r.index, r)).collect();

    for row in &mut rows {
        if let Some(raw) = raw_by_index.remove(&row.index) {
            row.descriptors = raw.descriptors;
            row.substituents_counts = raw.substituents_counts;
            row.lotus_scaffolds = raw.lotus_scaffolds;
            row.stereo_tags = raw.stereo_tags;
            row.num_atoms = raw.num_atoms;
            row.motif_hits = raw.motif_hits;
            row.np_score_available = raw.np_score.is_some();

            // Build motif label vec for dataset-common detection.
            let motif_labels = &raw.motif_labels;

            let evidence = assess_np_evidence(EvidenceInputs {
                descriptors: &row.descriptors,
                motifs: motif_labels,
                motif_hits: &row.motif_hits,
                stereo_tags: &row.stereo_tags,
                np_score: raw.np_score,
                np_confidence: raw.np_confidence,
                dataset_context: &dataset_context,
                lotus_scaffolds: &row.lotus_scaffolds,
            });

            row.np_likeness = evidence.np_likeness;
            row.np_label = evidence.np_label;
            row.np_confidence = evidence.np_confidence;
            row.ring_family = evidence.ring_family;
            row.motif_context = evidence.motif_context.clone();
            let mut notes = evidence.evidence_notes;
            notes.push(evidence.motif_context);
            row.evidence_notes = notes;
        }
    }

    // Compute chemist's checklist — needs database evidence + Ertl score +
    // structural descriptors, all of which are now available.
    for row in &mut rows {
        row.chemist_checks = run_checks(row);
    }

    // Compute verdicts — needs database evidence + Ertl score + model flag.
    for row in &mut rows {
        row.verdict = row_verdict(row);
    }

    Ok(ImportOutcome {
        rows,
        motifs: motif_summary,
        unique_inchikeys: unique_keys.len(),
        unique_substituents,
        endpoints: enrichment_outcome.endpoints,
        warnings: enrichment_outcome.warnings,
    })
}

/// Build a `DatasetMotifContext` from the per-motif molecule counts.
///
/// A motif is considered "dataset-common" when it appears in ≥
/// `common_threshold` molecules, where `common_threshold` is the larger of
/// `ceil(total / 10)` and 2 — i.e. at least 10 % of the set (minimum 2).
#[cfg(target_arch = "wasm32")]
fn compute_dataset_context(motif_summaries: &[MotifSummary], total: usize) -> DatasetMotifContext {
    let mut motif_counts: HashMap<String, usize> = HashMap::new();
    for motif in motif_summaries {
        motif_counts.insert(motif.label.clone(), motif.count);
    }

    let common_threshold = ((total as f64 / 10.0).ceil() as usize).max(2);

    DatasetMotifContext {
        motif_counts,
        common_threshold,
    }
}

#[cfg(target_arch = "wasm32")]
pub struct ImportOutcome {
    pub rows: Vec<MoleculeRow>,
    pub motifs: Vec<MotifSummary>,
    pub unique_inchikeys: usize,
    pub unique_substituents: usize,
    pub endpoints: Vec<EndpointStatus>,
    pub warnings: Vec<String>,
}

#[cfg(target_arch = "wasm32")]
pub fn begin_import(
    file: web_sys::File,
    file_name_value: String,
    mut file_name: Signal<String>,
    mut status: Signal<String>,
    mut busy: Signal<bool>,
    mut drag_active: Signal<bool>,
    mut rows: Signal<Vec<MoleculeRow>>,
    mut motifs: Signal<Vec<MotifSummary>>,
    mut endpoints: Signal<Vec<EndpointStatus>>,
    mut warnings: Signal<Vec<String>>,
) {
    file_name.set(file_name_value);
    busy.set(true);
    drag_active.set(false);
    status.set("Preparing…".to_string());
    rows.set(Vec::new());
    motifs.set(Vec::new());
    endpoints.set(Vec::new());
    warnings.set(Vec::new());

    spawn(async move {
        match import_csv(file, status).await {
            Ok(outcome) => {
                let row_count = outcome.rows.len();
                let motif_count = outcome.motifs.len();
                rows.set(outcome.rows);
                motifs.set(outcome.motifs);
                endpoints.set(outcome.endpoints);
                warnings.set(outcome.warnings.clone());
                if outcome.warnings.is_empty() {
                    status.set(format!(
                        "Done — {row_count} results, {motif_count} motifs, {sub_count} Ertl substituents, {unique} unique InChIKeys",
                        unique = outcome.unique_inchikeys,
                        sub_count = outcome.unique_substituents
                    ));
                } else {
                    status.set(format!(
                        "Done with QLever warnings — {row_count} results, {motif_count} motifs, {sub_count} Ertl substituents, {unique} unique InChIKeys",
                        unique = outcome.unique_inchikeys,
                        sub_count = outcome.unique_substituents
                    ));
                }
            }
            Err(err) => {
                status.set(format!("Error reading CSV: {err}"));
            }
        }
        busy.set(false);
    });
}

#[cfg(target_arch = "wasm32")]
pub fn begin_import_from_text(
    text: String,
    file_name_value: String,
    mut file_name: Signal<String>,
    mut status: Signal<String>,
    mut busy: Signal<bool>,
    mut drag_active: Signal<bool>,
    mut rows: Signal<Vec<MoleculeRow>>,
    mut motifs: Signal<Vec<MotifSummary>>,
    mut endpoints: Signal<Vec<EndpointStatus>>,
    mut warnings: Signal<Vec<String>>,
) {
    file_name.set(file_name_value);
    busy.set(true);
    drag_active.set(false);
    status.set("Preparing demo…".to_string());
    rows.set(Vec::new());
    motifs.set(Vec::new());
    endpoints.set(Vec::new());
    warnings.set(Vec::new());

    spawn(async move {
        match import_csv_text(&text, status).await {
            Ok(outcome) => {
                let row_count = outcome.rows.len();
                let motif_count = outcome.motifs.len();
                rows.set(outcome.rows);
                motifs.set(outcome.motifs);
                endpoints.set(outcome.endpoints);
                warnings.set(outcome.warnings.clone());
                if outcome.warnings.is_empty() {
                    status.set(format!(
                        "Demo loaded — {row_count} results, {motif_count} motifs, {sub_count} Ertl substituents, {unique} unique InChIKeys",
                        unique = outcome.unique_inchikeys,
                        sub_count = outcome.unique_substituents
                    ));
                } else {
                    status.set(format!(
                        "Demo loaded with QLever warnings — {row_count} results, {motif_count} motifs, {sub_count} Ertl substituents, {unique} unique InChIKeys",
                        unique = outcome.unique_inchikeys,
                        sub_count = outcome.unique_substituents
                    ));
                }
            }
            Err(err) => {
                status.set(format!("Error loading demo data: {err}"));
            }
        }
        busy.set(false);
    });
}

#[cfg(target_arch = "wasm32")]
fn merge_enrichment(
    mut rows: Vec<MoleculeRow>,
    enrichment_outcome: &EnrichmentOutcome,
    inchikey_to_indices: &std::collections::HashMap<String, Vec<usize>>,
) -> Vec<MoleculeRow> {
    web_sys::console::log_1(
        &format!(
            "merge_enrichment: {} inchikeys to process",
            inchikey_to_indices.len()
        )
        .into(),
    );
    web_sys::console::log_1(
        &format!(
            "LOTUS hits available: {}",
            enrichment_outcome.enrichment.lotus.len()
        )
        .into(),
    );
    web_sys::console::log_1(
        &format!(
            "PubChem hits available: {}",
            enrichment_outcome.enrichment.pubchem.len()
        )
        .into(),
    );

    for (inchikey_connectivity, indices) in inchikey_to_indices {
        // Check lotus by the InChIKey connectivity layer key
        if let Some(summary) = enrichment_outcome
            .enrichment
            .lotus
            .get(inchikey_connectivity)
        {
            web_sys::console::log_1(
                &format!(
                    "Found LOTUS hit for {}: {} QIDs",
                    inchikey_connectivity,
                    summary.compounds.len()
                )
                .into(),
            );
            for &idx in indices {
                if idx < rows.len() {
                    rows[idx].lotus_taxa = summary.taxa.iter().cloned().collect();
                    rows[idx].lotus_compounds = summary.compounds.iter().cloned().collect();
                    rows[idx].lotus_compounds_with_taxa = summary.compounds_with_taxa.clone();
                }
            }
        }
        // Check pubchem by the InChIKey connectivity layer key
        if let Some(summary) = enrichment_outcome
            .enrichment
            .pubchem
            .get(inchikey_connectivity)
        {
            web_sys::console::log_1(
                &format!(
                    "Found PubChem hit for {}: {} CIDs",
                    inchikey_connectivity,
                    summary.cids.len()
                )
                .into(),
            );
            for &idx in indices {
                if idx < rows.len() {
                    rows[idx].pubchem_cids = summary.cids.iter().cloned().collect();
                }
            }
        }
    }
    web_sys::console::log_1(&format!("merge_enrichment complete").into());
    rows
}

#[cfg(target_arch = "wasm32")]
fn sorted_motifs(mut motifs: Vec<MotifSummary>) -> Vec<MotifSummary> {
    motifs.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then(left.source_class.cmp(&right.source_class))
            .then(left.kingdom.cmp(&right.kingdom))
            .then(left.kingdoms.len().cmp(&right.kingdoms.len()))
            .then(left.label.cmp(&right.label))
    });
    motifs
}

#[cfg(target_arch = "wasm32")]
fn error_row(index: usize, label: String, smiles: String, error: String) -> MoleculeRow {
    MoleculeRow {
        index,
        label,
        smiles,
        canonical_smiles: String::new(),
        inchikey: String::new(),
        svg: None,
        motifs: Vec::new(),
        substituents_counts: std::collections::HashMap::new(),
        lotus_scaffolds: Vec::new(),
        lotus_taxa: Vec::new(),
        lotus_compounds: Vec::new(),
        lotus_compounds_with_taxa: BTreeSet::new(),
        pubchem_cids: Vec::new(),
        np_likeness: 0.0,
        np_label: "—".to_string(),
        np_confidence: 0.0,
        np_score_available: false,
        ring_family: String::new(),
        evidence_notes: Vec::new(),
        motif_context: String::new(),
        chemist_checks: Vec::new(),
        verdict: String::new(),
        error: Some(error),
        descriptors: RdkitDescriptors::default(),
        stereo_tags: Vec::new(),
        num_atoms: 0,
        motif_hits: Vec::new(),
    }
}
