// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Round-trip enumeration and coverage used for confidence scoring.
//!
//! `enumerate` expands a positional CX-SMILES into every distinct molecule
//! (one per variable-position combination) and canonicalises them; `enumerate_repeating`
//! expands the repeating case by splicing `n` copies of the unit. Both feed
//! [`roundtrip_coverage`], which counts how many canonical inputs survive the
//! round-trip.

use chematic::core::{AtomIdx, BondOrder, Molecule, MoleculeBuilder};
use chematic::smiles::canonical_smiles;

use super::positional::{FloatingDef, Target};
use super::repeating::splice_repeat;
use super::types::{Coverage, RepeatUnit};

/// Enumerate every distinct molecule implied by a positional CX-SMILES.
pub fn enumerate(scaffold: &Molecule, defs: &[FloatingDef], targets: &[Target]) -> Vec<String> {
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
    let ranges: Vec<&Vec<usize>> = var
        .iter()
        .map(|&i| match &targets[i] {
            Target::Variable(p) => p,
            Target::Fixed(_) => unreachable!(),
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

fn next_combo(combo: &mut [usize], ranges: &[&Vec<usize>]) -> bool {
    for i in (0..combo.len()).rev() {
        combo[i] += 1;
        if combo[i] < ranges[i].len() {
            return true;
        }
        combo[i] = 0;
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
        let _ = b.add_bond(sc[be.atom1.0 as usize], sc[be.atom2.0 as usize], be.order);
    }
    let mut part_attach: Vec<AtomIdx> = Vec::with_capacity(defs.len());
    for def in defs {
        let mut pmap: Vec<AtomIdx> = Vec::with_capacity(def.atoms.len());
        for atom in &def.atoms {
            pmap.push(b.add_atom(atom.clone()));
        }
        for (a1, a2, order) in &def.bonds {
            let _ = b.add_bond(pmap[*a1], pmap[*a2], *order);
        }
        part_attach.push(pmap[def.attachment]);
    }
    let mut var = 0usize;
    for (pi, _def) in defs.iter().enumerate() {
        match &targets[pi] {
            Target::Variable(positions) => {
                let pos = positions[combo[var]];
                var += 1;
                let _ = b.add_bond(part_attach[pi], sc[pos], BondOrder::Single);
            }
            Target::Fixed(other) => {
                let _ = b.add_bond(part_attach[pi], part_attach[*other], BondOrder::Single);
            }
        }
    }
    b.build()
}

/// Enumerate the repeating case: scaffold + (count-1) extra copies of the unit.
pub fn enumerate_repeating(scaffold: &Molecule, unit: &RepeatUnit) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for n in unit.min..=unit.max {
        out.push(canonical_smiles(&splice_repeat(scaffold, &unit.atoms, n)));
    }
    dedup_sort(out)
}

/// Round-trip report: how many original inputs re-appear after expanding CX-SMILES.
pub fn roundtrip_coverage(enumerated: &[String], group: &[Molecule]) -> (usize, Coverage) {
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

pub fn dedup_sort(mut v: Vec<String>) -> Vec<String> {
    v.sort();
    v.dedup();
    v
}
