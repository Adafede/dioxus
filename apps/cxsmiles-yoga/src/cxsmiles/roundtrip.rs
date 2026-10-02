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
use super::types::{Coverage, CxError, RepeatUnit};

/// The most arrangements `enumerate` will expand before refusing.
///
/// The count is the product of the per-group position lists, so it is the size
/// of the expansion and nothing else. A group with two positions and another
/// with three is six; a group with twenty and another with twenty is 400
/// billion, and expanding that in a browser tab does not finish.
///
/// Refusing is better than trying, and much better than the alternative: the
/// loop below is the only exit of which is `next_combo` saying it is finished,
/// so an expansion this size is not slow, it is unbounded. Real inputs land in
/// single digits or low tens — the four-molecule chlorine series in the tests is
/// four — and 10,000 is far above anything a user is waiting for. That ratio is
/// what the `an_ordinary_*` tests below check, behaviourally: a ceiling that was
/// small enough to refuse them would make them fail, which is the point of
/// writing them against real fixture sizes rather than asserting the constant.
const MAX_ARRANGEMENTS: usize = 10_000;

/// The largest single molecule either expansion will build, in atoms.
///
/// [`MAX_ARRANGEMENTS`] bounds how many molecules; this bounds how big any one
/// of them may be, and the two together bound the work without needing a
/// ceiling on the product.
///
/// The product is not the quantity to cap, and getting that wrong is what the
/// 17 GB was. [`enumerate`] builds one scaffold-sized molecule per arrangement, so
/// the count is a good proxy there. `enumerate_repeating` builds an `n`-unit
/// molecule at iteration `n` and holds every result at once, so its memory is the
/// product — and a count of exactly `MAX_ARRANGEMENTS` is *not* refused by a `>`
/// check, which is exactly what a 20,000-atom input over a one-atom unit
/// produces: a range sitting on the ceiling, expanding to nothing the count could
/// see.
///
/// Canonicalisation is also superlinear in molecule size, which is the reason
/// this is a separate ceiling at all rather than a larger count. One 20,000-atom
/// chain took over a minute to canonicalise, where 20,000 one-atom molecules take
/// milliseconds — so a single huge molecule is worth refusing well before the
/// aggregate gets anywhere near 10^7.
///
/// 4,096 atoms is far past any molecule worth encoding here, so this does not
/// fire on input that means anything.
const MAX_MOLECULE_ATOMS: usize = 4_096;

