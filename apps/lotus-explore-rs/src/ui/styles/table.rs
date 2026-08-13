// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Lotus-specific table styling.

use super::tokens;
use ui::prelude::*;

/// Table header cell: bold with bottom border.
pub fn table_header_cell() -> String {
    StyleBuilder::new()
        .font_weight("700")
        .font_size("var(--fs-0)")
        .padding(&format!("{} {}", "10px", tokens::CELL_PADDING_X))
        .border("1px solid var(--border)")
        .border_bottom("2px solid var(--border)")
        .background_color("var(--surface-soft)")
        .build()
}

/// Header label: left-aligned with text overflow handling.
pub fn table_header_label() -> String {
    StyleBuilder::new()
        .font_weight("700")
        .text_align("left")
        .property("line-height", "1.3")
        .build()
}

/// Sort button: transparent button for column sorting.
pub fn table_sort_button() -> String {
    StyleBuilder::new()
        .display("inline-flex")
        .align_items("center")
        .gap(tokens::GAP_XS)
        .cursor("pointer")
        .font_weight("700")
        .padding("0")
        .border("none")
        .build()
}

/// Sort icon: arrow indicator for sort direction.
pub fn table_sort_icon() -> String {
    StyleBuilder::new()
        .display("inline-flex")
        .align_items("center")
        .property("font-size", "0.9em")
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_header_cell_has_bold() {
        let style = table_header_cell();
        assert!(style.contains("font-weight"));
    }

    #[test]
    fn table_sort_button_has_no_border() {
        let style = table_sort_button();
        assert!(style.contains("border"));
        assert!(style.contains("none"));
    }

    #[test]
    fn table_sort_icon_is_inline_flex() {
        let style = table_sort_icon();
        assert!(style.contains("inline-flex"));
    }
}
