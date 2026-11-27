// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

//! `O(V + E)` aromatic ring component expansion for positional sites.

use chematic::core::{BondOrder, Molecule};
use rustc_hash::FxHashSet;

use crate::graph::{atom_idx, atom_usize};

pub(crate) fn aromatic_components(scaffold: &Molecule) -> Vec<Option<usize>> {
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

fn aromatic_signature(scaffold: &Molecule, atom_pos: usize) -> (u8, usize, i8, bool) {
    let idx = atom_idx(atom_pos);
    let atom = scaffold.atom(idx);
    (
        atom.element.atomic_number(),
        scaffold.neighbors(idx).count(),
        atom.charge,
        scaffold.implicit_hydrogen_count(idx) > 0,
    )
}

/// Expand `positions` with equivalent atoms in the same aromatic component (`O(V + E)`).
pub(crate) fn expand_aromatic_equiv(positions: &[usize], scaffold: &Molecule) -> Vec<usize> {
    let comp = aromatic_components(scaffold);
    let mut out = Vec::new();
    let mut targets: FxHashSet<(usize, u8, usize, i8, bool)> = FxHashSet::default();
    for &p in positions {
        match comp.get(p) {
            Some(Some(cid)) => {
                let (el, deg, chg, has_h) = aromatic_signature(scaffold, p);
                targets.insert((*cid, el, deg, chg, has_h));
            }
            Some(None) => out.push(p),
            None => {}
        }
    }
    for (q, c) in comp.iter().enumerate() {
        if let Some(cid) = c {
            let (el, deg, chg, has_h) = aromatic_signature(scaffold, q);
            if targets.contains(&(*cid, el, deg, chg, has_h)) {
                out.push(q);
            }
        }
    }
    out.sort_unstable();
    out.dedup();
    out
}
