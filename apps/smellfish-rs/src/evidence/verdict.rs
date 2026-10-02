// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! Verdict derivation: the human-facing one-liner verdict per row.
//!
//! Depends on `chemist` for motif counting (via `EvidenceCounts`) and
//! `ring_family` for structural classification.  The CSV categorization
//! helper (`category`) lives in `app::csv_export`.

use super::chemist::EvidenceCounts;
#[cfg(target_arch = "wasm32")]
use super::chemist::count_evidence;

/// Verdict string shown prominently in the UI.
///
/// Delegates the chemistry/structure classification to the native, unit-tested
/// [`classify_np_evidence`]; this wasm-only wrapper only extracts the flat
/// evidence signals from the RDKit-built [`MoleculeRow`](crate::model::MoleculeRow).
/// Keeping the threshold logic in a `cfg`-free pure function means the
/// assessment is chemistry-grounded and verifiable on native — it is not
/// derived from any LLM heuristic, and a score-only signal is never elevated
/// to "novel".
#[cfg(target_arch = "wasm32")]
#[must_use]
pub(crate) fn row_verdict(row: &crate::model::MoleculeRow) -> String {
    if let Some(err) = row.error.as_deref() {
        return format!("⚠ {err}");
    }

    let has_lotus = !row.lotus_taxa.is_empty();
    let has_pubchem = !row.pubchem_cids.is_empty();
    let np_score = if row.np_score_available {
        Some(row.np_likeness)
    } else {
        None
    };

    // Structural evidence (RDKit-derived — chemistry, not heuristic). The counts
    // are shared with `assess_np_evidence` via `chemist::count_evidence`, so the
    // natural/synthetic/kingdom split is computed once, never re-derived per
    // call site (the smell that previously made the verdict over-confident).
    let counts = count_evidence(&row.motifs, &row.motif_hits);

    classify_np_evidence(&EvidenceSignals {
        np_score,
        has_lotus,
        has_pubchem,
        lotus_scaffolds: row.lotus_scaffolds.len(),
        counts,
    })
}

/// Flat, cfg-free evidence signals consumed by [`classify_np_evidence`].
///
/// Bundled (rather than passed as bare arguments) to stay under
/// `clippy::too_many_arguments`. Every field is `Copy` so the struct derives
/// `Copy`; `counts` is the single structural-evidence computation shared with
/// [`assess_np_evidence`](super::assessment::assess_np_evidence), so the
/// natural/synthetic/kingdom split is never re-derived per call site.
///
/// Note the deliberate split between `has_lotus` (the molecule itself is a
/// LOTUS organism record — ground truth) and `lotus_scaffolds` (the molecule's
/// *scaffold* is prevalent in >1% of LOTUS compounds — a structural hint, not a
/// database hit on the molecule). The classifier treats them differently.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct EvidenceSignals {
    /// Ertl NP-likeness score (`None` when the model is unavailable).
    pub np_score: Option<f64>,
    /// The molecule itself is a LOTUS natural-product organism record.
    pub has_lotus: bool,
    /// The molecule is backed by `PubChem` records.
    pub has_pubchem: bool,
    /// Count of LOTUS 1%-prevalence scaffold matches (Rutz et al. mortar
    /// fragmentation — scaffolds appearing in >1% of LOTUS molecules). A
    /// structural *hint*, never conflated with `has_lotus`.
    pub lotus_scaffolds: usize,
    /// Full structural-evidence breakdown (motif counts + source/kingdom split).
    pub counts: EvidenceCounts,
}

