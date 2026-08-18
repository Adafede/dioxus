// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::features::explore::form_actions::FormAction;
use crate::features::explore::selectors::use_criteria_selector;
use crate::i18n::{TextKey, t};
use crate::models::ElementState;
use crate::state::use_form_criteria_context;
use dioxus::prelude::*;
use ui::prelude::*;

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
        div { style: range_pair_style(),
            label { style: crate::ui::style_constants::shared::label_small_style(), "{label}" }
            select {
                style: form_input_small_style(),
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

/// Formula element count range pair component (C, H, N, etc.).
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
        div { style: formula_num_pair_style(),
            p { style: formula_num_label_style(), "{label}" }
            div { class: "formula-minmax-grid",
                div { style: range_pair_style(),
                    label { style: crate::ui::style_constants::shared::label_small_style(), "{t(locale, TextKey::MinCount)}" }
                    input {
                        r#type: "number",
                        style: formula_input_small_style(),
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
                div { style: range_pair_style(),
                    label { style: crate::ui::style_constants::shared::label_small_style(), "{t(locale, TextKey::MaxCount)}" }
                    input {
                        r#type: "number",
                        style: formula_input_small_style(),
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
    let criteria = criteria.read();
    let enabled = criteria.formula_enabled;

    rsx! {
        div { style: form_section_style(),
            label { style: radio_label_style(),
                input {
                    r#type: "checkbox",
                    checked: enabled,
                    onchange: move |e| ctx.update(FormAction::FormulaEnabled(e.checked())),
                }
                "{t(locale, TextKey::FormulaFilter)}"
            }

            if enabled {
                div { style: formula_exact_row_style(),
                    label { style: crate::ui::style_constants::shared::label_small_style(), r#for: "formula-exact",
                        "{t(locale, TextKey::ExactFormula)}"
                    }
                    input {
                        id: "formula-exact",
                        name: "formula_exact",
                        r#type: "text",
                        style: formula_exact_input_style(),
                        autocomplete: "off",
                        spellcheck: "false",
                        placeholder: "C15H10O5",
                        value: "{criteria.formula_exact}",
                        oninput: move |e| ctx.update(FormAction::FormulaExact(e.value())),
                    }
                }

                div { class: "formula-grid",
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
                div { class: "formula-grid",
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
fn radio_label_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .align_items("center")
        .gap("6px")
        .font_size("var(--fs-0)")
        .cursor("pointer")
        .color("var(--text2)")
        .build()
}

fn range_pair_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .flex_direction("column")
        .gap("3px")
        .build()
}

fn form_input_small_style() -> String {
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

fn formula_num_pair_style() -> String {
    StyleBuilder::new()
        .border("1px solid var(--panel-border)")
        .border_radius("10px")
        .background_color("var(--panel-bg-soft)")
        .padding("8px")
        .display("flex")
        .flex_direction("column")
        .gap("6px")
        .build()
}

fn formula_num_label_style() -> String {
    StyleBuilder::new().color("var(--text2)").build()
}

fn formula_input_small_style() -> String {
    StyleBuilder::new()
        .width("100%")
        .property("min-width", "6ch")
        .padding("9px 6px")
        .background_color("var(--surface)")
        .border("1px solid var(--border)")
        .border_radius("4px")
        .color("var(--text)")
        .font_size("var(--fs-ui)")
        .font_family("var(--sans)")
        .property("font-variant-numeric", "tabular-nums")
        .property("transition", "border-color .15s")
        .build()
}

fn formula_exact_row_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .flex_direction("column")
        .gap("5px")
        .padding("10px 12px")
        .border("1px solid var(--panel-border)")
        .border_radius("12px")
        .background_color("var(--panel-bg-soft)")
        .property("margin-top", "4px")
        .border_left("1px solid var(--border)")
        .property("padding-left", "10px")
        .build()
}

fn formula_exact_input_style() -> String {
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
