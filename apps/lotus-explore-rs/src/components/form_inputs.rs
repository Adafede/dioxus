// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Focused, reusable form input components.

use crate::components::ui::{Button, ButtonSize, ButtonVariant};
use crate::hooks::use_locale;
use crate::i18n::{TextKey, t};
use crate::ui::classes;
use dioxus::prelude::*;

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
        div { class: "flex flex-col gap-1",
            if !label.is_empty() {
                label {
                    r#for: "{id}",
                    class: "{classes::LABEL}",
                    "{label}"
                }
            }

            input {
                id: "{id}",
                r#type: "text",
                class: "{classes::INPUT}",
                value: "{value}",
                placeholder: placeholder.unwrap_or_default(),
                aria_describedby: if !hint_id.is_empty() { "{hint_id}" } else { "" },
                oninput: move |e| on_change.call(e.value()),
            }

            if let Some(hint_text) = hint {
                p { id: "{hint_id}", class: "{classes::HINT}", "{hint_text}" }
            }
        }
    }
}

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
        div { class: "flex flex-col gap-1.5",
            label { class: "{classes::LABEL}", "{label}" }

            div { class: "flex items-center gap-2",
                div { class: "flex flex-1 flex-col gap-0.5",
                    label { class: "{classes::MICRO_LABEL}", r#for: "{min_id}", "{min_label}" }
                    input {
                        id: "{min_id}",
                        r#type: "number",
                        class: "{classes::INPUT_SM}",
                        value: "{min_value}",
                        oninput: move |e| on_min_change.call(parse_f64(&e.value())),
                    }
                }
                span { class: "self-end pb-1.5 text-subtle", "–" }
                div { class: "flex flex-1 flex-col gap-0.5",
                    label { class: "{classes::MICRO_LABEL}", r#for: "{max_id}", "{max_label}" }
                    input {
                        id: "{max_id}",
                        r#type: "number",
                        class: "{classes::INPUT_SM}",
                        value: "{max_value}",
                        oninput: move |e| on_max_change.call(parse_f64(&e.value())),
                    }
                }
            }
        }
    }
}

#[component]
pub fn SearchButton(
    #[props(default = false)] loading: bool,
    #[props(default = false)] is_dirty: bool,
    on_click: EventHandler<()>,
) -> Element {
    let locale = use_locale();

    rsx! {
        Button {
            label: if loading {
                t(locale, TextKey::Searching).to_string()
            } else {
                t(locale, TextKey::Search).to_string()
            },
            variant: if is_dirty { ButtonVariant::Accent } else { ButtonVariant::Primary },
            size: ButtonSize::Md,
            loading,
            disabled: loading,
            r#type: "submit",
            class: "w-full",
            aria_label: t(locale, TextKey::RunSearch).to_string(),
            onclick: move |_| on_click.call(()),
        }
    }
}
