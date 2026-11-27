// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

//! Shared fragment extraction, ordering, and SMILES assembly helpers.
//!
//! Breaking these primitives out of `positional` keeps the module graph
//! strictly acyclic (`roundtrip` and `positional` both depend on `fragment`).

use chematic::core::{Atom, AtomIdx, BondOrder, Molecule, MoleculeBuilder};
use chematic::smiles::write;
use rustc_hash::FxHashMap;

use crate::graph::atom_usize;

/// A floating fragment ready for serialisation and expansion.
#[derive(Clone)]
pub(crate) struct FloatingDef {
    pub(crate) atoms: Vec<Atom>,
    pub(crate) bonds: Vec<(usize, usize, BondOrder)>,
    /// Index into `atoms`, bonded to the scaffold or parent split atom.
    pub(crate) attachment: usize,
    /// Bond order joining `attachment` to its target.
    pub(crate) attach_order: BondOrder,
    /// `true` when emitted as the second half of a split (double-`m`) group.
    pub(crate) split: bool,
}

const fn bond_symbol(order: BondOrder) -> &'static str {
    match order {
        BondOrder::Double => "=",
        BondOrder::Triple => "#",
        _ => "",
    }
}

/// Write a fragment molecule with the attachment atom at index 0.
pub(crate) fn write_fragment_first(def: &FloatingDef) -> String {
    if def.atoms.is_empty() {
        return String::new();
    }
    let mut b = MoleculeBuilder::new();
    if def.attachment == 0 || def.attachment >= def.atoms.len() {
        drop(add_atoms_and_bonds(&mut b, &def.atoms, &def.bonds));
    } else {
        let (atoms, bonds, _) =
            reorder_attachment_first(def.atoms.clone(), def.bonds.clone(), def.attachment);
        drop(add_atoms_and_bonds(&mut b, &atoms, &bonds));
    }
    write(&b.build())
}

/// Floating fragment SMILES for display and base assembly: `[*]<bond><frag>`.
pub(crate) fn fragment_smiles(def: &FloatingDef) -> String {
    format!(
        "[*]{}{}",
        bond_symbol(def.attach_order),
        write_fragment_first(def)
    )
}

/// Assemble base SMILES plus star and attachment atom indices for `m:` blocks
/// using precomputed fragment SMILES aligned 1:1 with `defs`.
pub(crate) fn build_base_smiles_with_frags(
    scaffold_smiles: &str,
    defs: &[FloatingDef],
    frag_smiles: &[String],
    scaffold_len: usize,
) -> (String, Vec<usize>, Vec<usize>) {
    let total_len =
        scaffold_smiles.len() + frag_smiles.iter().map(|frag| 1 + frag.len()).sum::<usize>();
    let mut out = String::with_capacity(total_len);
    out.push_str(scaffold_smiles);
    let mut star_idx: Vec<usize> = Vec::with_capacity(defs.len());
    let mut attach_idx: Vec<usize> = Vec::with_capacity(defs.len());
    let mut offset = scaffold_len;
    for (def, frag) in defs.iter().zip(frag_smiles) {
        star_idx.push(offset);
        attach_idx.push(offset + 1);
        offset += 1 + def.atoms.len();
        out.push('.');
        out.push_str(frag);
    }
    (out, star_idx, attach_idx)
}

/// Assemble base SMILES plus star and attachment atom indices for `m:` blocks.
#[cfg(test)]
pub(crate) fn build_base_smiles(
    scaffold_smiles: &str,
    defs: &[FloatingDef],
    scaffold_len: usize,
) -> (String, Vec<usize>, Vec<usize>) {
    let frag_smiles: Vec<String> = defs.iter().map(fragment_smiles).collect();
    build_base_smiles_with_frags(scaffold_smiles, defs, &frag_smiles, scaffold_len)
}

/// Reorder `(atoms, bonds, attachment)` so that the attachment atom is index 0.
pub(crate) fn reorder_attachment_first(
    atoms: Vec<Atom>,
    bonds: Vec<(usize, usize, BondOrder)>,
    attach: usize,
) -> (Vec<Atom>, Vec<(usize, usize, BondOrder)>, usize) {
    let n = atoms.len();
    if attach == 0 || attach >= n {
        return (atoms, bonds, attach);
    }
    let order: Vec<usize> = std::iter::once(attach)
        .chain((0..n).filter(|&i| i != attach))
        .collect();
    let mut map = vec![0usize; n];
    for (new_i, &old_i) in order.iter().enumerate() {
        if let Some(slot) = map.get_mut(old_i) {
            *slot = new_i;
        }
    }
    let new_atoms: Vec<Atom> = order
        .iter()
        .filter_map(|&i| atoms.get(i).cloned())
        .collect();
    let new_bonds: Vec<(usize, usize, BondOrder)> = bonds
        .iter()
        .filter_map(|&(a, c, o)| Some((*map.get(a)?, *map.get(c)?, o)))
        .collect();
    (new_atoms, new_bonds, 0)
}

/// Extract a subgraph (atoms cloned + internal bonds) of `mol` for `subset`.
pub(crate) fn extract_subgraph(
    mol: &Molecule,
    subset: &[u32],
) -> (Vec<Atom>, Vec<(usize, usize, BondOrder)>, usize) {
    let map: FxHashMap<u32, usize> = subset
        .iter()
        .copied()
        .enumerate()
        .map(|(i, a)| (a, i))
        .collect();
    let atoms: Vec<Atom> = subset
        .iter()
        .map(|&a| mol.atom(AtomIdx(a)).clone())
        .collect();
    let mut bonds = Vec::new();
    for (_, be) in mol.bonds() {
        if let (Some(&a), Some(&c)) = (map.get(&be.atom1.0), map.get(&be.atom2.0)) {
            bonds.push((a, c, be.order));
        }
    }
    let attach = subset
        .iter()
        .position(|&a| {
            mol.neighbors(AtomIdx(a))
                .any(|(n, _)| !map.contains_key(&n.0) && n.0 != a)
        })
        .unwrap_or(0);
    (atoms, bonds, attach)
}

/// Add `atoms` to `builder`, then bond them by local index pairs.
pub(crate) fn add_atoms_and_bonds(
    builder: &mut MoleculeBuilder,
    atoms: &[Atom],
    bonds: &[(usize, usize, BondOrder)],
) -> Vec<AtomIdx> {
    let idx: Vec<AtomIdx> = atoms.iter().map(|a| builder.add_atom(a.clone())).collect();
    for &(a1, a2, order) in bonds {
        if let (Some(&x), Some(&y)) = (idx.get(a1), idx.get(a2)) {
            let _ = builder.add_bond(x, y, order);
        }
    }
    idx
}

/// Check whether `comp`'s sorted element multiset matches `target`.
pub(crate) fn matches_multiset(
    comp: &[u32],
    mol: &Molecule,
    target: &[u8],
    scratch: &mut Vec<u8>,
) -> bool {
    crate::graph::multiset_into(comp, mol, scratch);
    scratch.as_slice() == target
}

/// Build an O(1) lookup table from molecule atom index to query atom index.
pub(crate) fn reverse_match_table(hit: &crate::graph::Match, mol: &Molecule) -> Vec<Option<usize>> {
    let mut rev = vec![None; mol.atom_count()];
    for (&qi, &target_atom) in hit {
        if let Some(slot) = rev.get_mut(atom_usize(target_atom)) {
            *slot = Some(qi);
        }
    }
    rev
}
