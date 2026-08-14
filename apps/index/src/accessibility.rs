//! Accessibility utilities and components.
//!
//! This module provides reusable accessibility patterns:
//! - Skip link for keyboard navigation
//! - ARIA annotations
//! - Focus management helpers

use dioxus::prelude::*;
use ui::prelude::*;

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
            style: StyleBuilder::new().property("position", "absolute").property("top", "-100%").property("left", "0.5rem").property("z-index", "9999").padding("0.5rem 1rem").property("background", "transparent").color("#0b5cab").font_size("0.875rem").font_weight("600").border_radius("0 0 4px 4px").text_decoration("underline").build(),
            "Skip to main content"
        }
    }
}
