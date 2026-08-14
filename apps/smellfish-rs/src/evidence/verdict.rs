// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Verdict derivation: the human-facing one-liner verdict per row, its
//! machine-readable category (for CSV export), and the ring-family
//! classification. Depends on `chemist` for motif counting.

use crate::model::{RdkitDescriptors, normalized_source_class};

use super::chemist::{count_core_np_motifs, count_kingdom_enriched_hits};

/// Verdict string shown prominently in the UI.
#[cfg(target_arch = "wasm32")]
#[must_use]
pub fn row_verdict(row: &crate::model::MoleculeRow) -> String {
    if let Some(err) = row.error.as_deref() {
        return format!("⚠ {err}");
    }

    let has_lotus = !row.lotus_taxa.is_empty();
    let has_pubchem = !row.pubchem_cids.is_empty();
    let score = row.np_likeness;

    if !row.np_score_available {
        if has_lotus {
            return "🌿 LOTUS-backed evidence".to_string();
        }
        return "⚠ Ertl model not loaded".to_string();
    }

    if has_lotus && score >= 2.0 {
        return format!("🌿 LOTUS + strong NP score (Ertl {score:+.2})");
    }
    if has_lotus && score >= 0.5 {
        return format!("🌿 LOTUS hit (Ertl {score:+.2})");
    }
    if has_lotus {
        return format!("🌿 LOTUS-backed but weak score (Ertl {score:+.2})");
    }

    // ── Very weak/synthetic signals always stay yellow/red ──────────────
    if score <= -1.0 {
        return format!("⚠ Weak NP signals (Ertl {score:+.2})");
    }

    // ── PubChem hit evaluation ──────────────────────────────────────────
    // PubChem is a WEAK signal on its own; strong structural evidence required
    if has_pubchem {
        let np_motif_count = count_core_np_motifs(&row.motifs);
        let has_np_scaffold = row.ring_family.to_ascii_lowercase().contains("polycyclic")
            || row.ring_family.to_ascii_lowercase().contains("steroid")
            || row.ring_family.to_ascii_lowercase().contains("sugar")
            || row.ring_family.to_ascii_lowercase().contains("macrolide")
            || row.ring_family.to_ascii_lowercase().contains("flavonoid")
            || row
                .ring_family
                .to_ascii_lowercase()
                .contains("fused heteroaromatic");
        let structural_support = np_motif_count >= 1 || has_np_scaffold;
        let natural_weight = row
            .motif_hits
            .iter()
            .filter(|hit| normalized_source_class(&hit.source_class) == "natural")
            .count();
        let kingdom_enriched_hits = count_kingdom_enriched_hits(&row.motif_hits);
        let synthetic_weight = row
            .motif_hits
            .iter()
            .filter(|hit| normalized_source_class(&hit.source_class) == "synthetic")
            .count();
        let kingdom_support = kingdom_enriched_hits > 0;

        // ONLY go green if score is strong AND structure looks chemically NP-like.
        if score >= 2.0
            && (structural_support || natural_weight >= synthetic_weight || kingdom_support)
        {
            return format!("🌿 PubChem + strong NP evidence (Ertl {score:+.2})");
        }

        if score >= 2.0 {
            return format!("🌿 Strong NP score (Ertl {score:+.2})");
        }

        // Score 1.0-2.0 (ambiguous) with supporting structural cues → blue
        if score >= 1.0 && structural_support {
            return format!("📚 PubChem with NP motifs (Ertl {score:+.2})");
        }

        // Any PubChem with score 0.5-1.0 → yellow (needs citation)
        if score >= 0.5 {
            return format!("👃 Citation needed — PubChem hit (Ertl {score:+.2})");
        }

        // Score -1.0 to 0.5 → weak signals
        return format!("📚 PubChem — weak NP signals (Ertl {score:+.2})");
    }

    // Not in databases — structural evidence only
    let has_np_scaffold = row.ring_family.to_ascii_lowercase().contains("polycyclic")
        || row.ring_family.to_ascii_lowercase().contains("steroid")
        || row.ring_family.to_ascii_lowercase().contains("sugar")
        || row.ring_family.to_ascii_lowercase().contains("macrolide")
        || row.ring_family.to_ascii_lowercase().contains("flavonoid")
        || row
            .ring_family
            .to_ascii_lowercase()
            .contains("fused heteroaromatic");
    let has_np_motifs = count_core_np_motifs(&row.motifs) > 0;
    let natural_hits = row
        .motif_hits
        .iter()
        .filter(|hit| normalized_source_class(&hit.source_class) == "natural")
        .count();
    let kingdom_enriched_hits = row
        .motif_hits
        .iter()
        .filter(|hit| {
            normalized_source_class(&hit.source_class) == "natural" && !hit.kingdoms.is_empty()
        })
        .count();
    let natural_only_hits = natural_hits.saturating_sub(kingdom_enriched_hits);
    let synthetic_hits = row
        .motif_hits
        .iter()
        .filter(|hit| normalized_source_class(&hit.source_class) == "synthetic")
        .count();
    let natural_weight = natural_only_hits + kingdom_enriched_hits * 2;
    let synthetic_weight = synthetic_hits
        + row
            .motif_hits
            .iter()
            .filter(|hit| normalized_source_class(&hit.source_class) == "unknown")
            .count();
    let kingdom_support = kingdom_enriched_hits > 0;
    if score >= 2.0 && (has_np_scaffold || natural_weight >= synthetic_weight || kingdom_support) {
        return format!("🌿 Likely novel NP (Ertl {score:+.2})");
    }
    if score >= 1.0 && has_np_motifs && has_np_scaffold {
        return format!("🌿 Likely novel NP (Ertl {score:+.2})");
    }
    if score >= 1.0 && kingdom_support && kingdom_enriched_hits >= synthetic_weight {
        return format!("🌿 Kingdom-enriched motifs (Ertl {score:+.2})");
    }
    if score >= 1.0 && natural_only_hits > synthetic_weight && natural_only_hits > 0 {
        return format!("🌿 Natural-leaning motifs (Ertl {score:+.2})");
    }
    if score >= 1.0 && synthetic_weight > natural_weight && synthetic_hits > 0 {
        return format!("⚠ Synthetic-leaning motifs (Ertl {score:+.2})");
    }
    if score >= 2.0 {
        return format!("Strong NP score (Ertl {score:+.2})");
    }
    if score <= -1.0 {
        return format!("⚠ Weak NP signals (Ertl {score:+.2})");
    }
    format!("👃 Citation needed (Ertl {score:+.2})")
}

