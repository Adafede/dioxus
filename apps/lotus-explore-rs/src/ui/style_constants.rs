// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Comprehensive Design System for lotus-explore-rs
//!
//! Consolidates all UI element styling (buttons, forms, panels, notices, etc.) into a
//! centralized, reusable system. Eliminates duplication and ensures consistency across
//! the application.
//!
//! ## Architecture
//! - Color tokens (semantic, not literal)
//! - Spacing and sizing utilities
//! - Typography tokens
//! - Component style builders (buttons, forms, panels, notices, etc)
//! - Utility functions for dynamic styling (dark mode, etc)
//!
//! ## Philosophy
//! Rust-first approach using StyleBuilder for maintainability and type safety.
//! CSS in lotus_styles/ is reserved for global resets/utilities only.

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
    pub const ACCENT: &str = "var(--accent)";
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
    pub const PAGE_BRAND_GAP: &str = "8px 10px";
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
// BUTTON STYLE BUILDERS
// ============================================================================

pub mod buttons {
    //! Reusable button style functions to avoid duplication across components.
    use ui::prelude::*;

    /// Standard button style: inline-flex with border, surface background, and text color.
    /// Used in notices, loading, search panel components.
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
}

pub mod shared {
    //! Legacy module - functions have been moved to more organized modules.
    //! Left for backward compatibility with imports in other files.

    pub use super::forms::input_base_style;
    pub use super::forms::label_base_style;
    pub use super::forms::label_small_style;
    pub use super::forms::hint_text_style;
    pub use super::notices::notice_value_style;
    pub use super::utilities::sr_only_style;
}

pub mod primary_buttons {
    //! Primary action button styles (Search, Add Row, Generate, etc).
    use ui::prelude::*;

    /// Primary button: medium size with primary background, used for search and main actions.
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

    /// Primary button small: compact size for curation actions (Add Row, etc).
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
            .build()
    }

    /// Primary button full width: for block-level actions (Generate QuickStatements, etc).
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

    /// Base button with standard surface background (used for secondary/general actions).
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

    /// Extra-small button for compact UI elements (delete buttons in tables, etc).
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

    /// Filters toggle button: mobile-only button for showing/hiding filters.
    /// Display controlled by CSS media queries (filters-toggle class).
    /// Rust styles handle appearance (colors, sizing, etc) — display is inline-flex like search button.
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
    pub fn button_copy_style() -> String {
        StyleBuilder::new()
            .property("margin-left", "6px")
            .font_family("var(--sans), system-ui, sans-serif")
            .font_weight("500")
            .property("letter-spacing", ".02em")
            .color("var(--text2)")
            .background_color("var(--surface)")
            .border("1px solid var(--border)")
            .cursor("pointer")
            .property(
                "transition",
                "color .15s, background .15s, border-color .15s",
            )
            .property("vertical-align", "baseline")
            .build()
    }
}

// ============================================================================
// PANEL & CONTAINER STYLES
// ============================================================================

pub mod panels {
    //! Panel, card, and container styling for search panels, results, etc.
    use ui::prelude::*;

    /// Search panel container: border with subtle background.
    pub fn search_panel_style() -> String {
        StyleBuilder::new()
            .display("flex")
            .flex_direction("column")
            .gap("12px")
            .padding("12px")
            .background_color("var(--surface)")
            .border_radius("4px")
            .build()
    }

    /// Query summary inline: small font with padding and icon space.
    pub fn query_summary_style() -> String {
        StyleBuilder::new()
            .font_size("var(--fs-0)")
            .padding("8px 14px 8px 32px")
            .property("position", "relative")
            .build()
    }
}

// ============================================================================
// NOTICE & ALERT STYLES
// ============================================================================

pub mod notices {
    //! Notice, alert, and status message styling.
    use ui::prelude::*;

    /// Base notice: flex container with border, padding, and gap.
    pub fn notice_base_style() -> String {
        StyleBuilder::new()
            .display("flex")
            .align_items("flex-start")
            .gap("8px")
            .padding("10px 12px")
            .border_radius("4px")
            .border("1px solid")
            .font_size("var(--fs-0)")
            .build()
    }

    /// Notice in dark mode: dark background with light border and text.
    pub fn notice_dark_style() -> String {
        StyleBuilder::new()
            .display("flex")
            .align_items("flex-start")
            .gap("8px")
            .padding("10px 12px")
            .border_radius("4px")
            .border("1px solid #444")
            .background_color("#1a1a1a")
            .color("#fff")
            .font_size("var(--fs-0)")
            .build()
    }

    /// Notice value text: word-break for long content with proper line height.
    pub fn notice_value_style() -> String {
        StyleBuilder::new()
            .color("inherit")
            .property("word-break", "break-word")
            .property("line-height", "1.4")
            .build()
    }
}

// ============================================================================
// FORM ELEMENT STYLES
// ============================================================================

pub mod forms {
    //! Form input, label, and control styling.
    use ui::prelude::*;

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
}

// ============================================================================
// UTILITY & ACCESSIBILITY STYLES
// ============================================================================

pub mod utilities {
    //! Utility styles for common patterns and accessibility needs.
    use ui::prelude::*;

    /// Screen-reader only text: hidden from visual display but readable by assistive tech.
    pub fn sr_only_style() -> String {
        StyleBuilder::new()
            .property("position", "absolute")
            .property("width", "1px")
            .property("height", "1px")
            .property("padding", "0")
            .property("margin", "-1px")
            .property("overflow", "hidden")
            .property("clip", "rect(0,0,0,0)")
            .property("white-space", "nowrap")
            .property("border", "0")
            .build()
    }
}

