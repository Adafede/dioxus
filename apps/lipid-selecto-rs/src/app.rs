//! Dioxus UI for `lipid-selecto-rs`: drag-and-drop file upload (MGF or SMILES),
//! lipid classification with extensible rules, a "Download" button, and a gallery
//! of structure diagrams.
//!
//! The `app` entry point intentionally is **not** annotated with `#[component]`
//! so that `dioxus::launch(lipid_selecto_rs::app)` keeps working exactly like the
//! sibling `mgf-precursor-erro-rs` app.

use dioxus::events::{DragData, FormData};
use dioxus::html::HasFileData;
use dioxus::prelude::*;
use ui::prelude::*;

use crate::chemical_class::ChemicalClass;
use crate::format::LipidFormat;
use crate::parser::Analysis;
use crate::rules::LipidRuleLibrary;

mod browser;

#[cfg(target_arch = "wasm32")]
use self::browser::begin_analysis_from_blob;

/// Renders the lipid selection UI.
///
/// # Errors
///
/// Returns an error if the component tree fails to build or render.
#[allow(clippy::too_many_lines)]
pub fn app() -> Element {
    let mut status = use_signal(|| "Drop an MGF or SMILES file to begin.".to_string());
    let mut drag_active = use_signal(|| false);
    #[cfg(target_arch = "wasm32")]
    let file_name = use_signal(String::new);
    #[cfg(not(target_arch = "wasm32"))]
    let mut file_name = use_signal(String::new);
    #[cfg(target_arch = "wasm32")]
    let busy = use_signal(|| false);
    #[cfg(not(target_arch = "wasm32"))]
    let mut busy = use_signal(|| false);
    let analysis = use_signal(|| None::<Analysis>);
    let mut input_format = use_signal(|| None::<LipidFormat>);

    let rule_library = LipidRuleLibrary::defaults();

    // Initialize selected_classes with all ChemicalClass names (ensures Ceramide is included)
    let all_class_names: Vec<_> = ChemicalClass::defaults()
        .iter()
        .map(|c| c.name.clone())
        .collect();

    // Start with all classes selected for initial load
    let selected_classes = use_signal(|| all_class_names.clone());

    #[cfg(target_arch = "wasm32")]
    let rule_lib_for_file_change = rule_library.clone();
    let on_file_change = move |evt: Event<FormData>| match upload::extract_blob_from_file_data(
        &evt.data().files(),
    ) {
        Ok(Some(file)) => {
            let detected_format = LipidFormat::from_path(&file.name);
            input_format.set(detected_format);

            #[cfg(target_arch = "wasm32")]
            begin_analysis_from_blob(
                file.blob,
                file.name,
                file_name,
                status,
                busy,
                drag_active,
                analysis,
                detected_format,
                rule_lib_for_file_change.clone(),
            );

            #[cfg(not(target_arch = "wasm32"))]
            {
                file_name.set(file.name);
                status.set("This app runs in the browser — open it via `dx serve`.".to_string());
                busy.set(false);
            }
        }
        Ok(None) => status.set("No file selected.".to_string()),
        Err(msg) => status.set(msg),
    };

    let on_drag_enter = move |evt: Event<DragData>| {
        evt.prevent_default();
        drag_active.set(true);
    };
    let on_drag_over = move |evt: Event<DragData>| {
        evt.prevent_default();
        drag_active.set(true);
    };
    let on_drag_leave = move |evt: Event<DragData>| {
        evt.prevent_default();
        drag_active.set(false);
    };

    #[cfg(target_arch = "wasm32")]
    let rule_lib_for_drop = rule_library.clone();
    let on_drop = move |evt: Event<DragData>| {
        evt.prevent_default();
        drag_active.set(false);
        match upload::extract_blob_from_file_data(&evt.data().files()) {
            Ok(Some(file)) => {
                let detected_format = LipidFormat::from_path(&file.name);
                input_format.set(detected_format);

                #[cfg(target_arch = "wasm32")]
                begin_analysis_from_blob(
                    file.blob,
                    file.name,
                    file_name,
                    status,
                    busy,
                    drag_active,
                    analysis,
                    detected_format,
                    rule_lib_for_drop.clone(),
                );

                #[cfg(not(target_arch = "wasm32"))]
                {
                    file_name.set(file.name);
                    status
                        .set("This app runs in the browser — open it via `dx serve`.".to_string());
                    busy.set(false);
                }
            }
            Ok(None) => status.set("No file selected.".to_string()),
            Err(msg) => status.set(msg),
        }
    };

    #[cfg(target_arch = "wasm32")]
    let rule_lib_for_button = rule_library.clone();
    let upload_border = if *drag_active.read() {
        "#2563eb"
    } else {
        "#94a3b8"
    };
    let upload_background = if *drag_active.read() {
        "linear-gradient(135deg, rgba(219,234,254,0.96), rgba(239,246,255,0.94))"
    } else {
        "linear-gradient(135deg, rgba(248,250,252,0.95), rgba(239,246,255,0.95))"
    };

    rsx! {
        DocumentHead {
            title: "Lipid Selecto-rs".to_string(),
            lang: "en".to_string(),
            theme_colors: Some(("#f6f8fb", "#10141b")),
            scripts: vec!["https://scripts.simpleanalyticscdn.com/latest.js".to_string()],
            inline_style: Some(
                ".skip-link:focus{top:0!important;outline:3px solid #0b5cab;outline-offset:2px}\
                 -webkit-text-size-adjust:100%;-moz-text-size-adjust:100%;text-size-adjust:100%"
                    .to_string()
            ),
        }

        div {
            style: "min-height: 100vh; padding: 2rem 1rem 3rem; background: linear-gradient(135deg, #f8fafc 0%, #eef2ff 100%); color: #0f172a; font-family: ui-system, system-ui, sans-serif;",
            main { id: "main",
                style: "max-width: 1100px; margin: 0 auto;",
                h1 { style: "margin: 0 0 0.35rem; font-size: 1.8rem; letter-spacing: -0.02em;", "Lipid Selecto-rs" }
                p {
                    style: "margin: 0 0 1.25rem; color: #475569; font-size: 0.95rem; max-width: 60rem;",
                    "Drop an MGF or SMILES file and we'll filter it to keep only lipids matching extensible LIPID MAPS-aligned rules. Download as the same format you uploaded."
                }


                div {
                    style: "background: rgba(255,255,255,0.9); border: 1px solid rgba(148,163,184,0.22); border-radius: 20px; box-shadow: 0 12px 40px rgba(15, 23, 42, 0.08); padding: 1.25rem; margin-bottom: 1.25rem;",
                    h2 { style: "margin: 0 0 0.75rem; font-size: 1.1rem;", "Available Lipid Classes" }
                    div {
                        style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 0.75rem; max-height: 300px; overflow-y: auto;",
                        for rule in rule_library.sorted_by_priority() {
                            div {
                                style: "padding: 0.6rem 0.8rem; background: #f1f5f9; border: 1px solid #e2e8f0; border-radius: 8px; font-size: 0.85rem;",
                                strong { "{rule.name}" }
                                if !rule.description.is_empty() {
                                    span { style: "color: #64748b; font-size: 0.8rem; margin-left: 0.3rem;", " - {rule.description}" }
                                }
                                span { style: "color: #94a3b8; font-size: 0.75rem; margin-left: 0.5rem;", "({rule.family})" }
                            }
                        }
                    }
                }

                div {
                    style: "background: rgba(255,255,255,0.9); border: 1px solid rgba(148,163,184,0.22); border-radius: 20px; box-shadow: 0 12px 40px rgba(15, 23, 42, 0.08); padding: 1.25rem;",
                    label {
                        r#for: "mgf-upload",
                        style: format!(
                            "display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 0.6rem; min-height: 140px; width: 100%; box-sizing: border-box; position: relative; isolation: isolate; border: 2px dashed {upload_border}; border-radius: 18px; padding: 1.1rem; cursor: pointer; background: {upload_background}; color: #334155; font-weight: 600; text-align: center; transition: border-color 160ms ease, background 160ms ease, transform 160ms ease;",
                        ),
                        ondragenter: on_drag_enter,
                        ondragover: on_drag_over,
                        ondragleave: on_drag_leave,
                        ondrop: on_drop,
                        span { style: "font-size: 1rem;", "Drop an MGF or SMILES file here or click to browse" }
                        span { style: "font-size: 0.85rem; font-weight: 500; color: #64748b;", ".mgf or .smi files with SMILES/FORMULA annotations" }
                        input {
                            id: "mgf-upload",
                            r#type: "file",
                            accept: ".mgf,text/plain,*",
                            disabled: *busy.read(),
                            onchange: on_file_change,
                            style: "position: absolute; inset: 0; width: 100%; height: 100%; opacity: 0; cursor: pointer;",
                        }
                    }
                    p {
                        id: "mgf-upload-help",
                        style: "margin: 0.7rem 0 0; color: #475569; font-size: 0.9rem;",
                        "Accepts .mgf or .smi files. Use drag and drop or browse."
                    }
                    button {
                        r#type: "button",
                        style: "margin-top: 0.75rem; border: 1px solid #cbd5e1; border-radius: 8px; background: #f8fafc; color: #334155; font-size: 0.85rem; font-weight: 600; padding: 0.5rem 0.9rem; cursor: pointer; width: 100%;",
                        onclick: move |_| {
                            #[cfg(target_arch = "wasm32")]
                            let _ = browser::load_example_dataset(
                                file_name.clone(),
                                status.clone(),
                                busy.clone(),
                                drag_active.clone(),
                                analysis.clone(),
                                input_format.clone(),
                                rule_lib_for_button.clone(),
                            );
                            #[cfg(not(target_arch = "wasm32"))]
                            {
                                status.set("This app runs in the browser — open it via `dx serve`.".to_string());
                            }
                        },
                        "Load Example SMILES"
                    }
                    if !file_name.read().is_empty() {
                        p {
                            style: "margin: 0.35rem 0 0; color: #475569; font-size: 0.9rem;",
                            "Selected file: {file_name}"
                        }
                    }
                    p {
                        id: "mgf-upload-status",
                        role: "status",
                        aria_live: "polite",
                        aria_atomic: "true",
                        style: "margin: 0.7rem 0 0; font-weight: 600; color: #334155;",
                        "{status}"
                    }
                }

                if let Some(analysis) = analysis.read().as_ref() {
                    { self::summary(&analysis.summary, selected_classes, &analysis.all_classes) }
                    { self::gallery_with_filter(&analysis.gallery, &selected_classes.read()) }
                }
            }
        }
    }
}