/// Machine-readable category for CSV export — strips emojis and
/// normalises to "likely", "neutral", "caution", "skeptical", or "fishy".
pub fn category(verdict: &str) -> &'static str {
    let l = verdict.to_ascii_lowercase();

    // RED — Highly synthetic / negative signals / fishy (check first!)
    if l.contains("highly synthetic") || l.contains("smells fishy") {
        return "fishy";
    }

    // GREEN — High NP confidence (LOTUS or strong structural + Ertl score)
    if l.contains("lotus") && !l.contains("weak") {
        return "likely";
    }
    if l.contains("likely hit") {
        return "likely";
    }
    if l.contains("likely novel") {
        return "likely";
    }
    if l.contains("pubchem + strong") {
        return "likely";
    }
    if l.contains("strong np score") && !l.contains("weak") {
        return "likely";
    }

    // YELLOW — Skeptical / ambiguous (needs citation, uncertain origin)
    if l.contains("citation needed") {
        return "skeptical";
    }

    // BLUE — Moderate NP confidence (PubChem with some NP features)
    if l.contains("pubchem with") || l.contains("pubchem + np") {
        return "neutral";
    }

    // RED — Low/weak NP confidence
    if l.contains("weak np signals")
        || (l.contains("pubchem") && l.contains("weak"))
        || (l.contains("ertl") && (l.contains("−1") || l.contains("-1")))
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
