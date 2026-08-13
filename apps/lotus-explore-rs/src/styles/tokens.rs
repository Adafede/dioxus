// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Centralized design tokens for lotus-explore-rs.
//!
//! All dimension, spacing, sizing, color, and typography constants are defined
//! here as a single source of truth. Changes to these values cascade automatically
//! to all components that reference them.
//!
//! ## Organization
//! - **Spacing**: Fundamental spacing scale (XS → XL)
//! - **Heights**: Row heights for buttons, inputs, cells (with semantic naming)
//! - **Padding**: Vertical/horizontal padding for buttons, cells, and form elements
//! - **Responsive Values**: CSS clamp() values for fluid scaling
//! - **Table/Grid**: Table-specific dimensions
//! - **Borders & Radius**: Border widths and border-radius values
//! - **Colors**: CSS variable references to design system colors
//! - **Legacy Tokens**: Additional tokens from lotus design system

// ============================================================================
// SPACING SCALE (Primary)
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
// SPACING SCALE (Legacy tokens - alternate naming)
// ============================================================================

/// Space: 6px - minimal spacing
pub const SPACE_1: &str = "6px";

/// Space: 10px - small spacing
pub const SPACE_2: &str = "10px";

/// Space: 14px - medium spacing
pub const SPACE_3: &str = "14px";

/// Space: 20px - standard spacing
pub const SPACE_4: &str = "20px";

/// Space: 28px - large spacing
pub const SPACE_5: &str = "28px";

/// Layout: 10px - app layout gap and padding
pub const LAYOUT_GAP: &str = "10px";

/// Layout: 10px - sidebar padding
pub const SIDEBAR_PADDING: &str = "10px";

// ============================================================================
// MARGINS & PADDING (Component-specific)
// ============================================================================

/// Margin: vertical component of notice/share-bar margin (10px)
pub const MARGIN_NOTICE_V: &str = "10px";
/// Margin: horizontal component of notice/share-bar margin (22px)
pub const MARGIN_NOTICE_H: &str = "22px";

/// Padding: vertical component of form input padding (9px)
pub const FORM_INPUT_PADDING_V: &str = "9px";
/// Padding: horizontal component of form input padding (11px)
pub const FORM_INPUT_PADDING_H: &str = "11px";

/// Padding: vertical component of notice padding (9px)
pub const NOTICE_PADDING_V: &str = "9px";
/// Padding: horizontal component of notice padding (12px)
pub const NOTICE_PADDING_H: &str = "12px";

/// Padding: vertical component of share bar padding (7px)
pub const SHARE_BAR_PADDING_V: &str = "7px";
/// Padding: horizontal component of share bar padding (12px)
pub const SHARE_BAR_PADDING_H: &str = "12px";

/// Padding: top component of page header padding (14px)
pub const PAGE_HEADER_PADDING_T: &str = "14px";
/// Padding: horizontal component of page header padding (22px)
pub const PAGE_HEADER_PADDING_H: &str = "22px";
/// Padding: bottom component of page header padding (10px)
pub const PAGE_HEADER_PADDING_B: &str = "10px";

/// Padding: vertical component of search panel padding (18px)
pub const SEARCH_PANEL_PADDING_V: &str = "18px";
/// Padding: horizontal component of search panel padding (16px)
pub const SEARCH_PANEL_PADDING_H: &str = "16px";

/// Padding: vertical component of form section padding (10px)
pub const FORM_SECTION_PADDING_V: &str = "10px";
/// Padding: horizontal component of form section padding (12px)
pub const FORM_SECTION_PADDING_H: &str = "12px";

/// Padding: vertical component of share bar input padding (4px)
pub const SHARE_BAR_INPUT_PADDING_V: &str = "4px";
/// Padding: horizontal component of share bar input padding (8px)
pub const SHARE_BAR_INPUT_PADDING_H: &str = "8px";

/// Padding: vertical component of file input button padding (6px)
pub const FILE_BUTTON_PADDING_V: &str = "6px";
/// Padding: horizontal component of file input button padding (10px)
pub const FILE_BUTTON_PADDING_H: &str = "10px";

/// Padding: vertical component of curation table cell padding (8px)
pub const TABLE_CELL_PADDING_V: &str = "8px";
/// Padding: horizontal component of curation table cell padding (10px)
pub const TABLE_CELL_PADDING_H: &str = "10px";

/// Padding: loading state padding (48px)
pub const LOADING_STATE_PADDING: &str = "48px";

/// Padding: vertical component of empty state padding (64px)
pub const EMPTY_STATE_PADDING_V: &str = "64px";
/// Padding: horizontal component of empty state padding (24px)
pub const EMPTY_STATE_PADDING_H: &str = "24px";

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
// GAPS (Primary scale)
// ============================================================================

/// Extra-small gap between elements: 4px
pub const GAP_XS: &str = "4px";

/// Extra-small gap between elements: 6px
pub const GAP_XXS: &str = "6px";

/// Small gap between elements: 6px
pub const GAP_SM: &str = "6px";

/// Medium gap between elements: 8px
pub const GAP_MD: &str = "8px";

