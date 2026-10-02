// The `tests` tests, extracted from `repeating.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `repeating` module, so
// `use super::*` below reaches exactly what it did before the move.
use super::super::graph::components;
use super::*;
use chematic::smiles::{canonical_smiles, parse};

fn mol(smiles: &str) -> Molecule {
    parse(smiles).expect("the fixtures in this module are valid SMILES")
}

// ── gcd: the fold's seed is 0, and that is the whole contract ────────────

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
    // The fold is what makes this the GCD of *all* deltas rather than of
    // the first two, so a test with three distinct values is the one that
    // would catch a fold that stopped early.
    assert_eq!(gcd_vec(&[4, 6, 10]), 2, "gcd(4, 6, 10) is 2");
    assert_eq!(gcd_vec(&[3, 3, 3]), 3, "an all-equal list is that value");
    assert_eq!(
        gcd_vec(&[0, 5]),
        5,
        "a leading zero is the seed, not a constraint on the answer"
    );
}

// ── center_dist: which candidate wins ties ────────────────────────────────

#[test]
fn center_dist_is_zero_for_the_middle_atom() {
    // n = 5 puts the centre at index 2, so a single atom there is the
    // closest possible fragment to the centre of the scaffold.
    assert_eq!(center_dist(&[2], 5), 0, "the middle atom has distance 0");
}

#[test]
fn center_dist_sums_the_offsets_before_taking_the_magnitude() {
    // n = 5, centre 2. Atoms 1 and 3 are one step out on *opposite* sides,
    // so their signed offsets cancel and the fragment scores 0 — it is
    // centred. Summing the magnitudes instead, or forgetting the sum
    // entirely, would score the same fragment 2 and pick a different
    // candidate, so this is the assertion that fixes the order of the two
    // operations.
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
        "a doubled central atom is still at the centre, because the \
         offsets are summed and both are zero"
    );
    assert!(
        center_dist(&[0, 0], 5) > center_dist(&[2, 2], 5),
        "a doubled off-centre atom is not"
    );
}

// ── multiset / unit_multiset: the element counts, and the division ───────

#[test]
fn multiset_is_the_sorted_elements_of_those_atoms() {
    // CCC: three carbons.
    let c = mol("CCC");
    assert_eq!(multiset(&[0, 1, 2], &c), vec![6, 6, 6], "three carbons");
}

#[test]
fn unit_multiset_divides_by_the_repeat_count() {
    // The unit is one third of a nine-carbon pattern, so the per-unit
    // multiset is three carbons, not nine. This division is the whole
    // reason `unit_size` is a parameter, and a mutant that changes the
    // divisor changes the answer.
    let long = mol("CCCCCCCCC");
    assert_eq!(
        unit_multiset(&[0, 1, 2, 3, 4, 5, 6, 7, 8], &long, 3),
        vec![6, 6, 6],
        "nine pattern atoms over a unit of three is three carbons"
    );
}

#[test]
fn unit_multiset_truncates_a_pattern_that_is_not_a_whole_multiple() {
    // Five atoms of one element over a unit of two is two and a half units.
    // Integer division keeps two, and *not* rounding up: the count is a
    // number of atoms the unit is known to contain, so a third would be
    // invented. This is the one arithmetic in the file that a rounding
    // change would silently get wrong.
    let long = mol("CCCCC");
    assert_eq!(
        unit_multiset(&[0, 1, 2, 3, 4], &long, 2),
        vec![6, 6],
        "5 / 2 is 2, not 3"
    );
}

// ── is_internal: exactly two external bonds ──────────────────────────────

#[test]
fn a_fragment_with_two_external_bonds_is_internal() {
    // In CCC the middle atom has one bond out to each end: two external
    // bonds, which is the "internal" shape the search looks for.
    let c = mol("CCC");
    assert!(is_internal(&[1], &c), "the middle carbon of a chain");
}

#[test]
fn a_terminal_atom_is_not_internal() {
    // One external bond, not two.
    let c = mol("CCC");
    assert!(!is_internal(&[0], &c), "an end of a chain has one");
}

#[test]
fn an_isolated_atom_has_no_external_bonds_and_is_not_internal() {
    // A lone atom in a one-atom molecule has zero neighbours at all, which
    // is neither one nor two. `ext == 2` rather than `ext >= 2` is what
    // rejects it.
    let c = mol("C");
    assert!(
        !is_internal(&[0], &c),
        "zero external bonds is not internal"
    );
}

