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

use crate::chemical_class::lmsd_all;
use crate::format::LipidFormat;
use crate::parser::Analysis;
use crate::rules::LipidRuleLibrary;

mod browser;
mod components;

#[cfg(target_arch = "wasm32")]
use self::browser::begin_analysis_from_blob;
use self::components::{gallery_with_filter, lipid_classes_card, summary};

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

    // Initialize selected_classes with all LMSD class names (ensures all classes are available)
    let all_class_names: Vec<_> = lmsd_all().iter().map(|c| c.name.clone()).collect();

    // Start with all classes selected for initial load
    let selected_classes = use_signal(|| all_class_names.clone());

    // Shared filter signals for m/z, precursor, and adduct
    let mz_min = use_signal(|| 50.0f64);
    let mz_max = use_signal(|| 1500.0f64);
    let precursor_min = use_signal(|| 0.0f64);
    let precursor_max = use_signal(|| 1000.0f64);
    let adduct_filter = use_signal(|| String::new());

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

                { lipid_classes_card() }
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
                        hint: ".mgf or .smi files (SMILES annotation recommended for classification)",
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
                    { summary(&analysis.summary, selected_classes, &analysis.all_classes, &analysis.filtered_mgf, &analysis.gallery, &analysis.blocks, ctx.input_format.read().clone().unwrap_or(crate::format::LipidFormat::Mgf), mz_min, mz_max, precursor_min, precursor_max, adduct_filter) }
                    { gallery_with_filter(&analysis.gallery, &selected_classes.read(), *mz_min.read(), *mz_max.read(), *precursor_min.read(), *precursor_max.read(), &adduct_filter.read()) }
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
