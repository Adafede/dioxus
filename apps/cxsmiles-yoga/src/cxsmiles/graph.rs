// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

//! Graph helpers shared by the positional and repeating builders.
//!
//! These bridge `chematic`'s `Molecule`/`QueryMolecule` representations: converting
//! a molecule to a query, extracting connected subgraphs, and scoring a query
//! match by how many scaffold atoms it recovers.

use chematic::core::{AtomIdx, BondOrder, Molecule, MoleculeBuilder};
use chematic::smarts::{
    AtomPrimitive, AtomQuery, BondPrimitive, BondQuery, QueryAtom, QueryBond, QueryMolecule,
    find_matches,
};
use rustc_hash::FxHashMap;
use std::collections::HashSet;

use super::types::CxError;

/// A single embedding of a query into a molecule: query-atom index → molecule atom.
pub(crate) type Match = FxHashMap<usize, AtomIdx>;

/// `Molecule` → `QueryMolecule` (element + bond-order). Atom `i` of the query
/// corresponds to atom `i` of the molecule.
pub(crate) fn molecule_to_query(mol: &Molecule) -> QueryMolecule {
    let n = mol.atom_count();
    let atoms: Vec<QueryAtom> = (0..n as u32)
        .map(|i| {
            let a = mol.atom(AtomIdx(i));
            QueryAtom {
                query: AtomQuery::Primitive(AtomPrimitive::AtomicNum(a.element.atomic_number())),
                atom_map: a.atom_map,
            }
        })
        .collect();
    let mut bonds: Vec<QueryBond> = Vec::new();
    let mut adj: Vec<Vec<(usize, usize)>> = vec![Vec::new(); n];
    for (_, be) in mol.bonds() {
        bonds.push(QueryBond {
            atom1: be.atom1.0 as usize,
            atom2: be.atom2.0 as usize,
            query: BondQuery::Primitive(bond_to_primitive(be.order)),
        });
    }
    // A second pass over `bonds` rather than filling `adj` while pushing: the
    // adjacency list of atom `a` is bond indices, so it is clearer to read them
    // back off the bond list than to maintain two structures in lockstep.
    for (qi, bond) in bonds.iter().enumerate() {
        if let Some(neighbours) = adj.get_mut(bond.atom1) {
            neighbours.push((qi, bond.atom2));
        }
        if let Some(neighbours) = adj.get_mut(bond.atom2) {
            neighbours.push((qi, bond.atom1));
        }
    }
    QueryMolecule { atoms, bonds, adj }
}

pub(crate) const fn bond_to_primitive(order: BondOrder) -> BondPrimitive {
    match order {
        BondOrder::Single => BondPrimitive::Single,
        BondOrder::Double => BondPrimitive::Double,
        BondOrder::Triple => BondPrimitive::Triple,
        BondOrder::Aromatic => BondPrimitive::Aromatic,
        _ => BondPrimitive::Any,
    }
}

/// Build a molecule from the subset of `mol`'s atoms and the bonds between them.
pub(crate) fn subgraph(mol: &Molecule, keep: &[bool]) -> Molecule {
    let n = mol.atom_count();
    let mut b = MoleculeBuilder::new();
    let mut map: Vec<Option<AtomIdx>> = vec![None; n];
    for (i, &keep_atom) in keep.iter().enumerate() {
        if keep_atom && let Some(slot) = map.get_mut(i) {
            *slot = Some(b.add_atom(mol.atom(AtomIdx(i as u32)).clone()));
        }
    }
    for (_, be) in mol.bonds() {
        // Both endpoints are mapped above (keep[a] && keep[c] ⇒ map is Some).
        if let (Some(ma), Some(mc)) = (
            map.get(be.atom1.0 as usize).copied().flatten(),
            map.get(be.atom2.0 as usize).copied().flatten(),
        ) {
            let _ = b.add_bond(ma, mc, be.order);
        }
    }
    b.build()
}

