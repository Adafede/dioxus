// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

// The `limits` tests, extracted from `roundtrip.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `roundtrip` module, so
// `use super::*` below reaches exactly what it did before the move.
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
fn a_few_arrangements_of_a_scaffold_past_the_atom_ceiling_are_refused_too() {
    // The same ceiling on the other function. Four arrangements is a factor
    // of 2,500 below the count ceiling, which therefore says nothing, and one
    // of those arrangements is a molecule past the ceiling. Four arrangements of
    // one atom would be nothing at all, which is the point: the count is a
    // proxy here, and this is what bounds it.
    //
    // Four arrangements rather than one, and a scaffold barely past the
    // ceiling rather than five times past it, for a reason that only shows up
    // under mutation: with this check broken the four arrangements really are
    // built, and at 20,000 atoms each takes a minute to canonicalise, which
    // turns a killed mutant into a 300-second timeout. Four arrangements of
    // 4,097 atoms is four canonicalisations, and it is the difference between
    // a timeout and a kill.
    let scaffold = mol(&"C".repeat(MAX_MOLECULE_ATOMS + 1));
    let err = enumerate(&scaffold, &[], &targets_of(&[2, 2]))
        .expect_err("one atom past the ceiling is past it");
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
    // difference over the unit size, so a long input over a large unit gives a
    // long range without giving a long *count*.
    //
    // The unit is `MAX_MOLECULE_ATOMS` atoms rather than one, which is what
    // keeps the mutant cheap. With the molecule ceiling broken this test's
    // 10,000 counts are walked for real, and over a one-atom unit that is
    // 10,000 canonicalisations of chains up to 4,096 atoms long — minutes,
    // and a timeout rather than a kill. Over a ceiling-sized unit the loop is
    // capped at a single count whatever happens to the checks, because a
    // molecule has to stay inside `MAX_MOLECULE_ATOMS` to be built at all.
    let scaffold = mol(&"C".repeat(MAX_MOLECULE_ATOMS + 1));
    let unit = RepeatUnit {
        atoms: (0..MAX_MOLECULE_ATOMS).collect(),
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

// ── the two sides of a ceiling, one atom and one arrangement apart ─────────
//
// Every ceiling above is `x > LIMIT`, and `>` has two mutants that no test in
// the file was reaching: `>=` refuses an input sitting exactly on the ceiling,
// which is legal, and `==` refuses only that one input, which lets everything
// above it through — the 17 GB, once more.
//
// Reaching them is not a matter of writing a slow test. Every arrangement of a
// one-atom scaffold is a one-atom molecule, so ten thousand of them is eleven
// milliseconds, and ten thousand *plus one* is the same eleven milliseconds
// rather than the minute it takes when the guard is what is missing. Both sides
// of both ceilings are therefore cheap to stand on, and these four tests are
// what stand on them.

#[test]
fn an_expansion_sitting_exactly_on_the_count_ceiling_is_expanded() {
    // 100 x 100 is 10,000, which is the ceiling, and `>` is `>`.
    let scaffold = mol("C");
    let out = enumerate(&scaffold, &[], &targets_of(&[100, 100]))
        .expect("sitting on the ceiling is legal; only past it is not");
    assert_eq!(out.len(), 1, "all identical, so one distinct molecule");
}

#[test]
fn one_past_the_count_ceiling_is_refused() {
    // 73 x 137 is 10,001: the first integer above the ceiling that factors
    // into two group widths. With `>` mutated to `==` this expands, in about
    // the time the ten thousand above it takes.
    let scaffold = mol("C");
    let err = enumerate(&scaffold, &[], &targets_of(&[73, 137]))
        .expect_err("10,001 arrangements is one past the ceiling");
    assert!(
        err.to_string().contains("arrangements to expand"),
        "and it is the count ceiling that says so: {err}"
    );
}

#[test]
fn a_scaffold_sitting_exactly_on_the_atom_ceiling_is_expanded() {
    // 4,096 atoms is the ceiling, and one arrangement of it is one
    // canonicalisation of one four-thousand-atom chain.
    let scaffold = mol(&"C".repeat(MAX_MOLECULE_ATOMS));
    let out = enumerate(&scaffold, &[], &targets_of(&[1]))
        .expect("sitting on the ceiling is legal; only past it is not");
    assert_eq!(out.len(), 1, "one arrangement, one molecule");
}

#[test]
fn one_atom_past_the_atom_ceiling_is_refused() {
    let scaffold = mol(&"C".repeat(MAX_MOLECULE_ATOMS + 1));
    let err = enumerate(&scaffold, &[], &targets_of(&[1]))
        .expect_err("one atom past the ceiling is past it");
    assert!(
        err.to_string().contains(&MAX_MOLECULE_ATOMS.to_string()),
        "and the message names the ceiling: {err}"
    );
}

#[test]
fn an_expansion_sitting_exactly_on_the_atom_ceiling_is_expanded() {
    // The same boundary on `enumerate_repeating`, where it is `max x unit_size`
    // rather than the scaffold's own size. One count of a 4,096-atom unit, so
    // the largest molecule is the ceiling and nothing is being multiplied.
    let scaffold = mol(&"C".repeat(MAX_MOLECULE_ATOMS + 1));
    let unit = RepeatUnit {
        atoms: (0..MAX_MOLECULE_ATOMS).collect(),
        min: 1,
        max: 1,
    };
    let out = enumerate_repeating(&scaffold, &unit).expect("exactly on the ceiling");
    assert_eq!(out.len(), 1, "one count, one molecule");
}

#[test]
fn one_atom_past_the_atom_ceiling_is_refused_on_the_repeat_side_too() {
    let scaffold = mol(&"C".repeat(MAX_MOLECULE_ATOMS + 2));
    let unit = RepeatUnit {
        atoms: (0..=MAX_MOLECULE_ATOMS).collect(),
        min: 1,
        max: 1,
    };
    let err = enumerate_repeating(&scaffold, &unit).expect_err("one atom past the ceiling");
    assert!(
        err.to_string().contains("largest expansion"),
        "and it is the molecule ceiling that says so: {err}"
    );
}
