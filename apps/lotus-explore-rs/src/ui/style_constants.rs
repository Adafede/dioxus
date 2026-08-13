// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Centralized UI style constants for colors, spacing, sizing, and typography.
//!
//! This module consolidates all magic values used across components, making it
//! easier to maintain consistency and update design tokens in one place.

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
    //! Shared utility styles used across multiple components.
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

    /// Notice value text: word-break for long content with proper line height.
    pub fn notice_value_style() -> String {
        StyleBuilder::new()
            .color("inherit")
            .property("word-break", "break-word")
            .property("line-height", "1.4")
            .build()
    }
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
    use super::*;

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
