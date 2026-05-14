// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use super::*;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::OnceLock;

#[cfg(target_arch = "wasm32")]
use js_sys::{Function, JSON, Promise, Reflect};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn natprod_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .build()
            .expect("curation http client")
    })
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
pub(super) struct BatchConvertResponse {
    pub(super) results: Vec<BatchConvertItem>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
pub(super) struct BatchConvertItem {
    pub(super) output: String,
    pub(super) success: bool,
    pub(super) error: String,
}

#[cfg(target_arch = "wasm32")]
pub(super) async fn rdkit_bridge_call(
    method: &str,
    smiles: &str,
) -> Result<JsValue, CurationError> {
    let window = web_sys::window().ok_or_else(|| {
        CurationError::Http("window is unavailable; rdkit.js bridge cannot be used".to_string())
    })?;
    let window_value = JsValue::from(window);
    let bridge = Reflect::get(&window_value, &JsValue::from_str("__lotusRdkit"))
        .map_err(|_| CurationError::Http("rdkit.js bridge lookup failed".to_string()))?;
    if bridge.is_null() || bridge.is_undefined() {
        return Err(CurationError::Http(
            "rdkit.js bridge is unavailable; ensure RDKit assets are loaded".to_string(),
        ));
    }

    let ready = Reflect::get(&bridge, &JsValue::from_str("ready"))
        .map_err(|_| CurationError::Http("rdkit.js readiness promise missing".to_string()))?;
    if let Ok(promise) = ready.dyn_into::<Promise>() {
        JsFuture::from(promise).await.map_err(|err| {
            CurationError::Http(format!("rdkit.js failed to initialize: {err:?}"))
        })?;
    }

    let function = Reflect::get(&bridge, &JsValue::from_str(method))
        .map_err(|_| CurationError::Http(format!("rdkit.js method '{method}' not found")))?
        .dyn_into::<Function>()
        .map_err(|_| CurationError::Http(format!("rdkit.js method '{method}' is not callable")))?;

    let result = function
        .call1(&bridge, &JsValue::from_str(smiles))
        .map_err(|err| CurationError::Http(format!("rdkit.js {method} call failed: {err:?}")))?;

    if let Ok(promise) = result.clone().dyn_into::<Promise>() {
        JsFuture::from(promise)
            .await
            .map_err(|err| CurationError::Http(format!("rdkit.js {method} failed: {err:?}")))
    } else {
        Ok(result)
    }
}

#[cfg(target_arch = "wasm32")]
pub(super) fn js_value_to_json(value: JsValue) -> Result<Value, CurationError> {
    let text = JSON::stringify(&value)
        .ok()
        .and_then(|s| s.as_string())
        .ok_or_else(|| {
            CurationError::Parse("rdkit.js returned a non-serializable value".to_string())
        })?;
    serde_json::from_str(&text)
        .map_err(|e| CurationError::Parse(format!("rdkit.js JSON parse error: {e}")))
}
