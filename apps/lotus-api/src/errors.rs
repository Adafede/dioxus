// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Error handling and HTTP response mapping for LOTUS API endpoints.
//!
//! This module provides a consistent error abstraction layer that maps
//! internal errors to appropriate HTTP status codes and JSON responses.

use axum::{Json, http::StatusCode, response::IntoResponse, response::Response};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub(crate) error: String,
}

#[derive(Debug)]
pub struct ApiError {
    pub(crate) status: StatusCode,
    pub(crate) message: String,
}

#[derive(Debug, Clone)]
pub struct SharedApiError {
    pub(crate) status: StatusCode,
    pub(crate) message: String,
}

impl ApiError {
    pub(crate) fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    pub(crate) fn upstream(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_GATEWAY,
            message: message.into(),
        }
    }

    pub(crate) fn overloaded(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            message: message.into(),
        }
    }
}

impl From<ApiError> for SharedApiError {
    fn from(value: ApiError) -> Self {
        Self {
            status: value.status,
            message: value.message,
        }
    }
}

impl From<SharedApiError> for ApiError {
    fn from(value: SharedApiError) -> Self {
        Self {
            status: value.status,
            message: value.message,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = Json(ErrorResponse {
            error: self.message,
        });
        (self.status, body).into_response()
    }
}
