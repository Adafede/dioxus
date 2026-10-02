// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

// The `tests` tests, extracted from `positional.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `positional` module, so
// `use super::*` below reaches exactly what it did before the move.
use super::*;
use chematic::smiles::{canonical_smiles, parse};

fn mol(smiles: &str) -> Molecule {
    parse(smiles).expect("the fixtures in this module are valid SMILES")
}

/// A linear chain of `n` carbons, as indices.
fn chain(n: u32) -> Vec<u32> {
    (0..n).collect()
}

// ── is_matched ──────────────────────────────────────────────────────────

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

// ── boundary_edges / split_floating_and_recovered ────────────────────────

#[test]
fn a_component_joined_to_the_scaffold_once_is_floating() {
    // CCC, the middle carbon floating: one bond out to each end, and the
    // ends are matched scaffold. Two boundary edges, not one — which is
    // what makes `== 1` a two-atom pendant group rather than any fragment.
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
    // The end carbon of CCC, with atom 1 matched: one bond out, so the
    // component is a pendant group.
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
    // Nothing matched, so nothing is scaffold and nothing is a boundary.
    let c = mol("CCC");
    assert_eq!(
        boundary_edges(&[1], &[false; 3], &c),
        0,
        "with no matched neighbours there is no boundary"
    );
}

#[test]
fn components_join_the_scaffold_once_become_floating_and_the_rest_recovered() {
    // One component with a single boundary edge, one with two. The `== 1`
    // is the whole decision, so both directions are pinned: a lone pendant
    // goes one way and a re-integrating fragment the other.
    let c = mol("CCC");
    let matched = [false, true, false];
    // Atom 1 is matched, so {0} and {2} each have one boundary edge and
    // {1} has none of its own as a component.
    let (floating, recovered) =
        split_floating_and_recovered(&[vec![0], vec![2], vec![1]], &matched, &c);
    assert_eq!(floating, vec![vec![0], vec![2]], "both ends are pendant");
    assert_eq!(recovered, vec![1], "the matched atom is recovered");
}

// ── extract_subgraph ────────────────────────────────────────────────────

#[test]
fn extracting_a_subgraph_takes_only_its_own_bonds() {
    // CCC, taking the last two carbons: both atoms, and the one bond
    // between them. The bond to the excluded carbon must not appear.
    let c = mol("CCC");
    let (atoms, bonds, _attach) = extract_subgraph(&c, &[1, 2]);
    assert_eq!(atoms.len(), 2, "two atoms were asked for");
    assert_eq!(bonds.len(), 1, "and only the bond inside the subset");
}

#[test]
fn extracting_reindexes_the_bond_endpoints() {
    // Taking carbons 2 and 0 out of order: the bond between them must come
    // back as 0–1, because the callers index `bonds` by position in
    // `atoms` and would otherwise address the wrong atom.
    let c = mol("CCC");
    let (atoms, bonds, _attach) = extract_subgraph(&c, &[2, 0]);
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
    let (_atoms, bonds, _attach) = extract_subgraph(&c, &[1, 2]);
    assert_eq!(bonds, vec![(0, 1, BondOrder::Single)], "1–2 becomes 0–1");
}

#[test]
fn the_attachment_is_a_position_within_the_subset_not_a_molecule_index() {
    // Carbons 1 and 2 of CCC: the atom at *position* 0 of the subset
    // (molecule atom 1) is joined to the excluded atom 0, and the atom at
    // position 1 (molecule atom 2) to the excluded atom 2. Both qualify, the
    // first is taken, and the answer is a subset position — callers index
    // the returned `atoms` with it. Returning the molecule index 1 here
    // would address the wrong atom, by one, for every subset that did not
    // begin at zero.
    let c = mol("CCC");
    let (_atoms, _bonds, attach) = extract_subgraph(&c, &[1, 2]);
    assert_eq!(
        attach, 0,
        "position 0 of the subset, which is molecule atom 1"
    );
}

#[test]
fn the_attachment_of_a_closed_subset_is_zero() {
    // A whole molecule has no neighbour outside itself, so there is no
    // boundary atom and the fallback is index 0.
    let c = mol("CC");
    let (_atoms, _bonds, attach) = extract_subgraph(&c, &[0, 1]);
    assert_eq!(attach, 0, "no boundary atom, so index 0");
}

// ── reorder_attachment_first ────────────────────────────────────────────

#[test]
fn reordering_with_the_attachment_already_first_changes_nothing() {
    // `attach == 0` returns the input untouched — no copy, no remap. A
    // mutant that drops the early return still produces the right answer
    // here, which is why the identity case is asserted separately from the
    // reordering case.
    let m = mol("CC");
    let (atoms, bonds, attach) = extract_subgraph(&m, &[0, 1]);
    let (a2, b2, at2) = reorder_attachment_first(atoms.clone(), bonds.clone(), attach);
    assert_eq!(a2.len(), atoms.len(), "same atoms");
    assert_eq!(b2, bonds, "same bonds");
    assert_eq!(at2, 0, "still index 0");
}

#[test]
fn reordering_puts_the_attachment_first_and_remaps_the_bonds() {
    // CCC, taking carbons 1 and 2 with the attachment at 1 (index 0 in the
    // subset). Reordering with attachment index 1 swaps them, so the bond
    // 0–1 has to become 1–0 or the molecule changes.
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
        "0–1 is remapped to 1–0 by the swap"
    );
}

