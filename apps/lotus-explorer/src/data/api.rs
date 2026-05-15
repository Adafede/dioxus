// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Data access layer — reserved for future API-level concerns.
//!
//! ## History / Design Decision
//!
//! This module previously contained `ApiLayer`, a zero-size struct that added
//! a thin validation wrapper around `api::client::search`.  That abstraction
//! was removed because:
//!
//! * **Input validation already happens upstream**: `orchestrator::validate_search_criteria`
//!   runs the `EmptyInput` check before any I/O is attempted.  Duplicating it
//!   here forced callers through a 4-hop error conversion:
//!   `ApiClientError → AppError → RepositoryError → DomainError`.
//!
//! * **`HybridRepository::api_search`** now calls `api::search` directly, mapping
//!   `ApiClientError → RepositoryError` via the existing `From` implementation.
//!   This keeps the conversion chain at 2 hops and eliminates the intermediate
//!   `AppError` / `core::error::ErrorKind` types.
//!
//! ## Future Extension Points
//!
//! When genuine API-layer concerns arise (request deduplication, caching,
//! circuit breaking, rate limiting), re-introduce them here as cohesive
//! features rather than adding another conversion layer.
