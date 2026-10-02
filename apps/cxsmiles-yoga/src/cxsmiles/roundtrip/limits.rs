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
