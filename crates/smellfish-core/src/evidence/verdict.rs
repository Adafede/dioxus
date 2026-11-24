// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! Typed verdict derivation combining Ertl NP-likeness, database hits, and motif evidence.

use crate::model::{MoleculeRow, Verdict, VerdictCategory};

use super::chemist::{EvidenceCounts, count_evidence};

/// Flat evidence inputs consumed by [`classify_np_evidence`].
#[derive(Clone, Copy, Debug, Default)]
pub struct EvidenceSignals {
    /// Ertl NP-likeness score (`None` when unloaded).
    pub np_score: Option<f64>,
    /// Molecule itself is backed by a LOTUS organism record.
    pub has_lotus: bool,
    /// Molecule is backed by `PubChem` records.
    pub has_pubchem: bool,
    /// Number of LOTUS 1%-prevalence scaffold matches.
    pub lotus_scaffolds: usize,
    /// Structural motif counts shared with assessment.
    pub counts: EvidenceCounts,
}

impl EvidenceSignals {
    const fn natural_dominant(self) -> bool {
        self.counts.natural > 0 && self.counts.natural >= self.counts.synthetic
    }

    const fn synthetic_majority(self) -> bool {
        self.counts.synthetic > 0 && self.counts.synthetic > self.counts.natural
    }

    const fn structural_support(self) -> bool {
        self.counts.scaffold > 0 && self.counts.np_core > 0 && self.natural_dominant()
    }

    const fn decoration_heavy(self) -> bool {
        self.counts.decoration > self.counts.scaffold
    }
}

/// Derives the screening verdict for a molecule row, computing motif counts directly.
#[must_use]
pub fn row_verdict(row: &MoleculeRow) -> Verdict {
    let counts = count_evidence(&row.motifs, &row.motif_hits);
    row_verdict_with_counts(row, counts)
}

/// Derives the screening verdict for a molecule row reusing precomputed [`EvidenceCounts`].
#[must_use]
pub fn row_verdict_with_counts(row: &MoleculeRow, counts: EvidenceCounts) -> Verdict {
    if let Some(err) = row.error.as_deref() {
        return Verdict::error(err);
    }

    classify_np_evidence(&EvidenceSignals {
        np_score: row.np_score.map(|np| np.score),
        has_lotus: !row.lotus_taxa.is_empty(),
        has_pubchem: !row.pubchem_cids.is_empty(),
        lotus_scaffolds: row.lotus_scaffolds.len(),
        counts,
    })
}

/// Pure evidence classifier returning both a [`VerdictCategory`] and explanation message.
#[must_use]
pub fn classify_np_evidence(signals: &EvidenceSignals) -> Verdict {
    let Some(score) = signals.np_score else {
        return classify_without_model(*signals);
    };
    if score <= -2.0 {
        return Verdict::new(
            VerdictCategory::Fishy,
            format!("👃 Smells fishy — highly synthetic (Ertl {score:+.2})"),
        );
    }
    let synthetic_lean = signals.synthetic_majority() || signals.decoration_heavy() || score < 0.0;
    if synthetic_lean && !signals.has_lotus {
        return Verdict::new(
            VerdictCategory::Caution,
            format!("🟧 Synthetic-leaning structure (Ertl {score:+.2})"),
        );
    }
    if signals.has_lotus {
        return classify_with_lotus(*signals, score);
    }
    if signals.lotus_scaffolds > 0 {
        return classify_with_lotus_scaffold(*signals, score);
    }
    if signals.has_pubchem {
        return classify_with_pubchem(*signals, score);
    }
    classify_unbacked(*signals, score)
}

fn classify_without_model(signals: EvidenceSignals) -> Verdict {
    if signals.lotus_scaffolds > 0 && !signals.synthetic_majority() && signals.structural_support()
    {
        return Verdict::new(
            VerdictCategory::Likely,
            "🌿 LOTUS-prevalent scaffold + NP structure — supporting evidence (Ertl unavailable)",
        );
    }
    if signals.has_lotus && signals.structural_support() {
        return Verdict::new(
            VerdictCategory::Likely,
            "🌿 LOTUS + NP structure (Ertl unavailable)",
        );
    }
    if signals.has_lotus {
        return Verdict::new(
            VerdictCategory::Likely,
            "🌿 LOTUS organism record (Ertl unavailable)",
        );
    }
    Verdict::new(
        VerdictCategory::Skeptical,
        "⚠ Citation needed — no Ertl model and no structural evidence",
    )
}

