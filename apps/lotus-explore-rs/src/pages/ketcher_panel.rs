// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Ketcher molecule editor panel.

use crate::document_head::asset_url;
use crate::i18n::{TextKey, t};
use crate::ui::classes;
use dioxus::prelude::*;

#[component]
pub fn KetcherPanel() -> Element {
    let locale = crate::hooks::use_locale();
    let mut ketcher_ready = use_signal(|| false);
    let ketcher_url = asset_url("assets/ketcher/index.html");
    rsx! {
        div {
            class: "flex w-full flex-col gap-3",
            div {
                class: "flex flex-col gap-3 p-4",
                p { class: "{classes::HINT} px-1",
                    "{t(locale, TextKey::KetcherHintA)}"
                    strong { class: "text-muted", "{t(locale, TextKey::KetcherSummary)}" }
                    "{t(locale, TextKey::KetcherHintB)}"
                    em { "{t(locale, TextKey::EditCopyDaylightSmiles)}" }
                    "{t(locale, TextKey::KetcherHintC)}"
                    em { "{t(locale, TextKey::CopyExtendedSmilesMol)}" }
                    "{t(locale, TextKey::KetcherHintD)}"
                }
                if *ketcher_ready.read() {
                    iframe {
                        src: "{ketcher_url}",
                        title: "{t(locale, TextKey::KetcherIframeTitle)}",
                        class: "min-h-[420px] w-full flex-1 border-0 bg-surface",
                    }
                } else {
                    button {
                        aria_label: "{t(locale, TextKey::KetcherIframeTitle)}",
                        class: "flex min-h-[420px] w-full flex-1 cursor-pointer items-center justify-center border-0 bg-bg text-center hover:bg-panel-soft {classes::FOCUS_RING_BTN} focus-visible:rounded",
                        onclick: move |_| ketcher_ready.set(true),
                        em {
                            class: "{classes::HINT}",
                            "{t(locale, TextKey::KetcherClickToLoad)}"
                        }
                    }
                }
            }
        }
    }
}
