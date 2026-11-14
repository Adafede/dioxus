// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Footer component with semantic HTML and external link security.

#![expect(
    clippy::empty_enums,
    reason = "Dioxus Props derive macro generates an empty helper enum"
)]

use crate::theme::{ColorScheme, Radius, Spacing, StyleBuilder, Typography};
use dioxus::prelude::*;

/// Properties for the Footer component
#[allow(clippy::module_name_repetitions)] // UI component type intentionally matches module name
#[derive(Clone, Props, Debug, Eq, PartialEq)]
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
/// ```rust
/// use dioxus::prelude::*;
/// use ui::prelude::Footer;
///
/// fn app() -> Element {
///     rsx! {
///         Footer {
///             dark: false,
///         }
///     }
/// }
///
/// let mut dom = VirtualDom::new(app);
/// dom.rebuild_in_place();
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
        .border_radius(Radius::LG)
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

    let link_style = StyleBuilder::new()
        .color(colors.accent)
        .text_decoration("none")
        .border_bottom("1px solid transparent")
        .build();

    rsx! {
        footer { style: footer_style,
            p { style: text_style,
                "Built with "
                a {
                    href: "https://dioxuslabs.com",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    style: &link_style,
                    "Dioxus"
                }
                " • "
                a {
                    href: "https://github.com/adafede/dioxus",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    style: &link_style,
                    "View source"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::render_debug;

    fn render_light_footer() -> Element {
        rsx! {
            Footer { dark: false }
        }
    }

    fn render_dark_footer() -> Element {
        rsx! {
            Footer { dark: true }
        }
    }

    #[test]
    fn footer_renders_themed_styles() {
        let light = render_debug(render_light_footer);
        assert!(
            light.contains(ColorScheme::LIGHT.bg),
            "light footer should use light background in {light}"
        );
        assert!(
            light.contains(ColorScheme::LIGHT.accent),
            "light footer should style links with light accent in {light}"
        );

        let dark = render_debug(render_dark_footer);
        assert!(
            dark.contains(ColorScheme::DARK.bg),
            "dark footer should use dark background in {dark}"
        );
        assert!(
            dark.contains(ColorScheme::DARK.text3),
            "dark footer should use dark muted text in {dark}"
        );
    }
}
