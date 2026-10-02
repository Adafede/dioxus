// The `tests` tests, extracted from `roundtrip.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `roundtrip` module, so
// `use super::*` below reaches exactly what it did before the move.
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
