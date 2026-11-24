// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

use std::collections::HashMap;

use chematic::smiles;

use crate::LipidFamily;
use crate::lipids::classify::{
    classify_formula, classify_smiles, classify_spectrum, counts_from_molecule, formula_counts,
};

fn class_of(smiles: &str) -> Option<LipidFamily> {
    classify_smiles(smiles).map(|c| c.family)
}

#[test]
fn acyclic_lipids_are_recognized() {
    assert_eq!(
        class_of("CCCCCCCCCCCCCCCC(=O)O"),
        Some(LipidFamily::FattyAcyls)
    );
    assert_eq!(
        class_of("CCCC=CC=CC=CC=CC=CC(=O)O"),
        Some(LipidFamily::FattyAcyls)
    );
    assert_eq!(
        class_of("CCCCCCCCCCCCCCC(=O)OC"),
        Some(LipidFamily::Glycerolipids)
    );
    assert_eq!(
        class_of("CCCCCCCCCCCCC=CC(C(CO)N)O"),
        Some(LipidFamily::Sphingolipids)
    );
    assert_eq!(
        class_of("CCCCCCCCCCCCCCC(=O)N[C@H](CO)CCCCCCCCCC"),
        Some(LipidFamily::Sphingolipids)
    );
    assert_eq!(
        class_of("CCCCCCCCCC=CC=CC=CC=CC=CC(=O)OC(C)COP(=O)(O)OCC[N+](C)(C)C"),
        Some(LipidFamily::Glycerophospholipids)
    );
}

#[test]
fn cofactors_and_metabolites_are_rejected() {
    for smi in [
        "C(C1C(C(C(C(O1)O)O)O)O)O",
        "CN1C=NC2=C1C(=O)N(C)C(=O)N2C",
        "CC1C(C(C(O1)OP(=O)(O)O)OP(=O)(O)O)N2C=NC3=C2N=CN=C3N",
        "C[N+](C)(C)CCO",
        "CC1=C(C(=CC=C1)S(=O)(=O)O)C(=O)O",
        "CC(C)(COP(=O)(O)OP(=O)(O)OCC1C(C(C(O1)N2C=NC3=C(N=CN=C32)N)O)OP(=O)(O)O)C(C(=O)NCCC(=O)NCCS)O",
    ] {
        assert_eq!(class_of(smi), None, "expected non-lipid for {smi}");
    }
}

#[test]
fn formula_fallback_matches_and_rejects_boundaries() {
    assert_eq!(classify_formula("C16H32O2"), Some(LipidFamily::FattyAcyls));
    assert_eq!(classify_formula("C18H36O2"), Some(LipidFamily::FattyAcyls));
    assert_eq!(classify_formula("C7H14O2"), Some(LipidFamily::FattyAcyls));
    assert_eq!(classify_formula("C40H80O2"), Some(LipidFamily::FattyAcyls));
    assert_eq!(classify_formula("C6H12O2"), None);
    assert_eq!(classify_formula("O2"), None);
    assert_eq!(classify_formula("CH4"), None);
    assert_eq!(
        classify_formula("C20H38O8P"),
        Some(LipidFamily::Glycerophospholipids)
    );
    assert_eq!(
        classify_formula("C39H74O6"),
        Some(LipidFamily::Glycerolipids)
    );
    assert_eq!(classify_formula("C27H46O"), None);
    assert_eq!(classify_formula("C10H15N5O9P2"), None);
    assert_eq!(classify_formula("C21H36N7O16P3S"), None);
    assert_eq!(classify_formula("C6H12O6"), None);
    assert_eq!(classify_formula(""), None);
    assert_eq!(classify_formula("not a formula"), None);
    assert_eq!(classify_formula("C"), None);
    assert_eq!(classify_formula("XYZ123"), None);
}

#[test]
fn counts_and_spectrum_fallback_work_as_expected() {
    let mol = smiles::parse("CCCCCCCCCCCCCCCC(=O)O").unwrap();
    let counts = counts_from_molecule(&mol);
    assert_eq!((counts.carbon, counts.hydrogen, counts.oxygen), (16, 32, 2));
    assert_eq!(counts.formula_string(), "C16H32O2");

    let from_none = classify_spectrum(None, Some("C16H32O2")).unwrap();
    assert_eq!(from_none.family, LipidFamily::FattyAcyls);
    assert!(!from_none.derived_from_smiles);

    let from_blank = classify_spectrum(Some("  "), Some("C16H32O2")).unwrap();
    assert_eq!(from_blank.family, LipidFamily::FattyAcyls);
    assert!(classify_spectrum(None, None).is_none());

    let mut map = HashMap::from([
        ("C".to_string(), 10),
        ("F".to_string(), 1),
        ("Cl".to_string(), 2),
        ("Br".to_string(), 1),
        ("I".to_string(), 1),
        ("Fe".to_string(), 1),
    ]);
    let hal = formula_counts(&map);
    assert_eq!((hal.carbon, hal.halogens, hal.oxygen), (10, 5, 0));
    map.clear();
    assert_eq!(formula_counts(&map).carbon, 0);
}
