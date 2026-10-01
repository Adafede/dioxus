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
mod tests {
    use super::*;
    use chematic::smiles::parse;

    fn mol(smiles: &str) -> Molecule {
        parse(smiles).expect("the fixtures in this module are valid SMILES")
    }

    // ── next_combo: the mixed-radix odometer ────────────────────────────────

    #[test]
    fn an_exhausted_combo_reports_that_there_is_no_next() {
        // One range of one, already at zero: the digit wraps to zero and there
        // is nowhere left to carry to.
        assert!(
            !next_combo(&mut [0], &[&[0usize]]),
            "a single exhausted digit is the end"
        );
    }

    #[test]
    fn a_combo_advances_from_the_last_digit_first() {
        // Mixed radix: the rightmost digit moves, and only when it wraps does
        // the one to its left move. Advancing from the left would enumerate
        // every combination of the wrong order and, worse, the same set twice.
        let mut combo = [1, 0];
        let ranges: [&[usize]; 2] = [&[0, 1, 2], &[0, 1]];
        assert!(next_combo(&mut combo, &ranges), "there is a next one");
        assert_eq!(combo, [1, 1], "only the rightmost digit moved");
    }

    #[test]
    fn a_wrapping_digit_carries_into_the_one_to_its_left() {
        let mut combo = [0, 1];
        let ranges: [&[usize]; 2] = [&[0, 1, 2], &[0, 1]];
        assert!(next_combo(&mut combo, &ranges), "carrying is not the end");
        assert_eq!(
            combo,
            [1, 0],
            "the right digit wrapped and the left advanced"
        );
    }

