// The `budget` tests, extracted from `repeating.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `repeating` module, so
// `use super::*` below reaches exactly what it did before the move.
use super::{Molecule, locate_repeat_in_scaffold, multiset};
use chematic::smiles::parse;

fn mol(smiles: &str) -> Molecule {
    parse(smiles).expect("the fixtures in this module are valid SMILES")
}

#[test]
fn a_unit_larger_than_the_scaffold_is_refused_immediately() {
    // Six atoms cannot hold a seven-atom fragment. The search would find
    // nothing, but it would get there by exhausting the budget, and the
    // budget is the expensive way to say no.
    let c = mol("CCCCCC");
    let err = locate_repeat_in_scaffold(&c, &[6; 7], 7)
        .expect_err("a seven-atom unit does not fit in a six-atom scaffold");
    assert!(
        err.to_string().contains("cannot lie inside"),
        "the error says the sizes do not fit: {err}"
    );
}

#[test]
fn a_unit_of_no_atoms_is_refused_rather_than_searched() {
    // Zero atoms is not a connected fragment, and the depth check
    // `frag.len() == unit_size` is never true for it, so the search would
    // walk every atom of the molecule and then report nothing.
    let c = mol("CCC");
    let err = locate_repeat_in_scaffold(&c, &[], 0).expect_err("zero atoms is not a unit");
    assert!(
        err.to_string().contains("cannot lie inside"),
        "the same guard covers it: {err}"
    );
}

#[test]
fn a_refusal_costs_nothing() {
    // The point of the size guard: a refusal is O(1). Measured as "the
    // budget is untouched", which is what makes the big-unit case cheap
    // rather than merely survivable.
    let c = mol("CCCCCC");
    assert!(
        locate_repeat_in_scaffold(&c, &[6; 7], 7).is_err(),
        "refused"
    );
    // And the same call with a size that fits is answered from the normal
    // path, which is what the budget is protecting.
    let target = multiset(&[1], &c);
    assert!(
        locate_repeat_in_scaffold(&c, &target, 1).is_ok(),
        "a one-atom unit is found, so the normal path is intact"
    );
}

#[test]
fn the_budget_is_far_above_what_a_real_search_spends() {
    // A one-atom unit over a twelve-atom molecule examines on the order of
    // atoms × degree fragments. The constant is six orders of magnitude
    // above that, so the guard never fires on real input and the tests that
    // pass today are not passing because of it.
    // A four-carbon chain: atom 1 is bonded to 0 and 2, so it has the two
    // external bonds `is_internal` is looking for.
    let c = mol("CCCC");
    let target = multiset(&[1], &c);
    let found = locate_repeat_in_scaffold(&c, &target, 1).expect("atom 1 is internal");
    assert_eq!(found, vec![1], "and it is the one found");
}
