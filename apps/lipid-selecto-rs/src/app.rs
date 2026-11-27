// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Dioxus UI for `lipid-selecto-rs`: upload, class filter controls, and gallery.

#![expect(
    clippy::empty_enums,
    reason = "Dioxus Props derive macro generates an empty helper enum"
)]

#[cfg(target_arch = "wasm32")]
mod browser;
mod classes_card;
mod download;
mod filter_controls;
mod gallery_panel;
mod summary_panel;
mod types;

use dioxus::events::{DragData, FormData};
use dioxus::html::HasFileData;
use dioxus::prelude::*;
use lipid_core::{Analysis, ClassMatchSet, LipidFormat, lmsd_classes};
use ui::{
    ColorScheme, DocumentHead, EdamAnnotations, EdamTerm, SimpleAnalyticsBadge, StyleBuilder,
    Typography, UploadZone, WebMcpConfig, skip_link,
};

use self::classes_card::LipidClassesCard;
use self::gallery_panel::GalleryPanel;
use self::summary_panel::SummaryPanel;
use self::types::SummaryFilters;

const WEBMCP_CONFIG: WebMcpConfig = WebMcpConfig {
    app_id: "lipid-selecto-rs",
    title: "Lipid Selecto-rs",
    description: "Web app for filtering lipid mass spectrometry data by chemical class with interactive structure visualization",
    inputs: &["lipid_file", "smarts_query"],
    outputs: &["filtered_lipids", "structures"],
    edam: EdamAnnotations {
        topics: &[
            EdamTerm {
                id: "topic_0153",
                label: "Lipids",
            },
            EdamTerm {
                id: "topic_2258",
                label: "Cheminformatics",
            },
            EdamTerm {
                id: "topic_3172",
                label: "Metabolomics",
            },
        ],
        operations: &[
            EdamTerm {
                id: "operation_3695",
                label: "Filtering",
            },
            EdamTerm {
                id: "operation_2990",
                label: "Classification",
            },
            EdamTerm {
                id: "operation_0570",
                label: "Structure visualisation",
            },
        ],
        input_data: &[
            EdamTerm {
                id: "data_0943",
                label: "Mass spectrometry spectra",
            },
            EdamTerm {
                id: "data_2301",
                label: "SMILES string",
            },
        ],
        input_formats: &[
            EdamTerm {
                id: "format_3651",
                label: "MGF",
            },
            EdamTerm {
                id: "format_1196",
                label: "SMILES",
            },
        ],
        output_data: &[
            EdamTerm {
                id: "data_0943",
                label: "Mass spectrometry spectra",
            },
            EdamTerm {
                id: "data_2301",
                label: "SMILES string",
            },
        ],
        output_formats: &[
            EdamTerm {
                id: "format_3651",
                label: "MGF",
            },
            EdamTerm {
                id: "format_1196",
                label: "SMILES",
            },
        ],
    },
};

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
    let generation = upload::use_upload_gen();
    let filters = SummaryFilters {
        selected_classes: use_signal(|| ClassMatchSet::all(lmsd_classes().len())),
        mz_min: use_signal(|| 0.0f64),
        mz_max: use_signal(|| 5000.0f64),
        precursor_min: use_signal(|| 0.0f64),
        precursor_max: use_signal(|| 5000.0f64),
        adduct_filter: use_signal(String::new),
    };
    let ctx = UploadCtx {
        file_name,
        status,
        busy,
        drag_active,
        analysis,
        input_format,
        adduct_filter: filters.adduct_filter,
        generation,
    };

    let on_file_change = move |evt: Event<FormData>| {
        handle_uploaded_files(ctx, &evt.data().files());
    };
    let on_drop = move |evt: Event<DragData>| {
        handle_uploaded_files(ctx, &evt.data().files());
    };
    let light = ColorScheme::LIGHT;
    let active_format = (*ctx.input_format.read()).unwrap_or(LipidFormat::Mgf);

    rsx! {
        DocumentHead {
            title: "Lipid Selecto-rs".to_string(),
            webmcp: Some(WEBMCP_CONFIG),
            lang: "en".to_string(),
            theme_colors: Some((light.bg, ColorScheme::DARK.bg)),
        }
        div {
            style: StyleBuilder::new().min_height("100vh").padding("2rem 1rem 3rem").property("background", "linear-gradient(135deg, #f8fafc 0%, #eef2ff 100%)").color(light.text).property("font-family", "ui-system, system-ui, sans-serif").build(),
            skip_link {}
            main { id: "main-content", style: StyleBuilder::new().property("max-width", "1100px").margin("0 auto").build(),
                h1 { style: StyleBuilder::new().margin("0 0 0.35rem").font_size("1.8rem").property("letter-spacing", "-0.02em").build(), "Lipid Selecto-rs" }
                p { style: StyleBuilder::new().margin("0 0 1.25rem").color(light.text2).font_size(Typography::BODY).property("max-width", "60rem").build(),
                    "Drop an MGF or SMILES file and we'll filter it to keep only lipids matching extensible LIPID MAPS-aligned rules. Download as the same format you uploaded."
                }
                LipidClassesCard {}
                { upload_panel(ctx, on_file_change, on_drop) }
                SummaryPanel { analysis, filters, input_format: active_format, status }
                GalleryPanel { analysis, filters }
            }
        }
        SimpleAnalyticsBadge {
            domain: "adafede.github.io".to_string(),
            path: "/dioxus/lipid-selecto-rs/".to_string(),
        }
    }
}

