// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Search panel and its subsection components.
//!
//! # Architecture
//!
//! `SearchPanel` is now extremely thin — it owns only two concerns:
//! * Presenting the search button (loading state + dirty indicator).
//! * Delegating form sections to context-aware subcomponents.
//!
//! Every form section (`TaxonInput`, `StructureSection`, `MassRangeInput`,
//! `YearRangeInput`, `FormulaSection`) reads and writes `FormCriteriaContext`
//! directly.  Search execution is invoked via `ExploreInteractions` context,
//! including Enter-key submission from `TaxonInput`.
//!
//! **Props eliminated vs original:** 38 (18 values + 18 callbacks for formula,
//! plus value + on_input for taxon, plus 4 for mass, plus 4 for year).

pub use crate::components::form_sections::{
    FormulaSection, MassRangeInput, TaxonInput, YearRangeInput,
};
use crate::features::explore::{FormAction, use_criteria_selector};

#[path = "search_panel/structure_model.rs"]
mod structure_model;

use crate::components::form_inputs::SearchButton;
use crate::features::explore::{use_explore_interactions, use_lifecycle_selector};
use crate::i18n::{TextKey, t, threshold_label};
use crate::models::*;
use crate::queries::classify_structure;
use crate::state::{use_form_criteria_context, use_results_context};
use crate::ui::a11y_contract::{SEARCH_PANEL_BODY_ID, SEARCH_PANEL_HEADING_ID};
use dioxus::prelude::*;
use ui::prelude::*;

#[component]
pub fn SearchPanel() -> Element {
    let state = use_results_context();
    let form_ctx = use_form_criteria_context();
    let interactions = use_explore_interactions();
    let locale = crate::hooks::use_locale();

    // Loading flag subscribed via selector so result churn does not rerender the sidebar.
    let loading = *use_lifecycle_selector(state.explore, |lifecycle| lifecycle.loading).read();
    // Dirty flag: show affordance when form changed since last search.
    let is_dirty = form_ctx.is_dirty();

    rsx! {
        section {
            aria_label: "{t(locale, TextKey::SearchFilters)}",
            aria_labelledby: SEARCH_PANEL_HEADING_ID,
            style: panel_stack_style("18px 16px", "14px"),

            h2 { id: SEARCH_PANEL_HEADING_ID, style: sr_only_style(), "{t(locale, TextKey::SearchFilters)}" }

            div { id: SEARCH_PANEL_BODY_ID, style: panel_stack_style("0", "12px"),
                // All sections are zero-prop — they read FormCriteriaContext.
                TaxonInput {}
                StructureSection {}
                MassRangeInput {}
                YearRangeInput {}
                FormulaSection {}
            }

            if loading {
                button {
                    r#type: "submit",
                    disabled: true,
                    aria_label: "{t(locale, TextKey::RunSearch)}",
                    style: search_button_style(is_dirty),
                    span { class: "spinner-sm", "aria-hidden": "true" }
                    "{t(locale, TextKey::Searching)}"
                }
            } else {
                SearchButton { on_click: move |_| interactions.search() }
            }
        }
    }
}

// ── Structure section: SMILES + Molfile V2000/V3000 + Ketcher ────────────────

