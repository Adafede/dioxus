// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Lotus-specific search form and control styling.

use ui::prelude::*;

/// Search button state indicator: color-coded status.
pub fn search_button_state() -> String {
    StyleBuilder::new()
        .property("font-weight", "600")
        .property("font-size", "var(--fs-0)")
        .build()
}

/// Radio group container: flex column with gap.
pub fn search_radio_group() -> String {
    StyleBuilder::new()
        .display("flex")
        .flex_direction("column")
        .gap("8px")
        .build()
}

/// Radio label: inline-flex with cursor pointer.
pub fn search_radio_label() -> String {
    StyleBuilder::new()
        .display("inline-flex")
        .align_items("center")
        .gap("8px")
        .cursor("pointer")
        .font_size("var(--fs-0)")
        .build()
}

/// Range input: standard input with consistent styling.
pub fn search_range_input() -> String {
    StyleBuilder::new()
        .property("width", "100%")
        .property("max-width", "120px")
        .background_color("var(--surface)")
        .border("1px solid var(--border)")
        .border_radius("4px")
        .padding("6px 8px")
        .font_size("var(--fs-0)")
        .build()
}

/// Textarea base: search textarea with consistent styling.
pub fn search_textarea() -> String {
    StyleBuilder::new()
        .property("width", "100%")
        .background_color("var(--surface)")
        .border("1px solid var(--border)")
        .border_radius("4px")
        .padding("8px 10px")
        .font_size("var(--fs-0)")
        .property("font-family", "var(--mono)")
        .property("line-height", "1.4")
        .property("resize", "vertical")
        .build()
}

/// Kind pill: inline-block badge for filter kinds.
pub fn search_kind_pill() -> String {
    StyleBuilder::new()
        .display("inline-block")
        .padding("4px 8px")
        .border_radius("12px")
        .background_color("var(--surface)")
        .border("1px solid var(--border)")
        .font_size("var(--fs-0)")
        .font_weight("600")
        .build()
}

/// Threshold section: container for threshold controls.
pub fn search_threshold_section() -> String {
    StyleBuilder::new()
        .display("flex")
        .flex_direction("column")
        .gap("8px")
        .padding("10px 0")
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_radio_group_is_flex_column() {
        let style = search_radio_group();
        assert!(style.contains("flex-direction"));
        assert!(style.contains("column"));
    }

    #[test]
    fn search_textarea_has_monospace() {
        let style = search_textarea();
        assert!(style.contains("--mono"));
    }

    #[test]
    fn search_kind_pill_is_inline_block() {
        let style = search_kind_pill();
        assert!(style.contains("inline-block"));
    }
}
