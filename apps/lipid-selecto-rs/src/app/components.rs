// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Reusable rendering components for the lipid-selecto-rs UI.
//!
//! Extracted from `app.rs` to keep the entry-point module focused on the
//! top-level layout/orchestration logic. All functions here are
//! `pub(super)` — visible only to the parent `app` module.

use dioxus::prelude::*;
use ui::prelude::*;

use crate::chemical_class::ChemicalClass;

use super::analysis::{collect_adduct_options, group_classes_by_family};
#[cfg(target_arch = "wasm32")]
use super::download::{download_filtered_mgf, download_filtered_smiles};
use super::family::family_rank;
use super::gallery::prepare_block_class_tags;
#[cfg(target_arch = "wasm32")]
use super::gallery::prepare_gallery_smiles;
use super::types::{GallerySmilesEntry, SummaryFilters};

pub(super) fn section_subheading() -> String {
    StyleBuilder::new()
        .margin("0 0 0.5rem")
        .font_size("1.05rem")
        .color("#0f172a")
        .build()
}

pub(super) fn checkbox_sm() -> String {
    StyleBuilder::new()
        .width("14px")
        .height("14px")
        .cursor("pointer")
        .build()
}

/// Renders the "Available Lipid Classes" card, showing LMSD-based classes
/// grouped by family with collapsible sections for each family.
pub(super) fn lipid_classes_card() -> Element {
    // Pre-compute family display data: (family_name, shown_classes, others_count, others_color)
    const MAX_CLASSES_PER_FAMILY: usize = 4;

    // Load LMSD-based class scheme (same as Results panel)
    let all_classes = crate::chemical_class::lmsd_all();

    // Group classes by family, then sort families by LIPID MAPS rank order.
    let mut families: Vec<(String, Vec<ChemicalClass>)> = Vec::new();
    for class in &all_classes {
        if let Some(entry) = families.iter_mut().find(|(f, _)| f == &class.family) {
            entry.1.push(class.clone());
        } else {
            families.push((class.family.clone(), vec![class.clone()]));
        }
    }
    families.sort_by_key(|(f, _)| family_rank(f));

    let family_display: Vec<(String, Vec<ChemicalClass>, usize, String)> = families
        .iter()
        .map(|(family, classes)| {
            let shown: Vec<ChemicalClass> = classes
                .iter()
                .take(MAX_CLASSES_PER_FAMILY)
                .cloned()
                .collect();
            let others = classes.len() - shown.len();
            // Use shade 4 color for "N others" - get it from the first "others" class
            let others_color = classes.get(MAX_CLASSES_PER_FAMILY).map_or_else(
                || {
                    classes
                        .last()
                        .map_or_else(|| "#cbd5e1".to_string(), |c| c.color.clone())
                },
                |c| c.color.clone(),
            );
            (family.clone(), shown, others, others_color)
        })
        .collect();

    // Use a single open-collapsed state for simplicity (all expanded by default)
    let family_expanded = use_signal(Vec::<usize>::new);

    rsx! {
                div {
                    style: StyleBuilder::new().property("background", "rgba(255,255,255,0.9)").border("1px solid rgba(148,163,184,0.22)").border_radius("20px").box_shadow("0 12px 40px rgba(15, 23, 42, 0.08)").padding("1.25rem").property("margin-bottom", "1.25rem").build(),
                    h2 { style: StyleBuilder::new().margin("0 0 0.5rem").font_size("1.1rem").build(), "Available Lipid Classes" }
                    p { style: StyleBuilder::new().margin("0 0 0.75rem").color("#64748b").font_size("0.8rem").build(), "LMSD classification scheme — {all_classes.len()} classes across {families.len()} families" }

                    // Family sections - stacked vertically with headers
                    div { style: StyleBuilder::new().display("flex").flex_direction("column").gap("0.3rem").property("max-height", "320px").property("overflow-y", "auto").build(),
                        for (idx, (family, shown_classes, others_count, others_color)) in family_display.iter().enumerate() {
                            // Family header with color bar
                            {
                                let family_clone = family.clone();
                                let family_color = shown_classes.first().map_or_else(|| others_color.clone(), |c| c.color.clone());
                                let idx_copy = idx;
                                let mut family_expanded_signal = family_expanded;
                                let is_expanded = family_expanded.read().contains(&idx);
                                rsx! {
                                    div { style: StyleBuilder::new().build(),
                                        div {
                                            style: StyleBuilder::new()
                                                .display("flex")
                                                .align_items("center")
                                                .justify_content("space-between")
                                                .padding("0.35rem 0.5rem")
                                                .cursor("pointer")
                                                .border_bottom("1px solid #f1f5f9")
                                                .build(),
                                            onclick: move |_| {
                                                let mut expanded = family_expanded_signal.read().clone();
                                                if expanded.contains(&idx_copy) {
                                                    expanded.retain(|&i| i != idx_copy);
                                                } else {
                                                    expanded.push(idx_copy);
                                                }
                                                family_expanded_signal.set(expanded);
                                            },
                                            div { style: StyleBuilder::new().display("flex").align_items("center").gap("0.3rem").build(),
                                                span { style: StyleBuilder::new().width("10px").height("10px").border_radius("50%").property("background", &family_color).build(), }
                                                strong { style: StyleBuilder::new().font_size("0.82rem").font_weight("700").color("#0f172a").build(), "{family_clone}" }
                                            }
                                            span { style: StyleBuilder::new().font_size("0.72rem").color("#94a3b8").build(), "{shown_classes.len() + others_count} classes" }
                                        }
                                        if is_expanded {
                                            div { style: StyleBuilder::new().display("flex").flex_wrap("wrap").gap("0.3rem").property("padding", "0.4rem 0 0.2rem 0.8rem").build(),
                                                for class in shown_classes.iter() {
                                                    div {
                                                        style: StyleBuilder::new().padding("0.3rem 0.5rem").property("background", "#f1f5f9").border("1px solid #e2e8f0").border_radius("6px").font_size("0.75rem").build(),
                                                        strong { style: StyleBuilder::new().property("word-break", "break-word").display("inline-flex").align_items("center").gap("0.25rem").build(),
                                                            span { style: StyleBuilder::new().width("6px").height("6px").border_radius("50%").property("background", &class.color).build(), }
                                                            "{class.name}"
                                                        }
                                                    }
                                                }
                                                if *others_count > 0 {
                                                    div {
                                                        style: StyleBuilder::new().padding("0.3rem 0.5rem").property("background", "#f1f5f9").border("1px solid #e2e8f0").border_radius("6px").font_size("0.75rem").build(),
                                                        strong { style: StyleBuilder::new().property("word-break", "break-word").display("inline-flex").align_items("center").gap("0.25rem").build(),
                                                            span { style: StyleBuilder::new().width("6px").height("6px").border_radius("50%").property("background", others_color).build(), }
                                                            "{others_count} more"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
    }
}

/// Renders the filter controls row: m/z range, precursor range, and adduct dropdown.
#[allow(unused_variables, unused_mut)]
fn filter_controls_row(
    mut mz_min: Signal<f64>,
    mut mz_max: Signal<f64>,
    mut precursor_min: Signal<f64>,
    mut precursor_max: Signal<f64>,
    mut adduct_filter: Signal<String>,
    adduct_options: &[String],
) -> Element {
    // Pre-build adduct option elements outside rsx! to avoid scoping issues
    let adduct_option_elements: Vec<Element> = adduct_options
        .iter()
        .map(|opt| {
            let opt_opt = opt.clone();
            rsx! { option { value: "{opt_opt}", "{opt_opt}" } }
        })
        .collect();

    rsx! {
        div { style: StyleBuilder::new().display("flex").flex_wrap("wrap").gap("0.75rem").align_items("flex-end").property("margin", "0.75rem 0").build(),
            // m/z range filter
            div { style: StyleBuilder::new().display("flex").flex_direction("column").gap("0.25rem").build(),
                span { style: StyleBuilder::new().font_size("0.7rem").color("#475569").font_weight("600").build(), "m/z range" }
                div { style: StyleBuilder::new().display("flex").gap("0.3rem").align_items("center").build(),
                    input {
                        r#type: "number",
                        placeholder: "min",
                        value: "{mz_min.read()}",
                        oninput: move |evt| {
                            if let Ok(val) = evt.value().parse::<f64>() {
                                mz_min.set(val);
                            }
                        },
                        style: StyleBuilder::new().width("75px").padding("0.3rem 0.5rem").border("1px solid #cbd5e1").border_radius("6px").font_size("0.8rem").build(),
                    }
                    span { style: StyleBuilder::new().color("#94a3b8").font_size("0.9rem").build(), "–" }
                    input {
                        r#type: "number",
                        placeholder: "max",
                        value: "{mz_max.read()}",
                        oninput: move |evt| {
                            if let Ok(val) = evt.value().parse::<f64>() {
                                mz_max.set(val);
                            }
                        },
                        style: StyleBuilder::new().width("75px").padding("0.3rem 0.5rem").border("1px solid #cbd5e1").border_radius("6px").font_size("0.8rem").build(),
                    }
                }
            }
            // precursor range filter
            div { style: StyleBuilder::new().display("flex").flex_direction("column").gap("0.25rem").build(),
                span { style: StyleBuilder::new().font_size("0.7rem").color("#475569").font_weight("600").build(), "precursor range" }
                div { style: StyleBuilder::new().display("flex").gap("0.3rem").align_items("center").build(),
                    input {
                        r#type: "number",
                        placeholder: "min",
                        value: "{precursor_min.read()}",
                        oninput: move |evt| {
                            if let Ok(val) = evt.value().parse::<f64>() {
                                precursor_min.set(val);
                            }
                        },
                        style: StyleBuilder::new().width("80px").padding("0.3rem 0.5rem").border("1px solid #cbd5e1").border_radius("6px").font_size("0.8rem").build(),
                    }
                    span { style: StyleBuilder::new().color("#94a3b8").font_size("0.9rem").build(), "–" }
                    input {
                        r#type: "number",
                        placeholder: "max",
                        value: "{precursor_max.read()}",
                        oninput: move |evt| {
                            if let Ok(val) = evt.value().parse::<f64>() {
                                precursor_max.set(val);
                            }
                        },
                        style: StyleBuilder::new().width("80px").padding("0.3rem 0.5rem").border("1px solid #cbd5e1").border_radius("6px").font_size("0.8rem").build(),
                    }
                }
            }
            // adduct dropdown
            div { style: StyleBuilder::new().display("flex").flex_direction("column").gap("0.25rem").build(),
                span { style: StyleBuilder::new().font_size("0.7rem").color("#475569").font_weight("600").build(), "adduct" }
                select {
                    value: "{adduct_filter.read()}",
                    onchange: move |evt| {
                        adduct_filter.set(evt.value());
                    },
                    style: StyleBuilder::new().width("110px").padding("0.3rem 0.5rem").border("1px solid #cbd5e1").border_radius("6px").font_size("0.8rem").build(),
                    option { value: "", "All" }
                    for elem in adduct_option_elements.iter() {
                        { elem }
                    }
                }
            }
        }
    }
}

/// Renders the family selection panel with a "Select All" checkbox.
#[allow(unused_variables, unused_mut)]
fn family_filter_panel(
    all_classes_owned: Vec<ChemicalClass>,
    mut selected_classes: Signal<Vec<String>>,
    families: &[(String, Vec<ChemicalClass>)],
) -> Element {
    rsx! {
        if !all_classes_owned.is_empty() {
            div { style: StyleBuilder::new().margin("0.8rem 0 0").padding("0.6rem").border("1px solid #e2e8f0").border_radius("10px").property("background", "#f8fafc").build(),
                div { style: StyleBuilder::new().display("flex").align_items("center").justify_content("space-between").property("margin-bottom", "0.6rem").build(),
                    h3 { style: StyleBuilder::new().margin("0").font_size("0.85rem").color("#0f172a").font_weight("700").property("text-transform", "uppercase").property("letter-spacing", "0.05em").build(), "Filter by chemical family" }
                    label { style: StyleBuilder::new().display("flex").align_items("center").gap("0.4rem").cursor("pointer").font_size("0.75rem").color("#475569").font_weight("600").build(),
                        input {
                            r#type: "checkbox",
                            checked: selected_classes.read().len() == all_classes_owned.len(),
                            onchange: move |_| {
                                let mut classes = selected_classes.read().clone();
                                if classes.len() == all_classes_owned.len() {
                                    classes.clear();
                                } else {
                                    classes = all_classes_owned.iter().map(|c| c.name.clone()).collect();
                                }
                                selected_classes.set(classes);
                            },
                            style: checkbox_sm(),
                        }
                        "Select All"
                    }
                }
                for (family, family_classes) in &families {
                    { family_entry(family, family_classes, selected_classes) }
                }
            }
        }
    }
}

#[allow(unused_variables, unused_mut)]
pub(super) fn family_entry(
    family: &str,
    family_classes: &[ChemicalClass],
    mut selected_classes: Signal<Vec<String>>,
) -> Element {
    let family_clone = family.to_string();
    let family_classes_clone = family_classes.to_vec();

    let selected_count = family_classes_clone
        .iter()
        .filter(|c| selected_classes.read().contains(&c.name))
        .count();
    let all_family_selected = selected_count == family_classes_clone.len();
    let some_family_selected = selected_count > 0 && !all_family_selected;

    rsx! {
        div { style: StyleBuilder::new().property("margin-bottom", "0.6rem").build(),
            label { style: StyleBuilder::new().display("flex").align_items("center").gap("0.4rem").cursor("pointer").property("margin-bottom", "0.3rem").build(),
                input {
                    r#type: "checkbox",
                    checked: all_family_selected || some_family_selected,
                    onchange: move |_| {
                        let mut classes = selected_classes.read().clone();
                        if all_family_selected || some_family_selected {
                            for c in &family_classes_clone {
                                classes.retain(|name| name != &c.name);
                            }
                        } else {
                            for c in &family_classes_clone {
                                if !classes.contains(&c.name) {
                                    classes.push(c.name.clone());
                                }
                            }
                        }
                        selected_classes.set(classes);
                    },
                    style: StyleBuilder::new().width("16px").height("16px").cursor("pointer").build(),
                }
                span { style: StyleBuilder::new().font_size("0.85rem").font_weight("700").color("#0f172a").build(), "{family_clone}" }
                if some_family_selected {
                    span { style: StyleBuilder::new().font_size("0.7rem").color("#94a3b8").build(), "({selected_count}/{family_classes.len()})" }
                }
            }
            ul { style: StyleBuilder::new().property("margin", "0 0 0 1.5rem").padding("0").property("list-style", "none").display("flex").flex_wrap("wrap").gap("0.4rem").build(),
                for class in family_classes.iter() {
                    {
                        let color = class.color.clone();
                        let class_name = class.name.clone();
                        let is_selected = selected_classes.read().contains(&class_name);
                        rsx! {
                            li { style: StyleBuilder::new().display("flex").align_items("center").build(),
                                label { style: StyleBuilder::new().display("flex").align_items("center").gap("0.4rem").cursor("pointer").build(),
                                    input {
                                        r#type: "checkbox",
                                        checked: is_selected,
                                        onchange: move |_| {
                                            let mut classes = selected_classes.read().clone();
                                            if is_selected {
                                                classes.retain(|c| c != &class_name);
                                            } else {
                                                classes.push(class_name.clone());
                                            }
                                            selected_classes.set(classes);
                                        },
                                        style: checkbox_sm(),
                                    }
                                    span { style: StyleBuilder::new().display("inline-flex").align_items("center").gap("0.3rem").padding("0.2rem 0.5rem").border_radius("999px").property("background", "#f1f5f9").font_size("0.75rem").font_weight("500").build(),
                                        span { style: StyleBuilder::new().width("8px").height("8px").border_radius("50%").property("background", &color).build(), }
                                        "{class.name}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[allow(unused_variables, unused_mut)]
fn download_buttons(
    filters: &SummaryFilters,
    input_format: crate::format::LipidFormat,
    filtered_mgf: &str,
    all_classes_owned: &[ChemicalClass],
    gallery: &[crate::parser::GalleryItem],
    blocks: &[crate::parser::SpectrumBlock],
) -> Element {
    let mut mz_min = filters.mz_min;
    let mut mz_max = filters.mz_max;
    let mut precursor_min = filters.precursor_min;
    let mut precursor_max = filters.precursor_max;
    let mut adduct_filter = filters.adduct_filter;
    let mut selected_classes = filters.selected_classes;
    // Clone gallery SMILES data for the download closure
    #[cfg(target_arch = "wasm32")]
    let gallery_smiles: Vec<GallerySmilesEntry> =
        prepare_gallery_smiles(gallery, all_classes_owned);
    #[cfg(not(target_arch = "wasm32"))]
    let gallery_smiles: Vec<GallerySmilesEntry> = Vec::new();

    // Clone block data for filtered MGF download
    #[cfg(target_arch = "wasm32")]
    let blocks_owned: Vec<crate::parser::SpectrumBlock> = blocks.to_vec();
    #[cfg(not(target_arch = "wasm32"))]
    let _blocks_owned: Vec<crate::parser::SpectrumBlock> = blocks.to_vec();

    // Pre-compute class tags for each block (for MGF download tagging)
    #[cfg_attr(not(target_arch = "wasm32"), allow(unused))]
    let block_class_tags: Vec<Vec<(String, String)>> =
        prepare_block_class_tags(blocks, gallery, all_classes_owned);

    // Clone filtered_mgf for the non-WASM download stub
    #[allow(unused)]
    let filtered_mgf_owned = filtered_mgf.to_string();

    rsx! {
        div { style: StyleBuilder::new().display("flex").gap("0.5rem").property("margin-top", "0.75rem").build(),
            if input_format == crate::format::LipidFormat::Mgf {
                button {
                    r#type: "button",
                    style: StyleBuilder::new().border("1px solid #cbd5e1").border_radius("8px").property("background", "#f8fafc").color("#334155").font_size("0.85rem").font_weight("600").padding("0.45rem 0.8rem").cursor("pointer").build(),
                    onclick: move |_| {
                        #[cfg(target_arch = "wasm32")]
                        download_filtered_mgf(&blocks_owned, &block_class_tags, *mz_min.read(), *mz_max.read(), *precursor_min.read(), *precursor_max.read(), &adduct_filter.read(), &selected_classes.read());
                        #[cfg(not(target_arch = "wasm32"))]
                        { let _ = &filtered_mgf_owned; }
                    },
                    "Download Filtered Lipids"
                }
            }
            if input_format == crate::format::LipidFormat::Smiles {
                button {
                    r#type: "button",
                    style: StyleBuilder::new().border("1px solid #cbd5e1").border_radius("8px").property("background", "#f8fafc").color("#334155").font_size("0.85rem").font_weight("600").padding("0.45rem 0.8rem").cursor("pointer").build(),
                    onclick: move |_| {
                        #[cfg(target_arch = "wasm32")]
                        download_filtered_smiles(&gallery_smiles, *mz_min.read(), *mz_max.read(), *precursor_min.read(), *precursor_max.read(), &adduct_filter.read(), &selected_classes.read());
                    },
                    "Download SMILES"
                }
            }
        }
    }
}

/// Renders the per-class summary panel with filter controls and download buttons.
pub(super) fn summary(
    summary_data: &crate::parser::Summary,
    filters: &SummaryFilters,
    all_classes: &[ChemicalClass],
    filtered_mgf: &str,
    gallery: &[crate::parser::GalleryItem],
    blocks: &[crate::parser::SpectrumBlock],
    input_format: crate::format::LipidFormat,
) -> Element {
    let selected_classes = filters.selected_classes;
    let mz_min = filters.mz_min;
    let mz_max = filters.mz_max;
    let precursor_min = filters.precursor_min;
    let precursor_max = filters.precursor_max;
    let adduct_filter = filters.adduct_filter;

    let lipid_spectra = summary_data.lipid_items;
    let total_spectra = summary_data.total_items;
    let skipped = summary_data.skipped;
    let unclassified = summary_data.unclassified;
    let all_classes_owned = all_classes.to_vec();
    let adduct_options = collect_adduct_options(gallery);
    let families = group_classes_by_family(&all_classes_owned);

    rsx! {
        div {
            style: StyleBuilder::new().property("margin-top", "1.25rem").padding("1rem 1.1rem").border("1px solid #e2e8f0").border_radius("16px").property("background", "linear-gradient(180deg, #ffffff 0%, #f8fafc 100%)").build(),
            h2 { style: section_subheading(), "Results" }
            div { style: StyleBuilder::new().display("flex").flex_wrap("wrap").gap("0.4rem").align_items("center").font_size("0.9rem").build(),
                span { style: StyleBuilder::new().color("#16a34a").font_weight("700").build(), "{lipid_spectra} items matching selected classes" }
                span { style: StyleBuilder::new().color("#475569").build(), "· out of {total_spectra} total" }
                if skipped > 0 {
                    span { style: StyleBuilder::new().color("#94a3b8").build(), "(skipped {skipped} items without SMILES or formula)" }
                }
                if unclassified > 0 {
                    span { style: StyleBuilder::new().color("#94a3b8").build(), "(ignored {unclassified} annotated non-lipid items)" }
                }
            }
            { filter_controls_row(mz_min, mz_max, precursor_min, precursor_max, adduct_filter, &adduct_options) }
            { family_filter_panel(all_classes_owned.clone(), selected_classes, &families) }
            { download_buttons(&SummaryFilters { selected_classes, mz_min, mz_max, precursor_min, precursor_max, adduct_filter }, input_format, filtered_mgf, &all_classes_owned, gallery, blocks) }
        }
    }
}

/// Builds a SMILES file content string from cloned gallery data with class tags.
pub(super) fn gallery_with_filter(
    gallery: &[crate::parser::GalleryItem],
    selected_classes: &[String],
    mz_min: f64,
    mz_max: f64,
    precursor_min: f64,
    precursor_max: f64,
    adduct_filter: &str,
) -> Element {
    // Filter gallery: must match a selected class AND satisfy range/adduct filters
    let filtered: Vec<_> = gallery
        .iter()
        .filter(|item| {
            if selected_classes.is_empty() {
                return false;
            }
            // Class filter
            let class_match = selected_classes
                .iter()
                .any(|class_name| item.class_matches.get(class_name).copied().unwrap_or(false));
            if !class_match {
                return false;
            }
            // m/z (exact_mass) range filter
            if item.exact_mass < mz_min || item.exact_mass > mz_max {
                return false;
            }
            // precursor range filter
            if let Some(pmz) = item.precursor_mz
                && (pmz < precursor_min || pmz > precursor_max)
            {
                return false;
            }
            // adduct filter
            if !adduct_filter.is_empty() && item.adduct.as_deref() != Some(adduct_filter) {
                return false;
            }
            true
        })
        .collect();

    let count = filtered.len();
    rsx! {
            div {
                style: StyleBuilder::new().property("margin-top", "1.25rem").build(),
                h2 { style: section_subheading(), "Structures matching selected classes ({count} shown)" }
                if count == 0 {
                    p { style: StyleBuilder::new().color("#64748b").build(), "Select one or more chemical families to see matching structures." }
                } else {
                    div {
                        style: StyleBuilder::new().display("grid").property("grid-template-columns", "repeat(auto-fill, minmax(320px, 1fr))").gap("0.75rem").build(),
                        for item in filtered.iter() {
                            {
                                let precursor_text = item
                                    .precursor_mz
                                    .map_or_else(|| "—".to_string(), |mz| format!("{mz:.3}"));
                                let adduct_text = item.adduct.as_deref().unwrap_or("-");
                                let bg_color = &item.primary_class_color;
                                rsx! {
                                    div { style: StyleBuilder::new()
                                            .property("background", &format!("linear-gradient(180deg, {bg_color}15 0%, {bg_color}08 100%)"))
                                            .padding("0.6rem 0.7rem")
                                            .border_radius("14px")
                                            .border(&format!("1px solid {bg_color}40"))
                                            .box_shadow("0 6px 16px rgba(15, 23, 42, 0.05)")
                                            .property("overflow", "hidden")
                                            .build(),
                                        div { style: StyleBuilder::new().display("flex").gap("0.55rem").align_items("flex-start").build(),
                                            div { style: StyleBuilder::new()
                                            .flex("0 0 auto")
                                            .width("160px")
                                            .height("120px")
                                            .display("grid")
                                            .property("place-items", "center")
                                            .property("background", &format!("{bg_color}10"))
                                            .border(&format!("1px solid {bg_color}30"))
                                            .border_radius("10px")
                                            .property("overflow", "hidden")
                                            .build(),
                                                div { style: StyleBuilder::new().width("100%").height("100%").display("flex").align_items("center").justify_content("center").build(),
                                                    div { dangerous_inner_html: item.svg.as_str() }
                                                }
                                            }
                                            div { style: StyleBuilder::new().flex("1 1 auto").property("min-width", "0").build(),
                                                if let Some(title) = &item.title {
                                                    div { style: StyleBuilder::new().property("margin-bottom", "0.25rem").color("#0f172a").font_size("0.82rem").font_weight("600").build(), "{title}" }
                                                }
                                                if let Some(smiles) = &item.smiles {
                                                    div { style: StyleBuilder::new().property("margin-bottom", "0.15rem").color("#64748b").font_size("0.72rem").property("font-family", "ui-monospace, monospace").property("max-width", "100%").property("overflow", "hidden").property("text-overflow", "ellipsis").property("white-space", "nowrap").build(), "{smiles}" }
                                                }
                                                div { style: StyleBuilder::new().color("#64748b").font_size("0.75rem").build(),
                                                    "m/z "
                                                    strong { style: StyleBuilder::new().color("#0f172a").build(), "{item.exact_mass:.3}" }
                                                    " · precursor "
                                                    strong { style: StyleBuilder::new().color("#0f172a").build(), "{precursor_text}" }
                                                    " · adduct "
                                                    strong { style: StyleBuilder::new().color("#0f172a").build(), "{adduct_text}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
    }
}
