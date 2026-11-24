// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Button component with multiple variants and states.

#![expect(
    clippy::empty_enums,
    reason = "Dioxus Props derive macro generates an empty helper enum"
)]

use crate::theme::{Radius, Shadow, Spacing, StyleBuilder, Theme, Typography};
use dioxus::prelude::*;

/// Button variant enumeration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonVariant {
    /// Primary action button (filled with accent color).
    Primary,
    /// Secondary action button (outline style).
    Secondary,
    /// Tertiary/subtle button (ghost style).
    Tertiary,
}

/// Properties for the [`Button`] component.
#[derive(Clone, Props, Debug, PartialEq)]
pub struct ButtonProps {
    /// Button label text.
    pub label: String,
    /// Visual style variant.
    #[props(default = ButtonVariant::Primary)]
    pub variant: ButtonVariant,
    /// Whether the button is disabled.
    #[props(default)]
    pub disabled: bool,
    /// Light/dark color theme.
    #[props(default)]
    pub theme: Theme,
    /// HTML button type. Defaults to `"button"`.
    #[props(default = "button")]
    pub r#type: &'static str,
    /// Optional click handler.
    #[props(default)]
    pub onclick: Option<EventHandler<Event<MouseData>>>,
}

/// Button component with Primary, Secondary, and Tertiary variants.
///
/// # Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use ui::{Button, ButtonVariant};
///
/// fn app() -> Element {
///     rsx! {
///         Button {
///             label: "Click me".to_string(),
///             variant: ButtonVariant::Primary,
///         }
///     }
/// }
///
/// let mut dom = VirtualDom::new(app);
/// dom.rebuild_in_place();
/// ```
#[component]
pub fn Button(props: ButtonProps) -> Element {
    let colors = props.theme.colors();
    let (bg_color, text_color, border_color, box_shadow) = match props.variant {
        ButtonVariant::Primary => (colors.accent, colors.surface, colors.accent, Shadow::XS),
        ButtonVariant::Secondary => (colors.surface, colors.text, colors.border, Shadow::XS),
        ButtonVariant::Tertiary => ("transparent", colors.accent, "transparent", "none"),
    };
    let border_value = if props.variant == ButtonVariant::Tertiary {
        "none".to_string()
    } else {
        format!("1px solid {border_color}")
    };

    let button_style = StyleBuilder::with_capacity(280)
        .background_color(bg_color)
        .color(text_color)
        .border(&border_value)
        .box_shadow(box_shadow)
        .padding(&format!("{} {}", Spacing::MD, Spacing::LG))
        .border_radius(Radius::SM)
        .font_size(Typography::UI)
        .font_family(Typography::SANS)
        .font_weight("500")
        .cursor(if props.disabled {
            "not-allowed"
        } else {
            "pointer"
        })
        .opacity(if props.disabled { "0.6" } else { "1" })
        .transition("transform 150ms")
        .build();

    let on_click = props.onclick;
    rsx! {
        button {
            r#type: props.r#type,
            style: button_style,
            disabled: props.disabled,
            onclick: move |evt: Event<MouseData>| {
                if let Some(handler) = on_click.as_ref() {
                    handler.call(evt);
                }
            },
            "{props.label}"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::render_debug;
    use crate::theme::ColorScheme;

    fn render_primary_submit() -> Element {
        rsx! {
            Button {
                label: "Save result".to_string(),
                variant: ButtonVariant::Primary,
                r#type: "submit",
            }
        }
    }

    fn render_secondary_dark() -> Element {
        rsx! {
            Button {
                label: "Inspect".to_string(),
                variant: ButtonVariant::Secondary,
                theme: Theme::Dark,
            }
        }
    }

    fn render_tertiary_disabled() -> Element {
        rsx! {
            Button {
                label: "Skip".to_string(),
                variant: ButtonVariant::Tertiary,
                disabled: true,
            }
        }
    }

    #[test]
    fn button_renders_variants_and_disabled_state() {
        let primary = render_debug(render_primary_submit);
        assert!(primary.contains("Save result") && primary.contains("submit"));

        let secondary = render_debug(render_secondary_dark);
        assert!(secondary.contains(ColorScheme::DARK.border));

        let tertiary = render_debug(render_tertiary_disabled);
        assert!(tertiary.contains("border: none"));
        assert!(tertiary.contains("cursor: not-allowed"));
        assert!(tertiary.contains("opacity: 0.6"));
    }
}
