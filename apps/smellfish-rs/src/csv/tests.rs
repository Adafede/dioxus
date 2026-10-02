// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

// The `tests` tests, extracted from `csv.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `csv` module, so
// `use super::*` below reaches exactly what it did before the move.
use super::*;

#[test]
fn detects_smiles_and_label_columns() {
    let rows = parse_csv_rows("name,smiles\nalpha,C1CCCCC1\n").expect("rows");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].index, 1);
    assert_eq!(rows[0].label, "alpha");
    assert_eq!(rows[0].smiles, "C1CCCCC1");
}

#[test]
fn falls_back_to_generated_labels() {
    let rows = parse_csv_rows("smiles\nCCO\n").expect("rows");
    assert_eq!(rows[0].label, "Molecule 1");
}

#[test]
fn parses_plain_smiles_lines() {
    let rows = parse_csv_rows("CCO\nC1CCCCC1\n").expect("rows");
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].smiles, "CCO");
    assert_eq!(rows[1].smiles, "C1CCCCC1");
}
