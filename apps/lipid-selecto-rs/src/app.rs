//! Dioxus UI for `lipid-selecto-rs`: drag-and-drop MGF upload, a lipid summary,
//! a "Download lipid MGF" button, and a gallery of structure diagrams.
//!
//! The `app` entry point intentionally is **not** annotated with `#[component]`
//! so that `dioxus::launch(lipid_selecto_rs::app)` keeps working exactly like the
//! sibling `mgf-precursor-erro-rs` app.

use dioxus::events::{DragData, FormData};
use dioxus::html::HasFileData;
use dioxus::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

use crate::parser::Analysis;
use crate::lipids::LipidClass;

mod browser;

#[cfg(target_arch = "wasm32")]
use self::browser::begin_analysis_from_blob;

/// Build the "download as" filename from the uploaded file's name.
fn download_filename(source: &str) -> String {
    let trimmed = source.trim();
    let stem = trimmed
        .strip_suffix(".mgf")
        .or_else(|| trimmed.strip_suffix(".MGF"))
        .unwrap_or(trimmed);
    let stem = stem.trim();
    if stem.is_empty() {
        "lipids_selected.mgf".to_string()
    } else {
        format!("lipids_{stem}.mgf")
    }
}

/// Renders the lipid selection UI.
///
/// # Errors
///
/// Returns an error if the component tree fails to build or render.
#[allow(clippy::too_many_lines)]
pub fn app() -> Element {
    let mut status = use_signal(|| "Drop an MGF file to begin.".to_string());
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
    
    let selected_classes = use_signal(|| vec![
        LipidClass::FattyAcyl,
        LipidClass::Glycerolipid,
        LipidClass::Glycerophospholipid,
        LipidClass::Sphingolipid,
        LipidClass::Sterol,
        LipidClass::Other,
    ]);

    let on_file_change = move |evt: Event<FormData>| {
        let Some(file) = evt.data().files().into_iter().next() else {
            status.set("No file selected.".to_string());
            return;
        };

        #[cfg(target_arch = "wasm32")]
        let Some(web_file) = file.inner().downcast_ref::<web_sys::File>() else {
            status.set("This file type is not supported in the browser.".to_string());
            return;
        };

        #[cfg(target_arch = "wasm32")]
        let Ok(blob) = web_file.clone().dyn_into::<web_sys::Blob>() else {
            status.set("Unable to read the selected file as a blob.".to_string());
            return;
        };

        #[cfg(target_arch = "wasm32")]
        begin_analysis_from_blob(
            blob,
            file.name(),
            file_name,
            status,
            busy,
            drag_active,
            analysis,
        );

        #[cfg(not(target_arch = "wasm32"))]
        {
            file_name.set(file.name());
            status.set("This app runs in the browser — open it via `dx serve`.".to_string());
            busy.set(false);
        }
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
    let on_drop = move |evt: Event<DragData>| {
        evt.prevent_default();
        drag_active.set(false);
        let Some(file) = evt.data().files().into_iter().next() else {
            status.set("No file selected.".to_string());
            return;
        };

        #[cfg(target_arch = "wasm32")]
        let Some(web_file) = file.inner().downcast_ref::<web_sys::File>() else {
            status.set("This file type is not supported in the browser.".to_string());
            return;
        };

        #[cfg(target_arch = "wasm32")]
        let Ok(blob) = web_file.clone().dyn_into::<web_sys::Blob>() else {
            status.set("Unable to read the selected file as a blob.".to_string());
            return;
        };

        #[cfg(target_arch = "wasm32")]
        begin_analysis_from_blob(
            blob,
            file.name(),
            file_name,
            status,
            busy,
            drag_active,
            analysis,
        );

        #[cfg(not(target_arch = "wasm32"))]
        {
            file_name.set(file.name());
            status.set("This app runs in the browser — open it via `dx serve`.".to_string());
            busy.set(false);
        }
    };

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
        div {
            style: "min-height: 100vh; padding: 2rem 1rem 3rem; background: linear-gradient(135deg, #f8fafc 0%, #eef2ff 100%); color: #0f172a; font-family: ui-system, system-ui, sans-serif;",
            main { id: "main",
                style: "max-width: 1100px; margin: 0 auto;",
                h1 { style: "margin: 0 0 0.35rem; font-size: 1.8rem; letter-spacing: -0.02em;", "Lipid Selecto-rs" }
                p {
                    style: "margin: 0 0 1.25rem; color: #475569; font-size: 0.95rem; max-width: 60rem;",
                    "Drop an MGF file whose spectra carry a SMILES (or formula) and we'll keep only the ones matching a lipid — fatty acyls, glycerolipids, phospholipids, sphingolipids and sterols — and give you a filtered MGF to download, with structure diagrams."
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
                        span { style: "font-size: 1rem;", "Drop an MGF file here or click to browse" }
                        span { style: "font-size: 0.85rem; font-weight: 500; color: #64748b;", ".mgf files containing SMILES=/FORMULA= entries" }
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
                        "Accepts .mgf files. Use drag and drop or browse."
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
                    { self::download_bar(&analysis.filtered_mgf, &file_name.read(), &selected_classes.read(), &analysis.blocks, status) }
                    { self::summary(analysis.summary.clone(), selected_classes) }
                    { self::gallery(&analysis.gallery) }
                }
            }
        }
    }
}

