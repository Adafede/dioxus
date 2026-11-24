// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

use std::collections::HashMap;

use chematic::smiles;

use crate::{ChemicalClass, LipidFamily, lmsd_classes};

#[test]
fn defaults_include_all_families_and_preserve_order() {
    let defaults = ChemicalClass::defaults();
    let names: Vec<_> = defaults.iter().map(|c| c.name.as_str()).collect();
    for expected in [
        "FA", "MUFA", "PUFA", "TG(AAA)", "DG(AA)", "MG(A)", "PC(AA)", "PE(AA)", "LPC(A)", "LPE(A)",
        "Cer(AS)", "SM(AS)", "ST", "PR", "SL", "PK",
    ] {
        assert!(names.contains(&expected), "missing default {expected}");
    }

    let fa = defaults.iter().find(|c| c.name == "FA").expect("FA class");
    let palmitic = smiles::parse("CCCCCCCCCCCCCCCC(=O)O").expect("valid SMILES");
    assert!(fa.matches(&palmitic));

    let map: HashMap<_, _> = defaults.iter().map(|c| (c.name.as_str(), c)).collect();
    assert_eq!(map.get("PC(AA)").map(|c| c.name.as_str()), Some("PC(AA)"));

    let gp: Vec<_> = defaults
        .iter()
        .filter(|c| c.family == LipidFamily::Glycerophospholipids)
        .map(|c| c.name.as_str())
        .collect();
    let inositol_pos = gp.iter().position(|&n| n == "PI(AA)").unwrap();
    let glycerol_pos = gp.iter().position(|&n| n == "PG(AA)").unwrap();
    assert!(inositol_pos < glycerol_pos);

    let mut seen = Vec::new();
    for c in &defaults {
        assert!(!c.name.is_empty() && !c.smarts.is_empty() && c.color.starts_with('#'));
        assert!(c.family.palette().contains(&c.color));
        if !seen.contains(&c.family) {
            seen.push(c.family);
        }
    }
    assert_eq!(seen, LipidFamily::ALL);
}

#[test]
fn all_86_lmsd_classes_compile_and_reject_non_lipid_decoys() {
    let classes = lmsd_classes();
    assert_eq!(classes.len(), 86, "Expected all 86 LMSD main classes");
    for (idx, class) in classes.iter().enumerate() {
        assert_eq!(class.index, idx);
        assert!(
            class.is_compiled(),
            "LMSD class {} failed to compile SMARTS: {}",
            class.name,
            class.smarts
        );
    }

    let decoys = [
        "CC1=C(C(=CC=C1)S(=O)(=O)O)C(=O)O",
        "C(C1C(C(C(C(O1)O)O)O)O)O",
        "CN1C=NC2=C1C(=O)N(C)C(=O)N2C",
        "CC1C(C(C(O1)OP(=O)(O)O)OP(=O)(O)O)N2C=NC3=C2N=CN=C3N",
        "C[N+](C)(C)CCO",
        "CCO",
        "CC(=O)O",
        "NCC(=O)O",
        "c1ccccc1C(=O)O",
        "OC(=O)CC(O)(CC(=O)O)C(=O)O",
        "NC(CO)C(=O)O",
        "NC(C(O)C)C(=O)O",
    ];
    for decoy_smiles in decoys {
        let mol = smiles::parse(decoy_smiles).expect("valid decoy SMILES");
        for class in classes {
            assert!(
                !class.matches(&mol),
                "Decoy {decoy_smiles} falsely matched LMSD class {}",
                class.name
            );
        }
    }
}

#[test]
fn lmsd_classes_match_all_eight_lipid_families() {
    let classes = lmsd_classes();
    let cases = [
        (LipidFamily::FattyAcyls, "CCCCCCCCCCCCCCCC(=O)O"),
        (
            LipidFamily::Glycerolipids,
            "CCCCCCCCCCCCCCCC(=O)OCC(O)COC(=O)CCCCCCCCCCCCCCC",
        ),
        (
            LipidFamily::Glycerophospholipids,
            "CCCCCCCCCCCCCCCC(=O)OCC(COP(=O)([O-])OCC[N+](C)(C)C)OC(=O)CCCCCCCCCCCCCCC",
        ),
        (
            LipidFamily::Sphingolipids,
            "CCCCCCCCCCCCCC=CC(O)C(CO)NC(=O)CCCCCCCCCCCCCCC",
        ),
        (
            LipidFamily::SterolLipids,
            "CC(C)CCCC(C)C1CCC2C1(CCC3C2CC=C4C3(CCC(C4)O)C)C",
        ),
        (LipidFamily::PrenolLipids, "CC(=CCCC(=CCCC(=CCO)C)C)C"),
        (
            LipidFamily::Saccharolipids,
            "CCCCCCCCCCCC(O)CC(=O)NC1C(O)C(O)C(CO)OC1O",
        ),
        (LipidFamily::Polyketides, "O=C1CC(c2ccccc2)Oc2ccccc12"),
    ];

    for (expected_family, smi) in cases {
        let mol = smiles::parse(smi).expect("valid lipid SMILES");
        assert!(
            classes
                .iter()
                .any(|c| c.family == expected_family && c.matches(&mol)),
            "Expected {smi} to match at least one class in {}",
            expected_family.name()
        );
    }
}
