// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Error recovery strategies and stale state handling patterns.
//!
//! This module codifies best practices for distinguishing recoverable vs fatal errors,
//! determining retry strategies, and clearing stale state during error conditions.
//!
//! ## Philosophy
//!
//! - **Transient vs Permanent**: Network timeouts are recoverable; validation errors are not.
//! - **Retry Policies**: Classified by error type; some errors should never retry.
//! - **State Cleanup**: Clear partial/stale results on error to prevent UI inconsistencies.
//! - **User Communication**: Categorize errors so UI can format them localization-aware.

use crate::features::explore::types::{DomainError, QueryStage};
use crate::repositories::RepositoryError;

// ── Error Classification ──────────────────────────────────────────────────────

/// Determines whether an error is recoverable and worth retrying.
#[must_use]
#[allow(dead_code)] // Public API for future error recovery wiring
pub fn is_retryable_error(error: &DomainError) -> bool {
    match error {
        // Validation errors should NEVER retry — user input is wrong.
        DomainError::Validation(_) => false,

        // Network errors MAY be transient — connection issues, timeouts, 502s.
        DomainError::Transport { source, .. } => is_retryable_transport_error(source),

        // Parse errors indicate data corruption or format change — don't retry.
        DomainError::Parse(_) => false,

        // Memory limits are usually permanent in WASM — don't retry.
        #[cfg(target_arch = "wasm32")]
        DomainError::MemoryLimit { .. } => false,
    }
}

/// Determines whether a repository/network error is transient and worth retrying.
#[must_use]
#[allow(dead_code)] // Public API for future error recovery wiring
pub fn is_retryable_transport_error(error: &RepositoryError) -> bool {
    match error {
        // API not configured: permanent.
        RepositoryError::NotConfigured => false,

        // Network errors: transient (connection refused, timeout, DNS failure).
        RepositoryError::Network(_) => true,

        // HTTP 5xx: transient (server error, may recover).
        // HTTP 4xx: permanent (client error, won't recover).
        RepositoryError::Http { status, .. } => *status >= 500,

        // Parse errors: permanent (data format issue).
        RepositoryError::Parse(_) => false,
    }
}

// ── Retry Timing Strategy ─────────────────────────────────────────────────────

/// Compute exponential backoff for a retry attempt.
///
/// ```ignore
/// let delay_ms = backoff_delay_ms(attempt: 0);  // ~100ms
/// let delay_ms = backoff_delay_ms(attempt: 1);  // ~200ms
/// let delay_ms = backoff_delay_ms(attempt: 2);  // ~400ms
/// ```
#[must_use]
#[allow(dead_code)] // Public API for future retry orchestration
pub fn backoff_delay_ms(attempt: u32) -> u64 {
    // Base 100ms, exponential 2^attempt, capped at 10s (max 10 attempts)
    let base = 100u64;
    let exponent = (attempt as u64).min(7); // Cap at 2^7 = 128
    (base * 2u64.pow(exponent as u32)).min(10_000)
}

// ── Error Stage Analysis ──────────────────────────────────────────────────────

/// Determine whether partial results should be cleared after error at given stage.
///
/// For example: if taxon resolution fails, results from previous searches are stale.
/// But if we fail during the display query, earlier counts are still valid.
#[must_use]
#[allow(dead_code)] // Public API for future state-cleanup orchestration
pub fn should_clear_state_on_error(error_stage: QueryStage) -> bool {
    match error_stage {
        // Taxon resolution failed — everything downstream is invalid.
        QueryStage::TaxonSearch => true,

        // Count failed — counts are invalid, but we can still use older results.
        QueryStage::CountQuery => false,

        // Display query failed — counts are valid, just no preview rows.
        QueryStage::DisplayQuery => false,

        // Fallback query failed — we tried to recover and failed.
        QueryStage::FallbackQuery => false,

        // Neither count nor preview succeeded.
        #[cfg(target_arch = "wasm32")]
        QueryStage::CountAndPreview => false,
    }
}

// ── User-Facing Recovery UI ───────────────────────────────────────────────────

/// Determine whether a "Retry" button should be shown for this error.
///
/// - Non-retryable errors (validation, parse): only "Dismiss"
/// - Retryable errors (network): both "Retry" and "Dismiss"
#[must_use]
#[allow(dead_code)] // Public API for future UI error handling improvements
pub fn should_show_retry_button(error: &DomainError) -> bool {
    is_retryable_error(error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_errors_not_retryable() {
        use crate::features::explore::types::ValidationFault;
        let err = DomainError::Validation(ValidationFault::EmptyInput);
        assert!(!is_retryable_error(&err));
    }

    #[test]
    fn network_errors_retryable() {
        let err = DomainError::Transport {
            stage: QueryStage::CountQuery,
            source: RepositoryError::network("timeout"),
        };
        assert!(is_retryable_error(&err));
    }

    #[test]
    fn http_4xx_not_retryable() {
        let err = DomainError::Transport {
            stage: QueryStage::DisplayQuery,
            source: RepositoryError::Http {
                status: 400,
                body: "bad request".into(),
            },
        };
        assert!(!is_retryable_error(&err));
    }

    #[test]
    fn http_5xx_retryable() {
        let err = DomainError::Transport {
            stage: QueryStage::DisplayQuery,
            source: RepositoryError::Http {
                status: 502,
                body: "bad gateway".into(),
            },
        };
        assert!(is_retryable_error(&err));
    }

    #[test]
    fn backoff_delay_grows_exponentially() {
        assert!(backoff_delay_ms(0) < backoff_delay_ms(1));
        assert!(backoff_delay_ms(1) < backoff_delay_ms(2));
        assert!(backoff_delay_ms(7) == backoff_delay_ms(8)); // Capped
    }

    #[test]
    fn backoff_delay_capped_at_10_seconds() {
        assert!(backoff_delay_ms(10) <= 10_000);
    }

    #[test]
    fn taxon_search_error_clears_state() {
        assert!(should_clear_state_on_error(QueryStage::TaxonSearch));
    }

    #[test]
    fn count_query_error_keeps_previous_results() {
        assert!(!should_clear_state_on_error(QueryStage::CountQuery));
    }

    #[test]
    fn display_query_error_keeps_previous_results() {
        assert!(!should_clear_state_on_error(QueryStage::DisplayQuery));
    }

    #[test]
    fn retry_button_shown_only_for_retryable_errors() {
        use crate::features::explore::types::ValidationFault;
        let validation_err = DomainError::Validation(ValidationFault::EmptyInput);
        assert!(!should_show_retry_button(&validation_err));

        let network_err = DomainError::Transport {
            stage: QueryStage::CountQuery,
            source: RepositoryError::network("timeout"),
        };
        assert!(should_show_retry_button(&network_err));
    }
}
