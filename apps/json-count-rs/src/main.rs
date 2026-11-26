// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the json-count-rs project

//! `json-count-rs` — count non-null JSON fields from an uploaded file.
//!
//! Drag-and-drop (or browse) a JSON file and see a live count of non-null
//! values per key, rendered as a sortable table backed by `json-count-core`.
//!
//! # Run locally
//!
//! ```bash
//! dx serve --package json-count-rs
//! ```
//!
//! # Build for the website
//!
//! ```bash
//! dx build --profile wasm-release --platform web --package json-count-rs
//! ```

mod app_state;
#[cfg(target_arch = "wasm32")]
mod wasm_entry;

use dioxus::events::{DragData, FormData};
use dioxus::html::HasFileData;
use dioxus::prelude::*;
use json_count_core::{ColumnResult, SortSpec};
use ui::{
    ColorScheme, DocumentHead, EdamAnnotations, EdamTerm, Radius, Shadow, SimpleAnalyticsBadge,
    Spacing, StyleBuilder, Typography, UploadZone, WebMcpConfig, skip_link,
};

use app_state::{JsonCountCtx, ResultsTable, handle_uploaded_files};

fn main() {
    launch(app);
}

const fn page_head() -> WebMcpConfig {
    WebMcpConfig {
        app_id: "json-count-rs",
        title: "JSON Count-rs",
        description: "Web app for counting non-null JSON fields from uploaded files",
        inputs: &["json_file"],
        outputs: &["field_stats"],
        edam: EdamAnnotations {
            topics: &[
                EdamTerm {
                    id: "topic_3071",
                    label: "Data management",
                },
                EdamTerm {
                    id: "topic_3366",
                    label: "Data integration and warehousing",
                },
            ],
            operations: &[
                EdamTerm {
                    id: "operation_2238",
                    label: "Statistical calculation",
                },
                EdamTerm {
                    id: "operation_0336",
                    label: "Format validation",
                },
            ],
            input_data: &[EdamTerm {
                id: "data_0006",
                label: "Data",
            }],
            input_formats: &[EdamTerm {
                id: "format_3464",
                label: "JSON",
            }],
            output_data: &[EdamTerm {
                id: "data_2048",
                label: "Report",
            }],
            output_formats: &[EdamTerm {
                id: "format_2331",
                label: "HTML",
            }],
        },
    }
}

#[component]
fn app() -> Element {
    ui::shared_signal!(file_name, String::new);

    let results = use_signal(Vec::<ColumnResult>::new);
    let sort_spec = use_signal(|| None::<SortSpec>);
    let status = use_signal(|| "Choose a JSON file to begin.".to_string());
    let busy = use_signal(|| false);
    let drag_active = use_signal(|| false);
    #[cfg(target_arch = "wasm32")]
    let generation = use_signal(|| 0u64);

    let ctx = JsonCountCtx {
        file_name,
        status,
        #[cfg(target_arch = "wasm32")]
        results,
        #[cfg(target_arch = "wasm32")]
        busy,
        #[cfg(target_arch = "wasm32")]
        drag_active,
        #[cfg(target_arch = "wasm32")]
        generation,
    };

    let on_file_change = move |evt: Event<FormData>| {
        handle_uploaded_files(
            ctx,
            upload::extract_blob_from_file_data(&evt.data().files()),
        );
    };
    let on_drop = move |evt: Event<DragData>| {
        handle_uploaded_files(
            ctx,
            upload::extract_blob_from_file_data(&evt.data().files()),
        );
    };

    rsx! {
        DocumentHead {
            title: "JSON Non-Null Field Counter".to_string(),
            webmcp: Some(page_head()),
            lang: "en".to_string(),
            theme_colors: Some(("#f6f8fb", "#10141b")),
        }

        div {
            style: StyleBuilder::new()
                .min_height("100vh")
                .padding(&format!("{} {} 3rem", Spacing::XL, Spacing::LG))
                .background_color(ColorScheme::LIGHT.bg)
                .color(ColorScheme::LIGHT.text)
                .font_family(Typography::SANS)
                .build(),

            skip_link {}

            main {
                id: "main-content",
                style: StyleBuilder::new()
                    .property("max-width", "760px")
                    .margin("0 auto")
                    .background_color(ColorScheme::LIGHT.surface)
                    .border(&format!("1px solid {}", ColorScheme::LIGHT.border))
                    .border_radius(Radius::LG)
                    .box_shadow(Shadow::MD)
                    .padding(Spacing::LG)
                    .build(),

                h1 {
                    style: StyleBuilder::new()
                        .margin("0 0 0.35rem 0")
                        .font_size(Typography::H1)
                        .font_weight("600")
                        .color(ColorScheme::LIGHT.text)
                        .build(),
                    "JSON Non-Null Field Counter"
                }

                p {
                    style: StyleBuilder::new()
                        .margin("0 0 1rem 0")
                        .color(ColorScheme::LIGHT.text2)
                        .font_size(Typography::BODY)
                        .line_height(Typography::LINE_HEIGHT)
                        .build(),
                    "Drop a JSON file into the upload area below or browse for it on disk. The scanner streams multi-gigabyte files in the browser while keeping memory bounded."
                }

                p {
                    id: "json-upload-help",
                    style: StyleBuilder::new()
                        .margin("0 0 1rem 0")
                        .color(ColorScheme::LIGHT.text3)
                        .font_size(Typography::LABEL)
                        .line_height(Typography::LINE_HEIGHT)
                        .build(),
                    "Accepts .json files. The upload area is keyboard focusable and supports drag and drop."
                }

                UploadZone {
                    file_name,
                    status,
                    busy,
                    drag_active,
                    on_file_change,
                    on_drop,
                    accept: ".json",
                    label: "Drop JSON file here or click to browse",
                    hint: ".json files only",
                    icon: "📁",
                }

                ResultsTable { results, sort_spec }
            }
        }
        SimpleAnalyticsBadge {
            domain: "adafede.github.io".to_string(),
            path: "/dioxus/json-count-rs/".to_string(),
        }
    }
}
