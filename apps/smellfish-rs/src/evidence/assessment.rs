// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Evidence assessment: builds an [`EvidenceAssessment`] for a single
//! molecule from the Ertl NP-likeness score plus structural motif context.
//! Depends on `verdict` (ring-family classification) and `chemist` (motif
//! counting), so it sits at the top of the evidence dependency graph.

use crate::model::{DatasetMotifContext, RdkitDescriptors, RdkitMotifHit};

use super::chemist::{EvidenceCounts, count_evidence, is_known_np_motif};
use super::verdict::classify_ring_family;

/// Result of assessing a single molecule against the evidence framework.
#[derive(Clone, Debug)]
pub struct EvidenceAssessment {
    /// Ertl NP-likeness score (range ≈ −5 to +5), or 0.0 when the model
    /// is not available.
    pub np_likeness: f64,
    /// Human-readable classification label derived from the score.
    pub np_label: String,
    /// Confidence in the score (fraction of Morgan bits found in the model).
    pub np_confidence: f64,
    /// Scaffold-family classification.
    pub ring_family: String,
    /// Evidence notes, each citing the underlying observation.
    pub evidence_notes: Vec<String>,
    /// One-line summary of the dataset-level motif context.
    pub motif_context: String,
}

/// Classification thresholds for the Ertl NP-likeness score.
///
/// Empirically grounded thresholds:
/// - **≥ 2.0**: Good NP evidence (high confidence natural product)
/// - **0.5–2.0**: Ambiguous NP signals (could be synthetic or semi-synthetic)
/// - **-1.0–0.5**: Bad/weak signals (predominantly synthetic features)
/// - **< -1.0**: Highly synthetic (strong negative signals—rare in real NPs)
pub fn np_likeness_label(score: f64) -> &'static str {
    if score >= 2.0 {
        "strong natural product"
    } else if score >= 0.5 {
        "NP-ambiguous"
    } else if score >= -1.0 {
        "weak NP signals"
    } else {
        "highly synthetic"
    }
}

/// Inputs bundled for [`assess_np_evidence`]. Passed by value (the struct is
/// `Copy` because every field is a shared reference or an `Option<f64>`) so the
/// call site stays free of lifetime annotation while the caller keeps ownership
/// of the descriptor/motif/slice data.
#[derive(Clone, Copy)]
pub struct EvidenceInputs<'a> {
    /// rdkit-derived whole-molecule descriptors.
    pub descriptors: &'a RdkitDescriptors,
    /// Ertl + user motif labels found in the molecule.
    pub motifs: &'a [String],
    /// Matched motif hits with source/kingdom metadata.
    pub motif_hits: &'a [RdkitMotifHit],
    /// Stereochemical centre tags detected in the 2D representation.
    pub stereo_tags: &'a [String],
    /// Ertl NP-likeness score (`None` when the model is unavailable).
    pub np_score: Option<f64>,
    /// Model confidence (fraction of Morgan bits found).
    pub np_confidence: Option<f64>,
    /// Dataset-level motif prevalence for common-scaffold flagging.
    pub dataset_context: &'a DatasetMotifContext,
    /// LOTUS 1%-scaffold matches for the molecule.
    pub lotus_scaffolds: &'a [String],
}

// Structural evidence counts now live in `chemist::EvidenceCounts` (cfg-free
// leaf module), shared by `assess_np_evidence` and the verdict classifier.

/// Assess NP evidence for a single molecule.
///
/// * `inputs.np_score` / `inputs.np_confidence` come from the rdkit.js bridge
///   (real Ertl model).  When `None`, the score cannot be computed and
///   structural observations are the only available evidence.
/// * `inputs.motifs` is the list of Ertl (2022) top-2000 NP substituent labels
///   found in the molecule via substructure matching.
/// * `inputs.dataset_context` carries motif prevalence across the entire
///   uploaded set so that per-row notes can flag dataset-common scaffolds.
pub fn assess_np_evidence(inputs: EvidenceInputs<'_>) -> EvidenceAssessment {
    let ring_family = classify_ring_family(inputs.descriptors, inputs.motifs);
    let counts = count_evidence(inputs.motifs, inputs.motif_hits);
    let natural_only_hits = counts
        .natural_hits
        .saturating_sub(counts.kingdom_enriched_hits);
    let kingdom_support = counts.kingdom_enriched_hits > 0;
    let score = inputs.np_score.unwrap_or(0.0);
    let confidence = inputs.np_confidence.unwrap_or(0.0);
    let conf_pct = confidence * 100.0;

    let mut notes = primary_evidence_notes(
        inputs.np_score,
        conf_pct,
        inputs.lotus_scaffolds,
        inputs.stereo_tags,
    );
    notes.extend(motif_evidence_notes(
        counts,
        natural_only_hits,
        inputs.motifs,
        inputs.dataset_context,
    ));

    EvidenceAssessment {
        np_likeness: score,
        np_label: np_likeness_label(score).to_string(),
        np_confidence: confidence,
        ring_family,
        evidence_notes: notes,
        motif_context: motif_context_label(counts, natural_only_hits, kingdom_support),
    }
}

