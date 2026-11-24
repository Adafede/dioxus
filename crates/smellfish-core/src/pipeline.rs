// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! Pure Pass-1 (dataset inspection & motif aggregation) and Pass-2 (enrichment merge & evidence
//! evaluation) orchestration.

use crate::evidence::{
    EvidenceInputs, assess_np_evidence_with_counts, count_evidence, row_verdict_with_counts,
    run_checks,
};
use crate::model::{
    DatasetMotifContext, EndpointStatus, EnrichmentOutcome, InChIKey14, LotusCompound, MoleculeRow,
    MotifSummary, NpScore, RawRow, RdkitInspectResponse, SourceClass, SubstituentCounts, Verdict,
};
use crate::svg::sanitize_svg;
use std::collections::{BTreeSet, HashMap, HashSet};

/// Single-row `RDKit` inspection pair consumed by [`prepare_pass1`].
#[derive(Debug)]
pub struct RowInspection {
    /// Parsed CSV / SMILES input row.
    pub raw: RawRow,
    /// Successful `RDKit` inspection payload or error message.
    pub outcome: Result<RdkitInspectResponse, String>,
}

/// Intermediate Pass-1 batch ready for SPARQL enrichment and [`finalize_pass2`].
#[derive(Debug)]
pub struct PreparedBatch {
    /// Provisional molecule rows before database enrichment and Pass-2 assessment.
    pub rows: Vec<MoleculeRow>,
    /// Sorted dataset-wide motif summaries.
    pub motifs: Vec<MotifSummary>,
    /// Dataset motif prevalence context for Pass-2 evaluation.
    pub dataset_context: DatasetMotifContext,
    /// Sorted deduplicated full `InChIKey`s to query remotely.
    pub unique_inchikeys: Vec<String>,
    /// Mapping from 14-char connectivity layer to indices in `rows`.
    pub connectivity_index: HashMap<String, Vec<usize>>,
}

/// Final screening output produced by [`finalize_pass2`].
#[derive(Debug, PartialEq)]
pub struct ImportOutcome {
    /// Assessed molecule rows.
    pub rows: Vec<MoleculeRow>,
    /// Sorted dataset-wide motif summaries.
    pub motifs: Vec<MotifSummary>,
    /// Number of unique `InChIKey`s queried.
    pub unique_inchikeys: usize,
    /// Count of distinct Ertl substituents matched across the dataset.
    pub unique_substituents: usize,
    /// SPARQL endpoint status chips.
    pub endpoints: Vec<EndpointStatus>,
    /// Non-fatal enrichment warnings.
    pub warnings: Vec<String>,
}

type MotifAccumulator = HashMap<String, (SourceClass, String, BTreeSet<String>, HashSet<usize>)>;

