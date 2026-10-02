// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

// The `tests` tests, extracted from `motif_display.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `motif_display` module, so
// `use super::*` below reaches exactly what it did before the move.
use super::*;

#[test]
fn natural_and_synthetic_are_distinct_buckets() {
    assert!(is_natural_source("natural"));
    assert!(!is_natural_source("synthetic"));
    assert!(!is_natural_source("unknown"));

    assert!(is_synthetic_source("synthetic"));
    assert!(!is_synthetic_source("natural"));
}

#[test]
fn chip_class_natural_is_np() {
    assert_eq!(chip_class_for("natural"), "chip chip-np");
    assert_eq!(chip_class_for("synthetic"), "chip alt");
    assert_eq!(chip_class_for("unknown"), "chip alt");
}

#[test]
fn display_label_includes_kingdom_for_natural() {
    let label = display_label("natural", "flavonoid", "plants", &["plants".to_string()]);
    assert!(label.contains("flavonoid"));
    assert!(label.contains("plants"));
}

#[test]
fn display_label_synthetic_prefix() {
    let label = display_label("synthetic", "ester", "", &[]);
    assert!(label.contains("synthetic"));
    assert!(label.contains("ester"));
}

#[test]
fn display_label_unclassified_prefix() {
    let label = display_label("unknown", "methyl", "", &[]);
    assert!(label.contains("unclassified"));
    assert!(label.contains("methyl"));
}
