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

fn section_subheading() -> String {
    StyleBuilder::new()
        .margin("0 0 0.5rem")
        .font_size("1.05rem")
        .color("#0f172a")
        .build()
}

fn checkbox_sm() -> String {
    StyleBuilder::new()
        .width("14px")
        .height("14px")
        .cursor("pointer")
        .build()
}

/// Renders the lipid selection UI.
///
/// # Errors
///
/// Returns an error if the component tree fails to build or render.
pub fn app() -> Element {
    let status = use_signal(|| "Drop an MGF or SMILES file to begin.".to_string());
    let drag_active = use_signal(|| false);
    let file_name = use_signal(String::new);
    let busy = use_signal(|| false);
    let analysis = use_signal(|| None::<Analysis>);
    let input_format = use_signal(|| None::<LipidFormat>);

    let rule_library = LipidRuleLibrary::defaults();

    // Initialize selected_classes with all ChemicalClass names (ensures Ceramide is included)
    let all_class_names: Vec<_> = ChemicalClass::defaults()
        .iter()
        .map(|c| c.name.clone())
        .collect();

    // Start with all classes selected for initial load
    let selected_classes = use_signal(|| all_class_names.clone());

    let ctx = UploadCtx {
        file_name,
        status,
        busy,
        analysis,
        input_format,
    };

    let rule_lib_for_change = rule_library.clone();
    let rule_lib_for_drop = rule_library.clone();

    let on_file_change = move |evt: Event<FormData>| {
        handle_uploaded_files(
            ctx,
            &rule_lib_for_change,
            upload::extract_blob_from_file_data(&evt.data().files()),
            drag_active,
        );
    };

    let on_drop = move |evt: Event<DragData>| {
        handle_uploaded_files(
            ctx,
            &rule_lib_for_drop,
            upload::extract_blob_from_file_data(&evt.data().files()),
            drag_active,
        );
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
            style: StyleBuilder::new().min_height("100vh").padding("2rem 1rem 3rem").property("background", "linear-gradient(135deg, #f8fafc 0%, #eef2ff 100%)").color("#0f172a").property("font-family", "ui-system, system-ui, sans-serif").build(),
            main { id: "main",
                style: StyleBuilder::new().property("max-width", "1100px").margin("0 auto").build(),
                h1 { style: StyleBuilder::new().margin("0 0 0.35rem").font_size("1.8rem").property("letter-spacing", "-0.02em").build(), "Lipid Selecto-rs" }
                p {
                    style: StyleBuilder::new().margin("0 0 1.25rem").color("#475569").font_size("0.95rem").property("max-width", "60rem").build(),
                    "Drop an MGF or SMILES file and we'll filter it to keep only lipids matching extensible LIPID MAPS-aligned rules. Download as the same format you uploaded."
                }

                { lipid_classes_card(&rule_library) }
                div {
                    style: StyleBuilder::new().property("background", "rgba(255,255,255,0.9)").border("1px solid rgba(148,163,184,0.22)").border_radius("20px").box_shadow("0 12px 40px rgba(15, 23, 42, 0.08)").padding("1.25rem").build(),
                    UploadZone {
                        file_name,
                        status,
                        busy,
                        drag_active,
                        on_file_change,
                        on_drop,
                        accept: ".mgf,text/plain,*",
                        label: "Drop an MGF or SMILES file here or click to browse",
                        hint: ".mgf or .smi files with SMILES/FORMULA annotations",
                        icon: "📁",
                    }
                    button {
                        r#type: "button",
                        style: StyleBuilder::new().property("margin-top", "0.75rem").border("1px solid #cbd5e1").border_radius("8px").property("background", "#f8fafc").color("#334155").font_size("0.85rem").font_weight("600").padding("0.5rem 0.9rem").cursor("pointer").width("100%").build(),
                        onclick: move |_| {
                            #[cfg(target_arch = "wasm32")]
                            let _ = browser::load_example_dataset(
                                ctx.file_name,
                                ctx.status,
                                ctx.busy,
                                drag_active,
                                ctx.analysis,
                                ctx.input_format,
                                rule_library.clone(),
                            );
                        },
                        "Load Example SMILES"
                    }
                }
                if let Some(analysis) = ctx.analysis.read().as_ref() {
                    { self::summary(&analysis.summary, selected_classes, &analysis.all_classes) }
                    { self::gallery_with_filter(&analysis.gallery, &selected_classes.read()) }
                }
            }
        }
    }
}
/// Shared upload-related `Signal`s, passed by value (`Signal` is `Copy`, so this
/// is a cheap snapshot of the live handles). Collapses the repeated
/// `#[cfg(target_arch = "wasm32")]` signal-declaration duplication: each drag/drop
/// handler owns its own snapshot and mutates the shared signal state.
#[derive(Clone, Copy)]
struct UploadCtx {
    file_name: Signal<String>,
    status: Signal<String>,
    busy: Signal<bool>,
    analysis: Signal<Option<Analysis>>,
    input_format: Signal<Option<LipidFormat>>,
}

