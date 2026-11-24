// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

use chematic::core::{AtomIdx, BondOrder};
use chematic::smarts::{BondPrimitive, find_matches};

use super::mol;
use crate::graph::{
    Match, best_match, bond_to_primitive, components, matched_mask, molecule_to_query, subgraph,
    unmatched_atoms,
};

#[test]
fn every_bond_order_maps_to_its_own_primitive() {
    assert_eq!(
        bond_to_primitive(BondOrder::Single),
        BondPrimitive::Single,
        "single"
    );
    assert_eq!(
        bond_to_primitive(BondOrder::Double),
        BondPrimitive::Double,
        "double"
    );
    assert_eq!(
        bond_to_primitive(BondOrder::Triple),
        BondPrimitive::Triple,
        "triple"
    );
    assert_eq!(
        bond_to_primitive(BondOrder::Aromatic),
        BondPrimitive::Aromatic,
        "aromatic"
    );
}

#[test]
fn a_query_has_one_atom_per_molecule_atom() {
    let c = mol("CCO");
    let q = molecule_to_query(&c);
    assert_eq!(q.atoms.len(), 3, "one query atom per molecule atom");
}

#[test]
fn a_query_has_one_bond_and_two_adjacency_entries_per_molecule_bond() {
    let c = mol("CCO");
    let q = molecule_to_query(&c);
    assert_eq!(q.bonds.len(), 2, "CCO has two bonds");
    let entries: usize = q.adj.iter().map(Vec::len).sum();
    assert_eq!(entries, 4, "each bond is listed from both of its ends");
}

#[test]
fn adjacency_is_symmetric() {
    let c = mol("CCO");
    let q = molecule_to_query(&c);
    for (qi, bond) in q.bonds.iter().enumerate() {
        let from_a = q.adj[bond.atom1]
            .iter()
            .any(|(b, other)| *b == qi && *other == bond.atom2);
        let from_b = q.adj[bond.atom2]
            .iter()
            .any(|(b, other)| *b == qi && *other == bond.atom1);
        assert!(from_a, "bond {qi} is listed from its first atom");
        assert!(from_b, "bond {qi} is listed from its second atom");
    }
}

#[test]
fn adjacency_neighbours_stay_inside_the_atom_list() {
    let c = mol("CCO");
    let q = molecule_to_query(&c);
    for (atom, neighbours) in q.adj.iter().enumerate() {
        for (_, other) in neighbours {
            assert!(
                *other < q.atoms.len(),
                "atom {atom} points at atom {other}, past the {} it has",
                q.atoms.len()
            );
        }
    }
}

#[test]
fn a_subgraph_keeps_exactly_the_asked_for_atoms() {
    let c = mol("CCO");
    let s = subgraph(&c, &[true, true, false]);
    assert_eq!(s.atom_count(), 2, "two atoms were kept");
}

#[test]
fn a_subgraph_keeps_only_bonds_between_kept_atoms() {
    let c = mol("CCO");
    let s = subgraph(&c, &[false, true, true]);
    assert_eq!(s.atom_count(), 2, "two atoms");
    assert_eq!(s.bonds().count(), 1, "and only the bond between them");
}

#[test]
fn keeping_nothing_yields_an_empty_molecule() {
    let c = mol("CCO");
    let s = subgraph(&c, &[false; 3]);
    assert_eq!(s.atom_count(), 0, "nothing kept");
    assert_eq!(s.bonds().count(), 0, "so no bond can survive either");
}

#[test]
fn a_shorter_mask_keeps_only_what_it_names() {
    let c = mol("CCO");
    let s = subgraph(&c, &[true]);
    assert_eq!(s.atom_count(), 1, "only the first atom was asked for");
}

#[test]
fn a_mask_comes_back_in_atom_order() {
    let c = mol("CCO");
    let mut hit: Match = Match::default();
    hit.insert(0, AtomIdx(1));
    hit.insert(1, AtomIdx(0));
    let mask = matched_mask(&hit, &c);
    assert_eq!(mask, vec![true, true, false], "atoms 0 and 1 are matched");
    assert_eq!(unmatched_atoms(&mask), vec![2], "and only atom 2 is not");
}

#[test]
fn an_empty_mask_leaves_every_atom_unmatched() {
    let c = mol("CCO");
    let mask = matched_mask(&Match::default(), &c);
    assert!(
        !mask.iter().any(|m| *m),
        "nothing matched, so no atom is set"
    );
    assert_eq!(
        unmatched_atoms(&mask),
        vec![0, 1, 2],
        "and all three are reported, in order"
    );
}

#[test]
fn a_hit_mentioning_an_atom_past_the_end_is_dropped_not_panicked_on() {
    let c = mol("CCO");
    let mut hit: Match = Match::default();
    hit.insert(0, AtomIdx(9));
    let mask = matched_mask(&hit, &c);
    assert_eq!(
        mask,
        vec![false; 3],
        "the unreachable atom is skipped and the rest is untouched"
    );
}

#[test]
fn a_chain_is_one_component() {
    let c = mol("CCC");
    let comps = components(&[0, 1, 2], &c);
    assert_eq!(comps.len(), 1, "the whole chain is connected");
}

#[test]
fn two_fragments_are_two_components() {
    let c = mol("CCC");
    let comps = components(&[0, 2], &c);
    assert_eq!(comps.len(), 2, "an excluded atom separates them");
}

#[test]
fn a_ring_is_one_component() {
    let c = mol("C1CC1");
    let comps = components(&[0, 1, 2], &c);
    assert_eq!(comps.len(), 1, "a ring is connected");
}

#[test]
fn asking_for_nothing_yields_nothing() {
    let c = mol("CCC");
    assert!(
        components(&[], &c).is_empty(),
        "no atoms means no components"
    );
}

#[test]
fn every_asked_for_atom_appears_in_exactly_one_component() {
    let c = mol("CCCCO");
    let asked = [0, 1, 2, 4];
    let comps = components(&asked, &c);
    let mut seen: Vec<u32> = comps.concat();
    seen.sort_unstable();
    assert_eq!(seen, asked, "a partition of exactly what was asked for");
}

#[test]
fn no_embedding_is_an_error_and_not_an_empty_match() {
    let q = molecule_to_query(&mol("C"));
    let err = best_match(&q, &mol("O")).expect_err("carbon does not match oxygen");
    assert!(
        err.to_string().contains("no match"),
        "the error says what failed: {err}"
    );
}

#[test]
fn every_embedding_of_a_query_has_the_same_size() {
    let q = molecule_to_query(&mol("CC"));
    let m = mol("CCCC");
    let hits = find_matches(&q, &m);
    assert!(hits.len() > 1, "the fixture must have several embeddings");
    for hit in &hits {
        assert_eq!(
            hit.len(),
            q.atoms.len(),
            "an embedding covers every query atom"
        );
    }
}

#[test]
fn a_branched_query_still_has_uniformly_sized_embeddings() {
    let q = molecule_to_query(&mol("CC(C)C"));
    let m = mol("CC(C)CC(C)C");
    let hits = find_matches(&q, &m);
    assert!(hits.len() > 1, "the fixture must have several embeddings");
    for hit in &hits {
        assert_eq!(hit.len(), q.atoms.len(), "one molecule atom per query atom");
    }
}
