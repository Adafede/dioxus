// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Lotus-specific styling with hierarchical naming.
//!
//! All modules follow composition-first design:
//! - Extend generic base styles from `ui::styles`
//! - Apply only lotus-specific overrides
//! - Use hierarchical naming: specific→general, not general→specific
//!
//! Naming convention:
//! - `<element>_<type>()` for components
//! - `<element>_<type>_lotus_<context>()` for app-specific variants
//!
//! Examples:
//! ```ignore
//! use crate::ui::styles::{cells, buttons, panels, search, table};
//!
//! // Taxon cell with lotus styling
//! let style = cells::cell_taxon();
//!
//! // Search button
//! let style = buttons::button_primary_lotus_search();
//! ```

pub mod buttons;
pub mod cells;
pub mod curation;
pub mod panels;
pub mod search;
pub mod table;
pub mod tokens;

// Re-export commonly used items for convenience
pub use buttons::{button_primary_lotus_search, button_small_lotus, button_xs_lotus_table};

#[allow(unused_imports)]
pub use cells::{
    cell_badges, cell_default, cell_link, cell_na, cell_reference, cell_reference_badges,
    cell_reference_id, cell_taxon, cell_taxon_id, cell_taxon_primary,
};

#[allow(unused_imports)]
pub use panels::{panel_iframe, panel_ketcher, panel_ketcher_wrap, panel_section_card};

#[allow(unused_imports)]
pub use search::{
    search_kind_pill, search_radio_group, search_radio_label, search_range_input, search_textarea,
    search_threshold_section,
};

#[allow(unused_imports)]
pub use table::{table_header_cell, table_header_label, table_sort_button, table_sort_icon};

// Re-export tokens for convenient access throughout the app
#[allow(unused_imports)]
pub use tokens::{
    BORDER_RADIUS_LG, BORDER_RADIUS_MD, BORDER_RADIUS_SM, BORDER_WIDTH, BUTTON_MIN_HEIGHT,
    BUTTON_MIN_HEIGHT_SM, BUTTON_MIN_HEIGHT_XS, BUTTON_PADDING_X, BUTTON_PADDING_Y, CELL_MAX_WIDTH,
    CELL_MIN_WIDTH, CELL_PADDING_X, CELL_PADDING_Y, COLOR_BORDER, COLOR_BORDER_RESULTS,
    COLOR_PRIMARY, COLOR_SURFACE, COLOR_TEXT, COLOR_TEXT_SECONDARY, COPY_BUTTON_MIN_HEIGHT,
    COPY_BUTTON_PADDING_X, COPY_BUTTON_PADDING_Y, FONT_SIZE_RESPONSIVE_BASE,
    FONT_SIZE_RESPONSIVE_BUTTON, FONT_SIZE_RESPONSIVE_MICRO, FONT_SIZE_RESPONSIVE_SMALL, GAP_LG,
    GAP_MD, GAP_SM, GAP_XL, GAP_XS, INPUT_MIN_HEIGHT, INPUT_PADDING_X, INPUT_PADDING_Y,
    PADDING_RESPONSIVE_X, PADDING_RESPONSIVE_Y, ROW_HEIGHT_LG, ROW_HEIGHT_MD, ROW_HEIGHT_SM,
    ROW_HEIGHT_XL, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XL, SPACING_XS, STAT_ROW_HEIGHT,
    TABLE_COLUMN_GAP, TABLE_ROW_GAP, button_copy_padding, button_padding, cell_padding,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_cell_functions_return_non_empty_strings() {
        assert!(!cell_taxon().is_empty());
        assert!(!cell_taxon_primary().is_empty());
        assert!(!cell_taxon_id().is_empty());
        assert!(!cell_reference().is_empty());
        assert!(!cell_reference_id().is_empty());
    }

    #[test]
    fn all_button_functions_return_non_empty_strings() {
        assert!(!button_small_lotus().is_empty());
        assert!(!button_primary_lotus_search().is_empty());
        assert!(!button_xs_lotus_table().is_empty());
    }

    #[test]
    fn all_panel_functions_return_non_empty_strings() {
        assert!(!panel_section_card().is_empty());
        assert!(!panel_ketcher_wrap().is_empty());
        assert!(!panel_ketcher().is_empty());
    }

    #[test]
    fn all_search_functions_return_non_empty_strings() {
        assert!(!search_radio_group().is_empty());
        assert!(!search_textarea().is_empty());
        assert!(!search_kind_pill().is_empty());
    }

    #[test]
    fn all_table_functions_return_non_empty_strings() {
        assert!(!table_header_cell().is_empty());
        assert!(!table_sort_button().is_empty());
    }
}
