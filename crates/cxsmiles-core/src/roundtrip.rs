// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

//! Round-trip enumeration and coverage evaluation for confidence scoring.

use chematic::core::{AtomIdx, BondOrder, Molecule, MoleculeBuilder};
use chematic::smiles::canonical_smiles;

use crate::error::CxError;
use crate::fragment::{FloatingDef, add_atoms_and_bonds};
use crate::graph::atom_usize;
use crate::splice::splice_repeat;
use crate::types::{AttachmentTarget, ConstructKind, Coverage, RepeatUnit};

pub(crate) const MAX_ARRANGEMENTS: usize = 10_000;
pub(crate) const MAX_MOLECULE_ATOMS: usize = 512;

/// Enumerate every distinct molecule implied by a positional CX-SMILES.
pub(crate) fn enumerate(
    scaffold: &Molecule,
    defs: &[FloatingDef],
    targets: &[AttachmentTarget],
) -> Result<Vec<String>, CxError> {
    let ranges: Vec<&[usize]> = targets
        .iter()
        .filter_map(|t| match t {
            AttachmentTarget::Variable(pos) => Some(pos.as_slice()),
            AttachmentTarget::Fixed(_) => None,
        })
        .collect();
    let mut rem_h = Vec::with_capacity(scaffold.atom_count());
    let base_rem_h: Vec<u8> = (0..scaffold.atom_count())
        .map(|i| scaffold.implicit_hydrogen_count(crate::graph::atom_idx(i)))
        .collect();
    if ranges.is_empty() {
        return Ok(dedup_sort(
            build_one(scaffold, defs, targets, &[], &base_rem_h, &mut rem_h)
                .into_iter()
                .map(|m| canonical_smiles(&m))
                .collect(),
        ));
    }
    let total = check_positional_bounds(scaffold, defs, &ranges)?;
    let mut combo = vec![0usize; ranges.len()];
    let mut out = Vec::with_capacity(total.min(MAX_ARRANGEMENTS));
    for _ in 0..total.min(MAX_ARRANGEMENTS) {
        for (&digit, range) in combo.iter().zip(&ranges) {
            if digit >= range.len() {
                return Err(CxError::InvalidPosition {
                    pos: digit,
                    group_len: range.len(),
                });
            }
        }
        if let Some(mol) = build_one(scaffold, defs, targets, &combo, &base_rem_h, &mut rem_h) {
            out.push(canonical_smiles(&mol));
        }
        if !next_combo(&mut combo, &ranges) {
            break;
        }
    }
    Ok(dedup_sort(out))
}

fn check_positional_bounds(
    scaffold: &Molecule,
    defs: &[FloatingDef],
    ranges: &[&[usize]],
) -> Result<usize, CxError> {
    let total = ranges
        .iter()
        .try_fold(1usize, |acc, range| acc.checked_mul(range.len()))
        .filter(|&total| total > 0)
        .ok_or(CxError::NoVariablePositions)?;
    if total > MAX_ARRANGEMENTS {
        return Err(CxError::TooManyArrangements {
            total,
            group_count: ranges.len(),
            group_sizes: ranges.iter().map(|r| r.len()).collect(),
            limit: MAX_ARRANGEMENTS,
        });
    }
    let extra_atoms: usize = defs.iter().map(|d| d.atoms.len()).sum();
    let expanded_atoms = scaffold.atom_count().saturating_add(extra_atoms);
    if expanded_atoms > MAX_MOLECULE_ATOMS {
        return Err(CxError::ScaffoldTooLarge {
            atoms: expanded_atoms,
            arrangements: total,
            limit: MAX_MOLECULE_ATOMS,
        });
    }
    for range in ranges {
        for &pos in *range {
            if pos >= scaffold.atom_count() {
                return Err(CxError::InvalidPosition {
                    pos,
                    group_len: scaffold.atom_count(),
                });
            }
        }
    }
    Ok(total)
}

/// Advance `combo` to the next mixed-radix choice, returning `false` on wrap.
pub(crate) fn next_combo(combo: &mut [usize], ranges: &[&[usize]]) -> bool {
    for (value, range) in combo.iter_mut().zip(ranges).rev() {
        *value += 1;
        if *value < range.len() {
            return true;
        }
        *value = 0;
    }
    false
}

const fn bond_valence_cost(order: BondOrder) -> u8 {
    match order {
        BondOrder::Double => 2,
        BondOrder::Triple => 3,
        _ => 1,
    }
}

