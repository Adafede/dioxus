// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Lightweight accessibility smoke tests.
//!
//! These tests guard critical ARIA/landmark contracts in source markup so
//! regressions are detected early during refactors.

#[cfg(test)]
mod tests {
    #[test]
    fn main_landmark_is_labelled_and_skip_link_targets_it() {
        let main_src = include_str!("../main.rs");
        assert!(main_src.contains("href: SKIP_TO_RESULTS_HREF"));
        assert!(main_src.contains("id: MAIN_PANEL_ID"));
        assert!(main_src.contains("aria_labelledby: PAGE_TITLE_ID"));
    }

    #[test]
    fn sidebar_filter_button_exposes_expanded_and_controls_relation() {
        let sidebar_src = include_str!("../components/layout/sidebar.rs");
        assert!(sidebar_src.contains("aria_controls: SEARCH_PANEL_BODY_ID"));
        assert!(sidebar_src.contains("aria_expanded:"));
        assert!(sidebar_src.contains("aria_labelledby: SEARCH_PANEL_HEADING_ID"));
    }

    #[test]
    fn sortable_headers_expose_action_oriented_aria_label() {
        let header_src = include_str!("../components/results_table/table_header.rs");
        assert!(header_src.contains("aria_sort_toggle"));
        assert!(header_src.contains("aria_label: \"{sort_aria}\""));
    }

    #[test]
    fn page_header_exposes_single_home_link_and_heading_id() {
        let header_src = include_str!("../components/layout/page_header.rs");
        assert!(header_src.contains("h1 { id: PAGE_TITLE_ID"));
        assert!(header_src.contains("class: \"page-title-link page-home-link\""));
        assert!(header_src.contains("aria_label: \"{t(locale, TextKey::GoToHomepage)}\""));
    }
}
