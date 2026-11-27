// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Browser file ingestion, remote example fetch, and `.svg`/`.mgf` downloads.

use dioxus::html::FileData;
use dioxus::prelude::WritableExt;
#[cfg(target_arch = "wasm32")]
use dioxus::prelude::spawn;

use super::UploadSignals;

#[cfg(target_arch = "wasm32")]
use mgf_analysis_core::{
    LineIngestor, MAX_RAW_RETENTION_BYTES, PrecursorStats, append_retained_line,
};
#[cfg(target_arch = "wasm32")]
use stream_io::BlobLines;
#[cfg(target_arch = "wasm32")]
use upload::{BlobSource, UploadError};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;
#[cfg(target_arch = "wasm32")]
use web_sys::Blob;

#[cfg(target_arch = "wasm32")]
const EXAMPLE_MGF_URL: &str =
    "https://raw.githubusercontent.com/zamboni-lab/MultiMS2/main/data/multims2_spectra.mgf";

#[cfg(target_arch = "wasm32")]
#[expect(
    clippy::future_not_send,
    reason = "wasm32 single-threaded executor captures JsFuture and Blob"
)]
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

/// Sanitizes a plot title into a `.svg` filename.
#[cfg(any(test, target_arch = "wasm32"))]
#[must_use]
pub(crate) fn svg_download_filename(title: &str) -> String {
    let mut slug = String::new();
    for ch in title.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
        } else if !slug.is_empty() && !slug.ends_with('_') {
            slug.push('_');
        }
    }
    let trimmed = slug.trim_matches('_');
    let stem = if trimmed.is_empty() {
        "plot"
    } else {
        trimmed.strip_suffix("_svg").unwrap_or(trimmed)
    };
    let stem = if stem.is_empty() { "plot" } else { stem };
    format!("{stem}.svg")
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn download_svg(svg: &str, title: &str) {
    let filename = svg_download_filename(title);
    drop(upload::download_svg(svg, &filename));
}

#[must_use]
pub(crate) fn recalibrated_mgf_filename(filename: &str) -> String {
    let base = if filename.len() >= 4
        && filename
            .get(filename.len() - 4..)
            .is_some_and(|ext| ext.eq_ignore_ascii_case(".mgf"))
    {
        filename.get(..filename.len() - 4).unwrap_or(filename)
    } else {
        filename
    };
    format!("{base}_recalibrated.mgf")
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn download_recalibrated_mgf(filename: &str, content: &str) -> Result<String, String> {
    let download_name = recalibrated_mgf_filename(filename);
    upload::download_text(content, &download_name).map_err(|err| err.to_string())?;
    Ok(download_name)
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn download_recalibrated_mgf(filename: &str, _content: &str) -> Result<String, String> {
    let _download_name = recalibrated_mgf_filename(filename);
    Err("Download is only available in the browser".to_string())
}

#[cfg(target_arch = "wasm32")]
#[expect(
    clippy::future_not_send,
    reason = "wasm32 single-threaded executor captures BlobSource and StreamError futures"
)]
async fn scan_blob_with_progress(
    blob: &Blob,
    on_progress: impl FnMut(u64, u64),
) -> Result<(PrecursorStats, String), UploadError> {
    let mut reader = BlobLines::new(BlobSource::new(blob), on_progress);
    let mut ingestor = LineIngestor::default();
    let mut raw_content = String::new();
    let mut raw_retention_capped = false;

    while let Some(line) = reader.next_line().await? {
        ingestor.feed_line(&line);
        append_retained_line(
            &mut raw_content,
            &mut raw_retention_capped,
            &line,
            MAX_RAW_RETENTION_BYTES,
        );
    }

    let mut result = ingestor.finish();
    result.raw_retention_capped = raw_retention_capped;
    Ok((result, raw_content))
}

