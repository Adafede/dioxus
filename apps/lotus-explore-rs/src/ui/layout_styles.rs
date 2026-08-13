// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Lotus-specific layout overrides and customizations.
//! Styling for headers, search buttons, and query panels.

use ui::prelude::*;

// ============================================================================
// COLOR TOKENS
// ============================================================================

pub mod stat_stripe_colors {
    //! Stripe colors for statistic badges (compounds, taxa, references, entries).
    pub const COMPOUND: &str = "var(--wd-compound-stripe)";
    pub const TAXON: &str = "var(--wd-taxon-stripe)";
    pub const REFERENCE: &str = "var(--wd-reference-stripe)";
    pub const ENTRIES: &str = "var(--wd-entries-stripe)";
}

pub mod borders {
    //! Border colors and styles used across components.
    pub const RESULTS_BORDER: &str = "var(--results-border)";
}

pub mod backgrounds {
    //! Background color tokens.
    pub const SURFACE: &str = "var(--surface)";
}

pub mod text {
    //! Text color tokens.
    pub const PRIMARY: &str = "var(--text)";
    pub const SECONDARY: &str = "var(--text2)";
}

pub mod shadows {
    //! Shadow tokens.
    pub const SHADOW_XS: &str = "var(--shadow-xs)";
}

// ============================================================================
// SPACING TOKENS
// ============================================================================

pub mod spacing {
    //! Spacing values for padding, margins, and gaps.
    pub const STAT_BADGE_PAD: &str = "10px 12px";
    pub const STAT_VALUE_GAP: &str = "8px";
    pub const STAT_BADGE_GAP: &str = "4px";
    pub const STAT_BAR_GAP: &str = "10px";
    pub const HEADER_META_GAP: &str = "12px";
    pub const QUERY_SUMMARY_PADDING: &str = "8px 14px 8px 32px";
}

// ============================================================================
// TYPOGRAPHY TOKENS
// ============================================================================

pub mod typography {
    //! Font size and weight tokens.
    pub const FONT_SIZE_STAT: &str = "var(--fs-stat)";
    pub const FONT_SIZE_0: &str = "var(--fs-0)";
    pub const FONT_WEIGHT_BOLD: &str = "800";
    pub const FONT_WEIGHT_SEMIBOLD: &str = "700";
    pub const LETTER_SPACING_TITLE: &str = "0.08em";
    pub const LETTER_SPACING_STAT: &str = "-0.02em";
    pub const LINE_HEIGHT_STAT: &str = "1.2";
}

// ============================================================================
// HEADER LAYOUT STYLES
// ============================================================================

/// Page header container with sticky positioning and bottom border.
pub fn lotus_page_header_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .flex_direction("column")
        .gap("8px")
        .property("padding-left", "max(18px, env(safe-area-inset-left))")
        .property("padding-right", "max(18px, env(safe-area-inset-right))")
        .build()
}

/// Page brand section: flex row with title and language switcher.
pub fn lotus_page_brand_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .flex_direction("row")
        .flex_wrap("wrap")
        .align_items("flex-start")
        .gap("8px 10px")
        .build()
}

/// Page title: main heading with min-width constraint.
pub fn lotus_page_title_style() -> String {
    StyleBuilder::new()
        .property("min-width", "0")
        .property("flex", "1 1 260px")
        .font_size("var(--fs-4)")
        .property("margin", "0")
        .build()
}

/// Page title link: inline-flex with text overflow handling.
pub fn lotus_page_title_link_style() -> String {
    StyleBuilder::new()
        .display("inline-flex")
        .property("max-width", "100%")
        .gap("8px")
        .text_decoration("none")
        .build()
}

/// Page title text: proper wrapping and line height.
pub fn lotus_page_title_text_style() -> String {
    StyleBuilder::new()
        .property("line-height", "1.1")
        .property("word-break", "break-word")
        .build()
}

/// Page subtitle: secondary color with smaller font.
pub fn lotus_page_subtitle_style() -> String {
    StyleBuilder::new()
        .font_size("var(--fs-1)")
        .property("margin", "0")
        .color("var(--text2)")
        .build()
}

/// Archive note section: inline display.
pub fn lotus_page_archive_note_style() -> String {
    StyleBuilder::new().display("inline").build()
}

/// Archive label: bold small-caps label.
pub fn lotus_page_archive_label_style() -> String {
    StyleBuilder::new()
        .font_weight("700")
        .property("font-variant", "small-caps")
        .build()
}

