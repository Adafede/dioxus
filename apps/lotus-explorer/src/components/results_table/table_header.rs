// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Table header with sortable columns.

use super::sort_helpers::{aria_sort_for, sort_icon_for};
use crate::i18n::{TextKey, aria_sort_toggle, t};
use crate::models::SortColumn;
use crate::models::SortDir;
use crate::models::SortState;
use dioxus::prelude::*;

#[derive(Clone, Copy)]
struct HeaderColumn {
    col: SortColumn,
    label: TextKey,
}

const SORTABLE_COLUMNS: [HeaderColumn; 6] = [
    HeaderColumn {
        col: SortColumn::Name,
        label: TextKey::Compound,
    },
    HeaderColumn {
        col: SortColumn::Mass,
        label: TextKey::Mass,
    },
    HeaderColumn {
        col: SortColumn::Formula,
        label: TextKey::Formula,
    },
    HeaderColumn {
        col: SortColumn::TaxonName,
        label: TextKey::TaxonCol,
    },
    HeaderColumn {
        col: SortColumn::RefTitle,
        label: TextKey::Reference,
    },
    HeaderColumn {
        col: SortColumn::PubYear,
        label: TextKey::Year,
    },
];

fn next_sort_is_descending(sort: SortState, col: SortColumn) -> bool {
    sort.col == col && sort.dir == SortDir::Asc
}

/// Table header row with sortable column headers.
#[component]
pub fn TableHeader(current_sort: SortState, on_sort_toggle: EventHandler<SortColumn>) -> Element {
    let locale = crate::hooks::use_locale();

    rsx! {
        tr {
            th { class: "th-static", scope: "col",
                span { class: "th-label", "{t(locale, TextKey::Structure)}" }
            }
            for header in SORTABLE_COLUMNS {
                SortableColumnHeader {
                    col: header.col,
                    label: header.label,
                    sort: current_sort,
                    on_toggle: on_sort_toggle,
                }
            }
        }
    }
}

/// Individual sortable column header.
#[component]
fn SortableColumnHeader(
    col: SortColumn,
    label: TextKey,
    sort: SortState,
    on_toggle: EventHandler<SortColumn>,
) -> Element {
    let locale = crate::hooks::use_locale();
    let label_text = t(locale, label);
    let next_descending = next_sort_is_descending(sort, col);
    let sort_aria = aria_sort_toggle(locale, label_text, next_descending);

    rsx! {
        th {
            class: "sort-th",
            scope: "col",
            aria_sort: "{aria_sort_for(&sort, col)}",
            button {
                class: "sort-btn",
                r#type: "button",
                aria_label: "{sort_aria}",
                title: "{sort_aria}",
                onclick: move |_| on_toggle.call(col),
                span { class: "th-label", "{label_text}" }
                span { class: "sort-icon", "aria-hidden": "true", {sort_icon_for(&sort, col)} }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_sort_is_descending_only_for_active_ascending_column() {
        let sort = SortState {
            col: SortColumn::Mass,
            dir: SortDir::Asc,
        };
        assert!(next_sort_is_descending(sort, SortColumn::Mass));
        assert!(!next_sort_is_descending(sort, SortColumn::Name));

        let sort_desc = SortState {
            col: SortColumn::Mass,
            dir: SortDir::Desc,
        };
        assert!(!next_sort_is_descending(sort_desc, SortColumn::Mass));
    }
}
