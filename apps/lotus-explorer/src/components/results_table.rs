// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::i18n::{TextKey, t};
use crate::models::*;
use crate::state::use_results_context;
use dioxus::prelude::*;
use std::sync::Arc;

#[path = "results_table/row_cells.rs"]
mod row_cells;
use row_cells::{ResultsRowsWindow, row_text};

#[path = "results_table/sort_helpers.rs"]
mod sort_helpers;

#[path = "results_table/table_toolbar_sections.rs"]
mod table_toolbar_sections;
use table_toolbar_sections::{
    CappedRowsNotice, DownloadActionsGroup, QueryPanel, StatBar,
};

#[path = "results_table/table_header.rs"]
mod table_header;
use table_header::TableHeader;

const TABLE_SCROLL_ID: &str = "results-table-scroll";
const VIRTUAL_OVERSCAN_ROWS: usize = 12;
const ROW_HEIGHT_PX_COMFORTABLE: usize = 114;
const TABLE_VIEWPORT_FALLBACK_PX: usize = 640;

/// Renders the full results section.
///
/// Reactive surface is deliberately narrow: this component subscribes only to
/// `entries` (for the empty-state check and sort index) and `locale`.  All
/// query-panel / stats / download signals are delegated to `ResultsToolbar`,
/// which subscribes to them independently.  Sort interactions therefore only
/// re-render `VirtualizedResultsTable`, not the toolbar or stats bar.
#[component]
pub fn ResultsTable() -> Element {
    let state = use_results_context();
    let explore = state.explore;
    let locale = *state.locale.read();
    let entries_len = explore.read().result.entries.len();

    // Memoised sort: compute a permutation of row indices instead of cloning
    // the whole Vec to sort it. Recomputes only when `entries` or `sort`
    // actually change.
    let sorted_indices: Memo<Arc<[u32]>> = use_memo(move || {
        let snapshot = explore.read();
        let rows = snapshot.result.entries.clone();
        let s = snapshot.result.sort;
        let mut idx: Vec<u32> = (0..rows.len() as u32).collect();
        idx.sort_by(|&a, &b| {
            let ea = &rows[a as usize];
            let eb = &rows[b as usize];
            let cmp = match s.col {
                SortColumn::Name => ea.name.cmp(&eb.name),
                SortColumn::Mass => ea
                    .mass
                    .partial_cmp(&eb.mass)
                    .unwrap_or(std::cmp::Ordering::Equal),
                SortColumn::Formula => ea.formula.cmp(&eb.formula),
                SortColumn::TaxonName => ea.taxon_name.cmp(&eb.taxon_name),
                SortColumn::PubYear => ea.pub_year.cmp(&eb.pub_year),
                SortColumn::RefTitle => ea.ref_title.cmp(&eb.ref_title),
            };
            if s.dir == SortDir::Desc {
                cmp.reverse()
            } else {
                cmp
            }
        });
        Arc::from(idx.into_boxed_slice())
    });

    let total = entries_len;

    rsx! {
        div { id: "results-section", class: "results-wrap",
            // Subscribes to query/stats/download signals; isolated from sort.
            ResultsToolbar {}

            if total == 0 {
                div { class: "empty-state",
                    p { "{t(locale, TextKey::NoResults)}" }
                }
            } else {
                VirtualizedResultsTable { explore, sorted_indices }
            }
        }
    }
}

/// Toolbar: query panel + stats bar + download actions + capped-rows notice.
///
/// Reads sparql_query, metadata_json, query_hash, result_hash, executed_criteria,
/// total_stats, total_matches, display_capped_rows, and entries (for fallback
/// stats) from context.  Intentionally separate from `ResultsTable` so that
/// sort changes never cause toolbar re-renders.
#[component]
fn ResultsToolbar() -> Element {
    let state = use_results_context();
    let explore = state.explore.read().clone();
    let entries = explore.result.entries.clone();
    let sparql_query = explore.result.sparql_query.clone();
    let metadata_json = explore.result.metadata_json.clone();
    let query_hash = explore.result.query_hash.clone();
    let result_hash = explore.result.result_hash.clone();
    let criteria = explore.ui.executed_criteria.clone();
    let total_stats = explore.result.total_stats.clone();
    let total_matches = explore.result.total_matches;
    let display_capped_rows = explore.result.display_capped_rows;

    // Fallback stats are memoised so they don't rerun on unrelated re-renders.
    let fallback_stats: Memo<DatasetStats> = use_memo(move || DatasetStats::from_entries(&entries));
    let display_stats = total_stats
        .as_ref()
        .cloned()
        .unwrap_or_else(|| fallback_stats.read().clone());
    let stats_partial = false;

    rsx! {
        QueryPanel { sparql_query: sparql_query.clone() }

        // ── Stats + toolbar ───────────────────────────────────────────
        div { class: "results-toolbar",
            StatBar {
                stats: display_stats,
                total_matches,
                stats_partial,
            }
            DownloadActionsGroup {
                criteria: criteria.clone(),
                sparql_query: sparql_query.clone(),
                metadata_json: metadata_json.clone(),
                query_hash,
                result_hash,
            }
        }
        if display_capped_rows {
            CappedRowsNotice {}
        }
    }
}

