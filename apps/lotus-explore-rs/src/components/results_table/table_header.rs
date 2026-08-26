// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Table header with sortable columns.

use super::header_model::{SortableHeaderModel, build_sortable_header_models};
use crate::i18n::{TextKey, aria_sort_toggle, t};
use crate::models::{SortColumn, SortState};
use dioxus::prelude::*;

#[component]
pub fn TableHeader(current_sort: SortState, on_sort_toggle: EventHandler<SortColumn>) -> Element {
    let locale = crate::hooks::use_locale();
    let headers = build_sortable_header_models(current_sort);

    rsx! {
        tr {
            class: "border-b border-border bg-panel-soft text-left text-subtle",
            th {
                scope: "col",
                class: "th-static p-2.5 text-micro font-semibold uppercase tracking-wider text-subtle select-none",
                span { "{t(locale, TextKey::Structure)}" }
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

#[component]
fn SortableColumnHeader(
    header: SortableHeaderModel,
    on_toggle: EventHandler<SortColumn>,
) -> Element {
    let locale = crate::hooks::use_locale();
    let label_text = t(locale, header.label);
    let sort_aria = aria_sort_toggle(locale, label_text, header.next_descending);
    let stripe = match header.col {
        SortColumn::Name => "border-l-4 border-l-wd-compound bg-stat-compound",
        SortColumn::TaxonName => "border-l-4 border-l-wd-taxon bg-stat-taxon",
        SortColumn::RefTitle => "border-l-4 border-l-wd-reference bg-stat-reference",
        _ => "",
    };

    rsx! {
        th {
            scope: "col",
            aria_sort: "{header.aria_sort}",
            class: "sort-th p-2.5 text-micro font-semibold uppercase tracking-wider text-subtle select-none {stripe}",
            button {
                r#type: "button",
                aria_label: "{sort_aria}",
                title: "{sort_aria}",
                class: "group inline-flex cursor-pointer items-center gap-1 border-0 bg-transparent p-0 text-inherit transition-colors hover:text-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/40 focus-visible:rounded-sm",
                onclick: move |_| on_toggle.call(header.col),
                span { "{label_text}" }
                span {
                    class: "text-subtle group-hover:text-accent",
                    "aria-hidden": "true",
                    {header.sort_icon}
                }
            }
        }
    }
}
