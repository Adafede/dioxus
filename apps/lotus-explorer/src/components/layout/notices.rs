// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Status, warning, and error notice components.
//!
//! All notice components read locale via `use_locale()` and explore state via
//! `ResultsContext` — no `explore` or `locale` props are drilled from `App`.

use crate::components::copy_button::CopyButton;
use crate::features::explore::types::{
    DomainError, ErrorKind, ParseFault, TaxonWarning, ValidationFault,
};
use crate::features::explore::url_state::absolute_share_url;
#[cfg(target_arch = "wasm32")]
use crate::i18n::err_wasm_large_query_fallback;
#[cfg(target_arch = "wasm32")]
use crate::i18n::error_hint_memory;
use crate::i18n::{
    Locale, TextKey, err_invalid_search_input, err_query_stage_failed, err_taxon_not_found,
    err_taxon_parse_failed, err_taxon_resolution_failed, err_unsupported_format, t,
    warn_ambiguous_taxon, warn_input_standardized,
};
use crate::state::use_results_context;
use dioxus::prelude::*;
use std::sync::Arc;

// ── i18n formatters (UI boundary) ─────────────────────────────────────────────

/// Format a [`DomainError`] into a locale-appropriate display string.
///
/// This is the **only** place in the codebase that converts structured domain
/// errors into user-visible strings.
pub fn format_domain_error(locale: Locale, err: &DomainError) -> String {
    match err {
        DomainError::Validation(v) => format_validation_fault(locale, v),
        DomainError::Transport { stage, source } => {
            err_query_stage_failed(locale, stage, &source.to_string())
        }
        DomainError::Parse(p) => format_parse_fault(locale, p),
        #[cfg(target_arch = "wasm32")]
        DomainError::MemoryLimit { .. } => err_wasm_large_query_fallback(locale, "large query"),
    }
}

fn format_validation_fault(locale: Locale, fault: &ValidationFault) -> String {
    match fault {
        ValidationFault::EmptyInput => err_invalid_search_input(locale),
        ValidationFault::TaxonNotFound { input } => err_taxon_not_found(locale, input),
        ValidationFault::TaxonResolutionNoMatch => err_taxon_resolution_failed(locale),
        ValidationFault::UnsupportedFormat { format } => err_unsupported_format(locale, format),
    }
}

fn format_parse_fault(locale: Locale, fault: &ParseFault) -> String {
    match fault {
        ParseFault::TaxonCsv { details } => err_taxon_parse_failed(locale, details),
        ParseFault::TaxonPick { details } => {
            err_query_stage_failed(locale, "taxon resolution", details)
        }
        ParseFault::CountCsv { details } => err_query_stage_failed(locale, "count parse", details),
        ParseFault::DisplayCsv { details } => {
            err_query_stage_failed(locale, "display parse", details)
        }
        ParseFault::FallbackCsv { details } => {
            err_query_stage_failed(locale, "fallback parse", details)
        }
    }
}

/// Format a [`TaxonWarning`] into a locale-appropriate display string.
pub fn format_taxon_warning(locale: Locale, warning: &TaxonWarning) -> String {
    match warning {
        TaxonWarning::Standardized {
            original,
            standardized,
        } => warn_input_standardized(locale, original, standardized),
        TaxonWarning::Ambiguous {
            chosen_name,
            chosen_qid,
            candidates,
        } => warn_ambiguous_taxon(locale, chosen_name, chosen_qid, &candidates.join(", ")),
        TaxonWarning::ApiMessage(msg) => msg.clone(),
    }
}

// ── Components ─────────────────────────────────────────────────────────────────

/// Share URL notice — shows the current shareable URL with a copy button.
///
/// `shareable_url` must be passed in (it is computed in `App` from criteria).
/// Locale is read via `use_locale()` — no locale prop needed.
#[component]
pub fn ShareNotice(shareable_url: Memo<Option<Arc<str>>>) -> Element {
    let locale = crate::hooks::use_locale();
    let share = shareable_url.read();
    let Some(share) = share.as_deref() else {
        return rsx! {};
    };
    rsx! {
        div { class: "notice notice-info", role: "status",
            span { class: "notice-label", "{t(locale, TextKey::Share)}" }
            input {
                class: "notice-value notice-copy-field mono",
                r#type: "text",
                readonly: true,
                value: "{share}",
                aria_label: "{t(locale, TextKey::CopyShareableLink)}",
            }
            CopyButton {
                text: Arc::<str>::from(absolute_share_url(share)),
                title: t(locale, TextKey::CopyShareableLink),
                locale,
            }
        }
    }
}

/// Taxon-resolution warning notice.
///
/// Zero props — reads locale via `use_locale()` and taxon_notice via
/// `ResultsContext`.
#[component]
pub fn TaxonNotice() -> Element {
    let locale = crate::hooks::use_locale();
    let explore = use_results_context().explore;
    let notice = explore.read().result.taxon_notice.clone();
    let Some(warning) = notice.as_ref() else {
        return rsx! {};
    };
    let text = format_taxon_warning(locale, warning);
    rsx! {
        div { class: "notice notice-warn", role: "status",
            span { class: "notice-label", "{t(locale, TextKey::Notice)}" }
            span { class: "notice-value", "{text}" }
        }
    }
}

// ── Error notice ──────────────────────────────────────────────────────────────

/// Error notice with optional retry and dismiss buttons.
///
/// Reads locale via `use_locale()` and lifecycle state via `ResultsContext`.
/// Only action handlers are props — they capture `criteria`, `explore`, `repo`
/// from `App` scope and cannot be replaced by context.
#[component]
pub fn ErrorNotice(on_dismiss: EventHandler<()>, on_retry: EventHandler<()>) -> Element {
    let locale = crate::hooks::use_locale();
    let explore = use_results_context().explore;
    let lifecycle = explore.read().lifecycle.clone();
    let Some(ref domain_err) = lifecycle.error else {
        return rsx! {};
    };
    let kind = domain_err.kind();
    let is_loading = lifecycle.loading;
    let msg = format_domain_error(locale, domain_err);
    rsx! {
        div { class: "notice notice-error", role: "alert",
            span { class: "notice-label", "{t(locale, TextKey::Error)}" }
            span { class: "notice-value", "{msg}" }
            span { class: "notice-value", "{error_hint_text(locale, kind)}" }
            if is_retryable(kind) && !is_loading {
                button {
                    class: "btn btn-sm",
                    r#type: "button",
                    onclick: move |_| on_retry.call(()),
                    "{t(locale, TextKey::Retry)}"
                }
            }
            button {
                class: "notice-dismiss",
                r#type: "button",
                aria_label: "{t(locale, TextKey::DismissError)}",
                onclick: move |_| on_dismiss.call(()),
                "×"
            }
        }
    }
}

// ── Pure helpers ──────────────────────────────────────────────────────────────

pub fn is_retryable(kind: ErrorKind) -> bool {
    matches!(
        kind,
        ErrorKind::Network | ErrorKind::Parse | ErrorKind::Unknown
    )
}

pub fn error_hint_text(locale: Locale, kind: ErrorKind) -> &'static str {
    match kind {
        ErrorKind::Validation => t(locale, TextKey::ErrorHintValidation),
        ErrorKind::Network => t(locale, TextKey::ErrorHintNetwork),
        ErrorKind::Parse => t(locale, TextKey::ErrorHintParse),
        #[cfg(target_arch = "wasm32")]
        ErrorKind::Memory => error_hint_memory(locale),
        ErrorKind::Unknown => t(locale, TextKey::ErrorHintUnknown),
    }
}
