// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! User-facing formatting for domain errors and warnings.

use crate::features::explore::types::{
    DomainError, ErrorKind, ParseFault, QueryStage, TaxonWarning, ValidationFault,
};
#[cfg(target_arch = "wasm32")]
use crate::i18n::error_hint_memory;
use crate::i18n::{
    Locale, TextKey, err_invalid_search_input, err_query_stage_failed, err_taxon_not_found,
    err_taxon_parse_failed, err_taxon_resolution_failed, err_unsupported_format, t,
    warn_ambiguous_taxon, warn_input_standardized,
};
use crate::repositories::RepositoryError;

pub fn format_domain_error(locale: Locale, err: &DomainError) -> String {
    match err {
        DomainError::Validation(v) => format_validation_fault(locale, v),
        DomainError::Transport { stage, source } => format_transport_fault(locale, *stage, source),
        DomainError::Parse(p) => format_parse_fault(locale, p),
        #[cfg(target_arch = "wasm32")]
        DomainError::MemoryLimit { .. } => error_hint_memory(locale).to_string(),
    }
}

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

fn format_transport_fault(locale: Locale, stage: QueryStage, source: &RepositoryError) -> String {
    let detail = transport_error_summary(source);
    let stage_label = stage_display_label(locale, stage);
    err_query_stage_failed(locale, stage_label, &detail)
}

fn stage_display_label(locale: Locale, stage: QueryStage) -> &'static str {
    match stage {
        QueryStage::TaxonSearch => t(locale, TextKey::StageTaxonSearch),
        QueryStage::CountQuery => t(locale, TextKey::StageCountQuery),
        QueryStage::DisplayQuery => t(locale, TextKey::StageDisplayQuery),
        QueryStage::FallbackQuery => t(locale, TextKey::StageFallbackQuery),
        #[cfg(target_arch = "wasm32")]
        QueryStage::CountAndPreview => t(locale, TextKey::StageDisplayQuery),
    }
}

fn transport_error_summary(source: &RepositoryError) -> String {
    let raw = match source {
        RepositoryError::NotConfigured => return "LOTUS API not configured".to_string(),
        RepositoryError::Network(detail) => detail.as_str(),
        RepositoryError::Parse(detail) => detail.as_str(),
        RepositoryError::Http { status, body } => {
            return format!("HTTP {status}: {}", compact_error_text(body));
        }
    };
    compact_error_text(raw)
}

fn compact_error_text(msg: &str) -> String {
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
            stage_display_label(locale, QueryStage::TaxonSearch),
            &compact_error_text(details),
        ),
        ParseFault::CountCsv { details } => err_query_stage_failed(
            locale,
            stage_display_label(locale, QueryStage::CountQuery),
            &compact_error_text(details),
        ),
        ParseFault::DisplayCsv { details } => err_query_stage_failed(
            locale,
            stage_display_label(locale, QueryStage::DisplayQuery),
            &compact_error_text(details),
        ),
        ParseFault::FallbackCsv { details } => err_query_stage_failed(
            locale,
            stage_display_label(locale, QueryStage::FallbackQuery),
            &compact_error_text(details),
        ),
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
        let summary = transport_error_summary(&RepositoryError::network(long));
        assert!(summary.chars().count() <= 221);
        assert!(summary.ends_with('…'));
    }
}
