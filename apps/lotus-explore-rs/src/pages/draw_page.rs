// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! The "Structure editor" tab: a full-pane Ketcher molecule editor.

use crate::components::search_panel::KetcherPanel;
use crate::i18n::view_label_draw;
use dioxus::prelude::*;
use ui::prelude::*;

#[component]
pub fn DrawPage() -> Element {
    let locale = crate::hooks::use_locale();
    rsx! {
        section { aria_label: "{view_label_draw(locale)}", style: panel_stack_style("12px 22px 18px", "0"), KetcherPanel {} }
    }
}

fn panel_stack_style(padding: &str, gap: &str) -> String {
    StyleBuilder::new()
        .display("flex")
        .flex_direction("column")
        .gap(gap)
        .padding(padding)
        .build()
}
