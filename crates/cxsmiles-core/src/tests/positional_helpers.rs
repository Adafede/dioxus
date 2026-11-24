// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

use chematic::core::{Atom, BondOrder, Molecule, MoleculeBuilder};
use chematic::smiles::canonical_smiles;

use super::mol;
use crate::aromatic::{expand_aromatic_equiv, same_aromatic_ring};
use crate::fragment::{add_atoms_and_bonds, extract_subgraph, reorder_attachment_first};
use crate::graph::molecule_to_query;
use crate::positional::{
    boundary_edges, equiv_positions, is_matched, split_floating_and_recovered,
};
use chematic::core::AtomIdx;

#[test]
fn a_mask_answers_only_about_atoms_it_covers() {
    let mask = [true, false, true];
    assert!(is_matched(&mask, 0), "atom 0 is set");
    assert!(!is_matched(&mask, 1), "atom 1 is not");
    assert!(
        !is_matched(&mask, 9),
        "an atom past the end of the mask is not matched, rather than a panic"
    );
}

#[test]
fn a_component_joined_to_the_scaffold_once_is_floating() {
    let c = mol("CCC");
    let matched = [true, false, true];
    assert_eq!(
        boundary_edges(&[1], &matched, &c),
        2,
        "the middle carbon is joined at both ends"
    );
}

#[test]
fn a_terminal_component_is_joined_once() {
    let c = mol("CCC");
    let matched = [false, true, true];
    assert_eq!(
        boundary_edges(&[0], &matched, &c),
        1,
        "an end is joined at one bond only"
    );
}

#[test]
fn an_isolated_component_has_no_boundary() {
    let c = mol("CCC");
    assert_eq!(
        boundary_edges(&[1], &[false; 3], &c),
        0,
        "with no matched neighbours there is no boundary"
    );
}

#[test]
fn components_join_the_scaffold_once_become_floating_and_the_rest_recovered() {
    let c = mol("CCC");
    let matched = [false, true, false];
    let (floating, recovered) =
        split_floating_and_recovered(&[vec![0], vec![2], vec![1]], &matched, &c);
    assert_eq!(floating, vec![vec![0], vec![2]], "both ends are pendant");
    assert_eq!(recovered, vec![1], "the matched atom is recovered");
}

#[test]
fn extracting_a_subgraph_takes_only_its_own_bonds() {
    let c = mol("CCC");
    let (atoms, bonds, _) = extract_subgraph(&c, &[1, 2]);
    assert_eq!(atoms.len(), 2, "two atoms were asked for");
    assert_eq!(bonds.len(), 1, "and only the bond inside the subset");
}

#[test]
fn extracting_reindexes_the_bond_endpoints() {
    let c = mol("CCC");
    let (atoms, bonds, _) = extract_subgraph(&c, &[2, 0]);
    assert_eq!(atoms.len(), 2, "two atoms");
    assert_eq!(
        bonds.len(),
        0,
        "carbons 0 and 2 are not bonded to each other"
    );
}

#[test]
fn extracting_consecutive_carbons_reindexes_to_zero_and_one() {
    let c = mol("CCCC");
    let (_, bonds, _) = extract_subgraph(&c, &[1, 2]);
    assert_eq!(bonds, vec![(0, 1, BondOrder::Single)], "1-2 becomes 0-1");
}

#[test]
fn the_attachment_is_a_position_within_the_subset_not_a_molecule_index() {
    let c = mol("CCC");
    let (_, _, attach) = extract_subgraph(&c, &[1, 2]);
    assert_eq!(
        attach, 0,
        "position 0 of the subset, which is molecule atom 1"
    );
}

#[test]
fn the_attachment_of_a_closed_subset_is_zero() {
    let c = mol("CC");
    let (_, _, attach) = extract_subgraph(&c, &[0, 1]);
    assert_eq!(attach, 0, "no boundary atom, so index 0");
}

#[test]
fn reordering_with_the_attachment_already_first_changes_nothing() {
    let m = mol("CC");
    let (atoms, bonds, attach) = extract_subgraph(&m, &[0, 1]);
    let (a2, b2, at2) = reorder_attachment_first(atoms.clone(), bonds.clone(), attach);
    assert_eq!(a2.len(), atoms.len(), "same atoms");
    assert_eq!(b2, bonds, "same bonds");
    assert_eq!(at2, 0, "still index 0");
}

