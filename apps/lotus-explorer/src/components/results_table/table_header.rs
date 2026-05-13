// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Table header with sortable columns.

use super::sort_helpers::{aria_sort_for, sort_icon_for};
use crate::features::explore::actions::ExploreAction;
use crate::features::explore::search_state::ExploreState;
use crate::features::explore::search_state::dispatch_explore_action;
use crate::i18n::TextKey;
use crate::i18n::t;
use crate::models::SortColumn;
use crate::models::SortState;
use dioxus::prelude::*;

/// Table header row with sortable column headers.
#[component]
pub fn TableHeader(explore: Signal<ExploreState>) -> Element {
    let locale = crate::hooks::use_locale();
    let current_sort = explore.read().result.sort;

    let toggle_sort = move |col: SortColumn| {
        move |_: MouseEvent| {
            dispatch_explore_action(explore, ExploreAction::SortToggled(col));
        }
    };

    rsx! {
        tr {
            th { class: "th-static", scope: "col", "{t(locale, TextKey::Structure)}" }
            SortableColumnHeader {
                col: SortColumn::Name,
                sort: current_sort,
                on_toggle: toggle_sort(SortColumn::Name),
            }
            SortableColumnHeader {
                col: SortColumn::Mass,
                sort: current_sort,
                on_toggle: toggle_sort(SortColumn::Mass),
            }
            SortableColumnHeader {
                col: SortColumn::Formula,
                sort: current_sort,
                on_toggle: toggle_sort(SortColumn::Formula),
            }
            SortableColumnHeader {
                col: SortColumn::TaxonName,
                sort: current_sort,
                on_toggle: toggle_sort(SortColumn::TaxonName),
            }
            SortableColumnHeader {
                col: SortColumn::RefTitle,
                sort: current_sort,
                on_toggle: toggle_sort(SortColumn::RefTitle),
            }
            SortableColumnHeader {
                col: SortColumn::PubYear,
                sort: current_sort,
                on_toggle: toggle_sort(SortColumn::PubYear),
            }
        }
    }
}

/// Individual sortable column header.
#[component]
fn SortableColumnHeader(
    col: SortColumn,
    sort: SortState,
    on_toggle: EventHandler<MouseEvent>,
) -> Element {
    let locale = crate::hooks::use_locale();
    let label = match col {
        SortColumn::Name => TextKey::Compound,
        SortColumn::Mass => TextKey::Mass,
        SortColumn::Formula => TextKey::Formula,
        SortColumn::TaxonName => TextKey::TaxonCol,
        SortColumn::RefTitle => TextKey::Reference,
        SortColumn::PubYear => TextKey::Year,
    };

    rsx! {
        th {
            class: "sort-th",
            scope: "col",
            aria_sort: "{aria_sort_for(&sort, col)}",
            button {
                class: "sort-btn",
                r#type: "button",
                aria_label: "{t(locale, label)}",
                onclick: move |e| on_toggle.call(e),
                "{t(locale, label)} "
                span { class: "sort-icon", "aria-hidden": "true",
                    {sort_icon_for(&sort, col)}
                }
            }
        }
    }
}
