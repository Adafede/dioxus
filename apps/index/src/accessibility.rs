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
            style: "position:absolute;top:-100%;left:0.5rem;z-index:9999;padding:0.5rem 1rem;background:#0b5cab;color:#fff;font-size:0.875rem;font-weight:600;border-radius:0 0 4px 4px;text-decoration:none;",
            "Skip to main content"
        }
    }
}
