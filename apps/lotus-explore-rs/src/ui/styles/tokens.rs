// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Centralized design tokens for lotus-explore-rs.
//!
//! All dimension, spacing, and sizing constants are defined here as a single
//! source of truth. Changes to these values cascade automatically to all
//! components that reference them.
//!
//! ## Organization
//! - **Spacing**: Fundamental spacing scale (XS → XL)
//! - **Heights**: Row heights for buttons, inputs, cells (with semantic naming)
//! - **Padding**: Vertical/horizontal padding for buttons, cells, and form elements
//! - **Responsive Values**: CSS clamp() values for fluid scaling
//! - **Table/Grid**: Table-specific dimensions

#![allow(dead_code)]
//! - **Borders & Radius**: Border widths and border-radius values
//! - **Colors**: CSS variable references to design system colors

// ============================================================================
// SPACING SCALE
// ============================================================================

/// Extra-small spacing: 4px
pub const SPACING_XS: &str = "4px";

/// Small spacing: 8px
pub const SPACING_SM: &str = "8px";

/// Medium spacing: 12px
pub const SPACING_MD: &str = "12px";

/// Large spacing: 16px
pub const SPACING_LG: &str = "16px";

/// Extra-large spacing: 18px
pub const SPACING_XL: &str = "18px";

// ============================================================================
// ROW HEIGHTS (Semantic sizes for buttons, inputs, cells)
// ============================================================================

/// Small row height: 32px (used by stat rows, compact cells)
pub const ROW_HEIGHT_SM: &str = "32px";

/// Medium row height: 40px (used by buttons, form inputs, normal cells)
pub const ROW_HEIGHT_MD: &str = "40px";

/// Large row height: 48px (used by expanded rows, large buttons)
pub const ROW_HEIGHT_LG: &str = "48px";

/// Extra-large row height: 56px (used by table headers)
pub const ROW_HEIGHT_XL: &str = "56px";

// ============================================================================
// BUTTON HEIGHTS & PADDING
// ============================================================================

/// Minimum height for primary buttons: 40px
pub const BUTTON_MIN_HEIGHT: &str = ROW_HEIGHT_MD;

/// Minimum height for small buttons: 34px
pub const BUTTON_MIN_HEIGHT_SM: &str = "34px";

/// Minimum height for extra-small buttons: 30px
pub const BUTTON_MIN_HEIGHT_XS: &str = "30px";

/// Vertical padding for buttons: 8px (top and bottom)
pub const BUTTON_PADDING_Y: &str = "8px";

/// Horizontal padding for buttons: 14px (left and right)
pub const BUTTON_PADDING_X: &str = "14px";

// ============================================================================
// INPUT HEIGHTS & PADDING
// ============================================================================

/// Minimum height for form inputs: 40px
pub const INPUT_MIN_HEIGHT: &str = ROW_HEIGHT_MD;

/// Vertical padding for form inputs: 8px
pub const INPUT_PADDING_Y: &str = "8px";

/// Horizontal padding for form inputs: 12px
pub const INPUT_PADDING_X: &str = "12px";

// ============================================================================
// COPY BUTTON HEIGHT & PADDING
// ============================================================================

/// Minimum height for copy button: 40px
pub const COPY_BUTTON_MIN_HEIGHT: &str = ROW_HEIGHT_MD;

/// Vertical padding for copy button (responsive)
pub const COPY_BUTTON_PADDING_Y: &str = "clamp(6px, 4px + 1vw, 12px)";

/// Horizontal padding for copy button (responsive)
pub const COPY_BUTTON_PADDING_X: &str = "clamp(10px, 8px + 1vw, 16px)";

// ============================================================================
// CELL PADDING
// ============================================================================

/// Vertical padding for table cells: 8px
pub const CELL_PADDING_Y: &str = "8px";

/// Horizontal padding for table cells: 12px
pub const CELL_PADDING_X: &str = "12px";

/// Combined cell padding (vertical and horizontal)
pub fn cell_padding() -> String {
    format!("{} {}", CELL_PADDING_Y, CELL_PADDING_X)
}

// ============================================================================
// RESPONSIVE FONT SIZES (fluid scaling)
// ============================================================================

/// Responsive micro font size for badges and small text
pub const FONT_SIZE_RESPONSIVE_MICRO: &str = "clamp(0.65rem, 0.63rem + 0.1vw, 0.7rem)";

/// Responsive small font size for compact text
pub const FONT_SIZE_RESPONSIVE_SMALL: &str = "clamp(0.7rem, 0.68rem + 0.1vw, 0.8rem)";

/// Responsive base font size for body text
pub const FONT_SIZE_RESPONSIVE_BASE: &str = "clamp(0.75rem, 0.73rem + 0.15vw, 0.85rem)";

/// Responsive button font size
pub const FONT_SIZE_RESPONSIVE_BUTTON: &str = "clamp(0.75rem, 0.7rem + 0.5vw, 0.95rem)";

// ============================================================================
// RESPONSIVE PADDING (fluid scaling)
// ============================================================================