/// Archive link: accent color, no wrap.
pub fn lotus_page_archive_link_style() -> String {
    StyleBuilder::new()
        .text_decoration("none")
        .color("var(--accent)")
        .property("white-space", "nowrap")
        .build()
}

// ============================================================================
// TABLE HEADER STYLES
// ============================================================================

/// Table header cell: simple styling with padding and border.
pub fn lotus_table_header_cell_style() -> String {
    StyleBuilder::new()
        .padding("9px 10px")
        .text_align("left")
        .font_size("var(--fs-label)")
        .font_weight("700")
        .border_bottom("1px solid var(--results-border)")
        .property("white-space", "nowrap")
        .property("user-select", "none")
        .build()
}

/// Header label text: block display with no-break constraint.
pub fn lotus_header_label_style() -> String {
    StyleBuilder::new()
        .display("block")
        .property("min-width", "max-content")
        .property("white-space", "nowrap")
        .property("overflow", "visible")
        .property("text-overflow", "clip")
        .property("line-height", "1.2")
        .font_weight("inherit")
        .font_size("inherit")
        .property("text-transform", "inherit")
        .property("letter-spacing", "inherit")
        .build()
}

/// Sort button: transparent grid-based layout for label + icon.
pub fn lotus_sort_button_style() -> String {
    StyleBuilder::new()
        .property("appearance", "none")
        .font_family("inherit")
        .padding("0")
        .property("margin", "0")
        .cursor("pointer")
        .display("grid")
        .align_items("start")
        .property("grid-template-columns", "auto auto")
        .property("column-gap", "6px")
        .property("width", "100%")
        .property("min-width", "max-content")
        .background_color("transparent")
        .border("none")
        .color("inherit")
        .build()
}

/// Sort icon: muted color, smaller font.
pub fn lotus_sort_icon_style() -> String {
    StyleBuilder::new()
        .color("var(--text3)")
        .font_size("var(--fs-0)")
        .font_weight("700")
        .property("line-height", "1")
        .build()
}

// ============================================================================
// SEARCH BUTTON STYLES
// ============================================================================

/// Search button when dirty (form has changes): emphasized background with accent.
pub fn lotus_search_button_dirty_style() -> String {
    StyleBuilder::new()
        .display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .gap("8px")
        .border_radius("4px")
        .property("min-height", "40px")
        .padding("11px 16px")
        .font_size("var(--fs-ui)")
        .font_weight("700")
        .cursor("pointer")
        .background_color("color-mix(in srgb, var(--btn-primary-bg) 90%, var(--accent))")
        .color("var(--text)")
        .box_shadow("var(--shadow-xs)")
        .property(
            "transition",
            "background .15s, box-shadow .15s, transform .12s ease",
        )
        .build()
}

/// Search button conditional: returns dirty style if true, base style if false.
pub fn lotus_search_button_state(dirty: bool) -> String {
    if dirty {
        lotus_search_button_dirty_style()
    } else {
        ui::styles::button_base_style()
    }
}

// ============================================================================
// PANEL CONTAINER STYLES
// ============================================================================

/// Flex column stack: vertical layout with padding and gap.
pub fn lotus_panel_stack_style(padding: &str, gap: &str) -> String {
    StyleBuilder::new()
        .display("flex")
        .flex_direction("column")
        .gap(gap)
        .padding(padding)
        .build()
}

