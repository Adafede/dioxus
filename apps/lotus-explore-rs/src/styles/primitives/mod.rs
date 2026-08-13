// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Atomic, single-concern style primitives for lotus-explore-rs UI.
//!
//! This module provides fine-grained, reusable CSS building blocks that can be
//! composed together to create complex component styles. Each primitive handles
//! a single concern (layout, spacing, typography, etc.) and returns a CSS string.
//!
//! Primitives use design tokens from this crate's token system for consistency
//! with the lotus-explore-rs design language.

pub mod interaction;
pub mod layout;
pub mod spacing;
pub mod typography;
pub mod visual;

pub use interaction::*;
pub use layout::*;
pub use spacing::*;
pub use typography::*;
pub use visual::*;

// ============================================================================
// COMPOSITION HELPERS
// ============================================================================

/// Button base: flex center + border + padding + cursor
pub fn button_base() -> String {
    format!(
        "{}; {}; {}; {}; {}",
        layout::flex_center(),
        spacing::padding_button(),
        visual::border_default(),
        visual::border_radius_sm(),
        interaction::cursor_pointer()
    )
}

/// Input base: surface bg + border + padding + text color
pub fn input_base() -> String {
    format!(
        "{}; {}; {}; {}; {}",
        visual::bg_surface(),
        visual::border_default(),
        visual::border_radius_sm(),
        spacing::padding_input(),
        typography::text_color_primary()
    )
}

/// Cell base: padding + text color
pub fn cell_base() -> String {
    format!(
        "{}; {}",
        spacing::padding_cell(),
        typography::text_color_primary()
    )
}

/// Panel base: border + panel bg + padding + shadow
pub fn panel_base() -> String {
    format!(
        "{}; {}; {}; {}",
        visual::border_default(),
        visual::bg_panel(),
        spacing::padding_md(),
        visual::shadow_sm()
    )
}

/// Label base: bold + font size + primary color
pub fn label_base() -> String {
    format!(
        "{}; {}; {}",
        typography::font_weight_bold(),
        typography::font_size_0(),
        typography::text_color_primary()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_base_composes_primitives() {
        let style = button_base();
        assert!(style.contains("display"));
        assert!(style.contains("flex"));
        assert!(style.contains("border"));
        assert!(style.contains("cursor"));
    }

    #[test]
    fn input_base_composes_primitives() {
        let style = input_base();
        assert!(style.contains("border"));
        assert!(style.contains("padding"));
        assert!(style.contains("background"));
    }

    #[test]
    fn panel_base_composes_primitives() {
        let style = panel_base();
        assert!(style.contains("border"));
        assert!(style.contains("background"));
        assert!(style.contains("padding"));
    }
}
