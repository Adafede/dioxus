// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Shared form element styling system for inputs, labels, textareas, etc.
//! Provides consistent styling across all apps for maximum reuse.

use crate::theme::StyleBuilder;

/// Base input field: background, border, text color, padding, sizing.
pub fn input_base_style() -> String {
    StyleBuilder::new()
        .background_color("var(--surface)")
        .border("1px solid var(--border)")
        .border_radius("4px")
        .color("var(--text)")
        .padding("9px 11px")
        .font_size("var(--fs-ui)")
        .property("width", "100%")
        .font_family("var(--sans)")
        .property("transition", "border-color .15s")
        .build()
}

/// Base form label: uppercase with specific font size and letter spacing.
pub fn label_base_style() -> String {
    StyleBuilder::new()
        .font_size("var(--fs-0)")
        .font_weight("700")
        .color("var(--critical-text)")
        .property("text-transform", "uppercase")
        .property("letter-spacing", "0.08em")
        .build()
}

/// Small label: normal case, regular text color.
pub fn label_small_style() -> String {
    StyleBuilder::new()
        .font_size("var(--fs-0)")
        .font_weight("700")
        .color("var(--text)")
        .property("text-transform", "none")
        .property("letter-spacing", "0")
        .build()
}

/// Hint text: smaller, secondary color.
pub fn hint_text_style() -> String {
    StyleBuilder::new()
        .font_size("var(--fs-0)")
        .color("var(--text2)")
        .build()
}

/// Form section container: flex column with border and soft background.
pub fn form_section_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .flex_direction("column")
        .gap("5px")
        .padding("10px 12px")
        .border("1px solid var(--panel-border)")
        .border_radius("12px")
        .background_color("var(--panel-bg-soft)")
        .build()
}

/// Range inputs container: flex row with items aligned to bottom.
pub fn range_inputs_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .align_items("flex-end")
        .gap("8px")
        .build()
}

/// Range pair container: flex column for min/max value pair.
pub fn range_pair_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .flex_direction("column")
        .gap("3px")
        .build()
}

/// Share input field: flex input with readonly styling.
pub fn share_input_style() -> String {
    StyleBuilder::new()
        .property("flex", "1 1 200px")
        .property("min-width", "min(200px, 100%)")
        .background_color("var(--surface)")
        .border("1px solid var(--border)")
        .border_radius("var(--radius-sm)")
        .color("var(--text)")
        .padding("4px 8px")
        .font_size("var(--fs-0)")
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_base_has_padding() {
        let style = input_base_style();
        assert!(style.contains("padding"));
    }

    #[test]
    fn label_base_is_uppercase() {
        let style = label_base_style();
        assert!(style.contains("uppercase"));
    }

    #[test]
    fn form_section_is_flex_column() {
        let style = form_section_style();
        assert!(style.contains("flex") && style.contains("column"));
    }
}