#[cfg(target_arch = "wasm32")]
fn start_analysis(blob: Blob, mut signals: UploadSignals, generation: u64) {
    spawn(async move {
        let total_bytes = BlobSource::new(&blob).total_bytes();
        if !signals.analysis_generation.is_current(generation) {
            return;
        }
        signals
            .status
            .set(format!("Scanning {total_bytes} bytes..."));

        match scan_blob_with_progress(&blob, |processed, total| {
            if signals.analysis_generation.is_current(generation) {
                signals.status.set(format!(
                    "Scanning... {}/{} bytes",
                    processed.min(total),
                    total
                ));
            }
        })
        .await
        {
            Ok((result, raw_content)) => {
                if !signals.analysis_generation.is_current(generation) {
                    return;
                }
                signals
                    .status
                    .set(format!("Processed {} spectra.", result.total_spectra));
                signals.original_mgf_content.set(raw_content);
                signals.metrics.set(Some(result));
            }
            Err(error) => {
                if !signals.analysis_generation.is_current(generation) {
                    return;
                }
                signals.status.set(format!("Error scanning MGF: {error}"));
                signals.original_mgf_content.set(String::new());
                signals.metrics.set(None);
            }
        }
        if signals.analysis_generation.is_current(generation) {
            signals.busy.set(false);
        }
    });
}

#[cfg(target_arch = "wasm32")]
fn begin_analysis_from_blob(blob: Blob, file_name: String, mut signals: UploadSignals) {
    let generation = signals.analysis_generation.advance();
    signals.file_name.set(file_name);
    signals.busy.set(true);
    signals.drag_active.set(false);
    signals.status.set("Reading MGF...".to_string());
    signals.original_mgf_content.set(String::new());
    signals.metrics.set(None);
    start_analysis(blob, signals, generation);
}

/// Dispatches uploaded file data on `wasm32` or displays browser-only notice on host.
pub(super) fn attempt_analysis_from_files(files: &[FileData], signals: UploadSignals) {
    upload::dispatch_file_upload(files, signals.status, |file| {
        #[cfg(target_arch = "wasm32")]
        begin_analysis_from_blob(file.blob, file.name, signals);

        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut signals = signals;
            signals.drag_active.set(false);
            signals.original_mgf_content.set(String::new());
            signals.metrics.set(None);
            signals.busy.set(false);
            upload::report_browser_only_upload(signals.file_name, signals.status, file.name);
        }
    });
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn load_example_mgf(mut signals: UploadSignals) {
    let generation = signals.analysis_generation.advance();
    signals.original_mgf_content.set(String::new());
    spawn(async move {
        signals.status.set("Loading example MGF...".to_string());
        signals.busy.set(true);
        signals.metrics.set(None);

        match fetch_remote_blob(EXAMPLE_MGF_URL).await {
            Ok(blob) => {
                if !signals.analysis_generation.is_current(generation) {
                    return;
                }
                signals.file_name.set("multims2_spectra.mgf".to_string());
                start_analysis(blob, signals, generation);
            }
            Err(error) => {
                if !signals.analysis_generation.is_current(generation) {
                    return;
                }
                signals.status.set(error);
                signals.busy.set(false);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{recalibrated_mgf_filename, svg_download_filename};

    #[test]
    fn recalibrated_mgf_filename_strips_mgf_suffix_case_insensitively() {
        assert_eq!(
            recalibrated_mgf_filename("spectra.mgf"),
            "spectra_recalibrated.mgf"
        );
        assert_eq!(
            recalibrated_mgf_filename("spectra.MGF"),
            "spectra_recalibrated.mgf"
        );
        assert_eq!(
            recalibrated_mgf_filename("spectra"),
            "spectra_recalibrated.mgf"
        );
    }

    #[test]
    fn svg_download_filename_sanitizes_titles_and_appends_svg() {
        assert_eq!(
            svg_download_filename("Absolute Mass Error Distribution"),
            "absolute_mass_error_distribution.svg"
        );
        assert_eq!(
            svg_download_filename("Error (ppm / mDa)"),
            "error_ppm_mda.svg"
        );
        assert_eq!(svg_download_filename("chart.svg"), "chart.svg");
        assert_eq!(svg_download_filename("   "), "plot.svg");
    }
}
