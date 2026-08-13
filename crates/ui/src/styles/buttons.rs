// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Shared button styling system for consistent UI across all apps.
//!
//! All buttons maintain a minimum height of 40px for touch accessibility,
//! with padding adjusted to maintain proper aspect ratios and visual balance.
//! Smaller variants (sm, xs) are only for space-constrained contexts.

use crate::theme::StyleBuilder;

// ============================================================================
// CORE BUTTON STYLES (40px min-height baseline)
// ============================================================================

/// Standard button style: inline-flex with border, surface background, and text color.
/// Used in notices, loading, search panel components.
/// Min-height: 40px | Padding: 8px 14px
pub fn button_base_style() -> String {
    StyleBuilder::new()
        .display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .gap("6px")
        .border("1px solid var(--border)")
        .border_radius("4px")
        .property("min-height", "40px")
        .padding("8px 14px")
        .font_size("var(--fs-0)")
        .font_weight("600")
        .cursor("pointer")
        .background_color("var(--surface)")
        .color("var(--text)")
        .box_shadow("var(--shadow-xs)")
        .property(
            "transition",
            "background .15s, border-color .15s, box-shadow .15s, transform .12s ease",
        )
        .build()
}

/// Transparent button variant: used for secondary actions like download status.
/// Min-height: 40px | Padding: 8px 14px
pub fn button_transparent_style() -> String {
    StyleBuilder::new()
        .display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .gap("6px")
        .border("1px solid var(--border)")
        .border_radius("8px")
        .property("min-height", "40px")
        .padding("8px 14px")
        .font_size("var(--fs-0)")
        .font_weight("600")
        .cursor("pointer")
        .background_color("transparent")
        .color("var(--text)")
        .property(
            "transition",
            "border-color .15s, background .15s, box-shadow .15s, transform .12s ease",
        )
        .build()
}

/// Primary button: medium size with primary background, used for search and main actions.
/// Min-height: 40px | Padding: 8px 14px
pub fn button_primary_style() -> String {
    StyleBuilder::new()
        .display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .gap("6px")
        .border("1px solid var(--btn-primary-bg)")
        .border_radius("4px")
        .property("min-height", "40px")
        .padding("8px 14px")
        .font_size("var(--fs-0)")
        .font_weight("600")
        .cursor("pointer")
        .background_color("var(--btn-primary-bg)")
        .color("#fff")
        .box_shadow("var(--shadow-xs)")
        .property(
            "transition",
            "background .15s, border-color .15s, box-shadow .15s, transform .12s ease",
        )
        .build()
}

/// Primary button full width: for block-level actions (Generate QuickStatements, etc).
/// Min-height: 40px (implicit via padding/line-height) | Padding: 8px 14px
pub fn button_primary_block_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .align_items("center")
        .justify_content("center")
        .gap("6px")
        .border("1px solid var(--btn-primary-bg)")
        .border_radius("4px")
        .padding("8px 14px")
        .font_size("var(--fs-0)")
        .font_weight("600")
        .cursor("pointer")
        .background_color("var(--btn-primary-bg)")
        .color("#fff")
        .property("width", "100%")
        .build()
}

/// Filters toggle button: mobile-only button for showing/hiding filters.
/// Display controlled by CSS media queries (filters-toggle class).
/// Rust styles handle appearance (colors, sizing, etc) — display is inline-flex like search button.
/// Min-height: 40px | Padding: 8px 14px
pub fn button_filters_toggle_style() -> String {
    StyleBuilder::new()
        .display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .gap("6px")
        .border("1px solid var(--border)")
        .border_radius("4px")
        .property("min-height", "40px")
        .padding("8px 14px")
        .font_size("var(--fs-0)")
        .font_weight("600")
        .cursor("pointer")
        .background_color("var(--btn-primary-bg)")
        .color("#fff")
        .box_shadow("var(--shadow-xs)")
        .property(
            "transition",
            "background .15s, box-shadow .15s, transform .12s ease",
        )
        .build()
}

