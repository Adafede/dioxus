// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Header metadata strip (resolved taxon QID, query/result hashes, total matches).
//!
//! Reads from [`crate::state::ResultsContext`] and `use_locale()` — zero props.

use crate::components::copy_button::CopyButton;
use crate::features::explore::use_header_meta_snapshot;
use crate::i18n::{TextKey, t};
use crate::state::use_form_criteria_context;
use crate::state::use_results_context;
use dioxus::prelude::*;
use std::sync::Arc;

fn hash_prefix(value: &str) -> &str {
    value.get(..12).unwrap_or(value)
}

#[component]
fn ResolvedTaxonMetaItem(locale: crate::i18n::Locale, qid: Arc<str>) -> Element {
    rsx! {
        span { class: "meta-item",
            span { class: "meta-key", "{t(locale, TextKey::ResolvedTaxon)}" }
            span { class: "meta-sep", ":" }
            span { class: "meta-val mono", "{qid}" }
            CopyButton {
                text: qid,
                title: t(locale, TextKey::CopyTaxonQid),
                locale,
            }
        }
    }
}

#[component]
fn QueryHashMetaItem(locale: crate::i18n::Locale, full_hash: Arc<str>) -> Element {
    rsx! {
        span { class: "meta-item",
            span { class: "meta-key", "{t(locale, TextKey::QueryHash)}" }
            span { class: "meta-sep", ":" }
            span { class: "meta-val mono", "{hash_prefix(&full_hash)}" }
            CopyButton {
                text: full_hash,
                title: t(locale, TextKey::CopyFullQueryHash),
                locale,
            }
        }
    }
}

#[component]
fn ResultHashMetaItem(locale: crate::i18n::Locale, full_hash: Arc<str>) -> Element {
    rsx! {
        span { class: "meta-item",
            span { class: "meta-key", "{t(locale, TextKey::ResultHash)}" }
            span { class: "meta-sep", ":" }
            span { class: "meta-val mono", "{hash_prefix(&full_hash)}" }
            CopyButton {
                text: full_hash,
                title: t(locale, TextKey::CopyFullResultHash),
                locale,
            }
        }
    }
}

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
    let header_snapshot = use_header_meta_snapshot(explore);
    let criteria =
        crate::features::explore::selectors::use_criteria_selector(form_ctx.criteria, |c| {
            c.clone()
        });

    let mut prev_criteria = use_signal(|| criteria.read().clone());
    let mut prev_meta = use_signal(|| header_snapshot.read().clone());
    let mut meta_visible = use_signal(|| {
        let snapshot = header_snapshot.read();
        snapshot.resolved_qid.is_some()
            || snapshot.query_hash.is_some()
            || snapshot.result_hash.is_some()
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
        let current_meta = header_snapshot.read().clone();
        if current_meta != *prev_meta.read() {
            meta_visible.set(
                current_meta.resolved_qid.is_some()
                    || current_meta.query_hash.is_some()
                    || current_meta.result_hash.is_some(),
            );
            prev_meta.set(current_meta);
        }
    });

    let snapshot_ref = header_snapshot.read();
    let resolved_qid_value = snapshot_ref.resolved_qid.as_deref();
    let query_hash_value = snapshot_ref.query_hash.as_deref();
    let result_hash_value = snapshot_ref.result_hash.as_deref();

    let has_meta = *meta_visible.read()
        && (resolved_qid_value.is_some()
            || query_hash_value.is_some()
            || result_hash_value.is_some());

    rsx! {
        if has_meta {
            div { class: "page-header-meta",
                if let Some(qid) = resolved_qid_value {
                    ResolvedTaxonMetaItem { locale, qid: Arc::from(qid) }
                }
                if let Some(qh) = query_hash_value {
                    QueryHashMetaItem { locale, full_hash: Arc::from(qh) }
                }
                if let Some(rh) = result_hash_value {
                    ResultHashMetaItem { locale, full_hash: Arc::from(rh) }
                }
            }
        }
    }
}