/// Responsive vertical padding that scales with viewport
pub const PADDING_RESPONSIVE_Y: &str = "clamp(6px, 4px + 1vw, 12px)";

/// Responsive horizontal padding that scales with viewport
pub const PADDING_RESPONSIVE_X: &str = "clamp(10px, 8px + 1vw, 16px)";

// ============================================================================
// TABLE & GRID DIMENSIONS
// ============================================================================

/// Standard stat row height: 40px
pub const STAT_ROW_HEIGHT: &str = ROW_HEIGHT_MD;

/// Minimum width for table cells
pub const CELL_MIN_WIDTH: &str = "120px";

/// Maximum width for table cells
pub const CELL_MAX_WIDTH: &str = "300px";

/// Gap between table rows
pub const TABLE_ROW_GAP: &str = "0";

/// Gap between table columns
pub const TABLE_COLUMN_GAP: &str = "0";

// ============================================================================
// BORDERS & RADIUS
// ============================================================================

/// Standard border width: 1px
pub const BORDER_WIDTH: &str = "1px";

/// Small border-radius: 4px
pub const BORDER_RADIUS_SM: &str = "4px";

/// Medium border-radius: 6px
pub const BORDER_RADIUS_MD: &str = "6px";

/// Large border-radius: 10px (used by cell containers)
pub const BORDER_RADIUS_LG: &str = "10px";

// ============================================================================
// GAPS & MARGINS
// ============================================================================

/// Extra-small gap between elements
pub const GAP_XS: &str = "4px";

/// Small gap between elements
pub const GAP_SM: &str = "6px";

/// Medium gap between elements
pub const GAP_MD: &str = "8px";

/// Large gap between elements
pub const GAP_LG: &str = "12px";

/// Extra-large gap between elements
pub const GAP_XL: &str = "18px";

// ============================================================================
// COLORS (references to CSS variables)
// ============================================================================

/// Primary background color
pub const COLOR_PRIMARY: &str = "var(--btn-primary-bg)";

/// Surface/panel background color
pub const COLOR_SURFACE: &str = "var(--surface)";

/// Primary text color
pub const COLOR_TEXT: &str = "var(--text)";

/// Secondary text color
pub const COLOR_TEXT_SECONDARY: &str = "var(--text2)";

/// Border color
pub const COLOR_BORDER: &str = "var(--border)";

/// Results border color
#[allow(dead_code)]
pub const COLOR_BORDER_RESULTS: &str = "var(--results-border)";

// ============================================================================
// SPECIFIC COMPONENT STYLES (convenience functions)
// ============================================================================

/// Button copy padding shorthand (vertical and horizontal)
#[allow(dead_code)]
pub fn button_copy_padding() -> String {
    format!("{} {}", COPY_BUTTON_PADDING_Y, COPY_BUTTON_PADDING_X)
}

/// Button padding shorthand (vertical and horizontal)
#[allow(dead_code)]
pub fn button_padding() -> String {
    format!("{} {}", BUTTON_PADDING_Y, BUTTON_PADDING_X)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spacing_scale_increases() {
        assert_eq!(SPACING_XS, "4px");
        assert_eq!(SPACING_SM, "8px");
        assert_eq!(SPACING_MD, "12px");
        assert_eq!(SPACING_LG, "16px");
        assert_eq!(SPACING_XL, "18px");
    }

    #[test]
    fn row_heights_increase() {
        assert_eq!(ROW_HEIGHT_SM, "32px");
        assert_eq!(ROW_HEIGHT_MD, "40px");
        assert_eq!(ROW_HEIGHT_LG, "48px");
        assert_eq!(ROW_HEIGHT_XL, "56px");
    }

    #[test]
    fn button_heights_consistent() {
        assert_eq!(BUTTON_MIN_HEIGHT, ROW_HEIGHT_MD);
        assert_eq!(COPY_BUTTON_MIN_HEIGHT, ROW_HEIGHT_MD);
    }

    #[test]
    fn cell_padding_combines_values() {
        let padding = cell_padding();
        assert!(padding.contains(CELL_PADDING_Y));
        assert!(padding.contains(CELL_PADDING_X));
    }

    #[test]
    fn button_padding_combines_values() {
        let padding = button_padding();
        assert!(padding.contains(BUTTON_PADDING_Y));
        assert!(padding.contains(BUTTON_PADDING_X));
    }

    #[test]
    fn button_copy_padding_combines_values() {
        let padding = button_copy_padding();
        assert!(padding.contains("clamp"));
    }

    #[test]
    fn responsive_values_use_clamp() {
        assert!(FONT_SIZE_RESPONSIVE_BASE.contains("clamp"));
        assert!(FONT_SIZE_RESPONSIVE_BUTTON.contains("clamp"));
        assert!(PADDING_RESPONSIVE_Y.contains("clamp"));
        assert!(PADDING_RESPONSIVE_X.contains("clamp"));
    }

    #[test]
    fn color_constants_reference_css_vars() {
        assert!(COLOR_PRIMARY.contains("var("));
        assert!(COLOR_SURFACE.contains("var("));
        assert!(COLOR_TEXT.contains("var("));
    }
}
