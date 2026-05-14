// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Header metadata strip (resolved taxon QID, query/result hashes, total matches).
//!
//! Reads from [`crate::state::ResultsContext`] and `use_locale()` — zero props.

use crate::components::copy_button::CopyButton;
use crate::i18n::{TextKey, format_count, t};
use crate::state::use_results_context;
use dioxus::prelude::*;
use std::sync::Arc;

/// Displays resolved-taxon QID, query/result hashes, and total-match count.
///
/// All items are gathered into a single `page-header-meta` card that only
/// renders when at least one value is present. Each row is a uniform
/// `p.page-meta > span.meta-item` structure.
#[component]
pub fn HeaderMetaSection() -> Element {
    let explore = use_results_context().explore;
    let locale = crate::hooks::use_locale();
    let explore = explore.read();

    let has_meta = explore.result.resolved_qid.is_some()
        || explore.result.query_hash.is_some()
        || explore.result.result_hash.is_some()
        || explore.result.total_matches.is_some();

    rsx! {
        if has_meta {
            div { class: "page-header-meta",
                if let Some(qid) = explore.result.resolved_qid.as_deref() {
                    p { class: "page-meta",
                        span { class: "meta-item",
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
                }
                if let Some(qh) = explore.result.query_hash.as_deref() {
                    p { class: "page-meta",
                        span { class: "meta-item",
                            span { class: "meta-key", "{t(locale, TextKey::QueryHash)}" }
                            span { class: "meta-sep", ":" }
                            span { class: "meta-val mono", "{&qh[..12]}" }
                            CopyButton {
                                text: Arc::<str>::from(qh),
                                title: t(locale, TextKey::CopyFullQueryHash),
                                locale,
                            }
                        }
                    }
                }
                if let Some(rh) = explore.result.result_hash.as_deref() {
                    p { class: "page-meta",
                        span { class: "meta-item",
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
                }
                if let Some(n) = explore.result.total_matches {
                    p { class: "page-meta",
                        span { class: "meta-item",
                            span { class: "meta-key", "{t(locale, TextKey::TotalMatches)}" }
                            span { class: "meta-sep", ":" }
                            span { class: "meta-val mono", "{format_count(locale, n)}" }
                        }
                    }
                }
            }
        }
    }
}
