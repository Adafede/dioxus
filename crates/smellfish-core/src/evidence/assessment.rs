// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! Per-molecule evidence note synthesis combining Ertl NP-likeness, scaffolds, and motifs.

use crate::model::{
    DatasetMotifContext, MotifContext, NpScore, RdkitDescriptors, RdkitMotifHit, RingFamily,
};

use super::chemist::{EvidenceCounts, count_evidence, is_known_np_motif};
use super::ring_family::classify_ring_family;

/// Assessment result built for a single molecule row.
#[derive(Clone, Debug, PartialEq)]
pub struct EvidenceAssessment {
    /// Scored Ertl NP-likeness + confidence (`None` when model unloaded).
    pub np_score: Option<NpScore>,
    /// Human-readable classification label derived from the score (0.0 fallback when unloaded).
    pub np_label: String,
    /// Scaffold ring-family classification.
    pub ring_family: RingFamily,
    /// Formatted evidence bullet notes.
    pub evidence_notes: Vec<String>,
    /// Typed summary of structural motif balance.
    pub motif_context: MotifContext,
}

/// Empirical classification labels for the Ertl NP-likeness score.
#[must_use]
pub const fn np_likeness_label(score: f64) -> &'static str {
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

/// Inputs required to assess NP evidence for one molecule.
#[derive(Clone, Copy, Debug)]
pub struct EvidenceInputs<'a> {
    /// Whole-molecule `RDKit` descriptors.
    pub descriptors: &'a RdkitDescriptors,
    /// Matched motif labels.
    pub motifs: &'a [String],
    /// Matched motif hits with source/kingdom attribution.
    pub motif_hits: &'a [RdkitMotifHit],
    /// Stereocenter tags detected in the 2D depiction.
    pub stereo_tags: &'a [String],
    /// Loaded Ertl score and confidence (`None` if unloaded).
    pub np_score: Option<NpScore>,
    /// Dataset-level motif prevalence context.
    pub dataset_context: &'a DatasetMotifContext,
    /// LOTUS 1%-prevalent scaffold SMILES hits.
    pub lotus_scaffolds: &'a [String],
}

/// Synthesizes evidence assessment for a molecule, computing motif counts directly.
#[must_use]
pub fn assess_np_evidence(inputs: &EvidenceInputs<'_>) -> EvidenceAssessment {
    let counts = count_evidence(inputs.motifs, inputs.motif_hits);
    assess_np_evidence_with_counts(inputs, counts)
}

/// Synthesizes evidence assessment reusing precomputed [`EvidenceCounts`].
#[must_use]
pub fn assess_np_evidence_with_counts(
    inputs: &EvidenceInputs<'_>,
    counts: EvidenceCounts,
) -> EvidenceAssessment {
    let ring_family = classify_ring_family(inputs.descriptors, inputs.motifs);
    let natural_only = counts.natural.saturating_sub(counts.kingdom_enriched);
    let kingdom_support = counts.kingdom_enriched > 0;
    let raw_score = inputs.np_score.map_or(0.0, |np| np.score);
    let conf_pct = inputs.np_score.map_or(0.0, |np| np.confidence * 100.0);

    let mut notes = primary_evidence_notes(
        inputs.np_score.map(|np| np.score),
        conf_pct,
        inputs.lotus_scaffolds,
        inputs.stereo_tags,
    );
    notes.extend(motif_evidence_notes(
        counts,
        natural_only,
        inputs.motifs,
        inputs.dataset_context,
    ));

    EvidenceAssessment {
        np_score: inputs.np_score,
        np_label: np_likeness_label(raw_score).to_string(),
        ring_family,
        evidence_notes: notes,
        motif_context: classify_motif_context(counts, natural_only, kingdom_support),
    }
}

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
        notes.push(format!(
            "✓ {} LOTUS 1% scaffold(s) matched — Rutz et al. mortar \
             fragmentation; scaffolds appearing in >1% of LOTUS compounds",
            lotus_scaffolds.len()
        ));
    }
    if !stereo_tags.is_empty() {
        notes.push(format!(
            "✓ {} stereochemical center(s) — consistent with enzymatic origin",
            stereo_tags.len()
        ));
    }
    notes
}

fn motif_evidence_notes(
    counts: EvidenceCounts,
    natural_only: usize,
    motifs: &[String],
    dataset_context: &DatasetMotifContext,
) -> Vec<String> {
    let mut notes = Vec::new();
    if counts.np_core > 0 {
        let shown = motifs
            .iter()
            .filter(|m| is_known_np_motif(m))
            .take(3)
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(", ");
        notes.push(format!(
            "✓ {} Ertl-style NP motif(s): {shown}",
            counts.np_core
        ));
    }
    if counts.scaffold > 0 && counts.np_core == 0 {
        notes.push(format!(
            "○ {} structural support motif(s) — \
             supportive only, not Ertl-enriched by itself",
            counts.scaffold
        ));
    }
    let total = counts.kingdom_enriched + natural_only + counts.synthetic + counts.unknown;
    if total > 0 {
        let natural_pct = pct(counts.kingdom_enriched + natural_only, total);
        let synthetic_pct = pct(counts.synthetic, total);
        notes.push(format!(
            "motif balance: {} kingdom-enriched / {} natural-only / {} synthetic / {} \
             unclassified ({natural_pct:.0}% natural, {synthetic_pct:.0}% synthetic)",
            counts.kingdom_enriched, natural_only, counts.synthetic, counts.unknown,
        ));
    }
    if counts.decoration > 0 && counts.np_core == 0 {
        notes.push(format!(
            "○ {} decoration motif(s) — support side-chain chemistry, \
             but weak NP evidence alone",
            counts.decoration
        ));
    }
    let shared = motifs
        .iter()
        .filter(|m| {
            dataset_context
                .motif_counts
                .get(*m)
                .is_some_and(|&c| c >= dataset_context.common_threshold)
        })
        .count();
    if shared > 0 {
        notes.push(format!(
            "✓ {shared} motif(s) shared with dataset — possible family"
        ));
    }
    notes
}

fn pct(part: usize, total: usize) -> f64 {
    let part_f = f64::from(u32::try_from(part).unwrap_or(u32::MAX));
    let total_f = f64::from(u32::try_from(total.max(1)).unwrap_or(u32::MAX));
    part_f * 100.0 / total_f
}

const fn classify_motif_context(
    counts: EvidenceCounts,
    natural_only: usize,
    kingdom_support: bool,
) -> MotifContext {
    if counts.np_core > 0 && counts.scaffold > 0 {
        MotifContext::ErtlCoreWithSupport
    } else if counts.np_core > 0 {
        MotifContext::ErtlCore
    } else if counts.scaffold > 0 && kingdom_support {
        MotifContext::KingdomEnrichedStructural
    } else if counts.scaffold > 0 && natural_only > 0 {
        MotifContext::NaturalStructural
    } else if counts.scaffold > 0 {
        MotifContext::StructuralSupport
    } else if kingdom_support && counts.kingdom_enriched > counts.synthetic {
        MotifContext::KingdomEnriched
    } else if natural_only > counts.synthetic && natural_only > 0 {
        MotifContext::NaturalLeaning
    } else if counts.synthetic > counts.natural && counts.synthetic > 0 {
        MotifContext::SyntheticLeaning
    } else if counts.decoration > 0 {
        MotifContext::DecorationOnly
    } else {
        MotifContext::NoneDetected
    }
}
