// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Lotus design tokens: centralized spacing, colors, typography, and other design values.

// ─────────────────────────────────────────────────────────────────────────────
// SPACING TOKENS (in pixels)
// ─────────────────────────────────────────────────────────────────────────────

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

/// Margin: 10px 22px - notice/share-bar margin (horizontal: 22px, vertical: 10px)
pub const MARGIN_NOTICE_V: &str = "10px";
pub const MARGIN_NOTICE_H: &str = "22px";

/// Padding: 9px 11px - form input padding
pub const FORM_INPUT_PADDING_V: &str = "9px";
pub const FORM_INPUT_PADDING_H: &str = "11px";

/// Padding: 9px 12px - notice padding
pub const NOTICE_PADDING_V: &str = "9px";
pub const NOTICE_PADDING_H: &str = "12px";

/// Padding: 7px 12px - share bar padding
pub const SHARE_BAR_PADDING_V: &str = "7px";
pub const SHARE_BAR_PADDING_H: &str = "12px";

/// Padding: 14px 22px 10px - page header padding
pub const PAGE_HEADER_PADDING_T: &str = "14px";
pub const PAGE_HEADER_PADDING_H: &str = "22px";
pub const PAGE_HEADER_PADDING_B: &str = "10px";

/// Padding: 18px 16px - search panel padding
pub const SEARCH_PANEL_PADDING_V: &str = "18px";
pub const SEARCH_PANEL_PADDING_H: &str = "16px";

/// Padding: 10px 12px - form section padding
pub const FORM_SECTION_PADDING_V: &str = "10px";
pub const FORM_SECTION_PADDING_H: &str = "12px";

/// Padding: 4px 8px - share bar input padding
pub const SHARE_BAR_INPUT_PADDING_V: &str = "4px";
pub const SHARE_BAR_INPUT_PADDING_H: &str = "8px";

/// Padding: 6px 10px - file input button padding
pub const FILE_BUTTON_PADDING_V: &str = "6px";
pub const FILE_BUTTON_PADDING_H: &str = "10px";

/// Padding: 8px 10px - curation table cell padding
pub const TABLE_CELL_PADDING_V: &str = "8px";
pub const TABLE_CELL_PADDING_H: &str = "10px";

/// Padding: 48px - loading state padding
pub const LOADING_STATE_PADDING: &str = "48px";

/// Padding: 64px 24px - empty state padding
pub const EMPTY_STATE_PADDING_V: &str = "64px";
pub const EMPTY_STATE_PADDING_H: &str = "24px";

/// Gap: 4px - small gaps
pub const GAP_XS: &str = "4px";

/// Gap: 6px - extra small gaps
pub const GAP_XXS: &str = "6px";

/// Gap: 10px - small-medium gaps
pub const GAP_SM: &str = "10px";

/// Gap: 12px - medium gaps
pub const GAP_MD: &str = "12px";

/// Gap: 14px - medium-large gaps
pub const GAP_LG: &str = "14px";

// ─────────────────────────────────────────────────────────────────────────────
// BORDER & RADIUS TOKENS
// ─────────────────────────────────────────────────────────────────────────────

/// Border radius: 10px - standard
pub const RADIUS: &str = "10px";

/// Border radius: 4px - small
pub const RADIUS_SM: &str = "4px";

/// Border radius: 16px - large (panels)
pub const RADIUS_LG: &str = "16px";

/// Border radius: 12px - medium
pub const RADIUS_MD: &str = "12px";

/// Border radius: 14px - slightly larger
pub const RADIUS_XL: &str = "14px";

/// Border radius: 3px - tiny
pub const RADIUS_XS: &str = "3px";

// ─────────────────────────────────────────────────────────────────────────────
// BORDER RADIUS CSS CUSTOM PROPERTIES (themeable, via CSS variables)
// ─────────────────────────────────────────────────────────────────────────────

/// Border radius CSS custom property — renders `var(--radius)` (themeable).
pub const BORDER_RADIUS: &str = "var(--radius)";

/// Border radius (small) CSS custom property — renders `var(--radius-sm)`.
pub const BORDER_RADIUS_SM: &str = "var(--radius-sm)";

// ─────────────────────────────────────────────────────────────────────────────
// COMPOSITE PRESETS (pre-built, byte-identical value combinations)
// ─────────────────────────────────────────────────────────────────────────────

/// 1px solid border using the default border color.
pub const BORDER_DEFAULT: &str = "1px solid var(--border)";

/// 1px solid border using the panel border color.
pub const BORDER_PANEL: &str = "1px solid var(--panel-border)";

/// 1px solid border using the results-table border color.
pub const BORDER_RESULTS: &str = "1px solid var(--results-border)";

/// Thick (3px) solid border using the default border color.
pub const BORDER_THICK: &str = "3px solid var(--border)";