#[component]
fn VirtualizedResultsTable(
    explore: Signal<crate::features::explore::search_state::ExploreState>,
    sorted_indices: Memo<Arc<[u32]>>,
) -> Element {
    let locale = crate::hooks::use_locale();
    let virtualization_config = crate::hooks::use_virtualization::VirtualizationConfig {
        row_height_px: ROW_HEIGHT_PX_COMFORTABLE,
        overscan_rows: VIRTUAL_OVERSCAN_ROWS,
        viewport_fallback_px: TABLE_VIEWPORT_FALLBACK_PX,
        scroll_id: TABLE_SCROLL_ID,
    };
    #[cfg(target_arch = "wasm32")]
    let first_visible_row = use_signal(|| 0usize);
    #[cfg(not(target_arch = "wasm32"))]
    let first_visible_row = use_signal(|| 0usize);

    #[cfg(target_arch = "wasm32")]
    let viewport_height_px = use_signal(|| TABLE_VIEWPORT_FALLBACK_PX);
    #[cfg(not(target_arch = "wasm32"))]
    let viewport_height_px = use_signal(|| TABLE_VIEWPORT_FALLBACK_PX);

    #[cfg(target_arch = "wasm32")]
    let mut scroll_host = use_signal(|| None::<web_sys::HtmlElement>);

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
    let virtualization = crate::hooks::use_virtualization::use_virtualization(
        virtualization_config,
        total,
        *first_visible_row.read(),
        *viewport_height_px.read(),
    );
    let row_text = row_text(locale);

    rsx! {
        div {
            id: virtualization_config.scroll_id,
            class: "table-scroll",
            onscroll: move |_| {
                #[cfg(target_arch = "wasm32")]
                {
                    use wasm_bindgen::JsCast;
                    let div = if let Some(existing) = scroll_host.peek().as_ref() {
                        existing.clone()
                    } else {
                        let Some(win) = web_sys::window() else {
                            return;
                        };
                        let Some(document) = win.document() else {
                            return;
                        };
                        let Some(node) = document.get_element_by_id(virtualization_config.scroll_id) else {
                            return;
                        };
                        let Ok(found) = node.dyn_into::<web_sys::HtmlElement>() else {
                            return;
                        };
                        *scroll_host.write() = Some(found.clone());
                        found
                    };

                    // Coalesce multiple native scroll events into one update per frame.
                    if *scroll_raf_scheduled.peek() {
                        return;
                    }
                    *scroll_raf_scheduled.write() = true;

                    let mut first_visible_row_sig = first_visible_row;
                    let mut viewport_height_px_sig = viewport_height_px;
                    let mut scroll_raf_scheduled_sig = scroll_raf_scheduled;
                    let mut scroll_raf_cb_sig = scroll_raf_cb;
                    let mut scroll_raf_id_sig = scroll_raf_id;
                    let div_for_raf = div.clone();
                    let raf_cb = wasm_bindgen::closure::Closure::wrap(
                        Box::new(move |_ts: f64| {
                            let top = div_for_raf.scroll_top().max(0) as usize;
                            let height = div_for_raf.client_height().max(0) as usize;
                            let next_first = (top / virtualization_config.row_height_px).min(total);
                            if next_first != *first_visible_row_sig.peek() {
                                *first_visible_row_sig.write() = next_first;
                            }
                            if height > 0 && height != *viewport_height_px_sig.peek() {
                                *viewport_height_px_sig.write() = height;
                            }
                            *scroll_raf_id_sig.write() = None;
                            *scroll_raf_scheduled_sig.write() = false;
                            *scroll_raf_cb_sig.write() = None;
                        }) as Box<dyn FnMut(f64)>,
                    );
                    *scroll_raf_cb.write() = Some(raf_cb);
                    let scheduled_id = if let Some(win) = web_sys::window() {
                        if let Some(cb) = scroll_raf_cb.peek().as_ref() {
                            win.request_animation_frame(cb.as_ref().unchecked_ref()).ok()
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    if let Some(id) = scheduled_id {
                        *scroll_raf_id.write() = Some(id);
                    } else {
                        *scroll_raf_id.write() = None;
                        *scroll_raf_scheduled.write() = false;
                        *scroll_raf_cb.write() = None;
                    }
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
                        // Keep a single read for each reactive source per window render.
                        let rows = explore.read().result.entries.clone();
                        let order = sorted_indices.read().clone();
                        {
                            rsx! {
                                ResultsRowsWindow {
                                    locale,
                                    text: row_text,
                                    rows,
                                    order,
                                    start_row: virtualization.start_row,
                                    visible_count: virtualization.end_row.saturating_sub(virtualization.start_row),
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

// ── Sub-components ────────────────────────────────────────────────────────────

