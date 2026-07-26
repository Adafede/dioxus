//! Accessibility utilities and components.
//!
//! This module provides reusable accessibility patterns:
//! - Skip link for keyboard navigation
//! - ARIA annotations
//! - Focus management helpers

use dioxus::prelude::*;

/// Skip link component for keyboard users.
///
/// Provides a hidden link that appears on keyboard focus, allowing users to skip
/// directly to the main content. This is a WCAG AAA best practice.
///
/// # Accessibility
///
/// - Keyboard navigable: Jump directly to `#main-content` with Tab key
/// - Screen reader friendly: Properly announced as a navigation link
/// - Visible on focus: Becomes visible when focused via keyboard
#[component]
pub fn SkipLink() -> Element {
    rsx! {
        a {
            href: "#main-content",
            class: "skip-link",
            "Skip to main content"
        }
    }
}
