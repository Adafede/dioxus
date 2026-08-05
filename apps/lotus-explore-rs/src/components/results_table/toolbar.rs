// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Toolbar assembly for the results table: query panel, stats bar, downloads,
//! and the capped-rows notice.

use super::table_toolbar_sections::{CappedRowsNotice, StatBar};
use dioxus::prelude::*;

/// Toolbar: query panel + stats bar + download actions + capped-rows notice.
///
/// Intentionally separate from `ResultsTable` so that sort changes never cause
/// toolbar re-renders. Each section reads only the slices of context it needs.
#[component]
pub(super) fn ResultsToolbar() -> Element {
    rsx! {
        super::table_toolbar_sections::QueryPanel {}
        div { class: "results-toolbar",
            StatBar {}
            super::table_toolbar_sections::DownloadActionsGroup {}
        }
        CappedRowsNotice {}
    }
}
