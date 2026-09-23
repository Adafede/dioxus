// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! Scaffold family classification from motif labels + `RDKit` descriptors.
//!
//! This is a structural analysis function that belongs in `chemist` (the
//! cfg-free leaf module for chemist's-eye-view structural classification),
//! not in `verdict` (which derives the human-readable verdict string).

use crate::model::RdkitDescriptors;

/// Classify the core scaffold family using motif labels and descriptor
/// heuristics.
#[must_use]
pub fn classify_ring_family(descriptors: &RdkitDescriptors, motifs: &[String]) -> String {
    let motif_text = motifs.join(" ").to_ascii_lowercase();

    // Steroids — tetracyclic fused ring system.
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
mod tests {
    use super::*;
    use crate::model::RdkitDescriptors;

    fn empty_descriptors() -> RdkitDescriptors {
        RdkitDescriptors::default()
    }

    #[test]
    fn polycyclic_aliphatic_scaffold() {
        let desc = RdkitDescriptors {
            ring_count: Some(4.0),
            aromatic_ring_count: Some(0.0),
            aliphatic_ring_count: Some(4.0),
            fraction_csp3: Some(0.75),
        };
        assert_eq!(
            classify_ring_family(&desc, &[]),
            "natural-product-like polycyclic scaffold"
        );
    }

    #[test]
    fn polyaromatic_scaffold() {
        let desc = RdkitDescriptors {
            ring_count: Some(3.0),
            aromatic_ring_count: Some(3.0),
            aliphatic_ring_count: Some(0.0),
            fraction_csp3: Some(0.10),
        };
        assert_eq!(classify_ring_family(&desc, &[]), "polyaromatic scaffold");
    }

    #[test]
    fn motif_labels_override_descriptor_heuristics() {
        assert_eq!(
            classify_ring_family(&empty_descriptors(), &["Flavone ring".to_string()]),
            "flavonoid-like scaffold"
        );
        assert_eq!(
            classify_ring_family(&empty_descriptors(), &["Indole ring".to_string()]),
            "fused heteroaromatic scaffold"
        );
        assert_eq!(
            classify_ring_family(
                &empty_descriptors(),
                &["Sugar-like oxygen ring".to_string()]
            ),
            "sugar-like oxygenated ring system"
        );
    }
}
