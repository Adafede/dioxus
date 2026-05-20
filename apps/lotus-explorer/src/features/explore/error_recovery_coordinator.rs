// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Error recovery coordination — intelligent retry decisions based on error classification.
//!
//! This module provides pure decision logic for determining whether a search should retry,
//! when, and with what backoff strategy. It uses SPARQL error classification to distinguish
//! transient upstream cache conflicts from permanent errors.

use crate::features::explore::transport_classification::{
    TransportFailureKind, classify_transport_error,
};
use crate::features::explore::types::{DomainError, QueryStage};
use crate::repositories::RepositoryError;

/// Encapsulates retry decision-making for a failed search operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ErrorRecoveryDecision {
    /// Whether the search should be retried.
    pub should_retry: bool,
    /// If retrying, how long to wait (milliseconds) before attempting.
    pub backoff_ms: Option<u64>,
    /// Short classification of the error for telemetry/logging.
    pub error_class: ErrorClass,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorClass {
    /// Validation error (user input is wrong, don't retry).
    Validation,
    /// Environment/application configuration problem.
    Configuration,
    /// Network timeout or connection error (transient, retry with backoff).
    Network,
    /// HTTP 5xx or comparable upstream failure.
    Server,
    /// Upstream SPARQL cache conflict (transient, retry immediately).
    CacheConflict,
    /// Rate limit or query queue full (transient, retry with backoff).
    RateLimit,
    /// HTTP 4xx request rejected for non-syntax reasons.
    BadRequest,
    /// Query syntax error (permanent, don't retry).
    QuerySyntax,
    /// Parse error (permanent, don't retry).
    Parse,
    /// Memory pressure in the browser runtime.
    #[cfg(target_arch = "wasm32")]
    Memory,
}

impl ErrorClass {
    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::Validation => "validation",
            Self::Configuration => "configuration",
            Self::Network => "network",
            Self::Server => "server",
            Self::CacheConflict => "cache_conflict",
            Self::RateLimit => "rate_limit",
            Self::BadRequest => "bad_request",
            Self::QuerySyntax => "query_syntax",
            Self::Parse => "parse",
            #[cfg(target_arch = "wasm32")]
            Self::Memory => "memory",
        }
    }
}

/// Determine retry strategy for a failed search given the error and attempt count.
///
/// # Arguments
/// * `error` — the domain error that caused the search to fail
/// * `attempt` — which retry attempt this is (0 = first attempt, 1 = first retry, etc.)
///
/// # Returns
/// A decision indicating whether to retry, with backoff timing if applicable.
pub fn classify_error_recovery(error: &DomainError, attempt: u32) -> ErrorRecoveryDecision {
    match error {
        DomainError::Validation(_) => ErrorRecoveryDecision {
            should_retry: false,
            backoff_ms: None,
            error_class: ErrorClass::Validation,
        },

        DomainError::Parse(_) => ErrorRecoveryDecision {
            should_retry: false,
            backoff_ms: None,
            error_class: ErrorClass::Parse,
        },

        DomainError::Transport { source, .. } => classify_transport_error_recovery(source, attempt),

        #[cfg(target_arch = "wasm32")]
        DomainError::MemoryLimit { .. } => ErrorRecoveryDecision {
            should_retry: false,
            backoff_ms: None,
            error_class: ErrorClass::Memory,
        },
    }
}

/// Classify a transport-layer error and determine retry strategy.
fn classify_transport_error_recovery(
    repo_error: &RepositoryError,
    attempt: u32,
) -> ErrorRecoveryDecision {
    match classify_transport_error(repo_error) {
        TransportFailureKind::Configuration => ErrorRecoveryDecision {
            should_retry: false,
            backoff_ms: None,
            error_class: ErrorClass::Configuration,
        },
        TransportFailureKind::Network => ErrorRecoveryDecision {
            should_retry: true,
            backoff_ms: Some(backoff_delay_ms(attempt)),
            error_class: ErrorClass::Network,
        },

        TransportFailureKind::Server => ErrorRecoveryDecision {
            should_retry: true,
            backoff_ms: Some(backoff_delay_ms(attempt)),
            error_class: ErrorClass::Server,
        },

        TransportFailureKind::CacheConflict => ErrorRecoveryDecision {
            should_retry: true,
            backoff_ms: Some(100),
            error_class: ErrorClass::CacheConflict,
        },

        TransportFailureKind::RateLimit => ErrorRecoveryDecision {
            should_retry: true,
            backoff_ms: Some(backoff_delay_ms(attempt)),
            error_class: ErrorClass::RateLimit,
        },

        TransportFailureKind::BadRequest => ErrorRecoveryDecision {
            should_retry: false,
            backoff_ms: None,
            error_class: ErrorClass::BadRequest,
        },

        TransportFailureKind::QuerySyntax => ErrorRecoveryDecision {
            should_retry: false,
            backoff_ms: None,
            error_class: ErrorClass::QuerySyntax,
        },

        TransportFailureKind::Parse => ErrorRecoveryDecision {
            should_retry: false,
            backoff_ms: None,
            error_class: ErrorClass::Parse,
        },
    }
}

