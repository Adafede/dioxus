// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Interaction primitives: cursor, transitions, focus.

use super::super::tokens::*;
use ui::theme::StyleBuilder;

// ============================================================================
// CURSOR PRIMITIVES
// ============================================================================

/// Pointer cursor: `cursor: pointer`
pub fn cursor_pointer() -> String {
    StyleBuilder::new().cursor("pointer").build()
}

/// Default cursor: `cursor: auto`
pub fn cursor_default() -> String {
    StyleBuilder::new().cursor("auto").build()
}

// ============================================================================
// TRANSITION PRIMITIVES
// ============================================================================

/// Fast transition: `transition: ... 0.1s`
pub fn transition_fast() -> String {
    StyleBuilder::new()
        .property(
            "transition",
            "background .1s, border-color .1s, box-shadow .1s, transform .1s ease",
        )
        .build()
}

/// Standard transition: `transition: ... 0.15s`
pub fn transition_standard() -> String {
    StyleBuilder::new()
        .property(
            "transition",
            "background .15s, border-color .15s, box-shadow .15s, transform .12s ease",
        )
        .build()
}

/// Slow transition: `transition: ... 0.3s`
pub fn transition_slow() -> String {
    StyleBuilder::new()
        .property(
            "transition",
            "background .3s, border-color .3s, box-shadow .3s, transform .3s ease",
        )
        .build()
}

// ============================================================================
// FOCUS PRIMITIVES
// ============================================================================

/// Focus outline: standard focus styles
pub fn focus_outline() -> String {
    StyleBuilder::new()
        .property("outline", "2px solid var(--accent)")
        .property("outline-offset", "2px")
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_pointer_is_set() {
        assert!(cursor_pointer().contains("cursor"));
        assert!(cursor_pointer().contains("pointer"));
    }

    #[test]
    fn transition_fast_uses_timing() {
        assert!(transition_fast().contains("transition"));
        assert!(transition_fast().contains(".1s"));
    }

    #[test]
    fn focus_outline_has_properties() {
        assert!(focus_outline().contains("outline"));
        assert!(focus_outline().contains("2px"));
    }
}
