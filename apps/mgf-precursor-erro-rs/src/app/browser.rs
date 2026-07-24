use dioxus::prelude::*;

#[cfg(target_arch = "wasm32")]
use crate::metrics::PrecursorStats;
#[cfg(target_arch = "wasm32")]
use crate::parser::parse_mgf_from_string;
#[cfg(target_arch = "wasm32")]
use js_sys::Array;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;
#[cfg(target_arch = "wasm32")]
use web_sys::{Blob, HtmlAnchorElement, Url};

#[cfg(target_arch = "wasm32")]
const EXAMPLE_MGF_URL: &str =
    "https://raw.githubusercontent.com/zamboni-lab/MultiMS2/main/data/multims2_spectra.mgf";

#[cfg(target_arch = "wasm32")]
async fn fetch_remote_blob(url: &str) -> Result<Blob, String> {
    let window = web_sys::window().ok_or_else(|| "Browser window unavailable.".to_string())?;
    let response_value = JsFuture::from(window.fetch_with_str(url))
        .await
        .map_err(|error| format!("Unable to fetch the example MGF: {error:?}"))?;
    let response: web_sys::Response = response_value
        .dyn_into()
        .map_err(|error| format!("Expected a fetch response: {error:?}"))?;
    if !response.ok() {
        return Err(format!(
            "The example MGF could not be loaded (HTTP {}).",
            response.status()
        ));
    }

    let text_value = JsFuture::from(
        response
            .text()
            .map_err(|error| format!("Unable to read the example MGF response body: {error:?}"))?,
    )
    .await
    .map_err(|error| format!("Unable to read the example MGF text: {error:?}"))?;
    let text = js_sys::JsString::from(text_value)
        .as_string()
        .ok_or_else(|| "The example MGF response was not valid text.".to_string())?;

    let array = Array::new();
    array.push(&JsValue::from(text));
    Blob::new_with_str_sequence(&array)
        .map_err(|error| format!("Unable to create a blob from the example MGF: {error:?}"))
}

#[cfg(target_arch = "wasm32")]
fn start_analysis(
    blob: Blob,
    status: Signal<String>,
    metrics: Signal<Option<PrecursorStats>>,
    busy: Signal<bool>,
    original_mgf_content: Signal<String>,
) {
    let mut status_for_progress = status;
    let mut metrics_for_results = metrics;
    let mut busy_for_results = busy;
    let mut original_content_signal = original_mgf_content;

    spawn(async move {
        let total_bytes = blob.size() as u64;
        status_for_progress.set(format!("Scanning {total_bytes} bytes..."));

        let text_result = JsFuture::from(blob.text())
            .await
            .ok()
            .and_then(|v| v.as_string());

        match text_result {
            Some(content) => {
                original_content_signal.set(content.clone());
                let result = match parse_mgf_from_string(&content) {
                    Ok(metrics) => metrics,
                    Err(error) => {
                        status_for_progress.set(format!("Error parsing MGF: {error:?}"));
                        PrecursorStats::default()
                    }
                };
                metrics_for_results.set(Some(result));
            }
            None => {
                status_for_progress.set("Error reading file content".to_string());
                metrics_for_results.set(Some(PrecursorStats::default()));
            }
        }
        busy_for_results.set(false);
    });
}

#[cfg(target_arch = "wasm32")]
pub fn begin_analysis_from_blob(
    blob: Blob,
    file_name: String,
    file_name_signal: Signal<String>,
    status: Signal<String>,
    metrics: Signal<Option<PrecursorStats>>,
    busy: Signal<bool>,
    drag_active: Signal<bool>,
    original_mgf_content: Signal<String>,
) {
    let mut file_name_for_state = file_name_signal;
    let mut status_for_state = status;
    let mut metrics_for_state = metrics;
    let mut busy_for_state = busy;
    let mut drag_active_for_state = drag_active;
    let original_content_signal = original_mgf_content;

    file_name_for_state.set(file_name);
    busy_for_state.set(true);
    drag_active_for_state.set(false);
    status_for_state.set("Reading MGF...".to_string());
    metrics_for_state.set(None);

    start_analysis(
        blob,
        status_for_state,
        metrics_for_state,
        busy_for_state,
        original_content_signal,
    );
}

