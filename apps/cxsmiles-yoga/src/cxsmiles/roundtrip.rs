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
        while next_combo(&mut combo, &ranges) {
            seen.push(combo.clone());
        }
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
        let out = enumerate(&scaffold, &[], &[Target::Fixed(0)]);
        assert_eq!(out.len(), 1, "one arrangement, not none");
    }

    #[test]
    fn no_targets_at_all_still_yields_the_scaffold() {
        let scaffold = mol("CC");
        let out = enumerate(&scaffold, &[], &[]);
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
        let out = enumerate(&scaffold, &[methyl()], &targets);
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
        let out = enumerate(&scaffold, &[], &targets);
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
        let enumerated = vec![chematic::smiles::canonical_smiles(&group[0])];
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
        let enumerated = vec![chematic::smiles::canonical_smiles(&group[0])];
        let (_, cov) = roundtrip_coverage(&enumerated, &group);
        assert_eq!(cov.covered, 2, "both rows are covered by one arrangement");
        assert_eq!(cov.total, 2, "out of two");
    }
}
