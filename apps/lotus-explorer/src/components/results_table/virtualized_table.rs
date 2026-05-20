// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Virtualized results table body and WASM scroll scheduling glue.

use super::render_model::build_virtualized_table_render_model;
use super::row_cells::{ResultsRowsWindow, row_text};
use super::table_view_model::TableViewModel;
use super::table_header::TableHeader;
use super::virtualization_controller::use_results_table_virtualization;
use crate::features::explore::interactions::use_explore_interactions;
use crate::i18n::{TextKey, t};
use crate::models::Rows;
use dioxus::prelude::*;

#[component]
pub(super) fn VirtualizedResultsTable(
    entries: Memo<Rows>,
    table_view_model: Memo<TableViewModel>,
) -> Element {
    let locale = crate::hooks::use_locale();
    let interactions = use_explore_interactions();
    let total = entries.read().len();
    let virtualization = use_results_table_virtualization(total);
    let text = row_text(locale);

    let view_model = table_view_model.read();
    let rows = entries.read().clone();
    let render_model = build_virtualized_table_render_model(&view_model, virtualization.state);
    let effect_virtualization = virtualization.clone();
    let scroll_virtualization = virtualization.clone();

    use_effect(move || {
        effect_virtualization.sync_after_render(total);
    });

    let on_scroll = move |_| scroll_virtualization.handle_scroll(total);

    rsx! {
        div {
            id: virtualization.config.scroll_id,
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
                        current_sort: render_model.current_sort,
                        on_sort_toggle: move |col| interactions.toggle_sort(col),
                    }
                }
                tbody {
                    if render_model.has_top_spacer() {
                        tr { class: "virtual-spacer-row", aria_hidden: "true",
                            td {
                                class: "virtual-spacer-cell",
                                colspan: "7",
                                style: "height: {render_model.top_spacer_px}px;",
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
                                    prepared_rows: render_model.prepared_rows.clone(),
                                    visible_order: render_model.visible_order.clone(),
                                }
                            }
                        }
                    }
                    if render_model.has_bottom_spacer() {
                        tr { class: "virtual-spacer-row", aria_hidden: "true",
                            td {
                                class: "virtual-spacer-cell",
                                colspan: "7",
                                style: "height: {render_model.bottom_spacer_px}px;",
                            }
                        }
                    }
                }
            }
        }
    }
}