/// Structure input reads criteria from `FormCriteriaContext` — no props needed.
#[component]
fn StructureSection() -> Element {
    let locale = crate::hooks::use_locale();
    let ctx = use_form_criteria_context();
    let c = ctx.criteria;
    let structure_fields = use_criteria_selector(c, |criteria| {
        (
            criteria.smiles.clone(),
            criteria.smiles_search_type,
            criteria.smiles_threshold,
        )
    });
    let (smiles, smiles_search_type, smiles_threshold) = structure_fields.read().clone();
    let smiles_for_kind = smiles.clone();
    // Memoised classifier: `classify_structure` uppercases the entire Molfile
    // on every call.  Recompute only when the SMILES text changes.
    let kind = use_memo(move || classify_structure(&smiles_for_kind));
    let kind_value = *kind.read();
    let view_model = structure_model::build_structure_section_model(kind_value, smiles_search_type);

    rsx! {
        div { style: section_card_style(),
            label { style: label_base_style(), r#for: "smiles-input",
                "{t(locale, TextKey::StructureSmilesOrMol)}"
            }
            textarea {
                id: "smiles-input",
                spellcheck: "false",
                placeholder: "{t(locale, TextKey::StructurePlaceholder)}",
                value: "{smiles}",
                oninput: move |e| ctx.update(FormAction::Smiles(e.value())),
                rows: "4",
                style: textarea_base_style(),
            }
            if let Some(note_key) = view_model.note_key {
                p { style: hint_text_style(),
                    span { style: kind_pill_style(&view_model.kind_class), "{kind_value.label()}" }
                    span { "{t(locale, note_key)}" }
                }
            } else {
                p { style: hint_text_style(), "{t(locale, TextKey::StructureHintEmpty)}" }
            }

            fieldset { style: radio_group_style(),
                legend { style: sr_only_style(), "{t(locale, TextKey::StructureSearchMode)}" }
                label { style: radio_label_style(),
                    input {
                        r#type: "radio",
                        name: "stype",
                        checked: smiles_search_type == SmilesSearchType::Substructure,
                        onchange: move |_| {
                            ctx.update(FormAction::SmilesSearchType(SmilesSearchType::Substructure))
                        },
                    }
                    "{t(locale, TextKey::Substructure)}"
                }
                label { style: radio_label_style(),
                    input {
                        r#type: "radio",
                        name: "stype",
                        checked: smiles_search_type == SmilesSearchType::Similarity,
                        onchange: move |_| {
                            ctx.update(FormAction::SmilesSearchType(SmilesSearchType::Similarity))
                        },
                    }
                    "{t(locale, TextKey::Similarity)}"
                }
            }
            if view_model.show_similarity_threshold {
                div { style: threshold_section_style(),
                    label { style: label_small_style(), r#for: "threshold-input",
                        "{threshold_label(locale, smiles_threshold)}"
                    }
                    input {
                        id: "threshold-input",
                        r#type: "range",
                        min: "0.0",
                        max: "1.0",
                        step: "0.01",
                        value: "{smiles_threshold}",
                        aria_valuemin: "0",
                        aria_valuemax: "1",
                        aria_valuenow: "{smiles_threshold}",
                        oninput: move |e| {
                            if let Ok(v) = e.value().parse::<f64>() {
                                ctx.update(FormAction::SmilesThreshold(v));
                            }
                        },
                        style: range_input_style(),
                    }
                }
            }
        }
    }
}

// ── Ketcher editor panel (full-width, rendered in the main content area) ─────

/// Relative URL at which the Ketcher standalone build is served.
const KETCHER_URL: &str = "assets/ketcher/index.html";

#[component]
pub fn KetcherPanel() -> Element {
    let locale = crate::hooks::use_locale();
    rsx! {
        section {
            aria_label: "{t(locale, TextKey::KetcherSummary)}",
            style: ketcher_panel_style(),
            div { style: ketcher_wrap_style(),
                p { style: hint_text_style(),
                    "{t(locale, TextKey::KetcherHintA)}"
                    strong { "{t(locale, TextKey::KetcherSummary)}" }
                    "{t(locale, TextKey::KetcherHintB)}"
                    em { "{t(locale, TextKey::EditCopyDaylightSmiles)}" }
                    "{t(locale, TextKey::KetcherHintC)}"
                    em { "{t(locale, TextKey::CopyExtendedSmilesMol)}" }
                    "{t(locale, TextKey::KetcherHintD)}"
                }
                iframe {
                    src: "{KETCHER_URL}",
                    title: "{t(locale, TextKey::KetcherIframeTitle)}",
                    "loading": "lazy",
                    "sandbox": "allow-scripts allow-same-origin allow-popups allow-forms allow-downloads",
                    style: iframe_style(),
                }
            }
        }
    }
}

fn radio_group_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .gap("14px")
        .property("border", "0")
        .property("padding", "0")
        .property("margin", "0")
        .build()
}

fn threshold_section_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .flex_direction("column")
        .gap("5px")
        .padding("10px")
        .property("border-left", "1px solid var(--border)")
        .property("margin-top", "4px")
        .build()
}