/// Top-only border radius (`0 0 var(--radius) var(--radius)`).
pub const BORDER_RADIUS_TOP: &str = "0 0 var(--radius) var(--radius)";

/// Surface tinted to 94% opacity (subtle background).
pub const SURFACE_94_TINT: &str = "color-mix(in srgb, var(--surface) 94%, transparent)";

/// Surface tinted to 90% opacity.
pub const SURFACE_90_TINT: &str = "color-mix(in srgb, var(--surface) 90%, transparent)";

/// Surface tinted to 88% opacity.
pub const SURFACE_88_TINT: &str = "color-mix(in srgb, var(--surface) 88%, transparent)";

/// Button primary background blended 90%/100% with accent.
pub const BTN_PRIMARY_ACCENT_TINT: &str =
    "color-mix(in srgb, var(--btn-primary-bg) 90%, var(--accent))";

/// 2px solid border using text color at 30% opacity.
pub const BORDER_TEXT_30_TINT: &str = "2px solid color-mix(in srgb, var(--text) 30%, transparent)";

// ─────────────────────────────────────────────────────────────────────────────
// TYPOGRAPHY TOKENS (via CSS variables - responsive)
// ─────────────────────────────────────────────────────────────────────────────

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

// ─────────────────────────────────────────────────────────────────────────────
// COLOR TOKENS (via CSS variables)
// ─────────────────────────────────────────────────────────────────────────────

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

// Wikidata color palette
pub const WD_COMPOUND: &str = "var(--wd-compound)";
pub const WD_TAXON: &str = "var(--wd-taxon)";
pub const WD_REFERENCE: &str = "var(--wd-reference)";
pub const WD_ENTRIES: &str = "var(--wd-entries)";

/// Footer/interactive colors: theme-aware versions used for labels, links, and row accents.
pub const FOOTER_WD_COMPOUND: &str = "var(--footer-wd-compound)";
pub const FOOTER_WD_TAXON: &str = "var(--footer-wd-taxon)";
pub const FOOTER_WD_REFERENCE: &str = "var(--footer-wd-reference)";
pub const FOOTER_WD_ENTRIES: &str = "var(--footer-wd-entries)";

/// Statistics panel colors, derived from the footer palette to stay in sync with theme.
pub const STAT_COMPOUND_BG: &str = "var(--stat-compound-bg)";
pub const STAT_COMPOUND_BORDER: &str = "var(--stat-compound-border)";
pub const STAT_COMPOUND_STRIPE: &str = "var(--stat-compound-stripe)";
pub const STAT_TAXON_BG: &str = "var(--stat-taxon-bg)";
pub const STAT_TAXON_BORDER: &str = "var(--stat-taxon-border)";
pub const STAT_TAXON_STRIPE: &str = "var(--stat-taxon-stripe)";
pub const STAT_REFERENCE_BG: &str = "var(--stat-reference-bg)";
pub const STAT_REFERENCE_BORDER: &str = "var(--stat-reference-border)";
pub const STAT_REFERENCE_STRIPE: &str = "var(--stat-reference-stripe)";
pub const STAT_TOTAL_BG: &str = "var(--stat-total-bg)";
pub const STAT_TOTAL_BORDER: &str = "var(--stat-total-border)";
pub const STAT_TOTAL_STRIPE: &str = "var(--stat-total-stripe)";

// ─────────────────────────────────────────────────────────────────────────────
// SHADOW TOKENS
// ─────────────────────────────────────────────────────────────────────────────

/// Shadow: extra small
pub const SHADOW_XS: &str = "var(--shadow-xs)";

/// Shadow: small
pub const SHADOW_SM: &str = "var(--shadow-sm)";

/// Shadow: medium
pub const SHADOW_MD: &str = "var(--shadow-md)";

// ─────────────────────────────────────────────────────────────────────────────
// ACCESSIBILITY TOKENS
// ─────────────────────────────────────────────────────────────────────────────

/// Minimum tap target size (WCAG)
pub const TAP_TARGET_MIN: &str = "40px";

/// Transition timing for smooth animations
pub const TRANSITION_TIMING: &str = ".15s";

/// Focus outline width
pub const FOCUS_OUTLINE_WIDTH: &str = "2px";

/// Focus outline offset
pub const FOCUS_OUTLINE_OFFSET: &str = "2px";

// ─────────────────────────────────────────────────────────────────────────────
// BREAKPOINTS (for media queries - in pixels)
// ─────────────────────────────────────────────────────────────────────────────

pub const BREAK_360: &str = "360px";
pub const BREAK_430: &str = "430px";
pub const BREAK_480: &str = "480px";
pub const BREAK_768: &str = "768px";
pub const BREAK_769: &str = "769px";
pub const BREAK_1023: &str = "1023px";
pub const BREAK_1024: &str = "1024px";
pub const BREAK_1440: &str = "1440px";
