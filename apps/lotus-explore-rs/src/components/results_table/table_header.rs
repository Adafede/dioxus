// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Table header with sortable columns.

use super::header_model::{SortableHeaderModel, build_sortable_header_models};
use crate::i18n::{TextKey, aria_sort_toggle, t};
use crate::models::{SortColumn, SortState};
use dioxus::prelude::*;
use ui::prelude::*;

/// Table header row with sortable column headers.
#[component]
pub fn TableHeader(current_sort: SortState, on_sort_toggle: EventHandler<SortColumn>) -> Element {
    let locale = crate::hooks::use_locale();
    let headers = build_sortable_header_models(current_sort);

    rsx! {
        tr {
            th { scope: "col", style: table_header_cell_style(),
                span { style: header_label_style(), "{t(locale, TextKey::Structure)}" }
            }
            for header in headers {
                SortableColumnHeader {
                    header,
                    on_toggle: on_sort_toggle,
                }
            }
        }
    }
}

/// Individual sortable column header.
#[component]
fn SortableColumnHeader(
    header: SortableHeaderModel,
    on_toggle: EventHandler<SortColumn>,
) -> Element {
    let locale = crate::hooks::use_locale();
    let label_text = t(locale, header.label);
    let sort_aria = aria_sort_toggle(locale, label_text, header.next_descending);

    rsx! {
        th {
            scope: "col",
            aria_sort: "{header.aria_sort}",
            style: table_header_cell_style(),
            button {
                r#type: "button",
                aria_label: "{sort_aria}",
                title: "{sort_aria}",
                style: sort_button_style(),
                onclick: move |_| on_toggle.call(header.col),
                span { style: header_label_style(), "{label_text}" }
                span { style: sort_icon_style(), "aria-hidden": "true", {header.sort_icon} }
            }
        }
    }
}

fn header_label_style() -> String {
    StyleBuilder::new()
        .display("block")
        .property("min-width", "max-content")
        .property("white-space", "nowrap")
        .property("overflow", "visible")
        .property("text-overflow", "clip")
        .property("line-height", "1.2")
        .font_weight("inherit")
        .font_size("inherit")
        .property("text-transform", "inherit")
        .property("letter-spacing", "inherit")
        .build()
}

fn table_header_cell_style() -> String {
    StyleBuilder::new()
        .padding("9px 10px")
        .text_align("left")
        .font_size("var(--fs-label)")
        .font_weight("700")
        .color("var(--critical-muted)")
        .border_bottom("1px solid var(--results-border)")
        .property("white-space", "nowrap")
        .property("user-select", "none")
        .property("text-transform", "uppercase")
        .property("letter-spacing", "0.08em")
        .property("width", "auto")
        .property("min-width", "max-content")
        .build()
}

fn sort_button_style() -> String {
    StyleBuilder::new()
        .property("appearance", "none")
        .background_color("transparent")
        .border("0")
        .color("inherit")
        .font_family("inherit")
        .padding("0")
        .property("margin", "0")
        .cursor("pointer")
        .display("grid")
        .align_items("start")
        .property("grid-template-columns", "auto auto")
        .property("column-gap", "6px")
        .property("width", "100%")
        .property("min-width", "max-content")
        .build()
}

fn sort_icon_style() -> String {
    StyleBuilder::new()
        .color("var(--text3)")
        .font_size("var(--fs-0)")
        .font_weight("700")
        .property("line-height", "1")
        .build()
}
