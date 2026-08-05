// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! The "Structure editor" tab: a full-pane Ketcher molecule editor.

use crate::components::search_panel::KetcherPanel;
use crate::i18n::view_label_draw;
use dioxus::prelude::*;

#[component]
pub fn DrawPage() -> Element {
    let locale = crate::hooks::use_locale();
    rsx! {
        section { class: "draw-wrap", aria_label: "{view_label_draw(locale)}", KetcherPanel {} }
    }
}
