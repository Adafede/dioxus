// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

use super::*;
use smellfish_core::model::SourceClass;

#[test]
fn natural_and_synthetic_are_distinct_buckets() {
    assert!(is_natural_source(SourceClass::Natural));
    assert!(!is_natural_source(SourceClass::Synthetic));
    assert!(!is_natural_source(SourceClass::Unknown));

    assert!(is_synthetic_source(SourceClass::Synthetic));
    assert!(!is_synthetic_source(SourceClass::Natural));
}

#[test]
fn chip_class_natural_is_np() {
    assert_eq!(chip_class_for(SourceClass::Natural), "chip chip-np");
    assert_eq!(chip_class_for(SourceClass::Synthetic), "chip chip-synth");
    assert_eq!(chip_class_for(SourceClass::Unknown), "chip alt");
}

#[test]
fn display_label_includes_kingdom_for_natural() {
    let label = display_label(
        SourceClass::Natural,
        "flavonoid",
        "plants",
        &["plants".to_string()],
    );
    assert!(label.contains("flavonoid"));
    assert!(label.contains("plants"));
}

#[test]
fn display_label_synthetic_and_unclassified_prefixes() {
    let synthetic = display_label(SourceClass::Synthetic, "ester", "", &[]);
    assert!(synthetic.contains("synthetic") && synthetic.contains("ester"));

    let unknown = display_label(SourceClass::Unknown, "methyl", "", &[]);
    assert!(unknown.contains("unclassified") && unknown.contains("methyl"));
}