fn search_button_style(dirty: bool) -> String {
    let mut style = button_base_style();
    if dirty {
        style = StyleBuilder::new()
            .display("inline-flex")
            .align_items("center")
            .justify_content("center")
            .gap("8px")
            .border("0")
            .border_radius("4px")
            .property("min-height", "40px")
            .padding("11px 16px")
            .font_size("var(--fs-ui)")
            .font_weight("700")
            .cursor("pointer")
            .background_color("color-mix(in srgb, var(--btn-primary-bg) 90%, var(--accent))")
            .color("#fff")
            .box_shadow("var(--shadow-xs)")
            .property(
                "transition",
                "background .15s, box-shadow .15s, transform .12s ease",
            )
            .build();
    }
    style
}

fn panel_stack_style(padding: &str, gap: &str) -> String {
    StyleBuilder::new()
        .display("flex")
        .flex_direction("column")
        .gap(gap)
        .padding(padding)
        .build()
}

fn sr_only_style() -> String {
    StyleBuilder::new()
        .property("position", "absolute")
        .property("width", "1px")
        .property("height", "1px")
        .property("padding", "0")
        .property("margin", "-1px")
        .property("overflow", "hidden")
        .property("clip", "rect(0,0,0,0)")
        .property("white-space", "nowrap")
        .property("border", "0")
        .build()
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
    StyleBuilder::new()
        .font_size("var(--fs-0)")
        .font_weight("700")
        .color("var(--critical-text)")
        .property("text-transform", "uppercase")
        .property("letter-spacing", "0.08em")
        .build()
}

fn label_small_style() -> String {
    StyleBuilder::new()
        .font_size("var(--fs-0)")
        .font_weight("700")
        .color("var(--text)")
        .property("text-transform", "none")
        .property("letter-spacing", "0")
        .build()
}

fn hint_text_style() -> String {
    StyleBuilder::new()
        .font_size("var(--fs-0)")
        .color("var(--text2)")
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

fn ketcher_panel_style() -> String {
    StyleBuilder::new()
        .property("margin", "0")
        .border("1px solid var(--panel-border)")
        .border_radius("var(--radius)")
        .background_color("var(--panel-bg-soft)")
        .box_shadow("var(--panel-shadow)")
        .property(
            "transition",
            "background .15s ease, border-color .15s ease, box-shadow .15s ease",
        )
        .build()
}

fn ketcher_wrap_style() -> String {
    panel_stack_style("0 14px 14px", "10px")
}

fn iframe_style() -> String {
    StyleBuilder::new()
        .property("width", "100%")
        .property("height", "min(78vh, 820px)")
        .property("min-height", "600px")
        .border("1px solid var(--border)")
        .border_radius("4px")
        .background_color("#fff")
        .build()
}

fn kind_pill_style(kind: &str) -> String {
    let background = match kind {
        "smiles" => "var(--accent2)",
        "mol2000" => "#c97a2b",
        "mol3000" => "#2b8f57",
        _ => "var(--text3)",
    };
    StyleBuilder::new()
        .display("inline-block")
        .padding("1px 7px")
        .border_radius("999px")
        .font_size("var(--fs-micro)")
        .font_weight("700")
        .property("letter-spacing", "1px")
        .property("text-transform", "uppercase")
        .property("margin-right", "6px")
        .color("#fff")
        .background_color(background)
        .build()
}

fn range_input_style() -> String {
    StyleBuilder::new()
        .property("width", "100%")
        .property("accent-color", "var(--accent)")
        .property("margin-top", "4px")
        .build()
}

fn textarea_base_style() -> String {
    input_base_style()
}

fn input_base_style() -> String {
    StyleBuilder::new()
        .background_color("var(--surface)")
        .border("1px solid var(--border)")
        .border_radius("4px")
        .color("var(--text)")
        .padding("9px 11px")
        .font_size("var(--fs-ui)")
        .property("width", "100%")
        .font_family("var(--sans)")
        .property("transition", "border-color .15s")
        .build()
}

fn button_base_style() -> String {
    StyleBuilder::new()
        .display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .gap("6px")
        .border("1px solid var(--border)")
        .border_radius("4px")
        .property("min-height", "40px")
        .padding("8px 14px")
        .font_size("var(--fs-0)")
        .font_weight("600")
        .cursor("pointer")
        .background_color("var(--surface)")
        .color("var(--text)")
        .box_shadow("var(--shadow-xs)")
        .property(
            "transition",
            "background .15s, border-color .15s, box-shadow .15s, transform .12s ease",
        )
        .build()
}
