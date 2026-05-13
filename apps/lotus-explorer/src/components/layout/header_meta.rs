// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Header metadata strip (resolved taxon QID, query/result hashes, total matches).
//!
//! Reads from [`crate::state::ResultsContext`] and `use_locale()` — zero props.

use crate::components::copy_button::CopyButton;
use crate::i18n::{TextKey, t};
use crate::state::use_results_context;
use dioxus::prelude::*;
use std::sync::Arc;

/// Displays resolved-taxon QID, query/result hashes, and total-match count.
///
/// Zero props — reads `ResultsContext.explore` for result data and
/// `use_locale()` for the active locale.
#[component]
pub fn HeaderMetaSection() -> Element {
    let explore = use_results_context().explore;
    let locale = crate::hooks::use_locale();
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
                span { class: "meta-sep", "" }
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
