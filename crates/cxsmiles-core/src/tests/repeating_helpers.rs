// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

use chematic::smiles::canonical_smiles;

use super::{chain_mol, mol};
use crate::graph::{components, multiset};
use crate::repeating::{
    center_dist, gcd, gcd_vec, is_internal, locate_repeat_in_scaffold, unit_multiset,
};
use crate::splice::{endpoint_for, splice_repeat};

#[test]
fn gcd_of_nothing_is_zero() {
    assert_eq!(gcd_vec(&[]), 0, "an empty fold starts at 0 and stays there");
}

#[test]
fn gcd_with_a_zero_is_the_other_number() {
    assert_eq!(gcd(7, 0), 7, "b == 0 terminates the recursion with a");
}

#[test]
fn gcd_is_the_largest_common_divisor() {
    assert_eq!(gcd(12, 18), 6, "12 and 18 share 6");
    assert_eq!(gcd(18, 12), 6, "and the order does not matter");
    assert_eq!(gcd(5, 5), 5, "a number divides itself");
    assert_eq!(gcd(1, 99), 1, "1 divides everything");
}

#[test]
fn gcd_vec_folds_across_every_element() {
    assert_eq!(gcd_vec(&[4, 6, 10]), 2, "gcd(4, 6, 10) is 2");
    assert_eq!(gcd_vec(&[3, 3, 3]), 3, "an all-equal list is that value");
    assert_eq!(
        gcd_vec(&[0, 5]),
        5,
        "a leading zero is the seed, not a constraint on the answer"
    );
}

#[test]
fn center_dist_is_zero_for_the_middle_atom() {
    assert_eq!(center_dist(&[2], 5), 0, "the middle atom has distance 0");
}

#[test]
fn center_dist_sums_the_offsets_before_taking_the_magnitude() {
    assert_eq!(center_dist(&[1, 3], 5), 0, "a symmetric pair is centred");
    assert_eq!(
        center_dist(&[1, 3], 5),
        center_dist(&[3, 1], 5),
        "and the atoms arrive in any order"
    );
    assert_eq!(center_dist(&[0, 4], 5), 0, "as does the outer pair");
}

#[test]
fn center_dist_grows_with_distance_from_the_middle() {
    assert!(
        center_dist(&[0], 5) > center_dist(&[1], 5),
        "atom 0 is further from the centre than atom 1"
    );
    assert_eq!(
        center_dist(&[2, 2], 5),
        0,
        "a doubled central atom is still at the centre"
    );
    assert!(
        center_dist(&[0, 0], 5) > center_dist(&[2, 2], 5),
        "a doubled off-centre atom is not"
    );
}

#[test]
fn multiset_is_the_sorted_elements_of_those_atoms() {
    let c = mol("CCC");
    assert_eq!(multiset(&[0, 1, 2], &c), vec![6, 6, 6], "three carbons");
}

#[test]
fn unit_multiset_divides_by_the_repeat_count() {
    let long = mol("CCCCCCCCC");
    assert_eq!(
        unit_multiset(&[0, 1, 2, 3, 4, 5, 6, 7, 8], &long, 3),
        vec![6, 6, 6],
        "nine pattern atoms over a unit of three is three carbons"
    );
}

#[test]
fn unit_multiset_truncates_a_pattern_that_is_not_a_whole_multiple() {
    let long = mol("CCCCC");
    assert_eq!(
        unit_multiset(&[0, 1, 2, 3, 4], &long, 2),
        vec![6, 6],
        "5 / 2 is 2, not 3"
    );
}

#[test]
fn unit_multiset_does_not_divide_by_zero_when_pattern_is_shorter_than_unit() {
    let long = mol("CCC");
    assert_eq!(
        unit_multiset(&[0], &long, 2),
        vec![6],
        "pattern shorter than unit_size clamps repeat count to 1"
    );
}

#[test]
fn a_fragment_with_two_external_bonds_is_internal() {
    let c = mol("CCC");
    assert!(is_internal(&[1], &c), "the middle carbon of a chain");
}

#[test]
fn a_terminal_atom_is_not_internal() {
    let c = mol("CCC");
    assert!(!is_internal(&[0], &c), "an end of a chain has one");
}

#[test]
fn an_isolated_atom_has_no_external_bonds_and_is_not_internal() {
    let c = mol("C");
    assert!(
        !is_internal(&[0], &c),
        "zero external bonds is not internal"
    );
}

