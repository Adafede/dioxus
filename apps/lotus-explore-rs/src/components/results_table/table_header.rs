// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Table header with sortable columns.

use super::header_model::{SortableHeaderModel, build_sortable_header_models};
use crate::i18n::{TextKey, aria_sort_toggle, t};
use crate::models::{SortColumn, SortState};
use crate::ui::style_constants::table;
use dioxus::prelude::*;

/// Table header row with sortable column headers.
#[component]
pub fn TableHeader(current_sort: SortState, on_sort_toggle: EventHandler<SortColumn>) -> Element {
    let locale = crate::hooks::use_locale();
    let headers = build_sortable_header_models(current_sort);

    rsx! {
        tr {
            th { scope: "col", style: table::table_header_cell_style(),
                span { style: table::header_label_style(), "{t(locale, TextKey::Structure)}" }
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
            style: table::table_header_cell_style(),
            button {
                r#type: "button",
                aria_label: "{sort_aria}",
                title: "{sort_aria}",
                style: table::sort_button_style(),
                onclick: move |_| on_toggle.call(header.col),
                span { style: table::header_label_style(), "{label_text}" }
                span { style: table::sort_icon_style(), "aria-hidden": "true", {header.sort_icon} }
            }
        }
    }
}
