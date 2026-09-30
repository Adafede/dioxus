// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

//! Round-trip enumeration and coverage used for confidence scoring.
//!
//! `enumerate` expands a positional CX-SMILES into every distinct molecule
//! (one per variable-position combination) and canonicalises them; `enumerate_repeating`
//! expands the repeating case by splicing `n` copies of the unit. Both feed
//! `roundtrip_coverage`, which counts how many canonical inputs survive the
//! round-trip.

use chematic::core::{AtomIdx, BondOrder, Molecule, MoleculeBuilder};
use chematic::smiles::canonical_smiles;

use super::positional::{FloatingDef, Target};
use super::repeating::splice_repeat;
use super::types::{Coverage, RepeatUnit};

/// Enumerate every distinct molecule implied by a positional CX-SMILES.
#[must_use]
pub(crate) fn enumerate(
    scaffold: &Molecule,
    defs: &[FloatingDef],
    targets: &[Target],
) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let var: Vec<usize> = targets
        .iter()
        .enumerate()
        .filter(|(_, t)| matches!(t, Target::Variable(_)))
        .map(|(i, _)| i)
        .collect();
    if var.is_empty() {
        out.push(canonical_smiles(&build_one(scaffold, defs, targets, &[])));
        return dedup_sort(out);
    }
    let ranges: Vec<&[usize]> = var
        .iter()
        .filter_map(|&i| match targets.get(i) {
            Some(Target::Variable(p)) => Some(p.as_slice()),
            _ => None,
        })
        .collect();
    let mut combo: Vec<usize> = vec![0; var.len()];
    loop {
        out.push(canonical_smiles(&build_one(
            scaffold, defs, targets, &combo,
        )));
        if !next_combo(&mut combo, &ranges) {
            break;
        }
    }
    dedup_sort(out)
}

/// Advance `combo` to the next mixed-radix choice, or report that there is none.
///
/// `combo` and `ranges` are built in lockstep one above, so they are the same
/// length and the two loops walk the same positions; zipping them is what keeps
/// that true instead of leaving it to a comment.
fn next_combo(combo: &mut [usize], ranges: &[&[usize]]) -> bool {
    for (value, range) in combo.iter_mut().zip(ranges).rev() {
        *value += 1;
        if *value < range.len() {
            return true;
        }
        *value = 0;
    }
    false
}

/// Build one concrete molecule for a given variable-position choice.
fn build_one(
    scaffold: &Molecule,
    defs: &[FloatingDef],
    targets: &[Target],
    combo: &[usize],
) -> Molecule {
    let mut b = MoleculeBuilder::new();
    let mut sc: Vec<AtomIdx> = Vec::with_capacity(scaffold.atom_count());
    for (_, atom) in scaffold.atoms() {
        sc.push(b.add_atom(atom.clone()));
    }
    for (_, be) in scaffold.bonds() {
        if let (Some(ma), Some(mc)) = (
            sc.get(be.atom1.0 as usize).copied(),
            sc.get(be.atom2.0 as usize).copied(),
        ) {
            let _ = b.add_bond(ma, mc, be.order);
        }
    }
    let mut part_attach: Vec<AtomIdx> = Vec::with_capacity(defs.len());
    for def in defs {
        let mut pmap: Vec<AtomIdx> = Vec::with_capacity(def.atoms.len());
        for atom in &def.atoms {
            pmap.push(b.add_atom(atom.clone()));
        }
        for (a1, a2, order) in &def.bonds {
            if let (Some(m1), Some(m2)) = (pmap.get(*a1), pmap.get(*a2)) {
                let _ = b.add_bond(*m1, *m2, *order);
            }
        }
        if let Some(&attachment) = pmap.get(def.attachment) {
            part_attach.push(attachment);
        }
    }
    let mut var = 0usize;
    for (attach, target) in part_attach.iter().zip(targets) {
        match target {
            Target::Variable(positions) => {
                // `combo` has one entry per `Variable` target, in order, which
                // is what `var` counts here.
                let chosen = combo.get(var).copied();
                var += 1;
                if let Some(pos) = chosen.and_then(|c| positions.get(c).copied())
                    && let Some(&p) = sc.get(pos)
                {
                    let _ = b.add_bond(*attach, p, BondOrder::Single);
                }
            }
            Target::Fixed(other) => {
                if let Some(target_attach) = part_attach.get(*other) {
                    let _ = b.add_bond(*attach, *target_attach, BondOrder::Single);
                }
            }
        }
    }
    b.build()
}

/// Enumerate the repeating case: scaffold + (count-1) extra copies of the unit.
#[must_use]
pub(crate) fn enumerate_repeating(scaffold: &Molecule, unit: &RepeatUnit) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for n in unit.min..=unit.max {
        out.push(canonical_smiles(&splice_repeat(scaffold, &unit.atoms, n)));
    }
    dedup_sort(out)
}

/// Round-trip report: how many original inputs re-appear after expanding CX-SMILES.
#[must_use]
pub(crate) fn roundtrip_coverage(enumerated: &[String], group: &[Molecule]) -> (usize, Coverage) {
    let canon: Vec<String> = group.iter().map(canonical_smiles).collect();
    let mut covered = 0;
    for c in &canon {
        if enumerated.iter().any(|e| e == c) {
            covered += 1;
        }
    }
    (
        covered,
        Coverage {
            covered,
            total: group.len(),
        },
    )
}

#[must_use]
pub(crate) fn dedup_sort(mut v: Vec<String>) -> Vec<String> {
    v.sort();
    v.dedup();
    v
}
