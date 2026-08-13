// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Typography primitives: font sizes, weights, and colors.

use super::super::tokens::*;
use ui::theme::StyleBuilder;

// ============================================================================
// FONT SIZE PRIMITIVES
// ============================================================================

/// Font size: FS_0 (smallest) - uses CSS variable
pub fn font_size_0() -> String {
    StyleBuilder::new().font_size("var(--fs-0)").build()
}

/// Font size: FS_1 - uses CSS variable
pub fn font_size_1() -> String {
    StyleBuilder::new().font_size("var(--fs-1)").build()
}

/// Font size: FS_2 - uses CSS variable
pub fn font_size_2() -> String {
    StyleBuilder::new().font_size("var(--fs-2)").build()
}

/// Font size: FS_3 (body) - uses CSS variable
pub fn font_size_3() -> String {
    StyleBuilder::new().font_size("var(--fs-3)").build()
}

/// Font size: FS_4 (largest) - uses CSS variable
pub fn font_size_4() -> String {
    StyleBuilder::new().font_size("var(--fs-4)").build()
}

/// Font size: FS_BODY - uses CSS variable
pub fn font_size_body() -> String {
    StyleBuilder::new().font_size("var(--fs-body)").build()
}

/// Font size: FS_LABEL - uses CSS variable
pub fn font_size_label() -> String {
    StyleBuilder::new().font_size("var(--fs-label)").build()
}

/// Font size: FS_MICRO - uses CSS variable
pub fn font_size_micro() -> String {
    StyleBuilder::new().font_size("var(--fs-micro)").build()
}

/// Font size: FS_STAT
pub fn font_size_stat() -> String {
    StyleBuilder::new().font_size("var(--fs-stat)").build()
}

// ============================================================================
// FONT WEIGHT PRIMITIVES
// ============================================================================

/// Normal font weight: `font-weight: 400`
pub fn font_weight_normal() -> String {
    StyleBuilder::new().font_weight("400").build()
}

/// Medium font weight: `font-weight: 500`
pub fn font_weight_medium() -> String {
    StyleBuilder::new().font_weight("500").build()
}

/// Semibold font weight: `font-weight: 600`
pub fn font_weight_semibold() -> String {
    StyleBuilder::new().font_weight("600").build()
}

/// Bold font weight: `font-weight: 700`
pub fn font_weight_bold() -> String {
    StyleBuilder::new().font_weight("700").build()
}

// ============================================================================
// TEXT COLOR PRIMITIVES
// ============================================================================

/// Primary text color: `color: var(--text)`
pub fn text_color_primary() -> String {
    StyleBuilder::new().color(COLOR_TEXT).build()
}

/// Secondary text color: `color: var(--text2)`
pub fn text_color_secondary() -> String {
    StyleBuilder::new().color(COLOR_TEXT_SECONDARY).build()
}

/// Muted text color: `color: var(--text3)`
pub fn text_color_muted() -> String {
    StyleBuilder::new().color("var(--text3)").build()
}

/// White text color: `color: #fff`
pub fn text_color_white() -> String {
    StyleBuilder::new().color("#fff").build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_size_0_uses_variable() {
        assert!(font_size_0().contains("--fs-0"));
    }

    #[test]
    fn font_weight_normal_is_set() {
        assert!(font_weight_normal().contains("font-weight"));
        assert!(font_weight_normal().contains("400"));
    }

    #[test]
    fn text_color_primary_uses_variable() {
        assert!(text_color_primary().contains("--text"));
    }
}
