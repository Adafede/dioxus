// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::api::config::api_base_url;
use crate::api::dto::{SearchRequest, SearchResponse};
use crate::api::error::ApiClientError;
use crate::models::SearchCriteria;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

pub async fn search(
    criteria: &SearchCriteria,
    limit: usize,
    include_counts: bool,
) -> Result<SearchResponse, ApiClientError> {
    let base = api_base_url().ok_or(ApiClientError::NotConfigured)?;
    let request = SearchRequest::from_criteria(criteria, limit, include_counts);
    match post_json(&base, "/v1/search", &request).await {
        Ok(response) => Ok(response),
        Err(ApiClientError::Http(status, _))
            if include_counts && (status == 502 || status == 504) =>
        {
            log::warn!("event=search phase=api state=retry_without_counts status={status}");
            let retry_request = SearchRequest::from_criteria(criteria, limit, false);
            post_json(&base, "/v1/search", &retry_request).await
        }
        Err(err) => Err(err),
    }
}

#[cfg(target_arch = "wasm32")]
pub async fn export_urls(
    criteria: &SearchCriteria,
) -> Result<crate::api::dto::ExportUrlResponse, ApiClientError> {
    let base = api_base_url().ok_or(ApiClientError::NotConfigured)?;
    let request = SearchRequest::from_criteria(criteria, 1, false);
    let response: crate::api::dto::ExportUrlResponse =
        post_json(&base, "/v1/export-url", &request).await?;
    Ok(normalize_export_urls(&base, response))
}

fn http_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(build_http_client)
}

fn build_http_client() -> reqwest::Client {
    #[cfg(target_arch = "wasm32")]
    {
        reqwest::Client::builder()
            .build()
            .expect("LOTUS explorer API client")
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(30))
            .pool_idle_timeout(Duration::from_secs(60))
            .pool_max_idle_per_host(8)
            .tcp_keepalive(Duration::from_secs(30))
            .build()
            .expect("LOTUS explorer API client")
    }
}

#[cfg(target_arch = "wasm32")]
fn normalize_export_urls(
    base: &str,
    mut response: crate::api::dto::ExportUrlResponse,
) -> crate::api::dto::ExportUrlResponse {
    response.csv_url = resolve_api_url(base, &response.csv_url);
    response.json_url = resolve_api_url(base, &response.json_url);
    response.rdf_url = resolve_api_url(base, &response.rdf_url);
    response.csv_gz_url = response.csv_gz_url.map(|url| resolve_api_url(base, &url));
    response.json_gz_url = response.json_gz_url.map(|url| resolve_api_url(base, &url));
    response.rdf_gz_url = response.rdf_gz_url.map(|url| resolve_api_url(base, &url));
    response
}

#[cfg(target_arch = "wasm32")]
fn resolve_api_url(base: &str, url: &str) -> String {
    let trimmed = url.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return trimmed.to_string();
    }
    let base = base.trim_end_matches('/');
    if trimmed.starts_with('/') {
        format!("{base}{trimmed}")
    } else {
        format!("{base}/{trimmed}")
    }
}

async fn post_json<Req, Res>(base: &str, path: &str, body: &Req) -> Result<Res, ApiClientError>
where
    Req: Serialize + ?Sized,
    Res: for<'de> Deserialize<'de>,
{
    let url = format!("{}{}", base.trim_end_matches('/'), path);
    let response = http_client()
        .post(url)
        .json(body)
        .send()
        .await
        .map_err(|e| ApiClientError::Network(e.to_string()))?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(ApiClientError::Http(status.as_u16(), body));
    }

    response
        .json::<Res>()
        .await
        .map_err(|e| ApiClientError::Parse(e.to_string()))
}

#[cfg(test)]
mod tests {
    #[cfg(target_arch = "wasm32")]
    use super::*;

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn resolve_api_url_joins_relative_paths() {
        assert_eq!(
            resolve_api_url("https://api.example.org", "/v1/export-file/abc/csv"),
            "https://api.example.org/v1/export-file/abc/csv"
        );
        assert_eq!(
            resolve_api_url("https://api.example.org/", "v1/export-file/abc/csv"),
            "https://api.example.org/v1/export-file/abc/csv"
        );
    }
}
