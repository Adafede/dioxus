// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

use super::{chain_mol, mol};
use crate::error::CxError;
use crate::roundtrip::{MAX_ARRANGEMENTS, MAX_MOLECULE_ATOMS, enumerate, enumerate_repeating};
use crate::types::{AttachmentTarget, RepeatUnit};

fn targets_of(widths: &[usize]) -> Vec<AttachmentTarget> {
    widths
        .iter()
        .map(|w| AttachmentTarget::Variable((0..*w).collect()))
        .collect()
}

#[test]
fn an_expansion_past_the_ceiling_is_refused_with_the_count_in_the_message() {
    let scaffold = mol("C");
    let err = enumerate(&scaffold, &[], &targets_of(&[12; 12]))
        .expect_err("8.9 trillion arrangements is past the ceiling");
    let message = err.to_string();
    assert!(message.contains("arrangements to expand"), "{message}");
    assert!(message.contains(&MAX_ARRANGEMENTS.to_string()), "{message}");
}

#[test]
fn a_few_arrangements_of_a_scaffold_past_the_atom_ceiling_are_refused_too() {
    let scaffold = chain_mol(MAX_MOLECULE_ATOMS + 1);
    let err = enumerate(&scaffold, &[], &targets_of(&[2, 2]))
        .expect_err("one atom past the ceiling is past it");
    assert!(err.to_string().contains(&MAX_MOLECULE_ATOMS.to_string()));
}

#[test]
fn an_ordinary_expansion_is_not_refused() {
    let scaffold = chain_mol(5);
    let out = enumerate(&scaffold, &[], &targets_of(&[5; 4])).expect("625 arrangements");
    assert_eq!(out.len(), 1, "all identical, so one distinct molecule");
}

#[test]
fn a_group_with_no_positions_is_refused_rather_than_expanded_nothing() {
    let scaffold = mol("C");
    let err = enumerate(&scaffold, &[], &targets_of(&[3, 0, 3]))
        .expect_err("a group with no positions cannot be expanded");
    assert!(err.to_string().contains("nothing to expand"), "{err}");
}

#[test]
fn a_position_past_the_scaffold_is_refused() {
    let scaffold = mol("CC");
    let err = enumerate(&scaffold, &[], &[AttachmentTarget::Variable(vec![0, 9])])
        .expect_err("an out-of-range position must return InvalidPosition");
    assert!(
        matches!(
            err,
            CxError::InvalidPosition {
                pos: 9,
                group_len: 2
            }
        ),
        "unexpected error: {err}"
    );
}

#[test]
fn a_repeat_range_past_the_ceiling_is_refused_with_the_range_in_the_message() {
    let scaffold = mol("CC");
    let unit = RepeatUnit {
        atoms: vec![0, 1],
        min: 1,
        max: MAX_ARRANGEMENTS + 10,
    };
    let err = enumerate_repeating(&scaffold, &unit).expect_err("the range is past the ceiling");
    assert!(err.to_string().contains("repeat counts to expand"), "{err}");
}

#[test]
fn a_repeat_range_sitting_exactly_on_the_count_ceiling_is_still_refused() {
    let scaffold = chain_mol(MAX_MOLECULE_ATOMS + 1);
    let unit = RepeatUnit {
        atoms: (0..MAX_MOLECULE_ATOMS).collect(),
        min: 1,
        max: MAX_ARRANGEMENTS,
    };
    let err = enumerate_repeating(&scaffold, &unit)
        .expect_err("exactly on the ceiling is still past what is safe to build");
    let message = err.to_string();
    assert!(message.contains("largest expansion"), "{message}");
    assert!(
        message.contains(&MAX_MOLECULE_ATOMS.to_string()),
        "{message}"
    );
}

#[test]
fn a_repeat_range_with_few_counts_and_a_huge_unit_is_refused_too() {
    let scaffold = chain_mol(20_000);
    let unit = RepeatUnit {
        atoms: (0..20_000).collect(),
        min: 1,
        max: 2,
    };
    let err = enumerate_repeating(&scaffold, &unit)
        .expect_err("a 40,000-atom molecule is past the ceiling");
    assert!(err.to_string().contains("largest expansion"), "{err}");
}

#[test]
fn repeat_multiplication_overflow_returns_expansion_overflow() {
    let scaffold = mol("CC");
    let unit = RepeatUnit {
        atoms: vec![0, 1],
        min: usize::MAX - 1,
        max: usize::MAX,
    };
    let err = enumerate_repeating(&scaffold, &unit).expect_err("usize::MAX * 2 overflows usize");
    assert!(
        matches!(err, CxError::ExpansionOverflow { .. }),
        "expected ExpansionOverflow, got {err:?}"
    );
}

#[test]
fn a_repeat_range_expensive_but_legal_is_still_expanded() {
    let scaffold = chain_mol(10);
    let unit = RepeatUnit {
        atoms: (0..10).collect(),
        min: 1,
        max: 50,
    };
    let out = enumerate_repeating(&scaffold, &unit).expect("500 atoms is legal");
    assert_eq!(out.len(), 50, "one arrangement per count");
}

#[test]
fn an_ordinary_repeat_range_is_expanded() {
    let scaffold = mol("CC");
    let unit = RepeatUnit {
        atoms: vec![0, 1],
        min: 1,
        max: 4,
    };
    let out = enumerate_repeating(&scaffold, &unit).expect("four repeat counts");
    assert_eq!(out.len(), 4, "one arrangement per count");
}

#[test]
fn positional_and_atom_ceilings_stand_on_exact_boundaries() {
    let c100 = chain_mol(100);
    let on_count = enumerate(&c100, &[], &targets_of(&[100, 100]))
        .expect("10,000 sitting on ceiling is legal");
    assert_eq!(on_count.len(), 1);

    let c137 = chain_mol(137);
    let past_count = enumerate(&c137, &[], &targets_of(&[73, 137]))
        .expect_err("10,001 is past the count ceiling");
    assert!(past_count.to_string().contains("arrangements to expand"));

    let on_atoms = chain_mol(MAX_MOLECULE_ATOMS);
    let on_atoms_out =
        enumerate(&on_atoms, &[], &targets_of(&[1])).expect("exact atom limit is legal");
    assert_eq!(on_atoms_out.len(), 1);

    let past_atoms = chain_mol(MAX_MOLECULE_ATOMS + 1);
    let err_atoms = enumerate(&past_atoms, &[], &targets_of(&[1]))
        .expect_err("one atom past the ceiling is refused");
    assert!(
        err_atoms
            .to_string()
            .contains(&MAX_MOLECULE_ATOMS.to_string())
    );

    let unit_ok = RepeatUnit {
        atoms: (0..MAX_MOLECULE_ATOMS).collect(),
        min: 1,
        max: 1,
    };
    assert_eq!(
        enumerate_repeating(&on_atoms, &unit_ok)
            .expect("exact repeat atom ceiling")
            .len(),
        1
    );

    let past_repeat_scaffold = chain_mol(MAX_MOLECULE_ATOMS + 2);
    let unit_over = RepeatUnit {
        atoms: (0..=MAX_MOLECULE_ATOMS).collect(),
        min: 1,
        max: 1,
    };
    let err_repeat = enumerate_repeating(&past_repeat_scaffold, &unit_over)
        .expect_err("one atom past the repeat ceiling");
    assert!(err_repeat.to_string().contains("largest expansion"));
}
