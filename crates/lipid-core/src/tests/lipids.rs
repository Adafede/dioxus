// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

use std::collections::HashMap;

use crate::LipidFamily;
use crate::lipids::classify::{classify_formula, classify_smiles, formula_counts};

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
        class_of("CCCCCCCCCCCCCCCC(=O)OCC(O)CO"),
        Some(LipidFamily::Glycerolipids)
    );
    assert_ne!(
        class_of("CCCCCCCCCCCCCCC(=O)OC"),
        Some(LipidFamily::Glycerolipids),
        "methyl palmitate is a fatty ester, not a glycerolipid"
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
fn spectrum_formula_fallback_and_missing_annotations_work_as_expected() {
    let classes = crate::lmsd_classes();
    let mut from_none = crate::parser::block::SpectrumBlock::new(1);
    from_none.formula = Some("C16H32O2".to_string());
    from_none.classify_and_match(classes);
    let cls = from_none.classification.as_ref().unwrap();
    assert_eq!(cls.family, LipidFamily::FattyAcyls);
    assert!(!cls.derived_from_smiles);

    let mut from_blank = crate::parser::block::SpectrumBlock::new(2);
    from_blank.psm_smiles = Some("  ".to_string());
    from_blank.formula = Some("C16H32O2".to_string());
    from_blank.classify_and_match(classes);
    assert_eq!(
        from_blank.classification.as_ref().map(|c| c.family),
        Some(LipidFamily::FattyAcyls)
    );

    let mut neither = crate::parser::block::SpectrumBlock::new(3);
    neither.classify_and_match(classes);
    assert!(neither.classification.is_none());

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

    // Non-lipid aromatic acid must not fall back to FORMULA= when valid SMILES matched 0 classes.
    let mut benzoic = crate::parser::block::SpectrumBlock::new(1);
    benzoic.psm_smiles = Some("c1ccccc1C(=O)O".to_string());
    benzoic.formula = Some("C7H6O2".to_string());
    benzoic.classify_and_match(crate::lmsd_classes());
    assert!(!benzoic.is_lipid());
    assert!(benzoic.classification.is_none());

    // Double-bond equivalents include phosphorus: C20H38O8P -> 20 - 19 + 0.5 + 1 = 2.5
    let gp = formula_counts(&HashMap::from([
        ("C".to_string(), 20),
        ("H".to_string(), 38),
        ("O".to_string(), 8),
        ("P".to_string(), 1),
    ]));
    assert!((gp.double_bond_equivalent() - 2.5).abs() < 1e-12);
    assert_eq!(classify_formula("C7H6O2"), None);
}

#[test]
fn lipid_family_and_class_match_set_invariants() {
    use crate::ClassMatchSet;
    for (rank, fam) in LipidFamily::ALL.iter().enumerate() {
        assert_eq!(fam.rank(), rank, "rank matches index");
        assert_eq!(fam.code().len(), 2, "two-letter family code");
        assert!(!fam.name().is_empty(), "non-empty display name");
        assert!(
            fam.category().contains(fam.code()),
            "category contains family code"
        );
        assert!(
            fam.fallback_code().contains(fam.code()),
            "fallback code contains family code"
        );
        assert_eq!(fam.palette().len(), 5, "five-shade palette");
    }

    assert!(ClassMatchSet::empty().is_empty(), "empty set is empty");
    assert_eq!(ClassMatchSet::empty().len(), 0, "empty set length is 0");
    assert_eq!(ClassMatchSet::all(0).len(), 0, "all(0) is empty");
    assert_eq!(ClassMatchSet::all(86).len(), 86, "all(86) sets 86 bits");
    assert_eq!(
        ClassMatchSet::all(200).len(),
        128,
        "all(200) clamps at 128 bits"
    );

    let mut mask = ClassMatchSet::empty();
    mask.insert(3);
    mask.insert(130);
    assert!(
        mask.contains(3) && !mask.contains(130),
        "out-of-range insert is ignored"
    );
    mask.toggle(3);
    assert!(!mask.contains(3), "toggle clears present bit");
    mask.toggle(5);
    assert!(mask.contains(5), "toggle sets absent bit");
    mask.remove(5);
    assert!(mask.is_empty(), "remove clears last bit");
    assert!(
        !ClassMatchSet::from_indices([1, 2]).intersects(ClassMatchSet::from_indices([3, 4])),
        "disjoint sets do not intersect"
    );
    assert!(
        ClassMatchSet::from_indices([1, 2]).intersects(ClassMatchSet::from_indices([2, 4])),
        "overlapping sets intersect"
    );
}
