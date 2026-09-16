// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! The "Structure editor" tab: a full-pane Ketcher molecule editor.

use crate::pages::ketcher_panel::KetcherPanel;
use dioxus::prelude::*;

#[component]
pub fn DrawPage() -> Element {
    rsx! {
        section {
            class: "page-section w-full max-w-none px-4 sm:px-6 lg:px-8",
            div { class: "px-4 sm:px-6 lg:px-8",
                div { class: "w-full rounded-2xl border border-panel-border bg-panel shadow-xs overflow-hidden",
                    KetcherPanel {}
                }
            }
        }
    }
}
