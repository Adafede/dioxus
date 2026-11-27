// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Summary header, mass/adduct filters, family checklist, and export button.

use dioxus::prelude::*;
use lipid_core::{
    Analysis, ChemicalClass, ClassMatchSet, LipidFamily, LipidFormat, collect_adduct_options,
    lmsd_classes_by_family, passes_item_filters,
};
use ui::{ColorScheme, StyleBuilder};

use super::download::download_filtered;
use super::filter_controls::filter_controls_row;
use super::types::SummaryFilters;

fn section_subheading() -> String {
    StyleBuilder::new()
        .margin("0 0 0.5rem")
        .font_size("1.05rem")
        .color(ColorScheme::LIGHT.text)
        .build()
}

fn checkbox_style(size: &str) -> String {
    StyleBuilder::new()
        .width(size)
        .height(size)
        .cursor("pointer")
        .build()
}

fn family_filter_panel(
    total_classes: usize,
    mut selected_classes: Signal<ClassMatchSet>,
    families: &[(LipidFamily, Vec<&'static ChemicalClass>)],
) -> Element {
    if total_classes == 0 {
        return rsx! {};
    }
    let current_len = selected_classes.read().len();
    rsx! {
        div { style: StyleBuilder::new().margin("0.8rem 0 0").padding("0.6rem").border("1px solid #e2e8f0").border_radius("10px").property("background", "#f8fafc").build(),
            div { style: StyleBuilder::new().display("flex").align_items("center").justify_content("space-between").property("margin-bottom", "0.6rem").build(),
                h3 { style: StyleBuilder::new().margin("0").font_size("0.85rem").color(ColorScheme::LIGHT.text).font_weight("700").property("text-transform", "uppercase").property("letter-spacing", "0.05em").build(), "Filter by chemical family" }
                label { style: StyleBuilder::new().display("flex").align_items("center").gap("0.4rem").cursor("pointer").font_size("0.75rem").color("#475569").font_weight("600").build(),
                    input {
                        r#type: "checkbox",
                        checked: current_len == total_classes,
                        onchange: move |_| {
                            if selected_classes.read().len() == total_classes {
                                selected_classes.set(ClassMatchSet::empty());
                            } else {
                                selected_classes.set(ClassMatchSet::all(total_classes));
                            }
                        },
                        style: checkbox_style("14px"),
                    }
                    "Select All"
                }
            }
            for (family, family_classes) in families {
                { family_entry(*family, family_classes, selected_classes) }
            }
        }
    }
}

fn family_entry(
    family: LipidFamily,
    family_classes: &[&'static ChemicalClass],
    mut selected_classes: Signal<ClassMatchSet>,
) -> Element {
    let selected = *selected_classes.read();
    let selected_count = family_classes
        .iter()
        .filter(|c| selected.contains(c.index))
        .count();
    let all_selected = selected_count == family_classes.len();
    let some_selected = selected_count > 0 && !all_selected;
    let family_indices: Vec<usize> = family_classes.iter().map(|c| c.index).collect();

    rsx! {
        div { key: "{family.code()}", style: StyleBuilder::new().property("margin-bottom", "0.6rem").build(),
            label { style: StyleBuilder::new().display("flex").align_items("center").gap("0.4rem").cursor("pointer").property("margin-bottom", "0.3rem").build(),
                input {
                    r#type: "checkbox",
                    checked: all_selected || some_selected,
                    onchange: move |_| {
                        let mut next = *selected_classes.read();
                        for &idx in &family_indices {
                            if all_selected || some_selected {
                                next.remove(idx);
                            } else {
                                next.insert(idx);
                            }
                        }
                        selected_classes.set(next);
                    },
                    style: checkbox_style("16px"),
                }
                span { style: StyleBuilder::new().font_size("0.85rem").font_weight("700").color(ColorScheme::LIGHT.text).build(), "{family.name()}" }
                if some_selected {
                    span { style: StyleBuilder::new().font_size("0.7rem").color("#94a3b8").build(), "({selected_count}/{family_classes.len()})" }
                }
            }
            ul { style: StyleBuilder::new().property("margin", "0 0 0 1.5rem").padding("0").property("list-style", "none").display("flex").flex_wrap("wrap").gap("0.4rem").build(),
                for class in family_classes {
                    { class_chip(class, selected_classes) }
                }
            }
        }
    }
}

fn class_chip(
    class: &'static ChemicalClass,
    mut selected_classes: Signal<ClassMatchSet>,
) -> Element {
    let is_selected = selected_classes.read().contains(class.index);
    rsx! {
        li { key: "{class.index}", style: StyleBuilder::new().display("flex").align_items("center").build(),
            label { style: StyleBuilder::new().display("flex").align_items("center").gap("0.4rem").cursor("pointer").build(),
                input {
                    r#type: "checkbox",
                    checked: is_selected,
                    onchange: move |_| {
                        let mut next = *selected_classes.read();
                        next.toggle(class.index);
                        selected_classes.set(next);
                    },
                    style: checkbox_style("14px"),
                }
                span {
                    title: "{class.smarts}",
                    style: StyleBuilder::new().display("inline-flex").align_items("center").gap("0.3rem").padding("0.2rem 0.5rem").border_radius("999px").property("background", "#f1f5f9").font_size("0.75rem").font_weight("500").build(),
                    span { style: StyleBuilder::new().width("8px").height("8px").border_radius("50%").property("background", class.color).build(), }
                    "{class.name}"
                }
            }
        }
    }
}

fn download_buttons(
    filters: SummaryFilters,
    input_format: LipidFormat,
    analysis_signal: Signal<Option<Analysis>>,
    status: Signal<String>,
) -> Element {
    let label = match input_format {
        LipidFormat::Mgf => "Download Filtered Lipids",
        LipidFormat::Smiles => "Download SMILES",
    };
    rsx! {
        div { style: StyleBuilder::new().display("flex").gap("0.5rem").property("margin-top", "0.75rem").build(),
            button {
                r#type: "button",
                style: StyleBuilder::new().border("1px solid #cbd5e1").border_radius("8px").property("background", "#f8fafc").color("#334155").font_size("0.85rem").font_weight("600").padding("0.45rem 0.8rem").cursor("pointer").build(),
                onclick: move |_| {
                    if let Some(analysis) = analysis_signal.read().as_ref() {
                        filters.with_criteria(|criteria| {
                            download_filtered(input_format, analysis, criteria, status);
                        });
                    }
                },
                "{label}"
            }
        }
    }
}

/// Renders counts, interactive filters (memoized options/families), and export controls.
#[component]
pub(super) fn SummaryPanel(
    analysis: Signal<Option<Analysis>>,
    filters: SummaryFilters,
    input_format: LipidFormat,
    status: Signal<String>,
) -> Element {
    let adduct_options = use_memo(move || {
        analysis
            .read()
            .as_ref()
            .map_or_else(Vec::new, |a| collect_adduct_options(&a.blocks))
    });
    let filtered_count = use_memo(move || {
        let guard = analysis.read();
        let Some(data) = guard.as_ref() else {
            return 0usize;
        };
        filters.with_criteria(|criteria| {
            data.blocks
                .iter()
                .filter(|block| {
                    block.is_lipid()
                        && passes_item_filters(
                            block.class_matches(),
                            block.exact_mass,
                            block.precursor_mz,
                            block.adduct.as_deref(),
                            criteria,
                        )
                })
                .count()
        })
    });
    let families = lmsd_classes_by_family();
    let guard = analysis.read();
    let Some(data) = guard.as_ref() else {
        return rsx! {};
    };
    let s = data.summary;
    let filtered_count = *filtered_count.read();

    rsx! {
        div {
            style: StyleBuilder::new().property("margin-top", "1.25rem").padding("1rem 1.1rem").border("1px solid #e2e8f0").border_radius("16px").property("background", "linear-gradient(180deg, #ffffff 0%, #f8fafc 100%)").build(),
            h2 { style: section_subheading(), "Results" }
            div { style: StyleBuilder::new().display("flex").flex_wrap("wrap").gap("0.4rem").align_items("center").font_size("0.9rem").build(),
                span { style: StyleBuilder::new().color("#16a34a").font_weight("700").build(), "{filtered_count} items matching filters" }
                span { style: StyleBuilder::new().color("#475569").build(), "· {s.lipid_items} lipids out of {s.total_items} total" }
                if s.skipped > 0 {
                    span { style: StyleBuilder::new().color("#94a3b8").build(), "(skipped {s.skipped} items without SMILES or formula)" }
                }
                if s.unclassified > 0 {
                    span { style: StyleBuilder::new().color("#94a3b8").build(), "(ignored {s.unclassified} annotated non-lipid items)" }
                }
            }
            { filter_controls_row(filters, &adduct_options.read()) }
            { family_filter_panel(data.all_classes.len(), filters.selected_classes, families) }
            { download_buttons(filters, input_format, analysis, status) }
        }
    }
}
