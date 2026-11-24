// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

use chematic::core::BondOrder;

use super::mol;
use crate::fragment::{
    FloatingDef, build_base_smiles, extract_subgraph, reorder_attachment_first,
    write_fragment_first,
};
use crate::graph::molecule_to_query;
use crate::positional::{build_positional_with_options, equiv_positions, floating_defs};

#[test]
fn a_single_atom_floating_group_is_never_split() {
    let c = mol("CCO");
    let defs = floating_defs(&c, &[1], &[true, false, true]);
    assert_eq!(defs.len(), 1, "one atom is one group, not two");
    assert!(!defs[0].split, "and it is not the split variant");
}

#[test]
fn a_pendant_two_atom_chain_splits_into_two_defs() {
    let c = mol("CCCO");
    let defs = floating_defs(&c, &[0, 1], &[false, false, true, true]);
    assert_eq!(defs.len(), 2, "a chain of two becomes two defs");
    assert!(!defs[0].split, "the first half carries the attachment");
    assert!(defs[1].split, "the second half is the split variant");
}

#[test]
fn a_branched_floating_group_is_not_split() {
    let c = mol("CC(C)C");
    let defs = floating_defs(&c, &[0, 2, 3], &[true, false, false, false, true]);
    assert_eq!(defs.len(), 1, "a branch stays whole");
    assert!(!defs[0].split, "and is not the split variant");
}

#[test]
fn an_unmatched_scaffold_offers_every_position() {
    let c = mol("CC");
    let scaffold_q = molecule_to_query(&c);
    let positions = equiv_positions(&c, &[], std::slice::from_ref(&c), &scaffold_q);
    assert_eq!(
        positions,
        vec![0, 1],
        "with no evidence, every scaffold atom is a candidate"
    );
}

#[test]
fn a_fragment_is_written_with_a_star_and_the_attachment_first() {
    let m = mol("CCO");
    let (atoms, bonds, attach) = extract_subgraph(&m, &[1, 2]);
    let (atoms, bonds, attach) = reorder_attachment_first(atoms, bonds, attach);
    let def = FloatingDef {
        atoms,
        bonds,
        attachment: attach,
        attach_order: BondOrder::Single,
        split: false,
    };
    let smi = write_fragment_first(&def);
    assert!(
        smi.starts_with("CO"),
        "the attachment (the oxygen) is written first: {smi}"
    );
}

#[test]
fn base_smiles_offsets_the_star_indices_by_the_scaffold_length() {
    let m = mol("CO");
    let (atoms, bonds, attach) = extract_subgraph(&m, &[1]);
    let def = FloatingDef {
        atoms,
        bonds,
        attachment: attach,
        attach_order: BondOrder::Single,
        split: false,
    };
    let (_smiles, star_idx, attach_idx) = build_base_smiles("CO", &[def], 2);
    assert_eq!(star_idx, vec![2], "the star follows the scaffold");
    assert_eq!(attach_idx, vec![3], "and the attachment follows the star");
}

#[test]
fn two_defs_skip_past_each_fragments_star_and_atoms() {
    let m = mol("CO");
    let (atoms, bonds, attach) = extract_subgraph(&m, &[1]);
    let one = FloatingDef {
        atoms,
        bonds,
        attachment: attach,
        attach_order: BondOrder::Single,
        split: false,
    };
    let two = one.clone();
    let (_smiles, star_idx, attach_idx) = build_base_smiles("C", &[one, two], 1);
    assert_eq!(
        star_idx,
        vec![1, 3],
        "each def consumes a star and its atoms"
    );
    assert_eq!(
        attach_idx,
        vec![2, 4],
        "and the attachment is the atom just after each star"
    );
}

#[test]
fn an_empty_group_is_an_error_and_not_a_panic() {
    let err =
        build_positional_with_options(&[], false).expect_err("there is nothing to build from");
    assert!(
        err.to_string().contains("group is empty"),
        "the error says which precondition failed: {err}"
    );
}

#[test]
fn reorder_and_write_fragment_handle_empty_and_out_of_range_attachment() {
    let empty_def = FloatingDef {
        atoms: Vec::new(),
        bonds: Vec::new(),
        attachment: 0,
        attach_order: BondOrder::Single,
        split: false,
    };
    assert_eq!(
        write_fragment_first(&empty_def),
        "",
        "empty fragment writes empty SMILES without panicking"
    );
    let (a, b, att) = reorder_attachment_first(Vec::new(), Vec::new(), 2);
    assert_eq!(a.len(), 0);
    assert_eq!(b.len(), 0);
    assert_eq!(att, 2);

    let m = mol("CCO");
    let (atoms, bonds, _) = extract_subgraph(&m, &[1, 2]);
    let oob_def = FloatingDef {
        atoms: atoms.clone(),
        bonds: bonds.clone(),
        attachment: 99,
        attach_order: BondOrder::Single,
        split: false,
    };
    assert_eq!(
        write_fragment_first(&oob_def),
        "CO",
        "out-of-range attachment still writes safely without panicking"
    );
    let (a2, b2, att2) = reorder_attachment_first(atoms, bonds, 99);
    assert_eq!(a2.len(), 2);
    assert_eq!(b2.len(), 1);
    assert_eq!(att2, 99);
}
