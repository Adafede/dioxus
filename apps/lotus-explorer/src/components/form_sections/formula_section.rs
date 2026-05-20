// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::features::explore::form_actions::FormAction;
use crate::features::explore::selectors::use_criteria_selector;
use crate::i18n::{TextKey, t};
use crate::models::ElementState;
use crate::state::use_form_criteria_context;
use dioxus::prelude::*;

use super::shared::{FormulaSectionState, parse_u16_input};

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

/// Formula filter controls section - reads and writes `FormCriteriaContext`.
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
