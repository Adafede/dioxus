// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! Motif classification + display formatting.
//!
//! The original `results.rs` had two near-identical sets of helpers —
//! `summary_is_natural` / `motif_is_natural` etc. — differing only by
//! whether they wrapped `MotifSummary` or `RdkitMotifHit`.  Both types
//! expose a `source_class: String` field, so the classification logic
//! collapses to a single function that takes `&str`.

use crate::model::{MotifSummary, RdkitMotifHit, normalized_source_class};

/// Classify a source-class string into one of three buckets.
fn classify_source(source_class: &str) -> &str {
    normalized_source_class(source_class)
}

/// Returns `true` when the source class indicates a natural-product origin.
#[must_use]
pub fn is_natural_source(source_class: &str) -> bool {
    classify_source(source_class) == "natural"
}

/// Returns `true` when the source class indicates a synthetic origin.
#[must_use]
pub fn is_synthetic_source(source_class: &str) -> bool {
    classify_source(source_class) == "synthetic"
}

/// Returns `true` when the source class is unclassified (neither natural nor synthetic).
#[must_use]
pub fn is_unclassified_source(source_class: &str) -> bool {
    classify_source(source_class) == "unknown"
}

/// Returns `"chip chip-np"` for natural sources, `"chip alt"` otherwise.
#[must_use]
pub fn chip_class_for(source_class: &str) -> &'static str {
    if is_natural_source(source_class) {
        "chip chip-np"
    } else {
        "chip alt"
    }
}

/// Build the display label for a motif, including kingdom attribution
/// for natural sources.
#[must_use]
pub fn display_label(
    source_class: &str,
    label: &str,
    kingdom: &str,
    kingdoms: &[String],
) -> String {
    if is_natural_source(source_class) {
        let kingdoms_str = if kingdoms.is_empty() {
            kingdom.to_string()
        } else if kingdoms.len() == 1 {
            kingdoms[0].clone()
        } else {
            kingdoms.join(" + ")
        };
        format!("{kingdoms_str} \u{00b7} {label}")
    } else if is_synthetic_source(source_class) {
        format!("synthetic \u{00b7} {label}")
    } else {
        format!("unclassified \u{00b7} {label}")
    }
}

// ─── Thin wrappers for MotifSummary ───────────────────────────────────

#[must_use]
pub fn summary_is_natural(motif: &MotifSummary) -> bool {
    is_natural_source(&motif.source_class)
}

#[must_use]
pub fn summary_is_synthetic(motif: &MotifSummary) -> bool {
    is_synthetic_source(&motif.source_class)
}

#[must_use]
pub fn summary_is_unclassified(motif: &MotifSummary) -> bool {
    is_unclassified_source(&motif.source_class)
}

#[must_use]
pub fn summary_chip_class(motif: &MotifSummary) -> &'static str {
    chip_class_for(&motif.source_class)
}

#[must_use]
pub fn summary_display_label(motif: &MotifSummary) -> String {
    display_label(
        &motif.source_class,
        &motif.label,
        &motif.kingdom,
        &motif.kingdoms,
    )
}

// ─── Thin wrappers for RdkitMotifHit ──────────────────────────────────

#[must_use]
pub fn motif_is_natural(motif: &RdkitMotifHit) -> bool {
    is_natural_source(&motif.source_class)
}

#[must_use]
pub fn motif_is_synthetic(motif: &RdkitMotifHit) -> bool {
    is_synthetic_source(&motif.source_class)
}

#[must_use]
pub fn motif_is_unclassified(motif: &RdkitMotifHit) -> bool {
    is_unclassified_source(&motif.source_class)
}

#[must_use]
pub fn motif_chip_class(motif: &RdkitMotifHit) -> &'static str {
    chip_class_for(&motif.source_class)
}

#[must_use]
pub fn motif_display_label(motif: &RdkitMotifHit) -> String {
    display_label(
        &motif.source_class,
        &motif.label,
        &motif.kingdom,
        &motif.kingdoms,
    )
}

#[cfg(test)]
mod tests {
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
}