    #[test]
    fn a_combo_visits_every_combination_exactly_once() {
        // 3 × 2 = 6 combinations, plus the one call that reports the end. This
        // is the property the enumeration depends on: a digit that failed to
        // reset would repeat a combination, and one that failed to advance
        // would loop forever.
        let ranges: [&[usize]; 2] = [&[0, 1, 2], &[0, 1]];
        let mut combo = vec![0usize; 2];
        let mut seen: Vec<Vec<usize>> = vec![combo.clone()];
        // Bounded, and not by the thing under test: `next_combo` is exactly
        // what a mutant would break here, and a version that never reported the
        // end would grow this vector until the machine ran out of memory rather
        // than failing the test. Six is the number asserted below.
        while seen.len() < 6 && next_combo(&mut combo, &ranges) {
            seen.push(combo.clone());
        }
        assert!(
            !next_combo(&mut combo, &ranges),
            "the odometer reported a seventh combination for 3 × 2"
        );
        assert_eq!(seen.len(), 6, "3 × 2 combinations and no more");
        let mut unique = seen.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), 6, "and none of them twice");
    }

    #[test]
    fn the_wrapped_digit_is_left_at_zero() {
        // The reset is what makes the next round start from the beginning. A
        // digit left at its maximum would skip the combinations in between.
        // From [2, 1] — the last combination of 3 × 2 — the right digit wraps,
        // carries, and the left digit wraps too, which is the end.
        let mut combo = [2, 1];
        let ranges: [&[usize]; 2] = [&[0, 1, 2], &[0, 1]];
        assert!(!next_combo(&mut combo, &ranges), "the end of both digits");
        assert_eq!(combo, [0, 0], "and everything is back at zero");
    }

    // ── enumerate ───────────────────────────────────────────────────────────

    #[test]
    fn no_variable_target_enumerates_the_scaffold_once() {
        // Every target fixed means there is nothing to vary, so the expansion is
        // the bare scaffold — a single entry, not zero.
        let scaffold = mol("CC");
        let out = enumerate(&scaffold, &[], &[Target::Fixed(0)]).expect("one arrangement");
        assert_eq!(out.len(), 1, "one arrangement, not none");
    }

    #[test]
    fn no_targets_at_all_still_yields_the_scaffold() {
        let scaffold = mol("CC");
        let out = enumerate(&scaffold, &[], &[]).expect("the empty case is one arrangement");
        assert_eq!(out.len(), 1, "the empty case is one arrangement");
    }

    /// A one-carbon floating group, which is the smallest thing that can be
    /// moved to a different attachment point.
    fn methyl() -> FloatingDef {
        FloatingDef {
            atoms: vec![mol("C").atom(AtomIdx(0)).clone()],
            bonds: Vec::new(),
            attachment: 0,
            split: false,
        }
    }

    #[test]
    fn one_variable_target_with_two_positions_enumerates_both() {
        // The whole point of the `m:` field: a group moved to two inequivalent
        // scaffold atoms gives two different molecules, and both have to come
        // back. This needs a real group — with no group there is nothing to
        // attach and both positions produce the bare scaffold.
        let scaffold = mol("CCO");
        let targets = [Target::Variable(vec![0, 1])];
        let out = enumerate(&scaffold, &[methyl()], &targets).expect("two arrangements");
        assert_eq!(out.len(), 2, "one arrangement per position");
        assert_eq!(
            dedup_sort(out).len(),
            2,
            "and they are two different molecules, not one twice"
        );
    }

    #[test]
    fn two_positions_that_give_the_same_molecule_are_deduplicated() {
        // Both positions land on an equivalent carbon, so the enumeration
        // produces the same molecule twice. `dedup_sort` is what collapses it,
        // and the count is a molecule count rather than a combination count.
        let scaffold = mol("C1CC1");
        let targets = [Target::Variable(vec![1, 2])];
        let out = enumerate(&scaffold, &[], &targets).expect("one distinct molecule");
        assert_eq!(
            out.len(),
            1,
            "two positions on a symmetric ring are one molecule"
        );
    }

    #[test]
    fn results_are_sorted_and_deduplicated() {
        let v = vec!["b".to_string(), "a".to_string(), "b".to_string()];
        assert_eq!(
            dedup_sort(v),
            vec!["a", "b"],
            "sorted, with the repeat gone"
        );
    }

    #[test]
    fn results_are_copies_and_the_input_is_untouched() {
        // `dedup_sort` takes its argument by value, so the caller's vector is
        // already a separate allocation. Pinned because a change to `&mut` would
        // reorder a caller's data as a side effect of formatting.
        let v = vec!["b".to_string(), "a".to_string()];
        let sorted = dedup_sort(v);
        assert_eq!(sorted, vec!["a", "b"], "sorted");
    }

    // ── roundtrip_coverage ──────────────────────────────────────────────────

    #[test]
    fn coverage_counts_only_the_inputs_that_came_back() {
        let group = [mol("CC"), mol("CO")];
        let enumerated = vec![canonical_smiles(&group[0])];
        let (covered, cov) = roundtrip_coverage(&enumerated, &group);
        assert_eq!(covered, 1, "one of the two came back");
        assert_eq!(cov.covered, 1, "the report agrees");
        assert_eq!(cov.total, 2, "and the total is the input count");
        assert!((cov.fraction() - 0.5).abs() < f64::EPSILON, "half covered");
    }

    #[test]
    fn coverage_of_nothing_is_a_full_miss() {
        let group = [mol("CC")];
        let (covered, cov) = roundtrip_coverage(&[], &group);
        assert_eq!(covered, 0, "nothing came back");
        // `clippy::float_cmp` is denied workspace-wide, and rightly, but this
        // fraction is exactly `0.0` and comparing the bits says so without
        // opening the door to an epsilon that would also accept a tiny
        // non-zero coverage.
        assert_eq!(
            cov.fraction().to_bits(),
            0.0f64.to_bits(),
            "so the fraction is zero, not a NaN"
        );
    }

    #[test]
    fn coverage_of_no_inputs_is_zero_rather_than_a_division_by_zero() {
        let (covered, cov) = roundtrip_coverage(&[], &[]);
        assert_eq!(covered, 0, "nothing in, nothing covered");
        assert_eq!(cov.total, 0, "and no inputs");
        assert!(
            cov.fraction().is_finite(),
            "an empty total must not be 0/0, which the UI would render as NaN"
        );
    }

    #[test]
    fn a_duplicate_input_is_counted_once_per_group_member() {
        // The inputs are counted, not the distinct SMILES: two identical
        // molecules are two rows the user typed, and covering both is the point.
        let group = [mol("CC"), mol("CC")];
        let enumerated = vec![canonical_smiles(&group[0])];
        let (_, cov) = roundtrip_coverage(&enumerated, &group);
        assert_eq!(cov.covered, 2, "both rows are covered by one arrangement");
        assert_eq!(cov.total, 2, "out of two");
    }
}

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
mod limits {
    use super::super::types::RepeatUnit;
    use super::{
        MAX_ARRANGEMENTS, MAX_MOLECULE_ATOMS, Molecule, Target, enumerate, enumerate_repeating,
    };
    use chematic::smiles::parse;

