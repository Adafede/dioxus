// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Lotus-specific table cell styling.
//! Extends base cell styles with lotus-specific colors, borders, and formatting.
//!
//! Naming hierarchy: `cell_<type>()` where type describes the cell variant.
//! Examples: `cell_taxon()`, `cell_reference()`, `cell_numeric()`

use super::tokens;
use ui::prelude::*;

// ============================================================================
// TAXON CELL STYLES
// ============================================================================

/// Taxon cell container: extends base cell with green inset border.
/// Naming: cell (WHAT) taxon (VARIANT)
#[allow(dead_code)]
pub fn cell_taxon() -> String {
    StyleBuilder::new()
        .padding(&tokens::cell_padding())
        .border_radius(tokens::BORDER_RADIUS_LG)
        .background_color("color-mix(in srgb, var(--surface) 90%, transparent)")
        .property(
            "box-shadow",
            "inset 3px 0 0 rgb(51 153 102 / 42%), inset 0 0 0 1px var(--results-border)",
        )
        .property("min-width", "0")
        .build()
}

/// Taxon cell primary text: italic font weight 500.
#[allow(dead_code)]
pub fn cell_taxon_primary() -> String {
    StyleBuilder::new()
        .font_weight("500")
        .property("font-style", "italic")
        .build()
}

/// Taxon ID badge: monospace with green background.
#[allow(dead_code)]
pub fn cell_taxon_id() -> String {
    StyleBuilder::new()
        .display("inline-block")
        .font_size("var(--fs-micro)")
        .padding("1px 5px")
        .border_radius("3px")
        .font_weight("600")
        .font_family("var(--mono)")
        .property("line-height", "1.5")
        .property("white-space", "nowrap")
        .background_color("var(--wd-taxon-soft-bg)")
        .color("var(--wd-taxon)")
        .border("1px solid var(--wd-taxon-soft-border)")
        .build()
}

// ============================================================================
// REFERENCE CELL STYLES
// ============================================================================

/// Reference cell container: extends base cell with pink inset border.
/// Naming: cell (WHAT) reference (VARIANT)
#[allow(dead_code)]
pub fn cell_reference() -> String {
    StyleBuilder::new()
        .display("flex")
        .flex_direction("column")
        .gap(tokens::GAP_XS)
        .padding(&tokens::cell_padding())
        .border_radius(tokens::BORDER_RADIUS_LG)
        .background_color("color-mix(in srgb, var(--surface) 90%, transparent)")
        .property(
            "box-shadow",
            "inset 3px 0 0 rgb(185 65 104 / 42%), inset 0 0 0 1px var(--results-border)",
        )
        .property("min-width", "0")
        .build()
}

/// Reference ID badge: inline-block with reference styling.
#[allow(dead_code)]
pub fn cell_reference_id() -> String {
    StyleBuilder::new()
        .display("inline-block")
        .font_size("var(--fs-micro)")
        .padding("1px 5px")
        .border_radius("3px")
        .font_weight("600")
        .font_family("var(--mono)")
        .property("line-height", "1.5")
        .property("white-space", "nowrap")
        .background_color("var(--wd-reference-soft-bg)")
        .color("var(--wd-reference)")
        .border("1px solid var(--wd-reference-soft-border)")
        .build()
}

/// Cell reference badge row: flex wrap with gap.
#[allow(dead_code)]
pub fn cell_reference_badges() -> String {
    StyleBuilder::new()
        .display("flex")
        .property("flex-wrap", "wrap")
        .gap(tokens::GAP_XS)
        .property("margin-top", tokens::GAP_XS)
        .property("min-width", "0")
        .build()
}

// ============================================================================
// GENERIC TABLE CELL STYLES
// ============================================================================

/// Generic table cell: base cell styling for any column.
#[allow(dead_code)]
pub fn cell_default() -> String {
    StyleBuilder::new()
        .padding(&tokens::cell_padding())
        .border_radius(tokens::BORDER_RADIUS_LG)
        .background_color("color-mix(in srgb, var(--surface) 90%, transparent)")
        .border("1px solid var(--results-border)")
        .property("min-width", "0")
        .build()
}

/// Cell primary text link: block display with word break.
#[allow(dead_code)]
pub fn cell_link() -> String {
    StyleBuilder::new()
        .color("var(--text)")
        .property("display", "block")
        .property("line-height", "1.4")
        .property("overflow-wrap", "break-word")
        .property("word-break", "break-word")
        .property("white-space", "normal")
        .build()
}

/// Cell badge row: flex wrap with gap.
#[allow(dead_code)]
pub fn cell_badges() -> String {
    StyleBuilder::new()
        .display("flex")
        .property("flex-wrap", "wrap")
        .gap(tokens::GAP_XS)
        .property("margin-top", tokens::GAP_XS)
        .property("min-width", "0")
        .build()
}

/// N/A placeholder: italic secondary text.
#[allow(dead_code)]
pub fn cell_na() -> String {
    StyleBuilder::new()
        .property("font-style", "italic")
        .color("var(--text2)")
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_taxon_has_inset_shadow() {
        let style = cell_taxon();
        assert!(style.contains("inset"));
    }

    #[test]
    fn cell_taxon_primary_is_italic() {
        let style = cell_taxon_primary();
        assert!(style.contains("italic"));
    }

    #[test]
    fn cell_taxon_id_has_color() {
        let style = cell_taxon_id();
        assert!(style.contains("--wd-taxon"));
    }

    #[test]
    fn cell_reference_is_flex_column() {
        let style = cell_reference();
        assert!(style.contains("flex-direction"));
        assert!(style.contains("column"));
    }

    #[test]
    fn cell_reference_id_has_reference_colors() {
        let style = cell_reference_id();
        assert!(style.contains("--wd-reference"));
    }
}