/// Connected components of `atoms` (u32 indices) using `mol`'s internal edges.
pub(crate) fn components(atoms: &[u32], mol: &Molecule) -> Vec<Vec<u32>> {
    let set: HashSet<u32> = atoms.iter().copied().collect();
    let mut seen: HashSet<u32> = HashSet::new();
    let mut comps: Vec<Vec<u32>> = Vec::new();
    for &start in atoms {
        if seen.contains(&start) {
            continue;
        }
        let mut comp: Vec<u32> = Vec::new();
        let mut stack = vec![start];
        seen.insert(start);
        while let Some(cur) = stack.pop() {
            comp.push(cur);
            for (nbr, _) in mol.neighbors(AtomIdx(cur)) {
                if set.contains(&nbr.0) && !seen.contains(&nbr.0) {
                    seen.insert(nbr.0);
                    stack.push(nbr.0);
                }
            }
        }
        comps.push(comp);
    }
    comps
}

/// The first embedding of `q` into `mol`, or an error if there is none.
///
/// There is deliberately no ranking here. An embedding maps every query atom
/// onto a distinct molecule atom, so every hit of one query has the same
/// length and there is no better one to prefer; `min_by_key` over that length
/// picked the first entry every time, having computed a constant three times
/// over. Picking the first directly says the same thing in one line. If the
/// matcher ever grows partial embeddings, this is where the preference belongs,
/// and `tests::embedding_size` is the test that says why there is not one now.
pub(crate) fn best_match(q: &QueryMolecule, mol: &Molecule) -> Result<Match, CxError> {
    find_matches(q, mol)
        .into_iter()
        .next()
        .ok_or_else(|| CxError("no match of query into molecule".into()))
}

pub(crate) fn matched_mask(h: &Match, mol: &Molecule) -> Vec<bool> {
    let mut m = vec![false; mol.atom_count()];
    for &a in h.values() {
        if let Some(slot) = m.get_mut(a.0 as usize) {
            *slot = true;
        }
    }
    m
}

pub(crate) fn unmatched_atoms(matched: &[bool]) -> Vec<u32> {
    matched
        .iter()
        .enumerate()
        .filter(|&(_, &m)| !m)
        .map(|(i, _)| i as u32)
        .collect()
}

#[cfg(test)]
// `expect` on a fixture that stopped parsing is a broken test, not a defect in
// the code under test; the out-of-range reads below are the assertions failing.
#[allow(clippy::indexing_slicing, clippy::expect_used)]
mod tests {
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
}

#[cfg(test)]
// A fixture that stopped parsing is a broken test, not a defect in the code
// under test.
#[allow(clippy::expect_used)]
mod embedding_size {
    use super::*;
    use chematic::smarts::find_matches;
    use chematic::smiles::parse;

    /// Every embedding of one query covers the same number of atoms.
    ///
    /// This is the reason `best_match` below takes the first hit rather than
    /// searching for a better one: `find_matches` maps each query atom onto a
    /// distinct molecule atom, so a hit's length is `q.atoms.len()` whatever
    /// the embedding, and there is no "best" to rank. The test is here because
    /// that is an invariant of the matcher, not of this function, and the
    /// arithmetic that used to rank hits was computing a constant.
    #[test]
    fn every_embedding_of_a_query_has_the_same_size() {
        // "CC" into "CCCC" has three embeddings, and each maps both query atoms.
        let q = molecule_to_query(&parse("CC").expect("valid SMILES"));
        let mol = parse("CCCC").expect("valid SMILES");
        let hits = find_matches(&q, &mol);
        assert!(hits.len() > 1, "the fixture must have several embeddings");
        for hit in &hits {
            assert_eq!(
                hit.len(),
                q.atoms.len(),
                "an embedding covers every query atom"
            );
        }
    }

    /// The same for a query with a branch, where the embeddings differ in which
    /// atom they start from.
    #[test]
    fn a_branched_query_still_has_uniformly_sized_embeddings() {
        let q = molecule_to_query(&parse("CC(C)C").expect("valid SMILES"));
        let mol = parse("CC(C)CC(C)C").expect("valid SMILES");
        let hits = find_matches(&q, &mol);
        assert!(hits.len() > 1, "the fixture must have several embeddings");
        for hit in &hits {
            assert_eq!(hit.len(), q.atoms.len(), "one molecule atom per query atom");
        }
    }
}
