// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Common UI utilities shared across all apps.
//!
//! Provides reusable components and constants for accessibility and consistency.

use dioxus::prelude::*;

/// Default skip link style for keyboard navigation to main content.
///
/// This style positions the link off-screen but visible when focused,
/// providing accessibility for keyboard users as per WCAG guidelines.
///
/// # Usage
///
/// Apply to an anchor element that points to the main content landmark:
///
/// ```html
/// <a href="#main-content" class="skip-link" style="position:absolute;top:-100%;...">Skip to main content</a>
/// ```
pub const SKIP_LINK_STYLE: &str = "position:absolute;top:-100%;left:0.5rem;z-index:9999;padding:0.5rem 1rem;background:transparent;color:#0b5cab;font-size:0.875rem;font-weight:600;border-radius:0 0 4px 4px;text-decoration:underline;";

/// Skip navigation link for keyboard accessibility.
///
/// Renders a visually-hidden link that becomes visible on focus,
/// allowing keyboard users to skip repetitive navigation sequences.
///
/// # Target Selection
///
/// The link defaults to `#main-content` but some apps use:
/// - `id="main"` (lotus-explore, mgf-precursor-erro-rs)
/// - `id="main-content"` (json-count-rs, index)
///
/// Apps should override the target using CSS or custom rendering if needed.
///
/// # Accessibility Compliance
///
/// WCAG 2.1 AA compliant:
/// - Visible on focus only
/// - Screen reader friendly
/// - No visual distraction for mouse users
///
/// # Example
///
/// ```ignore
/// use ui::prelude::skip_link;
///
/// fn app() -> Element {
///     rsx! {
///         skip_link {}
///
///         main {
///             id: "main-content",
///             // ... main content
///         }
///     }
/// }
/// ```
#[component]
pub fn skip_link() -> Element {
    rsx! {
        a {
            href: "#main-content",
            class: "skip-link",
            style: SKIP_LINK_STYLE,
            "Skip to main content"
        }
    }
}

/// Alternative skip link component for apps using a different main landmark ID.
///
/// Use this when your main content element uses `id="main"` instead of `id="main-content"`.
///
/// # Example
///
/// ```ignore
/// use ui::prelude::skip_link_main;
///
/// fn app() -> Element {
///     rsx! {
///         skip_link_main {}
///
///         main {
///             id: "main",
///             // ... main content
///         }
///     }
/// }
/// ```
#[component]
pub fn skip_link_main() -> Element {
    rsx! {
        a {
            href: "#main",
            class: "skip-link",
            style: SKIP_LINK_STYLE,
            "Skip to main content"
        }
    }
}