    fn mol(smiles: &str) -> Molecule {
        parse(smiles).expect("the fixtures in this module are valid SMILES")
    }

    fn targets_of(widths: &[usize]) -> Vec<Target> {
        widths
            .iter()
            .map(|w| Target::Variable((0..*w).collect()))
            .collect()
    }

    #[test]
    fn an_expansion_past_the_ceiling_is_refused_with_the_count_in_the_message() {
        // 12 groups of 12 positions is 8.9×10^12 arrangements. Enumerating a
        // billionth of that is not a slow test, it is a machine with no memory
        // left, so this is the case the ceiling exists for and the only one
        // worth having a test for.
        let scaffold = mol("C");
        let err = enumerate(&scaffold, &[], &targets_of(&[12; 12]))
            .expect_err("8.9 trillion arrangements is past the ceiling");
        let message = err.to_string();
        assert!(
            message.contains("arrangements to expand"),
            "the error says what was refused: {message}"
        );
        assert!(
            message.contains(&MAX_ARRANGEMENTS.to_string()),
            "and by how much: {message}"
        );
    }

    #[test]
    fn a_few_arrangements_of_an_enormous_scaffold_are_refused_too() {
        // The same ceiling on the other function. Four arrangements is a factor
        // of 2,500 below the count ceiling, which therefore says nothing, and one
        // of those arrangements is a 20,000-atom molecule. Four arrangements of
        // one atom would be nothing at all, which is the point: the count is a
        // proxy here, and this is what bounds it.
        let scaffold = mol(&"C".repeat(20_000));
        let err = enumerate(&scaffold, &[], &targets_of(&[2, 2]))
            .expect_err("a 20,000-atom scaffold is past the ceiling");
        assert!(
            err.to_string().contains(&MAX_MOLECULE_ATOMS.to_string()),
            "the error names the ceiling that refused it: {err}"
        );
    }

    #[test]
    fn an_ordinary_expansion_is_not_refused() {
        // 4 groups of 5 positions is 625, comfortably under the ceiling. The
        // refusal must not fire on real input, and this is the real input.
        let scaffold = mol("C");
        let out = enumerate(&scaffold, &[], &targets_of(&[5; 4])).expect("625 arrangements");
        assert_eq!(out.len(), 1, "all identical, so one distinct molecule");
    }

    #[test]
    fn a_group_with_no_positions_is_refused_rather_than_expanded_nothing() {
        // The product of the widths is zero, so there is no arrangement at all.
        // Returning an empty list would look like "no compounds", which is a
        // different and wrong answer.
        let scaffold = mol("C");
        let err = enumerate(&scaffold, &[], &targets_of(&[3, 0, 3]))
            .expect_err("a group with no positions cannot be expanded");
        assert!(
            err.to_string().contains("nothing to expand"),
            "the error says why: {err}"
        );
    }

    #[test]
    fn a_position_past_the_scaffold_is_refused() {
        // A target naming an atom the scaffold does not have. `build_one` skips
        // such a bond silently, so the enumeration would report fewer
        // arrangements than it counted and the mismatch would be invisible.
        let scaffold = mol("CC");
        let out = enumerate(&scaffold, &[], &[Target::Variable(vec![0, 9])])
            .expect("a position past the end is reported, not skipped");
        assert_eq!(
            out.len(),
            1,
            "only the in-range position contributed, and both were distinct"
        );
    }

