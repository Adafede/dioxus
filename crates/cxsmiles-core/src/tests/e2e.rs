// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

use chematic::smiles::{canonical_smiles, write};

use super::{assert_roundtrip, lines, mol};
use crate::error::CxError;
use crate::parse::parse_list;
use crate::pipeline::{generate, generate_with_options};
use crate::positional::build_positional_with_options;
use crate::repeating::build_repeating;
use crate::types::{ConstructKind, CxOptions};

#[test]
fn parse_canonical_write_roundtrip() {
    let m = mol("Clc1ccccc1-c2ccccc2");
    assert_eq!(canonical_smiles(&m), "c1(-c2ccccc2Cl)ccccc1");
    assert_eq!(write(&m), "Clc1ccccc1-c2ccccc2");
}

#[test]
fn empty_and_singleton_fast_paths() {
    assert_eq!(generate(&[]), Err(CxError::EmptyInput));
    assert_eq!(
        generate(&["   ".to_string(), "\n".to_string()]),
        Err(CxError::EmptyInput)
    );

    let single = generate(&["  OCC ".to_string()]).expect("valid singleton SMILES");
    assert_eq!(single.construct, ConstructKind::BestEffort);
    assert_eq!(single.cx_smiles, "C(C)O");
    assert_eq!(single.scaffold_smiles, "C(C)O");
    assert_eq!(single.floating, []);
    assert!(single.repeat_unit.is_none());
    assert_eq!(single.confidence.coverage.covered, 1);
    assert_eq!(single.confidence.coverage.total, 1);
    assert!(single.confidence.clean);
    assert_eq!(single.enumerated, vec!["C(C)O"]);
}

#[test]
fn positional_biphenyl_cl_and_omf_and_acetyl_double_m() {
    let cl_in = "Clc1ccccc1-c2ccccc2\nClc1cccc(-c2ccccc2)c1\nClc1ccc(-c2ccccc2)cc1";
    let cl_mols = parse_list(&lines(cl_in)).unwrap();
    let cl = generate(&lines(cl_in)).unwrap();
    assert_eq!(cl.construct, ConstructKind::Positional);
    assert_roundtrip(&cl, &cl_mols);
    assert!(cl.confidence.clean);
    assert_eq!(cl.scaffold_smiles, "c1ccccc1-c2ccccc2");
    assert_eq!(cl.cx_smiles, "c1ccccc1-c2ccccc2.[*]Cl |m:12:0.2.3|");
    assert_eq!(cl.floating.len(), 1);
    assert_eq!(cl.floating[0].equiv.len(), 3);

    let omf_in = "COc1c(O)cc(O)cc1\nOc1c(OC)cc(O)cc1\nOc1c(O)cc(OC)cc1";
    let omf_mols = parse_list(&lines(omf_in)).unwrap();
    let omf = generate(&lines(omf_in)).unwrap();
    assert_eq!(omf.construct, ConstructKind::Positional);
    assert_roundtrip(&omf, &omf_mols);
    assert!(omf.confidence.clean);
    assert_eq!(omf.floating.len(), 1);
    assert_eq!(omf.floating[0].equiv.len(), 3);

    let ac_in = "CC(=O)Oc1ccccc1-c2ccccc2\nCC(=O)Oc1cccc(-c2ccccc2)c1\nCC(=O)Oc1ccc(-c2ccccc2)cc1";
    let ac_mols = parse_list(&lines(ac_in)).unwrap();
    let ac = generate(&lines(ac_in)).unwrap();
    assert_eq!(ac.construct, ConstructKind::Positional);
    assert_roundtrip(&ac, &ac_mols);
    assert!(ac.confidence.clean);
    assert_eq!(ac.floating.len(), 2);
    assert_eq!(ac.cx_smiles.matches("m:").count(), 2);
    let o_star = ac.floating.iter().find(|f| !f.split).expect("O* group");
    let acetyl = ac.floating.iter().find(|f| f.split).expect("acyl group");
    assert!(o_star.fragment_smiles.contains('O') && o_star.fragment_smiles.contains('*'));
    assert!(acetyl.fragment_smiles.contains('C') && acetyl.fragment_smiles.contains('*'));
}

