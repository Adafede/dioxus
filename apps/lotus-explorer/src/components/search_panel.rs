// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Search panel and its sub-section components.
//!
//! ## Architecture
//!
//! `SearchPanel` reads only what it needs:
//! * `FormCriteriaContext` — live criteria signal + `is_dirty()` for the search
//!   button affordance.
//! * `SearchUiContext.explore` — lifecycle signal for the `loading` flag.
//!
//! Sub-sections (`FormulaSection`, `StructureSection`) consume `FormCriteriaContext`
//! directly so they need **zero data props** — they bypass SearchPanel entirely.
//! This eliminates 36 props that previously flowed through `SearchPanel` down to
//! `FormulaSection`.

pub use crate::components::form_sections::{
    FormulaSection, MassRangeInput, TaxonInput, YearRangeInput,
};

#[path = "search_panel/structure_model.rs"]
mod structure_model;

use crate::i18n::{TextKey, t, threshold_label};
use crate::models::*;
use crate::queries::classify_structure;
use crate::state::{use_form_criteria_context, use_search_ui_context};
use dioxus::prelude::*;

#[component]
pub fn SearchPanel(on_search: EventHandler<()>) -> Element {
    let state = use_search_ui_context();
    let form_ctx = use_form_criteria_context();
    let locale = crate::hooks::use_locale();

    // Read loading from the live explore signal — no stale copy in AppState.
    let loading = state.explore.read().lifecycle.loading;
    // Show a "dirty" indicator on the search button when the form has changed
    // since the last search.
    let is_dirty = form_ctx.is_dirty();

    let mut c = state.criteria;
    let criteria = c.read().clone();

    rsx! {
        section {
            class: "search-panel",
            aria_label: "{t(locale, TextKey::SearchFilters)}",

            div { class: "search-panel-body",

                // ── Taxon ────────────────────────────────────────────────────
                TaxonInput {
                    value: criteria.taxon.clone(),
                    on_input: move |v| c.write().taxon = v,
                    on_search,
                }

                // ── Structure (SMILES or Molfile V2000/V3000) ────────────────
                StructureSection {}

                // ── Mass range ───────────────────────────────────────────────
                MassRangeInput {
                    min_value: criteria.mass_min,
                    max_value: criteria.mass_max,
                    on_min: move |v| c.write().mass_min = v,
                    on_max: move |v| c.write().mass_max = v,
                }

                // ── Year range ───────────────────────────────────────────────
                YearRangeInput {
                    min_value: criteria.year_min,
                    max_value: criteria.year_max,
                    on_min: move |v| c.write().year_min = v,
                    on_max: move |v| c.write().year_max = v,
                }

                // ── Formula filter (zero props — reads context internally) ───
                FormulaSection {}
            }

            // ── Search button ───────────────────────────────────────────────
            // Shows a spinner while searching and a dot when the form has
            // changed since the last search (dirty state).
            button {
                class: if is_dirty && !loading { "search-btn search-btn--dirty" } else { "search-btn" },
                r#type: "submit",
                disabled: loading,
                aria_label: "{t(locale, TextKey::RunSearch)}",
                onclick: move |_| on_search.call(()),
                if loading {
                    span { class: "spinner-sm", "aria-hidden": "true" }
                    "{t(locale, TextKey::Searching)}"
                } else {
                    "{t(locale, TextKey::Search)}"
                }
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
    let mut c = ctx.criteria;
    // Memoised classifier: `classify_structure` uppercases the entire Molfile
    // on every call.  Recompute only when the SMILES text changes.
    let kind = use_memo(move || classify_structure(&c.read().smiles));
    let kind_value = *kind.read();
    let criteria = c.read().clone();
    let view_model =
        structure_model::build_structure_section_model(kind_value, criteria.smiles_search_type);

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
                value: "{criteria.smiles}",
                oninput: move |e| c.write().smiles = e.value(),
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
                        checked: criteria.smiles_search_type == SmilesSearchType::Substructure,
                        onchange: move |_| c.write().smiles_search_type = SmilesSearchType::Substructure,
                    }
                    "{t(locale, TextKey::Substructure)}"
                }
                label { class: "radio-label",
                    input {
                        r#type: "radio",
                        name: "stype",
                        checked: criteria.smiles_search_type == SmilesSearchType::Similarity,
                        onchange: move |_| c.write().smiles_search_type = SmilesSearchType::Similarity,
                    }
                    "{t(locale, TextKey::Similarity)}"
                }
            }
            if view_model.show_similarity_threshold {
                div { class: "form-section nested",
                    label { class: "form-label sm", r#for: "threshold-input",
                        "{threshold_label(locale, criteria.smiles_threshold)}"
                    }
                    input {
                        id: "threshold-input",
                        r#type: "range",
                        class: "range-input",
                        min: "0.0",
                        max: "1.0",
                        step: "0.01",
                        value: "{criteria.smiles_threshold}",
                        aria_valuemin: "0",
                        aria_valuemax: "1",
                        aria_valuenow: "{criteria.smiles_threshold}",
                        oninput: move |e| {
                            if let Ok(v) = e.value().parse::<f64>() {
                                c.write().smiles_threshold = v;
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
const KETCHER_URL: &str = "ketcher/index.html";

#[component]
pub fn KetcherPanel() -> Element {
    let locale = crate::hooks::use_locale();
    rsx! {
        section {
            class: "ketcher-panel",
            aria_label: "{t(locale, TextKey::KetcherSummary)}",
            div { class: "ketcher-wrap",
                h2 { class: "curation-title", "{t(locale, TextKey::KetcherSummary)}" }
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
