//! Footer component with semantic HTML and external link security.

use dioxus::prelude::*;
use crate::theme::{ColorScheme, Spacing, Typography, StyleBuilder};

/// Properties for the Footer component
#[derive(Clone, Props, Debug, PartialEq)]
pub struct FooterProps {
    /// Whether to use dark theme
    #[props(default)]
    pub dark: bool,
}

/// Page footer component with navigation and attribution.
///
/// Renders semantic `<footer>` with proper contentinfo landmark and accessible links.
/// All external links include `rel="noopener noreferrer"` for security and performance.
///
/// # Example
///
/// ```ignore
/// rsx! {
///     Footer {
///         dark: false,
///     }
/// }
/// ```
#[component]
pub fn Footer(props: FooterProps) -> Element {
    let colors = if props.dark {
        ColorScheme::DARK
    } else {
        ColorScheme::LIGHT
    };

    let footer_style = StyleBuilder::new()
        .background_color(colors.bg)
        .border(&format!("1px solid {}", colors.border))
        .padding(&format!("{} {}", Spacing::LG, Spacing::LG))
        .text_align("center")
        .build();

    let text_style = StyleBuilder::new()
        .font_size(Typography::BODY)
        .font_family(Typography::SANS)
        .color(colors.text3)
        .margin("0")
        .line_height(Typography::LINE_HEIGHT)
        .build();

    let link_style_value = || {
        StyleBuilder::new()
            .color(colors.accent)
            .text_decoration("none")
            .border_bottom("1px solid transparent")
            .transition(crate::theme::Interaction::TRANSITION_FAST)
            .build()
    };

    rsx! {
        footer { role: "contentinfo", style: footer_style,
            p { style: text_style,
                "Built with "
                a {
                    href: "https://dioxuslabs.com",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    style: link_style_value(),
                    "Dioxus"
                }
                " • "
                a {
                    href: "https://github.com/adrutz/dioxus",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    style: link_style_value(),
                    "View source"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn footer_styles_compile() {
        let colors = ColorScheme::LIGHT;
        let style = StyleBuilder::new()
            .color(colors.text3)
            .background_color(colors.bg)
            .build();

        assert!(style.contains("color"));
        assert!(style.contains("background-color"));
    }
}
