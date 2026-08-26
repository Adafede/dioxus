// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::features::explore::form_actions::FormAction;
use crate::features::explore::interactions::use_explore_interactions;
use crate::features::explore::selectors::use_criteria_selector;
use crate::i18n::{TextKey, t};
use crate::state::use_form_criteria_context;
use crate::ui::classes;
use dioxus::prelude::*;

use super::shared::{normalized_year_input_max, parse_f64_input, parse_u16_input};

#[component]
pub fn TaxonInput() -> Element {
    let locale = crate::hooks::use_locale();
    let ctx = use_form_criteria_context();
    let interactions = use_explore_interactions();
    let taxon = use_criteria_selector(ctx.criteria, |c| c.taxon.clone());

    rsx! {
        div { class: "{classes::SECTION}",
            label {
                class: "{classes::LABEL}",
                r#for: "taxon-input",
                "{t(locale, TextKey::Taxon)}"
            }
            input {
                id: "taxon-input",
                name: "taxon",
                r#type: "text",
                autocomplete: "off",
                spellcheck: "false",
                placeholder: "{t(locale, TextKey::TaxonPlaceholder)}",
                value: "{taxon.read()}",
                class: "{classes::INPUT}",
                oninput: move |e| ctx.update(FormAction::Taxon(e.value())),
                onkeydown: move |e| {
                    if e.key() == Key::Enter {
                        interactions.search();
                    }
                },
            }
            p { class: "{classes::HINT}", "{t(locale, TextKey::TaxonHint)}" }
        }
    }
}

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
            class: "{classes::SECTION}",
            p { id: "mass-range-label", class: "{classes::LABEL}", "{t(locale, TextKey::MolecularMass)}" }
            div { class: "grid grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-end gap-2",
                div { class: "flex min-w-0 flex-col gap-0.5",
                    label {
                        class: "{classes::MICRO_LABEL}",
                        r#for: "mass-min",
                        "{t(locale, TextKey::Min)}"
                    }
                    input {
                        id: "mass-min",
                        name: "mass_min",
                        r#type: "number",
                        min: "0",
                        max: "10000",
                        step: "1",
                        value: "{min_value}",
                        class: "{classes::INPUT_SM}",
                        oninput: move |e| {
                            if let Some(v) = parse_f64_input(&e.value()) {
                                ctx.update(FormAction::MassMin(v));
                            }
                        },
                    }
                }
                span { aria_hidden: "true", class: "pb-2 text-subtle", "-" }
                div { class: "flex min-w-0 flex-col gap-0.5",
                    label {
                        class: "{classes::MICRO_LABEL}",
                        r#for: "mass-max",
                        "{t(locale, TextKey::Max)}"
                    }
                    input {
                        id: "mass-max",
                        name: "mass_max",
                        r#type: "number",
                        min: "0",
                        max: "10000",
                        step: "1",
                        value: "{max_value}",
                        class: "{classes::INPUT_SM}",
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
            class: "{classes::SECTION}",
            p { id: "year-range-label", class: "{classes::LABEL}", "{t(locale, TextKey::PublicationYear)}" }
            div { class: "grid grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-end gap-2",
                div { class: "flex min-w-0 flex-col gap-0.5",
                    label {
                        class: "{classes::MICRO_LABEL}",
                        r#for: "year-min",
                        "{t(locale, TextKey::YearFrom)}"
                    }
                    input {
                        id: "year-min",
                        name: "year_min",
                        r#type: "number",
                        min: "{DEFAULT_YEAR_MIN}",
                        max: "{current}",
                        step: "1",
                        value: "{min_value}",
                        class: "{classes::INPUT_SM}",
                        oninput: move |e| {
                            if let Some(v) = parse_u16_input(&e.value()) {
                                ctx.update(FormAction::YearMin(v));
                            }
                        },
                    }
                }
                span { aria_hidden: "true", class: "pb-2 text-subtle", "-" }
                div { class: "flex min-w-0 flex-col gap-0.5",
                    label {
                        class: "{classes::MICRO_LABEL}",
                        r#for: "year-max",
                        "{t(locale, TextKey::YearTo)}"
                    }
                    input {
                        id: "year-max",
                        name: "year_max",
                        r#type: "number",
                        min: "{DEFAULT_YEAR_MIN}",
                        max: "{current}",
                        step: "1",
                        value: "{max_value}",
                        class: "{classes::INPUT_SM}",
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
