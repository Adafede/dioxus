// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Search panel and its sub-section components.
//!
//! ## Architecture
//!
//! `SearchPanel` is now extremely thin — it owns only two concerns:
//! * Presenting the search button (loading state + dirty indicator).
//! * Delegating form sections to context-aware sub-components.
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
            class: "search-panel",
            aria_label: "{t(locale, TextKey::SearchFilters)}",
            aria_labelledby: SEARCH_PANEL_HEADING_ID,

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
                    class: if is_dirty { "search-btn search-btn--dirty" } else { "search-btn" },
                    r#type: "submit",
                    disabled: true,
                    aria_label: "{t(locale, TextKey::RunSearch)}",
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
        div { class: "form-section",
            label { class: "form-label", r#for: "smiles-input",
                "{t(locale, TextKey::StructureSmilesOrMol)}"
            }
            textarea {
                id: "smiles-input",
                class: "form-textarea mono",
                spellcheck: "false",
                placeholder: "{t(locale, TextKey::StructurePlaceholder)}",
                value: "{smiles}",
                oninput: move |e| ctx.update(FormAction::Smiles(e.value())),
                rows: "4",
            }
            if let Some(note_key) = view_model.note_key {
                p { class: "form-hint",
                    span {
                        class: "kind-pill",
                        "data-kind": "{view_model.kind_class}",
                        "{kind_value.label()}"
                    }
                    span { class: "kind-note", "{t(locale, note_key)}" }
                }
            } else {
                p { class: "form-hint", "{t(locale, TextKey::StructureHintEmpty)}" }
            }

            fieldset { class: "radio-group", style: "border:0;padding:0;margin:0;",
                legend { class: "sr-only", "{t(locale, TextKey::StructureSearchMode)}" }
                label { class: "radio-label",
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
                label { class: "radio-label",
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
                div { class: "form-section nested",
                    label { class: "form-label sm", r#for: "threshold-input",
                        "{threshold_label(locale, smiles_threshold)}"
                    }
                    input {
                        id: "threshold-input",
                        r#type: "range",
                        class: "range-input",
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
            class: "ketcher-panel",
            aria_label: "{t(locale, TextKey::KetcherSummary)}",
            div { class: "ketcher-wrap",
                p { class: "ketcher-hint",
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
                    class: "ketcher-iframe",
                    title: "{t(locale, TextKey::KetcherIframeTitle)}",
                    "loading": "lazy",
                    "sandbox": "allow-scripts allow-same-origin allow-popups allow-forms allow-downloads",
                }
            }
        }
    }
}