/// Copy button: small secondary button for copying content to clipboard.
/// Responsive sizing: uses clamp() to scale with font-size and viewport.
/// Min-height: 40px | Padding: responsive via clamp()
pub fn button_copy_style() -> String {
    StyleBuilder::new()
        .display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .gap("6px")
        .property("margin-left", "6px")
        .font_family("var(--sans), system-ui, sans-serif")
        .font_weight("500")
        .property("letter-spacing", ".02em")
        .color("var(--text2)")
        .background_color("var(--surface)")
        .border("1px solid var(--border)")
        .border_radius("4px")
        .cursor("pointer")
        .property("height", "40px")
        .property("font-size", "0.75rem")
        .padding("8px 14px")
        .property("line-height", "1")
        .property(
            "transition",
            "color .15s, background .15s, border-color .15s",
        )
        .property("vertical-align", "middle")
        .build()
}

// ============================================================================
// COMPACT BUTTON VARIANTS (34px min-height)
// ============================================================================

/// Primary button small: compact size for curation actions (Add Row, etc).
/// Min-height: 34px | Padding: 6px 12px
pub fn button_primary_sm_style() -> String {
    StyleBuilder::new()
        .display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .gap("6px")
        .border("1px solid var(--btn-primary-bg)")
        .border_radius("4px")
        .padding("6px 12px")
        .font_size("var(--fs-0)")
        .font_weight("600")
        .cursor("pointer")
        .background_color("var(--btn-primary-bg)")
        .color("#fff")
        .property("min-height", "34px")
        .build()
}

/// Base button with standard surface background (used for secondary/general actions).
/// Min-height: 34px | Padding: 5px 10px
pub fn button_sm_style() -> String {
    StyleBuilder::new()
        .display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .gap("6px")
        .border("1px solid var(--border)")
        .border_radius("4px")
        .property("min-height", "34px")
        .padding("5px 10px")
        .font_size("var(--fs-0)")
        .font_weight("600")
        .cursor("pointer")
        .background_color("var(--surface)")
        .color("var(--text)")
        .box_shadow("var(--shadow-xs)")
        .property(
            "transition",
            "background .15s, border-color .15s, box-shadow .15s, transform .12s ease",
        )
        .build()
}

// ============================================================================
// EXTRA-SMALL BUTTON VARIANT (30px min-height)
// ============================================================================

/// Extra-small button for compact UI elements (delete buttons in tables, etc).
/// Min-height: 30px | Padding: 2px 8px
pub fn button_xs_style() -> String {
    StyleBuilder::new()
        .display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .gap("6px")
        .border("1px solid var(--border)")
        .border_radius("4px")
        .property("min-height", "30px")
        .padding("2px 8px")
        .font_size("var(--fs-label)")
        .font_weight("600")
        .cursor("pointer")
        .background_color("var(--surface)")
        .color("var(--text)")
        .property("line-height", "1.2")
        .box_shadow("var(--shadow-xs)")
        .property(
            "transition",
            "background .15s, border-color .15s, box-shadow .15s, transform .12s ease",
        )
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_base_has_40px_height() {
        let style = button_base_style();
        assert!(style.contains("min-height") && style.contains("40px"));
    }

    #[test]
    fn button_primary_has_40px_height() {
        let style = button_primary_style();
        assert!(style.contains("min-height") && style.contains("40px"));
    }

    #[test]
    fn button_copy_has_40px_height() {
        let style = button_copy_style();
        assert!(style.contains("min-height") && style.contains("40px"));
        assert!(style.contains("padding"));
    }

    #[test]
    fn button_sm_has_34px_height() {
        let style = button_sm_style();
        assert!(style.contains("min-height") && style.contains("34px"));
    }

    #[test]
    fn button_xs_has_30px_height() {
        let style = button_xs_style();
        assert!(style.contains("min-height") && style.contains("30px"));
    }

    #[test]
    fn button_copy_has_padding() {
        let style = button_copy_style();
        assert!(style.contains("padding"));
    }
}
