// The `tests` tests, extracted from `graph.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `graph` module, so
// `use super::*` below reaches exactly what it did before the move.
use super::*;
use chematic::smiles::parse;

fn mol(smiles: &str) -> Molecule {
    parse(smiles).expect("the fixtures in this module are valid SMILES")
}

// ── bond_to_primitive: every arm ─────────────────────────────────────────

#[test]
fn every_bond_order_maps_to_its_own_primitive() {
    // The four named orders are the whole point of the conversion, and the
    // wildcard is the one that would silently absorb a new `BondOrder`
    // variant, so each is asserted separately.
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

// ── molecule_to_query ───────────────────────────────────────────────────

#[test]
fn a_query_has_one_atom_per_molecule_atom() {
    let c = mol("CCO");
    let q = molecule_to_query(&c);
    assert_eq!(q.atoms.len(), 3, "one query atom per molecule atom");
}

#[test]
fn a_query_has_one_bond_and_two_adjacency_entries_per_molecule_bond() {
    // The adjacency list is what the matcher's traversal walks, so a bond
    // missing from it is a bond the query can never traverse even though
    // `bonds` says it is there.
    let c = mol("CCO");
    let q = molecule_to_query(&c);
    assert_eq!(q.bonds.len(), 2, "CCO has two bonds");
    let entries: usize = q.adj.iter().map(Vec::len).sum();
    assert_eq!(entries, 4, "each bond is listed from both of its ends");
}

#[test]
fn adjacency_is_symmetric() {
    // Every bond must appear in the neighbour list of both its atoms. The
    // second pass over `bonds` is what fills it, and a single-ended entry
    // would make the traversal depend on which atom it started from.
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
    // The second pass reads `bond.atom1`/`atom2` back out of the bond list;
    // a stale index there would push into the wrong row or miss the vector.
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

// ── subgraph ────────────────────────────────────────────────────────────

#[test]
fn a_subgraph_keeps_exactly_the_asked_for_atoms() {
    let c = mol("CCO");
    let keep = [true, true, false];
    let s = subgraph(&c, &keep);
    assert_eq!(s.atom_count(), 2, "two atoms were kept");
}

#[test]
fn a_subgraph_keeps_only_bonds_between_kept_atoms() {
    // CCO keeping the last two carbons: one bond survives, the bond to the
    // dropped oxygen does not. Both endpoints have to be mapped, which is
    // what the `if let (Some, Some)` guards.
    let c = mol("CCO");
    let keep = [false, true, true];
    let s = subgraph(&c, &keep);
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
    // The mask is indexed per atom and `map` is sized to the molecule, so a
    // mask that stops short must drop the atoms it does not reach rather
    // than read past its own end.
    let c = mol("CCO");
    let s = subgraph(&c, &[true]);
    assert_eq!(s.atom_count(), 1, "only the first atom was asked for");
}

// ── matched_mask / unmatched_atoms ──────────────────────────────────────

#[test]
fn a_mask_comes_back_in_atom_order() {
    // `unmatched_atoms` is the inverse of `matched_mask`: what one sets, the
    // other reports, in ascending order, and the two together must account
    // for every atom exactly once.
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
    // The mask is sized to the molecule, so a hit index the molecule does
    // not have has nowhere to go. It is skipped: an out-of-bounds write
    // here would be a panic on a caller-supplied match.
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

// ── components ──────────────────────────────────────────────────────────

#[test]
fn a_chain_is_one_component() {
    let c = mol("CCC");
    let comps = components(&[0, 1, 2], &c);
    assert_eq!(comps.len(), 1, "the whole chain is connected");
}

#[test]
fn two_fragments_are_two_components() {
    // CCC with the middle carbon excluded from the set: the two ends are
    // not connected *through* the set, so they are two components even
    // though the molecule has the bond between them.
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
    // The total across components must equal the input, with no atom lost
    // and none counted twice — the property that makes the split usable as
    // a partition.
    let c = mol("CCCCO");
    let asked = [0, 1, 2, 4];
    let comps = components(&asked, &c);
    let mut seen: Vec<u32> = comps.concat();
    seen.sort_unstable();
    assert_eq!(seen, asked, "a partition of exactly what was asked for");
}

// ── best_match ──────────────────────────────────────────────────────────

#[test]
fn no_embedding_is_an_error_and_not_an_empty_match() {
    // An empty `Match` would report "everything unmatched", which reads as
    // a result. There is no embedding here, and that has to be an error.
    let q = molecule_to_query(&mol("C"));
    let err = best_match(&q, &mol("O")).expect_err("carbon does not match oxygen");
    assert!(
        err.to_string().contains("no match"),
        "the error says what failed: {err}"
    );
}
