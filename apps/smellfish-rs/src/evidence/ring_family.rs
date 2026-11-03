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

    // Steroids — tetracyclic fused ring system.
    if has_any(&["steroid"]) {
        return "steroid-like fused ring system".to_string();
    }
    // Monosaccharides and THF rings — common in glycosylated NPs.
    if has_any(&["sugar", "tetrahydrofuran"]) {
        return "sugar-like oxygenated ring system".to_string();
    }
    // Macrocycles, lactones, lactams — hallmark macrocyclic NP scaffolds.
    if has_any(&["macrolide", "macrocycle", "lactone", "lactam"]) {
        return "macrolide-like oxygenated macrocycle".to_string();
    }
    // Benzopyran, flavone, flavonoid — plant secondary metabolite cores.
    if has_any(&["flavone", "flavonoid"]) {
        return "flavonoid-like scaffold".to_string();
    }
    // N-heteroaromatic scaffolds — common in both NPs and synthetic drugs.
    if has_any(&[
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
        return "fused heteroaromatic scaffold".to_string();
    }

    family_from_descriptors(descriptors)
}

/// Derive the scaffold family when motif labels don't provide a clue.
fn family_from_descriptors(descriptors: &RdkitDescriptors) -> String {
    let rings = descriptors.ring_count.unwrap_or(0.0);
    let aromatic = descriptors.aromatic_ring_count.unwrap_or(0.0);
    let aliphatic = descriptors.aliphatic_ring_count.unwrap_or(0.0);
    let csp3 = descriptors.fraction_csp3.unwrap_or(0.0);

    if rings <= 0.0 {
        return "acyclic".to_string();
    }
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
mod tests;
