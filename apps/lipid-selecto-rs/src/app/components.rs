//! Reusable rendering components for the lipid-selecto-rs UI.
//!
//! Extracted from `app.rs` to keep the entry-point module focused on the
//! top-level layout/orchestration logic. All functions here are
//! `pub(super)` — visible only to the parent `app` module.

use dioxus::prelude::*;
use ui::prelude::*;

use crate::chemical_class::ChemicalClass;

/// LIPID MAPS family rank order (FA → GL → GP → SP → ST → PR → SL → PK).
/// Used to sort the "Filter by chemical family" groups in the Results UI
/// so they display in the standard LIPID MAPS classification hierarchy,
/// consistent with the colour-attribution architecture in `rules::colors`.
static LIPID_MAPS_FAMILY_RANK: [(&str, usize); 8] = [
    ("Fatty Acyls", 0),
    ("Glycerolipids", 1),
    ("Glycerophospholipids", 2),
    ("Sphingolipids", 3),
    ("Sterol Lipids", 4),
    ("Prenol Lipids", 5),
    ("Saccharolipids", 6),
    ("Polyketides", 7),
];

/// Lookup helper for [`LIPID_MAPS_FAMILY_RANK`].
fn family_rank(family: &str) -> usize {
    LIPID_MAPS_FAMILY_RANK
        .iter()
        .find(|(name, _)| *name == family)
        .map_or(99, |(_, rank)| *rank)
}

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

/// Renders the "Available Lipid Classes" card, sorted by priority.
pub(super) fn lipid_classes_card(
    gallery: &[crate::parser::GalleryItem],
    mut mz_min: Signal<f64>,
    mut mz_max: Signal<f64>,
    mut precursor_min: Signal<f64>,
    mut precursor_max: Signal<f64>,
    mut adduct_filter: Signal<String>,
) -> Element {
    // Load LMSD-based class scheme (same as Results panel)
    let all_classes = crate::chemical_class::lmsd_all();

    // Collect unique adduct values from gallery items for dropdown
    let adduct_values: std::collections::BTreeSet<String> = gallery
        .iter()
        .filter_map(|item| item.adduct.as_ref())
        .filter(|a| !a.is_empty())
        .cloned()
        .collect();
    let mut adduct_options: Vec<String> = adduct_values.into_iter().collect();
    adduct_options.sort_by(|a, b| {
        let a_plus = a.contains('+');
        let b_plus = b.contains('+');
        match (a_plus, b_plus) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.cmp(b),
        }
    });

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

    const MAX_CLASSES_PER_FAMILY: usize = 4;

    // Pre-compute family display data: (family_name, shown_classes, others_count, others_color)
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
            let others_color = classes
                .iter()
                .skip(MAX_CLASSES_PER_FAMILY)
                .next()
                .map(|c| c.color.clone())
                .unwrap_or_else(|| {
                    classes
                        .last()
                        .map(|c| c.color.clone())
                        .unwrap_or("#cbd5e1".to_string())
                });
            (family.clone(), shown, others, others_color)
        })
        .collect();

    rsx! {
                div {
                    style: StyleBuilder::new().property("background", "rgba(255,255,255,0.9)").border("1px solid rgba(148,163,184,0.22)").border_radius("20px").box_shadow("0 12px 40px rgba(15, 23, 42, 0.08)").padding("1.25rem").property("margin-bottom", "1.25rem").build(),
                    h2 { style: StyleBuilder::new().margin("0 0 0.75rem").font_size("1.1rem").build(), "Available Lipid Classes" }

                    // Filter controls row
                    div { style: StyleBuilder::new().display("flex").flex_wrap("wrap").gap("0.75rem").align_items("flex-end").property("margin-bottom", "1rem").build(),
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
                                    style: StyleBuilder::new().width("70px").padding("0.25rem 0.4rem").border("1px solid #cbd5e1").border_radius("4px").font_size("0.8rem").build(),
                                }
                                span { style: StyleBuilder::new().color("#94a3b8").font_size("0.8rem").build(), "–" }
                                input {
                                    r#type: "number",
                                    placeholder: "max",
                                    value: "{mz_max.read()}",
                                    oninput: move |evt| {
                                        if let Ok(val) = evt.value().parse::<f64>() {
                                            mz_max.set(val);
                                        }
                                    },
                                    style: StyleBuilder::new().width("70px").padding("0.25rem 0.4rem").border("1px solid #cbd5e1").border_radius("4px").font_size("0.8rem").build(),
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
                                    style: StyleBuilder::new().width("75px").padding("0.25rem 0.4rem").border("1px solid #cbd5e1").border_radius("4px").font_size("0.8rem").build(),
                                }
                                span { style: StyleBuilder::new().color("#94a3b8").font_size("0.8rem").build(), "–" }
                                input {
                                    r#type: "number",
                                    placeholder: "max",
                                    value: "{precursor_max.read()}",
                                    oninput: move |evt| {
                                        if let Ok(val) = evt.value().parse::<f64>() {
                                            precursor_max.set(val);
                                        }
                                    },
                                    style: StyleBuilder::new().width("75px").padding("0.25rem 0.4rem").border("1px solid #cbd5e1").border_radius("4px").font_size("0.8rem").build(),
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
                                style: StyleBuilder::new().width("110px").padding("0.25rem 0.4rem").border("1px solid #cbd5e1").border_radius("4px").font_size("0.8rem").build(),
                                option { value: "", "All" }
                                for adduct_opt in adduct_options.iter() {
                                    option { value: "{adduct_opt}", "{adduct_opt}" }
                                }
                            }
                        }
                    }

                    // Class list - grouped by family with first 4 + "N others"
                    div {
                        style: StyleBuilder::new().display("grid").property("grid-template-columns", "repeat(auto-fit, minmax(200px, 1fr))").gap("0.75rem").property("max-height", "300px").property("overflow-y", "auto").build(),
                        for (family, shown_classes, others_count, others_color) in family_display.iter() {
                            for class in shown_classes.iter() {
                                div {
                                    style: StyleBuilder::new().padding("0.6rem 0.8rem").property("background", "#f1f5f9").border("1px solid #e2e8f0").border_radius("8px").font_size("0.85rem").build(),
                                    strong { style: StyleBuilder::new().property("word-break", "break-word").display("inline-flex").align_items("center").gap("0.3rem").build(),
                                        span { style: StyleBuilder::new().width("8px").height("8px").border_radius("50%").property("background", &class.color).build(), }
                                        "{class.name}"
                                    }
                                    span { style: StyleBuilder::new().color("#94a3b8").font_size("0.75rem").property("margin-left", "0.5rem").build(), "({class.family})" }
                                }
                            }
                            if *others_count > 0 {
                                div {
                                    style: StyleBuilder::new().padding("0.6rem 0.8rem").property("background", "#f1f5f9").border("1px solid #e2e8f0").border_radius("8px").font_size("0.85rem").build(),
                                    strong { style: StyleBuilder::new().property("word-break", "break-word").display("inline-flex").align_items("center").gap("0.3rem").build(),
                                        span { style: StyleBuilder::new().width("8px").height("8px").border_radius("50%").property("background", &others_color).build(), }
                                        "{family}"
                                    }
                                    span { style: StyleBuilder::new().color("#94a3b8").font_size("0.75rem").property("margin-left", "0.5rem").build(), "({others_count} others)" }
                                }
                            }
                        }
                    }
                }
    }
}

