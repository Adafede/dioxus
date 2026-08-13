// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Lotus-specific button styling.
//! Extends base button styles with lotus-specific colors and variants.
//!
//! Naming hierarchy: `button_<type>_lotus_<context>()`
//! Examples: `button_primary_lotus_search()`, `button_small_lotus_filter()`

use ui::styles::buttons;

/// Download group buttons: compact styled buttons in action toolbar.
pub fn button_small_lotus() -> String {
    buttons::button_sm_style()
}

/// Primary button for search actions.
pub fn button_primary_lotus_search() -> String {
    buttons::button_primary_style()
}

/// Primary button for filter actions (compact).
#[allow(dead_code)]
pub fn button_small_lotus_filter() -> String {
    buttons::button_sm_style()
}

/// Extra-small button for table actions (delete, etc).
pub fn button_xs_lotus_table() -> String {
    buttons::button_xs_style()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_small_lotus_not_empty() {
        assert!(!button_small_lotus().is_empty());
    }

    #[test]
    fn button_primary_lotus_search_not_empty() {
        assert!(!button_primary_lotus_search().is_empty());
    }

    #[test]
    fn button_xs_lotus_table_not_empty() {
        assert!(!button_xs_lotus_table().is_empty());
    }
}
