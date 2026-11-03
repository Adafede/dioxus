// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! Chemist's-eye-view structural checks and motif-counting helpers.
//!
//! This is a *leaf* module: it depends only on `crate::model` and on nothing
//! in `assessment`/`verdict`, so the dependency graph stays acyclic
//! (assessment → verdict → chemist).

#[cfg(target_arch = "wasm32")]
use crate::model::{ChemistCheck, MoleculeRow};
use crate::model::{RdkitMotifHit, normalized_source_class};

/// Quick-check audit that a natural-product chemist would run when
/// eyeballing a structure.
///
/// Each check returns a `ChemistCheck` with status `"pass"`, `"warn"`, or
/// `"fail"`.  The checks are:
///
/// 1. **NP-likeness** — the Ertl score is the statistical ground truth;
///    scores above 0.5 are supportive of NP origin.
/// 2. **Skeleton** — scaffolds that match known natural-product ring systems
///    (steroids, sugars, macrocycles, flavonoids, etc.) score positively;
///    pure polyaromatic systems are a red flag.
/// 3. **Database** — presence in LOTUS (curated NP database) or `PubChem`
///    provides orthogonal evidence.
///
/// **Stereochemistry is deliberately NOT checked** — 2D SMILES from mass
/// spectrometry annotation pipelines may not preserve stereochemical
/// information, so the absence of stereo tags is not a reliable negative
/// signal.
#[cfg(target_arch = "wasm32")]
#[must_use]
pub(crate) fn run_checks(row: &MoleculeRow) -> Vec<ChemistCheck> {
    let mut checks: Vec<ChemistCheck> = Vec::with_capacity(3);

    // 1 — NP-likeness score
    if !row.np_score_available {
        checks.push(ChemistCheck {
            name: "NP-likeness",
            status: "warn",
            detail: "Ertl model not loaded".into(),
        });
    } else if row.np_likeness >= 0.5 {
        checks.push(ChemistCheck {
            name: "NP-likeness",
            status: "pass",
            detail: format!("Ertl score {:+.2}", row.np_likeness),
        });
    } else if row.np_likeness > -0.5 {
        checks.push(ChemistCheck {
            name: "NP-likeness",
            status: "warn",
            detail: format!("Ertl score {:+.2} — borderline", row.np_likeness),
        });
    } else {
        checks.push(ChemistCheck {
            name: "NP-likeness",
            status: "fail",
            detail: format!("Ertl score {:+.2} — synthetic-leaning", row.np_likeness),
        });
    }

    // 2 — Skeleton / scaffold classification
    let family = &row.ring_family;
    let is_polyaromatic = contains_ignore_ascii_case(family, "polyaromatic");
    let is_np_skeleton = matches_any(
        family,
        &[
            "polycyclic",
            "steroid",
            "sugar",
            "macrolide",
            "flavonoid",
            "heteroaromatic",
        ],
    );
    if is_polyaromatic {
        checks.push(ChemistCheck {
            name: "Skeleton",
            status: "fail",
            detail: row.ring_family.clone(),
        });
    } else if is_np_skeleton {
        checks.push(ChemistCheck {
            name: "Skeleton",
            status: "pass",
            detail: row.ring_family.clone(),
        });
    } else {
        checks.push(ChemistCheck {
            name: "Skeleton",
            status: "warn",
            detail: row.ring_family.clone(),
        });
    }

    // 3 — Database presence
    let has_lotus = !row.lotus_taxa.is_empty();
    let has_pubchem = !row.pubchem_cids.is_empty();
    if has_lotus && has_pubchem {
        checks.push(ChemistCheck {
            name: "Database",
            status: "pass",
            detail: "LOTUS + PubChem".into(),
        });
    } else if has_lotus {
        checks.push(ChemistCheck {
            name: "Database",
            status: "pass",
            detail: "LOTUS taxa".into(),
        });
    } else if has_pubchem {
        checks.push(ChemistCheck {
            name: "Database",
            status: "warn",
            detail: "PubChem only".into(),
        });
    } else {
        checks.push(ChemistCheck {
            name: "Database",
            status: "fail",
            detail: "Not in LOTUS or PubChem".into(),
        });
    }

    checks
}

