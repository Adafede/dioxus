// The `pathological` tests, extracted from `repeating.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `repeating` module, so
// `use super::*` below reaches exactly what it did before the move.
use super::*;
use chematic::smiles::parse;

fn chain(atoms: usize) -> Molecule {
    parse(&"C".repeat(atoms)).expect("a chain of carbons")
}

#[test]
fn a_unit_larger_than_the_scaffold_never_starts_a_search() {
    let mol = chain(6);
    let err = locate_repeat_in_scaffold(&mol, &[6; 7], 7)
        .expect_err("a seven-atom unit does not fit in six atoms");
    assert!(err.to_string().contains("cannot lie inside"), "{err}");
}

#[test]
fn a_unit_that_is_most_of_the_scaffold_still_answers() {
    // Fourteen of sixteen atoms: the largest fraction of a molecule that is
    // still a plausible repeat unit, and the case where the search does the
    // most work it does for any real input. It answers, and it answers with
    // a fragment rather than with an error.
    let mol = chain(16);
    let found = locate_repeat_in_scaffold(&mol, &[6; 14], 14).expect("fourteen of sixteen");
    assert_eq!(found.len(), 14, "the whole interior of the chain");
    assert!(
        !found.contains(&0) && !found.contains(&15),
        "and it excludes the two ends, which are not internal: {found:?}"
    );
}

#[test]
fn a_unit_equal_to_the_whole_scaffold_answers_rather_than_looping() {
    // Every atom. There is no interior, so nothing qualifies — but the
    // search has to finish to say so, and it does.
    let mol = chain(8);
    let err = locate_repeat_in_scaffold(&mol, &[6; 8], 8)
        .expect_err("a whole molecule has no internal fragment of itself");
    assert!(
        err.to_string().contains("could not locate"),
        "not found, rather than not finished: {err}"
    );
}
