// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Consolidated lotus-explore-rs styles module.
//!
//! This module unifies style definitions previously split across `src/lotus_styles/`
//! and `src/ui/styles/`, organizing them by concern:
//!
//! - **`tokens`** — Centralized design tokens (colors, spacing, typography, etc.)
//! - **`primitives`** — Atomic StyleBuilder functions (flex, padding, text color, etc.)
//! - **`components`** — Higher-level component styles (buttons, cells, panels, etc.)
//! - **`layout`** — Application shell and frame layout
//! - **`accessibility`** — Focus indicators, keyboard nav, motion preferences
//! - **`responsive`** — Media query breakpoints and responsive overrides
//!
//! # Composition
//!
//! All styles are composed into a single CSS bundle via [`bundled_lotus_styles()`],
//! which combines CSS from layout, accessibility, responsive, and component modules.
//!
//! # Usage
//!
//! Import from this module:
//! ```ignore
//! use crate::styles::*;
//! use crate::styles::{tokens, primitives, components};
//! ```

pub mod accessibility;
pub mod components;
pub mod layout;
pub mod primitives;
pub mod responsive;
pub mod tokens;

// Re-export commonly used items for convenience
pub use components::*;
pub use primitives::*;
pub use tokens::*;

/// Bundled lotus styles: concatenates all CSS packs into a single stylesheet.
///
/// This function collects CSS from:
/// - Layout shell (app frame, sidebar, notices, share bar, etc.)
/// - Accessibility (focus, keyboard, motion, high contrast, forced colors)
/// - Responsive (media queries for different breakpoints)
///
/// The result is a single CSS string suitable for `<style>` tags or style bundles.
pub fn bundled_lotus_styles() -> String {
    [layout::css(), accessibility::css(), responsive::css()].join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_lotus_styles_is_non_empty() {
        let css = bundled_lotus_styles();
        assert!(!css.is_empty());
        assert!(css.len() > 100);
    }

    #[test]
    fn bundled_lotus_styles_contains_layout() {
        let css = bundled_lotus_styles();
        assert!(css.contains("app-layout") || css.contains("layout"));
    }

    #[test]
    fn bundled_lotus_styles_contains_accessibility() {
        let css = bundled_lotus_styles();
        assert!(css.contains("focus") || css.contains("prefers-"));
    }
}
