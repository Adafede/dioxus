// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the json-count-rs project

//! Application signals, upload dispatch, and memoized sortable results table.

#![expect(
    clippy::empty_enums,
    reason = "Dioxus Props derive macro generates an empty helper enum"
)]

use dioxus::html::FileData;
use dioxus::prelude::*;
use json_count_core::{
    ColumnResult, SortColumn, SortSpec, aria_sort_for, next_sort_state, sorted_results,
};
use ui::{ColorScheme, Spacing, StyleBuilder, Typography};
#[cfg(target_arch = "wasm32")]
use upload::UploadGen;

#[cfg(target_arch = "wasm32")]
use crate::wasm_entry::begin_scan_from_blob;

/// Shared reactive state handles passed to file-selection handlers.
#[derive(Clone, Copy)]
pub(crate) struct JsonCountCtx {
    pub(crate) file_name: Signal<String>,
    pub(crate) status: Signal<String>,
    #[cfg(target_arch = "wasm32")]
    pub(crate) results: Signal<Vec<ColumnResult>>,
    #[cfg(target_arch = "wasm32")]
    pub(crate) busy: Signal<bool>,
    #[cfg(target_arch = "wasm32")]
    pub(crate) drag_active: Signal<bool>,
    #[cfg(target_arch = "wasm32")]
    pub(crate) generation: UploadGen,
}

/// Dispatches an extracted browser file (or reports missing/error status).
pub(crate) fn handle_uploaded_files(ctx: JsonCountCtx, files: &[FileData]) {
    upload::dispatch_file_upload(files, ctx.status, |file| {
        #[cfg(target_arch = "wasm32")]
        begin_scan_from_blob(file.blob, file.name, ctx);

        #[cfg(not(target_arch = "wasm32"))]
        upload::report_browser_only_upload(ctx.file_name, ctx.status, file.name);
    });
}

fn header_style(align: &str) -> String {
    StyleBuilder::new()
        .text_align(align)
        .border(&format!("2px solid {}", ColorScheme::LIGHT.border))
        .padding(Spacing::SM)
        .build()
}

fn sort_button_style() -> String {
    StyleBuilder::new()
        .property("background", "transparent")
        .border("none")
        .padding("0")
        .font_weight("600")
        .color(ColorScheme::LIGHT.text)
        .cursor("pointer")
        .build()
}

/// Memoized sortable results table for scanned JSON columns.
#[component]
pub(crate) fn ResultsTable(
    results: Signal<Vec<ColumnResult>>,
    sort_spec: Signal<Option<SortSpec>>,
) -> Element {
    let sorted = use_memo(move || sorted_results(&results.read(), *sort_spec.read()));
    let rows = sorted.read();
    if rows.is_empty() {
        return rsx! {};
    }

    let cell_border = format!("1px solid {}", ColorScheme::LIGHT.border);
    let key_cell_style = StyleBuilder::new()
        .padding(Spacing::SM)
        .border(&cell_border)
        .build();
    let count_cell_style = StyleBuilder::new()
        .padding(Spacing::SM)
        .border(&cell_border)
        .text_align("right")
        .color(ColorScheme::LIGHT.green)
        .font_weight("600")
        .build();
    let active_sort = *sort_spec.read();

    rsx! {
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
                    for (col, label, align) in [
                        (SortColumn::Key, "Column", "left"),
                        (SortColumn::Count, "Non-null count", "right"),
                    ] {
                        th {
                            scope: "col",
                            "aria-sort": aria_sort_for(active_sort, col),
                            style: header_style(align),
                            button {
                                r#type: "button",
                                style: sort_button_style(),
                                onclick: move |_| {
                                    let next = next_sort_state(*sort_spec.read(), col);
                                    sort_spec.set(Some(next));
                                },
                                if let Some(spec) = active_sort.filter(|s| s.column == col) {
                                    { format!("{label}{}", spec.direction.indicator()) }
                                } else {
                                    "{label}"
                                }
                            }
                        }
                    }
                }
            }
            tbody {
                for col in rows.iter() {
                    tr {
                        key: "{col.key}",
                        td { style: "{key_cell_style}", "{col.key}" }
                        td { style: "{count_cell_style}", "{col.count}" }
                    }
                }
            }
        }
    }
}
