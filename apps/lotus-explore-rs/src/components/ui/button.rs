// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Shared Button component using Lotus token Tailwind classes.

use crate::ui::classes;
use dioxus::prelude::*;

/// Visual variant for the button.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ButtonVariant {
    /// Solid primary brand action
    #[default]
    Primary,
    /// Bordered secondary action on surface
    Secondary,
    /// Destructive action
    Danger,
    /// Emphasized primary (e.g. dirty search)
    Accent,
}

/// Size variant for the button.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ButtonSize {
    /// Compact toolbar / dense form control (34px min-height) - uses RADIUS_SM_CTRL (4px)
    #[default]
    Sm,
    /// Standard form / card button (40px min-height) - uses RADIUS_INPUT (6px)
    Md,
}

/// Props for the Button component.
#[derive(Props, Clone, PartialEq)]
pub struct ButtonProps {
    #[props(default)]
    pub label: Option<String>,
    #[props(default)]
    pub variant: ButtonVariant,
    #[props(default)]
    pub size: ButtonSize,
    #[props(default)]
    pub disabled: bool,
    #[props(default)]
    pub loading: bool,
    #[props(default = "button")]
    pub r#type: &'static str,
    #[props(default)]
    pub title: Option<String>,
    #[props(default)]
    pub aria_label: Option<String>,
    #[props(default)]
    pub aria_controls: Option<String>,
    #[props(default)]
    pub aria_expanded: Option<String>,
    #[props(default)]
    pub aria_pressed: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub onclick: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn Button(props: ButtonProps) -> Element {
    let size_classes = match props.size {
        ButtonSize::Sm => classes::BTN_SIZE_SM,
        ButtonSize::Md => classes::BTN_SIZE_MD,
    };

    // Variants map 1:1 to token composites — no inline duplication
    let variant_classes = match props.variant {
        ButtonVariant::Primary => classes::BTN_PRIMARY,
        ButtonVariant::Secondary => classes::BTN_SECONDARY,
        ButtonVariant::Danger => classes::BTN_DANGER,
        ButtonVariant::Accent => classes::BTN_ACCENT,
    };

    let state_classes = if props.disabled || props.loading {
        classes::DISABLED
    } else {
        classes::ACTIVE_SCALE
    };

    let custom_classes = props.class.as_deref().unwrap_or("");

    rsx! {
        button {
            r#type: props.r#type,
            disabled: props.disabled || props.loading,
            title: props.title.as_deref().unwrap_or_default(),
            aria_label: props.aria_label.as_deref().unwrap_or_default(),
            aria_controls: props.aria_controls.as_deref().unwrap_or_default(),
            aria_expanded: props.aria_expanded.as_deref().unwrap_or_default(),
            aria_pressed: props.aria_pressed.as_deref().unwrap_or_default(),
            class: "inline-flex items-center justify-center font-sans select-none {size_classes} {variant_classes} {state_classes} {custom_classes}",
            onclick: move |evt| {
                if !props.disabled && !props.loading
                    && let Some(handler) = props.onclick.as_ref() {
                        handler.call(evt);
                    }
            },
            if props.loading {
                span {
                    class: "inline-block size-3.5 {classes::RADIUS_FULL} border-2 border-current border-t-transparent animate-spin",
                    "aria-hidden": "true",
                }
            }
            if let Some(ref text) = props.label {
                span { "{text}" }
            }
            {props.children}
        }
    }
}
