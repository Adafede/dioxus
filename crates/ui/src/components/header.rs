// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Header component with semantic HTML and proper accessibility.

#![expect(
    clippy::empty_enums,
    reason = "Dioxus Props derive macro generates an empty helper enum"
)]

use crate::theme::{ColorScheme, Radius, Spacing, StyleBuilder, Typography};
use dioxus::prelude::*;

/// Properties for the Header component
#[allow(clippy::module_name_repetitions)] // UI component type intentionally matches module name
#[derive(Clone, Props, Debug, Eq, PartialEq)]
pub struct HeaderProps {
    /// Title text displayed in the header
    pub title: String,
    /// Optional subtitle or tagline
    pub subtitle: Option<String>,
    /// Whether to use dark theme
    #[props(default)]
    pub dark: bool,
}

/// Page header component with title and optional subtitle.
///
/// Renders semantic `<header>` with proper landmarks and accessible heading hierarchy.
/// Uses pure Rust styling for consistent theming.
///
/// # Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use ui::prelude::Header;
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
    let colors = if props.dark {
        ColorScheme::DARK
    } else {
        ColorScheme::LIGHT
    };

    let header_style = StyleBuilder::new()
        .background_color(colors.bg)
        .padding(&format!(
            "{} {} {} {}",
            Spacing::XL,
            Spacing::LG,
            Spacing::XL,
            Spacing::LG
        ))
        .border(&format!("1px solid {}", colors.border))
        .border_radius(Radius::LG)
        .text_align("center")
        .build();

    let title_style = StyleBuilder::new()
        .font_size(Typography::H1)
        .font_family(Typography::SANS)
        .font_weight("700")
        .color(colors.text)
        .margin("0 0 10px 0")
        .line_height(Typography::LINE_HEIGHT)
        .build();

    let subtitle_style = StyleBuilder::new()
        .font_size(Typography::BODY)
        .font_family(Typography::SANS)
        .color(colors.text2)
        .margin("0")
        .line_height(Typography::LINE_HEIGHT)
        .build();

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

    fn render_light_header() -> Element {
        rsx! {
            Header {
                title: "Lipid Selector".to_string(),
                subtitle: Some("SMARTS lipid classification".to_string()),
                dark: false,
            }
        }
    }

    fn render_dark_header_without_subtitle() -> Element {
        rsx! {
            Header {
                title: "Compact Header".to_string(),
                subtitle: None,
                dark: true,
            }
        }
    }

    #[test]
    fn header_renders_title_optional_subtitle_and_theme() {
        let light = render_debug(render_light_header);
        assert!(light.contains("Lipid Selector"), "missing title in {light}");
        assert!(
            light.contains("SMARTS lipid classification"),
            "missing subtitle in {light}"
        );
        assert!(
            light.contains(ColorScheme::LIGHT.text),
            "missing light heading color in {light}"
        );

        let dark = render_debug(render_dark_header_without_subtitle);
        assert!(
            dark.contains("Compact Header"),
            "missing dark title in {dark}"
        );
        assert!(
            dark.contains(ColorScheme::DARK.text),
            "missing dark heading color in {dark}"
        );
    }
}