/// Build one concrete molecule for a given variable-position choice.
fn build_one(
    scaffold: &Molecule,
    defs: &[FloatingDef],
    targets: &[AttachmentTarget],
    combo: &[usize],
    base_rem_h: &[u8],
    rem_h: &mut Vec<u8>,
) -> Option<Molecule> {
    let mut b = MoleculeBuilder::new();
    rem_h.clear();
    rem_h.extend_from_slice(base_rem_h);
    let sc: Vec<AtomIdx> = scaffold
        .atoms()
        .map(|(_, atom)| b.add_atom(atom.clone()))
        .collect();
    for (_, be) in scaffold.bonds() {
        if let (Some(&ma), Some(&mc)) = (sc.get(atom_usize(be.atom1)), sc.get(atom_usize(be.atom2)))
        {
            let _ = b.add_bond(ma, mc, be.order);
        }
    }
    let mut part_attach: Vec<(AtomIdx, BondOrder)> = Vec::with_capacity(defs.len());
    for def in defs {
        let pmap = add_atoms_and_bonds(&mut b, &def.atoms, &def.bonds);
        if let Some(&attachment) = pmap.get(def.attachment) {
            part_attach.push((attachment, def.attach_order));
        }
    }
    let mut choices = combo.iter();
    for (&(attach, order), target) in part_attach.iter().zip(targets) {
        match target {
            AttachmentTarget::Variable(positions) => {
                let pos = *choices.next().and_then(|&c| positions.get(c))?;
                let slot = rem_h.get_mut(pos)?;
                let cost = bond_valence_cost(order);
                if *slot < cost {
                    return None;
                }
                *slot -= cost;
                let p = *sc.get(pos)?;
                let _ = b.add_bond(attach, p, order);
            }
            AttachmentTarget::Fixed(other) => {
                if let Some(&(target_attach, _)) = part_attach.get(*other) {
                    let _ = b.add_bond(attach, target_attach, order);
                }
            }
        }
    }
    Some(b.build())
}

/// Enumerate all repeat counts from `unit.min` through `unit.max`.
pub(crate) fn enumerate_repeating(
    scaffold: &Molecule,
    unit: &RepeatUnit,
) -> Result<Vec<String>, CxError> {
    let total = unit.max.saturating_sub(unit.min).saturating_add(1);
    if total > MAX_ARRANGEMENTS {
        return Err(CxError::RepeatRangeTooLarge {
            total,
            min: unit.min,
            max: unit.max,
            limit: MAX_ARRANGEMENTS,
        });
    }
    let nonrepeat = scaffold.atom_count().saturating_sub(unit.atoms.len());
    let widest = unit
        .max
        .checked_mul(unit.atoms.len())
        .and_then(|expanded| nonrepeat.checked_add(expanded))
        .ok_or(CxError::ExpansionOverflow {
            min: unit.min,
            max: unit.max,
            unit_len: unit.atoms.len(),
        })?;
    if widest > MAX_MOLECULE_ATOMS {
        return Err(CxError::ExpansionTooLarge {
            widest,
            copies: unit.max,
            unit_len: unit.atoms.len(),
            limit: MAX_MOLECULE_ATOMS,
        });
    }
    let mut out = Vec::with_capacity(total);
    for n in unit.min..=unit.max {
        out.push(canonical_smiles(&splice_repeat(scaffold, &unit.atoms, n)));
    }
    Ok(dedup_sort(out))
}

/// Evaluate sorted canonical round-trip coverage from precomputed canonical SMILES.
#[must_use]
pub(crate) fn evaluate_construct_canons(
    enumerated_sorted: &[String],
    group_canons: &[String],
    clean_kind: ConstructKind,
) -> (ConstructKind, Coverage) {
    let covered = group_canons
        .iter()
        .filter(|smi| enumerated_sorted.binary_search(smi).is_ok())
        .count();
    let coverage = Coverage {
        covered,
        total: group_canons.len(),
    };
    let kind = if coverage.is_clean() {
        clean_kind
    } else {
        ConstructKind::BestEffort
    };
    (kind, coverage)
}

/// Evaluate sorted canonical round-trip coverage via `O(log M)` binary search.
#[cfg(test)]
#[must_use]
pub(crate) fn roundtrip_coverage(enumerated_sorted: &[String], group: &[Molecule]) -> Coverage {
    let group_canons: Vec<String> = group.iter().map(canonical_smiles).collect();
    evaluate_construct_canons(enumerated_sorted, &group_canons, ConstructKind::BestEffort).1
}

/// Sort and deduplicate a list of SMILES strings.
#[must_use]
pub(crate) fn dedup_sort(mut v: Vec<String>) -> Vec<String> {
    v.sort_unstable();
    v.dedup();
    v
}
