// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! [`HybridRepository`] — the production `LotusRepository` implementation.
//!
//! Strategy:
//! 1. If a REST API base URL is configured, try `api::search` first
//!    (gives exact counts + query in one round-trip).
//! 2. On API error or when not configured, return `None` / `Some(Err(…))`
//!    so the caller falls back to direct SPARQL execution.

use crate::api;
use crate::api::SearchResponse;
use crate::models::SearchCriteria;
use crate::repositories::{LotusRepository, RepositoryError};
use crate::sparql;
use shared::sparql::FetchError;

/// Zero-size, `Copy` production repository.
///
/// Holds no state of its own; all configuration is read from environment and
/// runtime globals (`api_base_url`, `sparql::execute_sparql_bytes`, etc.).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HybridRepository;

impl LotusRepository for HybridRepository {
    async fn api_search(
        &self,
        criteria: &SearchCriteria,
        limit: usize,
        include_counts: bool,
    ) -> Option<Result<SearchResponse, RepositoryError>> {
        if api::api_base_url().is_none() {
            return Some(Err(RepositoryError::NotConfigured));
        }
        // Call the transport client directly, mapping ApiClientError → RepositoryError
        // via the existing `From` implementation.  Bypassing the ApiLayer / AppError
        // intermediary eliminates a 4-hop conversion chain with no semantic benefit.
        Some(
            api::search(criteria, limit, include_counts)
                .await
                .map_err(RepositoryError::from),
        )
    }

    async fn sparql_bytes(&self, query: &str) -> Result<Vec<u8>, RepositoryError> {
        sparql::execute_sparql_bytes(query)
            .await
            .map_err(map_fetch_error)
    }

    async fn sparql_body(
        &self,
        query: &str,
    ) -> Result<shared::sparql::ResponseBody, RepositoryError> {
        sparql::execute_sparql_body(query)
            .await
            .map_err(map_fetch_error)
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn sparql_tempfile(
        &self,
        query: &str,
    ) -> Result<tempfile::NamedTempFile, RepositoryError> {
        sparql::execute_sparql_tempfile(query)
            .await
            .map_err(map_fetch_error)
    }
}

fn map_fetch_error(err: FetchError) -> RepositoryError {
    match err {
        FetchError::Http(status, body) => RepositoryError::Http { status, body },
        FetchError::Network(msg) => RepositoryError::network(msg),
        FetchError::Parse(msg) => RepositoryError::parse(msg),
        FetchError::Empty => RepositoryError::parse("query returned no results"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_fetch_error_preserves_http_status_and_body() {
        let mapped = map_fetch_error(FetchError::Http(400, "invalid query".to_string()));
        assert_eq!(
            mapped,
            RepositoryError::Http {
                status: 400,
                body: "invalid query".to_string(),
            }
        );
    }

    #[test]
    fn map_fetch_error_keeps_network_as_network() {
        let mapped = map_fetch_error(FetchError::Network("timeout".to_string()));
        assert!(matches!(mapped, RepositoryError::Network(_)));
    }
}