/// Primary evidence: the Ertl NP-likeness score, LOTUS scaffold hits, and
/// stereochemical complexity.
fn primary_evidence_notes(
    np_score: Option<f64>,
    conf_pct: f64,
    lotus_scaffolds: &[String],
    stereo_tags: &[String],
) -> Vec<String> {
    let mut notes = Vec::new();
    if let Some(s) = np_score {
        notes.push(format!(
            "Ertl NP-likeness score: {s:+.2} (model confidence {conf_pct:.0}%) — \
             Ertl et al., J. Chem. Inf. Model. 2008, 48, 68"
        ));
        if conf_pct < 50.0 {
            notes.push(format!(
                "Low model coverage ({conf_pct:.0}%) — Ertl score is \
                 unreliable for unusual fragments"
            ));
        }
    } else {
        notes.push("Ertl fragment model not loaded — NP-likeness score unavailable".into());
    }
    if !lotus_scaffolds.is_empty() {
        let matched = lotus_scaffolds.len();
        notes.push(format!(
            "✓ {matched} LOTUS 1% scaffold(s) matched — Rutz et al. mortar \
             fragmentation; scaffolds appearing in >1% of LOTUS compounds"
        ));
    }
    if !stereo_tags.is_empty() {
        let centers = stereo_tags.len();
        notes.push(format!(
            "✓ {centers} stereochemical center(s) — consistent with enzymatic origin"
        ));
    }
    notes
}

/// Secondary evidence: Ertl motif counts, motif balance, decoration notes and
/// dataset-common scaffold observations.
///
/// The `usize -> f64` ratio casts in the motif-balance block are unavoidable
/// (`f64` has no `From<usize>` impl, and motif counts are far below 2^53 so the
/// cast is lossless in practice); `cast_precision_loss` is kept here with a
/// documented reason rather than silently suppressed.
#[allow(clippy::cast_precision_loss)] // ratio math: usize -> f64 (no lossless `From`)
fn motif_evidence_notes(
    counts: EvidenceCounts,
    natural_only_hits: usize,
    motifs: &[String],
    dataset_context: &DatasetMotifContext,
) -> Vec<String> {
    let EvidenceCounts {
        np_core_hits,
        scaffold_hits,
        decoration_hits,
        kingdom_enriched_hits,
        synthetic_hits,
        unknown_hits,
        .. // natural_hits: consumed by `motif_context_label` via the shared `counts`.
    } = counts;
    let mut notes = Vec::new();
    if np_core_hits > 0 {
        let shown_motifs = motifs
            .iter()
            .filter(|m| is_known_np_motif(m))
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        notes.push(format!(
            "✓ {np_core_hits} Ertl-style NP motif(s): {shown_motifs}"
        ));
    }
    if scaffold_hits > 0 && np_core_hits == 0 {
        notes.push(format!(
            "○ {scaffold_hits} structural support motif(s) — \
             supportive only, not Ertl-enriched by itself"
        ));
    }
    if kingdom_enriched_hits > 0 || natural_only_hits > 0 || synthetic_hits > 0 || unknown_hits > 0
    {
        let total = kingdom_enriched_hits + natural_only_hits + synthetic_hits + unknown_hits;
        let natural_ratio = if total > 0 {
            (kingdom_enriched_hits + natural_only_hits) as f64 / total as f64
        } else {
            0.0
        };
        let synthetic_ratio = if total > 0 {
            synthetic_hits as f64 / total as f64
        } else {
            0.0
        };
        notes.push(format!(
            "motif balance: {kingdom_enriched_hits} kingdom-enriched / {natural_only_hits} \
             natural-only / {synthetic_hits} synthetic / {unknown_hits} unclassified \
             ({natural_ratio:.0}% natural, {synthetic_ratio:.0}% synthetic)"
        ));
    }
    if decoration_hits > 0 && np_core_hits == 0 {
        notes.push(format!(
            "○ {decoration_hits} decoration motif(s) — support side-chain chemistry, \
             but weak NP evidence alone"
        ));
    }
    let dataset_shared = motifs
        .iter()
        .filter(|m| {
            dataset_context
                .motif_counts
                .get(*m)
                .is_some_and(|&c| c >= dataset_context.common_threshold)
        })
        .count();
    if dataset_shared > 0 {
        notes.push(format!(
            "✓ {dataset_shared} motif(s) shared with dataset — possible family"
        ));
    }
    notes
}

/// One-line summary of the dataset-level structural-motif picture, mirroring
/// the Ertl structural-support hierarchy.
fn motif_context_label(
    counts: EvidenceCounts,
    natural_only_hits: usize,
    kingdom_support: bool,
) -> String {
    if counts.np_core_hits > 0 && counts.scaffold_hits > 0 {
        "Ertl-style NP core with structural support".to_string()
    } else if counts.np_core_hits > 0 {
        "Ertl-style NP core motifs".to_string()
    } else if counts.scaffold_hits > 0 && kingdom_support {
        "kingdom-enriched structural motifs".to_string()
    } else if counts.scaffold_hits > 0 && natural_only_hits > 0 {
        "natural-only structural motifs".to_string()
    } else if counts.scaffold_hits > 0 {
        "structural support motifs".to_string()
    } else if kingdom_support && counts.kingdom_enriched_hits > counts.synthetic_hits {
        "kingdom-enriched motifs".to_string()
    } else if natural_only_hits > counts.synthetic_hits && natural_only_hits > 0 {
        "natural-leaning motifs".to_string()
    } else if counts.synthetic_hits > counts.natural_hits && counts.synthetic_hits > 0 {
        "synthetic-leaning motifs".to_string()
    } else if counts.decoration_hits > 0 {
        "decoration motifs only".to_string()
    } else {
        "no structural motifs detected".to_string()
    }
}
