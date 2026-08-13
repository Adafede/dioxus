// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Shared UI styling system for all apps in the monorepo.
//!
//! This module provides reusable style builders for:
//! - **Buttons**: primary, secondary, copy, small, extra-small variants
//! - **Forms**: inputs, labels, textareas, hints, form sections
//! - **Notices**: alerts, notifications, dismissible messages
//! - **Panels**: containers, cards, search panels, query summaries
//! - **Theme**: utilities, accessibility, typography
//!
//! ## Usage
//!
//! ```ignore
//! use ui::styles::{buttons, forms, notices, panels, theme};
//!
//! let button_style = buttons::button_primary_style();
//! let input_style = forms::input_base_style();
//! let notice_style = notices::notice_base_style();
//! let panel_style = panels::search_panel_style();
//! let sr_only = theme::sr_only_style();
//! ```
//!
//! ## Design Principles
//!
//! - All buttons use consistent 40px minimum height for touch accessibility
//! - Padding is adjusted per size variant to maintain aspect ratios
//! - Styles are CSS-in-Rust using `StyleBuilder` for maintainability
//! - No external CSS dependencies for core styling
//! - Each module is independently testable and reusable

pub mod buttons;
pub mod forms;
pub mod notices;
pub mod panels;
pub mod theme;

// Re-export commonly used items for convenience
pub use buttons::{
    button_base_style, button_copy_style, button_filters_toggle_style, button_primary_block_style,
    button_primary_sm_style, button_primary_style, button_sm_style, button_transparent_style,
    button_xs_style,
};

pub use forms::{
    form_section_style, hint_text_style, input_base_style, label_base_style, label_small_style,
    range_inputs_style, range_pair_style, share_input_style,
};

pub use notices::{notice_base_style, notice_dark_style, notice_dismiss_style, notice_value_style};

pub use panels::{query_summary_style, search_panel_style};

pub use theme::sr_only_style;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_button_styles_return_non_empty_strings() {
        assert!(!button_base_style().is_empty());
        assert!(!button_primary_style().is_empty());
        assert!(!button_copy_style().is_empty());
        assert!(!button_sm_style().is_empty());
        assert!(!button_xs_style().is_empty());
    }

    #[test]
    fn all_form_styles_return_non_empty_strings() {
        assert!(!input_base_style().is_empty());
        assert!(!label_base_style().is_empty());
        assert!(!form_section_style().is_empty());
    }

    #[test]
    fn all_notice_styles_return_non_empty_strings() {
        assert!(!notice_base_style().is_empty());
        assert!(!notice_value_style().is_empty());
        assert!(!notice_dismiss_style().is_empty());
    }

    #[test]
    fn all_panel_styles_return_non_empty_strings() {
        assert!(!search_panel_style().is_empty());
        assert!(!query_summary_style().is_empty());
    }

    #[test]
    fn utility_styles_return_non_empty_strings() {
        assert!(!sr_only_style().is_empty());
    }
}
