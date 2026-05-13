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
use crate::repositories::LotusRepository;
use crate::sparql;

/// Zero-size, `Copy` production repository.
///
/// Holds no state of its own; all configuration is read from environment and
/// runtime globals (`api_base_url`, `sparql::execute_sparql_bytes`, etc.).
#[derive(Clone, Copy, Default)]
pub struct HybridRepository;

impl HybridRepository {
    pub fn new() -> Self {
        Self
    }
}

impl LotusRepository for HybridRepository {
    async fn api_search(
        &self,
        criteria: &SearchCriteria,
        limit: usize,
        include_counts: bool,
    ) -> Option<Result<SearchResponse, String>> {
        // No API base → caller skips the API path entirely.
        api::api_base_url()?;
        Some(
            api::search(criteria, limit, include_counts)
                .await
                .map_err(|e| e.to_string()),
        )
    }

    async fn sparql_bytes(&self, query: &str) -> Result<Vec<u8>, String> {
        sparql::execute_sparql_bytes(query)
            .await
            .map_err(|e| e.to_string())
    }
}