/// Enumerate every distinct molecule implied by a positional CX-SMILES.
///
/// # Errors
///
/// Returns a [`CxError`] when the expansion is larger than [`MAX_ARRANGEMENTS`],
/// or when one of the positions is out of range for the scaffold it names. The
/// latter means the target list and the scaffold disagree, which is a defect in
/// what was built rather than in the input.
pub(crate) fn enumerate(
    scaffold: &Molecule,
    defs: &[FloatingDef],
    targets: &[Target],
) -> Result<Vec<String>, CxError> {
    let mut out: Vec<String> = Vec::new();
    let var: Vec<usize> = targets
        .iter()
        .enumerate()
        .filter(|(_, t)| matches!(t, Target::Variable(_)))
        .map(|(i, _)| i)
        .collect();
    if var.is_empty() {
        out.push(canonical_smiles(&build_one(scaffold, defs, targets, &[])));
        return Ok(dedup_sort(out));
    }
    let ranges: Vec<&[usize]> = var
        .iter()
        .filter_map(|&i| match targets.get(i) {
            Some(Target::Variable(p)) => Some(p.as_slice()),
            _ => None,
        })
        .collect();

    // The arrangement count, computed before anything is built rather than
    // discovered by running out of memory. `checked_mul` so a pair of absurd
    // group sizes is an error rather than a wrap to a small number that the
    // loop would then exceed.
    let total = ranges
        .iter()
        .try_fold(1usize, |acc, range| acc.checked_mul(range.len()))
        .filter(|total| *total > 0)
        .ok_or_else(|| {
            CxError("a variable group has no positions, so there is nothing to expand".into())
        })?;
    if total > MAX_ARRANGEMENTS {
        return Err(CxError(format!(
            "{total} arrangements to expand, from {} groups of {:?} positions;              the limit is {MAX_ARRANGEMENTS}",
            ranges.len(),
            ranges.iter().map(|r| r.len()).collect::<Vec<_>>()
        )));
    }
    // The other half of the bound. `MAX_ARRANGEMENTS` says how many molecules and
    // this says how big; together they bound the work, and a single molecule is
    // worth capping on its own because canonicalisation is superlinear in size.
    if scaffold.atom_count() > MAX_MOLECULE_ATOMS {
        return Err(CxError(format!(
            "a scaffold of {} atoms cannot be expanded {} times; the limit is \
             {MAX_MOLECULE_ATOMS} atoms",
            scaffold.atom_count(),
            total
        )));
    }

    let mut combo: Vec<usize> = vec![0; var.len()];
    for _ in 0..total {
        for (digit, range) in combo.iter().zip(&ranges) {
            if *digit >= range.len() {
                return Err(CxError(format!(
                    "position {digit} is outside the {} positions of its group",
                    range.len()
                )));
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
///
/// # Errors
///
/// Returns a [`CxError`] when the range of repeat counts is larger than
/// [`MAX_ARRANGEMENTS`].
pub(crate) fn enumerate_repeating(
    scaffold: &Molecule,
    unit: &RepeatUnit,
) -> Result<Vec<String>, CxError> {
    // The range is the number of repeat counts between the two extreme inputs,
    // and every count expands to a molecule that many units long, so the range
    // length is the loop and the largest count is the molecule. Both come from
    // the atom-count difference divided by the unit size, which is a handful
    // for real inputs and unbounded if that arithmetic is wrong.
    //
    // The unit size itself is already bounded upstream: `locate_repeat_in_scaffold`
    // refuses a unit larger than the scaffold and gives up on a search that
    // would take more than `MAX_FRAGMENTS`, so by the time this is called the
    // unit is small enough that `MAX_ARRANGEMENTS` expansions are slow rather
    // than fatal.
    let total = unit.max.saturating_sub(unit.min).saturating_add(1);
    if total > MAX_ARRANGEMENTS {
        return Err(CxError(format!(
            "{total} repeat counts to expand ({} to {}); the limit is \
             {MAX_ARRANGEMENTS}",
            unit.min, unit.max
        )));
    }

    // The other half of the bound, and the half the count could not see. The
    // count ceiling above is `>`, not `>=`, so a range sitting exactly on
    // `MAX_ARRANGEMENTS` walks straight through it — and `count_max` reaches
    // those values honestly, being an atom-count difference over the unit size,
    // so a long input raises `max` without raising the count. Iteration `n` then
    // builds an `n`-unit molecule, growing, holding every result. 17 GB, twice,
    // at only two jobs.
    //
    // Capping the widest molecule is what stops it, and `checked_mul` because the
    // product can wrap to a small number and defeat the cap that way.
    let widest = unit.max.checked_mul(unit.atoms.len()).ok_or_else(|| {
        CxError(format!(
            "a repeat range of {} to {} over a {}-atom unit overflows; the inputs \
             are not a molecule",
            unit.min,
            unit.max,
            unit.atoms.len()
        ))
    })?;
    if widest > MAX_MOLECULE_ATOMS {
        return Err(CxError(format!(
            "{widest} atoms in the largest expansion ({} copies of a {}-atom unit); \
             the limit is {MAX_MOLECULE_ATOMS} atoms",
            unit.max,
            unit.atoms.len()
        )));
    }

    let mut out: Vec<String> = Vec::with_capacity(total);
    for n in unit.min..=unit.max {
        out.push(canonical_smiles(&splice_repeat(scaffold, &unit.atoms, n)));
    }
    Ok(dedup_sort(out))
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

#[cfg(test)]
// `expect` on a fixture that stopped parsing is a broken test, not a defect in
// the code under test; the out-of-range reads below are the assertions failing.
#[allow(clippy::indexing_slicing, clippy::expect_used)]
mod tests;

/// The refusal tests for the expansion ceilings.
///
/// A ceiling is only worth having if it is reachable without producing the
/// blow-up it exists to prevent, and that is not automatic: the original way
/// this failed was an unbounded loop appending to a `Vec`, so proving the guard
/// works by reaching the old failure would mean reaching the out-of-memory.
#[cfg(test)]
// `expect` is here because a fixture that stopped parsing is a broken test, not
// a defect in the code under test — the distinction the denial draws elsewhere.
#[allow(clippy::expect_used)]
mod limits;
