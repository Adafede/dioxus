//! Header component with semantic HTML and proper accessibility.

use crate::theme::{ColorScheme, Radius, Spacing, StyleBuilder, Typography};
use dioxus::prelude::*;

/// Properties for the Header component
#[derive(Clone, Props, Debug, PartialEq)]
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
/// ```ignore
/// rsx! {
///     Header {
///         title: "Welcome to Dioxus".to_string(),
///         subtitle: Some("Fast, productive Rust UI".to_string()),
///     }
/// }
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
        header { role: "banner", style: header_style,
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

    #[test]
    fn header_creates_valid_styles() {
        let colors = ColorScheme::LIGHT;
        let style = StyleBuilder::new()
            .background_color(colors.bg)
            .font_size(Typography::H1)
            .build();

        assert!(style.contains("background-color"));
        assert!(style.contains("font-size"));
    }
}
