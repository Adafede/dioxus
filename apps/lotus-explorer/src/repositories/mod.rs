// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Repository layer — a thin boundary between the search orchestration and the
//! concrete data-access backends (REST API and direct SPARQL).
//!
//! # Design rationale
//!
//! `orchestrator.rs` previously called `api::search` and
//! `sparql::execute_sparql_bytes` directly, mixing I/O concerns with business
//! logic.  Introducing a trait here gives us:
//!
//! * **Clean boundaries** — orchestration code does not import transport details
//! * **Testability** — unit tests can supply a `MockRepository` without network
//! * **Swappability** — a future ClickHouse or GraphQL backend is a new impl
//!
//! # Trait object vs generics
//!
//! We use `impl LotusRepository` (generics, monomorphised) rather than
//! `dyn LotusRepository` (dynamic dispatch) because:
//!
//! * `async fn` in trait currently requires `dyn`-unsafe workarounds on stable
//! * WASM futures are `!Send`, which would require boxing the returned futures
//! * Monomorphisation gives zero-overhead abstraction at compile time
//!
//! Concrete production code uses [`HybridRepository`], which tries the REST API
//! first (if `api_base` is configured) and falls back to direct SPARQL.

pub mod hybrid;
#[cfg(test)]
pub mod mock;

pub use hybrid::HybridRepository;

use crate::api::SearchResponse;
use crate::models::SearchCriteria;
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum RepositoryError {
    #[error("LOTUS API not configured")]
    NotConfigured,

    #[error("network error: {0}")]
    Network(String),

    #[error("HTTP {status}: {body}")]
    Http { status: u16, body: String },

    #[error("parse error: {0}")]
    Parse(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("unknown error: {message} [{context}]")]
    Unknown { message: String, context: String },
}

impl From<crate::api::ApiClientError> for RepositoryError {
    fn from(value: crate::api::ApiClientError) -> Self {
        match value {
            crate::api::ApiClientError::NotConfigured => Self::NotConfigured,
            crate::api::ApiClientError::Network(msg) => Self::Network(msg),
            crate::api::ApiClientError::Http(status, body) => Self::Http { status, body },
            crate::api::ApiClientError::Parse(msg) => Self::Parse(msg),
        }
    }
}

impl From<crate::core::error::AppError> for RepositoryError {
    fn from(value: crate::core::error::AppError) -> Self {
        let crate::core::error::AppError { kind, context } = value;
        match kind {
            crate::core::error::ErrorKind::Network(msg) => Self::Network(msg.to_string()),
            crate::core::error::ErrorKind::Http { status, message } => Self::Http {
                status,
                body: message.to_string(),
            },
            crate::core::error::ErrorKind::Parse(msg) => Self::Parse(msg.to_string()),
            crate::core::error::ErrorKind::Validation(err) => Self::Validation(err.to_string()),
            crate::core::error::ErrorKind::Unknown(msg) => Self::Unknown {
                message: msg.to_string(),
                context: context.to_string(),
            },
        }
    }
}

/// Boundary trait for data-access operations used by the search orchestrator.
///
/// Implementations may delegate to the REST API, SPARQL, or a test stub.
/// The two async methods cover the only two I/O paths in the orchestrator.
pub trait LotusRepository: Clone + 'static {
    /// Try the REST API fast path.  Returns:
    /// - `None` — API is not configured; caller should fall back to SPARQL
    /// - `Some(Ok(resp))` — successful API response
    /// - `Some(Err(reason))` — API call failed; caller should fall back
    async fn api_search(
        &self,
        criteria: &SearchCriteria,
        limit: usize,
        include_counts: bool,
    ) -> Option<Result<SearchResponse, RepositoryError>>;

    /// Execute a SPARQL query and return raw CSV bytes.
    async fn sparql_bytes(&self, query: &str) -> Result<Vec<u8>, RepositoryError>;
}
