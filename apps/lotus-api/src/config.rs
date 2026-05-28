// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Application configuration management from environment variables.
//!
//! Handles initialization of server settings, resource limits, and CORS policy
//! with sensible defaults and validation.

use axum::http::HeaderValue;
use std::{net::SocketAddr, time::Duration};
use tower_http::cors::{Any, CorsLayer};

#[derive(Debug, Clone)]
pub(crate) struct AppConfig {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) default_limit: usize,
    pub(crate) request_timeout: Duration,
    pub(crate) max_concurrency: usize,
    pub(crate) max_body_bytes: usize,
    pub(crate) cors_allowed_origins: Option<Vec<HeaderValue>>,
}

impl AppConfig {
    pub(crate) fn from_env() -> Result<Self, String> {
        Self::from_provider(|name| std::env::var(name).ok())
    }

    pub(crate) fn from_provider<F>(mut get: F) -> Result<Self, String>
    where
        F: FnMut(&str) -> Option<String>,
    {
        let host = get("HOST")
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "127.0.0.1".into());

        let port = parse_u16_env(get("PORT"), "PORT", 8787)?;
        let default_limit = parse_usize_env(get("DEFAULT_LIMIT"), "DEFAULT_LIMIT", 500)?
            .clamp(1, shared::lotus::models::TABLE_ROW_LIMIT);
        let request_timeout_ms =
            parse_usize_env(get("REQUEST_TIMEOUT_MS"), "REQUEST_TIMEOUT_MS", 45_000)?
                .clamp(1_000, 300_000);
        let max_concurrency =
            parse_usize_env(get("MAX_CONCURRENCY"), "MAX_CONCURRENCY", 256)?.clamp(8, 4_096);
        let max_body_bytes = parse_usize_env(get("MAX_BODY_BYTES"), "MAX_BODY_BYTES", 1_048_576)?
            .clamp(4 * 1024, 16 * 1024 * 1024);

        let app_env = get("APP_ENV")
            .map(|value| value.trim().to_ascii_lowercase())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "development".into());

        let cors_allowed_origins = parse_allowed_origins(get("CORS_ALLOWED_ORIGINS"))?;
        if app_env == "production" && cors_allowed_origins.is_none() {
            return Err("APP_ENV=production requires CORS_ALLOWED_ORIGINS to be configured".into());
        }

        Ok(Self {
            host,
            port,
            default_limit,
            request_timeout: Duration::from_millis(request_timeout_ms as u64),
            max_concurrency,
            max_body_bytes,
            cors_allowed_origins,
        })
    }

    pub(crate) fn bind_addr(&self) -> Result<SocketAddr, String> {
        format!("{}:{}", self.host, self.port)
            .parse::<SocketAddr>()
            .map_err(|e| format!("invalid bind address '{}:{}': {e}", self.host, self.port))
    }
}

fn parse_u16_env(value: Option<String>, name: &str, default_value: u16) -> Result<u16, String> {
    value.map_or(Ok(default_value), |raw| {
        raw.trim()
            .parse::<u16>()
            .map_err(|e| format!("{name} must be a valid u16: {e}"))
    })
}

fn parse_usize_env(
    value: Option<String>,
    name: &str,
    default_value: usize,
) -> Result<usize, String> {
    value.map_or(Ok(default_value), |raw| {
        raw.trim()
            .parse::<usize>()
            .map_err(|e| format!("{name} must be a valid non-negative integer: {e}"))
    })
}

fn parse_allowed_origins(value: Option<String>) -> Result<Option<Vec<HeaderValue>>, String> {
    let Some(raw) = value else {
        return Ok(None);
    };

    let mut origins = Vec::new();
    for origin in raw
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
    {
        if !origin.starts_with("http://") && !origin.starts_with("https://") {
            return Err(format!(
                "CORS_ALLOWED_ORIGINS entry '{origin}' must start with http:// or https://"
            ));
        }
        let header = HeaderValue::from_str(origin)
            .map_err(|_| format!("CORS_ALLOWED_ORIGINS contains invalid origin '{origin}'"))?;
        origins.push(header);
    }

    if origins.is_empty() {
        Ok(None)
    } else {
        Ok(Some(origins))
    }
}

pub(crate) fn build_cors_layer(config: &AppConfig) -> CorsLayer {
    let layer = CorsLayer::new().allow_methods(Any).allow_headers(Any);
    match &config.cors_allowed_origins {
        Some(origins) => layer.allow_origin(origins.clone()),
        None => layer.allow_origin(Any),
    }
}
