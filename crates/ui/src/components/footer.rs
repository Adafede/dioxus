// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Footer component with semantic HTML and external link security.

#![expect(
    clippy::empty_enums,
    reason = "Dioxus Props derive macro generates an empty helper enum"
)]

use crate::theme::{Radius, Spacing, StyleBuilder, Theme, Typography};
use dioxus::prelude::*;

/// Properties for the [`Footer`] component.
#[derive(Clone, Props, Debug, Eq, PartialEq)]
pub struct FooterProps {
    /// Light/dark color theme.
    #[props(default)]
    pub theme: Theme,
}

/// Page footer component with navigation and source attribution.
///
/// # Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use ui::{Footer, Theme};
///
/// fn app() -> Element {
///     rsx! {
///         Footer {
///             theme: Theme::Light,
///         }
///     }
/// }
///
/// let mut dom = VirtualDom::new(app);
/// dom.rebuild_in_place();
/// ```
#[component]
pub fn Footer(props: FooterProps) -> Element {
    let colors = props.theme.colors();

    let footer_style = StyleBuilder::with_capacity(140)
        .background_color(colors.bg)
        .border(&format!("1px solid {}", colors.border))
        .border_radius(Radius::LG)
        .padding(&format!("{} {}", Spacing::LG, Spacing::LG))
        .text_align("center")
        .build();

    let text_style = StyleBuilder::with_capacity(140)
        .font_size(Typography::BODY)
        .font_family(Typography::SANS)
        .color(colors.text3)
        .margin("0")
        .line_height(Typography::LINE_HEIGHT)
        .build();

    let link_style = StyleBuilder::with_capacity(96)
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
    use crate::theme::ColorScheme;

    fn render_light_footer() -> Element {
        rsx! { Footer { theme: Theme::Light } }
    }

    fn render_dark_footer() -> Element {
        rsx! { Footer { theme: Theme::Dark } }
    }

    #[test]
    fn footer_renders_themed_styles() {
        let light = render_debug(render_light_footer);
        assert!(
            light.contains(ColorScheme::LIGHT.bg) && light.contains(ColorScheme::LIGHT.accent),
            "light footer style mismatch: {light}"
        );

        let dark = render_debug(render_dark_footer);
        assert!(
            dark.contains(ColorScheme::DARK.bg) && dark.contains(ColorScheme::DARK.text3),
            "dark footer style mismatch: {dark}"
        );
    }
}
