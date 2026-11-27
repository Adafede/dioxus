// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

use chematic::core::{AtomIdx, Molecule, MoleculeBuilder};

use super::mol;
use crate::error::CxError;
use crate::graph::molecule_to_query;
use crate::repeating::{build_repeating, is_internal, repeat_pattern};
use crate::splice::add_scaffold_bonds;
use crate::types::ConstructKind;

fn pattern_of(longest: &str, scaffold: &str, unit_size: usize) -> Vec<u32> {
    let long = mol(longest);
    let q = molecule_to_query(&mol(scaffold));
    repeat_pattern(&long, unit_size, &q).expect("the scaffold is in the longest")
}

fn copied_bonds(scaffold: &Molecule, a: (u32, u32), b: (u32, u32)) -> Vec<(u32, u32)> {
    let mut builder = MoleculeBuilder::new();
    let mapped: Vec<AtomIdx> = scaffold
        .atoms()
        .map(|(_, atom)| builder.add_atom(atom.clone()))
        .collect();
    add_scaffold_bonds(&mut builder, scaffold, &mapped, &[(a.0, a.1), (b.0, b.1)]);
    let out = builder.build();
    out.bonds()
        .map(|(_, be)| (be.atom1.0, be.atom2.0))
        .collect()
}

#[test]
fn an_empty_group_is_an_error_and_not_a_panic() {
    let err = build_repeating(&[]).expect_err("there is nothing to build from");
    assert!(
        err.to_string().contains("group is empty"),
        "the error says which precondition failed: {err}"
    );
}

#[test]
fn uniform_size_group_returns_uniform_atom_counts_error() {
    let err = build_repeating(&[mol("CCC"), mol("CCO")])
        .expect_err("uniform sizes cannot form an Sg:n repeat");
    assert_eq!(err, CxError::UniformAtomCounts);
}

#[test]
fn the_pattern_is_the_whole_of_the_extra_atoms() {
    assert_eq!(
        pattern_of("CCCCCC", "CC", 2),
        vec![2, 3, 4, 5],
        "the four atoms the scaffold did not match"
    );
}

#[test]
fn a_component_that_is_not_a_whole_number_of_units_is_not_the_pattern() {
    assert_eq!(
        pattern_of("CCCCCC.O", "CC", 2),
        vec![2, 3, 4, 5],
        "four carbons, not the single oxygen"
    );
}

#[test]
fn a_long_component_that_does_not_divide_is_not_the_pattern() {
    assert_eq!(
        pattern_of("CCCCCCC.O", "CC", 2),
        vec![7],
        "the oxygen, because nothing else is a whole number of units"
    );
}

#[test]
fn the_smallest_component_is_used_when_nothing_divides() {
    assert_eq!(
        pattern_of("CC.CCO", "CC", 2),
        vec![2, 3, 4],
        "nothing is a whole number of two-atom units, so the smallest wins"
    );
}

#[test]
fn both_anchor_bonds_are_dropped_and_the_rest_are_kept() {
    let chain = mol("CCCC");
    let copied = copied_bonds(&chain, (0, 1), (2, 3));
    assert_eq!(
        copied,
        vec![(1, 2)],
        "the middle bond survives; the two anchor bonds do not"
    );
}

#[test]
fn one_anchor_bond_is_dropped_either_way_round() {
    let chain = mol("CCCC");
    assert_eq!(
        copied_bonds(&chain, (0, 1), (9, 9)),
        vec![(1, 2), (2, 3)],
        "naming the bond forwards drops it"
    );
    assert_eq!(
        copied_bonds(&chain, (1, 0), (9, 9)),
        vec![(1, 2), (2, 3)],
        "and naming it backwards drops the same bond"
    );
}

#[test]
fn incomplete_repeating_roundtrip_is_best_effort() {
    let group = [
        mol("CCCCCC"),
        mol("CCCCCCCC"),
        mol("CC(C)CCCCCCC"),
        mol("CCCCCCCCCC"),
    ];
    let r = build_repeating(&group).expect("returns a best-effort repeating result");
    assert!(!r.coverage.is_clean(), "branched isomer is not reproduced");
    assert_eq!(r.coverage.covered, 3);
    assert_eq!(r.coverage.total, 4);
    assert_eq!(
        r.construct,
        ConstructKind::BestEffort,
        "incomplete round-trip coverage must demote construct to BestEffort"
    );
}

#[test]
fn non_single_external_bonds_are_not_internal_and_ring_repeats_are_rejected() {
    let benzene = mol("c1ccccc1");
    assert!(
        !is_internal(&[1], &benzene),
        "aromatic ring carbon has aromatic external bonds, not single chain bonds"
    );
    let propene = mol("C=CC");
    assert!(
        !is_internal(&[1], &propene),
        "alkene carbon with a double external bond is not a single-bonded repeat link"
    );

    assert!(build_repeating(&[mol("Cc1ccccc1"), mol("CCc1ccccc1")]).is_err());
    assert!(build_repeating(&[mol("CC1CCCCC1"), mol("CCC1CCCCC1")]).is_err());

    let two_ok = build_repeating(&[mol("CCC"), mol("CCCC")])
        .expect("a two-member alkane homologue pair with covered == 2 must succeed");
    assert_eq!(two_ok.construct, ConstructKind::Repeating);
    assert_eq!(two_ok.coverage.covered, 2);

    let dup_only = build_repeating(&[mol("CCCC"), mol("CCCC"), mol("CC(C)CC")]);
    assert_eq!(
        dup_only.unwrap_err(),
        CxError::NoHomologueReproduced,
        "duplicate shortest molecule alone must not satisfy distinct-homologue guard"
    );

    // 4-atom branched unit (-C(F)(F)O-) requires ESU connected-subgraph search (not simple paths).
    let branched_mol = mol("CCC(F)(F)OC");
    let found = crate::repeating::locate_repeat_in_scaffold(&branched_mol, &[6, 8, 9, 9], 4)
        .expect("4-atom branched unit must be found by ESU search");
    assert_eq!(found, vec![2, 3, 4, 5]);

    let branched_series = [mol("CCCC(F)(F)OC(F)F"), mol("CCCC(F)(F)OC(F)(F)OC(F)F")];
    let branched_res =
        build_repeating(&branched_series).expect("4-atom branched repeat series must build");
    assert_eq!(branched_res.construct, ConstructKind::Repeating);
    assert!(branched_res.coverage.is_clean());
}
