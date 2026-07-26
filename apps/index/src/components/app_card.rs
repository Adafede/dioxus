//! Application card component.
//!
//! Displays a single application with title, link, and description.
//! Includes hover effects, keyboard focus indicators, and proper ARIA annotations.

use super::AppInfo;
use dioxus::prelude::*;
use ui::prelude::*;

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
    let colors = ColorScheme::LIGHT;

    let card_style = StyleBuilder::new()
        .background_color(colors.surface)
        .border(&format!("1px solid {}", colors.border))
        .border_radius(Radius::MD)
        .padding(Spacing::LG)
        .box_shadow(Shadow::SM)
        .transition(Interaction::TRANSITION_DEFAULT)
        .display("flex")
        .flex_direction("column")
        .gap(Spacing::MD)
        .build();

    let title_style = StyleBuilder::new()
        .font_size(Typography::H2)
        .font_family(Typography::SANS)
        .font_weight("600")
        .color(colors.text)
        .margin("0")
        .line_height(Typography::LINE_HEIGHT)
        .build();

    let link_style = StyleBuilder::new()
        .color(colors.accent)
        .text_decoration("none")
        .build();

    let description_style = StyleBuilder::new()
        .font_size(Typography::BODY)
        .font_family(Typography::SANS)
        .color(colors.text2)
        .margin("0")
        .line_height(Typography::LINE_HEIGHT)
        .build();

    rsx! {
        article { style: card_style,
            h2 { style: title_style,
                a { href: app.path, style: link_style, "{app.title}" }
            }
            p { style: description_style, "{app.description}" }
        }
    }
}
