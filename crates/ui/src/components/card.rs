// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Card component for displaying grouped content.

#![expect(
    clippy::empty_enums,
    reason = "Dioxus Props derive macro generates an empty helper enum"
)]

use crate::theme::{Interaction, Radius, Spacing, StyleBuilder, Theme, Typography};
use dioxus::prelude::*;

/// Properties for the [`Card`] component.
#[derive(Clone, Props, Debug, PartialEq)]
pub struct CardProps {
    /// Card title.
    pub title: String,
    /// Card content (description or body).
    #[props(default)]
    pub children: Option<Element>,
    /// Optional href making the card heading clickable.
    #[props(default)]
    pub href: Option<String>,
    /// Light/dark theme.
    #[props(default)]
    pub theme: Theme,
}

/// Card component for displaying grouped content in a semantic `<article>`.
///
/// # Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use ui::{Card, Theme};
///
/// fn app() -> Element {
///     rsx! {
///         Card {
///             title: "My App".to_string(),
///             href: Some("https://example.com".to_string()),
///             theme: Theme::Light,
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
    let colors = props.theme.colors();

    let card_style = StyleBuilder::with_capacity(240)
        .background_color(colors.surface)
        .border(&format!("1px solid {}", colors.border))
        .border_radius(Radius::MD)
        .padding(Spacing::LG)
        .box_shadow(props.theme.card_shadow())
        .transition(Interaction::TRANSITION_DEFAULT)
        .display("flex")
        .flex_direction("column")
        .gap(Spacing::MD)
        .build();

    let title_style = StyleBuilder::with_capacity(160)
        .font_size(Typography::H2)
        .font_family(Typography::SANS)
        .font_weight("600")
        .color(colors.text)
        .margin("0")
        .line_height(Typography::LINE_HEIGHT)
        .build();

    let content_style = StyleBuilder::with_capacity(160)
        .font_size(Typography::BODY)
        .font_family(Typography::SANS)
        .color(colors.text2)
        .margin("0")
        .line_height(Typography::LINE_HEIGHT)
        .build();

    rsx! {
        article {
            style: card_style,
            if let Some(href) = props.href {
                a {
                    href: href,
                    style: StyleBuilder::with_capacity(96)
                        .text_decoration("none")
                        .color("inherit")
                        .display("block")
                        .cursor("pointer")
                        .build(),
                    h2 { style: title_style, "{props.title}" }
                }
            } else {
                h2 { style: title_style, "{props.title}" }
            }
            if let Some(children) = props.children {
                div { style: content_style, {children} }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::render_debug;
    use crate::theme::ColorScheme;

    fn render_linked_light_card() -> Element {
        let description = "Interactive spectrum inspection";
        rsx! {
            Card {
                title: "MS/MS Viewer".to_string(),
                href: Some("https://example.org/msms".to_string()),
                theme: Theme::Light,
                "{description}"
            }
        }
    }

    fn render_plain_dark_card() -> Element {
        rsx! {
            Card {
                title: "Summary".to_string(),
                theme: Theme::Dark,
            }
        }
    }

    #[test]
    fn card_renders_linked_and_plain_variants() {
        let linked = render_debug(render_linked_light_card);
        assert!(
            linked.contains("MS/MS Viewer")
                && linked.contains("https://example.org/msms")
                && linked.contains("Interactive spectrum inspection"),
            "missing linked card contents in {linked}"
        );

        let plain = render_debug(render_plain_dark_card);
        assert!(
            plain.contains("Summary") && plain.contains(ColorScheme::DARK.surface),
            "missing dark plain card details in {plain}"
        );
        assert!(
            !plain.contains("cursor: pointer"),
            "non-linked card added link cursor"
        );
    }
}
