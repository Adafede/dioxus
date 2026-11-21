// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! Motif classification + display formatting.
//!
//! Shared `&str`-based helpers serve both `MotifSummary` and `RdkitMotifHit`:
//! both types expose a `source_class: String` field, so the classification logic
//! collapses to shared functions that take `&str`.

use crate::model::normalized_source_class;

/// Returns `true` when the source class indicates a natural-product origin.
#[must_use]
pub(super) fn is_natural_source(source_class: &str) -> bool {
    normalized_source_class(source_class) == "natural"
}

/// Returns `true` when the source class indicates a synthetic origin.
#[must_use]
pub(super) fn is_synthetic_source(source_class: &str) -> bool {
    normalized_source_class(source_class) == "synthetic"
}

/// Returns `true` when the source class is unclassified (neither natural nor synthetic).
#[must_use]
pub(super) fn is_unclassified_source(source_class: &str) -> bool {
    normalized_source_class(source_class) == "unknown"
}

/// Returns `"chip chip-np"` for natural sources, `"chip alt"` otherwise.
#[must_use]
pub(super) fn chip_class_for(source_class: &str) -> &'static str {
    if is_natural_source(source_class) {
        "chip chip-np"
    } else {
        "chip alt"
    }
}

/// Build the display label for a motif, including kingdom attribution
/// for natural sources.
#[must_use]
pub(super) fn display_label(
    source_class: &str,
    label: &str,
    kingdom: &str,
    kingdoms: &[String],
) -> String {
    if is_natural_source(source_class) {
        if let [only] = kingdoms {
            format!("{only} \u{00b7} {label}")
        } else if kingdoms.is_empty() {
            format!("{kingdom} \u{00b7} {label}")
        } else {
            format!("{} \u{00b7} {label}", kingdoms.join(" + "))
        }
    } else if is_synthetic_source(source_class) {
        format!("synthetic \u{00b7} {label}")
    } else {
        format!("unclassified \u{00b7} {label}")
    }
}

#[cfg(test)]
mod tests;