// ============================================================================
// DYNAMIC STYLING UTILITIES
// ============================================================================

pub mod header {
    //! Header and page title styling.
    use ui::prelude::*;

    /// Page header container with sticky positioning and bottom border.
    pub fn page_header_style() -> String {
        StyleBuilder::new()
            .display("flex")
            .flex_direction("column")
            .gap("8px")
            .property("padding-left", "max(18px, env(safe-area-inset-left))")
            .property("padding-right", "max(18px, env(safe-area-inset-right))")
            .build()
    }

    /// Page brand section: flex row with title and language switcher.
    pub fn page_brand_style() -> String {
        StyleBuilder::new()
            .display("flex")
            .flex_direction("row")
            .flex_wrap("wrap")
            .align_items("flex-start")
            .gap("8px 10px")
            .build()
    }

    /// Page title: main heading with min-width constraint.
    pub fn page_title_style() -> String {
        StyleBuilder::new()
            .property("min-width", "0")
            .property("flex", "1 1 260px")
            .font_size("var(--fs-4)")
            .property("margin", "0")
            .build()
    }

    /// Page title link: inline-flex with text overflow handling.
    pub fn page_title_link_style() -> String {
        StyleBuilder::new()
            .display("inline-flex")
            .property("max-width", "100%")
            .gap("8px")
            .text_decoration("none")
            .color("inherit")
            .build()
    }

    /// Page title text: proper wrapping and line height.
    pub fn page_title_text_style() -> String {
        StyleBuilder::new()
            .property("line-height", "1.1")
            .property("word-break", "break-word")
            .build()
    }

    /// Page subtitle: secondary color with smaller font.
    pub fn page_subtitle_style() -> String {
        StyleBuilder::new()
            .font_size("var(--fs-1)")
            .property("margin", "0")
            .color("var(--text2)")
            .build()
    }

    /// Archive note section: inline display.
    pub fn page_archive_note_style() -> String {
        StyleBuilder::new()
            .display("inline")
            .build()
    }

    /// Archive label: bold small-caps label.
    pub fn page_archive_label_style() -> String {
        StyleBuilder::new()
            .font_weight("700")
            .property("font-variant", "small-caps")
            .build()
    }

    /// Archive link: accent color, no wrap.
    pub fn page_archive_link_style() -> String {
        StyleBuilder::new()
            .text_decoration("none")
            .color("var(--accent)")
            .property("white-space", "nowrap")
            .build()
    }
}

pub mod table {
    //! Table header and sorting controls.
    use ui::prelude::*;

    /// Table header cell: uppercase label with border and padding.
    pub fn table_header_cell_style() -> String {
        StyleBuilder::new()
            .padding("9px 10px")
            .text_align("left")
            .font_size("var(--fs-label)")
            .font_weight("700")
            .color("var(--critical-muted)")
            .border_bottom("1px solid var(--results-border)")
            .property("white-space", "nowrap")
            .property("user-select", "none")
            .property("text-transform", "uppercase")
            .property("letter-spacing", "0.08em")
            .property("width", "auto")
            .property("min-width", "max-content")
            .build()
    }

    /// Header label text: block display with no-break constraint.
    pub fn header_label_style() -> String {
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
    pub fn sort_button_style() -> String {
        StyleBuilder::new()
            .property("appearance", "none")
            .background_color("transparent")
            .border("0")
            .color("inherit")
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
            .build()
    }

    /// Sort icon: muted color, smaller font.
    pub fn sort_icon_style() -> String {
        StyleBuilder::new()
            .color("var(--text3)")
            .font_size("var(--fs-0)")
            .font_weight("700")
            .property("line-height", "1")
            .build()
    }
}

pub mod theme {
    //! Theme and dynamic styling utilities (dark mode detection, etc).

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
}

// ============================================================================
// ENUM TYPES FOR TYPE-SAFE SELECTION
// ============================================================================

/// Type-safe stat stripe colors (instead of string parameters).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatStripe {
    Compound,
    Taxon,
    Reference,
    Entries,
}

impl StatStripe {
    /// Get the CSS color variable for this stripe.
    pub fn as_color(&self) -> &'static str {
        match self {
            Self::Compound => stat_stripe_colors::COMPOUND,
            Self::Taxon => stat_stripe_colors::TAXON,
            Self::Reference => stat_stripe_colors::REFERENCE,
            Self::Entries => stat_stripe_colors::ENTRIES,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StatStripe;
    use crate::ui::style_constants::{stat_stripe_colors, spacing};

    #[test]
    fn stat_stripe_colors_are_nonempty() {
        assert!(!stat_stripe_colors::COMPOUND.is_empty());
        assert!(!stat_stripe_colors::TAXON.is_empty());
        assert!(!stat_stripe_colors::REFERENCE.is_empty());
        assert!(!stat_stripe_colors::ENTRIES.is_empty());
    }

    #[test]
    fn spacing_tokens_parse() {
        // Verify format is valid for CSS
        assert!(spacing::STAT_BADGE_GAP.contains("px"));
    }

    #[test]
    fn stat_stripe_enum_colors_match() {
        assert_eq!(
            StatStripe::Compound.as_color(),
            stat_stripe_colors::COMPOUND
        );
        assert_eq!(StatStripe::Taxon.as_color(), stat_stripe_colors::TAXON);
        assert_eq!(
            StatStripe::Reference.as_color(),
            stat_stripe_colors::REFERENCE
        );
        assert_eq!(StatStripe::Entries.as_color(), stat_stripe_colors::ENTRIES);
    }
}
