use crate::csv::parse_csv_rows;
use crate::evidence::{assess_np_evidence, verdict_for_row};
use crate::model::{EndpointStatus, EnrichmentOutcome, MoleculeRow, MotifSummary};
#[cfg(target_arch = "wasm32")]
use crate::qlever::enrich_sources;
#[cfg(target_arch = "wasm32")]
use crate::rdkit::{rdkit_inspect, read_file_text};
use dioxus::prelude::{Signal, WritableExt, spawn};
use std::collections::{BTreeSet, HashMap, HashSet};

#[cfg(target_arch = "wasm32")]
pub async fn import_csv(file: web_sys::File) -> Result<ImportOutcome, String> {
    let text = read_file_text(&file).await?;
    let raw_rows = parse_csv_rows(&text)?;
    let mut parsed_rows = Vec::with_capacity(raw_rows.len());
    let mut motif_counts: HashMap<String, (String, String, HashSet<usize>)> = HashMap::new();
    let mut inchikeys = BTreeSet::new();
    for raw in raw_rows {
        match rdkit_inspect(&raw.smiles).await {
            Ok(inspect) => {
                if let Some(err) = inspect.error {
                    parsed_rows.push(error_row(raw.index, raw.label, raw.smiles, err));
                    continue;
                }

                let motifs_list = inspect.motifs.unwrap_or_default();
                let motif_labels = motifs_list
                    .iter()
                    .map(|hit| hit.label.clone())
                    .collect::<Vec<_>>();
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
                let evidence = assess_np_evidence(&descriptors, &motif_labels, &stereo_tags);

                parsed_rows.push(MoleculeRow {
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
                    np_likeness: evidence.np_likeness,
                    np_label: evidence.np_label,
                    ring_family: evidence.ring_family,
                    evidence_notes: {
                        let mut notes = evidence.evidence_notes;
                        notes.push(evidence.motif_context);
                        notes
                    },
                    verdict: String::new(),
                    error: None,
                });
            }
            Err(err) => parsed_rows.push(error_row(raw.index, raw.label, raw.smiles, err)),
        }
    }

    let motif_summary = motif_counts
        .into_iter()
        .map(|(label, (kind, smarts, rows))| MotifSummary {
            label,
            kind,
            smarts,
            count: rows.len(),
        })
        .collect::<Vec<_>>();

    let unique_keys = inchikeys.into_iter().collect::<Vec<_>>();
    let enrichment_outcome = enrich_sources(&unique_keys).await;
    let mut parsed_rows = merge_enrichment(parsed_rows, &enrichment_outcome);
    for row in &mut parsed_rows {
        row.verdict = verdict_for_row(row);
    }

    Ok(ImportOutcome {
        rows: parsed_rows,
        motifs: sorted_motifs(motif_summary),
        unique_inchikeys: unique_keys.len(),
        endpoints: enrichment_outcome.endpoints,
        warnings: enrichment_outcome.warnings,
    })
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
    status.set("Reading CSV...".to_string());
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
        ring_family: String::new(),
        evidence_notes: Vec::new(),
        verdict: String::new(),
        error: Some(error),
    }
}
