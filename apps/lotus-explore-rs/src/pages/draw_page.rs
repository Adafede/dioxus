// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! The "Structure editor" tab: a full-pane Ketcher molecule editor.

use crate::i18n::view_label_draw;
use crate::pages::ketcher_panel::KetcherPanel;
use dioxus::prelude::*;

#[component]
pub fn DrawPage() -> Element {
    let locale = crate::hooks::use_locale();
    rsx! {
        section {
            class: "results-wrap w-full max-w-[1760px] mx-auto px-4 sm:px-6 lg:px-8",
            aria_labelledby: "draw-page-heading",
            h2 {
                id: "draw-page-heading",
                class: "text-title font-semibold text-text mb-4",
                "{view_label_draw(locale)}"
            }
            div { class: "results-inner w-full",
                KetcherPanel {}
            }
        }
    }
}
