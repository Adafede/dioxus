//! `json-count-rs` — count non-null JSON fields from an uploaded file.
//!
//! Drag-and-drop (or browse) a JSON file and see a live count of non-null
//! values per key, rendered as a sortable table.
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
//! dx build --release --platform web --package json-count-rs
//! ```

use dioxus::events::{DragData, FormData};
use dioxus::html::HasFileData;
use dioxus::prelude::*;
use ui::prelude::*;

mod processing;

#[cfg(target_arch = "wasm32")]
use processing::begin_scan_from_blob;
fn main() {
    launch(app);
}

#[derive(Clone, PartialEq, Eq)]
struct ColumnResult {
    key: String,
    count: u64,
}

#[component]
fn app() -> Element {
    ui::shared_signal!(file_name, String::new);

    let results = use_signal(Vec::<ColumnResult>::new);
    let mut status = use_signal(|| "Choose a JSON file to begin.".to_string());
    let busy = use_signal(|| false);
    let drag_active = use_signal(|| false);

    let on_file_change = move |evt: Event<FormData>| match upload::extract_blob_from_file_data(
        &evt.data().files(),
    ) {
        Ok(Some(file)) => {
            #[cfg(target_arch = "wasm32")]
            begin_scan_from_blob(
                file.blob,
                file.name,
                file_name,
                status,
                results,
                busy,
                drag_active,
            );

            #[cfg(not(target_arch = "wasm32"))]
            {
                file_name.set(file.name);
                status.set("This app needs to run in a browser.".to_string());
            }
        }
        Ok(None) => status.set("No file selected.".to_string()),
        Err(msg) => status.set(msg),
    };

    let on_drop = move |evt: Event<DragData>| match upload::extract_blob_from_file_data(
        &evt.data().files(),
    ) {
        Ok(Some(file)) => {
            #[cfg(target_arch = "wasm32")]
            begin_scan_from_blob(
                file.blob,
                file.name,
                file_name,
                status,
                results,
                busy,
                drag_active,
            );

            #[cfg(not(target_arch = "wasm32"))]
            {
                file_name.set(file.name);
                status.set("This app needs to run in a browser.".to_string());
            }
        }
        Ok(None) => status.set("No file selected.".to_string()),
        Err(msg) => status.set(msg),
    };

    rsx! {
        // Document head — replaces static index.html meta tags & styles
        DocumentHead {
            title: "JSON Non-Null Field Counter".to_string(),
            lang: "en".to_string(),
            theme_colors: Some(("#f6f8fb", "#10141b")),
            scripts: vec!["https://scripts.simpleanalyticscdn.com/latest.js".to_string()],
            inline_style: Some(
                "body{font-family:var(--sans,system-ui, sans);-webkit-text-size-adjust:100%;\
                 -moz-text-size-adjust:100%;text-size-adjust:100%;}\
                 .skip-link:focus{top:0!important;outline:3px solid #0b5cab;outline-offset:2px}"
                    .to_string()
            ),
        }

        div {
            style: StyleBuilder::new()
                .property("min-height", "100vh")
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

                if !results.read().is_empty() {
                    table {
                        "aria-labelledby": "results-heading",
                        style: StyleBuilder::new()
                            .width("100%")
                            .property("border-collapse", "collapse")
                            .margin(&format!("{} 0 0", Spacing::LG))
                            .build(),
                        caption {
                            id: "results-heading",
                            style: StyleBuilder::new()
                                .margin("0 0 0.5rem 0")
                                .font_size(Typography::H2)
                                .font_weight("600")
                                .color(ColorScheme::LIGHT.text)
                                .text_align("left")
                                .build(),
                            "Non-null counts by column"
                        }

                        thead {
                            tr {
                                th {
                                    scope: "col",
                                    style: StyleBuilder::new()
                                        .text_align("left")
                                        .border(&format!("2px solid {}", ColorScheme::LIGHT.border))
                                        .padding(Spacing::SM)
                                        .font_weight("600")
                                        .color(ColorScheme::LIGHT.text)
                                        .build(),
                                    "Column"
                                }
                                th {
                                    scope: "col",
                                    style: StyleBuilder::new()
                                        .text_align("right")
                                        .border(&format!("2px solid {}", ColorScheme::LIGHT.border))
                                        .padding(Spacing::SM)
                                        .font_weight("600")
                                        .color(ColorScheme::LIGHT.text)
                                        .build(),
                                    "Non-null count"
                                }
                            }
                        }

                        tbody {
                            for col in results.read().iter() {
                                tr {
                                    td {
                                        style: StyleBuilder::new()
                                            .padding(Spacing::SM)
                                            .border(&format!("1px solid {}", ColorScheme::LIGHT.border))
                                            .build(),
                                        "{col.key}"
                                    }
                                    td {
                                        style: StyleBuilder::new()
                                            .padding(Spacing::SM)
                                            .border(&format!("1px solid {}", ColorScheme::LIGHT.border))
                                            .text_align("right")
                                            .color(ColorScheme::LIGHT.green)
                                            .font_weight("600")
                                            .build(),
                                        "{col.count}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            style { ".skip-link:focus {{ top: 0 !important; outline: 3px solid #0b5cab; outline-offset: 2px; }}" }
        }
    }
}