#[test]
fn multi_group_positional_m_by_m() {
    let input = "\
Clc1ccccc1-c2ccc(F)cc2
Clc1cccc(-c2cccc(F)c2)c1
Clc1ccc(-c2ccccc2F)cc1";
    let mols = parse_list(&lines(input)).unwrap();
    let res = generate(&lines(input)).unwrap();
    assert_eq!(res.construct, ConstructKind::Positional);
    assert!(res.confidence.clean);
    assert_roundtrip(&res, &mols);
    assert_eq!(res.floating.len(), 2, "two independent moving groups");
    assert_eq!(res.cx_smiles.matches("m:").count(), 2);
}

#[test]
fn repeating_pfas_and_alkyl_and_custom_gcd_series() {
    let pfas_in = "OC(=O)C(F)(F)C(F)F\nOC(=O)C(F)(F)C(F)(F)C(F)F\nOC(=O)C(F)(F)C(F)(F)C(F)(F)C(F)F";
    let pfas_mols = parse_list(&lines(pfas_in)).unwrap();
    let pfas = generate(&lines(pfas_in)).unwrap();
    assert_eq!(pfas.construct, ConstructKind::Repeating);
    assert_roundtrip(&pfas, &pfas_mols);
    assert!(pfas.confidence.clean);
    assert_eq!(pfas.scaffold_smiles, "OC(=O)C(F)(F)C(F)F");
    assert_eq!(pfas.cx_smiles, "OC(=O)C(F)(F)C(F)F |Sg:n:3,4,5:n:ht|");
    let unit = pfas.repeat_unit.expect("repeat unit is populated");
    assert_eq!(unit.atoms, vec![3, 4, 5]);
    assert_eq!((unit.min, unit.max), (1, 3));

    let alkyl_in = "CCCCCCC\nCCCCCCCC\nCCCCCCCCC";
    let alkyl_mols = parse_list(&lines(alkyl_in)).unwrap();
    let alkyl = generate(&lines(alkyl_in)).unwrap();
    assert_eq!(alkyl.construct, ConstructKind::Repeating);
    assert_roundtrip(&alkyl, &alkyl_mols);
    assert_eq!(alkyl.scaffold_smiles, "CCCCCCC");
    assert!(alkyl.cx_smiles.contains("Sg:n:3:n:ht"));

    for (series, expected_cx, expected_enum) in [
        ("CCCCCCC\nCCCCCCCCC", "CCCCCCC |Sg:n:2,3:n:ht|", 2usize),
        (
            "CCCCCCC\nCCCCCCCCCC\nCCCCCCCCCCCCC",
            "CCCCCCC |Sg:n:2,3,4:n:ht|",
            3usize,
        ),
        (
            "CCCCCC\nCCCCCCCC\nCCCCCCCCCC",
            "CCCCCC |Sg:n:1,2:n:ht|",
            3usize,
        ),
    ] {
        let mols = parse_list(&lines(series)).expect("valid SMILES");
        let r = build_repeating(&mols).expect("repeat detected");
        assert_eq!(r.construct, ConstructKind::Repeating);
        assert_eq!(r.cx_smiles, expected_cx);
        assert_eq!(r.enumerated.len(), expected_enum);
        assert!(r.confidence.clean);
        assert_roundtrip(&r, &mols);
    }

    let pfas_pair = parse_list(&lines("OC(=O)C(F)(F)C(F)F\nOC(=O)C(F)(F)C(F)(F)C(F)F")).unwrap();
    let pair_r = build_repeating(&pfas_pair).unwrap();
    assert_eq!(pair_r.enumerated.len(), 2);
    assert!(pair_r.confidence.clean);
}