/// Renders the "Download lipid MGF" button.
fn download_bar(
    filtered_mgf: &str,
    source_file: &str,
    selected_classes: &[LipidClass],
    blocks: &[crate::parser::SpectrumBlock],
    mut status: Signal<String>,
) -> Element {
    let download_name = download_filename(source_file);
    
    let filtered_content = if selected_classes.len() < 6 {
        crate::parser::build_filtered_mgf_with_classes(blocks, selected_classes).to_string()
    } else {
        filtered_mgf.to_string()
    };
    
    let name = download_name.clone();
    let empty = filtered_content.is_empty();
    rsx! {
        div {
            style: "margin-top: 1.25rem; padding: 0.9rem 1rem; border: 1px solid #e2e8f0; border-radius: 14px; background: #f8fafc;",
            button {
                r#type: "button",
                disabled: empty,
                style: "border: 1px solid #2563eb; border-radius: 999px; background: #eff6ff; color: #1d4ed8; font-size: 0.86rem; font-weight: 700; padding: 0.5rem 1rem; cursor: pointer;",
                onclick: move |_| {
                    if let Err(error) = browser::download_mgf(&filtered_content, &name) {
                        status.set(error);
                    }
                },
                "Download lipid MGF ({download_name})"
            }
            if !empty {
                p { style: "margin: 0.35rem 0 0; color: #64748b; font-size: 0.8rem;", "Ready to download {filtered_content.len()} bytes." }
            }
        }
    }
}

