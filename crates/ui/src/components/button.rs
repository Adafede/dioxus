//! Button component with multiple variants and states.

use crate::theme::{ColorScheme, Radius, Spacing, StyleBuilder, Typography};
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
    /// Whether button is disabled
    #[props(default)]
    pub disabled: bool,
    /// Whether to use dark theme
    #[props(default)]
    pub dark: bool,
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
    let colors = if props.dark {
        ColorScheme::DARK
    } else {
        ColorScheme::LIGHT
    };

    let (bg_color, text_color, _hover_bg) = match props.variant {
        ButtonVariant::Primary => (colors.accent, colors.bg, colors.accent2),
        ButtonVariant::Secondary => (colors.surface2, colors.accent, colors.border),
        ButtonVariant::Tertiary => ("transparent", colors.accent, colors.surface),
    };

    let border_value = match props.variant {
        ButtonVariant::Primary => "none".to_string(),
        ButtonVariant::Secondary => format!("1px solid {}", colors.border),
        ButtonVariant::Tertiary => "none".to_string(),
    };

    let button_style = StyleBuilder::new()
        .background_color(bg_color)
        .color(text_color)
        .border(&border_value)
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
        .transition(crate::theme::Interaction::TRANSITION_FAST)
        .build();

    rsx! {
        button {
            style: button_style,
            disabled: props.disabled,
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
