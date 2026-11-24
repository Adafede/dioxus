// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

use std::collections::HashMap;

use crate::mass::{exact_mass_from_formula_cached, exact_mass_from_smiles_cached};

#[test]
fn exact_mass_from_smiles_and_formula_pin_caffeine_blank_invalid_and_cache() {
    let mut smiles_cache = HashMap::new();
    let caffeine = "CN1C=NC2=C1C(=O)N(C(=O)N2C)C";
    let m1 = exact_mass_from_smiles_cached(&format!("  {caffeine}  "), &mut smiles_cache);
    let m2 = exact_mass_from_smiles_cached(caffeine, &mut smiles_cache);
    assert_eq!(m1, m2);
    let mass = m1.expect("caffeine exact mass");
    assert!((mass - 194.080_376).abs() < 1e-4, "got {mass}");

    assert_eq!(
        exact_mass_from_smiles_cached("   ", &mut smiles_cache),
        None
    );
    assert_eq!(
        exact_mass_from_smiles_cached("not_a_valid_smiles[", &mut smiles_cache),
        None
    );
    assert_eq!(
        exact_mass_from_smiles_cached("not_a_valid_smiles[", &mut smiles_cache),
        None
    );

    let mut formula_cache = HashMap::new();
    let f1 = exact_mass_from_formula_cached(" C8H10N4O2 ", &mut formula_cache);
    assert!((f1.expect("formula mass") - mass).abs() < 1e-6);
    assert_eq!(exact_mass_from_formula_cached("", &mut formula_cache), None);
    assert_eq!(
        exact_mass_from_formula_cached("Zz99", &mut formula_cache),
        None
    );
}
