// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Data access layer for API, including HTTP client abstraction and DTO transformations.

use crate::api::SearchResponse;
use crate::core::error::{AppError, ValidationError};
use crate::models::SearchCriteria;

/// High-level API data access layer that provides structured query building
/// and transformation with minimal transport concerns leaking into domain logic.
///
/// This layer sits between the domain (orchestrator, services) and the
/// concrete transport (HTTP client, SPARQL executor), providing:
/// - Unified error handling
/// - Validation at boundaries
/// - Future optimization points (caching, retry, rate limiting)
pub struct ApiLayer {
    // Future: Add caching, circuit breaker, etc. here
}

impl ApiLayer {
    pub fn new() -> Self {
        Self {}
    }

    /// Execute a search with proper error context and recovery hints.
    ///
    /// This is the primary entry point for search operations. It ensures
    /// that all errors carry context about what was happening during the
    /// failure, enabling better error recovery and user-facing messages.
    pub async fn search(
        &self,
        criteria: &SearchCriteria,
        limit: usize,
        include_counts: bool,
    ) -> Result<SearchResponse, AppError> {
        // Validate at boundary
        if !criteria.is_valid() {
            return Err(AppError::validation(
                ValidationError::EmptyInput,
                "validating search input",
            ));
        }

        // Call the underlying API with error bridging
        crate::api::search(criteria, limit, include_counts)
            .await
            .map_err(AppError::from)
    }

    /// Get the export URLs for a set of criteria.
    /// (WASM only - available when target_arch is wasm32)
    #[cfg(target_arch = "wasm32")]
    pub async fn export_urls(
        &self,
        criteria: &SearchCriteria,
    ) -> Result<crate::api::ExportUrlResponse, AppError> {
        if !criteria.is_valid() {
            return Err(AppError::validation(
                ValidationError::EmptyInput,
                "validating export request",
            ));
        }

        crate::api::export_urls(criteria)
            .await
            .map_err(AppError::from)
    }
}

impl Default for ApiLayer {
    fn default() -> Self {
        Self::new()
    }
}

/// Bridge from AppError back to legacy DomainError for backward compatibility.
/// This allows gradual migration without breaking existing code.
impl From<AppError> for crate::features::explore::types::DomainError {
    fn from(err: AppError) -> Self {
        use crate::features::explore::types::{DomainError, ValidationFault};

        match err.kind {
            crate::core::error::ErrorKind::Validation(
                crate::core::error::ValidationError::EmptyInput,
            ) => DomainError::Validation(ValidationFault::EmptyInput),
            crate::core::error::ErrorKind::Network(msg) => DomainError::Transport {
                stage: "network",
                source: crate::repositories::RepositoryError::Network(msg.to_string()),
            },
            crate::core::error::ErrorKind::Http { status, message } => DomainError::Transport {
                stage: "http",
                source: crate::repositories::RepositoryError::Http {
                    status,
                    body: message.to_string(),
                },
            },
            crate::core::error::ErrorKind::Parse(msg) => {
                DomainError::Parse(crate::features::explore::types::ParseFault::DisplayCsv {
                    details: msg.to_string(),
                })
            }
            _ => DomainError::Transport {
                stage: "unknown",
                source: crate::repositories::RepositoryError::Other(err.to_string()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_layer_validates_empty_criteria() {
        let api = ApiLayer::new();
        let invalid = SearchCriteria {
            taxon: "".into(),
            ..SearchCriteria::default()
        };

        // Use futures executor to run async code in unit tests without tokio.
        let result = futures::executor::block_on(async { api.search(&invalid, 100, true).await });

        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.context.as_ref(), "validating search input");
        }
    }
}
