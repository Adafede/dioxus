//! Card component for displaying grouped content.

use dioxus::prelude::*;
use crate::theme::{ColorScheme, Spacing, Typography, Radius, Shadow, StyleBuilder, Interaction};

/// Properties for the Card component
#[derive(Clone, Props, Debug, PartialEq)]
pub struct CardProps {
    /// Card title
    pub title: String,
    /// Card content (typically description or body)
    #[props(default)]
    pub children: Option<Element>,
    /// Optional href for card link (makes card clickable)
    #[props(default)]
    pub href: Option<String>,
    /// Whether to use dark theme
    #[props(default)]
    pub dark: bool,
}

/// Card component for displaying grouped content.
///
/// Renders a semantic `<article>` with consistent styling.
/// Can be used as a standalone card or as a clickable link card.
///
/// # Example
///
/// ```ignore
/// rsx! {
///     Card {
///         title: "My App".to_string(),
///         href: Some("https://example.com".to_string()),
///         dark: false,
///         "Description of the app"
///     }
/// }
/// ```
#[component]
pub fn Card(props: CardProps) -> Element {
    let colors = if props.dark {
        ColorScheme::DARK
    } else {
        ColorScheme::LIGHT
    };

    let card_style = StyleBuilder::new()
        .background_color(colors.surface)
        .border(&format!("1px solid {}", colors.border))
        .border_radius(Radius::MD)
        .padding(Spacing::LG)
        .box_shadow(if props.dark { Shadow::SM_DARK } else { Shadow::SM })
        .transition(crate::theme::Interaction::TRANSITION_DEFAULT)
        .display("flex")
        .flex_direction("column")
        .gap(Spacing::MD)
        .build();

    let card_hover_style = if props.href.is_some() {
        StyleBuilder::new()
            .background_color(colors.surface2)
            .box_shadow(if props.dark { Shadow::MD_DARK } else { Shadow::MD })
            .build()
    } else {
        String::new()
    };

    let title_style = StyleBuilder::new()
        .font_size(Typography::H2)
        .font_family(Typography::SANS)
        .font_weight("600")
        .color(colors.text)
        .margin("0")
        .line_height(Typography::LINE_HEIGHT)
        .build();

    let content_style = StyleBuilder::new()
        .font_size(Typography::BODY)
        .font_family(Typography::SANS)
        .color(colors.text2)
        .margin("0")
        .line_height(Typography::LINE_HEIGHT)
        .build();

    if let Some(href) = props.href {
        rsx! {
            article {
                style: card_style,
                class: if !card_hover_style.is_empty() { "card-hover" } else { "" },
                a {
                    href: href,
                    style: "text-decoration: none; color: inherit; display: block; cursor: pointer;".to_string(),
                    h2 { style: title_style, "{props.title}" }
                }
                if let Some(children) = props.children {
                    div { style: content_style, {children} }
                }
            }
        }
    } else {
        rsx! {
            article { style: card_style,
                h2 { style: title_style, "{props.title}" }
                if let Some(children) = props.children {
                    div { style: content_style, {children} }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_style_includes_spacing() {
        let colors = ColorScheme::LIGHT;
        let style = StyleBuilder::new()
            .padding(Spacing::LG)
            .border_radius(Radius::MD)
            .build();

        assert!(style.contains("padding"));
        assert!(style.contains("border-radius"));
    }
}
