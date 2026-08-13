// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Atomic, single-concern style primitives for composition.
//!
//! This module provides fine-grained, reusable CSS building blocks that can be
//! composed together to create complex component styles. Each primitive handles
//! a single concern (layout, spacing, typography, etc.) and returns a CSS string.
//!
//! Primitives are designed to be combined using `format!()`:
//!
//! ```ignore
//! pub fn button_primary() -> String {
//!     format!("{}; {}; {}",
//!         flex_center(),
//!         padding_md(),
//!         cursor_pointer()
//!     )
//! }
//! ```

use crate::theme::StyleBuilder;

// ============================================================================
// LAYOUT PRIMITIVES
// ============================================================================

/// Flex row container: `display: flex; flex-direction: row`
pub fn flex_row() -> String {
    StyleBuilder::new()
        .display("flex")
        .flex_direction("row")
        .build()
}

/// Flex column container: `display: flex; flex-direction: column`
pub fn flex_column() -> String {
    StyleBuilder::new()
        .display("flex")
        .flex_direction("column")
        .build()
}

/// Flex row with centered items: flex row + align center + justify center
pub fn flex_center() -> String {
    StyleBuilder::new()
        .display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .build()
}

/// Flex column with centered items: flex column + align center + justify center
pub fn flex_center_column() -> String {
    StyleBuilder::new()
        .display("flex")
        .flex_direction("column")
        .align_items("center")
        .justify_content("center")
        .build()
}

/// CSS grid container: `display: grid`
pub fn grid() -> String {
    StyleBuilder::new().display("grid").build()
}

/// Absolute positioning base: `position: absolute`
pub fn absolute() -> String {
    StyleBuilder::new().property("position", "absolute").build()
}

/// Relative positioning base: `position: relative`
pub fn relative() -> String {
    StyleBuilder::new().property("position", "relative").build()
}

/// Sticky positioning base: `position: sticky`
pub fn sticky() -> String {
    StyleBuilder::new().property("position", "sticky").build()
}

/// Full size: `width: 100%; height: 100%`
pub fn full_size() -> String {
    StyleBuilder::new()
        .property("width", "100%")
        .property("height", "100%")
        .build()
}

/// Full width: `width: 100%`
pub fn full_width() -> String {
    StyleBuilder::new().property("width", "100%").build()
}

/// Full height: `height: 100%`
pub fn full_height() -> String {
    StyleBuilder::new().property("height", "100%").build()
}

// ============================================================================
// SPACING PRIMITIVES
// ============================================================================

/// No padding: `padding: 0`
pub fn padding_none() -> String {
    StyleBuilder::new().padding("0").build()
}

/// Extra-small padding: `padding: 4px`
pub fn padding_xs() -> String {
    StyleBuilder::new().padding("4px").build()
}

/// Small padding: `padding: 8px`
pub fn padding_sm() -> String {
    StyleBuilder::new().padding("8px").build()
}

/// Medium padding: `padding: 12px`
pub fn padding_md() -> String {
    StyleBuilder::new().padding("12px").build()
}

/// Large padding: `padding: 16px`
pub fn padding_lg() -> String {
    StyleBuilder::new().padding("16px").build()
}

/// Button padding: `padding: 8px 14px`
pub fn padding_button() -> String {
    StyleBuilder::new().padding("8px 14px").build()
}

/// Input padding: `padding: 9px 11px`
pub fn padding_input() -> String {
    StyleBuilder::new().padding("9px 11px").build()
}

/// No margin: `margin: 0`
pub fn margin_none() -> String {
    StyleBuilder::new().margin("0").build()
}

/// Extra-small margin: `margin: 4px`
pub fn margin_xs() -> String {
    StyleBuilder::new().margin("4px").build()
}

/// Small margin: `margin: 8px`
pub fn margin_sm() -> String {
    StyleBuilder::new().margin("8px").build()
}

/// Medium margin: `margin: 12px`
pub fn margin_md() -> String {
    StyleBuilder::new().margin("12px").build()
}

/// Extra-small gap: `gap: 4px`
pub fn gap_xs() -> String {
    StyleBuilder::new().gap("4px").build()
}

