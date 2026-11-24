// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

#![expect(
    clippy::future_not_send,
    reason = "wasm32 JsFuture and JsValue interop futures are single-threaded"
)]

use js_sys::{JSON, Promise, Reflect};
use smellfish_core::model::{RdkitInspectResponse, parse_rdkit_inspect_json};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

/// Read a [`web_sys::Blob`] as text via the File API.
///
/// # Errors
///
/// Returns an error if the file read promise is rejected or does not resolve
/// to a string.
pub(crate) async fn read_file_text(file: &web_sys::Blob) -> Result<String, String> {
    let promise = file.text();
    let value = JsFuture::from(promise)
        .await
        .map_err(|err| format!("failed to read file: {err:?}"))?;
    value
        .as_string()
        .ok_or_else(|| "file did not resolve to text".to_string())
}

/// Inspect a SMILES string via the `RDKit.js` bridge.
///
/// # Errors
///
/// Returns an error if the bridge call fails or the JSON response cannot be deserialized.
pub(crate) async fn inspect_smiles(smiles: &str) -> Result<RdkitInspectResponse, String> {
    let value = bridge_call("inspect", smiles).await?;
    let text = JSON::stringify(&value)
        .ok()
        .and_then(|val| val.as_string())
        .ok_or_else(|| "rdkit.js returned a non-serializable value".to_string())?;
    parse_rdkit_inspect_json(&text).map_err(|err| err.to_string())
}

async fn bridge_call(method: &str, smiles: &str) -> Result<JsValue, String> {
    let window = web_sys::window().ok_or_else(|| "window is unavailable".to_string())?;
    let window_value = JsValue::from(window);
    let bridge = Reflect::get(&window_value, &JsValue::from_str("__smilesRdkit"))
        .map_err(|_| "rdkit.js bridge lookup failed".to_string())?;
    if bridge.is_null() || bridge.is_undefined() {
        return Err("rdkit.js bridge is unavailable".to_string());
    }

    let ready = Reflect::get(&bridge, &JsValue::from_str("ready"))
        .map_err(|_| "rdkit.js readiness promise missing".to_string())?;
    if let Ok(promise) = ready.dyn_into::<Promise>() {
        JsFuture::from(promise)
            .await
            .map_err(|err| format!("rdkit.js failed to initialize: {err:?}"))?;
    }

    let function = Reflect::get(&bridge, &JsValue::from_str(method))
        .map_err(|_| format!("rdkit.js method '{method}' not found"))?
        .dyn_into::<js_sys::Function>()
        .map_err(|_| format!("rdkit.js method '{method}' is not callable"))?;

    let result = function
        .call1(&bridge, &JsValue::from_str(smiles))
        .map_err(|err| format!("rdkit.js {method} call failed: {err:?}"))?;

    match result.dyn_into::<Promise>() {
        Ok(promise) => JsFuture::from(promise)
            .await
            .map_err(|err| format!("rdkit.js {method} failed: {err:?}")),
        Err(val) => Ok(val),
    }
}
