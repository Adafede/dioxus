// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::features::explore::form_actions::FormAction;
use crate::features::explore::interactions::use_explore_interactions;
use crate::features::explore::selectors::use_criteria_selector;
use crate::i18n::{TextKey, t};
use crate::state::use_form_criteria_context;
use dioxus::prelude::*;
use ui::prelude::*;

use super::shared::{normalized_year_input_max, parse_f64_input, parse_u16_input};

/// Taxon input section - reads value from `FormCriteriaContext`.
#[component]
pub fn TaxonInput() -> Element {
    let locale = crate::hooks::use_locale();
    let ctx = use_form_criteria_context();
    let interactions = use_explore_interactions();
    let taxon = use_criteria_selector(ctx.criteria, |c| c.taxon.clone());

    rsx! {
        div { style: section_card_style(),
            label { style: label_base_style(), r#for: "taxon-input", "{t(locale, TextKey::Taxon)}" }
            input {
                id: "taxon-input",
                r#type: "text",
                autocomplete: "off",
                spellcheck: "false",
                placeholder: "{t(locale, TextKey::TaxonPlaceholder)}",
                value: "{taxon.read()}",
                oninput: move |e| ctx.update(FormAction::Taxon(e.value())),
                onkeydown: move |e| {
                    if e.key() == Key::Enter {
                        interactions.search();
                    }
                },
                style: input_base_style(),
            }
            p { style: hint_text_style(), "{t(locale, TextKey::TaxonHint)}" }
        }
    }
}

/// Mass range input section - reads values from `FormCriteriaContext`.
#[component]
pub fn MassRangeInput() -> Element {
    let locale = crate::hooks::use_locale();
    let ctx = use_form_criteria_context();
    let mass_range = use_criteria_selector(ctx.criteria, |c| (c.mass_min, c.mass_max));
    let (min_value, max_value) = *mass_range.read();

    rsx! {
        div {
            role: "group",
            aria_labelledby: "mass-range-label",
            style: section_card_style(),
            p { id: "mass-range-label", style: label_base_style(), "{t(locale, TextKey::MolecularMass)}" }
            div { style: range_inputs_pair_style(),
                div { style: range_pair_style(),
                    label { style: label_small_style(), r#for: "mass-min", "{t(locale, TextKey::Min)}" }
                    input {
                        id: "mass-min",
                        r#type: "number",
                        min: "0",
                        max: "10000",
                        step: "1",
                        value: "{min_value}",
                        oninput: move |e| {
                            if let Some(v) = parse_f64_input(&e.value()) {
                                ctx.update(FormAction::MassMin(v));
                            }
                        },
                        style: input_base_style(),
                    }
                }
                span { aria_hidden: "true", style: range_separator_style(), "-" }
                div { style: range_pair_style(),
                    label { style: label_small_style(), r#for: "mass-max", "{t(locale, TextKey::Max)}" }
                    input {
                        id: "mass-max",
                        r#type: "number",
                        min: "0",
                        max: "10000",
                        step: "1",
                        value: "{max_value}",
                        oninput: move |e| {
                            if let Some(v) = parse_f64_input(&e.value()) {
                                ctx.update(FormAction::MassMax(v));
                            }
                        },
                        style: input_base_style(),
                    }
                }
            }
        }
    }
}

/// Year range input section - reads values from `FormCriteriaContext`.
#[component]
pub fn YearRangeInput() -> Element {
    use crate::models::DEFAULT_YEAR_MIN;

    let locale = crate::hooks::use_locale();
    let ctx = use_form_criteria_context();
    let year_range = use_criteria_selector(ctx.criteria, |c| (c.year_min, c.year_max));
    let (min_value, max_value) = *year_range.read();
    let current = normalized_year_input_max(crate::models::current_year());

    rsx! {
        div {
            role: "group",
            aria_labelledby: "year-range-label",
            style: section_card_style(),
            p { id: "year-range-label", style: label_base_style(), "{t(locale, TextKey::PublicationYear)}" }
            div { style: range_inputs_pair_style(),
                div { style: range_pair_style(),
                    label { style: label_small_style(), r#for: "year-min", "{t(locale, TextKey::YearFrom)}" }
                    input {
                        id: "year-min",
                        r#type: "number",
                        min: "{DEFAULT_YEAR_MIN}",
                        max: "{current}",
                        step: "1",
                        value: "{min_value}",
                        oninput: move |e| {
                            if let Some(v) = parse_u16_input(&e.value()) {
                                ctx.update(FormAction::YearMin(v));
                            }
                        },
                        style: input_base_style(),
                    }
                }
                span { aria_hidden: "true", style: range_separator_style(), "-" }
                div { style: range_pair_style(),
                    label { style: label_small_style(), r#for: "year-max", "{t(locale, TextKey::YearTo)}" }
                    input {
                        id: "year-max",
                        r#type: "number",
                        min: "{DEFAULT_YEAR_MIN}",
                        max: "{current}",
                        step: "1",
                        value: "{max_value}",
                        oninput: move |e| {
                            if let Some(v) = parse_u16_input(&e.value()) {
                                ctx.update(FormAction::YearMax(v));
                            }
                        },
                        style: input_base_style(),
                    }
                }
            }
        }
    }
}

fn section_card_style() -> String {
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

fn label_base_style() -> String {
    crate::ui::style_constants::shared::label_base_style()
}

fn label_small_style() -> String {
    crate::ui::style_constants::shared::label_small_style()
}

fn input_base_style() -> String {
    crate::ui::style_constants::shared::input_base_style()
}

fn hint_text_style() -> String {
    crate::ui::style_constants::shared::hint_text_style()
}

fn range_inputs_pair_style() -> String {
    StyleBuilder::new()
        .display("grid")
        .property(
            "grid-template-columns",
            "minmax(0, 1fr) auto minmax(0, 1fr)",
        )
        .align_items("end")
        .gap("8px")
        .build()
}

fn range_pair_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .flex_direction("column")
        .gap("3px")
        .build()
}

fn range_separator_style() -> String {
    StyleBuilder::new()
        .color("var(--text3)")
        .property("padding-bottom", "8px")
        .build()
}
