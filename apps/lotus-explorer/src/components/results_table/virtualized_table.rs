// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Virtualized results table body and WASM scroll scheduling glue.

use super::row_cells::{ResultsRowsWindow, row_text};
use super::table_header::TableHeader;
use super::{ROW_HEIGHT_PX_COMFORTABLE, TABLE_SCROLL_ID, TABLE_VIEWPORT_FALLBACK_PX, VIRTUAL_OVERSCAN_ROWS};
use crate::features::explore::search_state::ExploreState;
use crate::hooks::use_virtualization::{self, VirtualizationConfig};
use crate::i18n::{TextKey, t};
use dioxus::prelude::*;
use std::sync::Arc;

#[component]
pub(super) fn VirtualizedResultsTable(
    explore: Signal<ExploreState>,
    sorted_indices: Memo<Arc<[u32]>>,
) -> Element {
    let locale = crate::hooks::use_locale();
    let virtualization_config = VirtualizationConfig {
        row_height_px: ROW_HEIGHT_PX_COMFORTABLE,
        overscan_rows: VIRTUAL_OVERSCAN_ROWS,
        viewport_fallback_px: TABLE_VIEWPORT_FALLBACK_PX,
        scroll_id: TABLE_SCROLL_ID,
    };
    let first_visible_row = use_signal(|| 0usize);
    let viewport_height_px = use_signal(|| TABLE_VIEWPORT_FALLBACK_PX);

    #[cfg(target_arch = "wasm32")]
    let scroll_host = use_signal(|| None::<web_sys::HtmlElement>);
    #[cfg(target_arch = "wasm32")]
    let mut scroll_raf_scheduled = use_signal(|| false);
    #[cfg(target_arch = "wasm32")]
    let mut scroll_raf_cb = use_signal(|| None::<wasm_bindgen::closure::Closure<dyn FnMut(f64)>>);
    #[cfg(target_arch = "wasm32")]
    let mut scroll_raf_id = use_signal(|| None::<i32>);

    #[cfg(target_arch = "wasm32")]
    use_drop(move || {
        if let Some(id) = *scroll_raf_id.peek() {
            if let Some(win) = web_sys::window() {
                let _ = win.cancel_animation_frame(id);
            }
        }
        *scroll_raf_id.write() = None;
        *scroll_raf_scheduled.write() = false;
        *scroll_raf_cb.write() = None;
    });

    let total = explore.read().result.entries.len();
    let virtualization = use_virtualization::use_virtualization(
        virtualization_config,
        total,
        *first_visible_row.read(),
        *viewport_height_px.read(),
    );
    let text = row_text(locale);

    rsx! {
        div {
            id: virtualization_config.scroll_id,
            class: "table-scroll",
            onscroll: move |_| {
                #[cfg(target_arch = "wasm32")]
                {
                    super::scroll_runtime::schedule_virtual_scroll_frame(
                        scroll_host,
                        scroll_raf_scheduled,
                        scroll_raf_cb,
                        scroll_raf_id,
                        virtualization_config.scroll_id,
                        virtualization_config.row_height_px,
                        total,
                        first_visible_row,
                        viewport_height_px,
                    );
                }
            },
            table {
                class: "results-table",
                aria_label: "{t(locale, TextKey::TableTriplesAria)}",
                caption { class: "sr-only", "{t(locale, TextKey::TableTriplesAria)}" }
                thead {
                    TableHeader { explore }
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
                        let rows = explore.read().result.entries.clone();
                        let order = sorted_indices.read().clone();
                        {
                            rsx! {
                                ResultsRowsWindow {
                                    locale,
                                    text,
                                    rows,
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


