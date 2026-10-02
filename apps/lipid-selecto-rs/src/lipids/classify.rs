// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Lipid classification algorithm.
//!
//! SMARTS substructure patterns and the `is_acyclic` / `classify_smiles` /
//! `classify_formula` / `classify_spectrum` entry points over molecules and
//! spectra.  Domain types come from [`super::types`].

#[cfg(any(test, target_arch = "wasm32"))]
use super::types::LipidClassification;
#[cfg(any(test, target_arch = "wasm32"))]
use super::types::{ElementCounts, LipidClass};
#[cfg(any(test, target_arch = "wasm32"))]
use chematic::chem;
#[cfg(any(test, target_arch = "wasm32"))]
use chematic::core::Molecule;
#[cfg(any(test, target_arch = "wasm32"))]
use chematic::smarts;
#[cfg(any(test, target_arch = "wasm32"))]
use chematic::smiles;
#[cfg(any(test, target_arch = "wasm32"))]
use std::sync::LazyLock;

// REAL LIPID SIGNATURES - only actual lipid backbone structures
// NO RINGS. Just the chemistry that defines each lipid class.

/// Phosphate group with characteristic [PX4](=O) bonding
#[cfg(any(test, target_arch = "wasm32"))]
const PATTERN_PHOSPHATE: &str = "[PX4](=[OX1])";

/// Choline headgroup: quaternary nitrogen
#[cfg(any(test, target_arch = "wasm32"))]
const PATTERN_CHOLINE: &str = "[NX4+]";

/// Ethanolamine headgroup: secondary/primary amine bonded to saturated carbon
#[cfg(any(test, target_arch = "wasm32"))]
const PATTERN_ETHANOLAMINE: &str = "[NX3][CX4]";

/// Triglyceride: one carbon with 3 ester groups (glycerol backbone)
#[cfg(any(test, target_arch = "wasm32"))]
const PATTERN_TRIGLYCERIDE: &str =
    "[CX4]([OX2][CX3](=[OX1])[#6])([OX2][CX3](=[OX1])[#6])[OX2][CX3](=[OX1])";

/// Diglyceride: carbon with 2 ester groups
#[cfg(any(test, target_arch = "wasm32"))]
const PATTERN_DIGLYCERIDE: &str = "[CX4]([OX2][CX3](=[OX1])[#6])[OX2][CX3](=[OX1])";

/// Monoglyceride: single ester linkage
#[cfg(any(test, target_arch = "wasm32"))]
const PATTERN_MONOGLYCERIDE: &str = "[CX4][OX2][CX3](=[OX1])[#6]";

/// Long aliphatic chain: 8+ saturated carbons NOT in rings (NO aromatic, NO rings)
/// Uses [!a] for not aromatic and [!R] for not in ring
#[cfg(any(test, target_arch = "wasm32"))]
const PATTERN_ALIPHATIC_CHAIN: &str =
    "[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]";

/// Amide linkage: C(=O)-N (found in ceramides)
#[cfg(any(test, target_arch = "wasm32"))]
const PATTERN_AMIDE: &str = "[CX3](=[OX1])[NX3]";

/// Amino group: N bonded to aliphatic carbon (sphinganine backbones)
#[cfg(any(test, target_arch = "wasm32"))]
const PATTERN_AMINO: &str = "[NX3][CH0,CH1,CH2,CH3]";

/// Fatty acid: carboxylic acid with non-aromatic, acyclic carbon
#[cfg(any(test, target_arch = "wasm32"))]
const PATTERN_CARBOXYLIC_ACID: &str = "[#6;!a;!R][CX3](=[OX1])[OH]";

/// Returns `true` when `molecule` contains at least one match for `pattern`.
#[cfg(any(test, target_arch = "wasm32"))]
fn has_substructure(molecule: &Molecule, pattern: &str) -> bool {
    let Ok(query) = smarts::parse_smarts(pattern) else {
        return false;
    };
    !smarts::find_matches(&query, molecule).is_empty()
}

/// Check if a molecule is acyclic (contains no rings).
///
/// Returns `true` if the molecule has no ring atoms (aromatic or alicyclic).
/// Used to reject aromatic rings, nucleotides, sugars, steroids, etc.
///
/// The `[R]` pattern is pre-compiled once via `LazyLock` rather than re-parsed
/// per call. Measured, that is tidy-up and not a win worth chasing further:
/// parsing all ten `PATTERN_*` strings 200 times costs ~0.5 ms, while the
/// 4,820 `classify_smiles` calls those parses would have accompanied cost
/// ~347 ms. `find_matches` is ~99.5% of the work, so the other patterns in
/// `has_substructure` still re-parsing on every call is not worth a cache — a
/// shared cache for them was built and measured at under 0.5% and discarded.
#[must_use]
#[cfg(any(test, target_arch = "wasm32"))]
pub(crate) fn is_acyclic(molecule: &Molecule) -> bool {
    static RING_QUERY: LazyLock<Option<smarts::QueryMolecule>> =
        LazyLock::new(|| smarts::parse_smarts("[R]").ok());
    let Some(query) = &*RING_QUERY else {
        return true; // If we can't parse, assume acyclic
    };
    smarts::find_matches(query, molecule).is_empty()
}

