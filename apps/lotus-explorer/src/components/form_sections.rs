// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Focused sub-components for SearchPanel form sections.

use crate::i18n::{TextKey, t};
use crate::models::*;
use dioxus::prelude::*;

/// Taxon input section with label and hint.
#[component]
pub fn TaxonInput(
    value: String,
    on_input: EventHandler<String>,
    on_search: EventHandler<()>,
) -> Element {
    let locale = crate::hooks::use_locale();
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
                value: "{value}",
                oninput: move |e| on_input.call(e.value()),
                onkeydown: move |e| {
                    if e.key() == Key::Enter {
                        on_search.call(());
                    }
                },
            }
            p { class: "form-hint", "{t(locale, TextKey::TaxonHint)}" }
        }
    }
}

/// Mass range input section with min/max controls.
#[component]
pub fn MassRangeInput(
    min_value: f64,
    max_value: f64,
    on_min: EventHandler<f64>,
    on_max: EventHandler<f64>,
) -> Element {
    let locale = crate::hooks::use_locale();
    rsx! {
        fieldset { class: "form-section", style: "border:0;padding:0;margin:0;",
            legend { class: "form-label", "{t(locale, TextKey::MolecularMass)}" }
            div { class: "range-inputs",
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
                            if let Ok(v) = e.value().parse::<f64>() {
                                on_min.call(v);
                            }
                        },
                    }
                }
                span { class: "range-sep", "–" }
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
                            if let Ok(v) = e.value().parse::<f64>() {
                                on_max.call(v);
                            }
                        },
                    }
                }
            }
        }
    }
}

/// Year range input section with from/to controls.
#[component]
pub fn YearRangeInput(
    min_value: u16,
    max_value: u16,
    on_min: EventHandler<u16>,
    on_max: EventHandler<u16>,
) -> Element {
    use crate::components::search_panel::DEFAULT_YEAR_MIN;
    let locale = crate::hooks::use_locale();
    let current = crate::components::search_panel::current_year();

    rsx! {
        fieldset { class: "form-section", style: "border:0;padding:0;margin:0;",
            legend { class: "form-label", "{t(locale, TextKey::PublicationYear)}" }
            div { class: "range-inputs",
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
                            if let Ok(v) = e.value().parse::<u16>() {
                                on_min.call(v);
                            }
                        },
                    }
                }
                span { class: "range-sep", "–" }
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
                            if let Ok(v) = e.value().parse::<u16>() {
                                on_max.call(v);
                            }
                        },
                    }
                }
            }
        }
    }
}

/// Formula element count range pair component (C, H, N, etc).
#[component]
fn NumPair(
    label: &'static str,
    min_value: u16,
    max_value: u16,
    on_min: EventHandler<u16>,
    on_max: EventHandler<u16>,
) -> Element {
    let locale = crate::hooks::use_locale();

    rsx! {
        div { class: "range-pair",
            label { class: "form-label sm", "{label} {t(locale, TextKey::MinCount)}" }
            input {
                r#type: "number",
                class: "form-input sm",
                min: "0",
                max: "10000",
                aria_label: "{label} {t(locale, TextKey::MinCountAria)}",
                value: "{min_value}",
                oninput: move |e| {
                    if let Ok(v) = e.value().parse::<u16>() {
                        on_min.call(v);
                    }
                },
            }
            label { class: "form-label sm", "{label} {t(locale, TextKey::MaxCount)}" }
            input {
                r#type: "number",
                class: "form-input sm",
                min: "0",
                max: "10000",
                aria_label: "{label} {t(locale, TextKey::MaxCountAria)}",
                value: "{max_value}",
                oninput: move |e| {
                    if let Ok(v) = e.value().parse::<u16>() {
                        on_max.call(v);
                    }
                },
            }
        }
    }
}

/// Element requirement select component (F, Cl, Br, I).
#[component]
fn ElemStateSelect(
    label: &'static str,
    value: ElementState,
    on_change: EventHandler<ElementState>,
) -> Element {
    let locale = crate::hooks::use_locale();

    rsx! {
        div { class: "range-pair",
            label { class: "form-label sm", "{label}" }
            select {
                class: "form-input sm",
                aria_label: "{label} {t(locale, TextKey::ElementRequirement)}",
                value: "{value.as_str()}",
                onchange: move |e| on_change.call(ElementState::from_str(&e.value())),
                option { value: "allowed", "{t(locale, TextKey::ElementStateAllowed)}" }
                option { value: "required", "{t(locale, TextKey::ElementStateRequired)}" }
                option { value: "excluded", "{t(locale, TextKey::ElementStateExcluded)}" }
            }
        }
    }
}

