// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

use chem_depict::parse_smiles_safe;

use crate::{LipidFamily, lmsd_classes};

#[test]
fn lmsd_classes_cover_all_families_with_valid_palettes_in_order() {
    let classes = lmsd_classes();
    let mut seen = Vec::new();
    for c in classes {
        assert!(!c.name.is_empty() && !c.smarts.is_empty() && c.color.starts_with('#'));
        assert!(c.family.palette().contains(&c.color));
        if seen.last() != Some(&c.family) {
            assert!(!seen.contains(&c.family), "non-contiguous family order");
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
        let mol = parse_smiles_safe(decoy_smiles).expect("valid decoy SMILES");
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
        let mol = parse_smiles_safe(smi).expect("valid lipid SMILES");
        assert!(
            classes
                .iter()
                .any(|c| c.family == expected_family && c.matches(&mol)),
            "Expected {smi} to match at least one class in {}",
            expected_family.name()
        );
    }

    // Verify LMSD rules distinguish real lipids from structurally adjacent decoys per family.
    for (family, real_smi, decoy_smi) in [
        (
            LipidFamily::Glycerolipids,
            "CC(=O)OCC(OC(=O)C)COC(=O)C",
            "CC(=O)OC(OC(=O)C)(OC(=O)C)C",
        ),
        (
            LipidFamily::Glycerophospholipids,
            "CC(=O)OCC(OC(=O)C)COP(=O)([O-])OCC[N+](C)(C)C",
            "CP(=O)(O)(O)[N+](C)(C)C",
        ),
        (
            LipidFamily::SterolLipids,
            "CC12CCC3C(C1CCC2O)CCC4=CC(=O)CCC34C",
            "C1CCC2C(C1)CCC2(C)C",
        ),
    ] {
        let real_mol = parse_smiles_safe(real_smi).unwrap();
        assert!(
            classes
                .iter()
                .any(|c| c.family == family && c.matches(&real_mol)),
            "real {real_smi} failed to match {}",
            family.name()
        );
        let decoy_mol = parse_smiles_safe(decoy_smi).unwrap();
        assert!(
            !classes
                .iter()
                .any(|c| c.family == family && c.matches(&decoy_mol)),
            "adjacent decoy {decoy_smi} falsely matched {}",
            family.name()
        );
    }
}
