// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

// The `tests` tests, extracted from `mod.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `mod` module, so
// `use super::*` below reaches exactly what it did before the move.
use super::*;
use chematic::smiles;

#[test]
fn fatty_acid_matches_palmitic_acid() {
    let fa = ChemicalClass::defaults()
        .into_iter()
        .find(|c| c.name == "FA")
        .expect("FA class");
    let mol = smiles::parse("CCCCCCCCCCCCCCCC(=O)O").expect("valid SMILES");
    assert!(fa.matches(&mol));
}

#[test]
fn defaults_include_common_lipids() {
    let defaults = ChemicalClass::defaults();
    let names: Vec<_> = defaults.iter().map(|c| c.name.as_str()).collect();
    // FA, MUFA, PUFA
    assert!(names.contains(&"FA"));
    assert!(names.contains(&"MUFA"));
    assert!(names.contains(&"PUFA"));
    // GL
    assert!(names.contains(&"TG(AAA)"));
    assert!(names.contains(&"DG(AA)"));
    assert!(names.contains(&"MG(A)"));
    // GP
    assert!(names.contains(&"PC(AA)"));
    assert!(names.contains(&"PE(AA)"));
    assert!(names.contains(&"LPC(A)"));
    assert!(names.contains(&"LPE(A)"));
    // SP
    assert!(names.contains(&"Cer(AS)"));
    assert!(names.contains(&"SM(AS)"));
    // ST, PR, SL, PK
    assert!(names.contains(&"ST"));
    assert!(names.contains(&"PR"));
    assert!(names.contains(&"SL"));
    assert!(names.contains(&"PK"));
}

#[test]
fn defaults_map_provides_lookup() {
    let map = ChemicalClass::defaults_map();
    assert!(map.contains_key("PC(AA)"));
    assert_eq!(map.get("PC(AA)").map(|c| c.name.as_str()), Some("PC(AA)"));
}

#[test]
fn gp_class_order_matches_architecture_priority() {
    let gp: Vec<_> = ChemicalClass::defaults()
        .into_iter()
        .filter(|c| c.family == "Glycerophospholipids")
        .collect();
    let gp_names: Vec<_> = gp.iter().map(|c| c.name.as_str()).collect();
    // PI(AA) must come before PG(AA) — matches the LIPID MAPS architecture ordering
    let pi_pos = gp_names.iter().position(|&n| n == "PI(AA)").unwrap();
    let pg_idx = gp_names.iter().position(|&n| n == "PG(AA)").unwrap();
    assert!(
        pi_pos < pg_idx,
        "PI(AA) should appear before PG(AA) in the class ordering"
    );
}

#[test]
fn family_order_follows_lipid_maps_hierarchy() {
    let defaults = ChemicalClass::defaults();
    let families: Vec<&str> = defaults
        .iter()
        .map(|c| c.family.as_str())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    // Families should appear in LIPID MAPS order: FA, GL, GP, SP, ST, PR, SL, PK
    let expected = [
        "Fatty Acyls",
        "Glycerolipids",
        "Glycerophospholipids",
        "Sphingolipids",
        "Sterol Lipids",
        "Prenol Lipids",
        "Saccharolipids",
        "Polyketides",
    ];
    for (i, fam) in expected.iter().enumerate() {
        assert!(
            families.contains(fam),
            "Expected family {fam} at position {i}"
        );
    }
    // Verify the first occurrence order matches expected
    let mut seen: Vec<&str> = Vec::new();
    for class in &defaults {
        if !seen.contains(&class.family.as_str()) {
            seen.push(class.family.as_str());
        }
    }
    let expected_order: Vec<&str> = expected.to_vec();
    assert_eq!(
        seen, expected_order,
        "Family order should follow LIPID MAPS hierarchy"
    );
}
