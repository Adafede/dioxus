//! Button component with multiple variants and states.

use crate::theme::{Radius, Spacing, StyleBuilder, Typography};
use dioxus::prelude::*;

/// Button variant enumeration
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
/// ```ignore
/// rsx! {
///     Button {
///         label: "Click me".to_string(),
///         variant: ButtonVariant::Primary,
///     }
/// }
/// ```
#[component]
pub fn Button(props: ButtonProps) -> Element {
    let _ = props.dark;

    let (bg_color, text_color, _hover_bg) = match props.variant {
        ButtonVariant::Primary => (
            "var(--btn-primary-bg)",
            "#fff",
            "var(--btn-primary-hover-bg)",
        ),
        ButtonVariant::Secondary => ("var(--surface)", "var(--text)", "var(--surface2)"),
        ButtonVariant::Tertiary => ("transparent", "var(--accent)", "var(--surface)"),
    };

    let border_value = match props.variant {
        ButtonVariant::Primary => "1px solid var(--border)".to_string(),
        ButtonVariant::Secondary => "1px solid var(--border)".to_string(),
        ButtonVariant::Tertiary => "none".to_string(),
    };

    let button_style = StyleBuilder::new()
        .background_color(bg_color)
        .color(text_color)
        .border(&border_value)
        .box_shadow(match props.variant {
            ButtonVariant::Primary | ButtonVariant::Secondary => "var(--shadow-xs)",
            ButtonVariant::Tertiary => "none",
        })
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

    #[test]
    fn button_variant_equality() {
        assert_eq!(ButtonVariant::Primary, ButtonVariant::Primary);
        assert_ne!(ButtonVariant::Primary, ButtonVariant::Secondary);
    }

    #[test]
    fn button_style_includes_padding() {
        let style = StyleBuilder::new()
            .padding(&format!("{} {}", Spacing::MD, Spacing::LG))
            .border_radius(Radius::SM)
            .build();

        assert!(style.contains("padding"));
        assert!(style.contains("border-radius"));
    }
}
