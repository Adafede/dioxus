// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

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

// ── repeat_pattern: which unmatched component is the repeat unit ─────────

/// The pattern for `longest` given a `scaffold` query, in one call.
fn pattern_of(longest: &str, scaffold: &str, unit_size: usize) -> Vec<u32> {
    let long = mol(longest);
    let q = molecule_to_query(&mol(scaffold));
    repeat_pattern(&mol(scaffold), &long, unit_size, &q).expect("the scaffold is in the longest")
}

#[test]
fn the_pattern_is_the_whole_of_the_extra_atoms() {
    // "CC" aligned into "CCCCCC" leaves four carbons, one component of four,
    // and a unit of two divides it.
    assert_eq!(
        pattern_of("CCCCCC", "CC", 2),
        vec![2, 3, 4, 5],
        "the four atoms the scaffold did not match"
    );
}

/// A component that is *not* a whole number of units must not be chosen.
///
/// "CCCCCCO" aligned on "CC" leaves a chain of four carbons and a lone oxygen —
/// two components, one of four atoms and one of one. A unit of two divides the
/// four and not the one, so the four is the repeat and the oxygen is not.
///
/// This is the case the filter in `repeat_pattern` exists for, and every one
/// of its six mutants is invisible without it: the smallest component is the
/// oxygen, so a filter that lets anything through picks it, and the fallback
/// `min_by_key` picks it too. `min_by_key` alone — or a filter that only
/// checked the divisibility, or only the size — answers `vec![6]`.
#[test]
fn a_component_that_is_not_a_whole_number_of_units_is_not_the_pattern() {
    assert_eq!(
        pattern_of("CCCCCC.O", "CC", 2),
        vec![2, 3, 4, 5],
        "four carbons, not the single oxygen"
    );
}

/// And the same, for a component that is long enough but not divisible.
///
/// "CCCCCCC.O" aligned on "CC" leaves five carbons and an oxygen. The filter
/// asks for `len % unit_size == 0`, and seven is not even — but it is
/// certainly not *less than* a unit of two either, so a filter that asked only
/// the size question, or that accepted either condition rather than both, walks
/// straight past the divisibility and takes the seven.
#[test]
fn a_long_component_that_does_not_divide_is_not_the_pattern() {
    assert_eq!(
        pattern_of("CCCCCCC.O", "CC", 2),
        vec![7],
        "the oxygen, because nothing else is a whole number of units"
    );
}

/// The fallback, for the case where nothing divides.
///
/// With no component satisfying the filter the answer is the smallest one, so
/// `repeat_pattern` has something to say rather than refusing. "CC.CCO" over
/// "CC" with a unit of two leaves three atoms, which is one and a half units.
#[test]
fn the_smallest_component_is_used_when_nothing_divides() {
    assert_eq!(
        pattern_of("CC.CCO", "CC", 2),
        vec![2, 3, 4],
        "nothing is a whole number of two-atom units, so the smallest wins"
    );
}

// ── add_scaffold_bonds: which scaffold bonds survive the splice ──────────

/// The bonds `add_scaffold_bonds` copies out of a four-carbon chain, given
/// two anchor/endpoint pairs.
fn copied_bonds(scaffold: &Molecule, a: (u32, u32), b: (u32, u32)) -> Vec<(u32, u32)> {
    let mut builder = MoleculeBuilder::new();
    let mapped: Vec<AtomIdx> = scaffold
        .atoms()
        .map(|(_, atom)| builder.add_atom(atom.clone()))
        .collect();
    add_scaffold_bonds(&mut builder, scaffold, &mapped, a.0, a.1, b.0, b.1);
    let out = builder.build();
    out.bonds()
        .map(|(_, be)| (be.atom1.0, be.atom2.0))
        .collect()
}

/// Both anchor bonds are dropped, and only those.
///
/// CCCC is 0-1-2-3. Anchors (0,1) and (2,3) name the two bonds the splice
/// replaces, and 1-2 is the one in the middle that it keeps. All four disjuncts
/// of `is_anchor_bond` have a bond of their own here — the first pair forwards,
/// the second pair reversed — so the expression cannot be answered by three of
/// them, and joining them with `&&` instead of `||` copies all four bonds back
/// and leaves the unit attached at both ends of the original.
#[test]
fn both_anchor_bonds_are_dropped_and_the_rest_are_kept() {
    let chain = mol("CCCC");
    let copied = copied_bonds(&chain, (0, 1), (2, 3));
    assert_eq!(
        copied,
        vec![(1, 2)],
        "the middle bond survives; the two anchor bonds do not"
    );
}

/// One anchor pair on its own, to say the pairs are independent.
///
/// A single pair drops one bond and keeps the other two, which is what the
/// reversed disjunction in the middle of `is_anchor_bond` is for: the bond is
/// named from the anchor's side and from the endpoint's side, and both orderings
/// are the same bond.
#[test]
fn one_anchor_bond_is_dropped_either_way_round() {
    let chain = mol("CCCC");
    assert_eq!(
        copied_bonds(&chain, (0, 1), (9, 9)),
        vec![(1, 2), (2, 3)],
        "naming the bond forwards drops it"
    );
    assert_eq!(
        copied_bonds(&chain, (1, 0), (9, 9)),
        vec![(1, 2), (2, 3)],
        "and naming it backwards drops the same bond"
    );
}
