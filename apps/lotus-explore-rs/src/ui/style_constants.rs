// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! LEGACY MODULE - Deprecated in favor of organized submodules
//!
//! This module is kept for backward compatibility with existing code.
//! New code should import from specific submodules:
//!
//! **Reusable Generic Styles (from crates/ui):**
//! ```ignore
//! use ui::styles::{button_primary_style, input_base_style, notice_base_style};
//! ```
//!
//! **Lotus-Specific Styles:**
//! ```ignore
//! use crate::ui::layout_styles;
//! use crate::ui::table_styles;
//! use crate::ui::search_controls;
//! ```
//!
//! ## Module Organization
//! - **layout_styles.rs** - Page headers, tables, stats, queries, panels, downloads
//! - **table_styles.rs** - Table cells, taxon/reference formatting, ID badges
//! - **search_controls.rs** - Search form controls, radio groups, textareas, pills

// ============================================================================
// DIRECT RE-EXPORTS (Generic UI - from crates/ui)
// ============================================================================

// ============================================================================
// SUBMODULE ALIASES FOR BACKWARD COMPATIBILITY
// ============================================================================

/// Backward compat: Re-exports from layout_styles for table header and sort controls
pub mod table {
    pub use crate::ui::layout_styles::{
        lotus_header_label_style as header_label_style,
        lotus_sort_button_style as sort_button_style, lotus_sort_icon_style as sort_icon_style,
        lotus_table_header_cell_style as table_header_cell_style,
    };
}

/// Backward compat: Re-exports from table_styles for table cell styling
pub mod table_cells {
    pub use crate::ui::table_styles::{
        lotus_badge_row_style as badge_row_style, lotus_cell_primary_style as cell_primary_style,
        lotus_id_badge_style as id_badge_style, lotus_na_style as na_style,
        lotus_primary_link_style as primary_link_style,
        lotus_reference_cell_style as reference_cell_style,
        lotus_table_cell_style as table_cell_style, lotus_taxon_cell_style as taxon_cell_style,
    };
}

/// Backward compat: Re-exports from layout_styles for downloads
pub mod downloads {
    pub use crate::ui::layout_styles::{
        lotus_button_small_style as button_small_style, lotus_dl_group_style as dl_group_style,
        lotus_spinner_sm_style as spinner_sm_style,
        lotus_toolbar_actions_style as toolbar_actions_style,
    };
}

/// Backward compat: Re-exports from layout_styles for button styles
pub mod buttons {
    pub use ui::styles::button_base_style;
    pub use ui::styles::button_transparent_style;
}

/// Backward compat: Re-exports from layout_styles for stats
pub mod stats {
    use super::StatStripe;
    use crate::ui::layout_styles::{
        lotus_stat_badge_style, lotus_stat_bar_style, lotus_stat_label_style,
        lotus_stat_secondary_style, lotus_stat_value_row_style, lotus_stat_value_style,
    };

    /// Stat badge wrapper that accepts StatStripe enum
    pub fn stat_badge_style(stripe: StatStripe) -> String {
        lotus_stat_badge_style(stripe.as_color())
    }

    pub fn stat_bar_style() -> String {
        lotus_stat_bar_style()
    }

    pub fn stat_label_style() -> String {
        lotus_stat_label_style()
    }

    pub fn stat_secondary_style() -> String {
        lotus_stat_secondary_style()
    }

    pub fn stat_value_row_style() -> String {
        lotus_stat_value_row_style()
    }

    pub fn stat_value_style() -> String {
        lotus_stat_value_style()
    }
}

/// Backward compat: Re-exports from layout_styles for search controls
pub mod search_buttons {
    pub use crate::ui::layout_styles::lotus_search_button_state as search_button_state;
}

/// Backward compat: Re-exports from layout_styles for panel containers
pub mod panel_containers {
    pub use crate::ui::layout_styles::{
        lotus_iframe_style as iframe_style, lotus_ketcher_panel_style as ketcher_panel_style,
        lotus_ketcher_wrap_style as ketcher_wrap_style,
        lotus_section_card_style as section_card_style,
    };
}