#[test]
fn reordering_preserves_the_molecule() {
    // The real invariant behind the remap: the bonds still describe the
    // same molecule. A permutation that forgot to remap one endpoint would
    // satisfy the atom count and break this.
    let c = mol("CCC");
    let (atoms, bonds, _) = extract_subgraph(&c, &[0, 1]);
    let before = canonical_smiles(&molecule_of(&atoms, &bonds));
    let (a2, b2, _) = reorder_attachment_first(atoms, bonds, 1);
    let after = canonical_smiles(&molecule_of(&a2, &b2));
    assert_eq!(before, after, "a reordering is a relabelling, not an edit");
}

/// Rebuild a molecule from atoms and index pairs, for the two tests above.
fn molecule_of(atoms: &[Atom], bonds: &[(usize, usize, BondOrder)]) -> Molecule {
    let mut b = MoleculeBuilder::new();
    let idx: Vec<_> = atoms.iter().map(|a| b.add_atom(a.clone())).collect();
    for (a1, a2, order) in bonds {
        if let (Some(x), Some(y)) = (idx.get(*a1), idx.get(*a2)) {
            let _ = b.add_bond(*x, *y, *order);
        }
    }
    b.build()
}

// ── floating_defs: the chain test that decides the split ────────────────

#[test]
fn a_single_atom_floating_group_is_never_split() {
    // A one-atom fragment cannot be a chain, whatever its neighbours look
    // like, so it stays one def. `frag.len() > 1` is the guard.
    let c = mol("CCO");
    let keep = [true, false, true];
    let defs = floating_defs(&c, &[1], &keep);
    assert_eq!(defs.len(), 1, "one atom is one group, not two");
    assert!(!defs[0].split, "and it is not the split variant");
}

#[test]
fn a_pendant_two_atom_chain_splits_into_two_defs() {
    // CCO with the middle-and-... no: take carbons 0 and 1 with atom 2
    // matched, so the fragment is a two-atom chain hanging off the
    // scaffold at one end — exactly the shape the `Sg`/`m` builders treat
    // as splittable.
    let c = mol("CCCO");
    let keep = [false, false, true, true];
    let defs = floating_defs(&c, &[0, 1], &keep);
    assert_eq!(defs.len(), 2, "a chain of two becomes two defs");
    assert!(!defs[0].split, "the first half carries the attachment");
    assert!(defs[1].split, "the second half is the split variant");
}

#[test]
fn a_branched_floating_group_is_not_split() {
    // CC(C)C: the central carbon with three carbon neighbours, so the
    // attachment atom has two neighbours inside the fragment. `count() == 1`
    // is what refuses the split — a branch has no meaningful "rest".
    let c = mol("CC(C)C");
    let keep = [true, false, false, false, true];
    let defs = floating_defs(&c, &[0, 2, 3], &keep);
    assert_eq!(defs.len(), 1, "a branch stays whole");
    assert!(!defs[0].split, "and is not the split variant");
}

// ── equiv_positions: the empty fallback ────────────────────────────────

#[test]
fn an_unmatched_scaffold_offers_every_position() {
    // Nothing matched anywhere, so there is no evidence of a preferred
    // site and the answer is every scaffold atom. This is the `is_empty`
    // branch, and an empty answer here would produce a `m:` field with no
    // positions at all.
    let c = mol("CC");
    let scaffold_q = molecule_to_query(&c);
    let positions = equiv_positions(&[], std::slice::from_ref(&c), &scaffold_q);
    assert_eq!(
        positions,
        vec![0, 1],
        "with no evidence, every scaffold atom is a candidate"
    );
}

// ── build_base_smiles / write_fragment_first ────────────────────────────

#[test]
fn a_fragment_is_written_with_a_star_and_the_attachment_first() {
    // The `[*]` prefix is what marks the attachment, and the attachment has
    // to be the fragment's atom 0 or the prefix lies about which end bonds
    // to the scaffold.
    let m = mol("CCO");
    let (atoms, bonds, attach) = extract_subgraph(&m, &[1, 2]);
    let (atoms, bonds, attach) = reorder_attachment_first(atoms, bonds, attach);
    let def = FloatingDef {
        atoms,
        bonds,
        attachment: attach,
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
    // One def against a two-atom scaffold: the `*` is atom 2 and the
    // attachment atom 3. These are the numbers that go into the `m:`
    // field, so an off-by-one here is a wrong CX-SMILES string that still
    // looks well formed.
    let m = mol("CO");
    let (atoms, bonds, attach) = extract_subgraph(&m, &[1]);
    let def = FloatingDef {
        atoms,
        bonds,
        attachment: attach,
        split: false,
    };
    let (_smiles, star_idx, attach_idx) = build_base_smiles("CO", &[def], 2);
    assert_eq!(star_idx, vec![2], "the star follows the scaffold");
    assert_eq!(attach_idx, vec![3], "and the attachment follows the star");
}

#[test]
fn two_defs_skip_past_each_fragments_star_and_atoms() {
    // The second `*` sits after the first fragment *and* its `*`, so the
    // indices are not consecutive: scaffold 1, then the first star at 1 and
    // its one fragment atom at 2, so the second star is at 3. Writing them
    // as consecutive would put the second `*` on top of the first
    // fragment's atom — a well-formed CX-SMILES field that is wrong.
    let m = mol("CO");
    let (atoms, bonds, attach) = extract_subgraph(&m, &[1]);
    let one = FloatingDef {
        atoms,
        bonds,
        attachment: attach,
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

// ── build_positional ────────────────────────────────────────────────────

#[test]
fn an_empty_group_is_an_error_and_not_a_panic() {
    let err = build_positional(&[]).expect_err("there is nothing to build from");
    assert!(
        err.to_string().contains("group is empty"),
        "the error says which precondition failed: {err}"
    );
}

// ── a chain, for the fixture helpers above ──────────────────────────────

#[test]
fn the_chain_helper_is_the_indices_in_order() {
    assert_eq!(chain(3), vec![0, 1, 2], "0, 1, 2");
}