/// Renders the per-class summary panel.
fn summary(summary_data: crate::parser::Summary, mut selected_classes: Signal<Vec<LipidClass>>) -> Element {
    let all_selected = selected_classes.read().len() == 6;
    let lipid_spectra = summary_data.lipid_spectra;
    let total_spectra = summary_data.total_spectra;
    let skipped = summary_data.skipped;
    let unclassified = summary_data.unclassified;
    
    // Build list of class items as owned data
    let class_items: Vec<_> = summary_data
        .class_counts
        .iter()
        .map(|(class, count)| (*class, *count))
        .collect();
    
    rsx! {
        div {
            style: "margin-top: 1.25rem; padding: 1rem 1.1rem; border: 1px solid #e2e8f0; border-radius: 16px; background: linear-gradient(180deg, #ffffff 0%, #f8fafc 100%);",
            h2 { style: "margin: 0 0 0.5rem; font-size: 1.05rem; color: #0f172a;", "Results" }
            div { style: "display: flex; flex-wrap: wrap; gap: 0.4rem; align-items: center; font-size: 0.9rem;",
                span { style: "color: #16a34a; font-weight: 700;", "{lipid_spectra} lipid spectra selected" }
                span { style: "color: #475569;", "· out of {total_spectra} total" }
                if skipped > 0 {
                    span { style: "color: #94a3b8;", "(skipped {skipped} spectra without SMILES or formula)" }
                }
                if unclassified > 0 {
                    span { style: "color: #94a3b8;", "(ignored {unclassified} annotated non-lipid spectra)" }
                }
            }
            
            if !class_items.is_empty() {
                div { style: "margin: 0.8rem 0 0; padding: 0.6rem; border: 1px solid #e2e8f0; border-radius: 10px; background: #f8fafc;",
                    div { style: "margin-bottom: 0.6rem;",
                        label {
                            style: "display: flex; align-items: center; gap: 0.5rem; cursor: pointer; font-weight: 600; color: #334155;",
                            input {
                                r#type: "checkbox",
                                checked: all_selected,
                                onchange: move |_| {
                                    let classes = if all_selected {
                                        vec![]
                                    } else {
                                        vec![
                                            LipidClass::FattyAcyl,
                                            LipidClass::Glycerolipid,
                                            LipidClass::Glycerophospholipid,
                                            LipidClass::Sphingolipid,
                                            LipidClass::Sterol,
                                            LipidClass::Other,
                                        ]
                                    };
                                    selected_classes.set(classes);
                                },
                                style: "width: 16px; height: 16px; cursor: pointer;",
                            }
                            "All classes"
                        }
                    }
                    ul {
                        style: "margin: 0; padding: 0; list-style: none; display: flex; flex-wrap: wrap; gap: 0.6rem;",
                        for (class, count) in class_items.iter().filter(|(c, _)| selected_classes.read().contains(c)) {
                            {
                                let color = class.color();
                                let class_copy = *class;
                                rsx! {
                                    li {
                                        style: "display: flex; align-items: center; gap: 0.4rem;",
                                        label {
                                            style: "display: flex; align-items: center; gap: 0.4rem; cursor: pointer;",
                                            input {
                                                r#type: "checkbox",
                                                checked: true,
                                                onchange: move |_| {
                                                    let mut classes = selected_classes.read().clone();
                                                    classes.retain(|&c| c != class_copy);
                                                    selected_classes.set(classes);
                                                },
                                                style: "width: 14px; height: 14px; cursor: pointer;",
                                            }
                                            span { style: "display: inline-flex; align-items: center; gap: 0.3rem; padding: 0.2rem 0.5rem; border-radius: 999px; background: #f1f5f9; font-size: 0.8rem;",
                                                span { style: format!("width: 8px; height: 8px; border-radius: 50%; background: {color};"), }
                                                "{class.label()}: {count}"
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

/// Renders the structure-diagram gallery.
fn gallery(gallery: &[crate::parser::GalleryItem]) -> Element {
    let count = gallery.len();
    rsx! {
        div {
            style: "margin-top: 1.25rem;",
            h2 { style: "margin: 0 0 0.5rem; font-size: 1.05rem; color: #0f172a;", "Lipid structures ({count} shown)" }
            if count == 0 {
                p { style: "color: #64748b;", "No lipid spectra were selected." }
            } else {
                div {
                    style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: 0.75rem;",
                    for item in gallery.iter() {
                        {
                            let color = item.class.color();
                            let precursor_text = item
                                .precursor_mz
                                .map_or_else(|| "—".to_string(), |mz| format!("{mz:.3}"));
                            let charge_text = item.charge.as_deref().unwrap_or("—");
                            rsx! {
                                div { style: "background: linear-gradient(180deg, #ffffff 0%, #f8fafc 100%); padding: 0.6rem 0.7rem; border-radius: 14px; border: 1px solid #e2e8f0; box-shadow: 0 6px 16px rgba(15, 23, 42, 0.05); overflow: hidden;",
                                    div { style: "display: flex; gap: 0.55rem; align-items: flex-start;",
                                        div { style: "flex: 0 0 auto; width: 160px; height: 120px; display: grid; place-items: center; background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 10px; overflow: hidden;",
                                            div { style: "width: 100%; height: 100%; display: flex; align-items: center; justify-content: center;",
                                                div { dangerous_inner_html: item.svg.as_str() }
                                            }
                                        }
                                        div { style: "flex: 1 1 auto; min-width: 0;",
                                            div { style: "display: flex; align-items: baseline; gap: 0.45rem; flex-wrap: wrap;",
                                                span { style: format!("color: {color}; font-size: 0.72rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.04em; padding: 0.15rem 0.5rem; border-radius: 999px; background: {color}1a;"), "{item.class.label()}" }
                                                span { style: "color: #0f172a; font-size: 0.82rem; font-weight: 600;", "{item.formula}" }
                                                span { style: "color: #64748b; font-size: 0.78rem;", "m/z {item.exact_mass:.3}" }
                                            }
                                            if let Some(title) = &item.title {
                                                div { style: "margin-top: 0.25rem; color: #334155; font-size: 0.8rem; font-weight: 500; max-width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;", "{title}" }
                                            }
                                            if let Some(smiles) = &item.smiles {
                                                div { style: "margin-top: 0.15rem; color: #64748b; font-size: 0.72rem; font-family: ui-monospace, monospace; max-width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;", "{smiles}" }
                                            }
                                            div { style: "margin-top: 0.2rem; color: #64748b; font-size: 0.75rem;",
                                                "precursor "
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
