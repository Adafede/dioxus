// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Exact neutral monoisotopic mass derivation from SMILES and chemical formulas.

use std::collections::HashMap;
use std::hash::BuildHasher;
use std::str::FromStr;

use chematic::chem;
use molecular_formulas::prelude::*;

/// Computes and caches the neutral isotopologue mass derived from a SMILES string.
pub fn exact_mass_from_smiles_cached<S: BuildHasher>(
    smiles: &str,
    cache: &mut HashMap<String, Option<f64>, S>,
) -> Option<f64> {
    let trimmed = smiles.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(mass) = cache.get(trimmed) {
        return *mass;
    }

    let mass = exact_mass_from_smiles_uncached(trimmed);
    cache.insert(trimmed.to_string(), mass);
    mass
}

fn exact_mass_from_smiles_uncached(smiles: &str) -> Option<f64> {
    let mol = chem_depict::parse_smiles_safe(smiles).ok()?;
    let formula_str = chem::calc_mol_formula(&mol);
    let formula: ChemicalFormula<u32, i32> = ChemicalFormula::from_str(&formula_str).ok()?;
    Some(formula.isotopologue_mass())
}

/// Computes and caches the neutral isotopologue mass derived from a formula string.
pub fn exact_mass_from_formula_cached<S: BuildHasher>(
    formula: &str,
    cache: &mut HashMap<String, Option<f64>, S>,
) -> Option<f64> {
    let trimmed = formula.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(mass) = cache.get(trimmed) {
        return *mass;
    }

    let mass = ChemicalFormula::<u32, i32>::from_str(trimmed)
        .ok()
        .map(|parsed| parsed.isotopologue_mass());
    cache.insert(trimmed.to_string(), mass);
    mass
}
