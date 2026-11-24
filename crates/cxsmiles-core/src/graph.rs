// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

//! Graph helpers shared by positional, repeating, and round-trip modules.

use chematic::core::{AtomIdx, BondOrder, Molecule, MoleculeBuilder};
use chematic::smarts::{
    AtomPrimitive, AtomQuery, BondPrimitive, BondQuery, QueryAtom, QueryBond, QueryMolecule,
    find_matches,
};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::error::CxError;

/// A single embedding of a query into a molecule: query-atom index -> molecule atom.
pub(crate) type Match = FxHashMap<usize, AtomIdx>;

pub(crate) fn atom_idx(index: usize) -> AtomIdx {
    AtomIdx(u32::try_from(index).unwrap_or(u32::MAX))
}

pub(crate) fn atom_usize(idx: AtomIdx) -> usize {
    usize::try_from(idx.0).unwrap_or(usize::MAX)
}

pub(crate) fn usize_to_f64(value: usize) -> f64 {
    f64::from(u32::try_from(value).unwrap_or(u32::MAX))
}

/// Convert `Molecule` to `QueryMolecule` (element + bond-order).
pub(crate) fn molecule_to_query(mol: &Molecule) -> QueryMolecule {
    let n = mol.atom_count();
    let atoms: Vec<QueryAtom> = (0..n)
        .map(|i| {
            let a = mol.atom(atom_idx(i));
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
            atom1: atom_usize(be.atom1),
            atom2: atom_usize(be.atom2),
            query: BondQuery::Primitive(bond_to_primitive(be.order)),
        });
    }
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

/// Map bond order to a SMARTS bond primitive.
pub(crate) const fn bond_to_primitive(order: BondOrder) -> BondPrimitive {
    match order {
        BondOrder::Single => BondPrimitive::Single,
        BondOrder::Double => BondPrimitive::Double,
        BondOrder::Triple => BondPrimitive::Triple,
        BondOrder::Aromatic => BondPrimitive::Aromatic,
        _ => BondPrimitive::Any,
    }
}

/// Build a molecule from the kept subset of `mol`'s atoms and internal bonds.
pub(crate) fn subgraph(mol: &Molecule, keep: &[bool]) -> Molecule {
    let n = mol.atom_count();
    let mut b = MoleculeBuilder::new();
    let mut map: Vec<Option<AtomIdx>> = vec![None; n];
    for (i, &keep_atom) in keep.iter().enumerate() {
        if keep_atom && let Some(slot) = map.get_mut(i) {
            *slot = Some(b.add_atom(mol.atom(atom_idx(i)).clone()));
        }
    }
    for (_, be) in mol.bonds() {
        if let (Some(ma), Some(mc)) = (
            map.get(atom_usize(be.atom1)).copied().flatten(),
            map.get(atom_usize(be.atom2)).copied().flatten(),
        ) {
            let _ = b.add_bond(ma, mc, be.order);
        }
    }
    b.build()
}

/// Connected components of `atoms` (`u32` indices) along `mol` edges.
pub(crate) fn components(atoms: &[u32], mol: &Molecule) -> Vec<Vec<u32>> {
    let set: FxHashSet<u32> = atoms.iter().copied().collect();
    let mut seen: FxHashSet<u32> = FxHashSet::default();
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
            stack.extend(
                mol.neighbors(AtomIdx(cur))
                    .map(|(nbr, _)| nbr.0)
                    .filter(|&n| set.contains(&n) && seen.insert(n)),
            );
        }
        comps.push(comp);
    }
    comps
}

/// First embedding of `q` into `mol`, or [`CxError::NoScaffoldMatch`].
pub(crate) fn best_match(q: &QueryMolecule, mol: &Molecule) -> Result<Match, CxError> {
    find_matches(q, mol)
        .into_iter()
        .next()
        .ok_or(CxError::NoScaffoldMatch)
}

/// Boolean mask marking which molecule atoms participate in `hit`.
pub(crate) fn matched_mask(hit: &Match, mol: &Molecule) -> Vec<bool> {
    let mut mask = vec![false; mol.atom_count()];
    for &a in hit.values() {
        if let Some(slot) = mask.get_mut(atom_usize(a)) {
            *slot = true;
        }
    }
    mask
}

/// Atom indices in ascending order whose `matched` flag is `false`.
pub(crate) fn unmatched_atoms(matched: &[bool]) -> Vec<u32> {
    matched
        .iter()
        .enumerate()
        .filter(|(_, is_matched)| !**is_matched)
        .map(|(i, _)| u32::try_from(i).unwrap_or(u32::MAX))
        .collect()
}

/// Sorted atomic-number multiset written into `out`, reusing existing capacity.
pub(crate) fn multiset_into(atoms: &[u32], mol: &Molecule, out: &mut Vec<u8>) {
    out.clear();
    out.extend(
        atoms
            .iter()
            .map(|&i| mol.atom(AtomIdx(i)).element.atomic_number()),
    );
    out.sort_unstable();
}

/// Sorted atomic-number multiset of `atoms` in `mol`.
pub(crate) fn multiset(atoms: &[u32], mol: &Molecule) -> Vec<u8> {
    let mut out = Vec::with_capacity(atoms.len());
    multiset_into(atoms, mol, &mut out);
    out
}