/// Section card: bordered container with soft background and rounded corners.
pub fn lotus_section_card_style() -> String {
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

/// Ketcher panel: dialog container for structure editor with border and shadow.
pub fn lotus_ketcher_panel_style() -> String {
    StyleBuilder::new()
        .property("margin", "0")
        .border("1px solid var(--panel-border)")
        .border_radius("var(--radius)")
        .background_color("var(--panel-bg-soft)")
        .box_shadow("var(--panel-shadow)")
        .property(
            "transition",
            "background .15s ease, border-color .15s ease, box-shadow .15s ease",
        )
        .build()
}

/// Iframe container: full width with specific height constraints.
pub fn lotus_iframe_style() -> String {
    StyleBuilder::new()
        .property("width", "100%")
        .property("height", "min(78vh, 820px)")
        .property("min-height", "600px")
        .border("1px solid var(--border)")
        .border_radius("4px")
        .background_color("var(--surface)")
        .build()
}

/// Ketcher content wrapper: padding and gap for nested content.
pub fn lotus_ketcher_wrap_style() -> String {
    lotus_panel_stack_style("0 14px 14px", "10px")
}

// ============================================================================
// STAT BADGE STYLES
// ============================================================================

/// Stat badge container: flex column with border, stripe, and shadow.
pub fn lotus_stat_badge_style(stripe: &str) -> String {
    StyleBuilder::new()
        .display("flex")
        .flex_direction("column")
        .gap(spacing::STAT_BADGE_GAP)
        .property("min-width", "0")
        .padding(spacing::STAT_BADGE_PAD)
        .border_radius("12px")
        .property("border", &format!("1px solid {}", borders::RESULTS_BORDER))
        .background_color(backgrounds::SURFACE)
        .box_shadow(shadows::SHADOW_XS)
        .property("position", "relative")
        .property("overflow", "hidden")
        .property("flex", "1 1 0")
        .property("border-left", &format!("3px solid {stripe}"))
        .build()
}

/// Stat value text: large bold font with tabular numbers.
pub fn lotus_stat_value_style() -> String {
    StyleBuilder::new()
        .font_size(typography::FONT_SIZE_STAT)
        .font_weight(typography::FONT_WEIGHT_BOLD)
        .color(text::PRIMARY)
        .property("font-variant-numeric", "tabular-nums")
        .property("letter-spacing", typography::LETTER_SPACING_STAT)
        .property("min-width", "0")
        .property("flex", "0 1 auto")
        .property("line-height", typography::LINE_HEIGHT_STAT)
        .build()
}

/// Stat secondary value: smaller font, tabular numbers, overflow wrap.
pub fn lotus_stat_secondary_style() -> String {
    StyleBuilder::new()
        .font_size(typography::FONT_SIZE_0)
        .font_weight(typography::FONT_WEIGHT_SEMIBOLD)
        .color(text::PRIMARY)
        .property("font-variant-numeric", "tabular-nums")
        .property("min-width", "0")
        .property("max-width", "100%")
        .property("overflow-wrap", "anywhere")
        .property("flex", "0 0 auto")
        .build()
}

/// Stat bar container: auto-fit grid with responsive columns.
pub fn lotus_stat_bar_style() -> String {
    StyleBuilder::new()
        .display("grid")
        .property(
            "grid-template-columns",
            "repeat(auto-fit, minmax(120px, 1fr))",
        )
        .gap(spacing::STAT_BAR_GAP)
        .align_items("stretch")
        .property("width", "100%")
        .property("min-width", "0")
        .build()
}

/// Stat value row: flex wrap with baseline alignment.
pub fn lotus_stat_value_row_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .property("flex-wrap", "wrap")
        .align_items("baseline")
        .gap(spacing::STAT_VALUE_GAP)
        .property("min-width", "0")
        .property("width", "100%")
        .justify_content("center")
        .build()
}

/// Stat label: uppercase with centered text.
pub fn lotus_stat_label_style() -> String {
    StyleBuilder::new()
        .font_size(typography::FONT_SIZE_0)
        .color(text::SECONDARY)
        .property("text-transform", "uppercase")
        .property("letter-spacing", typography::LETTER_SPACING_TITLE)
        .font_weight(typography::FONT_WEIGHT_SEMIBOLD)
        .property("width", "100%")
        .text_align("center")
        .build()
}

// ============================================================================
// DOWNLOAD & TOOLBAR STYLES
// ============================================================================

/// Toolbar actions container: flex wrap with space-between.
pub fn lotus_toolbar_actions_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .property("flex-wrap", "wrap")
        .gap("8px")
        .align_items("center")
        .justify_content("space-between")
        .property("min-width", "0")
        .build()
}

/// Download group container: flex wrap with items.
pub fn lotus_dl_group_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .property("flex-wrap", "wrap")
        .gap("8px")
        .property("min-width", "0")
        .property("max-width", "100%")
        .align_items("stretch")
        .build()
}

/// Spinner small: animated circular loader.
pub fn lotus_spinner_sm_style() -> String {
    StyleBuilder::new()
        .property("width", "14px")
        .property("height", "14px")
        .border("2px solid color-mix(in srgb, var(--text) 30%, transparent)")
        .property("border-top-color", "var(--text)")
        .border_radius("50%")
        .property("animation", "spin .7s linear infinite")
        .property("display", "inline-block")
        .build()
}

