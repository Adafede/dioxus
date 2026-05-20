// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use tower::ServiceExt;
use utoipa::OpenApi;

use crate::{
    ApiDoc, build_router,
    config::AppConfig,
    query_logic::{apply_request, normalized_structure_input},
    state::AppState,
    state::{CachedExportResponse, prune_cache},
    types::{ExportUrlResponse, SearchRequest},
};

fn map_provider(values: &[(&str, &str)]) -> HashMap<String, String> {
    values
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

fn test_config() -> AppConfig {
    AppConfig::from_provider(|_| None).expect("test config")
}

async fn body_json(response: axum::response::Response) -> serde_json::Value {
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body bytes");
    serde_json::from_slice(&bytes).expect("json body")
}

#[test]
fn supports_u16_formula_ranges() {
    let req = SearchRequest {
        taxon: Some("*".to_string()),
        smiles: None,
        smiles_search_type: None,
        smiles_threshold: None,
        mass_min: None,
        mass_max: None,
        year_min: None,
        year_max: None,
        formula_exact: None,
        c_min: Some(1),
        c_max: Some(300),
        h_min: None,
        h_max: None,
        n_min: None,
        n_max: None,
        o_min: None,
        o_max: None,
        p_min: None,
        p_max: None,
        s_min: None,
        s_max: None,
        f_state: None,
        cl_state: None,
        br_state: None,
        i_state: None,
        limit: None,
        include_counts: None,
    };

    let c = apply_request(&req).expect("valid criteria");
    assert_eq!(c.c_max, 300);
}

#[test]
fn config_uses_safe_defaults() {
    let env = HashMap::<String, String>::new();
    let cfg = AppConfig::from_provider(|name| env.get(name).cloned()).expect("valid config");
    assert_eq!(cfg.host, "127.0.0.1");
    assert_eq!(cfg.port, 8787);
    assert_eq!(cfg.default_limit, 500);
    assert_eq!(cfg.request_timeout, Duration::from_millis(45_000));
    assert_eq!(cfg.max_concurrency, 256);
    assert_eq!(cfg.max_body_bytes, 1_048_576);
    assert!(cfg.cors_allowed_origins.is_none());
}

#[test]
fn config_reads_performance_tunables() {
    let env = map_provider(&[
        ("REQUEST_TIMEOUT_MS", "120000"),
        ("MAX_CONCURRENCY", "512"),
        ("MAX_BODY_BYTES", "2097152"),
    ]);
    let cfg = AppConfig::from_provider(|name| env.get(name).cloned()).expect("valid config");
    assert_eq!(cfg.request_timeout, Duration::from_millis(120_000));
    assert_eq!(cfg.max_concurrency, 512);
    assert_eq!(cfg.max_body_bytes, 2_097_152);
}

#[test]
fn config_rejects_invalid_port() {
    let env = map_provider(&[("PORT", "abc")]);
    let err = AppConfig::from_provider(|name| env.get(name).cloned())
        .expect_err("invalid PORT should fail");
    assert!(err.contains("PORT"));
}

#[test]
fn production_requires_explicit_cors_allowlist() {
    let env = map_provider(&[("APP_ENV", "production")]);
    let err = AppConfig::from_provider(|name| env.get(name).cloned())
        .expect_err("production without CORS origins should fail");
    assert!(err.contains("CORS_ALLOWED_ORIGINS"));
}

#[test]
fn parses_comma_separated_cors_origins() {
    let env = map_provider(&[(
        "CORS_ALLOWED_ORIGINS",
        "https://api.example.org, http://localhost:5173",
    )]);
    let cfg = AppConfig::from_provider(|name| env.get(name).cloned()).expect("valid config");
    assert_eq!(cfg.cors_allowed_origins.as_ref().map(Vec::len), Some(2));
}

#[test]
fn normalized_structure_preserves_multiline_molfile() {
    let molfile = "\n  Mrv\n\n  0  0  0  0  0  0            999 V3000\nM  END\n";
    let normalized = normalized_structure_input(molfile);
    assert!(normalized.starts_with('\n'));
    assert!(normalized.contains("V3000"));
}

#[test]
fn prune_cache_removes_oldest_when_over_capacity() {
    let mut cache = HashMap::from([
        (
            "a".to_string(),
            CachedExportResponse {
                inserted_at: Instant::now() - Duration::from_secs(30),
                value: ExportUrlResponse {
                    query: "a".into(),
                    csv_url: "a".into(),
                    json_url: "a".into(),
                    rdf_url: "a".into(),
                    csv_gz_url: "a".into(),
                    json_gz_url: "a".into(),
                    rdf_gz_url: "a".into(),
                },
            },
        ),
        (
            "b".to_string(),
            CachedExportResponse {
                inserted_at: Instant::now(),
                value: ExportUrlResponse {
                    query: "b".into(),
                    csv_url: "b".into(),
                    json_url: "b".into(),
                    rdf_url: "b".into(),
                    csv_gz_url: "b".into(),
                    json_gz_url: "b".into(),
                    rdf_gz_url: "b".into(),
                },
            },
        ),
    ]);

    prune_cache(&mut cache, Duration::from_secs(60), 1, |entry| {
        entry.inserted_at
    });
    assert!(cache.contains_key("b"));
    assert!(!cache.contains_key("a"));
}

#[tokio::test]
async fn health_route_returns_ok() {
    let config = test_config();
    let app = build_router(config.max_body_bytes, &config, AppState::new(&config));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("health response");

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn unknown_route_returns_not_found() {
    let config = test_config();
    let app = build_router(config.max_body_bytes, &config, AppState::new(&config));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/does-not-exist")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("not found response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn export_file_rejects_unsupported_format() {
    let config = test_config();
    let app = build_router(config.max_body_bytes, &config, AppState::new(&config));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/export-file/some-key/ttl")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("unsupported format response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let json = body_json(response).await;
    assert!(
        json.get("error")
            .and_then(serde_json::Value::as_str)
            .is_some()
    );
}

#[tokio::test]
async fn search_rejects_malformed_json_payload() {
    let config = test_config();
    let app = build_router(config.max_body_bytes, &config, AppState::new(&config));

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/search")
                .header("content-type", "application/json")
                .body(Body::from("{not-json"))
                .expect("request"),
        )
        .await
        .expect("malformed-json response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn openapi_json_endpoint_serves_core_paths() {
    let config = test_config();
    let app = build_router(config.max_body_bytes, &config, AppState::new(&config));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/openapi.json")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("openapi response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("openapi body bytes");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("openapi json");

    assert!(json["paths"].get("/health").is_some());
    assert!(json["paths"].get("/v1/search").is_some());
    assert!(json["paths"].get("/v1/export-url").is_some());
    assert!(
        json["paths"]
            .get("/v1/export-file/{cache_key}/{format}")
            .is_some()
    );
}

#[test]
fn openapi_contains_core_paths() {
    let doc = ApiDoc::openapi();
    assert!(doc.paths.paths.contains_key("/health"));
    assert!(doc.paths.paths.contains_key("/v1/search"));
    assert!(doc.paths.paths.contains_key("/v1/export-url"));
    assert!(
        doc.paths
            .paths
            .contains_key("/v1/export-file/{cache_key}/{format}")
    );
}

#[test]
fn openapi_contains_error_and_search_schemas() {
    let doc = ApiDoc::openapi();
    let components = doc.components.expect("openapi components");

    assert!(components.schemas.contains_key("ErrorResponse"));
    assert!(components.schemas.contains_key("SearchRequest"));
    assert!(components.schemas.contains_key("SearchResponse"));
    assert!(components.schemas.contains_key("ExportUrlResponse"));
}

#[tokio::test]
async fn export_file_rejects_unknown_cache_key() {
    let config = test_config();
    let app = build_router(config.max_body_bytes, &config, AppState::new(&config));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/export-file/missing/csv")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("unknown key response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let json = body_json(response).await;
    assert!(
        json.get("error")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|msg| msg.contains("expired") || msg.contains("unknown"))
    );
}
