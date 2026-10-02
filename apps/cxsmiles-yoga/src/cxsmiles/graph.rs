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
mod tests;

#[cfg(test)]
// A fixture that stopped parsing is a broken test, not a defect in the code
// under test.
#[allow(clippy::expect_used)]
mod embedding_size;