#[test]
fn a_whole_ring_is_not_internal() {
    // In C1CC1, taking *all three* atoms leaves no bond outside the
    // fragment, so it is a closed shell rather than a link in a chain.
    // Taking one atom of the same ring is internal — it has two bonds
    // leaving it — and that contrast is what the `== 2` is making.
    let ring = mol("C1CC1");
    assert!(
        !is_internal(&[0, 1, 2], &ring),
        "the whole ring, 0 external bonds"
    );
    assert!(is_internal(&[0], &ring), "one atom of it, 2 external bonds");
}

// ── locate_repeat_in_scaffold ────────────────────────────────────────────

#[test]
fn locating_a_unit_that_is_not_there_is_an_error() {
    // An oxygen multiset cannot be found in a hydrocarbon, and the error is
    // the only thing standing between this and a silently empty unit.
    let c = mol("CCC");
    let err = locate_repeat_in_scaffold(&c, &[8], 1).expect_err("no oxygen in CCC");
    assert!(
        err.to_string().contains("could not locate"),
        "the error says what failed: {err}"
    );
}

#[test]
fn locating_prefers_the_fragment_closest_to_the_centre() {
    // Two internal fragments of the same size exist in CCCCCCC; the search
    // must return the one nearer the middle, because that is the unit that
    // has a whole number of copies either side of it.
    let c = mol("CCCCCCC");
    let found = locate_repeat_in_scaffold(&c, &[6], 1).expect("a carbon is present");
    assert_eq!(found.len(), 1, "a unit of one atom yields one index");
    assert_eq!(
        center_dist(&[found[0] as u32], c.atom_count()),
        0,
        "the middle atom of CCCCCCC is index 3, and it is what is found"
    );
}

// ── endpoint_for ─────────────────────────────────────────────────────────

#[test]
fn the_endpoint_is_the_units_own_neighbour_of_the_anchor() {
    // CCC: anchor 0's only neighbour is 1, and 1 is in the unit.
    let c = mol("CCC");
    assert_eq!(endpoint_for(&c, &[1, 2], 0), 1, "atom 1 touches anchor 0");
}

#[test]
fn the_endpoint_falls_back_to_the_first_unit_atom() {
    // Anchor 0 in CCO has no neighbour in {2, 3} — atom 1 does — so the
    // fallback is the unit's own first atom.
    let c = mol("CCO");
    assert_eq!(
        endpoint_for(&c, &[2, 3], 0),
        2,
        "no match, so the unit's first atom is the endpoint"
    );
}

// ── splice_repeat ───────────────────────────────────────────────────────

#[test]
fn splicing_one_copy_is_the_scaffold() {
    // `n <= 1` is the guard that stops the loop below from asking for
    // `copy_atoms[0]` when there is no copy to ask for.
    let c = mol("CCC");
    assert_eq!(
        canonical_smiles(&splice_repeat(&c, &[0, 1, 2], 1)),
        canonical_smiles(&c),
        "one copy is the scaffold, unchanged"
    );
    assert_eq!(
        canonical_smiles(&splice_repeat(&c, &[0, 1, 2], 0)),
        canonical_smiles(&c),
        "zero copies is also the scaffold, rather than a panic"
    );
}

#[test]
fn splicing_three_copies_adds_six_carbons() {
    // CC + two more CC units: 2 + 2*2 = 6 atoms.
    let c = mol("CC");
    let out = splice_repeat(&c, &[0, 1], 3);
    assert_eq!(
        out.atom_count(),
        6,
        "a two-carbon unit repeated three times is six carbons"
    );
}

#[test]
fn splicing_keeps_the_copies_bonded_to_each_other() {
    // Each copy has to be bonded into the chain, not merely appended: a
    // pile of disconnected two-carbon fragments would have the right atom
    // count and the wrong molecule. One connected component is the
    // assertion; a bond count would depend on the degenerate fixture's
    // anchor handling, which is not what this test is about.
    let c = mol("CC");
    let out = splice_repeat(&c, &[0, 1], 3);
    let every_atom: Vec<u32> = (0..out.atom_count() as u32).collect();
    assert_eq!(
        components(&every_atom, &out).len(),
        1,
        "the spliced molecule is one piece, not several"
    );
}

// ── build_repeating ─────────────────────────────────────────────────────

#[test]
fn an_empty_group_is_an_error_and_not_a_panic() {
    let err = build_repeating(&[]).expect_err("there is nothing to build from");
    assert!(
        err.to_string().contains("group is empty"),
        "the error says which precondition failed: {err}"
    );
}
