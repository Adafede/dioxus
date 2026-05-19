// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! SPARQL endpoint error classification and recovery heuristics.
//!
//! QLever and other SPARQL endpoints may return errors that are transient
//! (cache invalidation, server hiccups) or permanent (bad query structure).

#![allow(dead_code)] // Public API for error recovery integration

use serde_json::Value;

/// Classify a SPARQL error response to determine retryability.
pub fn classify_sparql_exception(json: &Value) -> SparqlErrorClass {
    let exception = json.get("exception").and_then(|e| e.as_str()).unwrap_or("");

    if exception.contains("cache key") {
        return SparqlErrorClass::CacheConflict;
    }
    if exception.contains("timeout") || exception.contains("Too many") {
        return SparqlErrorClass::RateLimit;
    }
    if exception.contains("syntax") || exception.contains("grammar") {
        return SparqlErrorClass::QuerySyntax;
    }
    if exception.contains("no results") {
        return SparqlErrorClass::NoResults;
    }
    SparqlErrorClass::Unknown
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SparqlErrorClass {
    /// Cache key conflict on upstream endpoint (transient, retry-safe).
    CacheConflict,
    /// Rate limit or query queue full (transient, backoff recommended).
    RateLimit,
    /// Query syntax error (permanent, do not retry).
    QuerySyntax,
    /// No results for valid query (not an error, expected).
    NoResults,
    /// Unclassified error.
    Unknown,
}

impl SparqlErrorClass {
    pub fn is_retryable(self) -> bool {
        matches!(self, Self::CacheConflict | Self::RateLimit | Self::Unknown)
    }

    pub fn should_backoff(self) -> bool {
        matches!(self, Self::RateLimit)
    }
}
