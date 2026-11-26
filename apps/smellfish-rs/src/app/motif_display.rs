// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! Motif classification and chip display formatting over [`SourceClass`].

use smellfish_core::model::SourceClass;

pub(super) const fn is_natural_source(source_class: SourceClass) -> bool {
    matches!(source_class, SourceClass::Natural)
}

pub(super) const fn is_synthetic_source(source_class: SourceClass) -> bool {
    matches!(source_class, SourceClass::Synthetic)
}

pub(super) const fn is_unclassified_source(source_class: SourceClass) -> bool {
    matches!(source_class, SourceClass::Unknown)
}

#[must_use]
pub(super) const fn chip_class_for(source_class: SourceClass) -> &'static str {
    match source_class {
        SourceClass::Natural => "chip chip-np",
        SourceClass::Synthetic => "chip chip-synth",
        SourceClass::Unknown => "chip alt",
    }
}

#[must_use]
pub(super) fn display_label(
    source_class: SourceClass,
    label: &str,
    kingdom: &str,
    kingdoms: &[String],
) -> String {
    match source_class {
        SourceClass::Natural => {
            if let [only] = kingdoms {
                format!("{only} \u{00b7} {label}")
            } else if kingdoms.is_empty() {
                format!("{kingdom} \u{00b7} {label}")
            } else {
                format!("{} \u{00b7} {label}", kingdoms.join(" + "))
            }
        }
        SourceClass::Synthetic => format!("synthetic \u{00b7} {label}"),
        SourceClass::Unknown => format!("unclassified \u{00b7} {label}"),
    }
}

#[cfg(test)]
mod tests;
