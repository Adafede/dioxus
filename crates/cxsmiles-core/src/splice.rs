// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

//! Repeating-unit splicing for `Sg:n:` expansion.

use chematic::core::{AtomIdx, BondOrder, Molecule, MoleculeBuilder};

use crate::graph::atom_usize;

fn collect_anchors(scaffold: &Molecule, unit: &[u32]) -> (u32, u32) {
    let mut anchors: Vec<u32> = unit
        .iter()
        .flat_map(|&a| {
            scaffold
                .neighbors(AtomIdx(a))
                .filter(|(nb, _)| !unit.contains(&nb.0))
                .map(|(nb, _)| nb.0)
        })
        .collect();
    anchors.sort_unstable();
    anchors.dedup();
    (
        anchors.first().copied().unwrap_or(0),
        anchors.get(1).copied().unwrap_or(0),
    )
}

fn collect_unit_bonds(scaffold: &Molecule, unit: &[u32]) -> Vec<(usize, usize, BondOrder)> {
    scaffold
        .bonds()
        .filter_map(|(_, be)| {
            Some((
                pos_in_unit(unit, be.atom1.0)?,
                pos_in_unit(unit, be.atom2.0)?,
                be.order,
            ))
        })
        .collect()
}

fn pos_in_unit(unit: &[u32], atom: u32) -> Option<usize> {
    unit.iter().position(|&v| v == atom)
}

fn append_repeat_copies(
    b: &mut MoleculeBuilder,
    scaffold: &Molecule,
    sc: &[AtomIdx],
    unit: &[u32],
    unit_internal: &[(usize, usize, BondOrder)],
    copies: usize,
) -> Vec<Vec<AtomIdx>> {
    let mut copy_atoms: Vec<Vec<AtomIdx>> = Vec::with_capacity(copies);
    copy_atoms.push(
        unit.iter()
            .filter_map(|&a| sc.get(usize::try_from(a).unwrap_or(usize::MAX)))
            .copied()
            .collect(),
    );
    for _ in 1..copies {
        let block: Vec<AtomIdx> = unit
            .iter()
            .map(|&a| b.add_atom(scaffold.atom(AtomIdx(a)).clone()))
            .collect();
        for &(i1, i2, order) in unit_internal {
            if let (Some(&p1), Some(&p2)) = (block.get(i1), block.get(i2)) {
                let _ = b.add_bond(p1, p2, order);
            }
        }
        copy_atoms.push(block);
    }
    copy_atoms
}

fn bond_splice_chain(
    b: &mut MoleculeBuilder,
    sc: &[AtomIdx],
    copy_atoms: &[Vec<AtomIdx>],
    anchors: (u32, u32),
    rpos: (usize, usize),
) {
    let (anchor_a, anchor_b) = anchors;
    let (rpos_a, rpos_b) = rpos;
    if let (Some(&ra0), Some(&anchor_a_idx)) = (
        copy_atoms.first().and_then(|c| c.get(rpos_a)),
        sc.get(usize::try_from(anchor_a).unwrap_or(usize::MAX)),
    ) {
        let _ = b.add_bond(anchor_a_idx, ra0, BondOrder::Single);
    }
    for pair in copy_atoms.windows(2) {
        if let [cur, next] = pair
            && let (Some(&rb_k), Some(&ra_k1)) = (cur.get(rpos_b), next.get(rpos_a))
        {
            let _ = b.add_bond(rb_k, ra_k1, BondOrder::Single);
        }
    }
    if let (Some(&rb_last), Some(&anchor_b_idx)) = (
        copy_atoms.last().and_then(|c| c.get(rpos_b)),
        sc.get(usize::try_from(anchor_b).unwrap_or(usize::MAX)),
    ) {
        let _ = b.add_bond(rb_last, anchor_b_idx, BondOrder::Single);
    }
}

/// Splice `n` copies of `repeat_atoms` between its two external anchors.
pub(crate) fn splice_repeat(scaffold: &Molecule, repeat_atoms: &[usize], n: usize) -> Molecule {
    if n <= 1 {
        return scaffold.clone();
    }
    let unit: Vec<u32> = repeat_atoms
        .iter()
        .map(|&a| u32::try_from(a).unwrap_or(u32::MAX))
        .collect();
    let (anchor_a, anchor_b) = collect_anchors(scaffold, &unit);
    let ep_a = endpoint_for(scaffold, &unit, anchor_a);
    let ep_b = endpoint_for(scaffold, &unit, anchor_b);
    let rpos_a = pos_in_unit(&unit, ep_a).unwrap_or(0);
    let rpos_b = pos_in_unit(&unit, ep_b).unwrap_or(0);
    let unit_internal = collect_unit_bonds(scaffold, &unit);

    let mut b = MoleculeBuilder::new();
    let sc: Vec<AtomIdx> = scaffold
        .atoms()
        .map(|(_, atom)| b.add_atom(atom.clone()))
        .collect();
    let copy_atoms = append_repeat_copies(&mut b, scaffold, &sc, &unit, &unit_internal, n);
    add_scaffold_bonds(&mut b, scaffold, &sc, anchor_a, ep_a, anchor_b, ep_b);
    bond_splice_chain(
        &mut b,
        &sc,
        &copy_atoms,
        (anchor_a, anchor_b),
        (rpos_a, rpos_b),
    );
    b.build()
}

/// Copy every scaffold bond except the two anchor bonds replaced by splicing.
pub(crate) fn add_scaffold_bonds(
    b: &mut MoleculeBuilder,
    scaffold: &Molecule,
    sc: &[AtomIdx],
    anchor_a: u32,
    ep_a: u32,
    anchor_b: u32,
    ep_b: u32,
) {
    for (_, be) in scaffold.bonds() {
        let (a, c) = (be.atom1.0, be.atom2.0);
        let is_anchor_bond = (a == anchor_a && c == ep_a)
            || (c == anchor_a && a == ep_a)
            || (a == anchor_b && c == ep_b)
            || (c == anchor_b && a == ep_b);
        if !is_anchor_bond
            && let (Some(&ma), Some(&mc)) =
                (sc.get(atom_usize(be.atom1)), sc.get(atom_usize(be.atom2)))
        {
            let _ = b.add_bond(ma, mc, be.order);
        }
    }
}

/// First atom of `unit` that is bonded to `anchor` in `scaffold`, or `unit[0]`.
pub(crate) fn endpoint_for(scaffold: &Molecule, unit: &[u32], anchor: u32) -> u32 {
    scaffold
        .neighbors(AtomIdx(anchor))
        .find_map(|(nbr, _)| unit.contains(&nbr.0).then_some(nbr.0))
        .or_else(|| unit.first().copied())
        .unwrap_or(0)
}
