// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::components::copy_button::CopyButton;
use crate::i18n::{Locale, TextKey, t};
use dioxus::prelude::*;
use std::sync::Arc;

#[component]
pub fn HeaderMetaSection(
    explore: Signal<crate::features::explore::search_state::ExploreState>,
    locale: Signal<Locale>,
) -> Element {
    let locale = *locale.read();
    let explore = explore.read();
    rsx! {
        if let Some(qid) = explore.result.resolved_qid.as_deref() {
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
            explore.result.query_hash.as_deref(),
            explore.result.result_hash.as_deref(),
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
        if let Some(n) = explore.result.total_matches {
            p { class: "page-meta",
                span { class: "meta-key", "{t(locale, TextKey::TotalMatches)}" }
                span { class: "meta-sep", ":" }
                span { class: "meta-val mono", "{n}" }
            }
        }
    }
}