/// Download button small: flex button with border and transition.
pub fn lotus_button_small_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .align_items("center")
        .justify_content("center")
        .gap("6px")
        .border("1px solid var(--border)")
        .border_radius("8px")
        .padding("8px 12px")
        .font_size("var(--fs-0)")
        .font_weight("600")
        .cursor("pointer")
        .background_color("transparent")
        .color("var(--text)")
        .property("flex", "1 1 auto")
        .property("min-width", "0")
        .property("white-space", "nowrap")
        .property(
            "transition",
            "border-color .15s, background .15s, box-shadow .15s, transform .12s ease",
        )
        .build()
}

// ============================================================================
// QUERY PANEL STYLES
// ============================================================================

/// Query summary toggle: cursor pointer with gradient background.
pub fn lotus_query_summary_toggle_style() -> String {
    StyleBuilder::new()
        .cursor("pointer")
        .padding(spacing::QUERY_SUMMARY_PADDING)
        .font_size("var(--fs-0)")
        .color("var(--text2)")
        .property("user-select", "none")
        .property("letter-spacing", "0.04em")
        .font_weight("600")
        .property("list-style", "none")
        .property("position", "relative")
        .property("transition", "color .15s ease, background .15s ease")
        .property(
            "background",
            "linear-gradient(135deg, transparent 0%, rgba(255,255,255,0.02) 100%)",
        )
        .build()
}

/// Query summary chevron: positioned absolute with rotation transform.
pub fn lotus_query_summary_chevron_style(is_open: bool) -> String {
    let (rotation_deg, color) = if is_open {
        ("90deg", "var(--accent)")
    } else {
        ("0deg", "var(--text3)")
    };

    let transform = format!("translateY(-50%) rotate({})", rotation_deg);

    StyleBuilder::new()
        .property("position", "absolute")
        .property("left", "12px")
        .property("top", "50%")
        .property("transition", "transform .2s ease")
        .property("font-size", "14px")
        .property("line-height", "1")
        .color(color)
        .property("transform", &transform)
        .build()
}

/// Query panel container: soft background with border and shadow.
pub fn lotus_query_panel_style() -> String {
    StyleBuilder::new()
        .background_color("var(--panel-bg-soft)")
        .border("1px solid var(--panel-border)")
        .border_radius("var(--radius)")
        .box_shadow("var(--panel-shadow)")
        .property(
            "transition",
            "background .15s ease, border-color .15s ease, box-shadow .15s ease",
        )
        .build()
}

/// Query body content: positioned relative with bottom rounded corners.
pub fn lotus_query_body_style() -> String {
    StyleBuilder::new()
        .property("position", "relative")
        .border_radius("0 0 var(--radius) var(--radius)")
        .property("overflow", "hidden")
        .build()
}

/// Query text display: monospace font with left border accent.
pub fn lotus_query_text_style() -> String {
    StyleBuilder::new()
        .padding("12px 16px")
        .property("margin", "0")
        .font_family("var(--mono)")
        .font_size("var(--fs-0)")
        .color("var(--text)")
        .background_color("var(--bg2)")
        .property("border-left", "3px solid var(--wd-entries)")
        .property("white-space", "pre-wrap")
        .property("word-break", "break-word")
        .property("max-height", "320px")
        .property("overflow", "auto")
        .build()
}

// ============================================================================
// THEME UTILITIES
// ============================================================================

/// Detect if dark mode is active based on system preferences.
/// Must be called at component render time to detect changes.
#[cfg(target_arch = "wasm32")]
pub fn is_dark_mode() -> bool {
    if let Ok(window) = web_sys::window().ok_or("no window") {
        if let Ok(media) = window.match_media("(prefers-color-scheme: dark)") {
            if let Some(media_query) = media {
                return media_query.matches();
            }
        }
    }
    false
}

/// Fallback for non-WASM builds (always returns false).
#[cfg(not(target_arch = "wasm32"))]
pub fn is_dark_mode() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::{spacing, stat_stripe_colors};

    #[test]
    fn stat_stripe_colors_are_nonempty() {
        assert!(!stat_stripe_colors::COMPOUND.is_empty());
        assert!(!stat_stripe_colors::TAXON.is_empty());
        assert!(!stat_stripe_colors::REFERENCE.is_empty());
        assert!(!stat_stripe_colors::ENTRIES.is_empty());
    }

    #[test]
    fn spacing_tokens_parse() {
        assert!(spacing::STAT_BADGE_GAP.contains("px"));
    }
}
