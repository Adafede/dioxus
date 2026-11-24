// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! Chemist visual audit checks and structural motif classification counters.

use crate::model::{
    CheckStatus, ChemistCheck, MoleculeRow, RdkitMotifHit, SourceClass, contains_ignore_ascii_case,
    matches_any,
};

const SCAFFOLD_MOTIFS: &[&str] = &[
    "steroid",
    "sugar",
    "macrolide",
    "macrocycle",
    "lactone",
    "lactam",
    "flavone",
    "flavonoid",
    "indole",
    "quinoline",
    "isoquinoline",
    "benzofuran",
    "benzothiophene",
    "quinoxaline",
    "purine",
    "chromone",
    "coumarin",
    "morpholine",
    "piperidine",
    "piperazine",
    "tetrahydrofuran",
    "tetrahydropyran",
];

const DECORATION_MOTIFS: &[&str] = &[
    "aldehyde",
    "ketone",
    "carboxylic acid",
    "ester",
    "amide",
    "carbamate",
    "urea",
    "sulfonamide",
    "sulfone",
    "sulfoxide",
    "alcohol",
    "phenol",
    "ether",
    "methoxy",
    "amine",
    "halogen",
    "nitrile",
    "nitro",
    "thiol",
    "phosphate",
    "acetal",
    "epoxide",
    "allyl",
];

/// Runs the three visual chemist checks (NP-likeness score, skeleton, database presence).
#[must_use]
pub(crate) fn run_checks(row: &MoleculeRow) -> Vec<ChemistCheck> {
    let (np_status, np_detail) = match row.np_score {
        None => (CheckStatus::Warn, "Ertl model not loaded".into()),
        Some(np) if np.score >= 0.5 => (CheckStatus::Pass, format!("Ertl score {:+.2}", np.score)),
        Some(np) if np.score > -0.5 => (
            CheckStatus::Warn,
            format!("Ertl score {:+.2} — borderline", np.score),
        ),
        Some(np) => (
            CheckStatus::Fail,
            format!("Ertl score {:+.2} — synthetic-leaning", np.score),
        ),
    };

    let skel_status = if row.ring_family.is_polyaromatic() {
        CheckStatus::Fail
    } else if row.ring_family.is_np_skeleton() {
        CheckStatus::Pass
    } else {
        CheckStatus::Warn
    };

    let (db_status, db_detail) = match (!row.lotus_taxa.is_empty(), !row.pubchem_cids.is_empty()) {
        (true, true) => (CheckStatus::Pass, "LOTUS + PubChem"),
        (true, false) => (CheckStatus::Pass, "LOTUS taxa"),
        (false, true) => (CheckStatus::Warn, "PubChem only"),
        (false, false) => (CheckStatus::Fail, "Not in LOTUS or PubChem"),
    };

    vec![
        ChemistCheck {
            name: "NP-likeness",
            status: np_status,
            detail: np_detail,
        },
        ChemistCheck {
            name: "Skeleton",
            status: skel_status,
            detail: row.ring_family.as_str().to_string(),
        },
        ChemistCheck {
            name: "Database",
            status: db_status,
            detail: db_detail.into(),
        },
    ]
}

/// Returns `true` when `label` names a scaffold motif known to be enriched in natural products.
#[must_use]
pub(crate) fn is_known_np_motif(label: &str) -> bool {
    matches_any(label, SCAFFOLD_MOTIFS)
}

/// Returns `true` for scaffold ring systems characteristic of natural products.
#[must_use]
pub(crate) fn is_scaffold_motif(label: &str) -> bool {
    matches_any(label, SCAFFOLD_MOTIFS)
        || contains_ignore_ascii_case(label, "ring")
        || contains_ignore_ascii_case(label, "cyclohexane")
}

/// Returns `true` for side-chain / functional-group decoration motifs.
#[must_use]
pub(crate) fn is_decoration_motif(label: &str) -> bool {
    matches_any(label, DECORATION_MOTIFS)
}

/// Per-molecule structural evidence counts computed once and shared across assessment and verdict.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct EvidenceCounts {
    /// Ertl NP-typical scaffold motifs found in `motifs`.
    pub(crate) np_core: usize,
    /// Scaffold ring motif hits.
    pub(crate) scaffold: usize,
    /// Side-chain / decoration motifs found in `motifs`.
    pub(crate) decoration: usize,
    /// Motif hits classified as natural origin.
    pub(crate) natural: usize,
    /// Motif hits classified as synthetic origin.
    pub(crate) synthetic: usize,
    /// Motif hits with unclassified origin.
    pub(crate) unknown: usize,
    /// Natural motif hits carrying at least one kingdom annotation.
    pub(crate) kingdom_enriched: usize,
}

/// Computes structural evidence counts for a molecule once from motifs and motif hits.
#[must_use]
pub(crate) fn count_evidence(motifs: &[String], motif_hits: &[RdkitMotifHit]) -> EvidenceCounts {
    let mut counts = EvidenceCounts::default();

    for motif in motifs {
        if is_known_np_motif(motif) {
            counts.np_core += 1;
        }
        if is_decoration_motif(motif) {
            counts.decoration += 1;
        }
    }

    for hit in motif_hits {
        if is_scaffold_motif(&hit.label) {
            counts.scaffold += 1;
        }
        match hit.source_class {
            SourceClass::Natural => {
                counts.natural += 1;
                if !hit.kingdoms.is_empty() {
                    counts.kingdom_enriched += 1;
                }
            }
            SourceClass::Synthetic => counts.synthetic += 1,
            SourceClass::Unknown => counts.unknown += 1,
        }
    }

    counts
}
