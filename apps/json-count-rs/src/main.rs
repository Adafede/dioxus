// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the json-count-rs project

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
//! dx build --profile wasm-release --platform web --package json-count-rs
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

#[derive(Clone, PartialEq, Eq, Debug)]
struct ColumnResult {
    key: String,
    count: u64,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SortColumn {
    Key,
    Count,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SortDirection {
    Asc,
    Desc,
}

impl SortDirection {
    #[must_use]
    const fn toggled(self) -> Self {
        match self {
            Self::Asc => Self::Desc,
            Self::Desc => Self::Asc,
        }
    }

    #[must_use]
    const fn aria_sort(self) -> &'static str {
        match self {
            Self::Asc => "ascending",
            Self::Desc => "descending",
        }
    }

    #[must_use]
    const fn indicator(self) -> &'static str {
        match self {
            Self::Asc => " ▲",
            Self::Desc => " ▼",
        }
    }
}

#[must_use]
const fn next_sort_state(
    current_col: Option<SortColumn>,
    current_dir: SortDirection,
    clicked: SortColumn,
) -> (SortColumn, SortDirection) {
    match current_col {
        Some(col) if col as u8 == clicked as u8 => (clicked, current_dir.toggled()),
        _ => {
            let default_dir = match clicked {
                SortColumn::Key => SortDirection::Asc,
                SortColumn::Count => SortDirection::Desc,
            };
            (clicked, default_dir)
        }
    }
}

#[must_use]
fn aria_sort_for(
    active_col: Option<SortColumn>,
    active_dir: SortDirection,
    col: SortColumn,
) -> &'static str {
    if active_col == Some(col) {
        active_dir.aria_sort()
    } else {
        "none"
    }
}

#[must_use]
fn sorted_results(
    rows: &[ColumnResult],
    sort_col: Option<SortColumn>,
    sort_dir: SortDirection,
) -> Vec<ColumnResult> {
    let mut out = rows.to_vec();
    let Some(col) = sort_col else {
        return out;
    };
    out.sort_by(|a, b| {
        let ord = match col {
            SortColumn::Key => a.key.cmp(&b.key),
            SortColumn::Count => a.count.cmp(&b.count).then_with(|| a.key.cmp(&b.key)),
        };
        match sort_dir {
            SortDirection::Asc => ord,
            SortDirection::Desc => ord.reverse(),
        }
    });
    out
}

/// The signals the file-accepting handlers write to.
///
/// `Signals` are `Copy` handles, so taking this by value and calling `.set` on
/// the fields mutates the same state the component reads.
///
/// The three scanning signals are `wasm32`-only because the branch that reads
/// them is: on native there is no scan to run, only the "needs a browser"
/// status, and an unread field is a warning under `-D warnings`.
#[derive(Clone, Copy)]
struct JsonCountCtx {
    file_name: Signal<String>,
    status: Signal<String>,
    #[cfg(target_arch = "wasm32")]
    results: Signal<Vec<ColumnResult>>,
    #[cfg(target_arch = "wasm32")]
    busy: Signal<bool>,
    #[cfg(target_arch = "wasm32")]
    drag_active: Signal<bool>,
}

/// Acts on an already-extracted file, reporting status when extraction
/// produced nothing.
///
/// Shared by the file-picker and drag-drop handlers, which are identical
/// apart from the event type they receive.
fn handle_uploaded_files(
    mut ctx: JsonCountCtx,
    result: Result<Option<upload::ExtractedFile>, String>,
) {
    match result {
        Ok(Some(file)) => {
            #[cfg(target_arch = "wasm32")]
            begin_scan_from_blob(
                file.blob,
                file.name,
                ctx.file_name,
                ctx.status,
                ctx.results,
                ctx.busy,
                ctx.drag_active,
            );

            #[cfg(not(target_arch = "wasm32"))]
            {
                ctx.file_name.set(file.name);
                ctx.status
                    .set("This app needs to run in a browser.".to_string());
            }
        }
        Ok(None) => ctx.status.set("No file selected.".to_string()),
        Err(msg) => ctx.status.set(msg),
    }
}

