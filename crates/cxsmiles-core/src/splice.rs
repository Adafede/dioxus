// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

//! Repeating-unit splicing for `Sg:n:` expansion.

use chematic::core::{AtomIdx, BondOrder, Molecule, MoleculeBuilder};

use crate::graph::{atom_usize, u32_to_usize, usize_to_u32};

fn build_unit_pos(atom_count: usize, unit: &[u32]) -> Vec<Option<usize>> {
    let mut pos = vec![None; atom_count];
    for (idx, &atom) in unit.iter().enumerate() {
        if let Some(slot) = pos.get_mut(u32_to_usize(atom)) {
            *slot = Some(idx);
        }
    }
    pos
}

fn pos_in_table(unit_pos: &[Option<usize>], atom: u32) -> Option<usize> {
    unit_pos.get(u32_to_usize(atom)).copied().flatten()
}

fn collect_anchors(
    scaffold: &Molecule,
    unit: &[u32],
    unit_pos: &[Option<usize>],
) -> Option<(u32, u32)> {
    let mut anchors: Vec<u32> = unit
        .iter()
        .flat_map(|&a| {
            scaffold
                .neighbors(AtomIdx(a))
                .filter(|(nb, _)| pos_in_table(unit_pos, nb.0).is_none())
                .map(|(nb, _)| nb.0)
        })
        .collect();
    anchors.sort_unstable();
    anchors.dedup();
    let (&a, &b) = (anchors.first()?, anchors.get(1)?);
    if anchors.len() != 2 {
        return None;
    }
    Some((a, b))
}

fn collect_unit_bonds(
    scaffold: &Molecule,
    unit_pos: &[Option<usize>],
) -> Vec<(usize, usize, BondOrder)> {
    scaffold
        .bonds()
        .filter_map(|(_, be)| {
            Some((
                pos_in_table(unit_pos, be.atom1.0)?,
                pos_in_table(unit_pos, be.atom2.0)?,
                be.order,
            ))
        })
        .collect()
}

fn copy_scaffold_atoms(scaffold: &Molecule, b: &mut MoleculeBuilder) -> Vec<AtomIdx> {
    scaffold
        .atoms()
        .map(|(_, atom)| b.add_atom(atom.clone()))
        .collect()
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
            .filter_map(|&a| sc.get(u32_to_usize(a)))
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
        sc.get(u32_to_usize(anchor_a)),
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
        sc.get(u32_to_usize(anchor_b)),
    ) {
        let _ = b.add_bond(rb_last, anchor_b_idx, BondOrder::Single);
    }
}

/// Splice `n` copies of `repeat_atoms` between its two external anchors, or chain whole-molecule repeats.
pub(crate) fn splice_repeat(scaffold: &Molecule, repeat_atoms: &[usize], n: usize) -> Molecule {
    if n <= 1 || repeat_atoms.is_empty() {
        return scaffold.clone();
    }
    let unit: Vec<u32> = repeat_atoms.iter().copied().map(usize_to_u32).collect();
    let unit_pos = build_unit_pos(scaffold.atom_count(), &unit);
    let unit_internal = collect_unit_bonds(scaffold, &unit_pos);

    if unit.len() == scaffold.atom_count() {
        let mut b = MoleculeBuilder::new();
        let sc = copy_scaffold_atoms(scaffold, &mut b);
        let copy_atoms = append_repeat_copies(&mut b, scaffold, &sc, &unit, &unit_internal, n);
        add_scaffold_bonds(&mut b, scaffold, &sc, &[]);
        let last = unit.len() - 1;
        for pair in copy_atoms.windows(2) {
            if let [cur, next] = pair
                && let (Some(&rb_k), Some(&ra_k1)) = (cur.get(last), next.first())
            {
                let _ = b.add_bond(rb_k, ra_k1, BondOrder::Single);
            }
        }
        return b.build();
    }

    let Some((anchor_a, anchor_b)) = collect_anchors(scaffold, &unit, &unit_pos) else {
        return scaffold.clone();
    };
    let ep_a = endpoint_for_table(scaffold, &unit, &unit_pos, anchor_a);
    let ep_b = endpoint_for_table(scaffold, &unit, &unit_pos, anchor_b);
    let rpos_a = pos_in_table(&unit_pos, ep_a).unwrap_or(0);
    let rpos_b = pos_in_table(&unit_pos, ep_b).unwrap_or(0);

    let mut b = MoleculeBuilder::new();
    let sc = copy_scaffold_atoms(scaffold, &mut b);
    let copy_atoms = append_repeat_copies(&mut b, scaffold, &sc, &unit, &unit_internal, n);
    add_scaffold_bonds(&mut b, scaffold, &sc, &[(anchor_a, ep_a), (anchor_b, ep_b)]);
    bond_splice_chain(
        &mut b,
        &sc,
        &copy_atoms,
        (anchor_a, anchor_b),
        (rpos_a, rpos_b),
    );
    b.build()
}

/// Copy every scaffold bond except the listed undirected `(u, v)` pairs.
pub(crate) fn add_scaffold_bonds(
    b: &mut MoleculeBuilder,
    scaffold: &Molecule,
    sc: &[AtomIdx],
    skip_bonds: &[(u32, u32)],
) {
    for (_, be) in scaffold.bonds() {
        let (a, c) = (be.atom1.0, be.atom2.0);
        let skipped = skip_bonds
            .iter()
            .any(|&(u, v)| (a == u && c == v) || (c == u && a == v));
        if !skipped
            && let (Some(&ma), Some(&mc)) =
                (sc.get(atom_usize(be.atom1)), sc.get(atom_usize(be.atom2)))
        {
            let _ = b.add_bond(ma, mc, be.order);
        }
    }
}

fn endpoint_for_table(
    scaffold: &Molecule,
    unit: &[u32],
    unit_pos: &[Option<usize>],
    anchor: u32,
) -> u32 {
    scaffold
        .neighbors(AtomIdx(anchor))
        .find_map(|(nbr, _)| pos_in_table(unit_pos, nbr.0).map(|_| nbr.0))
        .or_else(|| unit.first().copied())
        .unwrap_or(0)
}

/// First atom of `unit` that is bonded to `anchor` in `scaffold`, or `unit[0]`.
#[cfg(test)]
pub(crate) fn endpoint_for(scaffold: &Molecule, unit: &[u32], anchor: u32) -> u32 {
    let unit_pos = build_unit_pos(scaffold.atom_count(), unit);
    endpoint_for_table(scaffold, unit, &unit_pos, anchor)
}
