// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Component-specific style compositions.

pub mod buttons;
pub mod cells;
pub mod curation;
pub mod panels;
pub mod search;
pub mod table;

pub use buttons::*;
pub use cells::*;
pub use curation::*;
pub use panels::*;
pub use search::*;
pub use table::*;

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
