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
use super::download::{download_filtered_mgf, download_filtered_smiles};
use super::gallery::passes_item_filters;
use super::types::SummaryFilters;

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
    // Load LMSD-based class scheme (same as Results panel)
    let all_classes = crate::chemical_class::lmsd_classes();

    // Group classes by family, then sort families by LIPID MAPS rank order.
    let families = group_classes_by_family(all_classes);

    let family_expanded = use_signal(Vec::<usize>::new);

    rsx! {
                div {
                    style: StyleBuilder::new().property("background", "rgba(255,255,255,0.9)").border("1px solid rgba(148,163,184,0.22)").border_radius("20px").box_shadow("0 12px 40px rgba(15, 23, 42, 0.08)").padding("1.25rem").property("margin-bottom", "1.25rem").build(),
                    h2 { style: StyleBuilder::new().margin("0 0 0.5rem").font_size("1.1rem").build(), "Available Lipid Classes" }
                    p { style: StyleBuilder::new().margin("0 0 0.75rem").color("#64748b").font_size("0.8rem").build(), "LMSD classification scheme — {all_classes.len()} classes across {families.len()} families" }

                    // Family sections - stacked vertically with headers
                    div { style: StyleBuilder::new().display("flex").flex_direction("column").gap("0.3rem").property("max-height", "320px").property("overflow-y", "auto").build(),
                        for (idx, (family, classes)) in families.iter().enumerate() {
                            // Family header with color bar
                            {
                                let family_clone = family.clone();
                                let family_color = classes.first().map_or_else(|| "#cbd5e1".to_string(), |c| c.color.clone());
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
                                            span { style: StyleBuilder::new().font_size("0.72rem").color("#94a3b8").build(), "{classes.len()} classes" }
                                        }
                                        if is_expanded {
                                            div { style: StyleBuilder::new().display("flex").flex_wrap("wrap").gap("0.3rem").property("padding", "0.4rem 0 0.2rem 0.8rem").build(),
                                                for class in classes.iter() {
                                                    div {
                                                        title: "{class.smarts}",
                                                        style: StyleBuilder::new().padding("0.3rem 0.5rem").property("background", "#f1f5f9").border("1px solid #e2e8f0").border_radius("6px").font_size("0.75rem").build(),
                                                        strong { style: StyleBuilder::new().property("word-break", "break-word").display("inline-flex").align_items("center").gap("0.25rem").build(),
                                                            span { style: StyleBuilder::new().width("6px").height("6px").border_radius("50%").property("background", &class.color).build(), }
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
                }
    }
}

pub(super) fn parse_filter_bound(raw: &str, empty_default: f64) -> Option<f64> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Some(empty_default);
    }
    trimmed.parse::<f64>().ok().filter(|v| v.is_finite())
}

#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables, unused_mut))]
fn bound_input(
    mut target: Signal<f64>,
    placeholder: &'static str,
    empty_default: f64,
    width: &'static str,
) -> Element {
    rsx! {
        input {
            r#type: "number",
            placeholder,
            value: "{target.read()}",
            oninput: move |evt| {
                if let Some(val) = parse_filter_bound(&evt.value(), empty_default) {
                    target.set(val);
                }
            },
            style: StyleBuilder::new().width(width).padding("0.3rem 0.5rem").border("1px solid #cbd5e1").border_radius("6px").font_size("0.8rem").build(),
        }
    }
}

/// Renders the filter controls row: exact mass range, precursor m/z range, and adduct dropdown.
#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables, unused_mut))]
fn filter_controls_row(
    mz_min: Signal<f64>,
    mz_max: Signal<f64>,
    precursor_min: Signal<f64>,
    precursor_max: Signal<f64>,
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
            // Exact mass range filter
            div { style: StyleBuilder::new().display("flex").flex_direction("column").gap("0.25rem").build(),
                span { style: StyleBuilder::new().font_size("0.7rem").color("#475569").font_weight("600").build(), "Exact mass (Da)" }
                div { style: StyleBuilder::new().display("flex").gap("0.3rem").align_items("center").build(),
                    { bound_input(mz_min, "min", 0.0, "75px") }
                    span { style: StyleBuilder::new().color("#94a3b8").font_size("0.9rem").build(), "–" }
                    { bound_input(mz_max, "max", 5000.0, "75px") }
                }
            }
            // Precursor m/z range filter
            div { style: StyleBuilder::new().display("flex").flex_direction("column").gap("0.25rem").build(),
                span { style: StyleBuilder::new().font_size("0.7rem").color("#475569").font_weight("600").build(), "Precursor m/z" }
                div { style: StyleBuilder::new().display("flex").gap("0.3rem").align_items("center").build(),
                    { bound_input(precursor_min, "min", 0.0, "80px") }
                    span { style: StyleBuilder::new().color("#94a3b8").font_size("0.9rem").build(), "–" }
                    { bound_input(precursor_max, "max", 5000.0, "80px") }
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
#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables, unused_mut))]
fn family_filter_panel(
    all_classes: &[ChemicalClass],
    mut selected_classes: Signal<Vec<String>>,
    families: &[(String, Vec<ChemicalClass>)],
) -> Element {
    let all_names: Vec<String> = all_classes.iter().map(|c| c.name.clone()).collect();
    let total_classes = all_names.len();
    rsx! {
        if total_classes > 0 {
            div { style: StyleBuilder::new().margin("0.8rem 0 0").padding("0.6rem").border("1px solid #e2e8f0").border_radius("10px").property("background", "#f8fafc").build(),
                div { style: StyleBuilder::new().display("flex").align_items("center").justify_content("space-between").property("margin-bottom", "0.6rem").build(),
                    h3 { style: StyleBuilder::new().margin("0").font_size("0.85rem").color("#0f172a").font_weight("700").property("text-transform", "uppercase").property("letter-spacing", "0.05em").build(), "Filter by chemical family" }
                    label { style: StyleBuilder::new().display("flex").align_items("center").gap("0.4rem").cursor("pointer").font_size("0.75rem").color("#475569").font_weight("600").build(),
                        input {
                            r#type: "checkbox",
                            checked: selected_classes.read().len() == total_classes,
                            onchange: move |_| {
                                let mut classes = selected_classes.read().clone();
                                if classes.len() == total_classes {
                                    classes.clear();
                                } else {
                                    classes.clone_from(&all_names);
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

#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables, unused_mut))]
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
                                    span {
                                        title: "{class.smarts}",
                                        style: StyleBuilder::new().display("inline-flex").align_items("center").gap("0.3rem").padding("0.2rem 0.5rem").border_radius("999px").property("background", "#f1f5f9").font_size("0.75rem").font_weight("500").build(),
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

#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables, unused_mut))]
fn download_buttons(
    filters: &SummaryFilters,
    input_format: crate::format::LipidFormat,
    analysis_signal: Signal<Option<crate::parser::Analysis>>,
) -> Element {
    let mz_min = filters.mz_min;
    let mz_max = filters.mz_max;
    let precursor_min = filters.precursor_min;
    let precursor_max = filters.precursor_max;
    let adduct_filter = filters.adduct_filter;
    let selected_classes = filters.selected_classes;

    rsx! {
        div { style: StyleBuilder::new().display("flex").gap("0.5rem").property("margin-top", "0.75rem").build(),
            if input_format == crate::format::LipidFormat::Mgf {
                button {
                    r#type: "button",
                    style: StyleBuilder::new().border("1px solid #cbd5e1").border_radius("8px").property("background", "#f8fafc").color("#334155").font_size("0.85rem").font_weight("600").padding("0.45rem 0.8rem").cursor("pointer").build(),
                    onclick: move |_| {
                        if let Some(analysis) = analysis_signal.read().as_ref() {
                            download_filtered_mgf(
                                &analysis.blocks,
                                &analysis.gallery,
                                &analysis.all_classes,
                                *mz_min.read(),
                                *mz_max.read(),
                                *precursor_min.read(),
                                *precursor_max.read(),
                                &adduct_filter.read(),
                                &selected_classes.read(),
                            );
                        }
                    },
                    "Download Filtered Lipids"
                }
            }
            if input_format == crate::format::LipidFormat::Smiles {
                button {
                    r#type: "button",
                    style: StyleBuilder::new().border("1px solid #cbd5e1").border_radius("8px").property("background", "#f8fafc").color("#334155").font_size("0.85rem").font_weight("600").padding("0.45rem 0.8rem").cursor("pointer").build(),
                    onclick: move |_| {
                        if let Some(analysis) = analysis_signal.read().as_ref() {
                            download_filtered_smiles(
                                &analysis.blocks,
                                &analysis.gallery,
                                &analysis.all_classes,
                                *mz_min.read(),
                                *mz_max.read(),
                                *precursor_min.read(),
                                *precursor_max.read(),
                                &adduct_filter.read(),
                                &selected_classes.read(),
                            );
                        }
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
    blocks: &[crate::parser::SpectrumBlock],
    analysis_signal: Signal<Option<crate::parser::Analysis>>,
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
    let adduct_options = collect_adduct_options(blocks);
    let families = group_classes_by_family(all_classes);

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
            { family_filter_panel(all_classes, selected_classes, &families) }
            { download_buttons(&SummaryFilters { selected_classes, mz_min, mz_max, precursor_min, precursor_max, adduct_filter }, input_format, analysis_signal) }
        }
    }
}

fn gallery_class_badge(item: &crate::parser::GalleryItem) -> Element {
    let bg_color = &item.primary_class_color;
    let badge_label = item.primary_class_name.as_deref().or_else(|| {
        item.classification
            .as_ref()
            .map(|c| c.class.lipidmaps_category())
    });
    let badge_title = item.classification.as_ref().map_or_else(String::new, |c| {
        let src = if c.derived_from_smiles {
            "SMILES"
        } else {
            "formula"
        };
        format!("{} ({src})", c.class.lipidmaps_category())
    });
    badge_label.map_or_else(
        || rsx! {},
        |class_name| {
            rsx! {
                div {
                    title: "{badge_title}",
                    style: StyleBuilder::new().display("inline-flex").align_items("center").gap("0.25rem").padding("0.1rem 0.45rem").property("margin-bottom", "0.2rem").border_radius("999px").property("background", &format!("{bg_color}22")).border(&format!("1px solid {bg_color}55")).color("#0f172a").font_size("0.68rem").font_weight("600").build(),
                    span { style: StyleBuilder::new().width("6px").height("6px").border_radius("50%").property("background", bg_color).build(), }
                    "{class_name}"
                }
            }
        },
    )
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
            passes_item_filters(
                Some(&item.class_matches),
                item.exact_mass,
                item.precursor_mz,
                item.adduct.as_deref(),
                selected_classes,
                mz_min,
                mz_max,
                precursor_min,
                precursor_max,
                adduct_filter,
            )
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
                                                    div { style: StyleBuilder::new().property("margin-bottom", "0.2rem").color("#0f172a").font_size("0.82rem").font_weight("600").build(), "{title}" }
                                                }
                                                { gallery_class_badge(item) }
                                                if let Some(smiles) = &item.smiles {
                                                    div { style: StyleBuilder::new().property("margin-bottom", "0.15rem").color("#64748b").font_size("0.72rem").property("font-family", "ui-monospace, monospace").property("max-width", "100%").property("overflow", "hidden").property("text-overflow", "ellipsis").property("white-space", "nowrap").build(), "{smiles}" }
                                                }
                                                div { style: StyleBuilder::new().color("#64748b").font_size("0.75rem").build(),
                                                    "exact mass "
                                                    strong { style: StyleBuilder::new().color("#0f172a").build(), "{item.exact_mass:.3} Da" }
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
