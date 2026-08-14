// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Verdict derivation: the human-facing one-liner verdict per row, its
//! machine-readable category (for CSV export), and the ring-family
//! classification. Depends on `chemist` for motif counting.

use crate::model::RdkitDescriptors;

use super::chemist::{EvidenceCounts, count_evidence};

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
pub fn row_verdict(row: &crate::model::MoleculeRow) -> String {
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
#[derive(Clone, Copy, Default)]
pub struct EvidenceSignals {
    /// Ertl NP-likeness score (`None` when the model is unavailable).
    pub np_score: Option<f64>,
    /// The molecule itself is a LOTUS natural-product organism record.
    pub has_lotus: bool,
    /// The molecule is backed by PubChem records.
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
pub fn classify_np_evidence(signals: &EvidenceSignals) -> String {
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
    if np_score.is_none() {
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
    }

    let score = np_score.unwrap();

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

/// Machine-readable category for CSV export — strips emojis and
/// normalises to "likely", "neutral", "caution", "skeptical", or "fishy".
///
/// Order matters: `"citation needed"` is matched *before* `"lotus"` so that a
/// LOTUS-backenced molecule that still reads "citation needed" is not mis-filed
/// as "likely"; `"synthetic-leaning"` is matched before both so a structural
/// warning is an orange `caution`, not green.
pub fn category(verdict: &str) -> &'static str {
    let l = verdict.to_ascii_lowercase();

    // RED — Highly synthetic / fishy (check first!).
    if l.contains("highly synthetic") || l.contains("smells fishy") {
        return "fishy";
    }

    // ORANGE — Synthetic-leaning structural warning.
    if l.contains("synthetic-leaning") {
        return "caution";
    }

    // YELLOW — Skeptical (needs citation). Before `lotus` so a LOTUS-backed
    // molecule that still reads "citation needed" is not filed as "likely".
    if l.contains("citation needed") {
        return "skeptical";
    }

    // GREEN — High NP confidence (LOTUS or strong structural + Ertl score).
    if l.contains("lotus") {
        return "likely";
    }
    if l.contains("likely hit") || l.contains("likely novel") {
        return "likely";
    }
    if l.contains("pubchem + strong") {
        return "likely";
    }
    if l.contains("strong np score") && !l.contains("weak") {
        return "likely";
    }

    // BLUE — Moderate NP confidence (PubChem with some NP features).
    if l.contains("pubchem with") || l.contains("pubchem + np") {
        return "neutral";
    }

    // RED — Low/weak NP confidence.
    if l.contains("weak np signals")
        || (l.contains("pubchem") && l.contains("weak"))
        || (l.contains("ertl") && l.contains("-1"))
    {
        return "caution";
    }

    "neutral"
}

/// Classify the core scaffold family using motif SMARTS matches and
/// descriptor-based heuristics.
#[must_use]
pub fn classify_ring_family(descriptors: &RdkitDescriptors, motifs: &[String]) -> String {
    let motif_text = motifs.join(" ").to_ascii_lowercase();

    // Steroids — tetracyclic fused ring system with characteristic
    // cyclopentanoperhydrophenanthrene core.
    if motif_text.contains("steroid") {
        return "steroid-like fused ring system".to_string();
    }
    // Monosaccharides and THF rings — common in glycosylated NPs.
    if motif_text.contains("sugar") || motif_text.contains("tetrahydrofuran") {
        return "sugar-like oxygenated ring system".to_string();
    }
    // Macrocycles, lactones, lactams — hallmark macrocyclic NP scaffolds.
    if motif_text.contains("macrolide")
        || motif_text.contains("macrocycle")
        || motif_text.contains("lactone")
        || motif_text.contains("lactam")
    {
        return "macrolide-like oxygenated macrocycle".to_string();
    }
    // Benzopyran, flavone, flavonoid — plant secondary metabolite cores.
    if motif_text.contains("flavone") || motif_text.contains("flavonoid") {
        return "flavonoid-like scaffold".to_string();
    }
    // N-heteroaromatic scaffolds — common in both NPs and synthetic drugs.
    if motif_text.contains("indole")
        || motif_text.contains("quinoline")
        || motif_text.contains("isoquinoline")
        || motif_text.contains("benzofuran")
        || motif_text.contains("benzothiophene")
        || motif_text.contains("quinoxaline")
        || motif_text.contains("purine")
        || motif_text.contains("chromone")
        || motif_text.contains("coumarin")
    {
        return "fused heteroaromatic scaffold".to_string();
    }

    let rings = descriptors.ring_count.unwrap_or(0.0);
    let aromatic = descriptors.aromatic_ring_count.unwrap_or(0.0);
    let aliphatic = descriptors.aliphatic_ring_count.unwrap_or(0.0);
    let csp3 = descriptors.fraction_csp3.unwrap_or(0.0);

    if rings <= 0.0 {
        return "acyclic".to_string();
    }
    // Three or more aromatic rings → polyaromatic (PAHs, not typical of NPs).
    if aromatic >= 3.0 {
        return "polyaromatic scaffold".to_string();
    }
    if aromatic > 0.0 && aliphatic > 0.0 {
        return "mixed aromatic/aliphatic scaffold".to_string();
    }
    if aliphatic >= 2.0 || csp3 >= 0.5 {
        return "natural-product-like polycyclic scaffold".to_string();
    }
    "compact ring scaffold".to_string()
}

#[cfg(test)]
mod classify_tests {
    use super::super::chemist::{EvidenceCounts, count_evidence};
    use super::{EvidenceSignals, category, classify_np_evidence};

    /// Build evidence counts without touching `count_evidence` — used to
    /// isolate the classifier thresholds.
    fn counts(
        np_core: usize,
        scaffold: usize,
        decoration: usize,
        natural: usize,
        synthetic: usize,
        kingdom: usize,
    ) -> EvidenceCounts {
        EvidenceCounts {
            np_core_hits: np_core,
            scaffold_hits: scaffold,
            decoration_hits: decoration,
            natural_hits: natural,
            synthetic_hits: synthetic,
            unknown_hits: 0,
            kingdom_enriched_hits: kingdom,
        }
    }

    fn ev(
        score: f64,
        lotus: bool,
        pubchem: bool,
        lotus_sc: usize,
        c: EvidenceCounts,
    ) -> EvidenceSignals {
        EvidenceSignals {
            np_score: Some(score),
            has_lotus: lotus,
            has_pubchem: pubchem,
            lotus_scaffolds: lotus_sc,
            counts: c,
        }
    }

    fn ev_none(lotus: bool, pubchem: bool, lotus_sc: usize, c: EvidenceCounts) -> EvidenceSignals {
        EvidenceSignals {
            np_score: None,
            has_lotus: lotus,
            has_pubchem: pubchem,
            lotus_scaffolds: lotus_sc,
            counts: c,
        }
    }

    #[test]
    fn no_model_no_evidence_is_citation() {
        assert_eq!(
            classify_np_evidence(&ev_none(false, false, 0, counts(0, 0, 0, 0, 0, 0))),
            "⚠ Citation needed — no Ertl model and no structural evidence"
        );
    }

    #[test]
    fn no_model_lotus_scaffold_without_structure_is_citation() {
        assert!(
            classify_np_evidence(&ev_none(false, false, 1, counts(0, 0, 0, 0, 0, 0)))
                .contains("Citation needed")
        );
    }

    #[test]
    fn no_model_lotus_scaffold_with_structure_is_supporting() {
        let v = classify_np_evidence(&ev_none(false, false, 1, counts(2, 1, 0, 3, 0, 1)));
        assert!(
            v.contains("🌿 LOTUS-prevalent scaffold + NP structure"),
            "got: {v}"
        );
    }

    #[test]
    fn strongly_negative_smells_fishy() {
        let v = classify_np_evidence(&ev(-2.5, false, false, 0, counts(0, 0, 0, 0, 0, 0)));
        assert!(v.contains("Smells fishy — highly synthetic"), "got: {v}");
    }

    #[test]
    fn synthetic_majority_is_orange_warning() {
        let v = classify_np_evidence(&ev(2.5, false, false, 0, counts(0, 0, 0, 0, 3, 0)));
        assert!(v.contains("🟧 Synthetic-leaning"), "got: {v}");
        assert!(!v.contains("LOTUS"), "got: {v}");
    }

    #[test]
    fn decoration_heavy_is_orange_warning() {
        let v = classify_np_evidence(&ev(2.5, false, false, 0, counts(0, 0, 1, 0, 0, 0)));
        assert!(v.contains("🟧 Synthetic-leaning"), "got: {v}");
    }

    #[test]
    fn negative_score_is_orange_warning() {
        let v = classify_np_evidence(&ev(-1.0, false, false, 0, counts(0, 0, 0, 0, 0, 0)));
        assert!(v.contains("🟧 Synthetic-leaning"), "got: {v}");
    }

    #[test]
    fn lotus_scaffold_hint_alone_is_citation() {
        let v = classify_np_evidence(&ev(2.5, false, false, 1, counts(0, 0, 0, 0, 0, 0)));
        assert!(v.contains("Citation needed"), "got: {v}");
        assert!(!v.contains("strong NP evidence"), "got: {v}");
    }

    #[test]
    fn lotus_scaffold_hint_with_corroboration_is_likely() {
        let v = classify_np_evidence(&ev(2.5, false, false, 1, counts(2, 1, 0, 3, 0, 1)));
        assert!(
            v.contains("🌿 Likely NP — LOTUS-prevalent scaffold"),
            "got: {v}"
        );
        assert!(v.contains("Ertl +2.50"), "got: {v}");
    }

    #[test]
    fn lotus_scaffold_strong_score_is_good_candidate() {
        // The exact regression the chemistry flagged: a +4.59 Ertl score on a
        // LOTUS-prevalent scaffold with scaffolds present must NOT be a
        // citation-needed nose — it is a likely NP candidate.
        let v = classify_np_evidence(&ev(4.59, false, false, 2, counts(0, 2, 0, 2, 0, 0)));
        assert!(
            v.contains("🌿 Likely NP — LOTUS-prevalent scaffold"),
            "got: {v}"
        );
        assert!(v.contains("Ertl +4.59"), "got: {v}");
        assert!(!v.contains("Citation needed"), "got: {v}");
    }

    #[test]
    fn lotus_scaffold_strong_score_with_natural_motifs_is_good_candidate() {
        // Regression for the reported false nose: a very strong Ertl (+4.76) on
        // a LOTUS-prevalent scaffold, corroborated by natural-source motif hits
        // + kingdom enrichment (and *no* scaffold keyword in motif_hits) — a
        // good candidate, never "citation needed / no structural corroboration".
        let v = classify_np_evidence(&ev(4.76, false, false, 9, counts(0, 0, 0, 12, 0, 7)));
        assert!(v.contains("🌿 Likely NP"), "got: {v}");
        assert!(v.contains("Ertl +4.76"), "got: {v}");
        assert!(!v.contains("Citation needed"), "got: {v}");
        assert_eq!(category(&v), "likely");
    }

    #[test]
    fn lotus_scaffold_does_not_claim_lotus_organism_record() {
        // A scaffold hint must NOT be worded as a LOTUS database hit.
        let v = classify_np_evidence(&ev(2.5, false, false, 1, counts(2, 1, 0, 3, 0, 1)));
        assert!(!v.contains("LOTUS organism record"), "got: {v}");
        assert!(!v.contains("LOTUS-backed"), "got: {v}");
    }

    #[test]
    fn lotus_strong_requires_structure() {
        let v = classify_np_evidence(&ev(2.5, true, false, 0, counts(2, 1, 0, 3, 0, 1)));
        assert!(v.contains("🌿 LOTUS + strong NP evidence"), "got: {v}");
        assert!(v.contains("Ertl +2.50"), "got: {v}");
    }

    #[test]
    fn lotus_organism_record_backed() {
        let v = classify_np_evidence(&ev(1.5, true, false, 0, counts(0, 0, 0, 0, 0, 0)));
        assert!(v.contains("🌿 LOTUS organism record"), "got: {v}");
        assert!(!v.contains("strong NP"), "got: {v}");
    }

    #[test]
    fn lotus_with_pubchem_recorded_as_lotus() {
        // LOTUS is (almost) always backed by PubChem, so a LOTUS organism that
        // is also in PubChem is simply a LOTUS record — there is no separate
        // "LOTUS + PubChem agree" tier.
        let v = classify_np_evidence(&ev(1.2, true, true, 0, counts(0, 0, 0, 1, 0, 0)));
        assert!(v.contains("🌿 LOTUS organism record"), "got: {v}");
        assert!(!v.contains("PubChem agree"), "got: {v}");
    }

    #[test]
    fn pubchem_strong_requires_natural_dominance() {
        let v = classify_np_evidence(&ev(2.6, false, true, 0, counts(2, 1, 0, 3, 0, 1)));
        assert!(v.contains("🌿 PubChem + strong NP evidence"), "got: {v}");
    }

    #[test]
    fn pubchem_synthetic_majority_is_orange() {
        let v = classify_np_evidence(&ev(2.6, false, true, 0, counts(0, 0, 0, 0, 3, 0)));
        assert!(v.contains("🟧 Synthetic-leaning"), "got: {v}");
        assert!(!v.contains("PubChem + strong"), "got: {v}");
    }

    #[test]
    fn pubchem_weak_score_is_weak_signals() {
        let v = classify_np_evidence(&ev(0.6, false, true, 0, counts(0, 0, 0, 0, 0, 0)));
        assert!(v.contains("📚 PubChem hit — weak NP signals"), "got: {v}");
    }

    #[test]
    fn novel_np_requires_all_chemistry_signals() {
        let v = classify_np_evidence(&ev(2.6, false, false, 0, counts(2, 1, 0, 4, 0, 1)));
        assert!(v.contains("🌿 Likely novel NP"), "got: {v}");
        assert!(v.contains("Ertl +2.60"), "got: {v}");
    }

    #[test]
    fn novel_np_without_kingdom_is_citation() {
        let v = classify_np_evidence(&ev(2.6, false, false, 0, counts(2, 1, 0, 4, 0, 0)));
        assert!(v.contains("Citation needed"), "got: {v}");
        assert!(!v.contains("novel NP"), "got: {v}");
    }

    #[test]
    fn novel_np_blocked_by_synthetic_majority_is_orange() {
        let v = classify_np_evidence(&ev(2.6, false, false, 0, counts(2, 1, 0, 4, 5, 1)));
        assert!(v.contains("🟧 Synthetic-leaning"), "got: {v}");
        assert!(!v.contains("novel NP"), "got: {v}");
    }

    #[test]
    fn novel_np_blocked_by_decoration_heavy_is_orange() {
        let v = classify_np_evidence(&ev(2.6, false, false, 0, counts(2, 1, 3, 4, 0, 1)));
        assert!(v.contains("🟧 Synthetic-leaning"), "got: {v}");
        assert!(!v.contains("novel NP"), "got: {v}");
    }

    #[test]
    fn high_score_no_db_no_structure_is_citation() {
        let v = classify_np_evidence(&ev(2.0, false, false, 0, counts(0, 0, 0, 0, 0, 0)));
        assert!(v.contains("Citation needed"), "got: {v}");
        assert!(!v.contains("novel NP"), "got: {v}");
    }

    #[test]
    fn low_score_no_evidence_is_citation() {
        assert_eq!(
            classify_np_evidence(&ev(0.3, false, false, 0, counts(0, 0, 0, 0, 0, 0))),
            "👃 Citation needed (Ertl +0.30)"
        );
    }

    #[test]
    fn count_evidence_feeds_classifier() {
        // The shared count_evidence must produce the same counts the classifier
        // reads, so an all-natural molecule scores as likely-novel when strong.
        let motifs = [
            "Steroid-like fused ring".to_string(),
            "Flavone ring".to_string(),
        ];
        let hits = [
            crate::model::RdkitMotifHit {
                label: "Flavone ring".to_string(),
                source_class: "natural".to_string(),
                kingdom: "plants".to_string(),
                kingdoms: vec!["plants".to_string()],
            },
            crate::model::RdkitMotifHit {
                label: "Steroid fused ring".to_string(),
                source_class: "natural".to_string(),
                kingdom: "plants".to_string(),
                kingdoms: vec!["plants".to_string()],
            },
        ];
        let counts = count_evidence(&motifs, &hits);
        assert_eq!(counts.np_core_hits, 2);
        assert_eq!(counts.scaffold_hits, 2);
        assert_eq!(counts.natural_hits, 2);
        assert_eq!(counts.kingdom_enriched_hits, 2);
        assert_eq!(counts.synthetic_hits, 0);
        let v = classify_np_evidence(&ev(2.6, false, false, 0, counts));
        assert!(v.contains("🌿 Likely novel NP"), "got: {v}");
    }
}