/// Pure threshold logic behind [`row_verdict`](row_verdict).
///
/// A verdict is "likely a natural product" only when **chemistry** — real
/// structural evidence — backs it, and `score ≥ 2.0` is treated as a *single*
/// argument, never enough on its own. Concretely:
///
/// - **LOTUS organism record** (`has_lotus`) is ground truth: a curated NP with
///   a living source. A non-negative Ertl score upgrades it ("+ strong NP
///   evidence"); otherwise it stays "LOTUS-backed".
/// - **LOTUS 1%-prevalent scaffold** (`lotus_scaffolds`) is a *structural hint*
///   — a scaffold genuinely common in the LOTUS corpus — and is **never**
///   conflated with a database hit on the molecule (it does not mean the
///   molecule itself is in LOTUS). It lifts a verdict to "likely NP" when the
///   Ertl score is strong/confident **and** the molecule carries an NP-typical
///   structural motif (a scaffold, or ≥1 NP substituent); kingdom enrichment and
///   natural-source dominance are *not* required (they are frequently
///   unlabelled for corpus scaffolds), but a synthetic-leaning structure still
///   overrides the hint to the orange warning.
/// - **Synthetic-leaning structure** (synthetic-source-dominant motifs,
///   decoration-heavy side-chains, or a negative Ertl score) is an **orange
///   warning**, not a "likely" signal — a high Ertl score cannot rescue a
///   synthetic-looking structure.
/// - **Ertl NP-likeness** is one weighted input. Only `score ≥ 2.0` PLUS ≥2 NP
///   substituent motifs PLUS an NP scaffold PLUS natural-dominant balance PLUS
///   kingdom enrichment justifies "likely novel NP" when no DB backing exists.
///
/// Score-only signals (high Ertl, no DB, no corroborating structure) are always
/// "citation needed".
#[must_use]
#[allow(clippy::too_many_lines)] // single-responsibility: evidence → verdict classification
pub(crate) fn classify_np_evidence(signals: &EvidenceSignals) -> String {
    let EvidenceSignals {
        np_score,
        has_lotus,
        has_pubchem,
        lotus_scaffolds,
        counts,
    } = *signals;

    let has_lotus_scaffold = lotus_scaffolds > 0;
    // Natural-dominant: natural-source motif hits are a non-zero majority.
    let natural_dominant = counts.natural_hits > 0 && counts.natural_hits >= counts.synthetic_hits;
    let synthetic_majority =
        counts.synthetic_hits > 0 && counts.synthetic_hits > counts.natural_hits;
    // NP-typical scaffold + NP-typical substituent motifs + natural balance.
    let structural_support =
        counts.scaffold_hits > 0 && counts.np_core_hits > 0 && natural_dominant;
    let decoration_heavy = counts.decoration_hits > counts.scaffold_hits;
    let kingdom_support = counts.kingdom_enriched_hits > 0;

    // ---- No Ertl model: only LOTUS scaffolds + structure can speak. ----
    // The destructure makes `score` directly available below — no later
    // re-check or unwrap needed.
    let Some(score) = np_score else {
        if has_lotus_scaffold && !synthetic_majority && structural_support {
            return "🌿 LOTUS-prevalent scaffold + NP structure — supporting evidence (Ertl unavailable)"
                .to_string();
        }
        if has_lotus && structural_support {
            return "🌿 LOTUS + NP structure (Ertl unavailable)".to_string();
        }
        if has_lotus {
            return "🌿 LOTUS organism record (Ertl unavailable)".to_string();
        }
        return "⚠ Citation needed — no Ertl model and no structural evidence".to_string();
    };

    // Strongly negative Ertl: highly synthetic (red flag — atypical of NPs).
    if score <= -2.0 {
        return format!("👃 Smells fishy — highly synthetic (Ertl {score:+.2})");
    }

    let strong = score >= 2.0;
    let confident = score >= 1.0;
    // Orange warning: synthetic-source-dominant motifs, decoration-heavy
    // side-chains, or a negative Ertl score. A high score does not override a
    // synthetic-looking structure. (A LOTUS *organism record* is ground truth,
    // so it is exempt — a real NP flagged by motif noise stays likely.)
    let synthetic_lean = synthetic_majority || decoration_heavy || score < 0.0;
    if synthetic_lean && !has_lotus {
        return format!("🟧 Synthetic-leaning structure (Ertl {score:+.2})");
    }

    // ---- LOTUS organism record (ground truth): strongest database evidence.
    // LOTUS is (almost) always backed by PubChem, so a distinct "LOTUS +
    // PubChem agree" verdict would carry no information — PubChem is implicit.
    // The scaffold-hint below is the *other* LOTUS signal and is kept separate
    // so a LOTUS-prevalent *scaffold* is never conflated with a LOTUS
    // *organism record*. ----
    if has_lotus {
        if strong && structural_support {
            return format!("🌿 LOTUS + strong NP evidence (Ertl {score:+.2})");
        }
        if confident && structural_support {
            return format!("🌿 LOTUS-backed NP evidence (Ertl {score:+.2})");
        }
        return format!("🌿 LOTUS organism record (Ertl {score:+.2})");
    }

    // ---- LOTUS 1%-scaffold HINT (NOT a molecule record — a structural hint).
    // A scaffold prevalent in >1% of LOTUS is genuinely NP-typical, so it must
    // NOT be conflated with a database hit on the molecule. On its own it is a
    // hint; it lifts a verdict to "likely NP" when corroborated by INDEPENDENT
    // chemistry — natural-source motif hits and/or kingdom taxonomy and/or an
    // NP-typical scaffold/substituent motif — plus a non-trivial Ertl score.
    // (This is the fix for the +4.6-on-a-LOTUS-scaffold false nose: the natural
    // motif hits + kingdom enrichment are the corroboration, not just keyword
    // scaffold labels.) ----
    if has_lotus_scaffold {
        // Corroborating structural evidence beyond the scaffold hint itself:
        // any natural-source motif hit, kingdom enrichment, or an NP-typical
        // scaffold/substituent motif. The LOTUS-prevalent scaffold match is the
        // *hint*; these are the independent chemistry signals.
        let corroborates = counts.natural_hits > 0
            || counts.kingdom_enriched_hits > 0
            || counts.scaffold_hits > 0
            || counts.np_core_hits > 0;
        if strong && corroborates {
            return format!(
                "🌿 Likely NP — LOTUS-prevalent scaffold + strong Ertl + natural motifs (Ertl {score:+.2})"
            );
        }
        if strong {
            return format!(
                "👃 Citation needed — LOTUS scaffold hint, no natural corroboration (Ertl {score:+.2})"
            );
        }
        if confident && corroborates {
            return format!(
                "🌿 Likely NP — LOTUS-prevalent scaffold + Ertl + natural motifs (Ertl {score:+.2})"
            );
        }
        if confident {
            return format!(
                "👃 Citation needed — LOTUS scaffold hint, Ertl-only (Ertl {score:+.2})"
            );
        }
        return format!(
            "👃 Citation needed — LOTUS scaffold hint, insufficient corroboration (Ertl {score:+.2})"
        );
    }

    // ---- PubChem only (no LOTUS): weak DB signal — demand score + structure. ----
    if has_pubchem {
        if strong && structural_support {
            return format!("🌿 PubChem + strong NP evidence (Ertl {score:+.2})");
        }
        if strong {
            return format!(
                "👃 Citation needed — PubChem hit, unsupported structure (Ertl {score:+.2})"
            );
        }
        if confident && structural_support {
            return format!("👃 Citation needed — PubChem + strong NP-likeness (Ertl {score:+.2})");
        }
        if confident {
            return format!("📚 PubChem hit — NP-ambiguous (Ertl {score:+.2})");
        }
        return format!("📚 PubChem hit — weak NP signals (Ertl {score:+.2})");
    }

    // ---- No database evidence: stringently require multiple corroborating
    // arguments (never a single score/scaffold) for "likely novel NP". ----
    if strong
        && counts.np_core_hits >= 2
        && counts.scaffold_hits > 0
        && natural_dominant
        && kingdom_support
        && !decoration_heavy
    {
        return format!("🌿 Likely novel NP (Ertl {score:+.2})");
    }
    if strong && structural_support {
        return format!("👃 Citation needed — strong NP-likeness, no DB (Ertl {score:+.2})");
    }
    if strong {
        return format!(
            "👃 Citation needed — strong Ertl but unsupported structure (Ertl {score:+.2})"
        );
    }
    if confident && structural_support {
        return format!("👃 Citation needed — strong NP-likeness, no DB (Ertl {score:+.2})");
    }
    if score >= 0.5 {
        return format!("👃 Citation needed — borderline NP-likeness (Ertl {score:+.2})");
    }
    format!("👃 Citation needed (Ertl {score:+.2})")
}

#[cfg(test)]
mod classify_tests;
