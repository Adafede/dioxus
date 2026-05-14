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
use crate::i18n::error_hint_memory;
use crate::i18n::{
    Locale, TextKey, err_invalid_search_input, err_query_stage_failed, err_taxon_not_found,
    err_taxon_parse_failed, err_taxon_resolution_failed, err_unsupported_format, t,
    warn_ambiguous_taxon, warn_input_standardized,
};
use crate::repositories::RepositoryError;
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
        DomainError::Transport { stage, source } => format_transport_fault(locale, stage, source),
        DomainError::Parse(p) => format_parse_fault(locale, p),
        #[cfg(target_arch = "wasm32")]
        DomainError::MemoryLimit { .. } => error_hint_memory(locale).to_string(),
    }
}

fn format_transport_fault(locale: Locale, stage: &str, source: &RepositoryError) -> String {
    let detail = transport_error_summary(source);
    let stage_label = stage_display_label(locale, stage);
    err_query_stage_failed(locale, &stage_label, &detail)
}

fn stage_display_label(locale: Locale, stage: &str) -> String {
    match (locale, stage) {
        (Locale::En, "taxon_search") => "taxon lookup".to_string(),
        (Locale::En, "count query") => "result counting".to_string(),
        (Locale::En, "display query") => "preview fetch".to_string(),
        (Locale::En, "fallback query") => "fallback fetch".to_string(),

        (Locale::Fr, "taxon_search") => "resolution du taxon".to_string(),
        (Locale::Fr, "count query") => "comptage des resultats".to_string(),
        (Locale::Fr, "display query") => "recuperation de l'apercu".to_string(),
        (Locale::Fr, "fallback query") => "recuperation de secours".to_string(),

        (Locale::De, "taxon_search") => "Taxon-Auflosung".to_string(),
        (Locale::De, "count query") => "Ergebniszahlung".to_string(),
        (Locale::De, "display query") => "Vorschauabruf".to_string(),
        (Locale::De, "fallback query") => "Fallback-Abruf".to_string(),

        (Locale::It, "taxon_search") => "risoluzione del taxon".to_string(),
        (Locale::It, "count query") => "conteggio risultati".to_string(),
        (Locale::It, "display query") => "recupero anteprima".to_string(),
        (Locale::It, "fallback query") => "recupero di fallback".to_string(),

        _ => stage.to_string(),
    }
}

fn transport_error_summary(source: &RepositoryError) -> String {
    let raw = match source {
        RepositoryError::Network(msg)
        | RepositoryError::Parse(msg)
        | RepositoryError::Other(msg) => msg.as_str(),
        RepositoryError::Http { status, body } => {
            return format!("HTTP {status}: {}", compact_error_text(body));
        }
    };
    compact_error_text(raw)
}

fn compact_error_text(msg: &str) -> String {
    // Many backend failures return a huge JSON payload with `exception`, `query`,
    // and runtime details. Keep only the exception text for user-facing notices.
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(msg)
        && let Some(exception) = value.get("exception").and_then(|v| v.as_str())
    {
        return truncate_for_notice(exception);
    }

    let trimmed = msg.lines().next().unwrap_or(msg).trim();
    truncate_for_notice(trimmed)
}

fn truncate_for_notice(text: &str) -> String {
    const MAX_CHARS: usize = 220;
    if text.chars().count() <= MAX_CHARS {
        return text.to_string();
    }
    let mut out = text.chars().take(MAX_CHARS).collect::<String>();
    out.push('…');
    out
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
        ParseFault::TaxonCsv { details } => {
            err_taxon_parse_failed(locale, &compact_error_text(details))
        }
        ParseFault::TaxonPick { details } => err_query_stage_failed(
            locale,
            &stage_display_label(locale, "taxon_search"),
            &compact_error_text(details),
        ),
        ParseFault::CountCsv { details } => err_query_stage_failed(
            locale,
            &stage_display_label(locale, "count query"),
            &compact_error_text(details),
        ),
        ParseFault::DisplayCsv { details } => err_query_stage_failed(
            locale,
            &stage_display_label(locale, "display query"),
            &compact_error_text(details),
        ),
        ParseFault::FallbackCsv { details } => err_query_stage_failed(
            locale,
            &stage_display_label(locale, "fallback query"),
            &compact_error_text(details),
        ),
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
        ErrorKind::Memory => "",
        ErrorKind::Unknown => t(locale, TextKey::ErrorHintUnknown),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_error_text_uses_exception_from_json() {
        let payload = r#"{"exception":"Upstream service returned HTTP 500","query":"SELECT ..."}"#;
        assert_eq!(
            compact_error_text(payload),
            "Upstream service returned HTTP 500"
        );
    }

    #[test]
    fn transport_error_summary_truncates_long_network_message() {
        let long = "x".repeat(400);
        let summary = transport_error_summary(&RepositoryError::Network(long));
        assert!(summary.chars().count() <= 221);
        assert!(summary.ends_with('…'));
    }
}
