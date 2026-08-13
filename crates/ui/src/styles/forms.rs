// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Shared form element styling system for inputs, labels, textareas, etc.
//! Provides consistent styling across all apps for maximum reuse.

use super::primitives::*;
use crate::theme::StyleBuilder;

/// Base input field: background, border, text color, padding, sizing.
pub fn input_base_style() -> String {
    format!(
        "{}; {}; {}; {}; {}",
        bg_surface(),
        border_default(),
        border_radius_sm(),
        padding_input(),
        StyleBuilder::new()
            .color("var(--text)")
            .font_size("var(--fs-ui)")
            .property("width", "100%")
            .font_family("var(--sans)")
            .property("transition", "border-color .15s")
            .build()
    )
}

/// Base form label: uppercase with specific font size and letter spacing.
pub fn label_base_style() -> String {
    format!(
        "{}; {}; {}",
        font_size_xs(),
        font_weight_bold(),
        StyleBuilder::new()
            .color("var(--critical-text)")
            .property("text-transform", "uppercase")
            .property("letter-spacing", "0.08em")
            .build()
    )
}

/// Small label: normal case, regular text color.
pub fn label_small_style() -> String {
    format!(
        "{}; {}; {}",
        font_size_xs(),
        font_weight_bold(),
        text_color_primary()
    )
}

/// Hint text: smaller, secondary color.
pub fn hint_text_style() -> String {
    format!("{}; {}", font_size_xs(), text_color_secondary())
}

/// Form section container: flex column with border and soft background.
pub fn form_section_style() -> String {
    format!(
        "{}; {}",
        flex_column(),
        StyleBuilder::new()
            .gap("5px")
            .padding("10px 12px")
            .border("1px solid var(--panel-border)")
            .border_radius("12px")
            .background_color("var(--panel-bg-soft)")
            .build()
    )
}

/// Range inputs container: flex row with items aligned to bottom.
pub fn range_inputs_style() -> String {
    format!(
        "{}; {}",
        flex_row(),
        StyleBuilder::new()
            .align_items("flex-end")
            .gap("8px")
            .build()
    )
}

/// Range pair container: flex column for min/max value pair.
pub fn range_pair_style() -> String {
    format!("{}; {}", flex_column(), gap_xs())
}

/// Share input field: flex input with readonly styling.
pub fn share_input_style() -> String {
    format!(
        "{}; {}; {}",
        bg_surface(),
        border_default(),
        StyleBuilder::new()
            .property("flex", "1 1 200px")
            .property("min-width", "min(200px, 100%)")
            .border_radius("var(--radius-sm)")
            .color("var(--text)")
            .padding("4px 8px")
            .font_size("var(--fs-0)")
            .build()
    )
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
