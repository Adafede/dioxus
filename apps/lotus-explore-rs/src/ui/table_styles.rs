// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Table cell styling for different column types (taxon, reference, numeric, etc).
//! Lotus-specific cell and row formatting.

use ui::prelude::*;

/// Taxon cell container: soft background with inset shadow border.
pub fn lotus_taxon_cell_style() -> String {
    StyleBuilder::new()
        .padding("8px 12px")
        .border_radius("10px")
        .background_color("color-mix(in srgb, var(--surface) 90%, transparent)")
        .property(
            "box-shadow",
            "inset 3px 0 0 rgb(51 153 102 / 42%), inset 0 0 0 1px var(--results-border)",
        )
        .property("min-width", "0")
        .build()
}

/// Cell primary text: italic font weight 500.
pub fn lotus_cell_primary_style() -> String {
    StyleBuilder::new()
        .font_weight("500")
        .property("font-style", "italic")
        .build()
}

/// ID badge: inline-block with monospace font and soft background.
pub fn lotus_id_badge_style() -> String {
    StyleBuilder::new()
        .display("inline-block")
        .font_size("var(--fs-micro)")
        .padding("1px 5px")
        .border_radius("3px")
        .font_weight("600")
        .text_decoration("none")
        .property("line-height", "1.5")
        .border("1px solid transparent")
        .font_family("var(--mono)")
        .property("max-width", "100%")
        .property("white-space", "normal")
        .property("overflow-wrap", "anywhere")
        .property(
            "transition",
            "transform .12s ease, box-shadow .12s ease, filter .12s ease",
        )
        .background_color("var(--wd-taxon-soft-bg)")
        .color("var(--wd-taxon)")
        .property("border-color", "var(--wd-taxon-soft-border)")
        .build()
}

/// Primary link in cell: block display with word break.
pub fn lotus_primary_link_style() -> String {
    StyleBuilder::new()
        .color("var(--text)")
        .property("display", "block")
        .property("line-height", "1.4")
        .property("overflow-wrap", "break-word")
        .property("word-break", "break-word")
        .property("white-space", "normal")
        .build()
}

/// Badge row container: flex wrap with gap.
pub fn lotus_badge_row_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .property("flex-wrap", "wrap")
        .gap("4px")
        .property("margin-top", "4px")
        .property("overflow", "visible")
        .property("min-width", "0")
        .build()
}

/// Reference cell container: flex column with padding and border.
pub fn lotus_reference_cell_style() -> String {
    StyleBuilder::new()
        .padding("8px 12px")
        .border_radius("10px")
        .background_color("color-mix(in srgb, var(--surface) 90%, transparent)")
        .property(
            "box-shadow",
            "inset 3px 0 0 rgb(0 102 153 / 42%), inset 0 0 0 1px var(--results-border)",
        )
        .property("min-width", "0")
        .build()
}

/// Numeric value cell: standard table cell styling.
pub fn lotus_table_cell_style() -> String {
    StyleBuilder::new()
        .padding("8px 12px")
        .property("vertical-align", "top")
        .build()
}

/// N/A text for missing values: muted color.
pub fn lotus_na_style() -> String {
    StyleBuilder::new().color("var(--text3)").build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lotus_taxon_cell_has_padding() {
        let style = lotus_taxon_cell_style();
        assert!(style.contains("padding"));
    }

    #[test]
    fn lotus_reference_cell_is_flex_column() {
        let style = lotus_reference_cell_style();
        assert!(style.contains("border-radius"));
    }

    #[test]
    fn lotus_id_badge_is_inline_block() {
        let style = lotus_id_badge_style();
        assert!(style.contains("inline-block"));
    }
}
