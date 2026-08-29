// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Search panel and its subsection components.

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
use crate::ui::classes;
use dioxus::prelude::*;

pub fn SearchPanel() -> Element {
    let state = use_results_context();
    let form_ctx = use_form_criteria_context();
    let interactions = use_explore_interactions();
    let locale = crate::hooks::use_locale();

    let loading = *use_lifecycle_selector(state.explore, |lifecycle| lifecycle.loading).read();
    let is_dirty = form_ctx.is_dirty();
    let form_search = interactions.clone();
    let button_search = interactions.clone();

    let search_schema = r#"{"type":"object","properties":{"taxon":{"type":"string","description":"Taxon name, Wikidata QID, or * for all taxa"},"smiles":{"type":"string","description":"SMILES or Molfile input"},"mass_min":{"type":"number","description":"Minimum molecular mass in Da"},"mass_max":{"type":"number","description":"Maximum molecular mass in Da"},"year_min":{"type":"integer","description":"Minimum publication year"},"year_max":{"type":"integer","description":"Maximum publication year"},"formula":{"type":"string","description":"Exact formula filter"}},"additionalProperties":true}"#;

    rsx! {
        form {
            id: "lotus-search-form",
            class: "search-panel",
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
                TaxonInput {}
                StructureSection {}
                MassRangeInput {}
                YearRangeInput {}
                FormulaSection {}
            }

            SearchButton {
                loading,
                is_dirty,
                on_click: move |_| button_search.search(),
            }
        }
    }
}

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
    let kind = use_memo(move || classify_structure(&smiles_for_kind));
    let kind_value = *kind.read();
    let view_model = structure_model::build_structure_section_model(kind_value, smiles_search_type);

    rsx! {
        div { class: "{classes::SECTION}",
            label {
                class: "{classes::LABEL}",
                r#for: "smiles-input",
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
                class: "min-h-16 resize-y font-mono {classes::INPUT}",
            }
            if let Some(note_key) = view_model.note_key {
                p { class: "flex flex-wrap items-center gap-2 {classes::HINT}",
                    span {
                        class: "rounded-lotus-sm bg-accent/10 px-1.5 py-0.5 font-semibold text-accent",
                        "{kind_value.label()}"
                    }
                    span { "{t(locale, note_key)}" }
                }
            }

            fieldset { class: "m-0 flex flex-wrap gap-3 border-0 p-0",
                legend { class: "sr-only", "{t(locale, TextKey::StructureSearchMode)}" }
                label { class: "inline-flex items-center gap-1.5 text-ui text-muted",
                    input {
                        r#type: "radio",
                        name: "stype",
                        class: "accent-accent h-4 w-4 focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2",
                        checked: smiles_search_type == SmilesSearchType::Substructure,
                        onchange: move |_| {
                            ctx.update(FormAction::SmilesSearchType(SmilesSearchType::Substructure))
                        },
                    }
                    "{t(locale, TextKey::Substructure)}"
                }
                label { class: "inline-flex items-center gap-1.5 text-ui text-muted",
                    input {
                        r#type: "radio",
                        name: "stype",
                        class: "accent-accent h-4 w-4 focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2",
                        checked: smiles_search_type == SmilesSearchType::Similarity,
                        onchange: move |_| {
                            ctx.update(FormAction::SmilesSearchType(SmilesSearchType::Similarity))
                        },
                    }
                    "{t(locale, TextKey::Similarity)}"
                }
            }
            if view_model.show_similarity_threshold {
                div { class: "flex flex-col gap-1",
                    label {
                        class: "{classes::MICRO_LABEL}",
                        r#for: "threshold-input",
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
                        class: "w-full accent-accent cursor-pointer appearance-none h-2 bg-border rounded-full focus-visible:outline-none {classes::FOCUS_RING} focus-visible:ring-offset-2",
                        oninput: move |e| {
                            if let Ok(v) = e.value().parse::<f64>() {
                                ctx.update(FormAction::SmilesThreshold(v));
                            }
                        },
                    }
                }
            }
        }
    }
}
