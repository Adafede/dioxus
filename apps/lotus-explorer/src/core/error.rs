// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Runtime error model shared across data and repository layers.

use std::sync::Arc;
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum ErrorKind {
    #[error("Network error: {0}")]
    Network(Arc<str>),

    #[error("HTTP {status}: {message}")]
    Http { status: u16, message: Arc<str> },

    #[error("Parse error: {0}")]
    Parse(Arc<str>),

    #[error("Validation error: {0}")]
    Validation(ValidationError),

    #[error("Unknown error: {0}")]
    Unknown(Arc<str>),
}

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum ValidationError {
    #[error("input is empty")]
    EmptyInput,
}

#[derive(Clone, Debug, Error)]
#[error("{kind} [{context}]")]
pub struct AppError {
    pub kind: ErrorKind,
    pub context: Arc<str>,
}

impl AppError {
    pub fn network(msg: impl Into<Arc<str>>, context: impl Into<Arc<str>>) -> Self {
        Self {
            kind: ErrorKind::Network(msg.into()),
            context: context.into(),
        }
    }

    pub fn http(status: u16, msg: impl Into<Arc<str>>, context: impl Into<Arc<str>>) -> Self {
        Self {
            kind: ErrorKind::Http {
                status,
                message: msg.into(),
            },
            context: context.into(),
        }
    }

    pub fn parse(msg: impl Into<Arc<str>>, context: impl Into<Arc<str>>) -> Self {
        Self {
            kind: ErrorKind::Parse(msg.into()),
            context: context.into(),
        }
    }

    pub fn validation(error: ValidationError, context: impl Into<Arc<str>>) -> Self {
        Self {
            kind: ErrorKind::Validation(error),
            context: context.into(),
        }
    }

    pub fn unknown(msg: impl Into<Arc<str>>, context: impl Into<Arc<str>>) -> Self {
        Self {
            kind: ErrorKind::Unknown(msg.into()),
            context: context.into(),
        }
    }
}

impl From<crate::api::ApiClientError> for AppError {
    fn from(err: crate::api::ApiClientError) -> Self {
        match err {
            crate::api::ApiClientError::NotConfigured => {
                Self::unknown("API not configured", "checking API availability")
            }
            crate::api::ApiClientError::Network(msg) => Self::network(msg, "calling API"),
            crate::api::ApiClientError::Http(status, body) => {
                Self::http(status, body, "calling API")
            }
            crate::api::ApiClientError::Parse(msg) => Self::parse(msg, "parsing API response"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_error_has_context() {
        let err = AppError::network("connection refused", "connecting to API");
        assert_eq!(err.context.as_ref(), "connecting to API");
        assert!(matches!(err.kind, ErrorKind::Network(_)));
    }

    #[test]
    fn http_error_carries_status() {
        let err = AppError::http(500, "internal server error", "calling API");
        assert!(matches!(err.kind, ErrorKind::Http { status: 500, .. }));
    }

    #[test]
    fn validation_error_round_trips() {
        let err = AppError::validation(ValidationError::EmptyInput, "validating input");
        assert_eq!(
            err.to_string(),
            "Validation error: input is empty [validating input]"
        );
    }
}
