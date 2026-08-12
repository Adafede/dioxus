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
        div { style: form_section_style(),
            if !label.is_empty() {
                label { r#for: "{id}", style: form_label_style(), "{label}" }
            }

            input {
                id: "{id}",
                r#type: "text",
                style: form_input_style(),
                value: "{value}",
                placeholder: placeholder.unwrap_or_default(),
                aria_describedby: if !hint_id.is_empty() { "{hint_id}" } else { "" },
                oninput: move |e| on_change.call(e.value()),
            }

            if let Some(hint_text) = hint {
                p { id: "{hint_id}", style: form_hint_style(), "{hint_text}" }
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
        div { style: form_section_style(),
            label { style: form_label_style(), "{label}" }

            div { style: range_inputs_style(),
                div { style: range_pair_style(),
                    label { style: form_label_small_style(), r#for: "{min_id}", "{min_label}" }
                    input {
                        id: "{min_id}",
                        r#type: "number",
                        style: form_input_style(),
                        value: "{min_value}",

                        oninput: move |e| on_min_change.call(parse_f64(&e.value())),
                    }
                }

                div { style: range_pair_style(),
                    label { style: form_label_small_style(), r#for: "{max_id}", "{max_label}" }
                    input {
                        id: "{max_id}",
                        r#type: "number",
                        style: form_input_style(),
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

/// Helper to construct form section styles.
/// Note: This function is called from RSX macros which Clippy cannot analyze through
/// the proc macro expansion, resulting in a false positive "never used" warning.
#[expect(dead_code)]
fn form_section_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .flex_direction("column")
        .gap("5px")
        .padding("10px 12px")
        .border("1px solid var(--panel-border)")
        .border_radius("12px")
        .background_color("var(--panel-bg-soft)")
        .build()
}

/// Helper to construct form label styles.
/// Note: This function is called from RSX macros which Clippy cannot analyze through
/// the proc macro expansion, resulting in a false positive "never used" warning.
#[expect(dead_code)]
fn form_label_style() -> String {
    StyleBuilder::new()
        .font_size("var(--fs-0)")
        .font_weight("700")
        .color("var(--critical-text)")
        .property("text-transform", "uppercase")
        .property("letter-spacing", "0.08em")
        .build()
}

/// Helper to construct small form label styles.
/// Note: This function is called from RSX macros which Clippy cannot analyze through
/// the proc macro expansion, resulting in a false positive "never used" warning.
#[expect(dead_code)]
fn form_label_small_style() -> String {
    crate::ui::style_constants::shared::label_small_style()
}

/// Helper to construct form input styles.
/// Note: This function is called from RSX macros which Clippy cannot analyze through
/// the proc macro expansion, resulting in a false positive "never used" warning.
#[expect(dead_code)]
fn form_input_style() -> String {
    StyleBuilder::new()
        .width("100%")
        .background_color("var(--surface)")
        .border("1px solid var(--border)")
        .border_radius("4px")
        .color("var(--text)")
        .padding("9px 11px")
        .font_size("var(--fs-ui)")
        .font_family("var(--sans)")
        .property("transition", "border-color .15s")
        .build()
}

/// Helper to construct form hint styles.
/// Note: This function is called from RSX macros which Clippy cannot analyze through
/// the proc macro expansion, resulting in a false positive "never used" warning.
#[expect(dead_code)]
fn form_hint_style() -> String {
    StyleBuilder::new()
        .font_size("var(--fs-0)")
        .color("var(--text2)")
        .build()
}

/// Helper to construct range inputs styles.
/// Note: This function is called from RSX macros which Clippy cannot analyze through
/// the proc macro expansion, resulting in a false positive "never used" warning.
#[expect(dead_code)]
fn range_inputs_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .align_items("flex-end")
        .gap("8px")
        .build()
}

/// Helper to construct range pair styles.
/// Note: This function is called from RSX macros which Clippy cannot analyze through
/// the proc macro expansion, resulting in a false positive "never used" warning.
#[expect(dead_code)]
fn range_pair_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .flex_direction("column")
        .gap("3px")
        .build()
}

fn button_primary_style() -> String {
    crate::ui::style_constants::primary_buttons::button_primary_style()
}
