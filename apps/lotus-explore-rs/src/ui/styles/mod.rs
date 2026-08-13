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
pub mod panels;
pub mod search;
pub mod table;

// Re-export commonly used items for convenience
pub use buttons::{button_primary_lotus_search, button_small_lotus, button_xs_lotus_table};

pub use cells::{
    cell_badges, cell_default, cell_link, cell_na, cell_reference, cell_reference_badges,
    cell_reference_id, cell_taxon, cell_taxon_id, cell_taxon_primary,
};

pub use panels::{panel_iframe, panel_ketcher, panel_ketcher_wrap, panel_section_card};

pub use search::{
    search_kind_pill, search_radio_group, search_radio_label, search_range_input, search_textarea,
    search_threshold_section,
};

pub use table::{table_header_cell, table_header_label, table_sort_button, table_sort_icon};

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