/// Formula filter controls section (checkbox + elemental composition).
#[component]
pub fn FormulaSection(
    enabled: bool,
    formula_exact: String,
    c_min: u16,
    c_max: u16,
    h_min: u16,
    h_max: u16,
    n_min: u16,
    n_max: u16,
    o_min: u16,
    o_max: u16,
    p_min: u16,
    p_max: u16,
    s_min: u16,
    s_max: u16,
    f_state: ElementState,
    cl_state: ElementState,
    br_state: ElementState,
    i_state: ElementState,
    on_enabled: EventHandler<bool>,
    on_formula_exact: EventHandler<String>,
    on_c_min: EventHandler<u16>,
    on_c_max: EventHandler<u16>,
    on_h_min: EventHandler<u16>,
    on_h_max: EventHandler<u16>,
    on_n_min: EventHandler<u16>,
    on_n_max: EventHandler<u16>,
    on_o_min: EventHandler<u16>,
    on_o_max: EventHandler<u16>,
    on_p_min: EventHandler<u16>,
    on_p_max: EventHandler<u16>,
    on_s_min: EventHandler<u16>,
    on_s_max: EventHandler<u16>,
    on_f_state: EventHandler<ElementState>,
    on_cl_state: EventHandler<ElementState>,
    on_br_state: EventHandler<ElementState>,
    on_i_state: EventHandler<ElementState>,
) -> Element {
    let locale = crate::hooks::use_locale();

    rsx! {
        div { class: "form-section",
            label { class: "radio-label",
                input {
                    r#type: "checkbox",
                    checked: enabled,
                    onchange: move |e| on_enabled.call(e.checked()),
                }
                "{t(locale, TextKey::FormulaFilter)}"
            }

            if enabled {
                div { class: "form-section nested",
                    label { class: "form-label sm", r#for: "formula-exact",
                        "{t(locale, TextKey::ExactFormula)}"
                    }
                    input {
                        id: "formula-exact",
                        r#type: "text",
                        class: "form-input sm",
                        autocomplete: "off",
                        spellcheck: "false",
                        placeholder: "C15H10O5",
                        value: "{formula_exact}",
                        oninput: move |e| on_formula_exact.call(e.value()),
                    }
                }

                div { class: "range-inputs",
                    NumPair {
                        label: "C",
                        min_value: c_min,
                        max_value: c_max,
                        on_min: move |v| on_c_min.call(v),
                        on_max: move |v| on_c_max.call(v),
                    }
                    NumPair {
                        label: "H",
                        min_value: h_min,
                        max_value: h_max,
                        on_min: move |v| on_h_min.call(v),
                        on_max: move |v| on_h_max.call(v),
                    }
                    NumPair {
                        label: "N",
                        min_value: n_min,
                        max_value: n_max,
                        on_min: move |v| on_n_min.call(v),
                        on_max: move |v| on_n_max.call(v),
                    }
                }
                div { class: "range-inputs",
                    NumPair {
                        label: "O",
                        min_value: o_min,
                        max_value: o_max,
                        on_min: move |v| on_o_min.call(v),
                        on_max: move |v| on_o_max.call(v),
                    }
                    NumPair {
                        label: "P",
                        min_value: p_min,
                        max_value: p_max,
                        on_min: move |v| on_p_min.call(v),
                        on_max: move |v| on_p_max.call(v),
                    }
                    NumPair {
                        label: "S",
                        min_value: s_min,
                        max_value: s_max,
                        on_min: move |v| on_s_min.call(v),
                        on_max: move |v| on_s_max.call(v),
                    }
                }
                div { class: "range-inputs",
                    ElemStateSelect {
                        label: "F",
                        value: f_state,
                        on_change: move |v| on_f_state.call(v),
                    }
                    ElemStateSelect {
                        label: "Cl",
                        value: cl_state,
                        on_change: move |v| on_cl_state.call(v),
                    }
                    ElemStateSelect {
                        label: "Br",
                        value: br_state,
                        on_change: move |v| on_br_state.call(v),
                    }
                    ElemStateSelect {
                        label: "I",
                        value: i_state,
                        on_change: move |v| on_i_state.call(v),
                    }
                }
            }
        }
    }
}