/// Backward compat: Re-exports from search_controls for form controls
pub mod search_controls {
    pub use crate::ui::search_controls::{
        lotus_kind_pill_style as kind_pill_style, lotus_radio_group_style as radio_group_style,
        lotus_radio_label_style as radio_label_style, lotus_range_input_style as range_input_style,
        lotus_textarea_base_style as textarea_base_style,
        lotus_threshold_section_style as threshold_section_style,
    };
}

/// Backward compat: Re-exports from layout_styles for header
pub mod header {
    pub use crate::ui::layout_styles::{
        lotus_page_archive_label_style as page_archive_label_style,
        lotus_page_archive_link_style as page_archive_link_style,
        lotus_page_archive_note_style as page_archive_note_style,
        lotus_page_brand_style as page_brand_style, lotus_page_header_style as page_header_style,
        lotus_page_subtitle_style as page_subtitle_style,
        lotus_page_title_link_style as page_title_link_style,
        lotus_page_title_style as page_title_style,
        lotus_page_title_text_style as page_title_text_style,
    };
}

/// Backward compat: Re-exports from layout_styles for query panel
pub mod query {
    pub use crate::ui::layout_styles::{
        lotus_query_body_style as query_body_style, lotus_query_panel_style as query_panel_style,
        lotus_query_summary_chevron_style as query_summary_chevron_style,
        lotus_query_summary_toggle_style as query_summary_style,
        lotus_query_text_style as query_text_style,
    };
}

/// Backward compat: Re-exports from layout_styles for theme
pub mod theme {
    pub use crate::ui::layout_styles::is_dark_mode;
}

/// Backward compat: Shared utilities
pub mod shared {
    pub use ui::styles::{
        hint_text_style, input_base_style, label_base_style, label_small_style, notice_value_style,
        sr_only_style,
    };
}

// ============================================================================
// ADDITIONAL BACKWARD COMPAT MODULES
// ============================================================================

/// Backward compat: Notices (generic UI)
pub mod notices {
    pub use ui::styles::{notice_dismiss_style, notice_value_style, share_input_style};
}

/// Backward compat: Primary buttons (generic UI)
pub mod primary_buttons {
    pub use ui::styles::{
        button_filters_toggle_style, button_primary_block_style, button_primary_sm_style,
        button_primary_style, button_sm_style, button_xs_style,
    };
}

/// Backward compat: Forms (generic UI)
pub mod forms {
    pub use ui::styles::{
        form_section_style, hint_text_style, input_base_style, label_base_style, label_small_style,
        range_inputs_style, range_pair_style,
    };
}

/// Backward compat: Panels (generic UI)
pub mod panels {
    pub use ui::styles::search_panel_style;
}

/// Backward compat: Utilities (accessibility)
pub mod utilities {
    pub use ui::styles::sr_only_style;
}

// ============================================================================
// TOKEN MODULES
// ============================================================================

pub use crate::ui::layout_styles::{spacing, stat_stripe_colors};

// ============================================================================
// TYPE-SAFE ENUMS
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
    fn stat_stripe_enum_compound_matches_constant() {
        assert_eq!(
            StatStripe::Compound.as_color(),
            stat_stripe_colors::COMPOUND
        );
    }

    #[test]
    fn stat_stripe_enum_taxon_matches_constant() {
        assert_eq!(StatStripe::Taxon.as_color(), stat_stripe_colors::TAXON);
    }

    #[test]
    fn stat_stripe_enum_reference_matches_constant() {
        assert_eq!(
            StatStripe::Reference.as_color(),
            stat_stripe_colors::REFERENCE
        );
    }

    #[test]
    fn stat_stripe_enum_entries_matches_constant() {
        assert_eq!(StatStripe::Entries.as_color(), stat_stripe_colors::ENTRIES);
    }
}