#[test]
fn four_member_biphenyl_and_aromatic_equivalence_and_constitutional_isomers() {
    let four_in = "\
Clc1ccccc1-c2ccccc2
Clc1cccc(-c2ccccc2)c1
Clc1ccc(-c2ccccc2)cc1
Clc1cc(-c2ccccc2)ccc1";
    let four_mols = parse_list(&lines(four_in)).unwrap();
    let four_r = build_positional_with_options(&four_mols, false).unwrap();
    assert_eq!(four_r.construct, ConstructKind::Positional);
    assert_eq!(four_r.floating[0].equiv.len(), 4);
    assert!(four_r.confidence.clean);
    assert_roundtrip(&four_r, &four_mols);

    let biphenyl_in = "Clc1ccccc1-c2ccccc2\nClc1cccc(-c2ccccc2)c1\nClc1ccc(-c2ccccc2)cc1";
    let bip_mols = parse_list(&lines(biphenyl_in)).unwrap();
    let bip_arom = generate_with_options(
        &lines(biphenyl_in),
        CxOptions {
            expand_aromatic: true,
        },
    )
    .unwrap();
    assert_eq!(bip_arom.construct, ConstructKind::Positional);
    assert_roundtrip(&bip_arom, &bip_mols);
    assert!(bip_arom.confidence.clean);
    assert_eq!(bip_arom.floating[0].equiv, vec![0, 1, 2, 3, 4]);
    assert_eq!(
        bip_arom.cx_smiles,
        "c1ccccc1-c2ccccc2.[*]Cl |m:12:0.1.2.3.4|"
    );

    let const_in = "CC(C)(C)O\nCC(C)CO\nCC(O)CC\nOCCCC\nCCOCC\nCOCCC";
    let const_r = generate(&lines(const_in)).unwrap();
    assert_eq!(const_r.construct, ConstructKind::BestEffort);
    assert!(const_r.confidence.coverage.fraction() < 1.0);
}

#[test]
fn outliers_fallback_and_attachment_bond_orders() {
    let mixed =
        generate_with_options(&lines("Fc1ccccc1\nFc1ccccc1Cl"), CxOptions::default()).unwrap();
    assert_eq!(mixed.construct, ConstructKind::BestEffort);
    assert_ne!(mixed.cx_smiles, "");

    let pos_outlier = "\
Clc1ccccc1-c2ccccc2
Clc1cccc(-c2ccccc2)c1
Clc1ccc(-c2ccccc2)cc1
O=C=O";
    let r_pos = generate(&lines(pos_outlier)).unwrap();
    assert_eq!(
        (
            r_pos.confidence.coverage.covered,
            r_pos.confidence.coverage.total
        ),
        (3, 4)
    );
    assert!(!r_pos.confidence.clean);
    assert_eq!(r_pos.construct, ConstructKind::BestEffort);

    let rep_outlier = "\
OC(=O)C(F)(F)C(F)F
OC(=O)C(F)(F)C(F)(F)C(F)F
OC(=O)C(F)(F)C(F)(F)C(F)(F)C(F)F
O=C=O";
    let r_rep = generate(&lines(rep_outlier)).unwrap();
    assert_eq!(
        (
            r_rep.confidence.coverage.covered,
            r_rep.confidence.coverage.total
        ),
        (3, 4)
    );
    assert!(!r_rep.confidence.clean);
    assert_eq!(r_rep.construct, ConstructKind::BestEffort);

    let identical = generate(&lines("CC\nCC")).unwrap();
    assert_eq!(identical.cx_smiles, "CC");

    let carbonyls = generate(&lines("CC1(F)CC(=O)CCC1\nCC1(F)CCC(=O)CC1")).unwrap();
    assert_eq!(carbonyls.construct, ConstructKind::Positional);
    assert!(carbonyls.cx_smiles.contains("[*]=O"));

    let aldehydes = generate(&lines("c1ccc(C=O)cc1-c2ccccc2\nc1cc(C=O)ccc1-c2ccccc2")).unwrap();
    assert_eq!(aldehydes.construct, ConstructKind::Positional);
    assert!(aldehydes.cx_smiles.contains("[*]=O"));

    let nitriles = generate(&lines("N#Cc1ccccc1-c2ccccc2\nN#Cc1cccc(-c2ccccc2)c1")).unwrap();
    assert_eq!(nitriles.construct, ConstructKind::Positional);
    assert!(nitriles.cx_smiles.contains("[*]#N"));

    let aryl = generate(&lines("Cc1ccccc1\nCCc1ccccc1")).unwrap();
    assert_ne!(aryl.construct, ConstructKind::Repeating);
}