#[test]
fn a_whole_ring_is_not_internal() {
    let ring = mol("C1CC1");
    assert!(
        !is_internal(&[0, 1, 2], &ring),
        "the whole ring, 0 external bonds"
    );
    assert!(is_internal(&[0], &ring), "one atom of it, 2 external bonds");
}

#[test]
fn locating_a_unit_that_is_not_there_is_an_error() {
    let c = mol("CCC");
    let err = locate_repeat_in_scaffold(&c, &[8], 1).expect_err("no oxygen in CCC");
    assert!(
        err.to_string().contains("could not locate"),
        "the error says what failed: {err}"
    );
}

#[test]
fn locating_prefers_the_fragment_closest_to_the_centre() {
    let c = mol("CCCCCCC");
    let found = locate_repeat_in_scaffold(&c, &[6], 1).expect("a carbon is present");
    assert_eq!(found.len(), 1, "a unit of one atom yields one index");
    assert_eq!(
        center_dist(&[u32::try_from(found[0]).unwrap()], c.atom_count()),
        0,
        "the middle atom of CCCCCCC is index 3"
    );
}

#[test]
fn repeat_size_and_budget_guards_refuse_invalid_or_pathological_inputs() {
    let c6 = mol("CCCCCC");
    let err = locate_repeat_in_scaffold(&c6, &[6; 7], 7)
        .expect_err("a seven-atom unit does not fit in six");
    assert!(err.to_string().contains("cannot lie inside"), "{err}");

    let c3 = mol("CCC");
    let err_zero = locate_repeat_in_scaffold(&c3, &[], 0).expect_err("zero atoms");
    assert!(
        err_zero.to_string().contains("cannot lie inside"),
        "{err_zero}"
    );

    let target = multiset(&[1], &c6);
    assert!(locate_repeat_in_scaffold(&c6, &target, 1).is_ok());

    let c4 = mol("CCCC");
    let found =
        locate_repeat_in_scaffold(&c4, &multiset(&[1], &c4), 1).expect("atom 1 is internal");
    assert_eq!(found, vec![1]);

    let m16 = chain_mol(16);
    let found14 = locate_repeat_in_scaffold(&m16, &[6; 14], 14).expect("14 of 16");
    assert_eq!(found14.len(), 14);
    assert!(!found14.contains(&0) && !found14.contains(&15));

    let m8 = chain_mol(8);
    let err_whole = locate_repeat_in_scaffold(&m8, &[6; 8], 8).expect_err("whole molecule");
    assert!(
        err_whole.to_string().contains("could not locate"),
        "{err_whole}"
    );
}

#[test]
fn endpoint_and_splice_repeat_preserve_connectivity_and_atom_counts() {
    let c = mol("CCC");
    assert_eq!(endpoint_for(&c, &[1, 2], 0), 1, "atom 1 touches anchor 0");
    let cco = mol("CCO");
    assert_eq!(
        endpoint_for(&cco, &[2, 3], 0),
        2,
        "fallback to first unit atom"
    );

    assert_eq!(
        canonical_smiles(&splice_repeat(&c, &[0, 1, 2], 1)),
        canonical_smiles(&c),
        "one copy is unchanged"
    );
    assert_eq!(
        canonical_smiles(&splice_repeat(&c, &[0, 1, 2], 0)),
        canonical_smiles(&c),
        "zero copies is unchanged"
    );
    assert_eq!(
        canonical_smiles(&splice_repeat(&c, &[], 3)),
        canonical_smiles(&c),
        "empty repeat unit returns scaffold"
    );
    let propene = mol("C=CC");
    assert_eq!(
        canonical_smiles(&splice_repeat(&propene, &[1], 1)),
        canonical_smiles(&propene),
        "one copy preserves non-single anchor bonds untouched"
    );

    let cc = mol("CC");
    let out = splice_repeat(&cc, &[0, 1], 3);
    assert_eq!(out.atom_count(), 6, "two-carbon unit x3 = 6 carbons");
    let every_atom: Vec<u32> = (0..u32::try_from(out.atom_count()).unwrap()).collect();
    assert_eq!(
        components(&every_atom, &out).len(),
        1,
        "spliced molecule stays one connected component"
    );
}
