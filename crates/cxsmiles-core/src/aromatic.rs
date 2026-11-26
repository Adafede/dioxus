// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

//! `O(V + E)` aromatic ring component expansion for positional sites.

use chematic::core::{BondOrder, Molecule};

use crate::graph::{atom_idx, atom_usize};

fn aromatic_components(scaffold: &Molecule) -> Vec<Option<usize>> {
    let n = scaffold.atom_count();
    let mut comp = vec![None; n];
    let mut stack = Vec::new();
    let mut next_id = 0usize;
    for start in 0..n {
        if comp.get(start).copied().flatten().is_some() {
            continue;
        }
        let has_arom = scaffold
            .neighbors(atom_idx(start))
            .any(|(_, bi)| scaffold.bond(bi).order == BondOrder::Aromatic);
        if !has_arom {
            continue;
        }
        if let Some(slot) = comp.get_mut(start) {
            *slot = Some(next_id);
        }
        stack.push(start);
        while let Some(cur) = stack.pop() {
            for (nbr, bi) in scaffold.neighbors(atom_idx(cur)) {
                let nu = atom_usize(nbr);
                if scaffold.bond(bi).order == BondOrder::Aromatic
                    && let Some(slot) = comp.get_mut(nu)
                    && slot.is_none()
                {
                    *slot = Some(next_id);
                    stack.push(nu);
                }
            }
        }
        next_id += 1;
    }
    comp
}

/// Expand `positions` with equivalent atoms in the same aromatic component (`O(V + E)`).
pub(crate) fn expand_aromatic_equiv(positions: &[usize], scaffold: &Molecule) -> Vec<usize> {
    let n = scaffold.atom_count();
    let comp = aromatic_components(scaffold);
    let sigs: Vec<(u8, usize, i8, bool)> = (0..n)
        .map(|i| {
            let idx = atom_idx(i);
            let atom = scaffold.atom(idx);
            (
                atom.element.atomic_number(),
                scaffold.neighbors(idx).count(),
                atom.charge,
                scaffold.implicit_hydrogen_count(idx) > 0,
            )
        })
        .collect();
    let mut out = Vec::new();
    let mut targets: Vec<(usize, u8, usize, i8, bool)> = Vec::new();
    for &p in positions {
        match (comp.get(p), sigs.get(p)) {
            (Some(Some(cid)), Some(&(el, deg, chg, has_h))) => {
                if !targets.contains(&(*cid, el, deg, chg, has_h)) {
                    targets.push((*cid, el, deg, chg, has_h));
                }
            }
            (Some(None), Some(_)) => out.push(p),
            _ => {}
        }
    }
    for (q, (c, &(el, deg, chg, has_h))) in comp.iter().zip(&sigs).enumerate() {
        if let Some(cid) = c
            && targets.contains(&(*cid, el, deg, chg, has_h))
        {
            out.push(q);
        }
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// Whether `start` and `target` share an aromatic-bond component in `scaffold`.
#[cfg(test)]
pub(crate) fn same_aromatic_ring(scaffold: &Molecule, start: u32, target: u32) -> bool {
    let comp = aromatic_components(scaffold);
    let su = usize::try_from(start).unwrap_or(usize::MAX);
    let tu = usize::try_from(target).unwrap_or(usize::MAX);
    matches!((comp.get(su), comp.get(tu)), (Some(Some(a)), Some(Some(b))) if a == b)
}
