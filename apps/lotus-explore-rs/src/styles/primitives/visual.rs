// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Visual primitives: borders, shadows, backgrounds, radius.

use super::super::tokens::*;
use ui::theme::StyleBuilder;

// ============================================================================
// BACKGROUND PRIMITIVES
// ============================================================================

/// Surface background: `background-color: var(--surface)`
pub fn bg_surface() -> String {
    StyleBuilder::new().background_color(COLOR_SURFACE).build()
}

/// Panel background: `background-color: var(--panel-bg)`
pub fn bg_panel() -> String {
    StyleBuilder::new()
        .background_color("var(--panel-bg)")
        .build()
}

/// Primary button background: `background-color: var(--btn-primary-bg)`
pub fn bg_primary() -> String {
    StyleBuilder::new().background_color(COLOR_PRIMARY).build()
}

/// Transparent background: `background-color: transparent`
pub fn bg_transparent() -> String {
    StyleBuilder::new().background_color("transparent").build()
}

// ============================================================================
// BORDER PRIMITIVES
// ============================================================================

/// Default border: `border: 1px solid var(--border)`
pub fn border_default() -> String {
    StyleBuilder::new()
        .border(&format!("{} solid {}", BORDER_WIDTH, COLOR_BORDER))
        .build()
}

/// No border: `border: none`
pub fn border_none() -> String {
    StyleBuilder::new().border("none").build()
}

/// Small border radius from tokens
pub fn border_radius_sm() -> String {
    StyleBuilder::new().border_radius(BORDER_RADIUS_SM).build()
}

/// Medium border radius from tokens
pub fn border_radius_md() -> String {
    StyleBuilder::new().border_radius(BORDER_RADIUS_MD).build()
}

/// Large border radius from tokens
pub fn border_radius_lg() -> String {
    StyleBuilder::new().border_radius(BORDER_RADIUS_LG).build()
}

// ============================================================================
// SHADOW PRIMITIVES
// ============================================================================

/// Extra-small shadow: `box-shadow: var(--shadow-xs)`
pub fn shadow_xs() -> String {
    StyleBuilder::new().box_shadow("var(--shadow-xs)").build()
}

/// Small shadow: `box-shadow: var(--shadow-sm)`
pub fn shadow_sm() -> String {
    StyleBuilder::new().box_shadow("var(--shadow-sm)").build()
}

/// Medium shadow: `box-shadow: var(--shadow-md)`
pub fn shadow_md() -> String {
    StyleBuilder::new().box_shadow("var(--shadow-md)").build()
}

/// No shadow: `box-shadow: none`
pub fn shadow_none() -> String {
    StyleBuilder::new().box_shadow("none").build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bg_surface_uses_variable() {
        assert!(bg_surface().contains("--surface"));
    }

    #[test]
    fn border_default_contains_border() {
        assert!(border_default().contains("border"));
        assert!(border_default().contains("1px"));
    }

    #[test]
    fn shadow_sm_contains_shadow() {
        assert!(shadow_sm().contains("box-shadow"));
        assert!(shadow_sm().contains("--shadow-sm"));
    }
}
