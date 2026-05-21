// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Header metadata strip (resolved taxon QID, query/result hashes, total matches).
//!
//! Reads from [`crate::state::ResultsContext`] and `use_locale()` — zero props.

use crate::components::copy_button::CopyButton;
use crate::i18n::{TextKey, t};
use crate::state::use_form_criteria_context;
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
    let form_ctx = use_form_criteria_context();
    let locale = crate::hooks::use_locale();
    let resolved_qid =
        crate::features::explore::selectors::use_result_selector(explore, |result| {
            result.resolved_qid.clone()
        });
    let query_hash = crate::features::explore::selectors::use_result_selector(explore, |result| {
        result.query_hash.clone()
    });
    let result_hash = crate::features::explore::selectors::use_result_selector(explore, |result| {
        result.result_hash.clone()
    });
    let criteria =
        crate::features::explore::selectors::use_criteria_selector(form_ctx.criteria, |c| {
            c.clone()
        });

    let mut prev_criteria = use_signal(|| criteria.read().clone());
    let mut prev_meta = use_signal(|| {
        (
            resolved_qid.read().clone(),
            query_hash.read().clone(),
            result_hash.read().clone(),
        )
    });
    let mut meta_visible = use_signal(|| {
        resolved_qid.read().is_some() || query_hash.read().is_some() || result_hash.read().is_some()
    });

    // Criteria changes invalidate the entire metadata strip until fresh results arrive.
    use_effect(move || {
        let current_criteria = criteria.read().clone();
        if current_criteria != *prev_criteria.read() {
            meta_visible.set(false);
            prev_criteria.set(current_criteria);
        }
    });

    // Show metadata again when a fresh metadata tuple is produced.
    use_effect(move || {
        let current_meta = (
            resolved_qid.read().clone(),
            query_hash.read().clone(),
            result_hash.read().clone(),
        );
        if current_meta != *prev_meta.read() {
            meta_visible.set(current_meta.0.is_some() || current_meta.1.is_some() || current_meta.2.is_some());
            prev_meta.set(current_meta);
        }
    });

    let resolved_qid_value = resolved_qid.read().clone();
    let query_hash_value = query_hash.read().clone();
    let result_hash_value = result_hash.read().clone();

    let has_meta = *meta_visible.read()
        && (resolved_qid_value.is_some() || query_hash_value.is_some() || result_hash_value.is_some());

    rsx! {
        if has_meta {
            div { class: "page-header-meta",
                if let Some(qid) = resolved_qid_value.as_deref() {
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
                if let Some(qh) = query_hash_value.as_deref() {
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
                if let Some(rh) = result_hash_value.as_deref() {
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
        }
    }
}
