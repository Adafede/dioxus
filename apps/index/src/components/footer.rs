//! Footer component.
//!
//! Page footer with navigation links and attribution. Includes semantic markup for crawlability and accessibility.

use dioxus::prelude::*;

/// Page footer component.
///
/// Renders the site footer with:
/// - Semantic `<footer>` element with `role="contentinfo"`
/// - External links with proper attributes (`target="_blank"`, `rel="noopener noreferrer"`)
/// - Accessible link annotations
/// - WCAG AAA text contrast
///
/// # Accessibility
///
/// - Contentinfo landmark role for navigation
/// - Proper link attributes to prevent security/performance issues
/// - High contrast text on background
/// - Keyboard navigable
///
/// # Security
///
/// - External links use `rel="noopener noreferrer"` to prevent window hijacking
/// - `target="_blank"` used only with proper rel attributes
#[component]
pub fn Footer() -> Element {
    rsx! {
        footer { role: "contentinfo",
            p {
                "Built with "
                a {
                    href: "https://dioxuslabs.com",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    "Dioxus"
                }
                " • "
                a {
                    href: "https://github.com/adrutz/dioxus",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    "View source on GitHub"
                }
            }
        }
    }
}