#[test]
fn reordering_puts_the_attachment_first_and_remaps_the_bonds() {
    let c = mol("CCC");
    let (atoms, bonds, attach) = extract_subgraph(&c, &[1, 2]);
    assert_eq!(attach, 0, "the fixture's attachment is index 0");
    let (a2, b2, at2) = reorder_attachment_first(atoms, bonds, 1);
    assert_eq!(at2, 0, "the reordered attachment is always index 0");
    assert_eq!(a2.len(), 2, "both atoms survive");
    assert_eq!(b2.len(), 1, "the bond survives");
    assert_eq!(
        b2[0],
        (1, 0, BondOrder::Single),
        "0-1 is remapped to 1-0 by the swap"
    );
}

fn molecule_of(atoms: &[Atom], bonds: &[(usize, usize, BondOrder)]) -> Molecule {
    let mut b = MoleculeBuilder::new();
    let _indices = add_atoms_and_bonds(&mut b, atoms, bonds);
    b.build()
}

#[test]
fn reordering_preserves_the_molecule() {
    let c = mol("CCC");
    let (atoms, bonds, _) = extract_subgraph(&c, &[0, 1]);
    let before = canonical_smiles(&molecule_of(&atoms, &bonds));
    let (a2, b2, _) = reorder_attachment_first(atoms, bonds, 1);
    let after = canonical_smiles(&molecule_of(&a2, &b2));
    assert_eq!(before, after, "a reordering is a relabelling, not an edit");
}

#[test]
fn same_aromatic_ring_stays_within_one_aromatic_component() {
    let biphenyl = mol("c1ccccc1-c2ccccc2");
    assert!(
        same_aromatic_ring(&biphenyl, 0, 4),
        "atoms 0 and 4 are in the first phenyl ring"
    );
    assert!(
        !same_aromatic_ring(&biphenyl, 0, 6),
        "atom 6 is in the second phenyl ring across a single bond"
    );
    let propane = mol("CCC");
    assert!(
        !same_aromatic_ring(&propane, 0, 1),
        "aliphatic single bonds do not connect an aromatic ring"
    );
}

#[test]
fn expand_aromatic_equiv_adds_equivalent_ring_positions_only() {
    let biphenyl = mol("c1ccccc1-c2ccccc2");
    assert_eq!(
        expand_aromatic_equiv(&[0, 2, 3], &biphenyl),
        vec![0, 1, 2, 3, 4],
        "expands to all 5 unsubstituted aromatic CH carbons of the first ring"
    );
    let phenylpyridine = mol("c1ccncc1-c2ccccc2");
    assert_eq!(
        expand_aromatic_equiv(&[0], &phenylpyridine),
        vec![0, 1, 2, 4],
        "excludes heteroatom N (3), ipso C (5), and the second ring"
    );
    let propane = mol("CCC");
    assert_eq!(
        expand_aromatic_equiv(&[0], &propane),
        vec![0],
        "aliphatic position does not expand across single bonds"
    );
    let benzene = mol("c1ccccc1");
    assert_eq!(
        expand_aromatic_equiv(&[0], &benzene),
        vec![0, 1, 2, 3, 4, 5],
        "unsubstituted benzene expands to all six ring carbons"
    );
}

#[test]
fn equiv_positions_filters_to_matching_component() {
    let scaffold_q = molecule_to_query(&mol("CCCO"));
    let m1 = mol("ClCC(F)CO");
    let m2 = mol("CC(Cl)C(F)O");
    assert_eq!(m1.atom(AtomIdx(0)).element.atomic_number(), 17, "Cl at 0");
    assert_eq!(m1.atom(AtomIdx(3)).element.atomic_number(), 9, "F at 3");
    let group = [m1.clone(), m2];
    assert_eq!(
        equiv_positions(&m1, &[0], &group, &scaffold_q),
        vec![0, 1],
        "Cl positions exclude F's attachment at C2"
    );
    assert_eq!(
        equiv_positions(&m1, &[3], &group, &scaffold_q),
        vec![1, 2],
        "F positions exclude Cl's attachment at C0"
    );
}