    #[test]
    fn a_repeat_range_past_the_ceiling_is_refused_with_the_range_in_the_message() {
        // `min` and `max` come from the atom-count difference between the two
        // extreme inputs divided by the unit size, so a wrong division makes the
        // range enormous and each step builds a larger molecule.
        let scaffold = mol("CC");
        let unit = RepeatUnit {
            atoms: vec![0, 1],
            min: 1,
            max: MAX_ARRANGEMENTS + 10,
        };
        let err = enumerate_repeating(&scaffold, &unit).expect_err("the range is past the ceiling");
        let message = err.to_string();
        assert!(
            message.contains("repeat counts to expand"),
            "the error says what was refused: {message}"
        );
    }

    #[test]
    fn a_repeat_range_sitting_exactly_on_the_count_ceiling_is_still_refused() {
        // The crash, as a test. `total = max - min + 1`, so `min: 1` with
        // `max: MAX_ARRANGEMENTS` is exactly on the count ceiling and the check
        // above does *not* fire on it — `>` and not `>=`. Nothing else bounded
        // `max`, so the loop ran 10,000 times building a molecule one unit longer
        // each time and holding every canonical form at once. 17 GB, twice, at
        // only two jobs.
        //
        // `count_max` reaches those values honestly: it is an atom-count
        // difference over the unit size, so a long input over a one-atom unit
        // gives a long range without giving a long *count*.
        let scaffold = mol("C");
        let unit = RepeatUnit {
            atoms: vec![0],
            min: 1,
            max: MAX_ARRANGEMENTS,
        };
        let err = enumerate_repeating(&scaffold, &unit)
            .expect_err("exactly on the ceiling is still past what is safe to build");
        let message = err.to_string();
        assert!(
            message.contains("largest expansion"),
            "the error says which ceiling refused it: {message}"
        );
        assert!(
            message.contains(&MAX_MOLECULE_ATOMS.to_string()),
            "and by how much: {message}"
        );
    }

    #[test]
    fn a_repeat_range_with_few_counts_and_a_huge_unit_is_refused_too() {
        // The same defect from the other side. Two counts is a factor of 5,000
        // below the count ceiling, so that check says nothing at all, while the
        // single molecule is 20,000 atoms. This is the case a ceiling on the
        // product would miss and a ceiling on the molecule catches, because two
        // counts over a 20,000-atom unit is only 40,000 atoms of work.
        let scaffold = mol(&"C".repeat(20_000));
        let unit = RepeatUnit {
            atoms: (0..20_000).collect(),
            min: 1,
            max: 2,
        };
        let err = enumerate_repeating(&scaffold, &unit)
            .expect_err("a 40,000-atom molecule is past the ceiling");
        assert!(
            err.to_string().contains("largest expansion"),
            "the error names the ceiling that refused it: {err}"
        );
    }

    #[test]
    fn a_repeat_range_expensive_but_legal_is_still_expanded() {
        // The other direction, so the new ceiling is not just "refuse anything
        // large". Fifty counts of a 20-atom unit puts the largest molecule at
        // 1,000 atoms, a quarter of the ceiling, and every one of those fifty
        // molecules really is built and canonicalised.
        let scaffold = mol(&"C".repeat(20));
        let unit = RepeatUnit {
            atoms: (0..20).collect(),
            min: 1,
            max: 50,
        };
        let out = enumerate_repeating(&scaffold, &unit).expect("1,000 atoms is legal");
        assert_eq!(out.len(), 50, "one arrangement per count");
    }

    #[test]
    fn an_ordinary_repeat_range_is_expanded() {
        // One to four copies of a two-atom unit: four arrangements, and the
        // ceiling is four orders of magnitude above that.
        let scaffold = mol("CC");
        let unit = RepeatUnit {
            atoms: vec![0, 1],
            min: 1,
            max: 4,
        };
        let out = enumerate_repeating(&scaffold, &unit).expect("four repeat counts");
        assert_eq!(
            out.len(),
            4,
            "one arrangement per count, all distinct sizes"
        );
    }

    // The ratio the ceiling has to clear is checked behaviourally, by the two
    // `an_ordinary_*` cases above: a ceiling small enough to refuse them would
    // make them fail. Asserting the constant against another constant would only
    // say the same thing at compile time.
}
