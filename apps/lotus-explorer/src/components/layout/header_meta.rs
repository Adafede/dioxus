// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::components::copy_button::CopyButton;
use crate::i18n::{Locale, TextKey, t};
use dioxus::prelude::*;
use std::sync::Arc;

#[component]
pub fn HeaderMetaSection(
    resolved_qid: Signal<Option<String>>,
    query_hash: Signal<Option<String>>,
    result_hash: Signal<Option<String>>,
    total_matches: Signal<Option<usize>>,
    locale: Signal<Locale>,
) -> Element {
    let locale = *locale.read();
    rsx! {
        if let Some(qid) = resolved_qid.read().as_deref() {
            p { class: "page-meta",
                span { class: "meta-key", "{t(locale, TextKey::ResolvedTaxon)}" }
                span { class: "meta-sep", ":" }
                span { class: "meta-val mono", "{qid}" }
                CopyButton {
                    text: Arc::<str>::from(qid),
                    title: t(locale, TextKey::CopyTaxonQid),
                    locale,
                }
            }
        }
        if let (Some(qh), Some(rh)) = (
            query_hash.read().as_deref(),
            result_hash.read().as_deref(),
        )
        {
            p { class: "page-meta",
                span { class: "meta-key", "{t(locale, TextKey::QueryHash)}" }
                span { class: "meta-sep", ":" }
                span { class: "meta-val mono", "{&qh[..12]}" }
                CopyButton {
                    text: Arc::<str>::from(qh),
                    title: t(locale, TextKey::CopyFullQueryHash),
                    locale,
                }
                span { class: "meta-sep", "·" }
                span { class: "meta-key", "{t(locale, TextKey::ResultHash)}" }
                span { class: "meta-sep", ":" }
                span { class: "meta-val mono", "{&rh[..12]}" }
                CopyButton {
                    text: Arc::<str>::from(rh),
                    title: t(locale, TextKey::CopyFullResultHash),
                    locale,
                }
            }
        }
        if let Some(n) = *total_matches.read() {
            p { class: "page-meta",
                span { class: "meta-key", "{t(locale, TextKey::TotalMatches)}" }
                span { class: "meta-sep", ":" }
                span { class: "meta-val mono", "{n}" }
            }
        }
    }
}
