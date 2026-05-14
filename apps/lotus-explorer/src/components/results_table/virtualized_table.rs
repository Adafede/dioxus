// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Virtualized results table body and WASM scroll scheduling glue.

use super::row_cells::{ResultsRowsWindow, row_text};
use super::table_header::TableHeader;
use super::{
    ROW_HEIGHT_PX_COMFORTABLE, TABLE_SCROLL_ID, TABLE_VIEWPORT_FALLBACK_PX, VIRTUAL_OVERSCAN_ROWS,
};
use crate::features::explore::actions::ExploreAction;
use crate::features::explore::search_state::ExploreState;
use crate::features::explore::search_state::dispatch_explore_action;
use crate::hooks::use_virtualization::{self, VirtualizationConfig};
use crate::i18n::{TextKey, t};
use crate::models::{Rows, SortState};
use dioxus::prelude::*;
use std::sync::Arc;

#[component]
pub(super) fn VirtualizedResultsTable(
    explore: Signal<ExploreState>,
    entries: Memo<Rows>,
    sort_state: Memo<SortState>,
    sorted_indices: Memo<Arc<[u32]>>,
) -> Element {
    let locale = crate::hooks::use_locale();
    let total = entries.read().len();
    #[cfg_attr(not(target_arch = "wasm32"), allow(unused_mut))]
    let mut row_height_px = use_signal(|| ROW_HEIGHT_PX_COMFORTABLE);
    let virtualization_config = VirtualizationConfig {
        row_height_px: *row_height_px.read(),
        overscan_rows: VIRTUAL_OVERSCAN_ROWS,
        viewport_fallback_px: TABLE_VIEWPORT_FALLBACK_PX,
        scroll_id: TABLE_SCROLL_ID,
    };
    // Keep all rows visible (results are already capped upstream for wasm).
    let full_viewport_px = total
        .saturating_mul(*row_height_px.read())
        .max(TABLE_VIEWPORT_FALLBACK_PX);

    let virtualization =
        use_virtualization::use_virtualization(virtualization_config, total, 0, full_viewport_px);
    let text = row_text(locale);
    let current_sort = *sort_state.read();

    rsx! {
        div { id: virtualization_config.scroll_id, class: "table-scroll",
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
                        on_sort_toggle: move |col| {
                            dispatch_explore_action(explore, ExploreAction::SortToggled(col));
                        },
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
                        let rows = entries.read().clone();
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