/// Renders the per-class summary panel.
#[allow(clippy::too_many_lines)]
fn summary(
    summary_data: &crate::parser::Summary,
    mut selected_classes: Signal<Vec<String>>,
    all_classes: &[ChemicalClass],
) -> Element {
    let lipid_spectra = summary_data.lipid_items;
    let total_spectra = summary_data.total_items;
    let skipped = summary_data.skipped;
    let unclassified = summary_data.unclassified;

    // Convert to owned data to avoid lifetime issues in closures
    let all_classes_owned = all_classes.to_vec();

    // Group classes by family, preserving order
    let mut families: Vec<(String, Vec<ChemicalClass>)> = Vec::new();
    for class in &all_classes_owned {
        if let Some(entry) = families.iter_mut().find(|(f, _)| f == &class.family) {
            entry.1.push(class.clone());
        } else {
            families.push((class.family.clone(), vec![class.clone()]));
        }
    }

    rsx! {
        div {
            style: "margin-top: 1.25rem; padding: 1rem 1.1rem; border: 1px solid #e2e8f0; border-radius: 16px; background: linear-gradient(180deg, #ffffff 0%, #f8fafc 100%);",
            h2 { style: "margin: 0 0 0.5rem; font-size: 1.05rem; color: #0f172a;", "Results" }
            div { style: "display: flex; flex-wrap: wrap; gap: 0.4rem; align-items: center; font-size: 0.9rem;",
                span { style: "color: #16a34a; font-weight: 700;", "{lipid_spectra} items matching selected classes" }
                span { style: "color: #475569;", "· out of {total_spectra} total" }
                if skipped > 0 {
                    span { style: "color: #94a3b8;", "(skipped {skipped} items without SMILES or formula)" }
                }
                if unclassified > 0 {
                    span { style: "color: #94a3b8;", "(ignored {unclassified} annotated non-lipid items)" }
                }
            }

            if !all_classes_owned.is_empty() {
                div { style: "margin: 0.8rem 0 0; padding: 0.6rem; border: 1px solid #e2e8f0; border-radius: 10px; background: #f8fafc;",
                    div { style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 0.6rem;",
                        h3 { style: "margin: 0; font-size: 0.85rem; color: #0f172a; font-weight: 700; text-transform: uppercase; letter-spacing: 0.05em;", "Filter by chemical family" }
                        label { style: "display: flex; align-items: center; gap: 0.4rem; cursor: pointer; font-size: 0.75rem; color: #475569; font-weight: 600;",
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
                                style: "width: 14px; height: 14px; cursor: pointer;",
                            }
                            "Select All"
                        }
                    }
                    for (family, family_classes) in families.iter() {
                        {
                            let family_clone = family.clone();
                            let family_classes_clone = family_classes.clone();

                            // Check how many children are selected
                            let selected_count = family_classes_clone.iter()
                                .filter(|c| selected_classes.read().contains(&c.name))
                                .count();
                            let all_family_selected = selected_count == family_classes_clone.len();
                            let some_family_selected = selected_count > 0 && !all_family_selected;

                            rsx! {
                                div { style: "margin-bottom: 0.6rem;",
                                    label { style: "display: flex; align-items: center; gap: 0.4rem; cursor: pointer; margin-bottom: 0.3rem;",
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
                                            style: "width: 16px; height: 16px; cursor: pointer;",
                                        }
                                        span { style: "font-size: 0.85rem; font-weight: 700; color: #0f172a;", "{family_clone}" }
                                        if some_family_selected {
                                            span { style: "font-size: 0.7rem; color: #94a3b8;", "({selected_count}/{family_classes.len()})" }
                                        }
                                    }
                                    ul { style: "margin: 0 0 0 1.5rem; padding: 0; list-style: none; display: flex; flex-wrap: wrap; gap: 0.4rem;",
                                        for class in family_classes.iter() {
                                            {
                                                let color = class.color.clone();
                                                let class_name = class.name.clone();
                                                let is_selected = selected_classes.read().contains(&class_name);
                                                rsx! {
                                                    li { style: "display: flex; align-items: center;",
                                                        label { style: "display: flex; align-items: center; gap: 0.4rem; cursor: pointer;",
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
                                                                style: "width: 14px; height: 14px; cursor: pointer;",
                                                            }
                                                            span { style: "display: inline-flex; align-items: center; gap: 0.3rem; padding: 0.2rem 0.5rem; border-radius: 999px; background: #f1f5f9; font-size: 0.75rem; font-weight: 500;",
                                                                span { style: format!("width: 8px; height: 8px; border-radius: 50%; background: {color};"), }
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
        }
    }
}

/// Renders the structure-diagram gallery, filtered by selected classes.
fn gallery_with_filter(
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
                style: "margin-top: 1.25rem;",
                h2 { style: "margin: 0 0 0.5rem; font-size: 1.05rem; color: #0f172a;", "Structures matching selected classes ({count} shown)" }
                if count == 0 {
                    p { style: "color: #64748b;", "Select one or more chemical families to see matching structures." }
                } else {
                    div {
                        style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: 0.75rem;",
                        for item in filtered.iter() {
                            {
                                let precursor_text = item
                                    .precursor_mz
                                    .map_or_else(|| "—".to_string(), |mz| format!("{mz:.3}"));
                                let charge_text = item.charge.as_deref().unwrap_or("—");
                                let bg_color = &item.primary_class_color;
                                rsx! {
                                    div { style: "background: linear-gradient(180deg, {bg_color}15 0%, {bg_color}08 100%); padding: 0.6rem 0.7rem; border-radius: 14px; border: 1px solid {bg_color}40; box-shadow: 0 6px 16px rgba(15, 23, 42, 0.05); overflow: hidden;",
                                        div { style: "display: flex; gap: 0.55rem; align-items: flex-start;",
                                            div { style: "flex: 0 0 auto; width: 160px; height: 120px; display: grid; place-items: center; background: {bg_color}10; border: 1px solid {bg_color}30; border-radius: 10px; overflow: hidden;",
                                                div { style: "width: 100%; height: 100%; display: flex; align-items: center; justify-content: center;",
                                                    div { dangerous_inner_html: item.svg.as_str() }
                                                }
                                            }
                                            div { style: "flex: 1 1 auto; min-width: 0;",
                                                if let Some(title) = &item.title {
                                                    div { style: "margin-bottom: 0.25rem; color: #0f172a; font-size: 0.82rem; font-weight: 600;", "{title}" }
                                                }
                                                if let Some(smiles) = &item.smiles {
                                                    div { style: "margin-bottom: 0.15rem; color: #64748b; font-size: 0.72rem; font-family: ui-monospace, monospace; max-width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;", "{smiles}" }
                                                }
                                                div { style: "color: #64748b; font-size: 0.75rem;",
                                                    "m/z "
                                                    strong { style: "color: #0f172a;", "{item.exact_mass:.3}" }
                                                    " · precursor "
                                                    strong { style: "color: #0f172a;", "{precursor_text}" }
                                                    " · charge "
                                                    strong { style: "color: #0f172a;", "{charge_text}" }
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
