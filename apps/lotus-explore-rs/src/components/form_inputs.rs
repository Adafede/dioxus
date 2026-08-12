// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Focused, reusable form input components.
//!
//! These components are building blocks for larger forms. They accept props
//! directly rather than relying on context, making them easy to test and reuse.
//!
//! Future: Can be enhanced to use EnhancedFormContext for context-aware version.

use crate::hooks::use_locale;
use crate::i18n::{TextKey, t};
use dioxus::prelude::*;
use ui::prelude::*;

/// Generic reusable text input component
#[component]
pub fn TextInput(
    id: String,
    label: String,
    value: String,
    on_change: EventHandler<String>,
    placeholder: Option<String>,
    hint: Option<String>,
) -> Element {
    let hint_id = if hint.is_some() {
        format!("{id}-hint")
    } else {
        String::new()
    };

    rsx! {
        div { class: "form-section",
            if !label.is_empty() {
                label { r#for: "{id}", class: "form-label", "{label}" }
            }

            input {
                id: "{id}",
                r#type: "text",
                class: "form-input",
                value: "{value}",
                placeholder: placeholder.unwrap_or_default(),
                aria_describedby: if !hint_id.is_empty() { "{hint_id}" } else { "" },
                oninput: move |e| on_change.call(e.value()),
            }

            if let Some(hint_text) = hint {
                p { id: "{hint_id}", class: "form-hint sr-hint", "{hint_text}" }
            }
        }
    }
}

/// Generic number range input component (reusable for mass, year, etc.)
#[component]
pub fn RangeInput(
    label: String,
    min_value: f64,
    max_value: f64,
    on_min_change: EventHandler<f64>,
    on_max_change: EventHandler<f64>,
    min_label: String,
    max_label: String,
) -> Element {
    let parse_f64 = |s: &str| s.parse::<f64>().unwrap_or(0.0);
    let min_id = "range-min-input";
    let max_id = "range-max-input";

    rsx! {
        div { class: "form-section",
            label { class: "form-label", "{label}" }

            div { class: "range-inputs",
                div { class: "range-pair",
                    label { class: "form-label sm", r#for: "{min_id}", "{min_label}" }
                    input {
                        id: "{min_id}",
                        r#type: "number",
                        class: "form-input",
                        value: "{min_value}",

                        oninput: move |e| on_min_change.call(parse_f64(&e.value())),
                    }
                }

                div { class: "range-pair",
                    label { class: "form-label sm", r#for: "{max_id}", "{max_label}" }
                    input {
                        id: "{max_id}",
                        r#type: "number",
                        class: "form-input",
                        value: "{max_value}",

                        oninput: move |e| on_max_change.call(parse_f64(&e.value())),
                    }
                }
            }
        }
    }
}

/// Simplified search button
#[component]
pub fn SearchButton(on_click: EventHandler<()>) -> Element {
    let locale = use_locale();

    rsx! {
        button {
            r#type: "button",
            aria_label: "{t(locale, TextKey::RunSearch)}",
            style: button_primary_style(),
            onclick: move |_| on_click.call(()),
            "{t(locale, TextKey::Search)}"
        }
    }
}

fn button_primary_style() -> String {
    StyleBuilder::new()
        .display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .gap("6px")
        .border("1px solid var(--btn-primary-bg)")
        .border_radius("4px")
        .property("min-height", "40px")
        .padding("8px 14px")
        .font_size("var(--fs-0)")
        .font_weight("600")
        .cursor("pointer")
        .background_color("var(--btn-primary-bg)")
        .color("#fff")
        .box_shadow("var(--shadow-xs)")
        .property(
            "transition",
            "background .15s, border-color .15s, box-shadow .15s, transform .12s ease",
        )
        .build()
}
