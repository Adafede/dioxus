// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Ketcher molecule editor panel.
//!
//! Full-width section rendered in the structure editor page. The Ketcher
//! standalone bundle (~29 MB / ~8.8 MB gzipped) is deferred until the user
//! clicks to load it, keeping the initial load path light.

use crate::i18n::{TextKey, t};
use dioxus::prelude::*;

/// Relative URL at which the Ketcher standalone build is served.
const KETCHER_URL: &str = "assets/ketcher/index.html";

#[component]
pub fn KetcherPanel() -> Element {
    let locale = crate::hooks::use_locale();
    // The Ketcher standalone bundle (`main.<hash>.js`, ~29MB / ~8.8MB gzipped)
    // is the app's own vendored same-origin static asset (fetched at build
    // time by the `fetch-ketcher` host binary). It is far too large to fetch
    // on the initial load path: under a throttled connection (Lighthouse /
    // PageSpeed Insights) the download can't finish in time and Chrome aborts
    // it as `net::ERR_CONNECTION_FAILED` — which is why this reproduces there
    // (and on flaky links) but not on an unthrottled `curl`. Defer the fetch
    // until the user opens the editor (a click), so automated/headless runs
    // never pull the heavy bundle onto the critical path. A same-sized
    // placeholder keeps the layout stable until then.
    let mut ketcher_ready = use_signal(|| false);
    rsx! {
        section {
            aria_label: "{t(locale, TextKey::KetcherSummary)}",
            style: crate::ui::style_constants::panel_containers::ketcher_panel_style(),
            div { style: crate::ui::style_constants::panel_containers::ketcher_wrap_style(),
                p { style: crate::ui::style_constants::forms::hint_text_style(),
                    "{t(locale, TextKey::KetcherHintA)}"
                    strong { "{t(locale, TextKey::KetcherSummary)}" }
                    "{t(locale, TextKey::KetcherHintB)}"
                    em { "{t(locale, TextKey::EditCopyDaylightSmiles)}" }
                    "{t(locale, TextKey::KetcherHintC)}"
                    em { "{t(locale, TextKey::CopyExtendedSmilesMol)}" }
                    "{t(locale, TextKey::KetcherHintD)}"
                }
                if *ketcher_ready.read() {
                    iframe {
                        src: "{KETCHER_URL}",
                        title: "{t(locale, TextKey::KetcherIframeTitle)}",
                        // Same-origin iframe (no sandbox) so the ~29MB script
                        // compiles in this page's renderer. (Re-add
                        // `sandbox="allow-scripts allow-same-origin allow-
                        // popups allow-forms allow-downloads"` to restore
                        // isolation.)
                        style: crate::ui::style_constants::panel_containers::iframe_style(),
                    }
                } else {
                    // Same box as the iframe; click loads the editor.
                    button {
                        aria_label: "{t(locale, TextKey::KetcherIframeTitle)}",
                        onclick: move |_| ketcher_ready.set(true),
                        style: crate::ui::style_constants::panel_containers::iframe_style(),
                        em {
                            style: crate::ui::style_constants::forms::hint_text_style(),
                            "{t(locale, TextKey::KetcherClickToLoad)}"
                        }
                    }
                }
            }
        }
    }
}