/// Small gap: `gap: 6px`
pub fn gap_sm() -> String {
    StyleBuilder::new().gap("6px").build()
}

/// Medium gap: `gap: 8px`
pub fn gap_md() -> String {
    StyleBuilder::new().gap("8px").build()
}

// ============================================================================
// TYPOGRAPHY PRIMITIVES
// ============================================================================

/// Extra-small font size: `font-size: var(--fs-0)`
pub fn font_size_xs() -> String {
    StyleBuilder::new().font_size("var(--fs-0)").build()
}

/// Small font size: `font-size: var(--fs-1)`
pub fn font_size_sm() -> String {
    StyleBuilder::new().font_size("var(--fs-1)").build()
}

/// Medium font size: `font-size: var(--fs-2)`
pub fn font_size_md() -> String {
    StyleBuilder::new().font_size("var(--fs-2)").build()
}

/// Large font size: `font-size: var(--fs-3)`
pub fn font_size_lg() -> String {
    StyleBuilder::new().font_size("var(--fs-3)").build()
}

/// Extra-large font size: `font-size: var(--fs-4)`
pub fn font_size_xl() -> String {
    StyleBuilder::new().font_size("var(--fs-4)").build()
}

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

/// Primary text color: `color: var(--text)`
pub fn text_color_primary() -> String {
    StyleBuilder::new().color("var(--text)").build()
}

/// Secondary text color: `color: var(--text2)`
pub fn text_color_secondary() -> String {
    StyleBuilder::new().color("var(--text2)").build()
}

/// Muted text color: `color: var(--text3)`
pub fn text_color_muted() -> String {
    StyleBuilder::new().color("var(--text3)").build()
}

/// White text color: `color: #fff`
pub fn text_color_white() -> String {
    StyleBuilder::new().color("#fff").build()
}

// ============================================================================
// VISUAL PRIMITIVES
// ============================================================================

/// Surface background: `background-color: var(--surface)`
pub fn bg_surface() -> String {
    StyleBuilder::new()
        .background_color("var(--surface)")
        .build()
}

/// Panel background: `background-color: var(--panel-bg)`
pub fn bg_panel() -> String {
    StyleBuilder::new()
        .background_color("var(--panel-bg)")
        .build()
}

/// Primary button background: `background-color: var(--btn-primary-bg)`
pub fn bg_primary() -> String {
    StyleBuilder::new()
        .background_color("var(--btn-primary-bg)")
        .build()
}

/// Transparent background: `background-color: transparent`
pub fn bg_transparent() -> String {
    StyleBuilder::new().background_color("transparent").build()
}

/// Default border: `border: 1px solid var(--border)`
pub fn border_default() -> String {
    StyleBuilder::new()
        .border("1px solid var(--border)")
        .build()
}

/// No border: `border: none`
pub fn border_none() -> String {
    StyleBuilder::new().border("none").build()
}

/// Small border radius: `border-radius: 4px`
pub fn border_radius_sm() -> String {
    StyleBuilder::new().border_radius("4px").build()
}

/// Medium border radius: `border-radius: 6px`
pub fn border_radius_md() -> String {
    StyleBuilder::new().border_radius("6px").build()
}

/// Large border radius: `border-radius: 10px`
pub fn border_radius_lg() -> String {
    StyleBuilder::new().border_radius("10px").build()
}

/// Extra-small shadow: `box-shadow: var(--shadow-xs)`
pub fn shadow_xs() -> String {
    StyleBuilder::new().box_shadow("var(--shadow-xs)").build()
}

/// Small shadow: `box-shadow: var(--shadow-sm)`
pub fn shadow_sm() -> String {
    StyleBuilder::new().box_shadow("var(--shadow-sm)").build()
}

/// Medium shadow: `box-shadow: var(--shadow-md)`
pub fn shadow_md() -> String {
    StyleBuilder::new().box_shadow("var(--shadow-md)").build()
}

/// No shadow: `box-shadow: none`
pub fn shadow_none() -> String {
    StyleBuilder::new().box_shadow("none").build()
}

// ============================================================================
// INTERACTION PRIMITIVES
// ============================================================================

