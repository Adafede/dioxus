// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Virtualized results table body and WASM scroll scheduling glue.

use super::render_model::build_virtualized_table_render_model;
use super::row_cells::{ResultsRowsWindow, row_text};
use super::table_header::TableHeader;
use super::table_view_model::TableViewModel;
use super::virtualization_controller::use_results_table_virtualization;
use crate::features::explore::interactions::use_explore_interactions;
use crate::features::explore::selectors::ArcPtrEq;
use crate::i18n::{TextKey, t};
use crate::models::CompoundEntry;
use dioxus::prelude::*;
use ui::prelude::*;
use ui::styles::lotus::tokens::FOOTER_WD_ENTRIES;

#[component]
pub(super) fn VirtualizedResultsTable(
    entries: Memo<ArcPtrEq<[CompoundEntry]>>,
    table_view_model: Memo<TableViewModel>,
) -> Element {
    let locale = crate::hooks::use_locale();
    let interactions = use_explore_interactions();
    let total = entries.read().0.len();
    let virtualization = use_results_table_virtualization(total);
    let text = row_text(locale);

    let view_model = table_view_model.read();
    let rows = entries.read().0.clone();
    let render_model = build_virtualized_table_render_model(&view_model, virtualization.state);
    let mut effect_virtualization = virtualization.clone();
    let scroll_virtualization = virtualization.clone();

    use_effect(move || {
        effect_virtualization.sync_after_render(total);
    });

    let on_scroll = move |_| scroll_virtualization.handle_scroll(total);

    rsx! {
        div {
            id: virtualization.config.scroll_id,
            role: "region",
            tabindex: "0",
            aria_label: "{t(locale, TextKey::TableTriplesAria)}",
            style: table_scroll_style(),
            onscroll: on_scroll,
            table {
                aria_label: "{t(locale, TextKey::TableTriplesAria)}",
                style: results_table_style(),
                caption { style: crate::ui::style_constants::shared::sr_only_style(), "{t(locale, TextKey::TableTriplesAria)}" }
                colgroup {
                    col { style: col_style("124px") }
                    col { style: col_style("31ch") }
                    col { style: col_style("12ch") }
                    col { style: col_style("12ch") }
                    col { style: col_style("20ch") }
                    col { style: col_style("45ch") }
                    col { style: col_style("4ch") }
                }
                thead {
                    TableHeader {
                        current_sort: render_model.current_sort,
                        on_sort_toggle: move |col| interactions.toggle_sort(col),
                    }
                }
                tbody {
                    if render_model.has_top_spacer() {
                        tr { aria_hidden: "true",
                            td {
                                colspan: "7",
                                style: spacer_cell_style(render_model.top_spacer_px),
                            }
                        }
                    }
                    {
                        {
                            rsx! {
                                ResultsRowsWindow {
                                    locale,
                                    text,
                                    rows: rows,
                                    prepared_rows: render_model.prepared_rows.clone(),
                                    order: render_model.sorted_indices.clone(),
                                    start_row: render_model.start_row,
                                    end_row: render_model.end_row,
                                }
                            }
                        }
                    }
                    if render_model.has_bottom_spacer() {
                        tr { aria_hidden: "true",
                            td {
                                colspan: "7",
                                style: spacer_cell_style(render_model.bottom_spacer_px),
                            }
                        }
                    }
                }
            }
        }
    }
}

fn col_style(width: &str) -> String {
    StyleBuilder::new().property("width", width).build()
}

fn spacer_cell_style(height: usize) -> String {
    StyleBuilder::new()
        .property("height", &format!("{height}px"))
        .build()
}

fn table_scroll_style() -> String {
    StyleBuilder::new()
        .property("overflow", "auto")
        .property("max-height", "min(72vh, 980px)")
        .border("1px solid var(--results-border)")
        .property("border-left", &format!("4px solid {}", FOOTER_WD_ENTRIES))
        .border_radius("14px")
        .background_color("transparent")
        .box_shadow("var(--panel-shadow)")
        .property(
            "transition",
            "background .15s ease, border-color .15s ease, box-shadow .15s ease",
        )
        .build()
}

fn results_table_style() -> String {
    StyleBuilder::new()
        .property("width", "100%")
        .property("min-width", "max-content")
        .property("border-collapse", "collapse")
        .font_size("var(--fs-ui)")
        .property("table-layout", "auto")
        .property("word-break", "break-word")
        .build()
}
