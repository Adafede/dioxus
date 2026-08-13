// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Shared table cell styling system for all apps.
//!
//! Provides base cell styles that can be extended with app-specific variants.
//! All cells use consistent padding, borders, and flex layout.

use crate::theme::StyleBuilder;

/// Base table cell: flex container with standard padding and border.
/// Used as foundation for all cell variants.
pub fn cell_default() -> String {
    StyleBuilder::new()
        .padding("8px 12px")
        .border_radius("4px")
        .background_color("var(--surface)")
        .border("1px solid var(--border)")
        .property("min-width", "0")
        .build()
}

/// Cell text: block display with word wrapping for content overflow.
pub fn cell_text() -> String {
    StyleBuilder::new()
        .display("block")
        .property("line-height", "1.4")
        .property("overflow-wrap", "break-word")
        .property("word-break", "break-word")
        .property("white-space", "normal")
        .build()
}

/// Cell badge: inline-block with monospace font for IDs.
pub fn cell_badge() -> String {
    StyleBuilder::new()
        .display("inline-block")
        .font_size("var(--fs-micro)")
        .padding("1px 5px")
        .border_radius("3px")
        .font_weight("600")
        .font_family("var(--mono)")
        .property("line-height", "1.5")
        .property("white-space", "nowrap")
        .build()
}

/// Cell row with badges: flex wrap with gap for multiple badges.
pub fn cell_badge_row() -> String {
    StyleBuilder::new()
        .display("flex")
        .property("flex-wrap", "wrap")
        .gap("4px")
        .property("margin-top", "4px")
        .property("min-width", "0")
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_default_has_padding() {
        let style = cell_default();
        assert!(style.contains("padding"));
    }

    #[test]
    fn cell_text_is_block_display() {
        let style = cell_text();
        assert!(style.contains("display"));
        assert!(style.contains("block"));
    }

    #[test]
    fn cell_badge_is_inline_block() {
        let style = cell_badge();
        assert!(style.contains("inline-block"));
    }

    #[test]
    fn cell_badge_row_uses_flex_wrap() {
        let style = cell_badge_row();
        assert!(style.contains("flex-wrap"));
    }
}
