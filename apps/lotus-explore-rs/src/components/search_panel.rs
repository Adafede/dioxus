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
use crate::ui::style_constants;
use dioxus::prelude::*;
use ui::prelude::*;
pub fn SearchPanel() -> Element {
    let state = use_results_context();
    let form_ctx = use_form_criteria_context();
    let interactions = use_explore_interactions();
    let locale = crate::hooks::use_locale();

    // Loading flag subscribed via selector so result churn does not rerender the sidebar.
    let loading = *use_lifecycle_selector(state.explore, |lifecycle| lifecycle.loading).read();
    // Dirty flag: show affordance when form changed since last search.
    let is_dirty = form_ctx.is_dirty();
    let form_search = interactions.clone();
    let button_search = interactions.clone();

    let search_schema = r#"{"type":"object","properties":{"taxon":{"type":"string","description":"Taxon name, Wikidata QID, or * for all taxa"},"smiles":{"type":"string","description":"SMILES or Molfile input"},"mass_min":{"type":"number","description":"Minimum molecular mass in Da"},"mass_max":{"type":"number","description":"Maximum molecular mass in Da"},"year_min":{"type":"integer","description":"Minimum publication year"},"year_max":{"type":"integer","description":"Maximum publication year"},"formula":{"type":"string","description":"Exact formula filter"}},"additionalProperties":true}"#;

    rsx! {
        form {
            id: "lotus-search-form",
            class: "search-panel",
            style: crate::ui::style_constants::panels::search_panel_style(),
            aria_label: "{t(locale, TextKey::SearchFilters)}",
            aria_labelledby: SEARCH_PANEL_HEADING_ID,
            "data-webmcp-id": "lotus-search-form",
            "data-webmcp-type": "form",
            "data-webmcp-name": "LOTUS search form",
            "data-webmcp-description": "Search compounds by taxon, SMILES or Molfile, mass range, publication year, and formula constraints.",
            "data-webmcp-schema": "{search_schema}",
            "data-mcp-id": "lotus-search-form",
            "data-mcp-type": "form",
            "data-mcp-name": "LOTUS search form",
            "data-mcp-description": "Search compounds by taxon, SMILES or Molfile, mass range, publication year, and formula constraints.",
            "data-mcp-schema": "{search_schema}",
            onsubmit: move |evt: Event<FormData>| {
                evt.prevent_default();
                form_search.search();
            },
            h2 { id: SEARCH_PANEL_HEADING_ID, class: "sr-only", "{t(locale, TextKey::SearchFilters)}" }

            div { id: SEARCH_PANEL_BODY_ID, class: "search-panel-body",
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
                    style: style_constants::search_buttons::search_button_state(is_dirty),
                    span { class: "spinner-sm", "aria-hidden": "true" }
                    "{t(locale, TextKey::Searching)}"
                }
            } else {
                SearchButton { on_click: move |_| button_search.search() }
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
        div { style: structure_section_style(),
            label { style: crate::ui::style_constants::forms::label_base_style(), r#for: "smiles-input",
                "{t(locale, TextKey::StructureSmilesOrMol)}"
            }
            textarea {
                id: "smiles-input",
                name: "smiles",
                spellcheck: "false",
                placeholder: "{t(locale, TextKey::StructurePlaceholder)}",
                value: "{smiles}",
                oninput: move |e| ctx.update(FormAction::Smiles(e.value())),
                rows: "2",
                style: crate::ui::style_constants::search_controls::textarea_base_style(),
            }
            if let Some(note_key) = view_model.note_key {
                p { style: crate::ui::style_constants::forms::hint_text_style(),
                    span { style: style_constants::search_controls::kind_pill_style(view_model.kind_class), "{kind_value.label()}" }
                    span { "{t(locale, note_key)}" }
                }
            }

            fieldset { style: crate::ui::style_constants::search_controls::radio_group_style(),
                legend { style: crate::ui::style_constants::utilities::sr_only_style(), "{t(locale, TextKey::StructureSearchMode)}" }
                label { style: crate::ui::style_constants::search_controls::radio_label_style(),
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
                label { style: crate::ui::style_constants::search_controls::radio_label_style(),
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
                div { style: crate::ui::style_constants::search_controls::threshold_section_style(),
                    label { style: crate::ui::style_constants::forms::label_small_style(), r#for: "threshold-input",
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
                        style: crate::ui::style_constants::search_controls::range_input_style(),
                    }
                }
            }
        }
    }
}

fn structure_section_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .flex_direction("column")
        .gap("5px")
        .padding("10px 12px")
        .border("1px solid var(--panel-border)")
        .border_radius("12px")
        .background_color("transparent")
        .build()
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
            style: crate::ui::style_constants::panel_containers::ketcher_panel_style(),
            div { style: crate::ui::style_constants::panel_containers::ketcher_wrap_style(),
                p { style: crate::ui::style_constants::forms::hint_text_style(),
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
                    style: crate::ui::style_constants::panel_containers::iframe_style(),
                }
            }
        }
    }
}