fn classify_with_lotus(signals: EvidenceSignals, score: f64) -> Verdict {
    if score >= 2.0 && signals.structural_support() {
        Verdict::new(
            VerdictCategory::Likely,
            format!("🌿 LOTUS + strong NP evidence (Ertl {score:+.2})"),
        )
    } else if score >= 1.0 && signals.structural_support() {
        Verdict::new(
            VerdictCategory::Likely,
            format!("🌿 LOTUS-backed NP evidence (Ertl {score:+.2})"),
        )
    } else {
        Verdict::new(
            VerdictCategory::Likely,
            format!("🌿 LOTUS organism record (Ertl {score:+.2})"),
        )
    }
}

fn classify_with_lotus_scaffold(signals: EvidenceSignals, score: f64) -> Verdict {
    let corroborates = signals.counts.natural > 0
        || signals.counts.kingdom_enriched > 0
        || signals.counts.scaffold > 0
        || signals.counts.np_core > 0;

    if score >= 2.0 && corroborates {
        Verdict::new(
            VerdictCategory::Likely,
            format!(
                "🌿 Likely NP — LOTUS-prevalent scaffold + strong Ertl + natural motifs (Ertl {score:+.2})"
            ),
        )
    } else if score >= 2.0 {
        Verdict::new(
            VerdictCategory::Skeptical,
            format!(
                "👃 Citation needed — LOTUS scaffold hint, no natural corroboration (Ertl {score:+.2})"
            ),
        )
    } else if score >= 1.0 && corroborates {
        Verdict::new(
            VerdictCategory::Likely,
            format!(
                "🌿 Likely NP — LOTUS-prevalent scaffold + Ertl + natural motifs (Ertl {score:+.2})"
            ),
        )
    } else if score >= 1.0 {
        Verdict::new(
            VerdictCategory::Skeptical,
            format!("👃 Citation needed — LOTUS scaffold hint, Ertl-only (Ertl {score:+.2})"),
        )
    } else {
        Verdict::new(
            VerdictCategory::Skeptical,
            format!(
                "👃 Citation needed — LOTUS scaffold hint, insufficient corroboration (Ertl {score:+.2})"
            ),
        )
    }
}

fn classify_with_pubchem(signals: EvidenceSignals, score: f64) -> Verdict {
    if score >= 2.0 && signals.structural_support() {
        Verdict::new(
            VerdictCategory::Likely,
            format!("🌿 PubChem + strong NP evidence (Ertl {score:+.2})"),
        )
    } else if score >= 2.0 {
        Verdict::new(
            VerdictCategory::Skeptical,
            format!("👃 Citation needed — PubChem hit, unsupported structure (Ertl {score:+.2})"),
        )
    } else if score >= 1.0 && signals.structural_support() {
        Verdict::new(
            VerdictCategory::Skeptical,
            format!("👃 Citation needed — PubChem + strong NP-likeness (Ertl {score:+.2})"),
        )
    } else if score >= 1.0 {
        Verdict::new(
            VerdictCategory::Neutral,
            format!("📚 PubChem hit — NP-ambiguous (Ertl {score:+.2})"),
        )
    } else {
        Verdict::new(
            VerdictCategory::Caution,
            format!("📚 PubChem hit — weak NP signals (Ertl {score:+.2})"),
        )
    }
}

fn classify_unbacked(signals: EvidenceSignals, score: f64) -> Verdict {
    let strong = score >= 2.0;
    let confident = score >= 1.0;

    if strong
        && signals.counts.np_core >= 2
        && signals.counts.scaffold > 0
        && signals.natural_dominant()
        && signals.counts.kingdom_enriched > 0
        && !signals.decoration_heavy()
    {
        return Verdict::new(
            VerdictCategory::Likely,
            format!("🌿 Likely novel NP (Ertl {score:+.2})"),
        );
    }

    let message = if strong && signals.structural_support() {
        format!("👃 Citation needed — strong NP-likeness, no DB (Ertl {score:+.2})")
    } else if strong {
        format!("👃 Citation needed — strong Ertl but unsupported structure (Ertl {score:+.2})")
    } else if confident && signals.structural_support() {
        format!("👃 Citation needed — moderate NP-likeness, no DB (Ertl {score:+.2})")
    } else if score >= 0.5 {
        format!("👃 Citation needed — borderline NP-likeness (Ertl {score:+.2})")
    } else {
        format!("👃 Citation needed (Ertl {score:+.2})")
    };
    Verdict::new(VerdictCategory::Skeptical, message)
}
