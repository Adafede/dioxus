// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Shared Button component using Lotus token Tailwind classes.

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
    /// Compact toolbar / dense form control
    #[default]
    Sm,
    /// Standard form / card button
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
        ButtonSize::Sm => "min-h-[34px] gap-1.5 px-3 py-1.5 text-ui rounded-sm",
        ButtonSize::Md => "min-h-10 gap-2 px-3.5 py-2 text-ui rounded-md",
    };

    // Keep variants flat — tokens already carry light/dark.
    let variant_classes = match props.variant {
        ButtonVariant::Primary => {
            "border border-border bg-accent text-bg font-semibold shadow-xs hover:bg-accent-2"
        }
        ButtonVariant::Secondary => {
            "border border-border bg-surface text-text font-semibold shadow-xs hover:bg-bg"
        }
        ButtonVariant::Danger => {
            "border border-danger/35 bg-danger/10 text-danger font-semibold hover:bg-danger/15"
        }
        ButtonVariant::Accent => {
            "border border-border bg-accent text-bg font-semibold shadow-xs ring-2 ring-accent/40 hover:bg-accent-2"
        }
    };

    let state_classes = if props.disabled || props.loading {
        "opacity-60 cursor-not-allowed pointer-events-none"
    } else {
        "cursor-pointer"
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
            class: "inline-flex items-center justify-center font-sans select-none transition-[background,border-color,box-shadow] duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/40 {size_classes} {variant_classes} {state_classes} {custom_classes}",
            onclick: move |evt| {
                if !props.disabled && !props.loading {
                    if let Some(handler) = props.onclick.as_ref() {
                        handler.call(evt);
                    }
                }
            },
            if props.loading {
                span {
                    class: "inline-block size-3.5 rounded-full border-2 border-current border-t-transparent animate-spin",
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
