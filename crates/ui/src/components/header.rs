// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Header component with semantic HTML and proper accessibility.

#![expect(
    clippy::empty_enums,
    reason = "Dioxus Props derive macro generates an empty helper enum"
)]

use crate::theme::{Radius, Spacing, StyleBuilder, Theme, Typography};
use dioxus::prelude::*;

/// Properties for the [`Header`] component.
#[derive(Clone, Props, Debug, Eq, PartialEq)]
pub struct HeaderProps {
    /// Title text displayed in the header.
    pub title: String,
    /// Optional subtitle or tagline.
    pub subtitle: Option<String>,
    /// Light/dark color theme.
    #[props(default)]
    pub theme: Theme,
}

/// Page header component with title and optional subtitle.
///
/// # Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use ui::Header;
///
/// fn app() -> Element {
///     rsx! {
///         Header {
///             title: "Welcome to Dioxus".to_string(),
///             subtitle: Some("Fast, productive Rust UI".to_string()),
///         }
///     }
/// }
///
/// let mut dom = VirtualDom::new(app);
/// dom.rebuild_in_place();
/// ```
#[component]
pub fn Header(props: HeaderProps) -> Element {
    let colors = props.theme.colors();

    let header_style = StyleBuilder::with_capacity(140)
        .background_color(colors.bg)
        .property_fmt(
            "padding",
            format_args!(
                "{} {} {} {}",
                Spacing::XL,
                Spacing::LG,
                Spacing::XL,
                Spacing::LG
            ),
        )
        .property_fmt("border", format_args!("1px solid {}", colors.border))
        .border_radius(Radius::LG)
        .text_align("center")
        .build();

    let title_style = StyleBuilder::with_capacity(160)
        .font_size(Typography::H1)
        .font_family(Typography::SANS)
        .font_weight("700")
        .color(colors.text)
        .margin("0 0 10px 0")
        .line_height(Typography::LINE_HEIGHT)
        .build();

    let subtitle_style = StyleBuilder::body_text_style(colors.text2).build();

    rsx! {
        header { style: header_style,
            h1 { style: title_style, "{props.title}" }
            if let Some(subtitle) = props.subtitle {
                p { style: subtitle_style, "{subtitle}" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::render_debug;
    use crate::theme::ColorScheme;

    fn render_light_header() -> Element {
        rsx! {
            Header {
                title: "Lipid Selector".to_string(),
                subtitle: Some("SMARTS lipid classification".to_string()),
                theme: Theme::Light,
            }
        }
    }

    fn render_dark_header_without_subtitle() -> Element {
        rsx! {
            Header {
                title: "Compact Header".to_string(),
                subtitle: None,
                theme: Theme::Dark,
            }
        }
    }

    #[test]
    fn header_renders_title_optional_subtitle_and_theme() {
        let light = render_debug(render_light_header);
        assert!(
            light.contains("Lipid Selector")
                && light.contains("SMARTS lipid classification")
                && light.contains(ColorScheme::LIGHT.text),
            "light header mismatch: {light}"
        );

        let dark = render_debug(render_dark_header_without_subtitle);
        assert!(
            dark.contains("Compact Header") && dark.contains(ColorScheme::DARK.text),
            "dark header mismatch: {dark}"
        );
    }
}
