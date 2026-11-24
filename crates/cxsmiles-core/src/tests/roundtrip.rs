// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

use chematic::core::{AtomIdx, BondOrder};
use chematic::smiles::canonical_smiles;

use super::mol;
use crate::fragment::FloatingDef;
use crate::roundtrip::{dedup_sort, enumerate, next_combo, roundtrip_coverage};
use crate::types::AttachmentTarget;

fn methyl() -> FloatingDef {
    FloatingDef {
        atoms: vec![mol("C").atom(AtomIdx(0)).clone()],
        bonds: Vec::new(),
        attachment: 0,
        attach_order: BondOrder::Single,
        split: false,
    }
}

#[test]
fn an_exhausted_combo_reports_that_there_is_no_next() {
    assert!(
        !next_combo(&mut [0], &[&[0usize]]),
        "a single exhausted digit is the end"
    );
}

#[test]
fn a_combo_advances_from_the_last_digit_first() {
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
    let ranges: [&[usize]; 2] = [&[0, 1, 2], &[0, 1]];
    let mut combo = vec![0usize; 2];
    let mut seen: Vec<Vec<usize>> = vec![combo.clone()];
    while seen.len() < 6 && next_combo(&mut combo, &ranges) {
        seen.push(combo.clone());
    }
    assert!(
        !next_combo(&mut combo, &ranges),
        "the odometer reported a seventh combination for 3 x 2"
    );
    assert_eq!(seen.len(), 6, "3 x 2 combinations and no more");
    let mut unique = seen;
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), 6, "and none of them twice");
}

#[test]
fn the_wrapped_digit_is_left_at_zero() {
    let mut combo = [2, 1];
    let ranges: [&[usize]; 2] = [&[0, 1, 2], &[0, 1]];
    assert!(!next_combo(&mut combo, &ranges), "the end of both digits");
    assert_eq!(combo, [0, 0], "and everything is back at zero");
}

#[test]
fn no_variable_target_enumerates_the_scaffold_once() {
    let scaffold = mol("CC");
    let out = enumerate(&scaffold, &[], &[AttachmentTarget::Fixed(0)]).expect("one arrangement");
    assert_eq!(out.len(), 1, "one arrangement, not none");
}

#[test]
fn no_targets_at_all_still_yields_the_scaffold() {
    let scaffold = mol("CC");
    let out = enumerate(&scaffold, &[], &[]).expect("the empty case is one arrangement");
    assert_eq!(out.len(), 1, "the empty case is one arrangement");
}

#[test]
fn one_variable_target_with_two_positions_enumerates_both() {
    let scaffold = mol("CCO");
    let targets = [AttachmentTarget::Variable(vec![0, 1])];
    let out = enumerate(&scaffold, &[methyl()], &targets).expect("two arrangements");
    assert_eq!(out.len(), 2, "one arrangement per position");
    assert_eq!(dedup_sort(out).len(), 2, "two distinct molecules");
}

#[test]
fn two_positions_that_give_the_same_molecule_are_deduplicated() {
    let scaffold = mol("C1CC1");
    let targets = [AttachmentTarget::Variable(vec![1, 2])];
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
fn coverage_counts_only_the_inputs_that_came_back() {
    let group = [mol("CC"), mol("CO")];
    let enumerated = dedup_sort(vec![canonical_smiles(&group[0])]);
    let cov = roundtrip_coverage(&enumerated, &group);
    assert_eq!(cov.covered, 1, "one of the two came back");
    assert_eq!(cov.total, 2, "and the total is the input count");
    assert!(!cov.is_clean(), "1/2 is not clean");
    assert!((cov.fraction() - 0.5).abs() < f64::EPSILON, "half covered");
}

#[test]
fn coverage_of_nothing_is_a_full_miss() {
    let group = [mol("CC")];
    let cov = roundtrip_coverage(&[], &group);
    assert_eq!(cov.covered, 0, "nothing came back");
    assert!(!cov.is_clean(), "0/1 is not clean");
    assert_eq!(
        cov.fraction().to_bits(),
        0.0f64.to_bits(),
        "fraction is 0.0"
    );
}

#[test]
fn coverage_of_no_inputs_is_zero_rather_than_a_division_by_zero() {
    let cov = roundtrip_coverage(&[], &[]);
    assert_eq!(cov.covered, 0, "nothing in, nothing covered");
    assert_eq!(cov.total, 0, "and no inputs");
    assert!(!cov.is_clean(), "0/0 is not clean");
    assert!(cov.fraction().is_finite(), "empty total must not be NaN");
}

#[test]
fn a_duplicate_input_is_counted_once_per_group_member() {
    let group = [mol("CC"), mol("CC")];
    let enumerated = dedup_sort(vec![canonical_smiles(&group[0])]);
    let cov = roundtrip_coverage(&enumerated, &group);
    assert_eq!(cov.covered, 2, "both rows are covered by one arrangement");
    assert_eq!(cov.total, 2, "out of two");
    assert!(cov.is_clean(), "2/2 is clean");
}
