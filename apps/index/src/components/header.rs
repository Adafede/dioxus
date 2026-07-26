//! Header component.
//!
//! Page header with title and subtitle. Includes proper semantic markup for SEO and accessibility.

use dioxus::prelude::*;

/// Page header component.
///
/// Renders the main heading and subtitle with:
/// - Semantic `<header>` element with `role="banner"`
/// - Proper heading hierarchy (h1)
/// - Responsive typography
/// - WCAG AAA compliant text contrast
///
/// # Accessibility
///
/// - Banner landmark role for navigation
/// - Single h1 per page for screen readers
/// - Large, readable text (minimum 16px base)
/// - Clear visual hierarchy
#[component]
pub fn Header() -> Element {
    rsx! {
        header { role: "banner",
            h1 { "🦀 Dioxus Experiments" }
            p { class: "subtitle",
                "A collection of open-source Rust/WASM applications exploring UI patterns, performance optimization, and data processing."
            }
        }
    }
}
