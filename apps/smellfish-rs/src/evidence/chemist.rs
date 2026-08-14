// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Chemist's-eye-view structural checks and motif-counting helpers.
//!
//! This is a *leaf* module: it depends only on `crate::model` and on nothing
//! in `assessment`/`verdict`, so the dependency graph stays acyclic
//! (assessment → verdict → chemist).

use crate::model::{ChemistCheck, MoleculeRow, RdkitMotifHit, normalized_source_class};

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
pub fn run_checks(row: &MoleculeRow) -> Vec<ChemistCheck> {
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
    let family = row.ring_family.to_ascii_lowercase();
    let is_polyaromatic = family.contains("polyaromatic");
    let is_np_skeleton = family.contains("polycyclic")
        || family.contains("steroid")
        || family.contains("sugar")
        || family.contains("macrolide")
        || family.contains("flavonoid")
        || family.contains("heteroaromatic");
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

/// Motif labels that are known to be enriched in natural products, based on
/// Ertl & Schuhmann (J. Nat. Prod. 2019, Vol. 82, 1258-1263,
/// DOI 10.1021/acs.jnatprod.8b01022)
/// and Wetzel et al. (CHIMIA 2007, DOI 10.2533/chimia.2007.355).
///
/// Used to highlight motifs that are characteristic of NP biosynthesis
/// in the UI.
#[must_use]
pub fn is_known_np_motif(label: &str) -> bool {
    let l = label.to_ascii_lowercase();
    // NP scaffold classes
    l.contains("steroid")
        || l.contains("sugar")
        || l.contains("macrolide")
        || l.contains("macrocycle")
        || l.contains("lactone")
        || l.contains("lactam")
        || l.contains("flavone")
        || l.contains("flavonoid")
        || l.contains("indole")
        || l.contains("quinoline")
        || l.contains("isoquinoline")
        || l.contains("benzofuran")
        || l.contains("benzothiophene")
        || l.contains("quinoxaline")
        || l.contains("purine")
        || l.contains("chromone")
        || l.contains("coumarin")
        || l.contains("tetrahydrofuran")
        || l.contains("tetrahydropyran")
        || l.contains("piperidine")
        || l.contains("piperazine")
        || l.contains("morpholine")
}

pub fn count_core_np_motifs(motifs: &[String]) -> usize {
    motifs.iter().filter(|m| is_known_np_motif(m)).count()
}

pub fn count_scaffold_hits(hits: &[RdkitMotifHit]) -> usize {
    hits.iter()
        .filter(|hit| is_scaffold_motif(&hit.label))
        .count()
}

pub fn count_source_hits(hits: &[RdkitMotifHit], source_class: &str) -> usize {
    hits.iter()
        .filter(|hit| normalized_source_class(&hit.source_class) == source_class)
        .count()
}

pub fn count_kingdom_enriched_hits(hits: &[RdkitMotifHit]) -> usize {
    hits.iter()
        .filter(|hit| {
            normalized_source_class(&hit.source_class) == "natural" && !hit.kingdoms.is_empty()
        })
        .count()
}

pub fn count_decoration_motifs(motifs: &[String]) -> usize {
    motifs.iter().filter(|m| is_decoration_motif(m)).count()
}

/// Scaffold motifs are ring systems or cores characteristic of natural-product
/// scaffold classes.
#[must_use]
pub fn is_scaffold_motif(label: &str) -> bool {
    let l = label.to_ascii_lowercase();
    l.contains("ring")
        || l.contains("steroid")
        || l.contains("sugar")
        || l.contains("macrocycle")
        || l.contains("macrolide")
        || l.contains("lactone")
        || l.contains("lactam")
        || l.contains("flavone")
        || l.contains("flavonoid")
        || l.contains("indole")
        || l.contains("quinoline")
        || l.contains("isoquinoline")
        || l.contains("benzofuran")
        || l.contains("benzothiophene")
        || l.contains("quinoxaline")
        || l.contains("purine")
        || l.contains("chromone")
        || l.contains("coumarin")
        || l.contains("morpholine")
        || l.contains("piperidine")
        || l.contains("piperazine")
        || l.contains("tetrahydrofuran")
        || l.contains("tetrahydropyran")
        || l.contains("cyclohexane")
}

/// Decoration motifs are functional groups or side-chain fragments.
pub fn is_decoration_motif(label: &str) -> bool {
    let l = label.to_ascii_lowercase();
    l.contains("aldehyde")
        || l.contains("ketone")
        || l.contains("carboxylic acid")
        || l.contains("ester")
        || l.contains("amide")
        || l.contains("carbamate")
        || l.contains("urea")
        || l.contains("sulfonamide")
        || l.contains("sulfone")
        || l.contains("sulfoxide")
        || l.contains("alcohol")
        || l.contains("phenol")
        || l.contains("ether")
        || l.contains("methoxy")
        || l.contains("amine")
        || l.contains("halogen")
        || l.contains("nitrile")
        || l.contains("nitro")
        || l.contains("thiol")
        || l.contains("phosphate")
        || l.contains("acetal")
        || l.contains("epoxide")
        || l.contains("allyl")
}

/// Per-molecule structural evidence counts, computed once from the Ertl motif
/// labels and the motif hits and reused by both the verdict classifier and the
/// assessment note builders.
///
/// Centralised in `chemist` (a cfg-free leaf module) so the verdict threshold
/// logic is unit-testable on native *without* the rdkit.js bridge: the original
/// smellfish smell was that `row_verdict` and `assess_np_evidence` each
/// re-derived the natural/synthetic/kingdom split independently.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EvidenceCounts {
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
pub fn count_evidence(motifs: &[String], motif_hits: &[RdkitMotifHit]) -> EvidenceCounts {
    EvidenceCounts {
        np_core_hits: count_core_np_motifs(motifs),
        scaffold_hits: count_scaffold_hits(motif_hits),
        decoration_hits: count_decoration_motifs(motifs),
        natural_hits: count_source_hits(motif_hits, "natural"),
        synthetic_hits: count_source_hits(motif_hits, "synthetic"),
        unknown_hits: count_source_hits(motif_hits, "unknown"),
        kingdom_enriched_hits: count_kingdom_enriched_hits(motif_hits),
    }
}