/// Renders the per-class summary panel.
pub(super) fn summary(
    summary_data: &crate::parser::Summary,
    mut selected_classes: Signal<Vec<String>>,
    all_classes: &[ChemicalClass],
    _filtered_mgf: &str,
    _gallery: &[crate::parser::GalleryItem],
    _mz_min: Signal<f64>,
    _mz_max: Signal<f64>,
    _precursor_min: Signal<f64>,
    _precursor_max: Signal<f64>,
    _adduct_filter: Signal<String>,
) -> Element {
    let lipid_spectra = summary_data.lipid_items;
    let total_spectra = summary_data.total_items;
    let skipped = summary_data.skipped;
    let unclassified = summary_data.unclassified;

    // Convert to owned data to avoid lifetime issues in closures
    let all_classes_owned = all_classes.to_vec();

    // Group classes by family, then sort families by LIPID MAPS rank order.
    let mut families: Vec<(String, Vec<ChemicalClass>)> = Vec::new();
    for class in &all_classes_owned {
        if let Some(entry) = families.iter_mut().find(|(f, _)| f == &class.family) {
            entry.1.push(class.clone());
        } else {
            families.push((class.family.clone(), vec![class.clone()]));
        }
    }
    families.sort_by_key(|(f, _)| family_rank(f));

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
                                        // Uncheck all
                                        classes.clear();
                                    } else {
                                        // Check all
                                        classes = all_classes_owned.iter().map(|c| c.name.clone()).collect();
                                    }
                                    selected_classes.set(classes);
                                },
                                style: checkbox_sm(),
                            }
                            "Select All"
                        }
                    }
                    for (family, family_classes) in families.iter() {
                        { family_entry(family, family_classes, selected_classes) }
                    }
                }
            }
        }
    }
}

pub(super) fn family_entry(
    family: &str,
    family_classes: &[ChemicalClass],
    mut selected_classes: Signal<Vec<String>>,
) -> Element {
    let family_clone = family.to_string();
    let family_classes_clone = family_classes.to_vec();

    // Check how many children are selected
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
                            // Uncheck all in family
                            for c in &family_classes_clone {
                                classes.retain(|name| name != &c.name);
                            }
                        } else {
                            // Check all in family
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

/// Renders the structure-diagram gallery, filtered by selected classes.
pub(super) fn gallery_with_filter(
    gallery: &[crate::parser::GalleryItem],
    selected_classes: &[String],
) -> Element {
    // Filter gallery to only show items that match at least one selected class
    let filtered: Vec<_> = gallery
        .iter()
        .filter(|item| {
            if selected_classes.is_empty() {
                false
            } else {
                selected_classes
                    .iter()
                    .any(|class_name| item.class_matches.get(class_name).copied().unwrap_or(false))
            }
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
