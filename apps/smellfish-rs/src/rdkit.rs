use crate::model::RdkitInspectResponse;
use js_sys::{JSON, Promise, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

#[cfg(target_arch = "wasm32")]
pub async fn read_file_text(file: &web_sys::File) -> Result<String, String> {
    let promise = file.text();
    let value = JsFuture::from(promise)
        .await
        .map_err(|err| format!("failed to read file: {err:?}"))?;
    value
        .as_string()
        .ok_or_else(|| "file did not resolve to text".to_string())
}

#[cfg(target_arch = "wasm32")]
pub async fn rdkit_inspect(smiles: &str) -> Result<RdkitInspectResponse, String> {
    let value = rdkit_bridge_call("inspect", smiles).await?;
    let json = js_value_to_json(value)?;
    serde_json::from_value(json).map_err(|err| err.to_string())
}

#[cfg(target_arch = "wasm32")]
async fn rdkit_bridge_call(method: &str, smiles: &str) -> Result<JsValue, String> {
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

#[cfg(target_arch = "wasm32")]
fn js_value_to_json(value: JsValue) -> Result<serde_json::Value, String> {
    let text = JSON::stringify(&value)
        .ok()
        .and_then(|value| value.as_string())
        .ok_or_else(|| "rdkit.js returned a non-serializable value".to_string())?;
    serde_json::from_str(&text).map_err(|err| err.to_string())
}
