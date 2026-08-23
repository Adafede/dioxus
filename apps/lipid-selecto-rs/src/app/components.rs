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

/// Renders the "Available Lipid Classes" card, showing LMSD-based classes
/// grouped by family with collapsible sections for each family.
pub(super) fn lipid_classes_card() -> Element {
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

    // Use a single open-collapsed state for simplicity (all expanded by default)
    let family_expanded = use_signal(|| Vec::<usize>::new());

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
                                let family_color = shown_classes.first().map(|c| c.color.clone())
                                    .unwrap_or_else(|| others_color.clone());
                                let idx_copy = idx;
                                let mut family_expanded_signal = family_expanded.clone();
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
                                                            span { style: StyleBuilder::new().width("6px").height("6px").border_radius("50%").property("background", &others_color).build(), }
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

/// Renders the per-class summary panel with filter controls and download buttons.
#[cfg_attr(target_arch = "wasm32", allow(unused_variables))]
pub(super) fn summary(
    summary_data: &crate::parser::Summary,
    mut selected_classes: Signal<Vec<String>>,
    all_classes: &[ChemicalClass],
    filtered_mgf: &str,
    gallery: &[crate::parser::GalleryItem],
    blocks: &[crate::parser::SpectrumBlock],
    input_format: crate::format::LipidFormat,
    mut mz_min: Signal<f64>,
    mut mz_max: Signal<f64>,
    mut precursor_min: Signal<f64>,
    mut precursor_max: Signal<f64>,
    mut adduct_filter: Signal<String>,
) -> Element {
    let lipid_spectra = summary_data.lipid_items;
    let total_spectra = summary_data.total_items;
    let skipped = summary_data.skipped;
    let unclassified = summary_data.unclassified;

    // Convert to owned data to avoid lifetime issues in closures
    let all_classes_owned = all_classes.to_vec();

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
    for class in &all_classes_owned {
        if let Some(entry) = families.iter_mut().find(|(f, _)| f == &class.family) {
            entry.1.push(class.clone());
        } else {
            families.push((class.family.clone(), vec![class.clone()]));
        }
    }
    families.sort_by_key(|(f, _)| family_rank(f));

    // Clone filtered_mgf for the non-WASM download stub
    #[cfg(not(target_arch = "wasm32"))]
    #[allow(unused)]
    let filtered_mgf_owned = filtered_mgf.to_string();

    // Clone gallery SMILES data for the download closure (must be 'static for WASM event handlers)
    // Include class info (category, main_class, sub_class) for tagging exports
    #[cfg(target_arch = "wasm32")]
    let gallery_smiles: Vec<(
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = gallery
        .iter()
        .map(|item| {
            // Use the same primary_class_name as the UI (computed in gallery_item())
            let category = item
                .classification
                .as_ref()
                .map(|c| c.class.lipidmaps_category().to_string())
                .unwrap_or_else(|| {
                    item.primary_class_name
                        .as_ref()
                        .and_then(|name| {
                            all_classes
                                .iter()
                                .find(|c| &c.name == name)
                                .map(|c| c.family.clone())
                        })
                        .unwrap_or_else(|| "Other Lipids [-]".to_string())
                });
            let main_class = item
                .primary_class_name
                .clone()
                .unwrap_or_else(|| "-".to_string());
            (
                item.title.clone(),
                item.smiles.clone(),
                Some(category),
                Some(main_class),
                Some("-".to_string()),
            )
        })
        .collect();
    #[cfg(not(target_arch = "wasm32"))]
    #[allow(unused)]
    let gallery_smiles: Vec<(
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = Vec::new();

    // Clone block data for filtered MGF download (must be 'static for WASM event handlers)
    #[cfg(target_arch = "wasm32")]
    let blocks_owned: Vec<crate::parser::SpectrumBlock> = blocks.to_vec();
    #[cfg(not(target_arch = "wasm32"))]
    #[allow(unused)]
    let _blocks_owned: Vec<crate::parser::SpectrumBlock> = blocks.to_vec();

    // Pre-compute class tags for each block (for MGF download tagging)
    // Use gallery items' primary_class_name to ensure export matches UI
    #[cfg(target_arch = "wasm32")]
    let block_class_tags: Vec<Vec<(String, String)>> = blocks
        .iter()
        .map(|block| {
            // Find the corresponding gallery item by block_index to get the same class as the UI
            let gallery_item = gallery.iter().find(|item| item.block_index == block.index);
            if let Some(item) = gallery_item {
                get_class_tag_from_gallery(item, all_classes)
            } else {
                get_class_tag(block, all_classes)
            }
        })
        .collect();
    #[cfg(not(target_arch = "wasm32"))]
    #[allow(unused)]
    let _block_class_tags: Vec<Vec<(String, String)>> = blocks
        .iter()
        .map(|block| {
            let gallery_item = gallery.iter().find(|item| item.block_index == block.index);
            if let Some(item) = gallery_item {
                get_class_tag_from_gallery(item, all_classes)
            } else {
                get_class_tag(block, all_classes)
            }
        })
        .collect();

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

            // Filter controls row
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
                        for adduct_opt in adduct_options.iter() {
                            option { value: "{adduct_opt}", "{adduct_opt}" }
                        }
                    }
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
                    for (family, family_classes) in families.iter() {
                        { family_entry(family, family_classes, selected_classes) }
                    }
                }
            }

            // Download buttons - conditional on input format, with filter support
            div { style: StyleBuilder::new().display("flex").gap("0.5rem").property("margin-top", "0.75rem").build(),
                if input_format == crate::format::LipidFormat::Mgf {
                    button {
                        r#type: "button",
                        style: StyleBuilder::new().border("1px solid #cbd5e1").border_radius("8px").property("background", "#f8fafc").color("#334155").font_size("0.85rem").font_weight("600").padding("0.45rem 0.8rem").cursor("pointer").build(),
                        onclick: move |_| {
                            #[cfg(target_arch = "wasm32")]
                            {
                                let mz_min_val = *mz_min.read();
                                let mz_max_val = *mz_max.read();
                                let prec_min_val = *precursor_min.read();
                                let prec_max_val = *precursor_max.read();
                                let adduct_val = adduct_filter.read().clone();
                                // Filter blocks and tag each with its attributed lipid class
                                let mut mgf_content = String::new();
                                for (idx, block) in blocks_owned.iter().enumerate() {
                                    // m/z range (using exact_mass)
                                    if block.exact_mass < mz_min_val || block.exact_mass > mz_max_val {
                                        continue;
                                    }
                                    // precursor range
                                    if let Some(pmz) = block.precursor_mz {
                                        if pmz < prec_min_val || pmz > prec_max_val {
                                            continue;
                                        }
                                    }
                                    // adduct filter
                                    if !adduct_val.is_empty() && block.adduct.as_deref() != Some(&adduct_val) {
                                        continue;
                                    }
                                    // Tag each block with its attributed lipid class
                                    let class_tags = &block_class_tags[idx];
                                    let tagged = insert_class_comment(&block.raw, class_tags);
                                    mgf_content.push_str(&tagged);
                                    mgf_content.push_str("\n");
                                }
                                let _ = upload::download_text(&mgf_content, "lipids_filtered.mgf");
                            }
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
                            {
                                let smiles_content = build_smiles_from_cloned(&gallery_smiles);
                                let _ = upload::download_text(&smiles_content, "lipids.smi");
                            }
                        },
                        "Download SMILES"
                    }
                }
            }
        }
    }
}