/// Compute exponential backoff for retry attempt.
///
/// Base 100ms, exp 2^(attempt+1), capped at 10s.
fn backoff_delay_ms(attempt: u32) -> u64 {
    let base = 100u64;
    let exponent = ((attempt + 1) as u64).min(7); // Cap at 2^7 = 128
    (base * 2u64.pow(exponent as u32)).min(10_000)
}

/// Determine whether partial results should be cleared after error at given stage.
pub fn should_clear_state_on_error(error_stage: QueryStage) -> bool {
    match error_stage {
        QueryStage::TaxonSearch => true, // Everything downstream is invalid
        QueryStage::ResultsQuery => false, // Keep previous results visible
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::explore::types::ValidationFault;

    #[test]
    fn validation_error_does_not_retry() {
        let err = DomainError::Validation(ValidationFault::EmptyInput);
        let decision = classify_error_recovery(&err, 0);
        assert!(!decision.should_retry);
        assert_eq!(decision.error_class, ErrorClass::Validation);
    }

    #[test]
    fn network_error_retries_with_backoff() {
        let err = DomainError::Transport {
            stage: QueryStage::ResultsQuery,
            source: RepositoryError::network("connection refused"),
        };
        let decision = classify_error_recovery(&err, 0);
        assert!(decision.should_retry);
        assert_eq!(decision.backoff_ms, Some(200)); // 100 * 2^1
        assert_eq!(decision.error_class, ErrorClass::Network);
    }

    #[test]
    fn cache_conflict_retries_immediately() {
        let err = DomainError::Transport {
            stage: QueryStage::ResultsQuery,
            source: RepositoryError::parse(
                "Trying to insert a cache key which was already present",
            ),
        };
        let decision = classify_error_recovery(&err, 0);
        assert!(decision.should_retry);
        assert_eq!(decision.backoff_ms, Some(100)); // Immediate retry
        assert_eq!(decision.error_class, ErrorClass::CacheConflict);
    }

    #[test]
    fn rate_limit_error_retries_with_backoff() {
        let err = DomainError::Transport {
            stage: QueryStage::ResultsQuery,
            source: RepositoryError::parse("Too many queries in queue"),
        };
        let decision = classify_error_recovery(&err, 1);
        assert!(decision.should_retry);
        assert_eq!(decision.backoff_ms, Some(400)); // 100 * 2^2
        assert_eq!(decision.error_class, ErrorClass::RateLimit);
    }

    #[test]
    fn syntax_error_is_permanent() {
        let err = DomainError::Transport {
            stage: QueryStage::ResultsQuery,
            source: RepositoryError::parse("SPARQL syntax error near WHERE"),
        };
        let decision = classify_error_recovery(&err, 0);
        assert!(!decision.should_retry);
        assert_eq!(decision.error_class, ErrorClass::QuerySyntax);
    }

    #[test]
    fn http_syntax_error_is_classified_without_retry() {
        let err = DomainError::Transport {
            stage: QueryStage::ResultsQuery,
            source: RepositoryError::Http {
                status: 400,
                body: "Invalid SPARQL query: mismatched input 'AS' expecting ','".into(),
            },
        };
        let decision = classify_error_recovery(&err, 0);
        assert!(!decision.should_retry);
        assert_eq!(decision.error_class, ErrorClass::QuerySyntax);
    }

    #[test]
    fn http_5xx_is_retryable_server_error() {
        let err = DomainError::Transport {
            stage: QueryStage::ResultsQuery,
            source: RepositoryError::Http {
                status: 503,
                body: "temporary upstream failure".into(),
            },
        };
        let decision = classify_error_recovery(&err, 0);
        assert!(decision.should_retry);
        assert_eq!(decision.error_class, ErrorClass::Server);
        assert_eq!(decision.backoff_ms, Some(200));
    }

    #[test]
    fn not_configured_is_a_configuration_error() {
        let err = DomainError::Transport {
            stage: QueryStage::ResultsQuery,
            source: RepositoryError::NotConfigured,
        };
        let decision = classify_error_recovery(&err, 0);
        assert!(!decision.should_retry);
        assert_eq!(decision.error_class, ErrorClass::Configuration);
    }

    #[test]
    fn backoff_strategy_grows_exponentially_capped() {
        assert_eq!(backoff_delay_ms(0), 200);
        assert_eq!(backoff_delay_ms(1), 400);
        assert_eq!(backoff_delay_ms(2), 800);
        assert_eq!(backoff_delay_ms(6), 10_000); // Capped at 10s (100 * 2^7 would be 12_800)
        assert_eq!(backoff_delay_ms(7), 10_000); // Still capped
        assert_eq!(backoff_delay_ms(10), 10_000); // Still capped
    }

    #[test]
    fn should_clear_state_differs_by_stage() {
        assert!(should_clear_state_on_error(QueryStage::TaxonSearch));
        assert!(!should_clear_state_on_error(QueryStage::ResultsQuery));
    }
}
