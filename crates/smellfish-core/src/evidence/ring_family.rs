// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! Scaffold ring-family classification from motif labels and `RDKit` descriptors.

use crate::model::{RdkitDescriptors, RingFamily, matches_any};

/// Classifies the molecule's core scaffold ring family.
#[must_use]
pub fn classify_ring_family(descriptors: &RdkitDescriptors, motifs: &[String]) -> RingFamily {
    let has_any = |keywords: &[&str]| motifs.iter().any(|m| matches_any(m, keywords));

    if has_any(&["steroid"]) {
        RingFamily::Steroid
    } else if has_any(&["sugar", "tetrahydrofuran"]) {
        RingFamily::Sugar
    } else if has_any(&["macrolide", "macrocycle", "lactone", "lactam"]) {
        RingFamily::Macrolide
    } else if has_any(&["flavone", "flavonoid"]) {
        RingFamily::Flavonoid
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
        RingFamily::Heteroaromatic
    } else {
        family_from_descriptors(descriptors)
    }
}

fn family_from_descriptors(descriptors: &RdkitDescriptors) -> RingFamily {
    let rings = descriptors.ring_count.unwrap_or(0.0);
    let aromatic = descriptors.aromatic_ring_count.unwrap_or(0.0);
    let aliphatic = descriptors.aliphatic_ring_count.unwrap_or(0.0);
    let csp3 = descriptors.fraction_csp3.unwrap_or(0.0);

    if rings <= 0.0 {
        RingFamily::Acyclic
    } else if aromatic >= 3.0 {
        RingFamily::Polyaromatic
    } else if aromatic > 0.0 && aliphatic > 0.0 {
        RingFamily::MixedAromaticAliphatic
    } else if rings >= 2.0 && (aliphatic >= 2.0 || csp3 >= 0.5) {
        RingFamily::Polycyclic
    } else {
        RingFamily::CompactRing
    }
}
