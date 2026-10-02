// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

// The `tests` tests, extracted from `classify.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `classify` module, so
// `use super::*` below reaches exactly what it did before the move.
use super::*;

fn class_of(smiles: &str) -> Option<LipidClass> {
    classify_smiles(smiles).map(|c| c.class)
}

#[test]
fn fatty_acyls_are_recognized() {
    assert_eq!(
        class_of("CCCCCCCCCCCCCCCC(=O)O"),
        Some(LipidClass::FattyAcyl)
    );
    assert_eq!(
        class_of("CCCC=CC=CC=CC=CC=CC(=O)O"),
        Some(LipidClass::FattyAcyl)
    );
}

#[test]
fn esterified_fatty_acyl_is_glycerolipid() {
    assert_eq!(
        class_of("CCCCCCCCCCCCCCC(=O)OC"),
        Some(LipidClass::Glycerolipid)
    );
}

#[test]
fn sphingolipids_are_recognized() {
    assert_eq!(
        class_of("CCCCCCCCCCCCC=CC(C(CO)N)O"),
        Some(LipidClass::Sphingolipid)
    );
    assert_eq!(
        class_of("CCCCCCCCCCCCCCC(=O)N[C@H](CO)CCCCCCCCCC"),
        Some(LipidClass::Sphingolipid)
    );
}

#[test]
fn phospholipids_are_recognized() {
    assert_eq!(
        class_of("CCCCCCCCCC=CC=CC=CC=CC=CC(=O)OC(C)COP(=O)(O)OCC[N+](C)(C)C"),
        Some(LipidClass::Glycerophospholipid)
    );
}

#[test]
fn cofactors_and_metabolites_are_rejected() {
    assert_eq!(class_of("C(C1C(C(C(C(O1)O)O)O)O)O"), None); // glucose
    assert_eq!(class_of("CN1C=NC2=C1C(=O)N(C)C(=O)N2C"), None); // caffeine
    assert_eq!(
        class_of("CC1C(C(C(O1)OP(=O)(O)O)OP(=O)(O)O)N2C=NC3=C2N=CN=C3N"),
        None
    ); // ATP
    assert_eq!(class_of("C[N+](C)(C)CCO"), None); // choline
    assert_eq!(class_of("CC1=C(C(=CC=C1)S(=O)(=O)O)C(=O)O"), None); // aromatic sulfonic acid - should NOT be a lipid
    assert_eq!(
        class_of(
            "CC(C)(COP(=O)(O)OP(=O)(O)OCC1C(C(C(O1)N2C=NC3=C(N=CN=C32)N)O)OP(=O)(O)O)C(C(=O)NCCC(=O)NCCS)O"
        ),
        None
    ); // coenzyme A
}

#[test]
fn formula_fallback_classifies_fatty_acid() {
    assert_eq!(classify_formula("C16H32O2"), Some(LipidClass::FattyAcyl));
    assert_eq!(classify_formula("C18H36O2"), Some(LipidClass::FattyAcyl));
}

#[test]
fn formula_fallback_rejects_cholesterol() {
    // Sterols have 4-ring cores; formula-based classification cannot determine
    // ring structure, so sterol formulas are rejected to prevent false positives
    assert_eq!(classify_formula("C27H46O"), None);
}

#[test]
fn formula_fallback_rejects_cofactors() {
    assert_eq!(classify_formula("C10H15N5O9P2"), None); // ATP
    assert_eq!(classify_formula("C21H36N7O16P3S"), None); // coenzyme A
    assert_eq!(classify_formula("C6H12O6"), None); // glucose
    assert_eq!(classify_formula(""), None);
}

#[test]
fn counts_are_extracted_from_smiles() {
    let mol = smiles::parse("CCCCCCCCCCCCCCCC(=O)O").unwrap();
    let counts = counts_from_molecule(&mol);
    assert_eq!(counts.carbon, 16);
    assert_eq!(counts.hydrogen, 32);
    assert_eq!(counts.oxygen, 2);
    assert_eq!(counts.formula_string(), "C16H32O2");
}

