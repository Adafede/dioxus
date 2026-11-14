// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Card component for displaying grouped content.

#![expect(
    clippy::empty_enums,
    reason = "Dioxus Props derive macro generates an empty helper enum"
)]

use crate::theme::{ColorScheme, Radius, Shadow, Spacing, StyleBuilder, Typography};
use dioxus::prelude::*;

/// Properties for the Card component
#[allow(clippy::module_name_repetitions)] // UI component type intentionally matches module name
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
/// ```rust
/// use dioxus::prelude::*;
/// use ui::prelude::Card;
///
/// fn app() -> Element {
///     rsx! {
///         Card {
///             title: "My App".to_string(),
///             href: Some("https://example.com".to_string()),
///             dark: false,
///             "Description of the app"
///         }
///     }
/// }
///
/// let mut dom = VirtualDom::new(app);
/// dom.rebuild_in_place();
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
        .box_shadow(if props.dark {
            Shadow::SM_DARK
        } else {
            Shadow::SM
        })
        .transition(crate::theme::Interaction::TRANSITION_DEFAULT)
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
                a {
                    href: href,
                    style: StyleBuilder::new()
                        .text_decoration("none")
                        .color("inherit")
                        .display("block")
                        .cursor("pointer")
                        .build(),
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
    use crate::components::render_debug;

    fn render_linked_light_card() -> Element {
        let description = "Interactive spectrum inspection";
        rsx! {
            Card {
                title: "MS/MS Viewer".to_string(),
                href: Some("https://example.org/msms".to_string()),
                dark: false,
                "{description}"
            }
        }
    }

    fn render_plain_dark_card() -> Element {
        rsx! {
            Card {
                title: "Summary".to_string(),
                dark: true,
            }
        }
    }

    #[test]
    fn card_renders_linked_and_plain_variants() {
        let linked = render_debug(render_linked_light_card);
        assert!(
            linked.contains("MS/MS Viewer"),
            "missing linked card title in {linked}"
        );
        assert!(
            linked.contains("https://example.org/msms"),
            "missing card href in {linked}"
        );
        assert!(
            linked.contains("Interactive spectrum inspection"),
            "missing card body in {linked}"
        );

        let plain = render_debug(render_plain_dark_card);
        assert!(
            plain.contains("Summary"),
            "missing plain card title in {plain}"
        );
        assert!(
            plain.contains(ColorScheme::DARK.surface),
            "dark card should use dark surface color in {plain}"
        );
        assert!(
            !plain.contains("cursor: pointer"),
            "non-linked card should not add link cursor style in {plain}"
        );
    }
}
