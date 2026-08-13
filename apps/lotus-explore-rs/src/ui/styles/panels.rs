// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Lotus-specific panel and container styling.

use super::tokens;
use ui::prelude::*;

/// Section card container: flex column with padding and border.
#[allow(dead_code)]
pub fn panel_section_card() -> String {
    StyleBuilder::new()
        .display("flex")
        .flex_direction("column")
        .gap(tokens::SPACING_MD)
        .padding(&format!("{} {}", tokens::SPACING_MD, "14px"))
        .border("1px solid var(--panel-border)")
        .border_radius("8px")
        .background_color("var(--panel-bg-soft)")
        .build()
}

/// Ketcher panel wrapper: container for chemical structure editor.
#[allow(dead_code)]
pub fn panel_ketcher_wrap() -> String {
    StyleBuilder::new()
        .display("flex")
        .flex_direction("column")
        .border("1px solid var(--border)")
        .border_radius(tokens::BORDER_RADIUS_SM)
        .property("overflow", "hidden")
        .build()
}

/// Ketcher panel style: embeds Ketcher editor with proper sizing.
#[allow(dead_code)]
pub fn panel_ketcher() -> String {
    StyleBuilder::new()
        .property("width", "100%")
        .property("height", "400px")
        .build()
}

/// iframe style: full width and height.
#[allow(dead_code)]
pub fn panel_iframe() -> String {
    StyleBuilder::new()
        .property("width", "100%")
        .property("height", "100%")
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panel_section_card_is_flex_column() {
        let style = panel_section_card();
        assert!(style.contains("flex-direction"));
        assert!(style.contains("column"));
    }

    #[test]
    fn panel_ketcher_wrap_has_flex_column() {
        let style = panel_ketcher_wrap();
        assert!(style.contains("flex-direction"));
    }

    #[test]
    fn panel_ketcher_has_height() {
        let style = panel_ketcher();
        assert!(style.contains("height"));
    }
}
