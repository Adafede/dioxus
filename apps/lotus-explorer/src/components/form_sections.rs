// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Focused sub-components for SearchPanel form sections.
//!
//! Most sections are "context-aware" — they consume [`FormCriteriaContext`]
//! directly rather than receiving data and callbacks as props.  This keeps
//! `SearchPanel` thin and eliminates the 36-prop `FormulaSection` API that
//! existed previously.

use crate::features::explore::form_actions::FormAction;
use crate::features::explore::selectors::use_criteria_selector;
use crate::i18n::{TextKey, t};
use crate::models::*;
use crate::state::use_form_criteria_context;
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
struct FormulaSectionState {
    formula_enabled: bool,
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
}

fn parse_f64_input(raw: &str) -> Option<f64> {
    raw.parse::<f64>().ok()
}

fn parse_u16_input(raw: &str) -> Option<u16> {
    raw.parse::<u16>().ok()
}

#[must_use]
fn normalized_year_input_max(current_year: u16) -> u16 {
    current_year.max(crate::models::DEFAULT_YEAR_MIN)
}

/// Taxon input section — reads value from `FormCriteriaContext`.
///
/// Only prop: `on_search` to handle Enter-key submission.  The taxon value and
/// its mutation are handled internally via context, eliminating the `value` and
/// `on_input` props that previously had to flow through `SearchPanel`.
#[component]
pub fn TaxonInput(on_search: EventHandler<()>) -> Element {
    let locale = crate::hooks::use_locale();
    let ctx = use_form_criteria_context();
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
                        on_search.call(());
                    }
                },
            }
            p { class: "form-hint", "{t(locale, TextKey::TaxonHint)}" }
        }
    }
}

/// Mass range input section — reads values from `FormCriteriaContext`.
///
/// Zero props: both values and event handlers are covered by context dispatch.
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
                span { class: "range-sep range-sep--pair", "aria-hidden": "true", "–" }
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

