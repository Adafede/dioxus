// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::features::explore::form_actions::FormAction;
use crate::features::explore::interactions::use_explore_interactions;
use crate::features::explore::selectors::use_criteria_selector;
use crate::i18n::{TextKey, t};
use crate::state::use_form_criteria_context;
use dioxus::prelude::*;

use super::shared::{normalized_year_input_max, parse_f64_input, parse_u16_input};

/// Taxon input section - reads value from `FormCriteriaContext`.
#[component]
pub fn TaxonInput() -> Element {
    let locale = crate::hooks::use_locale();
    let ctx = use_form_criteria_context();
    let interactions = use_explore_interactions();
    let taxon = use_criteria_selector(ctx.criteria, |c| c.taxon.clone());

    rsx! {
        div { class: "form-section",
            label { class: "form-label", r#for: "taxon-input", "{t(locale, TextKey::Taxon)}" }
            input {
                id: "taxon-input",
                r#type: "text",
                class: "form-input",
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
            }
            p { class: "form-hint", "{t(locale, TextKey::TaxonHint)}" }
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
            class: "form-section",
            role: "group",
            aria_labelledby: "mass-range-label",
            p { id: "mass-range-label", class: "form-label", "{t(locale, TextKey::MolecularMass)}" }
            div { class: "range-inputs range-inputs--pair",
                div { class: "range-pair",
                    label { class: "form-label sm", r#for: "mass-min", "{t(locale, TextKey::Min)}" }
                    input {
                        id: "mass-min",
                        r#type: "number",
                        class: "form-input sm",
                        min: "0",
                        max: "10000",
                        step: "1",
                        value: "{min_value}",
                        oninput: move |e| {
                            if let Some(v) = parse_f64_input(&e.value()) {
                                ctx.update(FormAction::MassMin(v));
                            }
                        },
                    }
                }
                span { class: "range-sep range-sep--pair", "aria-hidden": "true", "-" }
                div { class: "range-pair",
                    label { class: "form-label sm", r#for: "mass-max", "{t(locale, TextKey::Max)}" }
                    input {
                        id: "mass-max",
                        r#type: "number",
                        class: "form-input sm",
                        min: "0",
                        max: "10000",
                        step: "1",
                        value: "{max_value}",
                        oninput: move |e| {
                            if let Some(v) = parse_f64_input(&e.value()) {
                                ctx.update(FormAction::MassMax(v));
                            }
                        },
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
            class: "form-section",
            role: "group",
            aria_labelledby: "year-range-label",
            p { id: "year-range-label", class: "form-label", "{t(locale, TextKey::PublicationYear)}" }
            div { class: "range-inputs range-inputs--pair",
                div { class: "range-pair",
                    label { class: "form-label sm", r#for: "year-min", "{t(locale, TextKey::YearFrom)}" }
                    input {
                        id: "year-min",
                        r#type: "number",
                        class: "form-input sm",
                        min: "{DEFAULT_YEAR_MIN}",
                        max: "{current}",
                        step: "1",
                        value: "{min_value}",
                        oninput: move |e| {
                            if let Some(v) = parse_u16_input(&e.value()) {
                                ctx.update(FormAction::YearMin(v));
                            }
                        },
                    }
                }
                span { class: "range-sep range-sep--pair", "aria-hidden": "true", "-" }
                div { class: "range-pair",
                    label { class: "form-label sm", r#for: "year-max", "{t(locale, TextKey::YearTo)}" }
                    input {
                        id: "year-max",
                        r#type: "number",
                        class: "form-input sm",
                        min: "{DEFAULT_YEAR_MIN}",
                        max: "{current}",
                        step: "1",
                        value: "{max_value}",
                        oninput: move |e| {
                            if let Some(v) = parse_u16_input(&e.value()) {
                                ctx.update(FormAction::YearMax(v));
                            }
                        },
                    }
                }
            }
        }
    }
}
