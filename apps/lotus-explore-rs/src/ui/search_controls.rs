// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Search form controls styling: radio groups, range inputs, textareas, etc.
//! Lotus-specific search panel customizations.

use ui::prelude::*;

/// Radio group fieldset: flex wrap, no border/padding/margin.
pub fn lotus_radio_group_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .flex_wrap("wrap")
        .gap("14px")
        .property("border", "0")
        .property("padding", "0")
        .property("margin", "0")
        .build()
}

/// Radio label: flex with gap, cursor pointer, secondary text color.
pub fn lotus_radio_label_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .align_items("center")
        .gap("6px")
        .font_size(FS_0)
        .cursor("pointer")
        .color(TEXT2)
        .build()
}

/// Range input slider: full width with accent-color.
pub fn lotus_range_input_style() -> String {
    StyleBuilder::new()
        .property("width", "100%")
        .property("accent-color", ACCENT)
        .property("margin-top", "4px")
        .build()
}

/// Textarea: surface background, border, full width, no resize.
pub fn lotus_textarea_base_style() -> String {
    StyleBuilder::new()
        .background_color(SURFACE)
        .border(BORDER_DEFAULT)
        .border_radius("4px")
        .color(TEXT)
        .padding("9px 11px")
        .font_size(FS_UI)
        .property("width", "100%")
        .property("max-width", "100%")
        .property("resize", "none")
        .font_family(FONT_SANS)
        .property("transition", "border-color .15s")
        .build()
}

/// Threshold section: column with left border and top margin.
pub fn lotus_threshold_section_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .flex_direction("column")
        .gap("5px")
        .padding("10px")
        .property("border-left", BORDER_DEFAULT)
        .property("margin-top", "4px")
        .build()
}

/// Kind pill badge: inline-block with background color and uppercase text.
pub fn lotus_kind_pill_style(kind: &str) -> String {
    let background = match kind {
        "smiles" => ACCENT2,
        "mol2000" => "#c97a2b",
        "mol3000" => "#2b8f57",
        _ => TEXT3,
    };
    StyleBuilder::new()
        .display("inline-block")
        .padding("1px 7px")
        .border_radius("999px")
        .font_size(FS_MICRO)
        .font_weight("700")
        .property("letter-spacing", "1px")
        .property("text-transform", "uppercase")
        .property("margin-right", "6px")
        .color(TEXT)
        .background_color(background)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lotus_radio_group_is_flex_wrap() {
        let style = lotus_radio_group_style();
        assert!(style.contains("flex") && style.contains("wrap"));
    }

    #[test]
    fn lotus_textarea_has_full_width() {
        let style = lotus_textarea_base_style();
        assert!(style.contains("width") && style.contains("100%"));
    }

    #[test]
    fn lotus_kind_pill_returns_nonempty() {
        assert!(!lotus_kind_pill_style("smiles").is_empty());
        assert!(!lotus_kind_pill_style("mol2000").is_empty());
    }
}
