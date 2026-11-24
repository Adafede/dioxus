// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

//! Round-trip enumeration and coverage evaluation for confidence scoring.

use chematic::core::{AtomIdx, BondOrder, Molecule, MoleculeBuilder};
use chematic::smiles::canonical_smiles;

use crate::error::CxError;
use crate::fragment::{FloatingDef, add_atoms_and_bonds};
use crate::graph::atom_usize;
use crate::splice::splice_repeat;
use crate::types::{AttachmentTarget, Confidence, ConstructKind, Coverage, RepeatUnit};

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
    if ranges.is_empty() {
        return Ok(dedup_sort(vec![canonical_smiles(&build_one(
            scaffold,
            defs,
            targets,
            &[],
        ))]));
    }
    let total = check_positional_bounds(scaffold, &ranges)?;
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
        out.push(canonical_smiles(&build_one(
            scaffold, defs, targets, &combo,
        )));
        if !next_combo(&mut combo, &ranges) {
            break;
        }
    }
    Ok(dedup_sort(out))
}

fn check_positional_bounds(scaffold: &Molecule, ranges: &[&[usize]]) -> Result<usize, CxError> {
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
    if scaffold.atom_count() > MAX_MOLECULE_ATOMS {
        return Err(CxError::ScaffoldTooLarge {
            atoms: scaffold.atom_count(),
            arrangements: total,
            limit: MAX_MOLECULE_ATOMS,
        });
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

/// Build one concrete molecule for a given variable-position choice.
fn build_one(
    scaffold: &Molecule,
    defs: &[FloatingDef],
    targets: &[AttachmentTarget],
    combo: &[usize],
) -> Molecule {
    let mut b = MoleculeBuilder::new();
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
                if let Some(&pos) = choices.next().and_then(|&c| positions.get(c))
                    && let Some(&p) = sc.get(pos)
                {
                    let _ = b.add_bond(attach, p, order);
                }
            }
            AttachmentTarget::Fixed(other) => {
                if let Some(&(target_attach, _)) = part_attach.get(*other) {
                    let _ = b.add_bond(attach, target_attach, order);
                }
            }
        }
    }
    b.build()
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
    let widest = unit
        .max
        .checked_mul(unit.atoms.len())
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
    let unit_size = unit.atoms.len().max(1);
    let count_cap = MAX_MOLECULE_ATOMS.saturating_div(unit_size);
    let mut out = Vec::with_capacity(total.min(count_cap));
    for n in (unit.min..=unit.max).take(count_cap) {
        out.push(canonical_smiles(&splice_repeat(scaffold, &unit.atoms, n)));
    }
    Ok(dedup_sort(out))
}

/// Evaluate sorted canonical round-trip coverage via `O(log M)` binary search.
#[must_use]
pub(crate) fn roundtrip_coverage(enumerated_sorted: &[String], group: &[Molecule]) -> Coverage {
    let covered = group
        .iter()
        .filter(|mol| {
            enumerated_sorted
                .binary_search(&canonical_smiles(mol))
                .is_ok()
        })
        .count();
    Coverage {
        covered,
        total: group.len(),
    }
}

/// Evaluate coverage and demote `clean_kind` to `ConstructKind::BestEffort` if incomplete.
#[must_use]
pub(crate) fn evaluate_construct(
    enumerated_sorted: &[String],
    mols: &[Molecule],
    clean_kind: ConstructKind,
) -> (ConstructKind, Confidence) {
    let coverage = roundtrip_coverage(enumerated_sorted, mols);
    let confidence = Confidence::from_coverage(coverage);
    let kind = if confidence.clean {
        clean_kind
    } else {
        ConstructKind::BestEffort
    };
    (kind, confidence)
}

/// Sort and deduplicate a list of SMILES strings.
#[must_use]
pub(crate) fn dedup_sort(mut v: Vec<String>) -> Vec<String> {
    v.sort_unstable();
    v.dedup();
    v
}
