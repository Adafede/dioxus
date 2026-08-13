// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Evidence assessment: builds an [`EvidenceAssessment`] for a single
//! molecule from the Ertl NP-likeness score plus structural motif context.
//! Depends on `verdict` (ring-family classification) and `chemist` (motif
//! counting), so it sits at the top of the evidence dependency graph.

use crate::model::{DatasetMotifContext, RdkitDescriptors, RdkitMotifHit};

use super::chemist::{
    count_core_np_motifs, count_decoration_motifs, count_kingdom_enriched_hits,
    count_scaffold_hits, count_source_hits, is_known_np_motif,
};
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

/// Assess NP evidence for a single molecule.
///
/// * `np_score` / `np_confidence` come from the rdkit.js bridge (real Ertl
///   model).  When `None`, the score cannot be computed and structural
///   observations are the only available evidence.
/// * `substituents` is the list of Ertl (2022) top-2000 NP substituent labels
///   found in the molecule via substructure matching.
/// * `dataset_context` carries motif prevalence across the entire uploaded
///   set so that per-row notes can flag dataset-common scaffolds.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
#[allow(clippy::cast_precision_loss)]
#[allow(clippy::uninlined_format_args)]
pub fn assess_np_evidence(
    descriptors: &RdkitDescriptors,
    motifs: &[String],
    motif_hits: &[RdkitMotifHit],
    stereo_tags: &[String],
    np_score: Option<f64>,
    np_confidence: Option<f64>,
    dataset_context: &DatasetMotifContext,
    lotus_scaffolds: &[String],
) -> EvidenceAssessment {
    let ring_family = classify_ring_family(descriptors, motifs);
    let mut notes = Vec::new();
    let np_core_hits = count_core_np_motifs(motifs);
    let scaffold_hits = count_scaffold_hits(motif_hits);
    let decoration_hits = count_decoration_motifs(motifs);
    let natural_hits = count_source_hits(motif_hits, "natural");
    let synthetic_hits = count_source_hits(motif_hits, "synthetic");
    let unknown_hits = count_source_hits(motif_hits, "unknown");
    let kingdom_enriched_hits = count_kingdom_enriched_hits(motif_hits);
    let natural_only_hits = natural_hits.saturating_sub(kingdom_enriched_hits);
    let kingdom_support = kingdom_enriched_hits > 0;
    let score = np_score.unwrap_or(0.0);
    let confidence = np_confidence.unwrap_or(0.0);

    // ── Primary evidence: real Ertl NP-likeness score ──────────────────
    if let Some(s) = np_score {
        notes.push(format!(
            "Ertl NP-likeness score: {s:+.2} (model confidence {:.0}%) — \
             Ertl et al., J. Chem. Inf. Model. 2008, 48, 68",
            confidence * 100.0
        ));
        if confidence < 0.5 {
            notes.push(format!(
                "Low model coverage ({:.0}%) — Ertl score is \
                 unreliable for unusual fragments",
                confidence * 100.0
            ));
        }
    } else {
        notes.push("Ertl fragment model not loaded — NP-likeness score unavailable".into());
    }

    // ── LOTUS 1-percent scaffold evidence (Rutz et al. mortar fragmentation) ──
    if !lotus_scaffolds.is_empty() {
        notes.push(format!(
            "✓ {} LOTUS 1% scaffold(s) matched — Rutz et al. mortar \
             fragmentation; scaffolds appearing in >1% of LOTUS compounds",
            lotus_scaffolds.len()
        ));
    }

    // Stereochemical complexity — stereocentres are hallmarks of enzymatic
    // biosynthesis; synthetic libraries average <1 stereocentre per molecule.
    // ── Stereochemistry signal ─────────────────────────────────────────
    // Stereochemistry is enriched in natural products due to enzymatic
    // biosynthesis. Presence is a positive signal; absence in 2D SMILES is
    // neutral (MS annotation pipelines often strip stereo).
    if !stereo_tags.is_empty() {
        notes.push(format!(
            "✓ {} stereochemical center(s) — consistent with enzymatic origin",
            stereo_tags.len()
        ));
    }

    // ── NP motif presence ─────────────────────────────────────────────
    if np_core_hits > 0 {
        notes.push(format!(
            "✓ {} Ertl-style NP motif(s): {}",
            np_core_hits,
            motifs
                .iter()
                .filter(|m| is_known_np_motif(m))
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    if scaffold_hits > 0 && np_core_hits == 0 {
        notes.push(format!(
            "○ {scaffold_hits} structural support motif(s) — supportive only, not Ertl-enriched by itself",
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
            "motif balance: {kingdom_enriched_hits} kingdom-enriched / {natural_only_hits} natural-only / {synthetic_hits} synthetic / {unknown_hits} unclassified ({natural_ratio:.0}% natural, {synthetic_ratio:.0}% synthetic)"
        ));
    }

    if decoration_hits > 0 && np_core_hits == 0 {
        notes.push(format!(
            "○ {decoration_hits} decoration motif(s) — supportive side-chain chemistry, but weak NP evidence alone",
        ));
    }

    // ── Dataset-level context ──────────────────────────────────────────
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

    // ── Motif context summary ──────────────────────────────────────────
    let motif_context = if np_core_hits > 0 && scaffold_hits > 0 {
        "Ertl-style NP core with structural support".to_string()
    } else if np_core_hits > 0 {
        "Ertl-style NP core motifs".to_string()
    } else if scaffold_hits > 0 && kingdom_support {
        "kingdom-enriched structural motifs".to_string()
    } else if scaffold_hits > 0 && natural_only_hits > 0 {
        "natural-only structural motifs".to_string()
    } else if scaffold_hits > 0 {
        "structural support motifs".to_string()
    } else if kingdom_support && kingdom_enriched_hits > synthetic_hits {
        "kingdom-enriched motifs".to_string()
    } else if natural_only_hits > synthetic_hits && natural_only_hits > 0 {
        "natural-leaning motifs".to_string()
    } else if synthetic_hits > natural_hits && synthetic_hits > 0 {
        "synthetic-leaning motifs".to_string()
    } else if decoration_hits > 0 {
        "decoration motifs only".to_string()
    } else {
        "no structural motifs detected".to_string()
    };

    let np_label = np_likeness_label(score).to_string();

    EvidenceAssessment {
        np_likeness: score,
        np_label,
        np_confidence: confidence,
        ring_family,
        evidence_notes: notes,
        motif_context,
    }
}