/// Large gap between elements: 12px
pub const GAP_LG: &str = "12px";

/// Extra-large gap between elements: 18px
pub const GAP_XL: &str = "18px";

// ============================================================================
// BORDERS & RADIUS
// ============================================================================

/// Standard border width: 1px
pub const BORDER_WIDTH: &str = "1px";

/// Border radius: 3px - tiny
pub const RADIUS_XS: &str = "3px";

/// Border radius: 4px - small
pub const RADIUS_SM: &str = "4px";

/// Border radius: 10px - standard
pub const RADIUS: &str = "10px";

/// Border radius: 12px - medium
pub const RADIUS_MD: &str = "12px";

/// Border radius: 14px - slightly larger
pub const RADIUS_XL: &str = "14px";

/// Border radius: 16px - large (panels)
pub const RADIUS_LG: &str = "16px";

/// Small border-radius: 4px
pub const BORDER_RADIUS_SM: &str = "4px";

/// Medium border-radius: 6px
pub const BORDER_RADIUS_MD: &str = "6px";

/// Large border-radius: 10px (used by cell containers)
pub const BORDER_RADIUS_LG: &str = "10px";

// ============================================================================
// TYPOGRAPHY TOKENS (via CSS variables - responsive)
// ============================================================================

/// Font family: sans-serif system stack
pub const FONT_SANS: &str = "var(--sans)";

/// Font family: monospace system stack
pub const FONT_MONO: &str = "var(--mono)";

/// Font size: --fs-0 (smallest)
pub const FS_0: &str = "var(--fs-0)";

/// Font size: --fs-1
pub const FS_1: &str = "var(--fs-1)";

/// Font size: --fs-2
pub const FS_2: &str = "var(--fs-2)";

/// Font size: --fs-3 (body)
pub const FS_3: &str = "var(--fs-3)";

/// Font size: --fs-4 (largest)
pub const FS_4: &str = "var(--fs-4)";

/// Font size: --fs-body (standard text)
pub const FS_BODY: &str = "var(--fs-body)";

/// Font size: --fs-label (uppercase labels)
pub const FS_LABEL: &str = "var(--fs-label)";

/// Font size: --fs-micro (smallest text)
pub const FS_MICRO: &str = "var(--fs-micro)";

/// Font size: --fs-ui (UI elements)
pub const FS_UI: &str = "var(--fs-ui)";

/// Font size: --fs-stat (statistics)
pub const FS_STAT: &str = "var(--fs-stat)";

// ============================================================================
// COLORS (references to CSS variables)
// ============================================================================

/// Background: main bg color
pub const BG: &str = "var(--bg)";

/// Background: secondary/alt bg
pub const BG2: &str = "var(--bg2)";

/// Background: surface
pub const SURFACE: &str = "var(--surface)";

/// Background: secondary surface
pub const SURFACE2: &str = "var(--surface2)";

/// Border: standard border color
pub const BORDER: &str = "var(--border)";

/// Text: primary text
pub const TEXT: &str = "var(--text)";

/// Text: secondary text
pub const TEXT2: &str = "var(--text2)";

/// Text: tertiary text (muted)
pub const TEXT3: &str = "var(--text3)";

/// Accent: primary accent color
pub const ACCENT: &str = "var(--accent)";

/// Accent: secondary accent
pub const ACCENT2: &str = "var(--accent2)";

/// Button primary background
pub const BTN_PRIMARY_BG: &str = "var(--btn-primary-bg)";

/// Button primary hover background
pub const BTN_PRIMARY_HOVER_BG: &str = "var(--btn-primary-hover-bg)";

/// Green color
pub const GREEN: &str = "var(--green)";

/// Red color
pub const RED: &str = "var(--red)";

/// Yellow color
pub const YELLOW: &str = "var(--yellow)";

/// Purple color
pub const PURPLE: &str = "var(--purple)";

/// Glass effect color
pub const GLASS: &str = "var(--glass)";

/// Ring (focus) color
pub const RING: &str = "var(--ring)";

/// Critical text color
pub const CRITICAL_TEXT: &str = "var(--critical-text)";

/// Critical muted text
pub const CRITICAL_MUTED: &str = "var(--critical-muted)";

/// Panel background
pub const PANEL_BG: &str = "var(--panel-bg)";

/// Panel background (soft)
pub const PANEL_BG_SOFT: &str = "var(--panel-bg-soft)";

/// Panel border
pub const PANEL_BORDER: &str = "var(--panel-border)";

/// Results border
pub const RESULTS_BORDER: &str = "var(--results-border)";

/// Panel shadow
pub const PANEL_SHADOW: &str = "var(--panel-shadow)";

/// Wikidata compound color
pub const WD_COMPOUND: &str = "var(--wd-compound)";
/// Wikidata taxon color
pub const WD_TAXON: &str = "var(--wd-taxon)";
/// Wikidata reference color
pub const WD_REFERENCE: &str = "var(--wd-reference)";
/// Wikidata entries color
pub const WD_ENTRIES: &str = "var(--wd-entries)";

