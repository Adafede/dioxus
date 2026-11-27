// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! Per-molecule evidence note synthesis combining Ertl NP-likeness, scaffolds, and motifs.

use crate::model::{DatasetMotifContext, MotifContext, NpScore, RdkitDescriptors, RingFamily};

use super::chemist::{EvidenceCounts, is_known_np_motif};
use super::ring_family::classify_ring_family;
use super::{ERTL_AMBIGUOUS, ERTL_FISHY, ERTL_MODERATE, ERTL_STRONG};

/// Assessment result built for a single molecule row.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EvidenceAssessment {
    /// Human-readable classification label derived from the score (0.0 fallback when unloaded).
    pub(crate) np_label: String,
    /// Scaffold ring-family classification.
    pub(crate) ring_family: RingFamily,
    /// Formatted evidence bullet notes.
    pub(crate) evidence_notes: Vec<String>,
    /// Typed summary of structural motif balance.
    pub(crate) motif_context: MotifContext,
}

/// Empirical classification labels for an optional finite Ertl NP-likeness score.
///
/// Thresholds match [`super::chemist::run_checks`] and [`super::verdict::classify_np_evidence`]:
/// `>= 2.0` strong NP, `>= 1.0` NP-like, `>= 0.0` ambiguous, `> -2.0` synthetic-leaning,
/// `<= -2.0` highly synthetic; `None` or non-finite returns `"unavailable"`.
#[must_use]
pub(crate) const fn np_likeness_label(score: Option<f64>) -> &'static str {
    let Some(s) = score else {
        return "unavailable";
    };
    if !s.is_finite() {
        "unavailable"
    } else if s >= ERTL_STRONG {
        "strong natural product"
    } else if s >= ERTL_MODERATE {
        "NP-like"
    } else if s >= ERTL_AMBIGUOUS {
        "NP-ambiguous"
    } else if s > ERTL_FISHY {
        "synthetic-leaning"
    } else {
        "highly synthetic"
    }
}

/// Inputs required to assess NP evidence for one molecule.
#[derive(Clone, Copy, Debug)]
pub(crate) struct EvidenceInputs<'a> {
    /// Whole-molecule `RDKit` descriptors.
    pub(crate) descriptors: &'a RdkitDescriptors,
    /// Matched motif hits.
    pub(crate) motif_hits: &'a [crate::model::RdkitMotifHit],
    /// Stereocenter tags detected in the 2D depiction.
    pub(crate) stereo_tags: &'a [String],
    /// Loaded Ertl score and confidence (`None` if unloaded).
    pub(crate) np_score: Option<NpScore>,
    /// Dataset-level motif prevalence context.
    pub(crate) dataset_context: &'a DatasetMotifContext,
    /// LOTUS 1%-prevalent scaffold SMILES hits.
    pub(crate) lotus_scaffolds: &'a [String],
}

/// Synthesizes evidence assessment for a molecule, computing motif counts directly.
#[cfg(test)]
#[must_use]
pub(crate) fn assess_np_evidence(inputs: &EvidenceInputs<'_>) -> EvidenceAssessment {
    let counts = super::chemist::count_evidence(inputs.motif_hits);
    assess_np_evidence_with_counts(inputs, counts)
}

/// Synthesizes evidence assessment reusing precomputed [`EvidenceCounts`].
#[must_use]
pub(crate) fn assess_np_evidence_with_counts(
    inputs: &EvidenceInputs<'_>,
    counts: EvidenceCounts,
) -> EvidenceAssessment {
    let ring_family = classify_ring_family(inputs.descriptors, inputs.motif_hits);
    let natural_only = counts.natural.saturating_sub(counts.kingdom_enriched);
    let kingdom_support = counts.kingdom_enriched > 0;
    let finite_score = inputs.np_score.map(|np| np.score).filter(|s| s.is_finite());
    let conf_pct = inputs
        .np_score
        .map(|np| np.confidence * 100.0)
        .filter(|c| c.is_finite())
        .unwrap_or(0.0);

    let mut notes = primary_evidence_notes(
        finite_score,
        conf_pct,
        inputs.lotus_scaffolds,
        inputs.stereo_tags,
    );
    notes.extend(motif_evidence_notes(
        counts,
        natural_only,
        inputs.motif_hits,
        inputs.dataset_context,
    ));

    EvidenceAssessment {
        np_label: np_likeness_label(finite_score).to_string(),
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
            "{} LOTUS 1% scaffold(s) matched — Rutz et al. mortar \
             fragmentation; scaffolds appearing in >1% of LOTUS compounds",
            lotus_scaffolds.len()
        ));
    }
    if !stereo_tags.is_empty() {
        notes.push(format!(
            "{} stereochemical center(s) — consistent with enzymatic origin",
            stereo_tags.len()
        ));
    }
    notes
}

fn motif_evidence_notes(
    counts: EvidenceCounts,
    natural_only: usize,
    motif_hits: &[crate::model::RdkitMotifHit],
    dataset_context: &DatasetMotifContext,
) -> Vec<String> {
    let mut notes = Vec::new();
    if counts.np_core > 0 {
        let shown = motif_hits
            .iter()
            .map(|h| h.label.as_str())
            .filter(|label| is_known_np_motif(label))
            .take(3)
            .collect::<Vec<_>>()
            .join(", ");
        notes.push(format!(
            "{} Ertl-style NP motif(s): {shown}",
            counts.np_core
        ));
    }
    if counts.scaffold > 0 && counts.np_core == 0 {
        notes.push(format!(
            "{} structural support motif(s) — \
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
            "{} decoration motif(s) — support side-chain chemistry, \
             but weak NP evidence alone",
            counts.decoration
        ));
    }
    let shared = motif_hits
        .iter()
        .filter(|h| {
            dataset_context
                .motif_counts
                .get(&h.label)
                .is_some_and(|&c| c >= dataset_context.common_threshold)
        })
        .count();
    if shared > 0 {
        notes.push(format!(
            "{shared} motif(s) shared with dataset — possible family"
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
    } else if counts.scaffold > 0 && kingdom_support && counts.natural >= counts.synthetic {
        MotifContext::KingdomEnrichedStructural
    } else if counts.scaffold > 0 && natural_only > 0 && counts.natural >= counts.synthetic {
        MotifContext::NaturalStructural
    } else if counts.scaffold > 0 {
        MotifContext::StructuralSupport
    } else if counts.synthetic > counts.natural {
        MotifContext::SyntheticLeaning
    } else if kingdom_support && counts.natural > counts.synthetic {
        MotifContext::KingdomEnriched
    } else if counts.natural > counts.synthetic {
        MotifContext::NaturalLeaning
    } else if counts.decoration > 0 {
        MotifContext::DecorationOnly
    } else {
        MotifContext::NoneDetected
    }
}