// --- classify_from_counts edge-cases (via classify_formula) ---

#[test]
fn formula_fatty_acid_boundary_min_c() {
    // 7 carbons is the minimum for the FattyAcyl range check (7..=40)
    assert_eq!(classify_formula("C7H14O2"), Some(LipidClass::FattyAcyl));
}

#[test]
fn formula_fatty_acyl_boundary_max_c() {
    // 40 carbons is the inclusive upper bound
    assert_eq!(classify_formula("C40H80O2"), Some(LipidClass::FattyAcyl));
}

#[test]
fn formula_fatty_acid_rejects_below_min_c() {
    // 6 carbons — below the (>=7) threshold
    assert_eq!(classify_formula("C6H12O2"), None);
}

#[test]
fn formula_rejects_no_carbon() {
    // All oxygens, no carbon → heavy < 4 check fails
    assert_eq!(classify_formula("O2"), None);
}

#[test]
fn formula_rejects_too_few_heavy_atoms() {
    // Carbon present but heavy atoms total < 4 → rejected
    assert_eq!(classify_formula("CH4"), None);
}

#[test]
fn formula_glycerophospholipid_classified() {
    // P >= 1, C >= 15, O >= 6, O/C <= 0.6 → Glycerophospholipid
    // C=20, O=8 → O/C = 0.4 ≤ 0.6 ✓
    assert_eq!(
        classify_formula("C20H38O8P"),
        Some(LipidClass::Glycerophospholipid)
    );
}

#[test]
fn formula_glycerophospholipid_rejects_high_oc_ratio() {
    // Too many oxygens relative to carbons → excluded as ATP-like
    assert_eq!(classify_formula("C10H15N5O9P2"), None);
}

#[test]
fn formula_handles_invalid_input() {
    assert_eq!(classify_formula("not a formula"), None);
    assert_eq!(classify_formula("C"), None);
    assert_eq!(classify_formula("XYZ123"), None);
}

#[test]
#[cfg(any(test, target_arch = "wasm32"))]
fn classify_spectrum_falls_back_to_formula() {
    // SMILES absent → use formula
    let result = classify_spectrum(None, Some("C16H32O2")).unwrap();
    assert_eq!(result.class, LipidClass::FattyAcyl);
    assert!(!result.derived_from_smiles);
}

#[test]
#[cfg(any(test, target_arch = "wasm32"))]
fn classify_spectrum_empty_smiles_uses_formula() {
    let result = classify_spectrum(Some("  "), Some("C16H32O2"));
    assert!(result.is_some());
    assert_eq!(result.unwrap().class, LipidClass::FattyAcyl);
}

#[test]
#[cfg(any(test, target_arch = "wasm32"))]
fn classify_spectrum_no_smiles_no_formula_returns_none() {
    assert!(classify_spectrum(None, None).is_none());
}

#[test]
#[cfg(any(test, target_arch = "wasm32"))]
fn formula_counts_handles_halogen_mixtures() {
    // Halogens F, Cl, Br, I should all aggregate into `halogens`.
    let mut map = std::collections::HashMap::new();
    map.insert("C".to_string(), 10u32);
    map.insert("F".to_string(), 1u32);
    map.insert("Cl".to_string(), 2u32);
    map.insert("Br".to_string(), 1u32);
    map.insert("I".to_string(), 1u32);
    let counts = formula_counts(&map);
    assert_eq!(counts.carbon, 10);
    assert_eq!(counts.halogens, 5);
}

#[test]
#[cfg(any(test, target_arch = "wasm32"))]
fn formula_counts_ignores_unknown_elements() {
    let mut map = std::collections::HashMap::new();
    map.insert("C".to_string(), 5u32);
    map.insert("Fe".to_string(), 1u32);
    map.insert("Zn".to_string(), 1u32);
    let counts = formula_counts(&map);
    assert_eq!(counts.carbon, 5);
    assert_eq!(counts.halogens, 0);
    assert_eq!(counts.oxygen, 0);
}