/// Builds a SMILES file content string from cloned gallery data with class tags.
#[cfg(target_arch = "wasm32")]
fn build_smiles_from_cloned(
    gallery: &[(
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    )],
) -> String {
    use std::fmt::Write;
    let mut content = String::new();
    for (title, smiles, category, main_class, sub_class) in gallery {
        if let Some(smiles) = smiles {
            // Format: SMILES <tab> Title <tab> CATEGORY <tab> MAIN_CLASS <tab> SUB_CLASS
            let cat_str = category.as_deref().filter(|s| !s.is_empty()).unwrap_or("-");
            let mc_str = main_class
                .as_deref()
                .filter(|s| !s.is_empty())
                .unwrap_or("-");
            let sc_str = sub_class
                .as_deref()
                .filter(|s| *s != "-" && !s.is_empty())
                .unwrap_or("-");
            match title {
                Some(title) => {
                    let _ = writeln!(
                        content,
                        "{}\t{}\t{}\t{}\t{}",
                        smiles, title, cat_str, mc_str, sc_str
                    );
                }
                None => {
                    let _ = writeln!(
                        content,
                        "{}\t{}\t{}\t{}\t{}",
                        smiles, "-", cat_str, mc_str, sc_str
                    );
                }
            }
        }
    }
    content
}

