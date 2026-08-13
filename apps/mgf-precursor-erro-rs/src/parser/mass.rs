// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Exact-mass lookup helpers: SMILES/formula → neutral mass (with
//! panic-tolerant parsing + per-call caching), and small numeric helpers
//! shared by the adduct, block and plotting layers.

use std::collections::{HashMap, HashSet};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::str::FromStr;

use mascot_rs::prelude::*;
use molecular_formulas::molecular_formula::MolecularFormula;
#[cfg(target_arch = "wasm32")]
use web_sys::console;

pub fn smiles_is_supported(smiles: &str) -> bool {
    !smiles.trim().is_empty()
}

#[must_use]
pub fn exact_mass_from_smiles(smiles: &str) -> Option<f64> {
    let mut cache = HashMap::new();
    let mut logged_failures = HashSet::new();
    exact_mass_from_smiles_cached(smiles, &mut cache, &mut logged_failures)
}

pub fn exact_mass_from_smiles_cached<S: std::hash::BuildHasher>(
    smiles: &str,
    cache: &mut HashMap<String, Option<f64>, S>,
    logged_failures: &mut HashSet<String, std::collections::hash_map::RandomState>,
) -> Option<f64> {
    let trimmed = smiles.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(mass) = cache.get(trimmed) {
        return *mass;
    }

    let mass = exact_mass_from_smiles_uncached(trimmed, logged_failures);
    cache.insert(trimmed.to_string(), mass);
    mass
}

fn exact_mass_from_smiles_uncached(
    smiles: &str,
    logged_failures: &mut HashSet<String, std::collections::hash_map::RandomState>,
) -> Option<f64> {
    if !smiles_is_supported(smiles) {
        let warning_key = format!("unsupported-smiles:{smiles}");
        if logged_failures.insert(warning_key) {
            #[cfg(target_arch = "wasm32")]
            console::warn_1(
                &format!("Skipping unsupported SMILES for mass parsing: {smiles}").into(),
            );
        }
        return None;
    }

    let parsed = catch_unwind(AssertUnwindSafe(|| {
        let parsed = Smiles::from_str(smiles).ok()?;
        let formula: ChemicalFormula<u32, i32> = ChemicalFormula::from(&parsed);
        Some(formula.isotopologue_mass())
    }));

    match parsed {
        Ok(mass) => {
            if mass.is_none() {
                let warning_key = format!("parse-failed-smiles:{smiles}");
                if logged_failures.insert(warning_key) {
                    #[cfg(target_arch = "wasm32")]
                    console::warn_1(&format!("SMILES parse failed for: {smiles}").into());
                }
            }
            mass
        }
        Err(panic) => {
            let _ = &panic;
            let warning_key = format!("panic-smiles:{smiles}");
            if logged_failures.insert(warning_key) {
                #[cfg(target_arch = "wasm32")]
                {
                    let panic_message = panic
                        .downcast_ref::<&str>()
                        .copied()
                        .or_else(|| panic.downcast_ref::<String>().map(String::as_str))
                        .unwrap_or("unknown panic");
                    console::warn_1(
                        &format!("SMILES parser panicked for: {smiles} ({panic_message})").into(),
                    );
                }
            }
            None
        }
    }
}

#[must_use]
pub fn exact_mass_from_formula(formula: &str) -> Option<f64> {
    let mut cache = HashMap::new();
    let mut logged_failures = HashSet::new();
    exact_mass_from_formula_cached(formula, &mut cache, &mut logged_failures)
}

pub fn exact_mass_from_formula_cached<S: std::hash::BuildHasher>(
    formula: &str,
    cache: &mut HashMap<String, Option<f64>, S>,
    logged_failures: &mut HashSet<String, std::collections::hash_map::RandomState>,
) -> Option<f64> {
    let trimmed = formula.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(mass) = cache.get(trimmed) {
        return *mass;
    }

    let parsed: std::result::Result<ChemicalFormula<u32, i32>, _> =
        ChemicalFormula::<u32, i32>::from_str(trimmed);
    let mass = parsed.map_or_else(
        |_| {
            let warning_key = format!("formula-parse-failed:{trimmed}");
            if logged_failures.insert(warning_key) {
                #[cfg(target_arch = "wasm32")]
                console::warn_1(&format!("Formula parse failed for: {trimmed}").into());
            }
            None
        },
        |parsed_formula| Some(parsed_formula.isotopologue_mass()),
    );
    cache.insert(trimmed.to_string(), mass);
    mass
}

#[must_use]
pub fn decimal_precision(value: &str) -> usize {
    let trimmed = value.trim();
    let Some((_, fractional)) = trimmed.split_once('.') else {
        return 0;
    };
    fractional.chars().take_while(char::is_ascii_digit).count()
}

#[must_use]
pub fn round_to_precision(value: f64, precision: usize) -> f64 {
    if precision == 0 {
        return value.round();
    }

    let precision = i32::try_from(precision).unwrap_or(i32::MAX);
    let factor = 10_f64.powi(precision);
    (value * factor).round() / factor
}
