// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Virtualized results table body and WASM scroll scheduling glue.

use super::row_cells::{PreparedRow, ResultsRowsWindow, row_text};
#[cfg(target_arch = "wasm32")]
use super::scroll_runtime;
use super::table_header::TableHeader;
use super::{
    ROW_HEIGHT_PX_COMFORTABLE, TABLE_SCROLL_ID, TABLE_VIEWPORT_FALLBACK_PX, VIRTUAL_OVERSCAN_ROWS,
};
use crate::features::explore::interactions::use_explore_interactions;
use crate::hooks::use_virtualization::{self, VirtualizationConfig};
use crate::i18n::{TextKey, t};
use crate::models::{Rows, SortState};
use dioxus::prelude::*;
use std::sync::Arc;

#[component]
pub(super) fn VirtualizedResultsTable(
    entries: Memo<Rows>,
    prepared_rows: Memo<Arc<[PreparedRow]>>,
    sort_state: Memo<SortState>,
    sorted_indices: Memo<Arc<[u32]>>,
) -> Element {
    let locale = crate::hooks::use_locale();
    let interactions = use_explore_interactions();
    let total = entries.read().len();
    #[cfg_attr(not(target_arch = "wasm32"), allow(unused_mut))]
    let mut row_height_px = use_signal(|| ROW_HEIGHT_PX_COMFORTABLE);
    #[cfg_attr(not(target_arch = "wasm32"), allow(unused_mut, unused_variables))]
    let mut first_visible_row = use_signal(|| 0usize);
    #[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
    let viewport_height_px = use_signal(|| TABLE_VIEWPORT_FALLBACK_PX);
    #[cfg(target_arch = "wasm32")]
    let scroll_host = use_signal(|| None::<web_sys::HtmlElement>);
    #[cfg(target_arch = "wasm32")]
    let scroll_raf_scheduled = use_signal(|| false);
    #[cfg(target_arch = "wasm32")]
    let scroll_raf_cb = use_signal(|| None::<wasm_bindgen::closure::Closure<dyn FnMut(f64)>>);
    #[cfg(target_arch = "wasm32")]
    let scroll_raf_id = use_signal(|| None::<i32>);

    let virtualization_config = VirtualizationConfig {
        row_height_px: *row_height_px.read(),
        overscan_rows: VIRTUAL_OVERSCAN_ROWS,
        viewport_fallback_px: TABLE_VIEWPORT_FALLBACK_PX,
        scroll_id: TABLE_SCROLL_ID,
    };

    #[cfg(target_arch = "wasm32")]
    let virtualization = use_virtualization::use_virtualization(
        virtualization_config,
        total,
        *first_visible_row.read(),
        *viewport_height_px.read(),
    );

    #[cfg(not(target_arch = "wasm32"))]
    let virtualization = use_virtualization::use_virtualization(
        virtualization_config,
        total,
        0,
        total
            .saturating_mul(*row_height_px.read())
            .max(TABLE_VIEWPORT_FALLBACK_PX),
    );
    let text = row_text(locale);
    let current_sort = *sort_state.read();
    let rows = entries.read().clone();
    let prepared = prepared_rows.read().clone();
    let order = sorted_indices.read().clone();

    #[cfg(target_arch = "wasm32")]
    use_effect(move || {
        if total == 0 {
            if *first_visible_row.read() != 0 {
                first_visible_row.set(0);
            }
            return;
        }

        let current_row_height = *row_height_px.read();
        let measured_row_height =
            scroll_runtime::measure_row_height_px(TABLE_SCROLL_ID, current_row_height);
        if measured_row_height != current_row_height {
            row_height_px.set(measured_row_height);
        }

        scroll_runtime::schedule_virtual_scroll_frame(
            scroll_host,
            scroll_raf_scheduled,
            scroll_raf_cb,
            scroll_raf_id,
            TABLE_SCROLL_ID,
            measured_row_height,
            total,
            first_visible_row,
            viewport_height_px,
        );
    });

    let on_scroll = move |_| {
        #[cfg(target_arch = "wasm32")]
        {
            scroll_runtime::schedule_virtual_scroll_frame(
                scroll_host,
                scroll_raf_scheduled,
                scroll_raf_cb,
                scroll_raf_id,
                TABLE_SCROLL_ID,
                *row_height_px.read(),
                total,
                first_visible_row,
                viewport_height_px,
            );
        }
    };

    rsx! {
        div {
            id: virtualization_config.scroll_id,
            class: "table-scroll",
            role: "region",
            tabindex: "0",
            aria_label: "{t(locale, TextKey::TableTriplesAria)}",
            onscroll: on_scroll,
            table {
                class: "results-table",
                aria_label: "{t(locale, TextKey::TableTriplesAria)}",
                caption { class: "sr-only", "{t(locale, TextKey::TableTriplesAria)}" }
                colgroup {
                    col { class: "col-structure" }
                    col { class: "col-compound" }
                    col { class: "col-mass" }
                    col { class: "col-formula" }
                    col { class: "col-taxon" }
                    col { class: "col-reference" }
                    col { class: "col-year" }
                }
                thead {
                    TableHeader {
                        current_sort,
                        on_sort_toggle: move |col| interactions.toggle_sort(col),
                    }
                }
                tbody {
                    if virtualization.top_spacer_px > 0 {
                        tr { class: "virtual-spacer-row", aria_hidden: "true",
                            td {
                                class: "virtual-spacer-cell",
                                colspan: "7",
                                style: "height: {virtualization.top_spacer_px}px;",
                            }
                        }
                    }
                    {
                        {
                            rsx! {
                                ResultsRowsWindow {
                                    locale,
                                    text,
                                    rows,
                                    prepared_rows: prepared,
                                    order,
                                    start_row: virtualization.start_row,
                                    visible_count: virtualization.visible_count(),
                                }
                            }
                        }
                    }
                    if virtualization.bottom_spacer_px > 0 {
                        tr { class: "virtual-spacer-row", aria_hidden: "true",
                            td {
                                class: "virtual-spacer-cell",
                                colspan: "7",
                                style: "height: {virtualization.bottom_spacer_px}px;",
                            }
                        }
                    }
                }
            }
        }
    }
}