/// Extract elemental counts from a parsed molecule using chematic's own
/// descriptors (heavy atoms plus implicit/explicit hydrogens).
#[cfg(any(test, target_arch = "wasm32"))]
fn counts_from_molecule(molecule: &Molecule) -> ElementCounts {
    ElementCounts {
        carbon: chem::num_carbons(molecule) as u32,
        hydrogen: chem::num_hydrogens(molecule) as u32,
        nitrogen: chem::num_nitrogens(molecule) as u32,
        oxygen: chem::num_oxygens(molecule) as u32,
        phosphorus: chem::num_phosphorus(molecule) as u32,
        sulfur: chem::num_sulfurs(molecule) as u32,
        halogens: (chem::num_fluorines(molecule)
            + chem::num_chlorines(molecule)
            + chem::num_bromines(molecule)
            + chem::num_iodines(molecule)) as u32,
    }
}

/// Structural (SMILES-based) lipid classification.
/// Only matches REAL lipid backbones: long aliphatic chains + characteristic functional groups.
/// NO RINGS. NO AROMATICS. NO NUCLEOTIDES. NO STEROIDS.
#[cfg(any(test, target_arch = "wasm32"))]
fn classify_molecule(molecule: &Molecule) -> Option<(LipidClass, ElementCounts)> {
    let counts = counts_from_molecule(molecule);

    // === CRITICAL: Lipids are ACYCLIC ===
    // Reject any molecule with rings: nucleotides, sugars, steroids, aromatic rings
    if !is_acyclic(molecule) {
        return None;
    }

    // === PHOSPHOLIPIDS (PC/PE): phosphate + headgroup + aliphatic chain ===
    // Require all three to avoid matching ATP, CoA, nucleotides
    if has_substructure(molecule, PATTERN_PHOSPHATE)
        && has_substructure(molecule, PATTERN_ALIPHATIC_CHAIN)
    {
        if has_substructure(molecule, PATTERN_CHOLINE) {
            return Some((LipidClass::Glycerophospholipid, counts));
        }
        if has_substructure(molecule, PATTERN_ETHANOLAMINE) {
            return Some((LipidClass::Glycerophospholipid, counts));
        }
    }

    // === TRIGLYCERIDES: 3 ester groups on glycerol backbone ===
    if has_substructure(molecule, PATTERN_TRIGLYCERIDE) {
        return Some((LipidClass::Glycerolipid, counts));
    }

    // === DIGLYCERIDES: 2 ester groups on glycerol backbone ===
    if has_substructure(molecule, PATTERN_DIGLYCERIDE) {
        return Some((LipidClass::Glycerolipid, counts));
    }

    // === MONOGLYCERIDES: 1 ester group + long chain ===
    if has_substructure(molecule, PATTERN_MONOGLYCERIDE)
        && has_substructure(molecule, PATTERN_ALIPHATIC_CHAIN)
    {
        return Some((LipidClass::Glycerolipid, counts));
    }

    // === CERAMIDES & SPHINGANINES: amide or amino group + long aliphatic chain ===
    if has_substructure(molecule, PATTERN_ALIPHATIC_CHAIN) {
        // True ceramides: amide linkage
        if has_substructure(molecule, PATTERN_AMIDE) {
            return Some((LipidClass::Sphingolipid, counts));
        }
        // Sphinganine: amino group on aliphatic backbone
        if has_substructure(molecule, PATTERN_AMINO) {
            return Some((LipidClass::Sphingolipid, counts));
        }
    }

    // === FATTY ACIDS: carboxylic acid + long aliphatic chain ===
    if has_substructure(molecule, PATTERN_CARBOXYLIC_ACID)
        && has_substructure(molecule, PATTERN_ALIPHATIC_CHAIN)
    {
        return Some((LipidClass::FattyAcyl, counts));
    }

    None
}

/// Classify a single SMILES string.
///
/// Returns the classification (with formula and exact mass) when the molecule
/// is recognized as a lipid, otherwise `None`.
#[must_use]
#[cfg(any(test, target_arch = "wasm32"))]
pub(crate) fn classify_smiles(smiles: &str) -> Option<LipidClassification> {
    let trimmed = smiles.trim();
    if trimmed.is_empty() {
        return None;
    }

    // SMILES parsers can panic on pathological input; isolate the call so a
    // malformed SMILES falls back to the formula classifier instead of crashing.
    let molecule =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| smiles::parse(trimmed)))
            .ok()
            .and_then(Result::ok)?;

    let (class, _counts) = classify_molecule(&molecule)?;

    Some(LipidClassification {
        class,
        derived_from_smiles: true,
    })
}