/// Aggregates Pass-1 `RDKit` inspections, sanitizes SVGs once, and builds the enrichment index.
#[must_use]
pub fn prepare_pass1(inspections: Vec<RowInspection>) -> PreparedBatch {
    let mut motif_counts: MotifAccumulator = HashMap::new();
    let mut unique_inchikeys = BTreeSet::new();
    let mut rows = Vec::with_capacity(inspections.len());

    for item in inspections {
        rows.push(build_pass1_row(
            item,
            &mut motif_counts,
            &mut unique_inchikeys,
        ));
    }

    let motifs = sorted_motifs(
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
    let dataset_context = compute_dataset_context(&motifs, rows.len());
    let connectivity_index = build_connectivity_index(&rows);

    PreparedBatch {
        rows,
        motifs,
        dataset_context,
        unique_inchikeys: unique_inchikeys.into_iter().collect(),
        connectivity_index,
    }
}

fn build_pass1_row(
    item: RowInspection,
    motif_counts: &mut MotifAccumulator,
    unique_inchikeys: &mut BTreeSet<String>,
) -> MoleculeRow {
    let RawRow {
        index,
        label,
        smiles,
    } = item.raw;
    let inspect = match item.outcome {
        Ok(inspect) => inspect,
        Err(err) => return error_row(index, label, smiles, err),
    };
    if let Some(err) = inspect.error {
        return error_row(index, label, smiles, err);
    }

    let motif_hits = inspect.motifs.unwrap_or_default();
    let motifs = motif_hits.iter().map(|h| h.label.clone()).collect();
    for hit in &motif_hits {
        let entry = motif_counts.entry(hit.label.clone()).or_insert_with(|| {
            (
                hit.source_class,
                hit.kingdom.clone(),
                hit.kingdoms.iter().cloned().collect(),
                HashSet::new(),
            )
        });
        entry.2.extend(hit.kingdoms.iter().cloned());
        entry.3.insert(index);
    }

    let inchikey = inspect.inchikey.unwrap_or_default();
    if !inchikey.is_empty() {
        unique_inchikeys.insert(inchikey.clone());
    }
    let substituents_counts =
        inspect
            .substituents
            .into_iter()
            .fold(SubstituentCounts::new(), |mut acc, sub| {
                *acc.entry(sub).or_insert(0) += 1;
                acc
            });
    let np_score = inspect.np_score.map(|score| NpScore {
        score,
        confidence: inspect.np_confidence.unwrap_or(0.0),
    });

    MoleculeRow {
        index,
        label,
        smiles,
        inchikey,
        svg: sanitize_svg(inspect.svg),
        motifs,
        motif_hits,
        substituents_counts,
        lotus_scaffolds: inspect.lotus_scaffolds,
        np_score,
        descriptors: inspect.descriptors.unwrap_or_default(),
        stereo_tags: inspect.stereo_tags.unwrap_or_default(),
        num_atoms: inspect.num_atoms.unwrap_or(0),
        ..MoleculeRow::default()
    }
}

/// Merges database enrichment results and completes Pass-2 evidence, checks, and verdicts.
#[must_use]
pub fn finalize_pass2(mut batch: PreparedBatch, enrichment: EnrichmentOutcome) -> ImportOutcome {
    merge_enrichment(&mut batch.rows, &enrichment, &batch.connectivity_index);

    let unique_substituents = batch
        .rows
        .iter()
        .flat_map(|r| r.substituents_counts.keys())
        .collect::<HashSet<_>>()
        .len();

    for row in &mut batch.rows {
        let counts = count_evidence(&row.motifs, &row.motif_hits);
        if row.error.is_none() {
            let evidence = assess_np_evidence_with_counts(
                &EvidenceInputs {
                    descriptors: &row.descriptors,
                    motifs: &row.motifs,
                    motif_hits: &row.motif_hits,
                    stereo_tags: &row.stereo_tags,
                    np_score: row.np_score,
                    dataset_context: &batch.dataset_context,
                    lotus_scaffolds: &row.lotus_scaffolds,
                },
                counts,
            );
            row.np_label = evidence.np_label;
            row.ring_family = evidence.ring_family;
            row.motif_context = evidence.motif_context;
            let mut notes = evidence.evidence_notes;
            notes.push(evidence.motif_context.as_str().to_string());
            row.evidence_notes = notes;
            row.chemist_checks = run_checks(row);
        }
        row.verdict = row_verdict_with_counts(row, counts);
    }

    ImportOutcome {
        rows: batch.rows,
        motifs: batch.motifs,
        unique_inchikeys: batch.unique_inchikeys.len(),
        unique_substituents,
        endpoints: enrichment.endpoints,
        warnings: enrichment.warnings,
    }
}

fn build_connectivity_index(rows: &[MoleculeRow]) -> HashMap<String, Vec<usize>> {
    let mut map: HashMap<String, Vec<usize>> = HashMap::new();
    for (idx, row) in rows.iter().enumerate() {
        if let Some(key) = InChIKey14::from_inchikey(&row.inchikey) {
            map.entry(key.as_str().to_string()).or_default().push(idx);
        }
    }
    map
}

fn merge_enrichment(
    rows: &mut [MoleculeRow],
    outcome: &EnrichmentOutcome,
    connectivity_index: &HashMap<String, Vec<usize>>,
) {
    for (connectivity, indices) in connectivity_index {
        if let Some(summary) = outcome.enrichment.lotus.get(connectivity) {
            for &idx in indices {
                if let Some(row) = rows.get_mut(idx) {
                    row.lotus_taxa = summary.taxa.iter().cloned().collect();
                    row.lotus_compounds = summary
                        .compounds
                        .iter()
                        .map(|qid| LotusCompound {
                            has_taxa: summary.compounds_with_taxa.contains(qid),
                            qid: qid.clone(),
                        })
                        .collect();
                }
            }
        }
        if let Some(summary) = outcome.enrichment.pubchem.get(connectivity) {
            for &idx in indices {
                if let Some(row) = rows.get_mut(idx) {
                    row.pubchem_cids = summary.cids.iter().cloned().collect();
                }
            }
        }
    }
}

fn compute_dataset_context(motif_summaries: &[MotifSummary], total: usize) -> DatasetMotifContext {
    DatasetMotifContext {
        motif_counts: motif_summaries
            .iter()
            .map(|motif| (motif.label.clone(), motif.count))
            .collect(),
        common_threshold: total.div_ceil(10).max(2),
    }
}

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

fn error_row(index: usize, label: String, smiles: String, error: String) -> MoleculeRow {
    MoleculeRow {
        index,
        label,
        smiles,
        np_label: "—".to_string(),
        verdict: Verdict::error(&error),
        error: Some(error),
        ..MoleculeRow::default()
    }
}