fn upload_panel(
    ctx: UploadCtx,
    on_file_change: impl FnMut(Event<FormData>) + 'static,
    on_drop: impl FnMut(Event<DragData>) + 'static,
) -> Element {
    rsx! {
        div { style: StyleBuilder::new().property("background", "rgba(255,255,255,0.9)").border("1px solid rgba(148,163,184,0.22)").border_radius("20px").box_shadow("0 12px 40px rgba(15, 23, 42, 0.08)").padding("1.25rem").build(),
            UploadZone {
                file_name: ctx.file_name,
                status: ctx.status,
                busy: ctx.busy,
                drag_active: ctx.drag_active,
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
                    browser::load_example_dataset(ctx);
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        let mut status = ctx.status;
                        status.set("This app runs in the browser — open it via `dx serve`.".to_string());
                    }
                },
                "Load Example SMILES"
            }
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct UploadCtx {
    pub(super) file_name: Signal<String>,
    pub(super) status: Signal<String>,
    pub(super) busy: Signal<bool>,
    pub(super) drag_active: Signal<bool>,
    pub(super) analysis: Signal<Option<Analysis>>,
    pub(super) input_format: Signal<Option<LipidFormat>>,
    pub(super) adduct_filter: Signal<String>,
    pub(super) generation: upload::UploadGen,
}

fn handle_uploaded_files(ctx: UploadCtx, files: &[dioxus::html::FileData]) {
    upload::dispatch_file_upload(files, ctx.status, |file| {
        let detected_format = LipidFormat::from_path(&file.name);
        process_file_upload(ctx, file, detected_format);
    });
}

#[cfg(target_arch = "wasm32")]
fn process_file_upload(
    mut ctx: UploadCtx,
    file: upload::ExtractedFile,
    format: Option<LipidFormat>,
) {
    ctx.input_format.set(format);
    browser::start_analysis(file.blob, file.name, ctx, format);
}

#[cfg(not(target_arch = "wasm32"))]
fn process_file_upload(
    mut ctx: UploadCtx,
    file: upload::ExtractedFile,
    format: Option<LipidFormat>,
) {
    ctx.generation.advance();
    ctx.adduct_filter.set(String::new());
    ctx.input_format.set(format);
    ctx.file_name.set(file.name);
    ctx.analysis.set(None);
    ctx.status
        .set("This app runs in the browser — open it via `dx serve`.".to_string());
    ctx.busy.set(false);
}