/// Formula-based classification decision from pre-computed element counts.
///
/// Shared by [`classify_formula`] (public API) and [`classify_spectrum`]
/// (which already parsed the formula) to avoid duplicating the decision tree
/// or re-parsing the formula string.
#[cfg(any(test, target_arch = "wasm32"))]
fn classify_from_counts(counts: &ElementCounts) -> Option<LipidClass> {
    let c = counts.carbon as i32;
    let h = counts.hydrogen as i32;
    let n = counts.nitrogen as i32;
    let p = counts.phosphorus as i32;
    let o = counts.oxygen as i32;
    let s = counts.sulfur as i32;
    let halogens = counts.halogens as i32;
    let db = counts.double_bond_equivalent();
    let heavy = c + n + o + p + s + halogens;

    if c == 0 || heavy < 4 {
        return None;
    }

    // Free fatty acid: RCOOH — exactly two oxygens, no P/N/S/halogen.
    if p == 0
        && n == 0
        && s == 0
        && halogens == 0
        && o == 2
        && (7..=40).contains(&c)
        && (1.0..=14.0).contains(&db)
        && h <= 2 * c + 2
        && h >= 2 * c - 28
    {
        return Some(LipidClass::FattyAcyl);
    }

    // NOTE: Do NOT classify sterols by formula alone - sterols have 4-ring cores
    // and are definitionally cyclic. Formula-based classification cannot determine
    // ring structure, so we reject all sterol-like formulas to be safe.

    // Glycerolipid: ester-rich, oxygen-rich but low O/C ratio (excludes sugars).
    if p == 0
        && n == 0
        && (3..=6).contains(&o)
        && c >= 8
        && f64::from(o) / f64::from(c) <= 0.25
        && db >= 1.0
        && h <= 2 * c + 2
        && h >= 2 * c - 28
    {
        return Some(LipidClass::Glycerolipid);
    }

    // Glycerophospholipid: phosphorus present with a substantial, oxygen-moderate
    // backbone (excludes ATP / coenzyme A which have very high O/C ratios).
    if p >= 1 && c >= 15 && o >= 6 && f64::from(o) / f64::from(c) <= 0.6 {
        return Some(LipidClass::Glycerophospholipid);
    }

    None
}

/// Conservative formula-only fallback (used when a SMILES is absent/unparseable
/// but a Hill-notation `FORMULA=` is available).
#[must_use]
#[cfg(test)]
pub(crate) fn classify_formula(formula: &str) -> Option<LipidClass> {
    let trimmed = formula.trim();
    if trimmed.is_empty() {
        return None;
    }

    let map = chem::parse_formula(trimmed).ok()?;
    let counts = formula_counts(&map);
    classify_from_counts(&counts)
}

/// Top-level entry point used by the MGF parser.
///
/// Tries the SMILES first (structural classification) and falls back to the
/// `FORMULA=` value when the SMILES is missing or cannot be parsed.
#[must_use]
#[cfg(any(test, target_arch = "wasm32"))]
pub(crate) fn classify_spectrum(
    smiles: Option<&str>,
    formula: Option<&str>,
) -> Option<LipidClassification> {
    if let Some(smiles) = smiles.filter(|value| !value.trim().is_empty())
        && let Some(classification) = classify_smiles(smiles)
    {
        return Some(classification);
    }

    let formula_str = formula.filter(|value| !value.trim().is_empty())?;
    let trimmed = formula_str.trim();
    let map = chem::parse_formula(trimmed).ok()?;
    let counts = formula_counts(&map);
    let class = classify_from_counts(&counts)?;
    Some(LipidClassification {
        class,
        derived_from_smiles: false,
    })
}

#[cfg(any(test, target_arch = "wasm32"))]
fn formula_counts(map: &std::collections::HashMap<String, u32>) -> ElementCounts {
    let mut counts = ElementCounts::default();
    let mut halogens = 0u32;
    for (symbol, value) in map {
        match symbol.as_str() {
            "C" => counts.carbon = *value,
            "H" => counts.hydrogen = *value,
            "N" => counts.nitrogen = *value,
            "O" => counts.oxygen = *value,
            "P" => counts.phosphorus = *value,
            "S" => counts.sulfur = *value,
            "F" | "Cl" | "Br" | "I" => halogens += *value,
            _ => {}
        }
    }
    counts.halogens = halogens;
    counts
}

#[cfg(test)]
#[expect(clippy::unwrap_used)] // tests unwrap fixtures/known modes to fail-fast on regression
mod tests;
