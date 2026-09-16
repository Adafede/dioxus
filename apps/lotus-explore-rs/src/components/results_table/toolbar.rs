// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Toolbar assembly for the results table.

use super::table_toolbar_sections::{CappedRowsNotice, StatBar};
use dioxus::prelude::*;

#[component]
pub(super) fn ResultsToolbar() -> Element {
    rsx! {
        div { class: "flex w-full min-w-0 flex-col gap-4 px-0 max-w-none",
            super::table_toolbar_sections::QueryPanel {}
            StatBar {}
            super::table_toolbar_sections::DownloadActionsGroup {}
        }
        CappedRowsNotice {}
    }
}