/// Wikidata compound stripe
pub const WD_COMPOUND_STRIPE: &str = "var(--wd-compound-stripe)";
/// Wikidata taxon stripe
pub const WD_TAXON_STRIPE: &str = "var(--wd-taxon-stripe)";
/// Wikidata reference stripe
pub const WD_REFERENCE_STRIPE: &str = "var(--wd-reference-stripe)";
/// Wikidata entries stripe
pub const WD_ENTRIES_STRIPE: &str = "var(--wd-entries-stripe)";

/// Wikidata compound soft background
pub const WD_COMPOUND_SOFT_BG: &str = "var(--wd-compound-soft-bg)";
/// Wikidata compound soft border
pub const WD_COMPOUND_SOFT_BORDER: &str = "var(--wd-compound-soft-border)";
/// Wikidata compound soft border (weak)
pub const WD_COMPOUND_SOFT_BORDER_WEAK: &str = "var(--wd-compound-soft-border-weak)";

/// Wikidata taxon soft background
pub const WD_TAXON_SOFT_BG: &str = "var(--wd-taxon-soft-bg)";
/// Wikidata taxon soft border
pub const WD_TAXON_SOFT_BORDER: &str = "var(--wd-taxon-soft-border)";

/// Wikidata reference soft background
pub const WD_REFERENCE_SOFT_BG: &str = "var(--wd-reference-soft-bg)";
/// Wikidata reference soft border
pub const WD_REFERENCE_SOFT_BORDER: &str = "var(--wd-reference-soft-border)";
/// Wikidata reference soft border (weak)
pub const WD_REFERENCE_SOFT_BORDER_WEAK: &str = "var(--wd-reference-soft-border-weak)";

/// Statistics compound background
pub const STAT_COMPOUND_BG: &str = "var(--stat-compound-bg)";
/// Statistics compound border
pub const STAT_COMPOUND_BORDER: &str = "var(--stat-compound-border)";
/// Statistics compound stripe
pub const STAT_COMPOUND_STRIPE: &str = "var(--stat-compound-stripe)";

/// Statistics taxon background
pub const STAT_TAXON_BG: &str = "var(--stat-taxon-bg)";
/// Statistics taxon border
pub const STAT_TAXON_BORDER: &str = "var(--stat-taxon-border)";
/// Statistics taxon stripe
pub const STAT_TAXON_STRIPE: &str = "var(--stat-taxon-stripe)";

/// Statistics reference background
pub const STAT_REFERENCE_BG: &str = "var(--stat-reference-bg)";
/// Statistics reference border
pub const STAT_REFERENCE_BORDER: &str = "var(--stat-reference-border)";
/// Statistics reference stripe
pub const STAT_REFERENCE_STRIPE: &str = "var(--stat-reference-stripe)";

/// Statistics total background
pub const STAT_TOTAL_BG: &str = "var(--stat-total-bg)";
/// Statistics total border
pub const STAT_TOTAL_BORDER: &str = "var(--stat-total-border)";
/// Statistics total stripe
pub const STAT_TOTAL_STRIPE: &str = "var(--stat-total-stripe)";

/// Primary background color (alias)
pub const COLOR_PRIMARY: &str = "var(--btn-primary-bg)";

/// Surface/panel background color (alias)
pub const COLOR_SURFACE: &str = "var(--surface)";

/// Primary text color (alias)
pub const COLOR_TEXT: &str = "var(--text)";

/// Secondary text color (alias)
pub const COLOR_TEXT_SECONDARY: &str = "var(--text2)";

/// Border color (alias)
pub const COLOR_BORDER: &str = "var(--border)";

/// Results border color (alias)
#[allow(dead_code)]
pub const COLOR_BORDER_RESULTS: &str = "var(--results-border)";

// ============================================================================
// SHADOW TOKENS
// ============================================================================

/// Shadow: extra small
pub const SHADOW_XS: &str = "var(--shadow-xs)";

/// Shadow: small
pub const SHADOW_SM: &str = "var(--shadow-sm)";

/// Shadow: medium
pub const SHADOW_MD: &str = "var(--shadow-md)";

// ============================================================================
// ACCESSIBILITY TOKENS
// ============================================================================

/// Minimum tap target size (WCAG)
pub const TAP_TARGET_MIN: &str = "40px";

/// Transition timing for smooth animations
pub const TRANSITION_TIMING: &str = ".15s";

/// Focus outline width
pub const FOCUS_OUTLINE_WIDTH: &str = "2px";

/// Focus outline offset
pub const FOCUS_OUTLINE_OFFSET: &str = "2px";

// ============================================================================
// BREAKPOINTS (for media queries - in pixels)
// ============================================================================

pub const BREAK_360: &str = "360px";
pub const BREAK_430: &str = "430px";
pub const BREAK_480: &str = "480px";
pub const BREAK_768: &str = "768px";
pub const BREAK_769: &str = "769px";
pub const BREAK_1023: &str = "1023px";
pub const BREAK_1024: &str = "1024px";
pub const BREAK_1440: &str = "1440px";

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
