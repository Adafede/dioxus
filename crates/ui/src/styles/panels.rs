// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Shared panel and container styling system for layout structures.
//! Provides consistent styling across all apps for maximum reuse.

use super::primitives::*;
use crate::theme::StyleBuilder;

/// Search panel container: border with subtle background.
pub fn search_panel_style() -> String {
    format!(
        "{}; {}; {}",
        flex_column(),
        gap_md(),
        StyleBuilder::new()
            .padding("12px")
            .background_color("var(--panel-bg-soft)")
            .border_radius("4px")
            .build()
    )
}

/// Query summary inline: small font with padding and icon space.
pub fn query_summary_style() -> String {
    format!(
        "{}; {}",
        font_size_xs(),
        StyleBuilder::new()
            .padding("8px 14px 8px 32px")
            .property("position", "relative")
            .build()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_panel_is_flex_column() {
        let style = search_panel_style();
        assert!(style.contains("flex") && style.contains("column"));
    }

    #[test]
    fn query_summary_has_position_relative() {
        let style = query_summary_style();
        assert!(style.contains("position") && style.contains("relative"));
    }
}
