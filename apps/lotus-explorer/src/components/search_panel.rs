use crate::i18n::{Locale, TextKey, t, threshold_label};
use crate::models::*;
use crate::queries::{StructureKind, classify_structure};
use dioxus::prelude::*;

#[component]
pub fn SearchPanel(
    criteria: Signal<SearchCriteria>,
    locale: Locale,
    on_search: EventHandler<()>,
    loading: bool,
) -> Element {
    let mut c = criteria;

    rsx! {
        section {
            class: "search-panel",
            aria_label: "{t(locale, TextKey::SearchFilters)}",

            div { class: "search-panel-body",

                // ── Taxon ────────────────────────────────────────────────────
                div { class: "form-section",
                    label { class: "form-label", r#for: "taxon-input", "{t(locale, TextKey::Taxon)}" }
                    input {
                        id: "taxon-input",
                        r#type: "text",
                        class: "form-input",
                        autocomplete: "off",
                        spellcheck: "false",
                        placeholder: "{t(locale, TextKey::TaxonPlaceholder)}",
                        value: "{c.read().taxon}",
                        oninput: move |e| c.write().taxon = e.value(),
                        onkeydown: move |e| {
                            if e.key() == Key::Enter {
                                on_search.call(());
                            }
                        },
                    }
                    p { class: "form-hint", "{t(locale, TextKey::TaxonHint)}" }
                }

                // ── Structure (SMILES or Molfile V2000/V3000) ────────────────
                StructureSection { criteria, locale }

                // ── Mass range ───────────────────────────────────────────────
                fieldset {
                    class: "form-section",
                    style: "border:0;padding:0;margin:0;",
                    legend { class: "form-label", "{t(locale, TextKey::MolecularMass)}" }
                    div { class: "range-inputs",
                        div { class: "range-pair",
                            label { class: "form-label sm", r#for: "mass-min",
                                "{t(locale, TextKey::Min)}"
                            }
                            input {
                                id: "mass-min",
                                r#type: "number",
                                class: "form-input sm",
                                min: "0",
                                max: "2000",
                                step: "1",
                                value: "{c.read().mass_min}",
                                oninput: move |e| {
                                    if let Ok(v) = e.value().parse::<f64>() {
                                        c.write().mass_min = v;
                                    }
                                },
                            }
                        }
                        span { class: "range-sep", "–" }
                        div { class: "range-pair",
                            label { class: "form-label sm", r#for: "mass-max",
                                "{t(locale, TextKey::Max)}"
                            }
                            input {
                                id: "mass-max",
                                r#type: "number",
                                class: "form-input sm",
                                min: "0",
                                max: "2000",
                                step: "1",
                                value: "{c.read().mass_max}",
                                oninput: move |e| {
                                    if let Ok(v) = e.value().parse::<f64>() {
                                        c.write().mass_max = v;
                                    }
                                },
                            }
                        }
                    }
                }

                // ── Year range ───────────────────────────────────────────────
                fieldset {
                    class: "form-section",
                    style: "border:0;padding:0;margin:0;",
                    legend { class: "form-label", "{t(locale, TextKey::PublicationYear)}" }
                    div { class: "range-inputs",
                        div { class: "range-pair",
                            label { class: "form-label sm", r#for: "year-min",
                                "{t(locale, TextKey::YearFrom)}"
                            }
                            input {
                                id: "year-min",
                                r#type: "number",
                                class: "form-input sm",
                                min: "{DEFAULT_YEAR_MIN}",
                                max: "{current_year()}",
                                step: "1",
                                value: "{c.read().year_min}",
                                oninput: move |e| {
                                    if let Ok(v) = e.value().parse::<i32>() {
                                        c.write().year_min = v;
                                    }
                                },
                            }
                        }
                        span { class: "range-sep", "–" }
                        div { class: "range-pair",
                            label { class: "form-label sm", r#for: "year-max",
                                "{t(locale, TextKey::YearTo)}"
                            }
                            input {
                                id: "year-max",
                                r#type: "number",
                                class: "form-input sm",
                                min: "{DEFAULT_YEAR_MIN}",
                                max: "{current_year()}",
                                step: "1",
                                value: "{c.read().year_max}",
                                oninput: move |e| {
                                    if let Ok(v) = e.value().parse::<i32>() {
                                        c.write().year_max = v;
                                    }
                                },
                            }
                        }
                    }
                }

                // ── Formula filter ───────────────────────────────────────────
                div { class: "form-section",
                    label { class: "radio-label",
                        input {
                            r#type: "checkbox",
                            checked: c.read().formula_enabled,
                            onchange: move |e| c.write().formula_enabled = e.checked(),
                        }
                        "{t(locale, TextKey::FormulaFilter)}"
                    }

                    if c.read().formula_enabled {
                        div { class: "form-section nested",
                            label {
                                class: "form-label sm",
                                r#for: "formula-exact",
                                "{t(locale, TextKey::ExactFormula)}"
                            }
                            input {
                                id: "formula-exact",
                                r#type: "text",
                                class: "form-input sm",
                                autocomplete: "off",
                                spellcheck: "false",
                                placeholder: "C15H10O5",
                                value: "{c.read().formula_exact}",
                                oninput: move |e| c.write().formula_exact = e.value(),
                            }
                        }

                        div { class: "range-inputs",
                            NumPair {
                                label: "C",
                                locale,
                                min_value: c.read().c_min,
                                max_value: c.read().c_max,
                                on_min: move |v| c.write().c_min = v,
                                on_max: move |v| c.write().c_max = v,
                            }
                            NumPair {
                                label: "H",
                                locale,
                                min_value: c.read().h_min,
                                max_value: c.read().h_max,
                                on_min: move |v| c.write().h_min = v,
                                on_max: move |v| c.write().h_max = v,
                            }
                            NumPair {
                                label: "N",
                                locale,
                                min_value: c.read().n_min,
                                max_value: c.read().n_max,
                                on_min: move |v| c.write().n_min = v,
                                on_max: move |v| c.write().n_max = v,
                            }
                        }
                        div { class: "range-inputs",
                            NumPair {
                                label: "O",
                                locale,
                                min_value: c.read().o_min,
                                max_value: c.read().o_max,
                                on_min: move |v| c.write().o_min = v,
                                on_max: move |v| c.write().o_max = v,
                            }
                            NumPair {
                                label: "P",
                                locale,
                                min_value: c.read().p_min,
                                max_value: c.read().p_max,
                                on_min: move |v| c.write().p_min = v,
                                on_max: move |v| c.write().p_max = v,
                            }
                            NumPair {
                                label: "S",
                                locale,
                                min_value: c.read().s_min,
                                max_value: c.read().s_max,
                                on_min: move |v| c.write().s_min = v,
                                on_max: move |v| c.write().s_max = v,
                            }
                        }
                        div { class: "range-inputs",
                            ElemStateSelect {
                                label: "F",
                                locale,
                                value: c.read().f_state,
                                on_change: move |v| c.write().f_state = v,
                            }
                            ElemStateSelect {
                                label: "Cl",
                                locale,
                                value: c.read().cl_state,
                                on_change: move |v| c.write().cl_state = v,
                            }
                            ElemStateSelect {
                                label: "Br",
                                locale,
                                value: c.read().br_state,
                                on_change: move |v| c.write().br_state = v,
                            }
                            ElemStateSelect {
                                label: "I",
                                locale,
                                value: c.read().i_state,
                                on_change: move |v| c.write().i_state = v,
                            }
                        }
                    }
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
fn StructureSection(criteria: Signal<SearchCriteria>, locale: Locale) -> Element {
    let mut c = criteria;
    // Memoise the classifier: `classify_structure` uppercases the whole
    // Molfile on every call. Recompute only when the SMILES text changes,
    // not on every unrelated re-render of the search panel.
    let kind = use_memo(move || classify_structure(&c.read().smiles));
    let kind_value = *kind.read();

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
                value: "{c.read().smiles}",
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
                        checked: c.read().smiles_search_type == SmilesSearchType::Substructure,
                        onchange: move |_| c.write().smiles_search_type = SmilesSearchType::Substructure,
                    }
                    "{t(locale, TextKey::Substructure)}"
                }
                label { class: "radio-label",
                    input {
                        r#type: "radio",
                        name: "stype",
                        checked: c.read().smiles_search_type == SmilesSearchType::Similarity,
                        onchange: move |_| c.write().smiles_search_type = SmilesSearchType::Similarity,
                    }
                    "{t(locale, TextKey::Similarity)}"
                }
            }
            if c.read().smiles_search_type == SmilesSearchType::Similarity {
                div { class: "form-section nested",
                    label { class: "form-label sm", r#for: "threshold-input",
                        "{threshold_label(locale, c.read().smiles_threshold)}"
                    }
                    input {
                        id: "threshold-input",
                        r#type: "range",
                        class: "range-input",
                        min: "0.0",
                        max: "1.0",
                        step: "0.01",
                        value: "{c.read().smiles_threshold}",
                        aria_valuemin: "0",
                        aria_valuemax: "1",
                        aria_valuenow: "{c.read().smiles_threshold}",
                        oninput: move |e| {
                            if let Ok(v) = e.value().parse::<f64>() {
                                c.write().smiles_threshold = v;
                            }
                        },
                    }
                }
            }

            // ── Ketcher editor ────────────────────────────────────────
            // Note: the Ketcher editor lives in the main content area so it
            // can use the full viewport width. See `KetcherPanel` below.
            p { class: "form-hint ketcher-hint",
                "{t(locale, TextKey::KetcherHintA)}"
                strong { "{t(locale, TextKey::KetcherSummary)}" }
                "{t(locale, TextKey::KetcherHintB)}"
                em { "{t(locale, TextKey::EditCopyDaylightSmiles)}" }
                "{t(locale, TextKey::KetcherHintC)}"
                em { "{t(locale, TextKey::CopyExtendedSmilesMol)}" }
                "{t(locale, TextKey::KetcherHintD)}"
            }
        }
    }
}

// ── Ketcher editor panel (full-width, rendered in the main content area) ─────

/// Relative URL at which the Ketcher standalone build is served.
/// Place the contents of Ketcher's `standalone/` folder at this path
/// (e.g. `assets/ketcher/` or `public/ketcher/` in the Dioxus project) —
/// this matches the Python notebook's `public/standalone/index.html`
/// convention.
const KETCHER_URL: &str = "ketcher/index.html";

#[component]
pub fn KetcherPanel(locale: Locale) -> Element {
    rsx! {
        details { class: "ketcher-panel",
            summary { "{t(locale, TextKey::KetcherSummary)}" }
            div { class: "ketcher-wrap",
                iframe {
                    src: "{KETCHER_URL}",
                    class: "ketcher-iframe",
                    title: "{t(locale, TextKey::KetcherIframeTitle)}",
                    "loading": "lazy",
                    "sandbox": "allow-scripts allow-same-origin allow-popups allow-forms allow-downloads",
                }
                p { class: "form-hint ketcher-hint",
                    "{t(locale, TextKey::KetcherHintA)}"
                    em { "{t(locale, TextKey::EditCopyDaylightSmiles)}" }
                    "{t(locale, TextKey::KetcherHintC)}"
                    em { "{t(locale, TextKey::CopyExtendedSmilesMol)}" }
                    "{t(locale, TextKey::KetcherHintD)}"
                }
                // The "Editor not loading? Download Ketcher standalone…"
                // install hint is intentionally hidden from end-users: the
                // standalone bundle is shipped with the app under
                // `public/ketcher/`.
                //
                // p { class: "form-hint ketcher-install",
                //     "Editor not loading? Download "
                //     a { href: "https://github.com/epam/ketcher/releases",
                //         target: "_blank", rel: "noopener noreferrer",
                //         "Ketcher standalone" }
                //     " and extract its "
                //     code { class: "mono", "standalone/" }
                //     " folder to "
                //     code { class: "mono", "public/ketcher/" }
                //     " in the app."
                // }
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

#[component]
fn NumPair(
    label: &'static str,
    locale: Locale,
    min_value: i32,
    max_value: i32,
    on_min: EventHandler<i32>,
    on_max: EventHandler<i32>,
) -> Element {
    rsx! {
        div { class: "range-pair",
            label { class: "form-label sm", "{label} {t(locale, TextKey::MinCount)}" }
            input {
                r#type: "number",
                class: "form-input sm",
                min: "0",
                max: "500",
                aria_label: "{label} {t(locale, TextKey::MinCountAria)}",
                value: "{min_value}",
                oninput: move |e| {
                    if let Ok(v) = e.value().parse::<i32>() {
                        on_min.call(v);
                    }
                },
            }
            label { class: "form-label sm", "{label} {t(locale, TextKey::MaxCount)}" }
            input {
                r#type: "number",
                class: "form-input sm",
                min: "0",
                max: "500",
                aria_label: "{label} {t(locale, TextKey::MaxCountAria)}",
                value: "{max_value}",
                oninput: move |e| {
                    if let Ok(v) = e.value().parse::<i32>() {
                        on_max.call(v);
                    }
                },
            }
        }
    }
}

#[component]
fn ElemStateSelect(
    label: &'static str,
    locale: Locale,
    value: ElementState,
    on_change: EventHandler<ElementState>,
) -> Element {
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
