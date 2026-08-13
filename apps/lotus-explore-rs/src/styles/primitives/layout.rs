// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Layout primitives: flex, grid, positioning.

use super::super::tokens::*;
use ui::theme::StyleBuilder;

/// Flex row container: `display: flex; flex-direction: row`
pub fn flex_row() -> String {
    StyleBuilder::new()
        .display("flex")
        .flex_direction("row")
        .build()
}

/// Flex column container: `display: flex; flex-direction: column`
pub fn flex_column() -> String {
    StyleBuilder::new()
        .display("flex")
        .flex_direction("column")
        .build()
}

/// Flex row with centered items: flex row + align center + justify center
pub fn flex_center() -> String {
    StyleBuilder::new()
        .display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .build()
}

/// Flex column with centered items: flex column + align center + justify center
pub fn flex_center_column() -> String {
    StyleBuilder::new()
        .display("flex")
        .flex_direction("column")
        .align_items("center")
        .justify_content("center")
        .build()
}

/// Grid container: `display: grid`
pub fn grid() -> String {
    StyleBuilder::new().display("grid").build()
}

/// Absolute positioning base: `position: absolute`
pub fn absolute() -> String {
    StyleBuilder::new().property("position", "absolute").build()
}

/// Relative positioning base: `position: relative`
pub fn relative() -> String {
    StyleBuilder::new().property("position", "relative").build()
}

/// Sticky positioning base: `position: sticky`
pub fn sticky() -> String {
    StyleBuilder::new().property("position", "sticky").build()
}

/// Full size: `width: 100%; height: 100%`
pub fn full_size() -> String {
    StyleBuilder::new()
        .property("width", "100%")
        .property("height", "100%")
        .build()
}

/// Full width: `width: 100%`
pub fn full_width() -> String {
    StyleBuilder::new().property("width", "100%").build()
}

/// Full height: `height: 100%`
pub fn full_height() -> String {
    StyleBuilder::new().property("height", "100%").build()
}

/// Flex row with center alignment and gap: flex center row + gap
pub fn flex_center_row_with_gap() -> String {
    format!("{}; {}", flex_center(), gap_sm())
}

/// Flex column with gap
pub fn flex_column_with_gap() -> String {
    format!("{}; {}", flex_column(), gap_sm())
}

// Re-export from spacing
use super::spacing::gap_sm;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flex_center_contains_display() {
        assert!(flex_center().contains("display"));
        assert!(flex_center().contains("flex"));
    }

    #[test]
    fn flex_center_contains_align_items() {
        assert!(flex_center().contains("align-items"));
        assert!(flex_center().contains("center"));
    }
}
