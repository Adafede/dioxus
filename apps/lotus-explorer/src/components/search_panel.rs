// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

pub use crate::components::form_sections::{
    FormulaSection, MassRangeInput, TaxonInput, YearRangeInput,
};

use crate::i18n::{Locale, TextKey, t, threshold_label};
use crate::models::*;
use crate::queries::{StructureKind, classify_structure};
use crate::state::use_search_ui_context;
use dioxus::prelude::*;
#[cfg(target_arch = "wasm32")]
use web_time::SystemTime;

pub const DEFAULT_YEAR_MIN: u16 = 1975;

pub fn current_year() -> u16 {
    #[cfg(target_arch = "wasm32")]
    {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(duration) => {
                let secs_per_year = 365.25 * 24.0 * 60.0 * 60.0;
                let years_since_epoch = duration.as_secs_f64() / secs_per_year;
                (1970.0 + years_since_epoch) as u16
            }
            Err(_) => 2025,
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        2025
    }
}

#[component]
pub fn SearchPanel(on_search: EventHandler<()>) -> Element {
    let state = use_search_ui_context();
    let locale = crate::hooks::use_locale();
    let loading = state.explore.read().lifecycle.loading;
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

                // ── Formula filter ───────────────────────────────────────────
                FormulaSection {
                    enabled: criteria.formula_enabled,
                    formula_exact: criteria.formula_exact.clone(),
                    c_min: criteria.c_min,
                    c_max: criteria.c_max,
                    h_min: criteria.h_min,
                    h_max: criteria.h_max,
                    n_min: criteria.n_min,
                    n_max: criteria.n_max,
                    o_min: criteria.o_min,
                    o_max: criteria.o_max,
                    p_min: criteria.p_min,
                    p_max: criteria.p_max,
                    s_min: criteria.s_min,
                    s_max: criteria.s_max,
                    f_state: criteria.f_state,
                    cl_state: criteria.cl_state,
                    br_state: criteria.br_state,
                    i_state: criteria.i_state,
                    on_enabled: move |v| c.write().formula_enabled = v,
                    on_formula_exact: move |v| c.write().formula_exact = v,
                    on_c_min: move |v| c.write().c_min = v,
                    on_c_max: move |v| c.write().c_max = v,
                    on_h_min: move |v| c.write().h_min = v,
                    on_h_max: move |v| c.write().h_max = v,
                    on_n_min: move |v| c.write().n_min = v,
                    on_n_max: move |v| c.write().n_max = v,
                    on_o_min: move |v| c.write().o_min = v,
                    on_o_max: move |v| c.write().o_max = v,
                    on_p_min: move |v| c.write().p_min = v,
                    on_p_max: move |v| c.write().p_max = v,
                    on_s_min: move |v| c.write().s_min = v,
                    on_s_max: move |v| c.write().s_max = v,
                    on_f_state: move |v| c.write().f_state = v,
                    on_cl_state: move |v| c.write().cl_state = v,
                    on_br_state: move |v| c.write().br_state = v,
                    on_i_state: move |v| c.write().i_state = v,
                }
            }

            // ── Search button ────────────────────────────────────────────
            button {
                class: "search-btn",
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

#[component]
fn StructureSection() -> Element {
    let locale = crate::hooks::use_locale();
    let state = use_search_ui_context();
    let mut c = state.criteria;
    // Memoise the classifier: `classify_structure` uppercases the whole
    // Molfile on every call. Recompute only when the SMILES text changes,
    // not on every unrelated re-render of the search panel.
    let kind = use_memo(move || classify_structure(&c.read().smiles));
    let kind_value = *kind.read();
    let criteria = c.read().clone();

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
            if kind_value != StructureKind::Empty {
                p { class: "form-hint",
                    span {
                        class: "kind-pill",
                        "data-kind": "{kind_class(kind_value)}",
                        "{kind_value.label()}"
                    }
                    span { class: "kind-note", {kind_note(kind_value, locale)} }
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
            if criteria.smiles_search_type == SmilesSearchType::Similarity {
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
/// Place the contents of Ketcher's `standalone/` folder at this path
/// (e.g. `assets/ketcher/` or `public/ketcher/` in the Dioxus project).
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

fn kind_class(k: StructureKind) -> &'static str {
    match k {
        StructureKind::Empty => "empty",
        StructureKind::Smiles => "smiles",
        StructureKind::MolfileV2000 => "mol2000",
        StructureKind::MolfileV3000 => "mol3000",
    }
}

fn kind_note(k: StructureKind, locale: Locale) -> &'static str {
    match k {
        StructureKind::Empty => "",
        StructureKind::Smiles => t(locale, TextKey::KindNoteSmiles),
        StructureKind::MolfileV2000 => t(locale, TextKey::KindNoteMol2000),
        StructureKind::MolfileV3000 => t(locale, TextKey::KindNoteMol3000),
    }
}
