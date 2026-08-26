// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! DRY Button component using Tailwind classes.
//!
//! Supports size and variant props for consistent styling across the application.

use dioxus::prelude::*;

/// Visual variant for the button.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ButtonVariant {
    /// Solid primary brand action (sky)
    #[default]
    Primary,
    /// Subtle bordered secondary action
    Secondary,
    /// Danger/destructive action (rose)
    Danger,
    /// Highlighted/dirty search state with prominent glow
    Accent,
}

/// Size variant for the button.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ButtonSize {
    /// Compact size for toolbars, table cells, and dense forms
    #[default]
    Sm,
    /// Standard form / card button
    Md,
}

/// Props for the Button component.
#[derive(Props, Clone, PartialEq)]
pub struct ButtonProps {
    /// Optional button text label
    #[props(default)]
    pub label: Option<String>,
    /// Visual style variant
    #[props(default)]
    pub variant: ButtonVariant,
    /// Size of the button
    #[props(default)]
    pub size: ButtonSize,
    /// Whether the button is disabled
    #[props(default)]
    pub disabled: bool,
    /// Whether to display a loading spinner
    #[props(default)]
    pub loading: bool,
    /// HTML button type (defaults to "button")
    #[props(default = "button")]
    pub r#type: &'static str,
    /// Optional tooltip title
    #[props(default)]
    pub title: Option<String>,
    /// Optional accessibility aria-label
    #[props(default)]
    pub aria_label: Option<String>,
    /// Additional CSS classes
    #[props(default)]
    pub class: Option<String>,
    /// Optional click handler
    #[props(default)]
    pub onclick: Option<EventHandler<MouseEvent>>,
    /// Optional child elements (icons, custom content)
    #[props(default)]
    pub children: Element,
}

/// DRY Button component used across the application.
#[component]
pub fn Button(props: ButtonProps) -> Element {
    let size_classes = match props.size {
        ButtonSize::Sm => "px-2.5 py-1 text-xs rounded-lg gap-1.5",
        ButtonSize::Md => "px-3.5 py-1.5 text-sm rounded-lg gap-2",
    };

    let variant_classes = match props.variant {
        ButtonVariant::Primary => {
            "bg-sky-600 hover:bg-sky-700 active:bg-sky-800 text-white font-semibold shadow-xs border border-transparent focus-visible:ring-2 focus-visible:ring-sky-500"
        }
        ButtonVariant::Secondary => {
            "border border-slate-300 dark:border-slate-600 bg-white dark:bg-slate-800 hover:bg-slate-100 dark:hover:bg-slate-700 text-slate-800 dark:text-slate-200 font-medium shadow-xs focus-visible:ring-2 focus-visible:ring-sky-500"
        }
        ButtonVariant::Danger => {
            "border border-rose-200 dark:border-rose-800 bg-rose-50 dark:bg-rose-950/40 hover:bg-rose-100 dark:hover:bg-rose-900/60 text-rose-700 dark:text-rose-300 font-medium focus-visible:ring-2 focus-visible:ring-rose-500"
        }
        ButtonVariant::Accent => {
            "border border-transparent bg-sky-600 hover:bg-sky-700 active:bg-sky-800 text-white font-bold shadow-md ring-2 ring-sky-400/40 focus-visible:ring-2 focus-visible:ring-sky-500"
        }
    };

    let disabled_classes = if props.disabled || props.loading {
        "opacity-60 cursor-not-allowed pointer-events-none"
    } else {
        "cursor-pointer active:scale-[0.98] transition-all duration-150"
    };

    let custom_classes = props.class.as_deref().unwrap_or("");

    rsx! {
        button {
            r#type: props.r#type,
            disabled: props.disabled || props.loading,
            title: props.title.as_deref().unwrap_or_default(),
            aria_label: props.aria_label.as_deref().unwrap_or_default(),
            class: "inline-flex items-center justify-center font-sans select-none {size_classes} {variant_classes} {disabled_classes} {custom_classes}",
            onclick: move |evt| {
                if !props.disabled && !props.loading {
                    if let Some(handler) = props.onclick.as_ref() {
                        handler.call(evt);
                    }
                }
            },
            if props.loading {
                span {
                    class: "inline-block w-3.5 h-3.5 border-2 border-current border-t-transparent rounded-full animate-spin",
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
