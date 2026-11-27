// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Root Dioxus component and shared signal state for `mgf-precursor-erro-rs`.

#![expect(
    clippy::empty_enums,
    reason = "Dioxus component macro generates a Props helper empty enum"
)]

use dioxus::events::{DragData, FormData};
use dioxus::html::HasFileData;
use dioxus::prelude::*;
use mgf_analysis_core::{CalibrationModel, ErrorUnit, PrecursorStats, ShrinkageLambda};
use ui::{
    DocumentHead, EdamAnnotations, EdamTerm, SimpleAnalyticsBadge, StyleBuilder, UploadZone,
    WebMcpConfig, skip_link,
};

mod browser;
mod diagnostics_panel;
mod plots;
mod recalibration_panel;
mod results;
mod warnings_panel;

use self::results::ResultsPanel;

const WEBMCP_CONFIG: WebMcpConfig = WebMcpConfig {
    app_id: "mgf-precursor-erro-rs",
    title: "MGF Precursor Erro-rs",
    description: "Web app for summarizing precursor mass errors from uploaded MGF files",
    inputs: &["mgf_file"],
    outputs: &["precursor_error_stats"],
    edam: EdamAnnotations {
        topics: &[
            EdamTerm {
                id: "topic_3520",
                label: "Proteomics experiment",
            },
            EdamTerm {
                id: "topic_3172",
                label: "Metabolomics",
            },
        ],
        operations: &[
            EdamTerm {
                id: "operation_3627",
                label: "Mass spectra calibration",
            },
            EdamTerm {
                id: "operation_2428",
                label: "Validation",
            },
            EdamTerm {
                id: "operation_0337",
                label: "Visualisation",
            },
        ],
        input_data: &[EdamTerm {
            id: "data_0943",
            label: "Mass spectrum",
        }],
        input_formats: &[EdamTerm {
            id: "format_3651",
            label: "MGF",
        }],
        output_data: &[
            EdamTerm {
                id: "data_0943",
                label: "Mass spectrum",
            },
            EdamTerm {
                id: "data_2884",
                label: "Plot",
            },
        ],
        output_formats: &[
            EdamTerm {
                id: "format_3651",
                label: "MGF",
            },
            EdamTerm {
                id: "format_3604",
                label: "SVG",
            },
        ],
    },
};

/// Grouped reactive handles shared across upload, status, and example-loading paths.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct UploadSignals {
    pub(super) file_name: Signal<String>,
    pub(super) status: Signal<String>,
    pub(super) metrics: Signal<Option<PrecursorStats>>,
    pub(super) busy: Signal<bool>,
    pub(super) drag_active: Signal<bool>,
    pub(super) original_mgf_content: Signal<String>,
    pub(super) analysis_generation: upload::UploadGen,
}

/// Renders the MGF precursor-error analysis UI.
///
/// # Errors
///
/// Returns a rendering error only if the RSX tree cannot be constructed.
pub fn app() -> Element {
    let signals = UploadSignals {
        file_name: use_signal(String::new),
        status: use_signal(|| "Drop an MGF file to begin.".to_string()),
        metrics: use_signal(|| None::<PrecursorStats>),
        busy: use_signal(|| false),
        drag_active: use_signal(|| false),
        original_mgf_content: use_signal(String::new),
        analysis_generation: upload::use_upload_gen(),
    };
    let calibration_model = use_signal(|| CalibrationModel::None);
    let preferred_lambda = use_signal(ShrinkageLambda::default);
    let cumulative_dist_tab = use_signal(|| ErrorUnit::Mda);

    let on_file_change = move |evt: Event<FormData>| {
        browser::attempt_analysis_from_files(&evt.data().files(), signals);
    };
    let on_drop = move |evt: Event<DragData>| {
        browser::attempt_analysis_from_files(&evt.data().files(), signals);
    };

    rsx! {
        DocumentHead {
            title: "MGF Precursor Error".to_string(),
            webmcp: Some(WEBMCP_CONFIG),
            lang: "en".to_string(),
            theme_colors: Some(("#f6f8fb", "#10141b")),
        }

        div {
            style: StyleBuilder::new().min_height("100vh").padding("2rem 1rem 3rem").property("background", "linear-gradient(135deg, #f8fafc 0%, #eef2ff 100%)").color("#0f172a").build(),
            skip_link {}

            main { id: "main-content",
                style: StyleBuilder::new().property("max-width", "960px").margin("0 auto").build(),
                h1 { style: StyleBuilder::new().margin("0 0 0.35rem").font_size("1.7rem").property("letter-spacing", "-0.02em").build(), "MGF Precursor Error" }
                p {
                    style: StyleBuilder::new().margin("0 0 1.25rem").color("#475569").font_size("0.95rem").build(),
                    "Upload an MGF file and explore precursor mass errors in Da and ppm."
                }

                div {
                    style: StyleBuilder::new().property("background", "rgba(255,255,255,0.9)").border("1px solid rgba(148,163,184,0.22)").border_radius("20px").box_shadow("0 12px 40px rgba(15, 23, 42, 0.08)").padding("1.25rem").property("backdrop-filter", "blur(12px)").build(),
                    UploadZone {
                        file_name: signals.file_name,
                        status: signals.status,
                        busy: signals.busy,
                        drag_active: signals.drag_active,
                        on_file_change,
                        on_drop,
                        accept: ".mgf",
                        label: "Drop an MGF file here or click to browse",
                        hint: "Accepts .mgf files. Use drag and drop or browse.",
                        icon: "📁",
                    }
                    p {
                        style: StyleBuilder::new().margin("0.5rem 0 0").color("#475569").font_size("0.85rem").build(),
                        "Plots cap at 5 mDa / 10 ppm for the signed-error views"
                    }

                    example_load_button { signals }

                    ResultsPanel {
                        signals,
                        calibration_model,
                        preferred_lambda,
                        cumulative_dist_tab,
                    }
                }
            }
        }
        SimpleAnalyticsBadge {
            domain: "adafede.github.io".to_string(),
            path: "/dioxus/mgf-precursor-erro-rs/".to_string(),
        }
    }
}

#[component]
fn example_load_button(mut signals: UploadSignals) -> Element {
    rsx! {
        if signals.file_name.read().is_empty()
            && signals.metrics.read().is_none()
            && !(*signals.busy.read())
        {
            button {
                r#type: "button",
                style: StyleBuilder::new().property("margin-top", "0.8rem").border("1px solid #2563eb").border_radius("999px").property("background", "#eff6ff").color("#1d4ed8").font_size("0.84rem").font_weight("700").padding("0.45rem 0.8rem").cursor("pointer").build(),
                onclick: move |_| {
                    #[cfg(target_arch = "wasm32")]
                    browser::load_example_mgf(signals);
                    #[cfg(not(target_arch = "wasm32"))]
                    signals
                        .status
                        .set("This app needs to run in a browser.".to_string());
                },
                "Load example MGF"
            }
        }
    }
}
