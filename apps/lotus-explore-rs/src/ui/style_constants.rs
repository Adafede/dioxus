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
    pub const PANEL_BORDER: &str = "var(--panel-border)";
    pub const RESULTS_BORDER: &str = "var(--results-border)";
}

pub mod backgrounds {
    //! Background color tokens.
    pub const SURFACE: &str = "var(--surface)";
    pub const PANEL_BG_SOFT: &str = "var(--panel-bg-soft)";
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
    pub const FORM_SECTION: &str = "10px 12px";
    pub const STAT_BADGE_PAD: &str = "10px 12px";
    pub const STAT_VALUE_GAP: &str = "8px";
    pub const STAT_BADGE_GAP: &str = "4px";
    pub const STAT_BAR_GAP: &str = "10px";
    pub const BUTTON_XS_PAD: &str = "2px 8px";
    pub const PAGE_HEADER_MARGIN: &str = "10px 22px 0";
    pub const PAGE_HEADER_PAD: &str = "10px 12px";
    pub const NESTED_SECTION: &str = "10px";
    pub const HEADER_META_GAP: &str = "12px";
    pub const HEADER_META_GAP_SMALL: &str = "4px";
    pub const QUERY_SUMMARY_PADDING: &str = "8px 14px 8px 32px";
}

// ============================================================================
// SIZE TOKENS
// ============================================================================

pub mod sizes {
    //! Size tokens for borders, corners, and dimensions.
    pub const BORDER_RADIUS: &str = "var(--radius)";
    pub const BORDER_RADIUS_SM: &str = "var(--radius-sm)";
    pub const BORDER_RADIUS_LG: &str = "12px";
    pub const BORDER_WIDTH: &str = "1px";
    pub const BORDER_WIDTH_STRIPE: &str = "3px";
}

// ============================================================================
// TYPOGRAPHY TOKENS
// ============================================================================

pub mod typography {
    //! Font size and weight tokens.
    pub const FONT_SIZE_STAT: &str = "var(--fs-stat)";
    pub const FONT_SIZE_0: &str = "var(--fs-0)";
    pub const FONT_SIZE_LABEL: &str = "var(--fs-label)";
    pub const FONT_SIZE_MICRO: &str = "var(--fs-micro)";
    pub const FONT_WEIGHT_BOLD: &str = "800";
    pub const FONT_WEIGHT_SEMIBOLD: &str = "700";
    pub const LETTER_SPACING_TITLE: &str = "0.08em";
    pub const LETTER_SPACING_STAT: &str = "-0.02em";
    pub const LINE_HEIGHT_STAT: &str = "1.2";
}

// ============================================================================
// TRANSITION TOKENS
// ============================================================================

pub mod transitions {
    //! Transition timing and easing.
    pub const PANEL_TRANSITION: &str =
        "background .15s ease, border-color .15s ease, box-shadow .15s ease";
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
        assert!(spacing::FORM_SECTION.contains("px"));
        assert!(spacing::STAT_BADGE_GAP.contains("px"));
    }

    #[test]
    fn stat_stripe_enum_colors_match() {
        assert_eq!(StatStripe::Compound.as_color(), stat_stripe_colors::COMPOUND);
        assert_eq!(StatStripe::Taxon.as_color(), stat_stripe_colors::TAXON);
        assert_eq!(StatStripe::Reference.as_color(), stat_stripe_colors::REFERENCE);
        assert_eq!(StatStripe::Entries.as_color(), stat_stripe_colors::ENTRIES);
    }
}
