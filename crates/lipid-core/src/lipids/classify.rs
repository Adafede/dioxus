// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Broad lipid classification algorithm with precompiled SMARTS queries.

use std::collections::HashMap;
use std::sync::LazyLock;

use chematic::chem;
use chematic::core::Molecule;
use chematic::smarts::{self, QueryMolecule};

use super::types::ElementCounts;
#[cfg(test)]
use super::types::LipidClassification;
use crate::family::LipidFamily;

macro_rules! lazy_smarts {
    ($name:ident, $pat:literal) => {
        static $name: LazyLock<Option<QueryMolecule>> =
            LazyLock::new(|| smarts::parse_smarts($pat).ok());
    };
}

lazy_smarts!(Q_RING, "[R]");
lazy_smarts!(Q_PHOSPHATE, "[PX4](=[OX1])([OX2])[OX2]");
lazy_smarts!(
    Q_GLYCEROPHOSPHATE,
    "[CX4][CHX4]([OX2])[CH2X4][OX2][PX4](=[OX1])([OX2])[OX2]"
);
lazy_smarts!(Q_CHOLINE, "[NX4+]");
lazy_smarts!(Q_ETHANOLAMINE, "[NX3][CX4]");
lazy_smarts!(
    Q_TRIGLYCERIDE,
    "[CH2X4]([OX2][CX3](=[OX1])[#6])[CHX4]([OX2][CX3](=[OX1])[#6])[CH2X4][OX2][CX3](=[OX1])[#6]"
);
lazy_smarts!(
    Q_DIGLYCERIDE_12,
    "[CH2X4]([OX2][CX3](=[OX1])[#6])[CHX4]([OX2][CX3](=[OX1])[#6])[CH2X4][OX2H]"
);
lazy_smarts!(
    Q_DIGLYCERIDE_13,
    "[CH2X4]([OX2][CX3](=[OX1])[#6])[CHX4]([OX2H])[CH2X4][OX2][CX3](=[OX1])[#6]"
);
lazy_smarts!(
    Q_MONOGLYCERIDE_1,
    "[CH2X4]([OX2][CX3](=[OX1])[#6])[CHX4]([OX2H])[CH2X4][OX2H]"
);
lazy_smarts!(
    Q_MONOGLYCERIDE_2,
    "[CH2X4]([OX2H])[CHX4]([OX2][CX3](=[OX1])[#6])[CH2X4][OX2H]"
);
lazy_smarts!(
    Q_ALIPHATIC_CHAIN,
    "[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]"
);
lazy_smarts!(Q_SPHINGOID, "[CH2X4]([OX2])[CHX4]([NX3])[CX4]");
lazy_smarts!(Q_CARBOXYLIC_ACID, "[#6;!a;!R][CX3](=[OX1])[OX2H1,OX1-]");

fn has_query(molecule: &Molecule, query: &LazyLock<Option<QueryMolecule>>) -> bool {
    let Some(parsed) = query.as_ref() else {
        return false;
    };
    smarts::has_match_with_config(parsed, molecule, &smarts::MatchConfig::default())
}

/// Returns `true` when `molecule` contains no ring atoms.
#[must_use]
pub(crate) fn is_acyclic(molecule: &Molecule) -> bool {
    let Some(query) = Q_RING.as_ref() else {
        return true;
    };
    !smarts::has_match_with_config(query, molecule, &smarts::MatchConfig::default())
}