/// Returns LIPID MAPS class tags from a GalleryItem, ensuring the export uses
/// the same class assignment shown in the UI (via `primary_class_name`).
///
/// CATEGORY comes from the broad classification (`LipidClass::lipidmaps_category()`),
/// MAIN_CLASS comes from `primary_class_name` (the first matching LMSD class),
/// SUB_CLASS is always "-" (no sub-subclass info available).
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
fn get_class_tag_from_gallery(
    item: &crate::parser::GalleryItem,
    classes: &[ChemicalClass],
) -> Vec<(String, String)> {
    let category = item
        .classification
        .as_ref()
        .map(|c| c.class.lipidmaps_category().to_string())
        .unwrap_or_else(|| {
            // Fallback: look up the class family from primary_class_name
            item.primary_class_name
                .as_ref()
                .and_then(|name| classes.iter().find(|c| &c.name == name))
                .map(|c| c.family.clone())
                .unwrap_or_else(|| "Other Lipids [-]".to_string())
        });

    let main_class = item
        .primary_class_name
        .clone()
        .unwrap_or_else(|| "-".to_string());

    vec![
        ("CATEGORY".to_string(), category),
        ("MAIN_CLASS".to_string(), main_class),
        ("SUB_CLASS".to_string(), "-".to_string()),
    ]
}

/// Returns LIPID MAPS class tags for a spectrum block as a list of (key, value) pairs.
/// This is a fallback used when no matching GalleryItem is found.
///
/// Uses the broad `LipidClassification` for CATEGORY (mapped to proper LIPID MAPS
/// category names with codes like "Fatty Acyls [FA]"), and the first matching
/// LMSD subclass name for MAIN_CLASS. SUB_CLASS is always "-" (no sub-subclass
/// info available).
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
fn get_class_tag(
    block: &crate::parser::SpectrumBlock,
    classes: &[ChemicalClass],
) -> Vec<(String, String)> {
    // Broad category from structural/formula classification (LIPID MAPS category names)
    let category = block
        .classification
        .as_ref()
        .map(|c| c.class.lipidmaps_category().to_string())
        .unwrap_or_else(|| "Other Lipids [-]".to_string());

    // First matching LMSD subclass
    let main_class = block
        .gallery_item_matches
        .as_ref()
        .and_then(|matches| {
            // Find first true match
            let matched_name = matches
                .iter()
                .find(|(_, matched)| **matched)
                .map(|(name, _)| name.clone());

            // If found, try to look it up in classes to get the proper name
            matched_name.and_then(|name| {
                classes
                    .iter()
                    .find(|c| c.name == name)
                    .map(|c| c.name.clone())
                    .or(Some(name))
            })
        })
        .unwrap_or_else(|| "-".to_string());

    vec![
        ("CATEGORY".to_string(), category),
        ("MAIN_CLASS".to_string(), main_class),
        ("SUB_CLASS".to_string(), "-".to_string()),
    ]
}

/// Inserts COMMENT= lines with LIPID_MAPS class tags in the MGF header block (after BEGIN IONS, before peaks).
/// Each item is a (key, value) pair like ("CATEGORY", "Fatty Acyls [FA]").
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
fn insert_class_comment(block_raw: &str, class_tags: &[(String, String)]) -> String {
    let comment_block: String = class_tags
        .iter()
        .map(|(k, v)| format!("COMMENT=LIPID_MAPS_{}={}\n", k, v))
        .collect();
    // Find the end of the BEGIN IONS header line and insert after it
    if let Some(begin_pos) = block_raw.find(|c: char| !c.is_whitespace()) {
        if block_raw[begin_pos..].starts_with("BEGIN IONS") {
            // Find end of the BEGIN IONS line
            let after_begin = &block_raw[begin_pos..];
            if let Some(newline_pos) = after_begin.find('\n') {
                let (before, after) = after_begin.split_at(newline_pos + 1);
                return format!("{}{}{}", before, comment_block, after);
            }
        }
    }
    // Fallback: prepend with the comment
    format!("{}{}", comment_block, block_raw)
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

/// Renders the structure-diagram gallery, filtered by selected classes and m/p/adduct filters.
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
            if let Some(pmz) = item.precursor_mz {
                if pmz < precursor_min || pmz > precursor_max {
                    return false;
                }
            }
            // adduct filter
            if !adduct_filter.is_empty() {
                if item.adduct.as_deref() != Some(adduct_filter) {
                    return false;
                }
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
