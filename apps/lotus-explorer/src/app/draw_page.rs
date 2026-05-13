// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! The "Draw" tab: a full-pane Ketcher molecule editor.

use crate::components::search_panel::KetcherPanel;
use crate::i18n::{Locale, view_label_draw};
use dioxus::prelude::*;

#[component]
pub fn DrawPage(locale: Locale) -> Element {
    rsx! {
        section { class: "draw-wrap", aria_label: "{view_label_draw(locale)}", KetcherPanel {} }
    }
}
