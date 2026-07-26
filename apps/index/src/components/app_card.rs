//! Application card component.
//!
//! Displays a single application with title, link, and description.
//! Includes hover effects, keyboard focus indicators, and proper ARIA annotations.

use super::AppInfo;
use dioxus::prelude::*;

/// Card component for a single application.
///
/// Renders an accessible card with:
/// - Semantic `<article>` element
/// - Interactive link within heading
/// - Hover and focus states
/// - High contrast colors (WCAG AAA)
///
/// # Props
///
/// - `app`: Application metadata ([`AppInfo`])
///
/// # Accessibility
///
/// - Proper heading hierarchy (h2)
/// - Link focus indicators
/// - Color contrast >= 7:1 ratio
/// - Keyboard navigation support
///
/// # Example
///
/// ```rust,no_run
/// # use dioxus::prelude::*;
/// # use index::components::{AppCard, AppInfo};
/// let app = AppInfo {
///     id: "my-app",
///     title: "🔬 My App",
///     path: "./my-app/",
///     description: "Does cool things",
/// };
/// rsx! {
///     AppCard { app: app }
/// }
/// ```
#[component]
pub fn AppCard(app: AppInfo) -> Element {
    rsx! {
        article { class: "app-card",
            h2 {
                a { href: app.path, "{app.title}" }
            }
            p { class: "description", "{app.description}" }
        }
    }
}