#[component]
fn app() -> Element {
    ui::shared_signal!(file_name, String::new);

    let results = use_signal(Vec::<ColumnResult>::new);
    let mut sort_col = use_signal(|| None::<SortColumn>);
    let mut sort_dir = use_signal(|| SortDirection::Asc);
    let status = use_signal(|| "Choose a JSON file to begin.".to_string());
    let busy = use_signal(|| false);
    let drag_active = use_signal(|| false);

    // The two handlers below differ only in the event type they take, so the
    // whole match lives in `handle_uploaded_files` once. A change to how a
    // chosen file is scanned then cannot land in the drag path but not the
    // picker path. `Signals` are `Copy` handles, so bundling them is free.
    let ctx = JsonCountCtx {
        file_name,
        status,
        #[cfg(target_arch = "wasm32")]
        results,
        #[cfg(target_arch = "wasm32")]
        busy,
        #[cfg(target_arch = "wasm32")]
        drag_active,
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
        // Document head — replaces static index.html meta tags & styles
        DocumentHead {
            title: "JSON Non-Null Field Counter".to_string(),
            webmcp: Some(WebMcpConfig {
                app_id: "json-count-rs",
                title: "JSON Count-rs",
                description: "Web app for counting non-null JSON fields from uploaded files",
                inputs: &["json_file"],
                outputs: &["field_stats"],
                edam: EdamAnnotations {
                    topics: &[
                        EdamTerm { id: "topic_3071", label: "Data management" },
                        EdamTerm { id: "topic_3366", label: "Data integration and warehousing" },
                    ],
                    operations: &[
                        EdamTerm { id: "operation_3438", label: "Calculating" },
                        EdamTerm { id: "operation_0336", label: "File validation" },
                    ],
                    input_data: &[EdamTerm { id: "data_0006", label: "Data" }],
                    input_formats: &[EdamTerm { id: "format_3464", label: "JSON" }],
                    output_data: &[EdamTerm { id: "data_2048", label: "Report" }],
                    output_formats: &[EdamTerm { id: "format_2331", label: "HTML" }],
                },
            }),
            lang: "en".to_string(),
            theme_colors: Some(("#f6f8fb", "#10141b")),
            scripts: vec!["https://scripts.simpleanalyticscdn.com/latest.js".to_string()],
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
                                    "aria-sort": aria_sort_for(*sort_col.read(), *sort_dir.read(), SortColumn::Key),
                                    style: StyleBuilder::new()
                                        .text_align("left")
                                        .border(&format!("2px solid {}", ColorScheme::LIGHT.border))
                                        .padding(Spacing::SM)
                                        .build(),
                                    button {
                                        r#type: "button",
                                        style: StyleBuilder::new()
                                            .property("background", "transparent")
                                            .border("none")
                                            .padding("0")
                                            .font_weight("600")
                                            .color(ColorScheme::LIGHT.text)
                                            .cursor("pointer")
                                            .build(),
                                        onclick: move |_| {
                                            let (next_col, next_dir) = next_sort_state(
                                                *sort_col.read(),
                                                *sort_dir.read(),
                                                SortColumn::Key,
                                            );
                                            sort_col.set(Some(next_col));
                                            sort_dir.set(next_dir);
                                        },
                                        if *sort_col.read() == Some(SortColumn::Key) {
                                            { format!("Column{}", sort_dir.read().indicator()) }
                                        } else {
                                            "Column"
                                        }
                                    }
                                }
                                th {
                                    scope: "col",
                                    "aria-sort": aria_sort_for(*sort_col.read(), *sort_dir.read(), SortColumn::Count),
                                    style: StyleBuilder::new()
                                        .text_align("right")
                                        .border(&format!("2px solid {}", ColorScheme::LIGHT.border))
                                        .padding(Spacing::SM)
                                        .build(),
                                    button {
                                        r#type: "button",
                                        style: StyleBuilder::new()
                                            .property("background", "transparent")
                                            .border("none")
                                            .padding("0")
                                            .font_weight("600")
                                            .color(ColorScheme::LIGHT.text)
                                            .cursor("pointer")
                                            .build(),
                                        onclick: move |_| {
                                            let (next_col, next_dir) = next_sort_state(
                                                *sort_col.read(),
                                                *sort_dir.read(),
                                                SortColumn::Count,
                                            );
                                            sort_col.set(Some(next_col));
                                            sort_dir.set(next_dir);
                                        },
                                        if *sort_col.read() == Some(SortColumn::Count) {
                                            { format!("Non-null count{}", sort_dir.read().indicator()) }
                                        } else {
                                            "Non-null count"
                                        }
                                    }
                                }
                            }
                        }

                        tbody {
                            for col in sorted_results(&results.read(), *sort_col.read(), *sort_dir.read()) {
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

        }
        SimpleAnalyticsBadge { domain: "adafede.github.io".to_string() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sort_state_transitions_and_aria_labels() {
        assert_eq!(
            aria_sort_for(None, SortDirection::Asc, SortColumn::Key),
            "none"
        );
        let (col, dir) = next_sort_state(None, SortDirection::Asc, SortColumn::Key);
        assert_eq!((col, dir), (SortColumn::Key, SortDirection::Asc));
        assert_eq!(aria_sort_for(Some(col), dir, SortColumn::Key), "ascending");
        assert_eq!(aria_sort_for(Some(col), dir, SortColumn::Count), "none");
        assert_eq!(dir.indicator(), " ▲");

        let (col2, dir2) = next_sort_state(Some(col), dir, SortColumn::Key);
        assert_eq!((col2, dir2), (SortColumn::Key, SortDirection::Desc));
        assert_eq!(
            aria_sort_for(Some(col2), dir2, SortColumn::Key),
            "descending"
        );
        assert_eq!(dir2.indicator(), " ▼");

        let (col3, dir3) = next_sort_state(Some(col2), dir2, SortColumn::Count);
        assert_eq!((col3, dir3), (SortColumn::Count, SortDirection::Desc));
        let (col4, dir4) = next_sort_state(Some(col3), dir3, SortColumn::Count);
        assert_eq!((col4, dir4), (SortColumn::Count, SortDirection::Asc));
    }

    #[test]
    fn sorted_results_orders_by_selected_column_and_direction() {
        let rows = vec![
            ColumnResult {
                key: "zeta".to_string(),
                count: 2,
            },
            ColumnResult {
                key: "alpha".to_string(),
                count: 10,
            },
            ColumnResult {
                key: "beta".to_string(),
                count: 2,
            },
        ];
        assert_eq!(sorted_results(&rows, None, SortDirection::Asc), rows);

        let key_asc = sorted_results(&rows, Some(SortColumn::Key), SortDirection::Asc);
        assert_eq!(
            key_asc.iter().map(|r| r.key.as_str()).collect::<Vec<_>>(),
            vec!["alpha", "beta", "zeta"]
        );

        let key_desc = sorted_results(&rows, Some(SortColumn::Key), SortDirection::Desc);
        assert_eq!(
            key_desc.iter().map(|r| r.key.as_str()).collect::<Vec<_>>(),
            vec!["zeta", "beta", "alpha"]
        );

        let count_desc = sorted_results(&rows, Some(SortColumn::Count), SortDirection::Desc);
        assert_eq!(
            count_desc
                .iter()
                .map(|r| (r.key.as_str(), r.count))
                .collect::<Vec<_>>(),
            vec![("alpha", 10), ("zeta", 2), ("beta", 2)]
        );

        let count_asc = sorted_results(&rows, Some(SortColumn::Count), SortDirection::Asc);
        assert_eq!(
            count_asc
                .iter()
                .map(|r| (r.key.as_str(), r.count))
                .collect::<Vec<_>>(),
            vec![("beta", 2), ("zeta", 2), ("alpha", 10)]
        );
    }
}