/// Shared WASM/native branch previously inlined in both `file_change` and
/// `on_drop`; collapses the duplicated `#[cfg(target_arch = "wasm32")]` block.
/// `ctx` is taken by value (`Signal`s are `Copy` handles, so `.set` mutates the
/// shared state the snapshot points at).
/// Resolves a dropped or browsed file: extracts its blob and delegates to the
/// wasm/native `process_file_upload` branch, reporting status on empty/failed
/// extraction. Collapses the match previously duplicated in both the `file_change`
/// and `drop` handlers.
fn handle_uploaded_files(
    mut ctx: UploadCtx,
    rule_library: &LipidRuleLibrary,
    result: Result<Option<upload::ExtractedFile>, String>,
    drag_active: Signal<bool>,
) {
    match result {
        Ok(Some(file)) => {
            let detected_format = LipidFormat::from_path(&file.name);
            process_file_upload(ctx, rule_library, file, detected_format, drag_active);
        }
        Ok(None) => ctx.status.set("No file selected.".to_string()),
        Err(msg) => ctx.status.set(msg),
    }
}

#[cfg(target_arch = "wasm32")]
fn process_file_upload(
    mut ctx: UploadCtx,
    rule_library: &LipidRuleLibrary,
    file: upload::ExtractedFile,
    format: Option<LipidFormat>,
    drag_active: Signal<bool>,
) {
    ctx.input_format.set(format);
    begin_analysis_from_blob(
        file.blob,
        file.name,
        ctx.file_name,
        ctx.status,
        ctx.busy,
        drag_active,
        ctx.analysis,
        format,
        rule_library.clone(),
    );
}

#[cfg(not(target_arch = "wasm32"))]
fn process_file_upload(
    mut ctx: UploadCtx,
    _rule_library: &LipidRuleLibrary,
    file: upload::ExtractedFile,
    format: Option<LipidFormat>,
    _drag_active: Signal<bool>,
) {
    ctx.input_format.set(format);
    ctx.file_name.set(file.name);
    ctx.status
        .set("This app runs in the browser — open it via `dx serve`.".to_string());
    ctx.busy.set(false);
}

/// Renders the "Available Lipid Classes" card, sorted by priority.
fn lipid_classes_card(rule_library: &LipidRuleLibrary) -> Element {
    rsx! {
                div {
                    style: StyleBuilder::new().property("background", "rgba(255,255,255,0.9)").border("1px solid rgba(148,163,184,0.22)").border_radius("20px").box_shadow("0 12px 40px rgba(15, 23, 42, 0.08)").padding("1.25rem").property("margin-bottom", "1.25rem").build(),
                    h2 { style: StyleBuilder::new().margin("0 0 0.75rem").font_size("1.1rem").build(), "Available Lipid Classes" }
                    div {
                        style: StyleBuilder::new().display("grid").property("grid-template-columns", "repeat(auto-fit, minmax(250px, 1fr))").gap("0.75rem").property("max-height", "300px").property("overflow-y", "auto").build(),
                        for rule in rule_library.sorted_by_priority() {
                            div {
                                style: StyleBuilder::new().padding("0.6rem 0.8rem").property("background", "#f1f5f9").border("1px solid #e2e8f0").border_radius("8px").font_size("0.85rem").build(),
                                strong { "{rule.name}" }
                                if !rule.description.is_empty() {
                                    span { style: StyleBuilder::new().color("#64748b").font_size("0.8rem").property("margin-left", "0.3rem").build(), " - {rule.description}" }
                                }
                                span { style: StyleBuilder::new().color("#94a3b8").font_size("0.75rem").property("margin-left", "0.5rem").build(), "({rule.family})" }
                            }
                        }
                    }
                }
    }
}

/// Renders the per-class summary panel.
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

fn family_entry(
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
                                let charge_text = item.charge.as_deref().unwrap_or("—");
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
                                                    " · charge "
                                                    strong { style: StyleBuilder::new().color("#0f172a").build(), "{charge_text}" }
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
