// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! Scaffold family classification from motif labels + `RDKit` descriptors.
//!
//! This is a structural analysis function that belongs in `chemist` (the
//! cfg-free leaf module for chemist's-eye-view structural classification),
//! not in `verdict` (which derives the human-readable verdict string).

use crate::model::RdkitDescriptors;

use super::chemist::matches_any;

/// Classify the core scaffold family using motif labels and descriptor
/// heuristics.
#[must_use]
pub(crate) fn classify_ring_family(descriptors: &RdkitDescriptors, motifs: &[String]) -> String {
    let has_any = |keywords: &[&str]| motifs.iter().any(|m| matches_any(m, keywords));

    let label = if has_any(&["steroid"]) {
        // Steroids — tetracyclic fused ring system.
        "steroid-like fused ring system"
    } else if has_any(&["sugar", "tetrahydrofuran"]) {
        // Monosaccharides and THF rings — common in glycosylated NPs.
        "sugar-like oxygenated ring system"
    } else if has_any(&["macrolide", "macrocycle", "lactone", "lactam"]) {
        // Macrocycles, lactones, lactams — hallmark macrocyclic NP scaffolds.
        "macrolide-like oxygenated macrocycle"
    } else if has_any(&["flavone", "flavonoid"]) {
        // Benzopyran, flavone, flavonoid — plant secondary metabolite cores.
        "flavonoid-like scaffold"
    } else if has_any(&[
        "indole",
        "quinoline",
        "isoquinoline",
        "benzofuran",
        "benzothiophene",
        "quinoxaline",
        "purine",
        "chromone",
        "coumarin",
    ]) {
        // N-heteroaromatic scaffolds — common in both NPs and synthetic drugs.
        "fused heteroaromatic scaffold"
    } else {
        family_from_descriptors(descriptors)
    };
    label.to_string()
}

/// Derive the scaffold family when motif labels don't provide a clue.
fn family_from_descriptors(descriptors: &RdkitDescriptors) -> &'static str {
    let rings = descriptors.ring_count.unwrap_or(0.0);
    let aromatic = descriptors.aromatic_ring_count.unwrap_or(0.0);
    let aliphatic = descriptors.aliphatic_ring_count.unwrap_or(0.0);
    let csp3 = descriptors.fraction_csp3.unwrap_or(0.0);

    if rings <= 0.0 {
        "acyclic"
    } else if aromatic >= 3.0 {
        "polyaromatic scaffold"
    } else if aromatic > 0.0 && aliphatic > 0.0 {
        "mixed aromatic/aliphatic scaffold"
    } else if rings >= 2.0 && (aliphatic >= 2.0 || csp3 >= 0.5) {
        "natural-product-like polycyclic scaffold"
    } else {
        "compact ring scaffold"
    }
}

#[cfg(test)]
mod tests;
