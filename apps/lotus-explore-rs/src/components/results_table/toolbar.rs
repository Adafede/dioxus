// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Toolbar assembly for the results table: query panel, stats bar, downloads,
//! and the capped-rows notice.

use super::table_toolbar_sections::{CappedRowsNotice, StatBar};
use dioxus::prelude::*;
use ui::prelude::*;

/// Toolbar: query panel + stats bar + download actions + capped-rows notice.
///
/// Intentionally separate from `ResultsTable` so that sort changes never cause
/// toolbar re-renders. Each section reads only the slices of context it needs.
#[component]
pub(super) fn ResultsToolbar() -> Element {
    rsx! {
        super::table_toolbar_sections::QueryPanel {}
        div { style: toolbar_panel_style(),
            StatBar {}
            super::table_toolbar_sections::DownloadActionsGroup {}
        }
        CappedRowsNotice {}
    }
}

fn toolbar_panel_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .flex_direction("column")
        .align_items("stretch")
        .gap("8px")
        .border("1px solid var(--results-border)")
        .border_radius("12px")
        .padding("10px 12px")
        .background_color("transparent")
        .box_shadow("var(--panel-shadow)")
        .property("width", "100%")
        .property("min-width", "0")
        .build()
}
