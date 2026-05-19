// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Retryable search orchestration — integration point for error recovery with the base orchestrator.
//!
//! This module demonstrates how to integrate the [`error_recovery_coordinator`] into search
//! orchestration. It provides utilities for determining retry behavior based on error classification.
//!
//! ## Usage
//!
//! After a search error occurs on the ExploreState, callers can:
//! 1. Extract the error from `explore_state.lifecycle.error`
//! 2. Call `classify_error_recovery(&error, attempt_count)` to determine retry strategy
//! 3. If `should_retry` is true, schedule a retry after `backoff_ms`
//! 4. Clear state conditionally based on `should_clear_state_on_error(error.query_stage())`

use crate::features::explore::error_recovery_coordinator::{
    classify_error_recovery, should_clear_state_on_error,
};
use crate::features::explore::types::DomainError;
use std::time::Duration;

/// Utility to compute retry scheduling for a failed search.
///
/// Returns the backoff duration before retry attempt, or None if the error is permanent.
#[allow(dead_code)]
pub fn compute_retry_schedule(
    error: &DomainError,
    attempt_count: u32,
    max_retries: u32,
) -> Option<Duration> {
    if attempt_count >= max_retries {
        return None;
    }

    let recovery = classify_error_recovery(error, attempt_count);
    if recovery.should_retry {
        recovery.backoff_ms.map(Duration::from_millis)
    } else {
        None
    }
}

/// Determines whether to preserve partial results when a search fails.
///
/// Returns `true` if state should be cleared (bad error at early stage),
/// `false` if state should be preserved (e.g., we have previous results to show).
pub fn should_preserve_results_on_error(error: &DomainError) -> bool {
    !should_clear_state_on_error(error.query_stage())
}

/// Utility to summarize retry eligibility for UI feedback.
pub fn retry_eligibility_summary(
    error: &DomainError,
    attempt_count: u32,
    max_retries: u32,
) -> RetryEligibility {
    if attempt_count >= max_retries {
        return RetryEligibility::MaxRetriesExceeded;
    }

    let recovery = classify_error_recovery(error, attempt_count);
    if recovery.should_retry {
        RetryEligibility::Retryable {
            backoff_ms: recovery.backoff_ms,
            next_attempt_number: attempt_count + 1,
        }
    } else {
        RetryEligibility::Permanent
    }
}

/// Summary of whether/how a search error can be retried.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RetryEligibility {
    /// Error is transient and can be retried.
    Retryable {
        backoff_ms: Option<u64>,
        next_attempt_number: u32,
    },
    /// Error is permanent and should not be retried.
    Permanent,
    /// Retry limit has been reached.
    MaxRetriesExceeded,
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::explore::types::{QueryStage, ValidationFault};
    use crate::repositories::RepositoryError;

    #[test]
    fn compute_retry_schedule_returns_none_at_max_retries() {
        let error = DomainError::Validation(ValidationFault::EmptyInput);
        let schedule = compute_retry_schedule(&error, 3, 3);
        assert_eq!(schedule, None);
    }

    #[test]
    fn compute_retry_schedule_returns_none_for_permanent_errors() {
        let error = DomainError::Validation(ValidationFault::EmptyInput);
        let schedule = compute_retry_schedule(&error, 0, 10);
        assert_eq!(schedule, None);
    }

    #[test]
    fn compute_retry_schedule_returns_duration_for_transient_errors() {
        let error = DomainError::Transport {
            stage: QueryStage::ResultsQuery,
            source: RepositoryError::network("connection refused"),
        };
        let schedule = compute_retry_schedule(&error, 0, 3);
        assert!(schedule.is_some());
        assert_eq!(schedule, Some(Duration::from_millis(200)));
    }

    #[test]
    fn should_preserve_results_on_results_query_error() {
        let error = DomainError::Transport {
            stage: QueryStage::ResultsQuery,
            source: RepositoryError::network("timeout"),
        };
        assert!(should_preserve_results_on_error(&error));
    }

    #[test]
    fn should_not_preserve_results_on_taxon_search_error() {
        let error = DomainError::Transport {
            stage: QueryStage::TaxonSearch,
            source: RepositoryError::network("timeout"),
        };
        assert!(!should_preserve_results_on_error(&error));
    }

    #[test]
    fn retry_eligibility_summary_for_transient_error() {
        let error = DomainError::Transport {
            stage: QueryStage::ResultsQuery,
            source: RepositoryError::network("connection reset"),
        };
        let summary = retry_eligibility_summary(&error, 0, 3);
        match summary {
            RetryEligibility::Retryable {
                backoff_ms,
                next_attempt_number,
            } => {
                assert_eq!(backoff_ms, Some(200));
                assert_eq!(next_attempt_number, 1);
            }
            _ => panic!("expected retryable error"),
        }
    }

    #[test]
    fn retry_eligibility_summary_for_permanent_error() {
        let error = DomainError::Validation(ValidationFault::TaxonTooLong);
        let summary = retry_eligibility_summary(&error, 0, 3);
        assert_eq!(summary, RetryEligibility::Permanent);
    }

    #[test]
    fn retry_eligibility_summary_max_retries_exceeded() {
        let error = DomainError::Transport {
            stage: QueryStage::ResultsQuery,
            source: RepositoryError::network("timeout"),
        };
        let summary = retry_eligibility_summary(&error, 3, 3);
        assert_eq!(summary, RetryEligibility::MaxRetriesExceeded);
    }
}
