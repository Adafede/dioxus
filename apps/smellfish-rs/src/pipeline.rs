use crate::csv::parse_csv_rows;
use crate::evidence::{assess_np_evidence, verdict_for_row};
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
    label: String,
    smiles: String,
    canonical_smiles: String,
    inchikey: String,
    svg: Option<String>,
    motif_labels: Vec<String>,
    descriptors: RdkitDescriptors,
    stereo_tags: Vec<String>,
    np_score: Option<f64>,
    np_confidence: Option<f64>,
    num_atoms: usize,
}

#[cfg(target_arch = "wasm32")]
pub async fn import_csv(file: web_sys::File) -> Result<ImportOutcome, String> {
    let text = read_file_text(&file).await?;
    let raw_rows = parse_csv_rows(&text)?;

    let mut inspect_rows = Vec::with_capacity(raw_rows.len());
    let mut motif_counts: HashMap<String, (String, String, HashSet<usize>)> = HashMap::new();
    let mut inchikeys = BTreeSet::new();
    let mut rows = Vec::with_capacity(raw_rows.len());

    // ═══════════════════════════════════════════════════════════════════
    // PASS 1 — RDKit inspection + raw data collection
    // ═══════════════════════════════════════════════════════════════════
    for raw in raw_rows {
        match rdkit_inspect(&raw.smiles).await {
            Ok(inspect) => {
                if let Some(err) = inspect.error {
                    rows.push(error_row(raw.index, raw.label, raw.smiles, err));
                    continue;
                }

                let motifs_list = inspect.motifs.unwrap_or_default();
                let motif_labels = motifs_list
                    .iter()
                    .map(|hit| hit.label.clone())
                    .collect::<Vec<_>>();

                // Record per-motif molecule membership for dataset prevalence.
                for hit in &motifs_list {
                    let entry = motif_counts
                        .entry(hit.label.clone())
                        .or_insert_with(|| (hit.kind.clone(), hit.smarts.clone(), HashSet::new()));
                    entry.2.insert(raw.index);
                }

                let inchikey = inspect.inchikey.unwrap_or_default();
                if !inchikey.is_empty() {
                    inchikeys.insert(inchikey.clone());
                }

                let canonical = inspect.canonicalsmiles.unwrap_or_default();
                let descriptors = inspect.descriptors.unwrap_or_default();
                let stereo_tags = inspect.stereo_tags.unwrap_or_default();

                inspect_rows.push(RawInspectRow {
                    index: raw.index,
                    label: raw.label.clone(),
                    smiles: raw.smiles.clone(),
                    canonical_smiles: canonical.clone(),
                    inchikey: inchikey.clone(),
                    svg: inspect.svg.clone(),
                    motif_labels: motif_labels.clone(),
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
                    lotus_taxa: Vec::new(),
                    lotus_compounds: Vec::new(),
                    pubchem_cids: Vec::new(),
                    pubchem_names: Vec::new(),
                    pubchem_taxa: Vec::new(),
                    np_likeness: 0.0,
                    np_label: String::new(),
                    np_confidence: 0.0,
                    np_score_available: false,
                    ring_family: String::new(),
                    evidence_notes: Vec::new(),
                    motif_context: String::new(),
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
            .map(|(label, (kind, smarts, rows_set))| MotifSummary {
                label,
                kind,
                smarts,
                count: rows_set.len(),
            })
            .collect(),
    );

    // ═══════════════════════════════════════════════════════════════════
    // Dataset-level context — computed once, used by every row
    // ═══════════════════════════════════════════════════════════════════
    let dataset_context = compute_dataset_context(&motif_summary, rows.len());

    // ═══════════════════════════════════════════════════════════════════
    // Database enrichment (LOTUS / PubChem) — batched for all rows
    // ═══════════════════════════════════════════════════════════════════
    let unique_keys = inchikeys.into_iter().collect::<Vec<_>>();
    let enrichment_outcome = enrich_sources(&unique_keys).await;
    let mut rows = merge_enrichment(rows, &enrichment_outcome);

    // ═══════════════════════════════════════════════════════════════════
    // PASS 2 — Evidence assessment using dataset context + Ertl score
    // ═══════════════════════════════════════════════════════════════════
    // Merge the stored raw inspect data back onto the rows.
    let mut raw_by_index: HashMap<usize, RawInspectRow> =
        inspect_rows.into_iter().map(|r| (r.index, r)).collect();

    for row in &mut rows {
        if let Some(raw) = raw_by_index.remove(&row.index) {
            row.descriptors = raw.descriptors;
            row.stereo_tags = raw.stereo_tags;
            row.num_atoms = raw.num_atoms;
            row.np_score_available = raw.np_score.is_some();

            // Build motif label vec for dataset-common detection.
            let motif_labels = &raw.motif_labels;

            let evidence = assess_np_evidence(
                &row.descriptors,
                motif_labels,
                &row.stereo_tags,
                raw.np_score,
                raw.np_confidence,
                raw.num_atoms,
                &dataset_context,
            );

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

    // Compute verdicts — needs database evidence + Ertl score + model flag.
    for row in &mut rows {
        row.verdict = verdict_for_row(row);
    }

    Ok(ImportOutcome {
        rows,
        motifs: motif_summary,
        unique_inchikeys: unique_keys.len(),
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
        total_molecules: total,
        common_threshold,
    }
}

#[cfg(target_arch = "wasm32")]
pub struct ImportOutcome {
    pub rows: Vec<MoleculeRow>,
    pub motifs: Vec<MotifSummary>,
    pub unique_inchikeys: usize,
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
    status.set("Reading CSV…".to_string());
    rows.set(Vec::new());
    motifs.set(Vec::new());
    endpoints.set(Vec::new());
    warnings.set(Vec::new());

    spawn(async move {
        match import_csv(file).await {
            Ok(outcome) => {
                let row_count = outcome.rows.len();
                let motif_count = outcome.motifs.len();
                rows.set(outcome.rows);
                motifs.set(outcome.motifs);
                endpoints.set(outcome.endpoints);
                warnings.set(outcome.warnings.clone());
                if outcome.warnings.is_empty() {
                    status.set(format!(
                        "Done — {row_count} rows, {motif_count} motifs, {unique} unique InChIKeys",
                        unique = outcome.unique_inchikeys
                    ));
                } else {
                    status.set(format!(
                        "Done with QLever warnings — {row_count} rows, {motif_count} motifs, {unique} unique InChIKeys",
                        unique = outcome.unique_inchikeys
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
fn merge_enrichment(
    rows: Vec<MoleculeRow>,
    enrichment_outcome: &EnrichmentOutcome,
) -> Vec<MoleculeRow> {
    rows.into_iter()
        .map(|mut row| {
            let enrichment = &enrichment_outcome.enrichment;
            if !row.inchikey.is_empty() {
                if let Some(summary) = enrichment.lotus.get(&row.inchikey) {
                    row.lotus_taxa = summary.taxa.iter().cloned().collect();
                    row.lotus_compounds = summary.compounds.iter().cloned().collect();
                }
                if let Some(summary) = enrichment.pubchem.get(&row.inchikey) {
                    row.pubchem_cids = summary.cids.iter().cloned().collect();
                    row.pubchem_names = summary.names.iter().cloned().collect();
                    row.pubchem_taxa = summary.taxa.iter().cloned().collect();
                }
            }
            row
        })
        .collect()
}

#[cfg(target_arch = "wasm32")]
fn sorted_motifs(mut motifs: Vec<MotifSummary>) -> Vec<MotifSummary> {
    motifs.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then(left.kind.cmp(&right.kind))
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
        lotus_taxa: Vec::new(),
        lotus_compounds: Vec::new(),
        pubchem_cids: Vec::new(),
        pubchem_names: Vec::new(),
        pubchem_taxa: Vec::new(),
        np_likeness: 0.0,
        np_label: "—".to_string(),
        np_confidence: 0.0,
        np_score_available: false,
        ring_family: String::new(),
        evidence_notes: Vec::new(),
        motif_context: String::new(),
        verdict: String::new(),
        error: Some(error),
        descriptors: RdkitDescriptors::default(),
        stereo_tags: Vec::new(),
        num_atoms: 0,
    }
}
