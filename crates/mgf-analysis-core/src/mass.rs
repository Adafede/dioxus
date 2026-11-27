// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Exact neutral monoisotopic mass derivation from SMILES and chemical formulas.

use std::collections::HashMap;
use std::hash::BuildHasher;
use std::str::FromStr;

use chematic::chem;
use molecular_formulas::prelude::*;

fn cached_mass_lookup<S: BuildHasher>(
    raw: &str,
    cache: &mut HashMap<String, Option<f64>, S>,
    compute: fn(&str) -> Option<f64>,
) -> Option<f64> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(mass) = cache.get(trimmed) {
        return *mass;
    }
    let mass = compute(trimmed);
    cache.insert(trimmed.to_string(), mass);
    mass
}

/// Computes and caches the neutral isotopologue mass derived from a SMILES string.
pub(crate) fn exact_mass_from_smiles_cached<S: BuildHasher>(
    smiles: &str,
    cache: &mut HashMap<String, Option<f64>, S>,
) -> Option<f64> {
    cached_mass_lookup(smiles, cache, exact_mass_from_smiles_uncached)
}

fn exact_mass_from_smiles_uncached(smiles: &str) -> Option<f64> {
    let mol = chem_depict::parse_smiles_safe(smiles).ok()?;
    let formula_str = chem::calc_mol_formula(&mol);
    let formula: ChemicalFormula<u32, i32> = ChemicalFormula::from_str(&formula_str).ok()?;
    Some(formula.isotopologue_mass())
}

fn exact_mass_from_formula_uncached(formula: &str) -> Option<f64> {
    ChemicalFormula::<u32, i32>::from_str(formula)
        .ok()
        .map(|parsed| parsed.isotopologue_mass())
}

/// Computes and caches the neutral isotopologue mass derived from a formula string.
pub(crate) fn exact_mass_from_formula_cached<S: BuildHasher>(
    formula: &str,
    cache: &mut HashMap<String, Option<f64>, S>,
) -> Option<f64> {
    cached_mass_lookup(formula, cache, exact_mass_from_formula_uncached)
}
