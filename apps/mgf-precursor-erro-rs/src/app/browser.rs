use dioxus::html::FileData;
#[cfg(target_arch = "wasm32")]
use dioxus::prelude::spawn;
use dioxus::prelude::{Signal, WritableExt};

use crate::metrics::PrecursorStats;
#[cfg(target_arch = "wasm32")]
use crate::parser::scan_blob_with_progress;
#[cfg(target_arch = "wasm32")]
use upload::UploadError;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;
#[cfg(target_arch = "wasm32")]
use web_sys::Blob;
#[cfg(target_arch = "wasm32")]
use web_sys::console;

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
    let blob_promise = response
        .blob()
        .map_err(|error| format!("Failed to call blob(): {error:?}"))?;
    JsFuture::from(blob_promise)
        .await
        .map_err(|error| format!("Unable to read the example MGF blob: {error:?}"))
        .and_then(|blob| {
            blob.dyn_into::<Blob>()
                .map_err(|error| format!("Expected a blob: {error:?}"))
        })
}

/// Downloads SVG content as a file.
#[cfg(target_arch = "wasm32")]
pub fn download_svg(svg: &str, filename: &str) {
    let _ = upload::download_text(svg, filename);
}

/// Downloads recalibrated MGF content as a `.mgf` file.
#[cfg(target_arch = "wasm32")]
pub fn download_recalibrated_mgf(filename: &str, content: &str) -> Result<(), String> {
    let base = if filename.ends_with(".mgf") {
        &filename[..filename.len() - 4]
    } else {
        filename
    };
    let download_name = format!("{base}_recalibrated.mgf");
    upload::download_text(content, &download_name)
}

/// Downloads recalibrated MGF content (native stub — returns `Err`).
///
/// # Errors
/// Always returns an error on native targets.
#[cfg(not(target_arch = "wasm32"))]
pub fn download_recalibrated_mgf(_filename: &str, _content: &str) -> Result<(), String> {
    Err("Download is only available in the browser".to_string())
}

/// Uses [`upload::BlobLines`] to read the blob in 16 MiB chunks and process
/// one MGF block line at a time, keeping memory bounded regardless of file
/// size.
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

        match scan_blob_with_progress(&blob, |processed, total| {
            status_for_progress.set(format!(
                "Scanning... {}/{} bytes",
                processed.min(total),
                total
            ));
        })
        .await
        {
            Ok(result) => {
                // Read the actual blob content for recalibration
                let blob_clone = blob.clone();
                let content_future = async move {
                    let text_promise = blob_clone.text();
                    JsFuture::from(text_promise)
                        .await
                        .map_err(|e| format!("Failed to read blob text: {e:?}"))
                        .and_then(|text| {
                            text.as_string()
                                .ok_or_else(|| "Failed to convert blob text to string".to_string())
                        })
                };

                // Store the actual content for recalibration
                original_content_signal.set(content_future.await.unwrap_or_else(|e| {
                    console::log_1(&format!("Error reading blob content: {e}").into());
                    format!("(streamed MGF, {total_bytes} bytes)")
                }));
                metrics_for_results.set(Some(result));
            }
            Err(UploadError::UnexpectedEof) => {
                status_for_progress.set("Unexpected end of file while reading.".to_string());
                metrics_for_results.set(Some(PrecursorStats::default()));
            }
            Err(error) => {
                status_for_progress.set(format!("Error scanning MGF: {error}"));
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
        original_mgf_content,
    );
}

/// Extracts a blob from the given file data and dispatches it to the analysis
/// pipeline on wasm32, or reports that a browser is required on native.
/// Consolidates the near-duplicate `on_file_change`/`on_drop` handlers.
#[allow(unused_mut, unused_variables)]
// cfg-mut foot-gun: the native arm calls `.set` on these bindings, while the
// wasm32 arm only forwards the handles to `begin_analysis_from_blob`, so the
// `mut` qualifiers (and `drag_active`/`original_mgf_content`) look unused on
// one target or the other.
pub fn attempt_analysis_from_files(
    files: &[FileData],
    mut file_name: Signal<String>,
    mut status: Signal<String>,
    mut metrics: Signal<Option<PrecursorStats>>,
    mut busy: Signal<bool>,
    drag_active: Signal<bool>,
    original_mgf_content: Signal<String>,
) {
    match upload::extract_blob_from_file_data(files) {
        Ok(Some(file)) => {
            #[cfg(target_arch = "wasm32")]
            begin_analysis_from_blob(
                file.blob,
                file.name,
                file_name,
                status,
                metrics,
                busy,
                drag_active,
                original_mgf_content,
            );

            #[cfg(not(target_arch = "wasm32"))]
            {
                file_name.set(file.name);
                metrics.set(None);
                status.set("This app needs to run in a browser.".to_string());
                busy.set(false);
            }
        }
        Ok(None) => status.set("No file selected.".to_string()),
        Err(msg) => status.set(msg),
    }
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
                    original_mgf_content,
                );
            }
            Err(error) => {
                status_for_progress.set(error);
                busy_for_results.set(false);
            }
        }
    });
}
