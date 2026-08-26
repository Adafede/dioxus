// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Toolbar assembly for the results table.

use super::table_toolbar_sections::{CappedRowsNotice, StatBar};
use dioxus::prelude::*;

#[component]
pub(super) fn ResultsToolbar() -> Element {
    rsx! {
        super::table_toolbar_sections::QueryPanel {}
        div {
            class: "flex w-full min-w-0 flex-col items-stretch gap-2 rounded-xl border border-panel-border bg-panel-soft p-2.5 shadow-xs sm:p-3",
            StatBar {}
            super::table_toolbar_sections::DownloadActionsGroup {}
        }
        CappedRowsNotice {}
    }
}
