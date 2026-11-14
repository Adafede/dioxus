// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Button component with multiple variants and states.

#![expect(
    clippy::empty_enums,
    reason = "Dioxus Props derive macro generates an empty helper enum"
)]

use crate::theme::{ColorScheme, Radius, Shadow, Spacing, StyleBuilder, Typography};
use dioxus::prelude::*;

/// Button variant enumeration
#[allow(clippy::module_name_repetitions)] // UI component type intentionally matches module name
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonVariant {
    /// Primary action button (filled with accent color)
    Primary,
    /// Secondary action button (outline style)
    Secondary,
    /// Tertiary/subtle button (ghost style)
    Tertiary,
}

/// Properties for the Button component
#[allow(clippy::module_name_repetitions)] // UI component type intentionally matches module name
#[derive(Clone, Props, Debug, PartialEq)]
pub struct ButtonProps {
    /// Button label text
    pub label: String,
    /// Button variant
    #[props(default = ButtonVariant::Primary)]
    pub variant: ButtonVariant,
    /// Whether the button is disabled
    #[props(default)]
    pub disabled: bool,
    /// Whether to use dark theme
    #[props(default)]
    pub dark: bool,
    /// HTML button type. Default to `button` to avoid accidental form submission.
    #[props(default = "button")]
    pub r#type: &'static str,
    /// Optional click handler. When `None` (the default), no `onclick`
    /// attribute is emitted — backward-compatible with call sites that omit it.
    #[props(default)]
    pub onclick: Option<EventHandler<Event<MouseData>>>,
}

/// Button component with multiple variants.
///
/// Provides accessible buttons with consistent styling across the application.
/// Supports Primary, Secondary, and Tertiary variants with proper focus and hover states.
///
/// # Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use ui::prelude::{Button, ButtonVariant};
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
    let colors = if props.dark {
        ColorScheme::DARK
    } else {
        ColorScheme::LIGHT
    };

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

    let button_style = StyleBuilder::new()
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
                dark: true,
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
        assert!(
            primary.contains("Save result"),
            "missing primary label in {primary}"
        );
        assert!(
            primary.contains("submit"),
            "missing submit type in {primary}"
        );

        let secondary = render_debug(render_secondary_dark);
        assert!(
            secondary.contains(ColorScheme::DARK.border),
            "dark secondary should use dark border in {secondary}"
        );

        let tertiary = render_debug(render_tertiary_disabled);
        assert!(
            tertiary.contains("border: none"),
            "tertiary button should omit border in {tertiary}"
        );
        assert!(
            tertiary.contains("cursor: not-allowed"),
            "disabled button should set not-allowed cursor in {tertiary}"
        );
        assert!(
            tertiary.contains("opacity: 0.6"),
            "disabled button should dim opacity in {tertiary}"
        );
    }
}