/// Pointer cursor: `cursor: pointer`
pub fn cursor_pointer() -> String {
    StyleBuilder::new().cursor("pointer").build()
}

/// Default cursor: `cursor: auto`
pub fn cursor_default() -> String {
    StyleBuilder::new().cursor("auto").build()
}

/// Fast transition: `transition: ... 0.1s`
pub fn transition_fast() -> String {
    StyleBuilder::new()
        .property(
            "transition",
            "background .1s, border-color .1s, box-shadow .1s, transform .1s ease",
        )
        .build()
}

/// Standard transition: `transition: ... 0.15s`
pub fn transition_standard() -> String {
    StyleBuilder::new()
        .property(
            "transition",
            "background .15s, border-color .15s, box-shadow .15s, transform .12s ease",
        )
        .build()
}

/// Slow transition: `transition: ... 0.3s`
pub fn transition_slow() -> String {
    StyleBuilder::new()
        .property(
            "transition",
            "background .3s, border-color .3s, box-shadow .3s, transform .3s ease",
        )
        .build()
}

/// Focus outline: standard focus styles
pub fn focus_outline() -> String {
    StyleBuilder::new()
        .property("outline", "2px solid var(--accent)")
        .property("outline-offset", "2px")
        .build()
}

// ============================================================================
// COMPOSITION HELPERS
// ============================================================================

/// Flex row with center alignment and gap: flex center row + gap
pub fn flex_center_row_with_gap() -> String {
    format!("{}; {}", flex_center(), gap_sm())
}

/// Flex column with gap
pub fn flex_column_with_gap() -> String {
    format!("{}; {}", flex_column(), gap_sm())
}

/// Button base: flex center + border + padding + cursor
pub fn button_base() -> String {
    format!(
        "{}; {}; {}; {}; {}",
        flex_center(),
        padding_button(),
        border_default(),
        border_radius_sm(),
        cursor_pointer()
    )
}

/// Input base: surface bg + border + padding + text color
pub fn input_base() -> String {
    format!(
        "{}; {}; {}; {}; {}",
        bg_surface(),
        border_default(),
        border_radius_sm(),
        padding_input(),
        text_color_primary()
    )
}

/// Panel base: border + panel bg + padding + shadow
pub fn panel_base() -> String {
    format!(
        "{}; {}; {}; {}",
        border_default(),
        bg_panel(),
        padding_md(),
        shadow_sm()
    )
}

/// Label base: uppercase + bold + primary color
pub fn label_base() -> String {
    format!(
        "{}; {}; {}",
        font_weight_bold(),
        font_size_xs(),
        text_color_primary()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flex_center_contains_display() {
        assert!(flex_center().contains("display"));
        assert!(flex_center().contains("flex"));
    }

    #[test]
    fn flex_center_contains_align_items() {
        assert!(flex_center().contains("align-items"));
        assert!(flex_center().contains("center"));
    }

    #[test]
    fn padding_xs_contains_4px() {
        assert!(padding_xs().contains("padding"));
        assert!(padding_xs().contains("4px"));
    }

    #[test]
    fn font_size_xs_uses_variable() {
        assert!(font_size_xs().contains("--fs-0"));
    }

    #[test]
    fn bg_surface_uses_variable() {
        assert!(bg_surface().contains("--surface"));
    }

    #[test]
    fn border_default_contains_border() {
        assert!(border_default().contains("border"));
        assert!(border_default().contains("1px"));
    }

    #[test]
    fn cursor_pointer_is_set() {
        assert!(cursor_pointer().contains("cursor"));
        assert!(cursor_pointer().contains("pointer"));
    }

    #[test]
    fn button_base_composes_primitives() {
        let style = button_base();
        assert!(style.contains("display"));
        assert!(style.contains("flex"));
        assert!(style.contains("border"));
        assert!(style.contains("cursor"));
    }

    #[test]
    fn input_base_composes_primitives() {
        let style = input_base();
        assert!(style.contains("border"));
        assert!(style.contains("padding"));
        assert!(style.contains("background"));
    }

    #[test]
    fn panel_base_composes_primitives() {
        let style = panel_base();
        assert!(style.contains("border"));
        assert!(style.contains("background"));
        assert!(style.contains("padding"));
    }
}
