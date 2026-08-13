// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

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
pub type Match = FxHashMap<usize, AtomIdx>;

/// `Molecule` → `QueryMolecule` (element + bond-order). Atom `i` of the query
/// corresponds to atom `i` of the molecule.
pub fn molecule_to_query(mol: &Molecule) -> QueryMolecule {
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
        let a = be.atom1.0 as usize;
        let b = be.atom2.0 as usize;
        let qi = bonds.len();
        bonds.push(QueryBond {
            atom1: a,
            atom2: b,
            query: BondQuery::Primitive(bond_to_primitive(be.order)),
        });
        adj[a].push((qi, b));
        adj[b].push((qi, a));
    }
    QueryMolecule { atoms, bonds, adj }
}

pub const fn bond_to_primitive(order: BondOrder) -> BondPrimitive {
    match order {
        BondOrder::Single => BondPrimitive::Single,
        BondOrder::Double => BondPrimitive::Double,
        BondOrder::Triple => BondPrimitive::Triple,
        BondOrder::Aromatic => BondPrimitive::Aromatic,
        _ => BondPrimitive::Any,
    }
}

/// Build a molecule from the subset of `mol`'s atoms and the bonds between them.
pub fn subgraph(mol: &Molecule, keep: &[bool]) -> Molecule {
    let n = mol.atom_count();
    let mut b = MoleculeBuilder::new();
    let mut map: Vec<Option<AtomIdx>> = vec![None; n];
    for i in 0..n as u32 {
        if keep[i as usize] {
            map[i as usize] = Some(b.add_atom(mol.atom(AtomIdx(i)).clone()));
        }
    }
    for (_, be) in mol.bonds() {
        let (a, c) = (be.atom1.0 as usize, be.atom2.0 as usize);
        if keep[a] && keep[c] {
            let _ = b.add_bond(
                map[a].expect("keep[a] guarantees the atom was mapped"),
                map[c].expect("keep[c] guarantees the atom was mapped"),
                be.order,
            );
        }
    }
    b.build()
}

/// Connected components of `atoms` (u32 indices) using `mol`'s internal edges.
pub fn components(atoms: &[u32], mol: &Molecule) -> Vec<Vec<u32>> {
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

/// One embedding of a query into `mol`, preferring the one that recovers the
/// most scaffold atoms (fewest unmatched).
pub fn best_match(q: &QueryMolecule, mol: &Molecule) -> Result<Match, CxError> {
    let hits = find_matches(q, mol);
    if hits.is_empty() {
        return Err(CxError("no match of query into molecule".into()));
    }
    Ok(hits
        .iter()
        .min_by_key(|h| mol.atom_count() - h.len())
        .cloned()
        .expect("hits non-empty: the empty case returns Err early above"))
}

pub fn unmatched_count(h: &Match, mol: &Molecule) -> usize {
    mol.atom_count() - h.len()
}

pub fn matched_mask(h: &Match, mol: &Molecule) -> Vec<bool> {
    let mut m = vec![false; mol.atom_count()];
    for &a in h.values() {
        m[a.0 as usize] = true;
    }
    m
}

pub fn unmatched_atoms(matched: &[bool]) -> Vec<u32> {
    (0..matched.len() as u32)
        .filter(|i| !matched[*i as usize])
        .collect()
}
