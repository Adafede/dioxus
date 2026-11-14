// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

// The `tests` tests, extracted from `mod.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `mod` module, so
// `use super::*` below reaches exactly what it did before the move.
use super::*;
use chematic::smiles::{parse, write};

fn lines(s: &str) -> Vec<String> {
    s.lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

fn assert_roundtrip(result: &CxResult, group: &[Molecule]) {
    let canon: Vec<String> = group.iter().map(canonical_smiles).collect();
    for c in &canon {
        assert!(
            result.enumerated.iter().any(|e| e == c),
            "input canonical SMILES {c} is NOT covered by round-trip; enumerated={:?}",
            result.enumerated
        );
    }
}

#[test]
fn parse_canonical_write_roundtrip() {
    let m = parse("Clc1ccccc1-c2ccccc2").unwrap();
    assert_eq!(canonical_smiles(&m), "c1(-c2ccccc2Cl)ccccc1");
    assert_eq!(write(&m), "Clc1ccccc1-c2ccccc2");
}

#[test]
fn positional_biphenyl_cl() {
    // Reference (RDKit-indexed): C1=CC=CC=C1C2=...Cl* |m:13:0.2.3|
    let input = "Clc1ccccc1-c2ccccc2\nClc1cccc(-c2ccccc2)c1\nClc1ccc(-c2ccccc2)cc1";
    let mols = parse_list(&lines(input)).unwrap();
    let r = generate(&lines(input)).unwrap();
    assert_eq!(r.construct, Construct::Positional);
    assert_roundtrip(&r, &mols);
    assert!(r.confidence.clean);
    assert_eq!(r.scaffold_smiles, "c1ccccc1-c2ccccc2");
    assert_eq!(r.floating.len(), 1);
    assert_eq!(r.floating[0].equiv.len(), 3);
    assert!(r.cx_smiles.contains("m:"));
    assert_eq!(r.cx_smiles.matches("m:").count(), 1);
}

#[test]
fn positional_omf_moving() {
    // Reference: OC1=C(O)C=C(O)C=C1.C* |m:10:0.3.6|
    let input = "COc1c(O)cc(O)cc1\nOc1c(OC)cc(O)cc1\nOc1c(O)cc(OC)cc1";
    let mols = parse_list(&lines(input)).unwrap();
    let r = generate(&lines(input)).unwrap();
    assert_eq!(r.construct, Construct::Positional);
    assert_roundtrip(&r, &mols);
    assert!(r.confidence.clean);
    assert_eq!(r.floating.len(), 1);
    assert_eq!(r.floating[0].equiv.len(), 3);
}

#[test]
fn positional_acetyl_double_m() {
    // Acetate (-OCOCH3) moves on a triol scaffold; emitted as two m: blocks
    // (O* + C(=O)(C)*) — the "double m: block variant".
    let input = "CC(=O)Oc1ccccc1-c2ccccc2\nCC(=O)Oc1cccc(-c2ccccc2)c1\nCC(=O)Oc1ccc(-c2ccccc2)cc1";
    let mols = parse_list(&lines(input)).unwrap();
    let r = generate(&lines(input)).unwrap();
    assert_eq!(r.construct, Construct::Positional);
    assert_roundtrip(&r, &mols);
    assert!(r.confidence.clean);
    assert_eq!(r.floating.len(), 2, "floating={:?}", r.floating);
    let o_star = r.floating.iter().find(|f| !f.split).expect("O* group");
    let acetyl = r
        .floating
        .iter()
        .find(|f| f.split)
        .expect("C(=O)(C)* group");
    assert!(o_star.fragment_smiles.contains('O') && o_star.fragment_smiles.contains('*'));
    assert!(
        acetyl.fragment_smiles.contains('C')
            && acetyl.fragment_smiles.contains('O')
            && acetyl.fragment_smiles.contains('*')
    );
    assert_eq!(o_star.equiv.len(), 3);
    assert_eq!(r.cx_smiles.matches("m:").count(), 2);
}

#[test]
fn repeating_pfas() {
    // Reference: OC(=O)C(F)(F)C(F)F |Sg:n:3,4,5:n:ht|
    let input = "OC(=O)C(F)(F)C(F)F\nOC(=O)C(F)(F)C(F)(F)C(F)F\nOC(=O)C(F)(F)C(F)(F)C(F)(F)C(F)F";
    let mols = parse_list(&lines(input)).unwrap();
    let r = generate(&lines(input)).unwrap();
    assert_eq!(r.construct, Construct::Repeating);
    assert_roundtrip(&r, &mols);
    assert!(r.confidence.clean);
    assert_eq!(r.scaffold_smiles, "OC(=O)C(F)(F)C(F)F");
    assert_eq!(r.cx_smiles, "OC(=O)C(F)(F)C(F)F |Sg:n:3,4,5:n:ht|");
    let unit = r.repeat_unit.expect("repeat unit is populated");
    assert_eq!(unit.atoms, vec![3, 4, 5]);
    assert_eq!(unit.min, 1);
    assert_eq!(unit.max, 3);
}

#[test]
fn repeating_alkyl() {
    // Reference: CCCCCCC |Sg:n:3:n:ht|
    let input = "CCCCCCC\nCCCCCCCC\nCCCCCCCCC";
    let mols = parse_list(&lines(input)).unwrap();
    let r = generate(&lines(input)).unwrap();
    assert_eq!(r.construct, Construct::Repeating);
    assert_roundtrip(&r, &mols);
    assert_eq!(r.scaffold_smiles, "CCCCCCC");
    assert!(r.cx_smiles.contains("Sg:n:3:n:ht"));
}

/// `build_repeating` end to end, for a series whose arithmetic is checkable
/// by hand.
///
/// The deltas drive the GCD, the GCD is the unit size, and the unit size is
/// the number of atom indices in the `Sg:n:` field — so the field is a
/// readout of the whole chain of arithmetic above it. These are the two
/// families that separate the operations:
///
/// | input lengths | deltas  | GCD | unit | `Sg:n:` |
/// |---------------|---------|-----|------|---------|
/// | 7, 9          | 2       | 2   | 2    | 2,3     |
/// | 7, 10, 13     | 3, 3    | 3   | 3    | 2,3,4   |
/// | 6, 8, 10      | 2, 2    | 2   | 2    | 1,2     |
fn assert_repeating(input: &str, expected_cx: &str, expected_enumerated: usize) {
    let mols = parse_list(&lines(input)).expect("the fixture is valid SMILES");
    let r = build_repeating(&mols).expect("the fixture has a repeat");
    assert_eq!(r.construct, Construct::Repeating, "construct");
    assert_eq!(r.cx_smiles, expected_cx, "cx-smiles");
    assert_eq!(
        r.enumerated.len(),
        expected_enumerated,
        "one arrangement per repeat count between the smallest and largest \
         input: {}",
        r.cx_smiles
    );
    assert!(
        r.confidence.clean,
        "every input came back, so the count range was right: {:?}",
        r.confidence
    );
    assert_roundtrip(&r, &mols);
}

#[test]
fn a_two_atom_repeat_unit_is_found() {
    // 7 and 9 carbons: one delta of 2, so the unit is two atoms.
    assert_repeating("CCCCCCC\nCCCCCCCCC", "CCCCCCC |Sg:n:2,3:n:ht|", 2);
}

#[test]
fn a_three_atom_repeat_unit_is_found() {
    // 7, 10 and 13 carbons: deltas of 3, so the unit is three atoms, not
    // one. A GCD that returned 1 here would put a single index in the field.
    assert_repeating(
        "CCCCCCC\nCCCCCCCCCC\nCCCCCCCCCCCCC",
        "CCCCCCC |Sg:n:2,3,4:n:ht|",
        3,
    );
}

#[test]
fn a_two_atom_repeat_unit_in_a_ten_atom_scaffold() {
    // 6, 8 and 10 carbons: the unit is at the far end of the scaffold, so
    // the indices are low. This is the case that separates "deltas of 2" from
    // "a delta of 1", which is what the single-carbon alkyl series gives.
    assert_repeating("CCCCCC\nCCCCCCCC\nCCCCCCCCCC", "CCCCCC |Sg:n:1,2:n:ht|", 3);
}

#[test]
fn a_repeat_count_of_one_is_enumerated_as_one() {
    // Two inputs differing by a whole unit is two repeat counts, and the
    // coverage is still complete — which is what says the min was not
    // rounded up past the smallest input.
    let input = "OC(=O)C(F)(F)C(F)F\nOC(=O)C(F)(F)C(F)(F)C(F)F";
    let mols = parse_list(&lines(input)).expect("valid SMILES");
    let r = build_repeating(&mols).expect("the fixture has a repeat");
    assert_eq!(r.enumerated.len(), 2, "one arrangement per repeat count");
    assert!(
        r.confidence.clean,
        "and the smallest input is inside the range: {:?}",
        r.confidence
    );
}

/// `build_positional` end to end for a four-member series, where the
/// equivalent-position list is the thing being computed.
#[test]
fn a_four_member_series_reports_four_equivalent_positions() {
    // Chlorine on each of the four distinct positions of one ring of a
    // biphenyl. The `m:` field lists every position chlorine was seen at,
    // so four inputs mean four positions, and the enumeration is the
    // product of the per-group choices.
    let input = "\
Clc1ccccc1-c2ccccc2
Clc1cccc(-c2ccccc2)c1
Clc1ccc(-c2ccccc2)cc1
Clc1cc(-c2ccccc2)ccc1";
    let mols = parse_list(&lines(input)).expect("valid SMILES");
    let r = build_positional(&mols).expect("the fixture has a moving group");
    assert_eq!(r.construct, Construct::Positional, "construct");
    assert_eq!(r.floating.len(), 1, "one moving group: {:?}", r.floating);
    assert_eq!(
        r.floating[0].equiv.len(),
        4,
        "chlorine was seen at four positions, so all four are equivalent"
    );
    assert!(
        r.confidence.clean,
        "every input came back: {:?}",
        r.confidence
    );
    assert_roundtrip(&r, &mols);
}

#[test]
fn the_positional_field_names_the_star_and_its_positions() {
    // The exact string, because it is the artefact this app produces: a
    // wrong index here is still a well-formed CX-SMILES field, and nothing
    // downstream of it would notice.
    let input = "\
Clc1ccccc1-c2ccccc2
Clc1cccc(-c2ccccc2)c1
Clc1ccc(-c2ccccc2)cc1";
    let r = build_positional(&parse_list(&lines(input)).expect("valid SMILES"))
        .expect("the fixture has a moving group");
    assert_eq!(
        r.cx_smiles, "c1ccccc1-c2ccccc2.[*]Cl |m:12:0.2.3|",
        "the emitted CX-SMILES"
    );
}

#[test]
fn best_effort_constitutional_isomers() {
    // Six constitutional isomers of C4H10O with no clean shared moving
    // group -> best-effort, sub-100% round-trip coverage.
    let input = "\
CC(C)(C)O
CC(C)CO
CC(O)CC
OCCCC
CCOCC
COCCC";
    let r = generate(&lines(input)).unwrap();
    assert_eq!(r.construct, Construct::BestEffort);
    assert!(
        r.confidence.coverage.fraction() < 1.0,
        "expected sub-100% coverage, got {:?}; cx={}",
        r.confidence,
        r.cx_smiles
    );
}

#[test]
fn positional_biphenyl_cl_with_aromatic_equivalence() {
    let input = "Clc1ccccc1-c2ccccc2\nClc1cccc(-c2ccccc2)c1\nClc1ccc(-c2ccccc2)cc1";
    let mols = parse_list(&lines(input)).unwrap();
    let r = generate_with_options(&lines(input), true).unwrap();
    assert_eq!(r.construct, Construct::Positional);
    assert_roundtrip(&r, &mols);
    assert!(r.confidence.clean);
    assert_eq!(r.floating[0].equiv, vec![0, 1, 2, 3, 4]);
    assert_eq!(r.cx_smiles, "c1ccccc1-c2ccccc2.[*]Cl |m:12:0.1.2.3.4|");
}

#[test]
fn non_uniform_group_without_internal_repeat_falls_back_to_positional() {
    // Fluorobenzene (7 atoms) and chlorobenzene (7 atoms) vs 1,2-dichlorobenzene (8 atoms):
    // extra Cl has atomic number 17, which is absent from fluorobenzene, so
    // `locate_repeat_in_scaffold` fails and `generate_with_options` falls back
    // to `build_positional_with_options`.
    let input = "Fc1ccccc1\nFc1ccccc1Cl";
    let r = generate_with_options(&lines(input), false).unwrap();
    assert_eq!(r.construct, Construct::BestEffort);
    assert_ne!(r.cx_smiles, "");
}

#[test]
fn generate_counts_dropped_outliers_in_coverage_and_demotes_to_best_effort() {
    // Three clustered biphenyl positional isomers plus an unrelated small molecule:
    // the outlier is excluded from the dominant cluster, so round-trip total must
    // count all four inputs (3/4) and demote construct to BestEffort.
    let pos_with_outlier = "\
Clc1ccccc1-c2ccccc2
Clc1cccc(-c2ccccc2)c1
Clc1ccc(-c2ccccc2)cc1
O=C=O";
    let r_pos = generate(&lines(pos_with_outlier)).unwrap();
    assert_eq!(r_pos.confidence.coverage.covered, 3);
    assert_eq!(r_pos.confidence.coverage.total, 4);
    assert!(
        !r_pos.confidence.clean,
        "dropped outlier prevents clean coverage"
    );
    assert_eq!(r_pos.construct, Construct::BestEffort);

    // Three clustered repeating PFAS chains plus an unrelated outlier.
    let rep_with_outlier = "\
OC(=O)C(F)(F)C(F)F
OC(=O)C(F)(F)C(F)(F)C(F)F
OC(=O)C(F)(F)C(F)(F)C(F)(F)C(F)F
O=C=O";
    let r_rep = generate(&lines(rep_with_outlier)).unwrap();
    assert_eq!(r_rep.confidence.coverage.covered, 3);
    assert_eq!(r_rep.confidence.coverage.total, 4);
    assert!(
        !r_rep.confidence.clean,
        "dropped outlier prevents clean coverage"
    );
    assert_eq!(r_rep.construct, Construct::BestEffort);
}