/// Motif labels that indicate scaffold-class characteristics of natural products.
///
/// Shared by [`is_scaffold_motif`] and [`is_known_np_motif`] so the keyword list
/// exists in exactly one place. `is_scaffold_motif` adds `"ring"` and
/// `"cyclohexane"` on top.
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

/// Decoration motifs (functional groups or side-chain fragments).
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

pub(super) use crate::model::{contains_ignore_ascii_case, matches_any};

/// Motif labels that are known to be enriched in natural products.
///
/// Based on Ertl & Schuhmann (J. Nat. Prod. 2019, Vol. 82, 1258-1263,
/// DOI 10.1021/acs.jnatprod.8b01022) and Wetzel et al. (CHIMIA 2007,
/// DOI 10.2533/chimia.2007.355).
///
/// Used to highlight motifs characteristic of NP biosynthesis in the UI.
#[must_use]
pub(crate) fn is_known_np_motif(label: &str) -> bool {
    matches_any(label, SCAFFOLD_MOTIFS)
}

/// Scaffold motifs are ring systems or cores characteristic of natural-product
/// scaffold classes.
#[must_use]
pub(crate) fn is_scaffold_motif(label: &str) -> bool {
    matches_any(label, SCAFFOLD_MOTIFS)
        || contains_ignore_ascii_case(label, "ring")
        || contains_ignore_ascii_case(label, "cyclohexane")
}

/// Decoration motifs are functional groups or side-chain fragments.
pub(crate) fn is_decoration_motif(label: &str) -> bool {
    matches_any(label, DECORATION_MOTIFS)
}

/// Per-molecule structural evidence counts.
///
/// Computed once from Ertl motif labels and motif hits, and reused by both
/// the verdict classifier and the assessment note builders.
///
/// Centralised in `chemist` (a cfg-free leaf module) so the verdict threshold
/// logic is unit-testable on native *without* the rdkit.js bridge.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[allow(clippy::struct_field_names)] // evidence counts share the `EvidenceCounts` naming pattern
pub(crate) struct EvidenceCounts {
    /// Ertl NP-typical substituent motifs found (Ertl & Schuhmann 2019 et al.).
    pub np_core_hits: usize,
    /// Structural scaffold motif hits (ring cores characteristic of NP scaffolds).
    pub scaffold_hits: usize,
    /// Decoration / side-chain motifs (synthetic-typical functional groups).
    pub decoration_hits: usize,
    /// Motif hits whose source is classified "natural".
    pub natural_hits: usize,
    /// Motif hits whose source is classified "synthetic".
    pub synthetic_hits: usize,
    /// Motif hits whose source is "unknown".
    pub unknown_hits: usize,
    /// Motif hits with a natural source AND ≥1 kingdom (taxonomy support).
    pub kingdom_enriched_hits: usize,
}

/// Compute the structural evidence counts once, so the verdict classifier and
/// the assessment note builders share a single source of truth.
///
/// Mirrors the Ertl "secondary evidence" hierarchy (Ertl 2003, *J. Am. Chem.
/// Soc.* 125, 10353; Ertl & Schuppenhauer 2011): scaffold cores vs. decoration
/// side-chains, and the natural/synthetic/unknown source split with kingdom
/// taxonomy.
pub(crate) fn count_evidence(motifs: &[String], motif_hits: &[RdkitMotifHit]) -> EvidenceCounts {
    let mut counts = EvidenceCounts::default();

    for motif in motifs {
        if is_known_np_motif(motif) {
            counts.np_core_hits += 1;
        }
        if is_decoration_motif(motif) {
            counts.decoration_hits += 1;
        }
    }

    for hit in motif_hits {
        if is_scaffold_motif(&hit.label) {
            counts.scaffold_hits += 1;
        }
        match normalized_source_class(&hit.source_class) {
            "natural" => {
                counts.natural_hits += 1;
                if !hit.kingdoms.is_empty() {
                    counts.kingdom_enriched_hits += 1;
                }
            }
            "synthetic" => counts.synthetic_hits += 1,
            _ => counts.unknown_hits += 1,
        }
    }

    counts
}