/// Structural broad lipid classification for an already-confirmed acyclic molecule.
#[must_use]
pub(crate) fn classify_acyclic_molecule(molecule: &Molecule) -> Option<LipidFamily> {
    let mut chain_cached: Option<bool> = None;
    let mut has_chain =
        || *chain_cached.get_or_insert_with(|| has_query(molecule, &Q_ALIPHATIC_CHAIN));

    if has_query(molecule, &Q_PHOSPHATE)
        && has_query(molecule, &Q_GLYCEROPHOSPHATE)
        && has_chain()
        && (has_query(molecule, &Q_CHOLINE) || has_query(molecule, &Q_ETHANOLAMINE))
    {
        return Some(LipidFamily::Glycerophospholipids);
    }
    if has_query(molecule, &Q_TRIGLYCERIDE)
        || has_query(molecule, &Q_DIGLYCERIDE_12)
        || has_query(molecule, &Q_DIGLYCERIDE_13)
    {
        return Some(LipidFamily::Glycerolipids);
    }
    if has_chain()
        && (has_query(molecule, &Q_MONOGLYCERIDE_1) || has_query(molecule, &Q_MONOGLYCERIDE_2))
    {
        return Some(LipidFamily::Glycerolipids);
    }
    if has_chain() && has_query(molecule, &Q_SPHINGOID) {
        return Some(LipidFamily::Sphingolipids);
    }
    if has_query(molecule, &Q_CARBOXYLIC_ACID) && has_chain() {
        return Some(LipidFamily::FattyAcyls);
    }
    None
}

#[cfg(test)]
pub(crate) fn classify_smiles(smiles: &str) -> Option<LipidClassification> {
    let trimmed = smiles.trim();
    if trimmed.is_empty() {
        return None;
    }
    let molecule = chem_depict::parse_smiles_safe(trimmed).ok()?;
    let family = is_acyclic(&molecule)
        .then(|| classify_acyclic_molecule(&molecule))
        .flatten()?;
    Some(LipidClassification {
        family,
        derived_from_smiles: true,
    })
}

fn classify_from_counts(counts: &ElementCounts) -> Option<LipidFamily> {
    let carbon = counts.carbon;
    let hydrogen = counts.hydrogen;
    let nitrogen = counts.nitrogen;
    let phosphorus = counts.phosphorus;
    let oxygen = counts.oxygen;
    let sulfur = counts.sulfur;
    let halogens = counts.halogens;
    let db = counts.double_bond_equivalent();
    let heavy = carbon + nitrogen + oxygen + phosphorus + sulfur + halogens;

    if carbon == 0 || heavy < 4 {
        return None;
    }
    let h_within_bounds = hydrogen > carbon
        && hydrogen <= 2 * carbon + 2
        && hydrogen >= (2 * carbon).saturating_sub(28);
    if phosphorus == 0
        && nitrogen == 0
        && sulfur == 0
        && halogens == 0
        && oxygen == 2
        && (7..=40).contains(&carbon)
        && (1.0..=14.0).contains(&db)
        && h_within_bounds
    {
        return Some(LipidFamily::FattyAcyls);
    }
    if phosphorus == 0
        && nitrogen == 0
        && sulfur == 0
        && halogens == 0
        && (3..=6).contains(&oxygen)
        && carbon >= 8
        && f64::from(oxygen) / f64::from(carbon) <= 0.25
        && db >= 1.0
        && h_within_bounds
    {
        return Some(LipidFamily::Glycerolipids);
    }
    if phosphorus >= 1
        && sulfur == 0
        && halogens == 0
        && nitrogen <= 2
        && carbon >= 15
        && oxygen >= 6
        && f64::from(oxygen) / f64::from(carbon) <= 0.6
        && h_within_bounds
    {
        return Some(LipidFamily::Glycerophospholipids);
    }
    None
}

/// Conservative formula-only fallback classifier when SMILES is missing or invalid.
#[must_use]
pub(crate) fn classify_formula(formula: &str) -> Option<LipidFamily> {
    let trimmed = formula.trim();
    if trimmed.is_empty() {
        return None;
    }
    let map = chem::parse_formula(trimmed).ok()?;
    let counts = formula_counts(&map);
    classify_from_counts(&counts)
}

pub(crate) fn formula_counts(
    map: &HashMap<String, u32, impl std::hash::BuildHasher>,
) -> ElementCounts {
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