/// Year range input section — reads values from `FormCriteriaContext`.
///
/// Zero props: both values and event handlers are covered by context dispatch.
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
                span { class: "range-sep range-sep--pair", "aria-hidden": "true", "–" }
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
        div { class: "formula-num-pair",
            p { class: "form-label sm formula-num-label", "{label}" }
            div { class: "formula-minmax-grid",
                div { class: "range-pair",
                    label { class: "form-label sm", "{t(locale, TextKey::MinCount)}" }
                    input {
                        r#type: "number",
                        class: "form-input sm",
                        min: "0",
                        max: "10000",
                        aria_label: "{label} {t(locale, TextKey::MinCountAria)}",
                        value: "{min_value}",
                        oninput: move |e| {
                            if let Some(v) = parse_u16_input(&e.value()) {
                                on_min.call(v);
                            }
                        },
                    }
                }
                div { class: "range-pair",
                    label { class: "form-label sm", "{t(locale, TextKey::MaxCount)}" }
                    input {
                        r#type: "number",
                        class: "form-input sm",
                        min: "0",
                        max: "10000",
                        aria_label: "{label} {t(locale, TextKey::MaxCountAria)}",
                        value: "{max_value}",
                        oninput: move |e| {
                            if let Some(v) = parse_u16_input(&e.value()) {
                                on_max.call(v);
                            }
                        },
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{normalized_year_input_max, parse_f64_input, parse_u16_input};

    #[test]
    fn parse_f64_input_accepts_valid_numbers_and_rejects_invalid_text() {
        assert_eq!(parse_f64_input("42"), Some(42.0));
        assert_eq!(parse_f64_input("2.5"), Some(2.5));
        assert_eq!(parse_f64_input("abc"), None);
    }

    #[test]
    fn parse_u16_input_accepts_positive_integers_only() {
        assert_eq!(parse_u16_input("007"), Some(7));
        assert_eq!(parse_u16_input("65535"), Some(u16::MAX));
        assert_eq!(parse_u16_input("-1"), None);
        assert_eq!(parse_u16_input("12.5"), None);
    }

    #[test]
    fn normalized_year_input_max_never_drops_below_default_floor() {
        assert_eq!(normalized_year_input_max(2030), 2030);
        assert_eq!(
            normalized_year_input_max(crate::models::DEFAULT_YEAR_MIN),
            crate::models::DEFAULT_YEAR_MIN
        );
        assert_eq!(
            normalized_year_input_max(1700),
            crate::models::DEFAULT_YEAR_MIN
        );
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
                onchange: move |e| on_change.call(e.value().parse::<ElementState>().unwrap_or_default()),
                option { value: "allowed", "{t(locale, TextKey::ElementStateAllowed)}" }
                option { value: "required", "{t(locale, TextKey::ElementStateRequired)}" }
                option { value: "excluded", "{t(locale, TextKey::ElementStateExcluded)}" }
            }
        }
    }
}

/// Formula filter controls section — reads and writes `FormCriteriaContext`.
///
/// ## Why zero props?
///
/// The previous version had **36 props** (18 values + 18 callbacks) that all
/// had to pass through `SearchPanel`.  Because `FormulaSection` is always
/// rendered inside the `FormCriteriaContext` provider, it can read the live
/// criteria signal and dispatch `FormAction`s directly — no intermediary
/// callbacks, no prop drilling, zero allocation overhead from cloning
/// handler closures at every parent render.
#[component]
pub fn FormulaSection() -> Element {
    let locale = crate::hooks::use_locale();
    let ctx = use_form_criteria_context();
    let criteria = use_criteria_selector(ctx.criteria, |c| FormulaSectionState {
        formula_enabled: c.formula_enabled,
        formula_exact: c.formula_exact.clone(),
        c_min: c.c_min,
        c_max: c.c_max,
        h_min: c.h_min,
        h_max: c.h_max,
        n_min: c.n_min,
        n_max: c.n_max,
        o_min: c.o_min,
        o_max: c.o_max,
        p_min: c.p_min,
        p_max: c.p_max,
        s_min: c.s_min,
        s_max: c.s_max,
        f_state: c.f_state,
        cl_state: c.cl_state,
        br_state: c.br_state,
        i_state: c.i_state,
    });
    let criteria = criteria.read().clone();
    let enabled = criteria.formula_enabled;

    rsx! {
        div { class: "form-section",
            label { class: "radio-label",
                input {
                    r#type: "checkbox",
                    checked: enabled,
                    onchange: move |e| ctx.update(FormAction::FormulaEnabled(e.checked())),
                }
                "{t(locale, TextKey::FormulaFilter)}"
            }

            if enabled {
                div { class: "form-section nested formula-exact-row",
                    label { class: "form-label sm", r#for: "formula-exact",
                        "{t(locale, TextKey::ExactFormula)}"
                    }
                    input {
                        id: "formula-exact",
                        r#type: "text",
                        class: "form-input formula-exact-input",
                        autocomplete: "off",
                        spellcheck: "false",
                        placeholder: "C15H10O5",
                        value: "{criteria.formula_exact}",
                        oninput: move |e| ctx.update(FormAction::FormulaExact(e.value())),
                    }
                }

                div { class: "formula-grid formula-grid--pairs",
                    NumPair {
                        label: "C",
                        min_value: criteria.c_min,
                        max_value: criteria.c_max,
                        on_min: move |v| ctx.update(FormAction::CMin(v)),
                        on_max: move |v| ctx.update(FormAction::CMax(v)),
                    }
                    NumPair {
                        label: "H",
                        min_value: criteria.h_min,
                        max_value: criteria.h_max,
                        on_min: move |v| ctx.update(FormAction::HMin(v)),
                        on_max: move |v| ctx.update(FormAction::HMax(v)),
                    }
                    NumPair {
                        label: "N",
                        min_value: criteria.n_min,
                        max_value: criteria.n_max,
                        on_min: move |v| ctx.update(FormAction::NMin(v)),
                        on_max: move |v| ctx.update(FormAction::NMax(v)),
                    }
                    NumPair {
                        label: "O",
                        min_value: criteria.o_min,
                        max_value: criteria.o_max,
                        on_min: move |v| ctx.update(FormAction::OMin(v)),
                        on_max: move |v| ctx.update(FormAction::OMax(v)),
                    }
                    NumPair {
                        label: "P",
                        min_value: criteria.p_min,
                        max_value: criteria.p_max,
                        on_min: move |v| ctx.update(FormAction::PMin(v)),
                        on_max: move |v| ctx.update(FormAction::PMax(v)),
                    }
                    NumPair {
                        label: "S",
                        min_value: criteria.s_min,
                        max_value: criteria.s_max,
                        on_min: move |v| ctx.update(FormAction::SMin(v)),
                        on_max: move |v| ctx.update(FormAction::SMax(v)),
                    }
                }
                div { class: "formula-grid formula-grid--halogens",
                    ElemStateSelect {
                        label: "F",
                        value: criteria.f_state,
                        on_change: move |v| ctx.update(FormAction::FState(v)),
                    }
                    ElemStateSelect {
                        label: "Cl",
                        value: criteria.cl_state,
                        on_change: move |v| ctx.update(FormAction::ClState(v)),
                    }
                    ElemStateSelect {
                        label: "Br",
                        value: criteria.br_state,
                        on_change: move |v| ctx.update(FormAction::BrState(v)),
                    }
                    ElemStateSelect {
                        label: "I",
                        value: criteria.i_state,
                        on_change: move |v| ctx.update(FormAction::IState(v)),
                    }
                }
            }
        }
    }
}
