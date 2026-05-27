// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::api::config::api_base_url;
use crate::api::dto::{PubchemBuildRequest, PubchemBuildResponse, PubchemFetchResponse};
use crate::api::error::ApiClientError;
use serde::{Serialize, de::DeserializeOwned};
use std::sync::OnceLock;

pub async fn fetch_pubchem_dataset() -> Result<PubchemFetchResponse, ApiClientError> {
    let Some(base) = api_base_url() else {
        return Err(ApiClientError::NotConfigured);
    };
    post_json::<(), PubchemFetchResponse>(&base, "/v1/pubchem-tree/fetch", &()).await
}

pub async fn build_pubchem_trees(session_id: &str) -> Result<PubchemBuildResponse, ApiClientError> {
    let Some(base) = api_base_url() else {
        return Err(ApiClientError::NotConfigured);
    };
    post_json(
        &base,
        "/v1/pubchem-tree/build",
        &PubchemBuildRequest {
            session_id: session_id.to_string(),
        },
    )
    .await
}

fn http_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .build()
            .expect("PubChem tree API client")
    })
}

async fn post_json<Req, Res>(base: &str, path: &str, body: &Req) -> Result<Res, ApiClientError>
where
    Req: Serialize + ?Sized,
    Res: DeserializeOwned,
{
    let url = absolutize_for_wasm(format!("{}{}", base.trim_end_matches('/'), path));
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

#[cfg(target_arch = "wasm32")]
fn absolutize_for_wasm(url: String) -> String {
    if url.starts_with("http://") || url.starts_with("https://") {
        return url;
    }
    if !url.starts_with('/') {
        return url;
    }
    let Some(window) = web_sys::window() else {
        return url;
    };
    let Ok(origin) = window.location().origin() else {
        return url;
    };
    format!("{}{}", origin.trim_end_matches('/'), url)
}

#[cfg(not(target_arch = "wasm32"))]
fn absolutize_for_wasm(url: String) -> String {
    url
}