#[cfg(target_arch = "wasm32")]
pub fn load_example_mgf(
    status: Signal<String>,
    metrics: Signal<Option<PrecursorStats>>,
    busy: Signal<bool>,
    file_name: Signal<String>,
    original_mgf_content: Signal<String>,
) {
    let mut status_for_progress = status;
    let mut metrics_for_results = metrics;
    let mut busy_for_results = busy;
    let mut file_name_for_results = file_name;
    let original_content_signal = original_mgf_content;

    spawn(async move {
        status_for_progress.set("Loading example MGF...".to_string());
        busy_for_results.set(true);
        metrics_for_results.set(None);

        match fetch_remote_blob(EXAMPLE_MGF_URL).await {
            Ok(blob) => {
                file_name_for_results.set("multims2_spectra.mgf".to_string());
                start_analysis(
                    blob,
                    status_for_progress,
                    metrics_for_results,
                    busy_for_results,
                    original_content_signal,
                );
            }
            Err(error) => {
                status_for_progress.set(error);
                busy_for_results.set(false);
            }
        }
    });
}

pub fn download_recalibrated_mgf(
    file_name: &str,
    original_content: &str,
    calibration_model: crate::recalibration::CalibrationModel,
    diagnostics: Option<&crate::diagnostics::RecalibrationStats>,
) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        use crate::recalibration::generate_recalibrated_mgf;
        use web_sys::console;

        console::log_1(
            &format!(
                "download_recalibrated_mgf called: file={}, model={:?}, content_len={}",
                file_name,
                calibration_model,
                original_content.len()
            )
            .into(),
        );

        let recalibrated =
            generate_recalibrated_mgf(original_content, calibration_model, diagnostics);

        console::log_1(
            &format!(
                "After recalibration: original_len={}, recalibrated_len={}",
                original_content.len(),
                recalibrated.len()
            )
            .into(),
        );

        if original_content == recalibrated {
            console::log_1(&"WARNING: Original and recalibrated are IDENTICAL!".into());
        } else {
            console::log_1(&"OK: Content was modified".into());
        }

        let array = Array::new();
        array.push(&JsValue::from(&recalibrated));
        let blob = Blob::new_with_str_sequence(&array).map_err(|_| "Failed to create blob")?;

        let url =
            Url::create_object_url_with_blob(&blob).map_err(|_| "Failed to create object URL")?;
        let window = web_sys::window().ok_or("No window object")?;
        let document = window.document().ok_or("No document object")?;
        let link = document
            .create_element("a")
            .map_err(|_| "Failed to create anchor element")?
            .dyn_into::<HtmlAnchorElement>()
            .map_err(|_| "Failed to cast to HtmlAnchorElement")?;

        link.set_href(&url);
        let download_name = if file_name.ends_with(".mgf") {
            format!("{}_recalibrated.mgf", &file_name[..file_name.len() - 4])
        } else {
            format!("{}_recalibrated.mgf", file_name)
        };
        link.set_download(&download_name);
        link.click();

        Url::revoke_object_url(&url).map_err(|_| "Failed to revoke object URL")?;
        return Ok(());
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (file_name, original_content, calibration_model, diagnostics);
        Err("Recalibration download is only available in the browser.".to_string())
    }
}

#[cfg(target_arch = "wasm32")]
pub fn download_svg(svg_markup: &str, filename: &str) {
    let safe_name = if filename.ends_with(".svg") {
        filename.to_string()
    } else {
        format!("{filename}.svg")
    };

    let array = Array::new();
    array.push(&JsValue::from(svg_markup));
    let blob = Blob::new_with_str_sequence(&array).unwrap();
    let url = Url::create_object_url_with_blob(&blob).unwrap();
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let anchor: HtmlAnchorElement = document
        .create_element("a")
        .unwrap()
        .dyn_into::<HtmlAnchorElement>()
        .unwrap();
    anchor.set_attribute("href", &url).unwrap();
    anchor.set_attribute("download", &safe_name).unwrap();
    anchor.set_attribute("style", "display:none").unwrap();
    document.body().unwrap().append_child(&anchor).unwrap();
    anchor.click();
    document.body().unwrap().remove_child(&anchor).unwrap();
    Url::revoke_object_url(&url).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn download_recalibrated_mgf_is_browser_only() {
        assert!(
            download_recalibrated_mgf(
                "test.mgf",
                "",
                crate::recalibration::CalibrationModel::None,
                None
            )
            .is_err()
        );
    }
}
