use dioxus::events::{DragData, FormData};
use dioxus::html::HasFileData;
use dioxus::prelude::*;

#[cfg(target_arch = "wasm32")]
use js_sys::Array;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;
#[cfg(target_arch = "wasm32")]
use web_sys::{Blob, HtmlAnchorElement, Response, Url, Window, console};

use crate::diagnostics::RecalibrationDiagnostics;
use crate::metrics::{PlotPoint, PrecursorMetrics};
#[cfg(target_arch = "wasm32")]
use crate::parser::{ScanError, scan_blob_with_progress};
use crate::plotting::{
    make_svg_responsive, render_absolute_mass_bias_svg, render_cumulative_error_three_curves,
    render_ecdf_svg, render_mass_bias_svg, render_recalibration_diagnostic_histogram,
    render_recalibration_diagnostic_ppm, render_recalibration_summary_text,
};
use crate::recalibration::CalibrationModel;

#[cfg(target_arch = "wasm32")]
const EXAMPLE_MGF_URL: &str =
    "https://raw.githubusercontent.com/zamboni-lab/MultiMS2/main/data/multims2_spectra.mgf";

#[cfg(target_arch = "wasm32")]
fn format_progress_message(processed: u64, total: u64) -> String {
    let safe_total = total.max(1);
    let displayed_processed = processed.min(safe_total);
    let percent = (displayed_processed * 100 / safe_total).min(100);
    format!("Scanning {displayed_processed}/{safe_total} bytes ({percent}%)...")
}

#[cfg(target_arch = "wasm32")]
/// Fetches an example MGF file from a remote URL and returns it as a browser blob.
///
/// # Errors
///
/// Returns an error if the request fails, the response is not ok, or the body cannot be read.
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
    metrics: Signal<Option<PrecursorMetrics>>,
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

        // Read blob as text (only once)
        let text_result = JsFuture::from(blob.text())
            .await
            .ok()
            .and_then(|v| v.as_string());

        match text_result {
            Some(content) => {
                // Store original content for download
                original_content_signal.set(content.clone());

                // Parse from the stored content
                let result = match crate::parser::parse_mgf_from_string(&content) {
                    Ok(metrics) => metrics,
                    Err(error) => {
                        status_for_progress.set(format!("Error parsing MGF: {error:?}"));
                        PrecursorMetrics::default()
                    }
                };
                metrics_for_results.set(Some(result));
            }
            None => {
                status_for_progress.set("Error reading file content".to_string());
                metrics_for_results.set(Some(PrecursorMetrics::default()));
            }
        }
        busy_for_results.set(false);
    });
}

#[cfg(target_arch = "wasm32")]
fn begin_analysis_from_blob(
    blob: Blob,
    file_name: String,
    file_name_signal: Signal<String>,
    status: Signal<String>,
    metrics: Signal<Option<PrecursorMetrics>>,
    busy: Signal<bool>,
    drag_active: Signal<bool>,
    original_mgf_content: Signal<String>,
) {
    let mut file_name_for_state = file_name_signal;
    let mut status_for_state = status;
    let mut metrics_for_state = metrics;
    let mut busy_for_state = busy;
    let mut drag_active_for_state = drag_active;
    let mut original_content_signal = original_mgf_content;

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
fn load_example_mgf(
    status: Signal<String>,
    metrics: Signal<Option<PrecursorMetrics>>,
    busy: Signal<bool>,
    file_name: Signal<String>,
    original_mgf_content: Signal<String>,
) {
    let mut status_for_progress = status;
    let mut metrics_for_results = metrics;
    let mut busy_for_results = busy;
    let mut file_name_for_results = file_name;
    let mut original_content_signal = original_mgf_content;

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

fn generate_recalibrated_mgf(
    original_content: &str,
    calibration_model: CalibrationModel,
    _diagnostics: Option<&RecalibrationDiagnostics>,
) -> String {
    #[cfg(target_arch = "wasm32")]
    use web_sys::console;

    if matches!(calibration_model, CalibrationModel::None) {
        #[cfg(target_arch = "wasm32")]
        console::log_1(&"Recalibration: model is None, returning original".into());
        return original_content.to_string();
    }

    let lambda = match calibration_model {
        CalibrationModel::TOFDa { lambda } => lambda,
        CalibrationModel::OrbitrapPPM { lambda } => lambda,
        _ => 0.0,
    };

    let mut result = String::new();
    let mut in_spectrum = false;
    let mut pepmass: Option<f64> = None;
    let mut spectrum_frags: Vec<String> = Vec::new();

    let mut spec_count = 0;
    let mut spec_with_frags = 0;
    let mut spec_recalibrated = 0;

    let lines: Vec<&str> = original_content.lines().collect();
    let mut idx = 0;

    while idx < lines.len() {
        let line = lines[idx];
        let trimmed = line.trim();

        if trimmed.eq_ignore_ascii_case("BEGIN IONS") {
            spec_count += 1;
            in_spectrum = true;
            pepmass = None;
            spectrum_frags.clear();
            result.push_str(line);
            result.push('\n');
            idx += 1;

            // Read spectrum content until END IONS
            while idx < lines.len() {
                let spec_line = lines[idx];
                let spec_trimmed = spec_line.trim();

                if spec_trimmed.eq_ignore_ascii_case("END IONS") {
                    break;
                }

                // Extract PRECURSOR_MZ or PEPMASS
                if spec_trimmed.to_uppercase().starts_with("PRECURSOR_MZ=") {
                    let pep_str = spec_trimmed[13..].split_whitespace().next().unwrap_or("0");
                    pepmass = pep_str.parse::<f64>().ok();
                    result.push_str(spec_line);
                    result.push('\n');
                    idx += 1;
                    continue;
                }
                if spec_trimmed.to_uppercase().starts_with("PEPMASS=") {
                    let pep_str = spec_trimmed[8..].split_whitespace().next().unwrap_or("0");
                    pepmass = pep_str.parse::<f64>().ok();
                    result.push_str(spec_line);
                    result.push('\n');
                    idx += 1;
                    continue;
                }

                // Check if fragment line (m/z intensity)
                let parts: Vec<&str> = spec_trimmed.split_whitespace().collect();
                if parts.len() >= 2
                    && parts[0].parse::<f64>().is_ok()
                    && parts[1].parse::<f64>().is_ok()
                {
                    // Fragment line - collect it
                    spectrum_frags.push(spec_line.to_string());
                } else {
                    // Metadata line - write immediately
                    result.push_str(spec_line);
                    result.push('\n');
                }
                idx += 1;
            }

            // Now process fragments
            if !spectrum_frags.is_empty() {
                spec_with_frags += 1;
            }

            if let Some(pm) = pepmass {
                // Find closest fragment to PEPMASS (MS2 precursor peak)
                let mut best_mz: Option<f64> = None;
                let mut best_delta = f64::INFINITY;

                for frag in &spectrum_frags {
                    let parts: Vec<&str> = frag.trim().split_whitespace().collect();
                    if let Ok(mz) = parts[0].parse::<f64>() {
                        let da = (mz - pm).abs();
                        let ppm = da * 1e6 / pm;
                        if da <= 0.02 && ppm <= 100.0 && da < best_delta {
                            best_delta = da;
                            best_mz = Some(mz);
                        }
                    }
                }

                // Recalibrate all fragments if MS2 peak found
                if let Some(ms2_peak) = best_mz {
                    spec_recalibrated += 1;
                    let delta = ms2_peak - pm;

                    for frag in &spectrum_frags {
                        let parts: Vec<&str> = frag.trim().split_whitespace().collect();
                        if parts.len() >= 2 {
                            if let (Ok(mz), Ok(intensity)) =
                                (parts[0].parse::<f64>(), parts[1].parse::<f64>())
                            {
                                let corrected_mz = match calibration_model {
                                    CalibrationModel::TOFDa { .. } => mz - lambda * delta,
                                    CalibrationModel::OrbitrapPPM { .. } => {
                                        let dppm = delta * 1e6 / pm;
                                        mz * (1.0 - lambda * dppm / 1e6)
                                    }
                                    _ => mz,
                                };
                                result.push_str(&format!("{} {}", corrected_mz, intensity));
                                for p in &parts[2..] {
                                    result.push(' ');
                                    result.push_str(p);
                                }
                                result.push('\n');
                                continue;
                            }
                        }
                        result.push_str(frag);
                        result.push('\n');
                    }
                } else {
                    for frag in &spectrum_frags {
                        result.push_str(frag);
                        result.push('\n');
                    }
                }
            } else {
                // No PEPMASS, write fragments as-is
                for frag in &spectrum_frags {
                    result.push_str(frag);
                    result.push('\n');
                }
            }

            // Write END IONS
            result.push_str("END IONS");
            result.push('\n');
            in_spectrum = false;
            idx += 1;
        } else {
            result.push_str(line);
            result.push('\n');
            idx += 1;
        }
    }

    result
}

#[cfg(target_arch = "wasm32")]
fn download_recalibrated_mgf(
    file_name: &str,
    original_content: &str,
    calibration_model: CalibrationModel,
    diagnostics: Option<&RecalibrationDiagnostics>,
) -> Result<(), String> {
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

    let recalibrated = generate_recalibrated_mgf(original_content, calibration_model, diagnostics);

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

    let url = Url::create_object_url_with_blob(&blob).map_err(|_| "Failed to create object URL")?;

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

    Ok(())
}

/// Renders the MGF precursor-error analysis UI.
///
/// # Errors
///
/// Returns an error if the component tree fails to build or render.
#[allow(clippy::too_many_lines)]
pub fn app() -> Element {
    let mut file_name = use_signal(String::new);
    let mut metrics = use_signal(|| None::<PrecursorMetrics>);
    let mut status = use_signal(|| "Drop an MGF file to begin.".to_string());
    let mut busy = use_signal(|| false);
    let mut drag_active = use_signal(|| false);
    let original_mgf_content = use_signal(String::new);

    // Recalibration control signals
    let mut calibration_model = use_signal(|| CalibrationModel::None);
    let mut lambda_value = use_signal(|| 0.5);
    let mut recalibration_diagnostics = use_signal(|| None::<RecalibrationDiagnostics>);
    let mut cumulative_dist_tab = use_signal(|| "mda"); // "mda" or "ppm"

    // Update diagnostics reactively when metrics, model, or lambda change
    use_effect(move || {
        if let Some(m) = metrics.read().as_ref() {
            update_recalibration_diagnostics(
                m,
                *calibration_model.read(),
                &mut recalibration_diagnostics,
            );
        }
    });

    let on_file_change = move |evt: Event<FormData>| {
        let Some(file) = evt.data().files().into_iter().next() else {
            status.set("No file selected.".to_string());
            return;
        };

        #[cfg(target_arch = "wasm32")]
        let Some(web_file) = file.inner().downcast_ref::<web_sys::File>() else {
            status.set("This file type is not supported in the browser.".to_string());
            return;
        };

        #[cfg(target_arch = "wasm32")]
        let Ok(blob) = web_file.clone().dyn_into::<Blob>() else {
            status.set("Unable to read the selected file as a blob.".to_string());
            return;
        };

        #[cfg(target_arch = "wasm32")]
        begin_analysis_from_blob(
            blob,
            file.name(),
            file_name,
            status,
            metrics,
            busy,
            drag_active,
            original_mgf_content,
        );

        #[cfg(not(target_arch = "wasm32"))]
        {
            file_name.set(file.name());
            metrics.set(None);
            status.set("This app needs to run in a browser.".to_string());
            busy.set(false);
        }
    };

    let on_drag_enter = move |evt: Event<DragData>| {
        evt.prevent_default();
        drag_active.set(true);
    };

    let on_drag_over = move |evt: Event<DragData>| {
        evt.prevent_default();
        drag_active.set(true);
    };

    let on_drag_leave = move |evt: Event<DragData>| {
        evt.prevent_default();
        drag_active.set(false);
    };

    let on_drop = move |evt: Event<DragData>| {
        evt.prevent_default();
        drag_active.set(false);
        let Some(file) = evt.data().files().into_iter().next() else {
            status.set("No file selected.".to_string());
            return;
        };

        #[cfg(target_arch = "wasm32")]
        let Some(web_file) = file.inner().downcast_ref::<web_sys::File>() else {
            status.set("This file type is not supported in the browser.".to_string());
            return;
        };

        #[cfg(target_arch = "wasm32")]
        let Ok(blob) = web_file.clone().dyn_into::<Blob>() else {
            status.set("Unable to read the selected file as a blob.".to_string());
            return;
        };

        #[cfg(target_arch = "wasm32")]
        begin_analysis_from_blob(
            blob,
            file.name(),
            file_name,
            status,
            metrics,
            busy,
            drag_active,
            original_mgf_content,
        );

        #[cfg(not(target_arch = "wasm32"))]
        {
            file_name.set(file.name());
            metrics.set(None);
            status.set("This app needs to run in a browser.".to_string());
            busy.set(false);
        }
    };

    rsx! {
        div {
            style: "min-height: 100vh; padding: 2rem 1rem 3rem; background: linear-gradient(135deg, #f8fafc 0%, #eef2ff 100%); color: #0f172a;",
            div {
                style: "max-width: 960px; margin: 0 auto;",
                div {
                    style: "display: flex; align-items: center; gap: 1rem; margin-bottom: 1.25rem;",
                    div {
                        h2 { style: "margin: 0; font-size: 1.7rem; letter-spacing: -0.02em;", "MGF Precursor Error" }
                        p {
                            style: "margin: 0.2rem 0 0; color: #475569; font-size: 0.95rem;",
                            "Upload an MGF file and explore precursor mass errors in Da and ppm."
                        }
                    }
                }

                div {
                    style: "background: rgba(255,255,255,0.9); border: 1px solid rgba(148,163,184,0.22); border-radius: 20px; box-shadow: 0 12px 40px rgba(15, 23, 42, 0.08); padding: 1.25rem; backdrop-filter: blur(12px);",
                    label {
                        r#for: "mgf-upload",
                        style: format!(
                            "display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 0.6rem; min-height: 140px; width: 100%; box-sizing: border-box; position: relative; isolation: isolate; border: 2px dashed {}; border-radius: 18px; padding: 1.1rem; cursor: pointer; background: {}; color: #334155; font-weight: 600; text-align: center; transition: border-color 160ms ease, background 160ms ease, transform 160ms ease;",
                            if *drag_active.read() { "#2563eb" } else { "#94a3b8" },
                            if *drag_active.read() { "linear-gradient(135deg, rgba(219,234,254,0.96), rgba(239,246,255,0.94))" } else { "linear-gradient(135deg, rgba(248,250,252,0.95), rgba(239,246,255,0.95))" }
                        ),
                        ondragenter: on_drag_enter,
                        ondragover: on_drag_over,
                        ondragleave: on_drag_leave,
                        ondrop: on_drop,
                        span { style: "font-size: 1rem;", "Drop an MGF file here or click to browse" }
                        span { style: "font-size: 0.85rem; font-weight: 500; color: #64748b;", ".mgf files only" }
                        span { style: "font-size: 0.8rem; font-weight: 500; color: #64748b;", "Plots cap at 5 mDa / 10 ppm for the signed-error views" }
                        input {
                            id: "mgf-upload",
                            r#type: "file",
                            accept: ".mgf",
                            disabled: *busy.read(),
                            onchange: on_file_change,
                            style: "position: absolute; inset: 0; width: 100%; height: 100%; opacity: 0; cursor: pointer;",
                        }
                    }

                    if file_name.read().is_empty() && metrics.read().is_none() && !(*busy.read()) {
                        button {
                            r#type: "button",
                            style: "margin-top: 0.8rem; border: 1px solid #2563eb; border-radius: 999px; background: #eff6ff; color: #1d4ed8; font-size: 0.84rem; font-weight: 700; padding: 0.45rem 0.8rem; cursor: pointer;",
                            onclick: move |_| {
                                #[cfg(target_arch = "wasm32")]
                                load_example_mgf(status, metrics, busy, file_name, original_mgf_content);
                                #[cfg(not(target_arch = "wasm32"))]
                                {
                                    status.set("This app needs to run in a browser.".to_string());
                                }
                            },
                            "Load example MGF"
                        }
                    }

                    p {
                        style: "margin: 0.7rem 0 0; color: #475569; font-size: 0.9rem;",
                        if !file_name.read().is_empty() {
                            "Selected file: {file_name}"
                        }
                    }

                    p { style: "margin: 0.7rem 0 0; font-weight: 600; color: #334155;", "{status}" }

                    if let Some(metrics) = metrics.read().as_ref() {
                        div {
                            style: "margin-top: 1rem; padding: 1rem; border: 1px solid #e2e8f0; border-radius: 16px; background: linear-gradient(180deg, #ffffff 0%, #f8fafc 100%);",
                            h3 { style: "margin: 0 0 0.4rem; font-size: 1rem;", "Summary" }
                            p { style: "margin: 0.35rem 0; color: #475569;", "Processed {metrics.total_spectra} spectra; compared {metrics.spectra} with usable reference masses." }
                            p { style: "margin: 0.35rem 0; color: #475569;", "{metrics.spectra_with_reference_mass} spectra had a usable reference mass." }

                            if metrics.skipped_spectra > 0 || !metrics.unrecognized_adducts.is_empty() {
                                div {
                                    style: "margin-top: 0.9rem; padding: 0.8rem 0.9rem; border: 1px solid #fcd34d; border-radius: 12px; background: #fffbeb; color: #92400e;",
                                    p { style: "margin: 0 0 0.35rem; font-weight: 700;", "Warnings" }
                                    p { style: "margin: 0; font-size: 0.9rem;", "{metrics.skipped_spectra} spectra were skipped because the adduct or reference mass could not be resolved." }
                                    if metrics.unparsed_smiles > 0 {
                                        p { style: "margin: 0.45rem 0 0; font-size: 0.88rem;", "{metrics.unparsed_smiles} spectra had SMILES that could not be parsed into a reference mass." }
                                    }
                                    if !metrics.unparsed_smiles_warnings.is_empty() {
                                        div { style: "margin-top: 0.6rem; padding: 0.7rem 0.8rem; border: 1px solid #fde68a; border-radius: 10px; background: #fffbeb; color: #92400e;",
                                            p { style: "margin: 0 0 0.35rem; font-weight: 700; font-size: 0.86rem;", "Excluded unparsed SMILES" }
                                            ul { style: "margin: 0.25rem 0 0 1.05rem; padding: 0; font-size: 0.84rem; max-height: 160px; overflow: auto;",
                                                {
                                                    let mut sorted_unparsed = metrics.unparsed_smiles_warnings.iter().collect::<Vec<_>>();
                                                    sorted_unparsed.sort_by(|(left_smiles, left_detail), (right_smiles, right_detail)| {
                                                        right_detail.count.cmp(&left_detail.count).then_with(|| left_smiles.cmp(right_smiles))
                                                    });
                                                    rsx! {
                                                        for (smiles, detail) in sorted_unparsed {
                                                            {
                                                                let formula_display = detail.formula.as_deref().filter(|value| !value.trim().is_empty()).map_or_else(String::new, |formula| format!(" [formula: {formula}]"));
                                                                let item_label = format!("{smiles} ({}){formula_display}", detail.count);
                                                                rsx! { li { "{item_label}" } }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    if !metrics.unrecognized_adducts.is_empty() {
                                        ul { style: "margin: 0.45rem 0 0 1.1rem; padding: 0; font-size: 0.88rem;",
                                            {
                                                let mut sorted_adducts = metrics.unrecognized_adducts.iter().collect::<Vec<_>>();
                                                sorted_adducts.sort_by(|(left_adduct, left_count), (right_adduct, right_count)| {
                                                    right_count.cmp(left_count).then_with(|| left_adduct.cmp(right_adduct))
                                                });
                                                rsx! {
                                                    for (adduct, count) in sorted_adducts {
                                                        li { "{adduct}: {count}" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            if !metrics.high_error_smiles.is_empty() {
                                div {
                                    style: "margin-top: 1rem; padding: 0.8rem 0.9rem; border: 1px solid #fecaca; border-radius: 12px; background: #fef2f2; color: #991b1b;",
                                    p { style: "margin: 0 0 0.35rem; font-weight: 700;", "SMILES for spectra above 0.01 Da" }
                                    ul { style: "margin: 0.25rem 0 0 1.1rem; padding: 0; font-size: 0.88rem; max-height: 240px; overflow: auto;",
                                        {
                                            let mut sorted_high_error = metrics.high_error_smiles.iter().collect::<Vec<_>>();
                                            sorted_high_error.sort_by(|(left_smiles, left_detail), (right_smiles, right_detail)| {
                                                right_detail.max_abs_error_da.unwrap_or_default().total_cmp(&left_detail.max_abs_error_da.unwrap_or_default()).then_with(|| right_detail.count.cmp(&left_detail.count)).then_with(|| left_smiles.cmp(right_smiles))
                                            });
                                            rsx! {
                                                for (smiles, detail) in sorted_high_error {
                                                    {
                                                        let suffix = if detail.count > 1 { format!(" (x{})", detail.count) } else { String::new() };
                                                        let calc_value = detail.calculated_mass.map_or_else(|| "n/a".to_string(), format_value);
                                                        let expected_value = detail.expected_mass.map_or_else(|| "n/a".to_string(), format_value);
                                                        let observed_value = detail.observed_precursor_mz.map_or_else(|| "n/a".to_string(), format_value);
                                                        let max_error_da = detail.max_abs_error_da.map_or_else(|| "n/a".to_string(), format_value);
                                                        let max_error_ppm = detail.max_abs_error_ppm.map_or_else(|| "n/a".to_string(), format_value);
                                                        let formula_suffix = detail.formula.as_deref().filter(|value| !value.trim().is_empty()).map_or_else(String::new, |formula| format!("; formula {formula}"));
                                                        let item_label = format!("{smiles}{suffix} — worst error {max_error_da} Da / {max_error_ppm} ppm (derived expected precursor {expected_value}; observed precursor {observed_value}; reference mass {calc_value}){formula_suffix}");
                                                        rsx! { li { "{item_label}" } }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            div {
                                style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 1rem; margin-top: 1rem;",
                                div { style: "background: linear-gradient(135deg, #ffffff 0%, #f8fafc 100%); padding: 0.9rem; border-radius: 14px; border: 1px solid #e2e8f0; box-shadow: 0 10px 24px rgba(15, 23, 42, 0.06);",
                                    h4 { style: "margin: 0 0 0.35rem; font-size: 0.95rem; color: #0f172a;", "Observed precursor 𝑚/𝑧 distribution" }
                                    p { style: "margin: 0; color: #64748b; font-size: 0.8rem;", "Range, mean, and median of the observed precursor 𝑚/𝑧 values" }
                                    div { style: "display: flex; flex-wrap: wrap; gap: 0.45rem; margin-top: 0.6rem;",
                                        span { style: "display: inline-flex; align-items: center; gap: 0.3rem; padding: 0.35rem 0.6rem; border-radius: 999px; background: #eff6ff; color: #1d4ed8; border: 1px solid #bfdbfe; font-size: 0.78rem; font-weight: 700;", "median {format_value(metrics.observed_precursor_median)}" }
                                        span { style: "display: inline-flex; align-items: center; gap: 0.3rem; padding: 0.35rem 0.6rem; border-radius: 999px; background: #f8fafc; color: #334155; border: 1px solid #e2e8f0; font-size: 0.78rem; font-weight: 700;", "mean {format_value(metrics.observed_precursor_mean)}" }
                                        span { style: "display: inline-flex; align-items: center; gap: 0.3rem; padding: 0.35rem 0.6rem; border-radius: 999px; background: #f8fafc; color: #334155; border: 1px solid #e2e8f0; font-size: 0.78rem; font-weight: 700;", "range {format_value(metrics.observed_precursor_max - metrics.observed_precursor_min)}" }
                                    }
                                }
                                div { style: "background: linear-gradient(135deg, #ffffff 0%, #f8fafc 100%); padding: 0.9rem; border-radius: 14px; border: 1px solid #e2e8f0; box-shadow: 0 10px 24px rgba(15, 23, 42, 0.06);",
                                    h4 { style: "margin: 0 0 0.35rem; font-size: 0.95rem; color: #0f172a;", "Absolute precursor error (Da)" }
                                    p { style: "margin: 0; color: #64748b; font-size: 0.8rem;", "Median absolute deviation, mean absolute deviation, and RMS in daltons" }
                                    div { style: "display: flex; flex-wrap: wrap; gap: 0.45rem; margin-top: 0.6rem;",
                                        span { style: "display: inline-flex; align-items: center; gap: 0.3rem; padding: 0.35rem 0.6rem; border-radius: 999px; background: #eff6ff; color: #1d4ed8; border: 1px solid #bfdbfe; font-size: 0.78rem; font-weight: 700;", "median {format_value(metrics.abs_error_da_median)}" }
                                        span { style: "display: inline-flex; align-items: center; gap: 0.3rem; padding: 0.35rem 0.6rem; border-radius: 999px; background: #f8fafc; color: #334155; border: 1px solid #e2e8f0; font-size: 0.78rem; font-weight: 700;", "mean {format_value(metrics.abs_error_da_mean)}" }
                                        span { style: "display: inline-flex; align-items: center; gap: 0.3rem; padding: 0.35rem 0.6rem; border-radius: 999px; background: #f8fafc; color: #334155; border: 1px solid #e2e8f0; font-size: 0.78rem; font-weight: 700;", "RMS {format_value(metrics.abs_error_da_rms)}" }
                                    }
                                }
                                div { style: "background: linear-gradient(135deg, #ffffff 0%, #f8fafc 100%); padding: 0.9rem; border-radius: 14px; border: 1px solid #e2e8f0; box-shadow: 0 10px 24px rgba(15, 23, 42, 0.06);",
                                    h4 { style: "margin: 0 0 0.35rem; font-size: 0.95rem; color: #0f172a;", "Relative precursor error (ppm)" }
                                    p { style: "margin: 0; color: #64748b; font-size: 0.8rem;", "Median relative deviation, mean relative deviation, and RMS of the ppm error" }
                                    div { style: "display: flex; flex-wrap: wrap; gap: 0.45rem; margin-top: 0.6rem;",
                                        span { style: "display: inline-flex; align-items: center; gap: 0.3rem; padding: 0.35rem 0.6rem; border-radius: 999px; background: #eff6ff; color: #1d4ed8; border: 1px solid #bfdbfe; font-size: 0.78rem; font-weight: 700;", "median {format_value(metrics.abs_error_ppm_median)}" }
                                        span { style: "display: inline-flex; align-items: center; gap: 0.3rem; padding: 0.35rem 0.6rem; border-radius: 999px; background: #f8fafc; color: #334155; border: 1px solid #e2e8f0; font-size: 0.78rem; font-weight: 700;", "mean {format_value(metrics.abs_error_ppm_mean)}" }
                                        span { style: "display: inline-flex; align-items: center; gap: 0.3rem; padding: 0.35rem 0.6rem; border-radius: 999px; background: #f8fafc; color: #334155; border: 1px solid #e2e8f0; font-size: 0.78rem; font-weight: 700;", "RMS {format_value(metrics.abs_error_ppm_rms)}" }
                                    }
                                }
                            }

                            div {
                                style: "margin-top: 1rem; padding: 0.95rem 1rem; border: 1px solid #e2e8f0; border-radius: 16px; background: linear-gradient(180deg, #ffffff 0%, #f8fafc 100%); box-shadow: 0 10px 24px rgba(15, 23, 42, 0.06);",
                                h4 { style: "margin: 0 0 0.25rem; font-size: 0.95rem; color: #0f172a;", "Tolerance-band compliance" }
                                p { style: "margin: 0 0 0.7rem; color: #64748b; font-size: 0.84rem;", "Counts of spectra up to each reported mass-error cutoff (cumulative)" }
                                div { style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 0.6rem;",
                                    div { style: tolerance_card_style(0),
                                        strong { style: "display:block; font-size: 0.8rem; margin-bottom: 0.25rem;", "≤ 0.1 mDa" }
                                        span { style: "font-size: 0.88rem; font-weight: 700;", "{format_cumulative_bucket_count(metrics, \"0.1_da\", metrics.spectra)}" }
                                    }
                                    div { style: tolerance_card_style(1),
                                        strong { style: "display:block; font-size: 0.8rem; margin-bottom: 0.25rem;", "≤ 0.5 mDa" }
                                        span { style: "font-size: 0.88rem; font-weight: 700;", "{format_cumulative_bucket_count(metrics, \"0.5_da\", metrics.spectra)}" }
                                    }
                                    div { style: tolerance_card_style(2),
                                        strong { style: "display:block; font-size: 0.8rem; margin-bottom: 0.25rem;", "≤ 1.0 mDa" }
                                        span { style: "font-size: 0.88rem; font-weight: 700;", "{format_cumulative_bucket_count(metrics, \"1.0_da\", metrics.spectra)}" }
                                    }
                                    div { style: tolerance_card_style(3),
                                        strong { style: "display:block; font-size: 0.8rem; margin-bottom: 0.25rem;", "≤ 5.0 mDa" }
                                        span { style: "font-size: 0.88rem; font-weight: 700;", "{format_cumulative_bucket_count(metrics, \"5.0_da\", metrics.spectra)}" }
                                    }
                                    div { style: tolerance_card_style(4),
                                        strong { style: "display:block; font-size: 0.8rem; margin-bottom: 0.25rem;", "> 5.0 mDa" }
                                        span { style: "font-size: 0.88rem; font-weight: 700;", "{format_cumulative_bucket_count(metrics, \">5.0_da\", metrics.spectra)}" }
                                    }
                                    div { style: tolerance_card_style(0),
                                        strong { style: "display:block; font-size: 0.8rem; margin-bottom: 0.25rem;", "≤ 0.5 ppm" }
                                        span { style: "font-size: 0.88rem; font-weight: 700;", "{format_cumulative_bucket_count(metrics, \"0.5_ppm\", metrics.spectra)}" }
                                    }
                                    div { style: tolerance_card_style(1),
                                        strong { style: "display:block; font-size: 0.8rem; margin-bottom: 0.25rem;", "≤ 1.0 ppm" }
                                        span { style: "font-size: 0.88rem; font-weight: 700;", "{format_cumulative_bucket_count(metrics, \"1.0_ppm\", metrics.spectra)}" }
                                    }
                                    div { style: tolerance_card_style(2),
                                        strong { style: "display:block; font-size: 0.8rem; margin-bottom: 0.25rem;", "≤ 5.0 ppm" }
                                        span { style: "font-size: 0.88rem; font-weight: 700;", "{format_cumulative_bucket_count(metrics, \"5.0_ppm\", metrics.spectra)}" }
                                    }
                                    div { style: tolerance_card_style(3),
                                        strong { style: "display:block; font-size: 0.8rem; margin-bottom: 0.25rem;", "≤ 10.0 ppm" }
                                        span { style: "font-size: 0.88rem; font-weight: 700;", "{format_cumulative_bucket_count(metrics, \"10.0_ppm\", metrics.spectra)}" }
                                    }
                                    div { style: tolerance_card_style(4),
                                        strong { style: "display:block; font-size: 0.8rem; margin-bottom: 0.25rem;", "> 10.0 ppm" }
                                        span { style: "font-size: 0.88rem; font-weight: 700;", "{format_cumulative_bucket_count(metrics, \">10.0_ppm\", metrics.spectra)}" }
                                    }
                                }
                            }

                            // Recalibration control panel
                            div {
                                style: "margin-top: 1.5rem; padding: 1.2rem; border: 2px solid #3b82f6; border-radius: 16px; background: linear-gradient(135deg, #dbeafe 0%, #eff6ff 100%);",
                                h3 { style: "margin: 0 0 0.8rem; font-size: 1.1rem; color: #1e40af;", "🔬 MS2 Fragment Recalibration" }
                                p { style: "margin: 0 0 1rem; color: #1e40af; font-size: 0.95rem;", "Apply precursor-driven recalibration to MS2 fragments using the discrepancy between MS1 and MS2 precursor m/z." }

                                div {
                                    style: "display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; margin-bottom: 1rem;",

                                    div {
                                        label { style: "display: block; font-weight: 600; color: #1e40af; margin-bottom: 0.4rem;", "Calibration Model" }
                                        select {
                                            value: match *calibration_model.read() {
                                                CalibrationModel::None => "none",
                                                CalibrationModel::TOFDa { .. } => "tof",
                                                CalibrationModel::OrbitrapPPM { .. } => "orbitrap",
                                            },
                                            onchange: move |evt| {
                                                let value = evt.value();
                                                let lambda = *lambda_value.read();
                                                calibration_model.set(match value.as_str() {
                                                    "tof" => CalibrationModel::TOFDa { lambda },
                                                    "orbitrap" => CalibrationModel::OrbitrapPPM { lambda },
                                                    _ => CalibrationModel::None,
                                                });
                                            },
                                            style: "width: 100%; padding: 0.5rem; border: 1px solid #0ea5e9; border-radius: 8px; background: white; color: #1e40af; font-weight: 500; cursor: pointer;",
                                            option { value: "none", "None (No Correction)" }
                                            option { value: "tof", "TOF (Absolute Da)" }
                                            option { value: "orbitrap", "Orbitrap (ppm)" }
                                        }
                                    }

                                    if !matches!(*calibration_model.read(), CalibrationModel::None) {
                                        div {
                                            label {
                                                style: "display: block; font-weight: 600; color: #1e40af; margin-bottom: 0.4rem;",
                                                "Lambda (λ): {format_lambda(*lambda_value.read())}"
                                            }
                                            input {
                                                r#type: "range",
                                                min: "0.0",
                                                max: "1.0",
                                                step: "0.05",
                                                value: format!("{}", lambda_value.read()),
                                                oninput: move |evt| {
                                                    let val: f64 = evt.value().parse().unwrap_or(0.5);
                                                    lambda_value.set(val);

                                                    // Update model with new lambda
                                                    let current_model = *calibration_model.read();
                                                    let new_model = match current_model {
                                                        CalibrationModel::TOFDa { .. } => CalibrationModel::TOFDa { lambda: val },
                                                        CalibrationModel::OrbitrapPPM { .. } => CalibrationModel::OrbitrapPPM { lambda: val },
                                                        _ => CalibrationModel::None,
                                                    };
                                                    calibration_model.set(new_model);
                                                },
                                                style: "width: 100%; cursor: pointer;",
                                            }
                                            p { style: "margin: 0.4rem 0 0; font-size: 0.85rem; color: #0ea5e9;", "0 = no correction, 1 = full correction" }
                                        }
                                    }
                                }
                            }

                            // Tolerance-band compliance AFTER recalibration (duplicated)
                            div {
                                style: "margin-top: 1rem; padding: 0.95rem 1rem; border: 1px solid #e2e8f0; border-radius: 16px; background: linear-gradient(180deg, #ffffff 0%, #f8fafc 100%); box-shadow: 0 10px 24px rgba(15, 23, 42, 0.06);",
                                h4 { style: "margin: 0 0 0.25rem; font-size: 0.95rem; color: #0f172a;", "Tolerance-band compliance (Recalibrated)" }
                                p { style: "margin: 0 0 0.7rem; color: #64748b; font-size: 0.84rem;", "Estimated compliance after MS2 fragment recalibration (preview)" }
                                div { style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 0.6rem;",
                                    if let Some(diag) = recalibration_diagnostics.read().as_ref() {
                                        // mDa after
                                        div { style: tolerance_card_style(0),
                                            strong { style: "display:block; font-size: 0.8rem; margin-bottom: 0.25rem;", "≤ 0.1 mDa" }
                                            span { style: "font-size: 0.88rem; font-weight: 700;", "{format_value(estimate_compliance_mda(&diag.error_da_after, 0.1))}" }
                                        }
                                        div { style: tolerance_card_style(1),
                                            strong { style: "display:block; font-size: 0.8rem; margin-bottom: 0.25rem;", "≤ 0.5 mDa" }
                                            span { style: "font-size: 0.88rem; font-weight: 700;", "{format_value(estimate_compliance_mda(&diag.error_da_after, 0.5))}" }
                                        }
                                        div { style: tolerance_card_style(2),
                                            strong { style: "display:block; font-size: 0.8rem; margin-bottom: 0.25rem;", "≤ 1.0 mDa" }
                                            span { style: "font-size: 0.88rem; font-weight: 700;", "{format_value(estimate_compliance_mda(&diag.error_da_after, 1.0))}" }
                                        }
                                        div { style: tolerance_card_style(3),
                                            strong { style: "display:block; font-size: 0.8rem; margin-bottom: 0.25rem;", "≤ 5.0 mDa" }
                                            span { style: "font-size: 0.88rem; font-weight: 700;", "{format_value(estimate_compliance_mda(&diag.error_da_after, 5.0))}" }
                                        }
                                        div { style: tolerance_card_style(4),
                                            strong { style: "display:block; font-size: 0.8rem; margin-bottom: 0.25rem;", "> 5.0 mDa" }
                                            span { style: "font-size: 0.88rem; font-weight: 700;", "{format_value(100.0 - estimate_compliance_mda(&diag.error_da_after, 5.0))}" }
                                        }
                                        // ppm after
                                        div { style: tolerance_card_style(0),
                                            strong { style: "display:block; font-size: 0.8rem; margin-bottom: 0.25rem;", "≤ 0.5 ppm" }
                                            span { style: "font-size: 0.88rem; font-weight: 700;", "{format_value(estimate_compliance_ppm(&diag.error_ppm_after, 0.5))}" }
                                        }
                                        div { style: tolerance_card_style(1),
                                            strong { style: "display:block; font-size: 0.8rem; margin-bottom: 0.25rem;", "≤ 1.0 ppm" }
                                            span { style: "font-size: 0.88rem; font-weight: 700;", "{format_value(estimate_compliance_ppm(&diag.error_ppm_after, 1.0))}" }
                                        }
                                        div { style: tolerance_card_style(2),
                                            strong { style: "display:block; font-size: 0.8rem; margin-bottom: 0.25rem;", "≤ 5.0 ppm" }
                                            span { style: "font-size: 0.88rem; font-weight: 700;", "{format_value(estimate_compliance_ppm(&diag.error_ppm_after, 5.0))}" }
                                        }
                                        div { style: tolerance_card_style(3),
                                            strong { style: "display:block; font-size: 0.8rem; margin-bottom: 0.25rem;", "≤ 10.0 ppm" }
                                            span { style: "font-size: 0.88rem; font-weight: 700;", "{format_value(estimate_compliance_ppm(&diag.error_ppm_after, 10.0))}" }
                                        }
                                        div { style: tolerance_card_style(4),
                                            strong { style: "display:block; font-size: 0.8rem; margin-bottom: 0.25rem;", "> 10.0 ppm" }
                                            span { style: "font-size: 0.88rem; font-weight: 700;", "{format_value(100.0 - estimate_compliance_ppm(&diag.error_ppm_after, 10.0))}" }
                                        }
                                    }
                                }
                            }

                            // Download recalibrated MGF button
                            if !original_mgf_content.read().is_empty() && !matches!(*calibration_model.read(), CalibrationModel::None) {
                                button {
                                    r#type: "button",
                                    style: "margin-top: 1rem; width: 100%; padding: 0.75rem 1rem; border: 2px solid #10b981; border-radius: 8px; background: #10b981; color: white; font-size: 0.95rem; font-weight: 700; cursor: pointer; transition: background 0.2s; hover:background #059669;",
                                    onclick: move |_| {
                                        #[cfg(target_arch = "wasm32")]
                                        {
                                            let file_name = file_name.read();
                                            let content = original_mgf_content.read();
                                            let model = *calibration_model.read();
                                            let diag = recalibration_diagnostics.read().clone();

                                            if let Err(e) = download_recalibrated_mgf(&file_name, &content, model, diag.as_ref()) {
                                                status.set(format!("Download error: {}", e));
                                            } else {
                                                status.set(format!("Downloaded: {}_recalibrated.mgf", if file_name.ends_with(".mgf") { &file_name[..file_name.len()-4] } else { &file_name }));
                                            }
                                        }
                                    },
                                    "Download Recalibrated MGF"
                                }
                            }

                            // Recalibration diagnostics display
                            if let Some(diag) = recalibration_diagnostics.read().as_ref() {
                                div {
                                    style: "margin-top: 1.5rem; padding: 1rem; border: 1px solid #e2e8f0; border-radius: 16px; background: linear-gradient(180deg, #ffffff 0%, #f8fafc 100%);",

                                    h4 { style: "margin: 0 0 1rem; font-size: 1rem; color: #1e40af;", "Recalibration Diagnostics" }

                                    // Summary statistics table
                                    div {
                                        style: "margin-bottom: 1.5rem;",
                                        dangerous_inner_html: render_recalibration_summary_text(
                                            diag.mean_error_ppm_before,
                                            diag.mean_error_ppm_after,
                                            diag.rms_error_ppm_before,
                                            diag.rms_error_ppm_after,
                                            diag.max_abs_error_ppm_before,
                                            diag.max_abs_error_ppm_after,
                                        ),
                                    }

                                    // Tabbed cumulative error distribution (ms1, ms2_before, ms2_after)
                                    div {
                                        style: "margin-bottom: 1.5rem;",
                                        h5 { style: "margin: 0 0 0.5rem; font-size: 0.95rem; color: #1e40af;", "Cumulative Error Distribution" }

                                        // Tab buttons
                                        div {
                                            style: "display: flex; gap: 0.5rem; margin-bottom: 0.8rem; border-bottom: 1px solid #e2e8f0; padding-bottom: 0.5rem;",
                                            button {
                                                style: if *cumulative_dist_tab.read() == "mda" {
                                                    "padding: 0.5rem 1rem; border: 2px solid #1e40af; background: #1e40af; color: white; border-radius: 6px; font-weight: 600; cursor: pointer;"
                                                } else {
                                                    "padding: 0.5rem 1rem; border: 1px solid #e2e8f0; background: white; color: #64748b; border-radius: 6px; cursor: pointer;"
                                                },
                                                onclick: move |_| {
                                                    cumulative_dist_tab.set("mda");
                                                },
                                                "mDa (Absolute)"
                                            }
                                            button {
                                                style: if *cumulative_dist_tab.read() == "ppm" {
                                                    "padding: 0.5rem 1rem; border: 2px solid #1e40af; background: #1e40af; color: white; border-radius: 6px; font-weight: 600; cursor: pointer;"
                                                } else {
                                                    "padding: 0.5rem 1rem; border: 1px solid #e2e8f0; background: white; color: #64748b; border-radius: 6px; cursor: pointer;"
                                                },
                                                onclick: move |_| {
                                                    cumulative_dist_tab.set("ppm");
                                                },
                                                "ppm (Relative)"
                                            }
                                        }

                                        // Tab content
                                        div {
                                            style: "background: white; padding: 1rem; border: 1px solid #e2e8f0; border-radius: 12px; overflow-x: auto;",
                                            p {
                                                style: "margin: 0 0 0.8rem; font-size: 0.9rem; color: #64748b;",
                                                strong { "Legend: " }
                                                "🔵 Blue = MS1 precursor (PEPMASS) vs theoretical | "
                                                "🟠 Orange = MS2 precursor before correction vs theoretical | "
                                                "🟢 Green = MS2 precursor after recalibration vs theoretical"
                                            }
                                            if *cumulative_dist_tab.read() == "mda" {
                                                div {
                                                    dangerous_inner_html: render_cumulative_error_three_curves(
                                                        &diag.error_da_ms1,
                                                        &diag.error_da_before,
                                                        &diag.error_da_after,
                                                        "mDa",
                                                        vec![0.1, 0.5, 1.0, 5.0],
                                                    ),
                                                }
                                            } else {
                                                div {
                                                    dangerous_inner_html: render_cumulative_error_three_curves(
                                                        &diag.error_ppm_ms1,
                                                        &diag.error_ppm_before,
                                                        &diag.error_ppm_after,
                                                        "ppm",
                                                        vec![0.5, 1.0, 5.0, 10.0],
                                                    ),
                                                }
                                            }
                                        }
                                    }

                                    // Error time series (supporting detail)
                                    div {
                                        style: "margin-bottom: 1.5rem;",
                                        h5 { style: "margin: 0 0 0.5rem; font-size: 0.95rem; color: #1e40af;", "Precursor error over time" }
                                        p { style: "margin: 0 0 0.8rem; font-size: 0.9rem; color: #64748b;",
                                            strong { "Legend: " }
                                            "🔵 Blue = MS2 before correction | "
                                            "🟢 Green = MS2 after correction" }
                                        div {
                                            style: "background: white; padding: 1rem; border: 1px solid #e2e8f0; border-radius: 12px; overflow-x: auto;",
                                            dangerous_inner_html: render_recalibration_diagnostic_ppm(
                                                &diag.error_ppm_before,
                                                &diag.error_ppm_after,
                                            ),
                                        }
                                    }

                                    // Histogram (supporting detail)
                                    div {
                                        h5 { style: "margin: 0 0 0.5rem; font-size: 0.95rem; color: #1e40af;", "Error distribution" }
                                        p { style: "margin: 0 0 0.8rem; font-size: 0.9rem; color: #64748b;",
                                            strong { "Legend: " }
                                            "🔵 Blue = MS2 before correction | "
                                            "🟢 Green = MS2 after correction" }
                                        div {
                                            style: "background: white; padding: 1rem; border: 1px solid #e2e8f0; border-radius: 12px; overflow-x: auto;",
                                            dangerous_inner_html: render_recalibration_diagnostic_histogram(
                                                &diag.error_ppm_before,
                                                &diag.error_ppm_after,
                                                20,
                                            ),
                                        }
                                    }
                                }
                            }

                            div {
                                if let Some(diag) = recalibration_diagnostics.read().as_ref() {
                                    ecdf_plot {
                                        title: "Absolute precursor-error cumulative distribution (mDa)".to_string(),
                                        subtitle: "Cumulative fraction below each tolerance cutoff, shown on a log10 scale".to_string(),
                                        values: diag.error_da_before.iter().chain(diag.error_da_after.iter()).copied().collect::<Vec<_>>(),
                                        thresholds: vec![0.1, 0.5, 1.0, 5.0],
                                        unit: "mDa".to_string(),
                                    }
                                    ecdf_plot {
                                        title: "Relative precursor-error cumulative distribution (ppm)".to_string(),
                                        subtitle: "Cumulative fraction below each ppm tolerance cutoff, shown on a log10 scale".to_string(),
                                        values: diag.error_ppm_before.iter().chain(diag.error_ppm_after.iter()).copied().collect::<Vec<_>>(),
                                        thresholds: vec![0.5, 1.0, 5.0, 10.0],
                                        unit: "ppm".to_string(),
                                    }
                                } else {
                                    ecdf_plot {
                                        title: "Absolute precursor-error cumulative distribution (mDa)".to_string(),
                                        subtitle: "Cumulative fraction below each tolerance cutoff, shown on a log10 scale".to_string(),
                                        values: metrics.absolute_error_da_values.iter().map(|value| value * 1000.0).collect::<Vec<_>>(),
                                        thresholds: vec![0.1, 0.5, 1.0, 5.0],
                                        unit: "mDa".to_string(),
                                    }
                                    ecdf_plot {
                                        title: "Relative precursor-error cumulative distribution (ppm)".to_string(),
                                        subtitle: "Cumulative fraction below each ppm tolerance cutoff, shown on a log10 scale".to_string(),
                                        values: metrics.absolute_error_ppm_values.clone(),
                                        thresholds: vec![0.5, 1.0, 5.0, 10.0],
                                        unit: "ppm".to_string(),
                                    }
                                }
                                absolute_mass_bias_plot {
                                    title: "Signed error vs. precursor 𝑚/𝑧 (mDa)".to_string(),
                                    subtitle: "Signed error at each precursor 𝑚/𝑧, centered on zero and shown with a symmetric y-axis".to_string(),
                                    points: metrics.plot_points.clone(),
                                    unit: "mDa".to_string(),
                                    ticks: vec![0.1, 0.5, 1.0, 5.0],
                                }
                                absolute_mass_bias_plot {
                                    title: "Signed relative error vs. precursor 𝑚/𝑧 (ppm)".to_string(),
                                    subtitle: "Signed relative error at each precursor 𝑚/𝑧, centered on zero and shown with a symmetric y-axis".to_string(),
                                    points: metrics.plot_points.clone(),
                                    unit: "ppm".to_string(),
                                    ticks: vec![0.5, 1.0, 5.0, 10.0],
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn format_value(value: f64) -> String {
    if value.is_finite() {
        format!("{value:.4}")
    } else {
        "n/a".to_string()
    }
}

fn format_count_with_percentage(count: usize, total: usize) -> String {
    if total == 0 {
        format!("{count} (0.0%)")
    } else {
        let count = u32::try_from(count).unwrap_or(u32::MAX);
        let total = u32::try_from(total).unwrap_or(u32::MAX);
        let pct = (f64::from(count) / f64::from(total)) * 100.0;
        format!("{count} ({pct:.1}%)")
    }
}

fn format_cumulative_bucket_count(
    metrics: &PrecursorMetrics,
    bucket: &str,
    total: usize,
) -> String {
    let count = match bucket {
        "0.1_da" => metrics.within_0_1_da,
        "0.5_da" => metrics.within_0_1_da + metrics.within_0_5_da,
        "1.0_da" => metrics.within_0_1_da + metrics.within_0_5_da + metrics.within_1_da,
        "5.0_da" => {
            metrics.within_0_1_da
                + metrics.within_0_5_da
                + metrics.within_1_da
                + metrics.within_5_da
        }
        ">5.0_da" => metrics.above_5_da,
        "0.5_ppm" => metrics.within_0_5_ppm,
        "1.0_ppm" => metrics.within_0_5_ppm + metrics.within_1_ppm,
        "5.0_ppm" => metrics.within_0_5_ppm + metrics.within_1_ppm + metrics.within_5_ppm,
        "10.0_ppm" => {
            metrics.within_0_5_ppm
                + metrics.within_1_ppm
                + metrics.within_5_ppm
                + metrics.within_10_ppm
        }
        ">10.0_ppm" => metrics.above_10_ppm,
        _ => 0,
    };
    format_count_with_percentage(count, total)
}

fn tolerance_card_style(index: usize) -> String {
    let color = crate::plotting::tolerance_step_color(index, 5);
    format!(
        "padding: 0.6rem 0.7rem; border-radius: 12px; border: 1px solid {color}; background: #f8fafc; color: {color};"
    )
}

fn estimate_compliance_mda(errors: &[f64], threshold_mda: f64) -> f64 {
    if errors.is_empty() {
        return 0.0;
    }
    let threshold_da = threshold_mda / 1000.0;
    let count = errors.iter().filter(|e| e.abs() <= threshold_da).count();
    (count as f64 / errors.len() as f64) * 100.0
}

fn estimate_compliance_ppm(errors: &[f64], threshold_ppm: f64) -> f64 {
    if errors.is_empty() {
        return 0.0;
    }
    let count = errors.iter().filter(|e| e.abs() <= threshold_ppm).count();
    (count as f64 / errors.len() as f64) * 100.0
}

#[component]
fn ecdf_plot(
    title: String,
    subtitle: String,
    values: Vec<f64>,
    thresholds: Vec<f64>,
    unit: String,
) -> Element {
    let title_for_svg = title.clone();
    let values_for_svg = values;
    let thresholds_for_svg = thresholds;
    let unit_for_svg = unit;
    let svg_markup = use_memo(move || {
        make_svg_responsive(render_ecdf_svg(
            &title_for_svg,
            &values_for_svg,
            &thresholds_for_svg,
            &unit_for_svg,
        ))
    });
    let svg_markup = svg_markup.read().clone();
    #[cfg(target_arch = "wasm32")]
    let download_markup = svg_markup.clone();

    rsx! {
        div {
            style: "padding: 0.95rem; border: 1px solid #e2e8f0; border-radius: 18px; background: linear-gradient(180deg, #ffffff 0%, #f8fafc 100%); box-shadow: 0 12px 24px rgba(15, 23, 42, 0.04);",
            div { style: "display: flex; align-items: center; justify-content: space-between; gap: 0.6rem; margin-bottom: 0.65rem;",
                div { style: "flex: 1;",
                    h4 { style: "margin: 0 0 0.2rem; font-size: 0.95rem; color: #0f172a;", "{title}" }
                    p { style: "margin: 0; color: #64748b; font-size: 0.84rem;", "{subtitle}" }
                }
                button {
                    r#type: "button",
                    style: "border: 1px solid #cbd5e1; border-radius: 999px; background: white; color: #334155; font-size: 0.76rem; font-weight: 700; padding: 0.35rem 0.65rem; cursor: pointer;",
                    onclick: move |_| {
                        #[cfg(target_arch = "wasm32")]
                        download_svg(&download_markup, &title);
                    },
                    "Download"
                }
            }
            div { style: "border-radius: 16px; overflow: visible; border: 1px solid #e2e8f0; background: #fcfdff;",
                dangerous_inner_html: svg_markup
            }
        }
    }
}

#[component]
fn mass_bias_plot(
    title: String,
    subtitle: String,
    points: Vec<PlotPoint>,
    other_label: Option<String>,
) -> Element {
    let title_for_svg = title.clone();
    let points_for_svg = points;
    let svg_markup = use_memo(move || {
        make_svg_responsive(render_mass_bias_svg(&title_for_svg, &points_for_svg))
    });
    let svg_markup = svg_markup.read().clone();
    #[cfg(target_arch = "wasm32")]
    let download_markup = svg_markup.clone();

    rsx! {
        div {
            style: "padding: 0.95rem; border: 1px solid #e2e8f0; border-radius: 18px; background: linear-gradient(180deg, #ffffff 0%, #f8fafc 100%); box-shadow: 0 12px 24px rgba(15, 23, 42, 0.04);",
            div { style: "display: flex; align-items: center; justify-content: space-between; gap: 0.6rem; margin-bottom: 0.65rem;",
                div { style: "flex: 1;",
                    h4 { style: "margin: 0 0 0.2rem; font-size: 0.95rem; color: #0f172a;", "{title}" }
                    p { style: "margin: 0; color: #64748b; font-size: 0.84rem;", "{subtitle}" }
                }
                button {
                    r#type: "button",
                    style: "border: 1px solid #cbd5e1; border-radius: 999px; background: white; color: #334155; font-size: 0.76rem; font-weight: 700; padding: 0.35rem 0.65rem; cursor: pointer;",
                    onclick: move |_| {
                        #[cfg(target_arch = "wasm32")]
                        download_svg(&download_markup, &title);
                    },
                    "Download"
                }
            }
            div { style: "border-radius: 16px; overflow: visible; border: 1px solid #e2e8f0; background: #fcfdff;",
                dangerous_inner_html: svg_markup
            }
        }
    }
}

#[component]
fn absolute_mass_bias_plot(
    title: String,
    subtitle: String,
    points: Vec<PlotPoint>,
    unit: String,
    ticks: Vec<f64>,
) -> Element {
    let title_for_svg = title.clone();
    let points_for_svg = points;
    let unit_for_svg = unit;
    let ticks_for_svg = ticks;
    let svg_markup = use_memo(move || {
        make_svg_responsive(render_absolute_mass_bias_svg(
            &title_for_svg,
            &points_for_svg,
            &unit_for_svg,
            &ticks_for_svg,
        ))
    });
    let svg_markup = svg_markup.read().clone();
    #[cfg(target_arch = "wasm32")]
    let download_markup = svg_markup.clone();

    rsx! {
        div {
            style: "padding: 0.95rem; border: 1px solid #e2e8f0; border-radius: 18px; background: linear-gradient(180deg, #ffffff 0%, #f8fafc 100%); box-shadow: 0 12px 24px rgba(15, 23, 42, 0.04);",
            div { style: "display: flex; align-items: center; justify-content: space-between; gap: 0.6rem; margin-bottom: 0.65rem;",
                div { style: "flex: 1;",
                    h4 { style: "margin: 0 0 0.2rem; font-size: 0.95rem; color: #0f172a;", "{title}" }
                    p { style: "margin: 0; color: #64748b; font-size: 0.84rem;", "{subtitle}" }
                }
                button {
                    r#type: "button",
                    style: "border: 1px solid #cbd5e1; border-radius: 999px; background: white; color: #334155; font-size: 0.76rem; font-weight: 700; padding: 0.35rem 0.65rem; cursor: pointer;",
                    onclick: move |_| {
                        #[cfg(target_arch = "wasm32")]
                        download_svg(&download_markup, &title);
                    },
                    "Download"
                }
            }
            div { style: "border-radius: 16px; overflow: visible; border: 1px solid #e2e8f0; background: #fcfdff;",
                dangerous_inner_html: svg_markup
            }
        }
    }
}

fn format_lambda(lambda: f64) -> String {
    format!("{:.2}", lambda)
}

#[cfg(target_arch = "wasm32")]
fn update_recalibration_diagnostics(
    metrics: &PrecursorMetrics,
    model: CalibrationModel,
    diagnostics_signal: &mut Signal<Option<RecalibrationDiagnostics>>,
) {
    if matches!(model, CalibrationModel::None) {
        diagnostics_signal.set(None);
        return;
    }

    let mut diag = RecalibrationDiagnostics::new();
    let lambda = match model {
        CalibrationModel::TOFDa { lambda } => lambda,
        CalibrationModel::OrbitrapPPM { lambda } => lambda,
        _ => 0.0,
    };

    // Process each plot point
    for point in &metrics.plot_points {
        // Skip if no theoretical mass available
        let Some(theoretical_mass) = point.expected_mass else {
            continue;
        };

        // Use actual MS2 precursor peak if observed, otherwise fall back to PEPMASS header
        let precursor_ms2 = point.ms2_precursor_peak.unwrap_or(point.pepmass_header);

        // MS1 precursor: PEPMASS header is our estimate
        // (When actual MS1 data is available, use that instead)
        let precursor_ms1 = point.pepmass_header;

        // Stage 1: error_ms1 = PEPMASS - theoretical
        let error_da_ms1 = precursor_ms1 - theoretical_mass;
        let error_ppm_ms1 = if theoretical_mass > 0.0 {
            error_da_ms1 * 1e6 / theoretical_mass
        } else {
            0.0
        };

        // Stage 2: error_ms2_before = MS2 - theoretical
        let error_da_before = precursor_ms2 - theoretical_mass;
        let error_ppm_before = if theoretical_mass > 0.0 {
            error_da_before * 1e6 / theoretical_mass
        } else {
            0.0
        };

        // Stage 3: delta_ms2_ms1 = MS2 - MS1
        let delta_ms2_ms1_da = precursor_ms2 - precursor_ms1;
        let delta_ppm_ms2_ms1 = if precursor_ms1 > 0.0 {
            delta_ms2_ms1_da * 1e6 / precursor_ms1
        } else {
            0.0
        };

        // Apply recalibration: error_ms2_after = (MS2 - λ × delta) - theoretical
        let precursor_ms2_after = match model {
            CalibrationModel::TOFDa { .. } => precursor_ms2 - lambda * delta_ms2_ms1_da,
            CalibrationModel::OrbitrapPPM { .. } => {
                precursor_ms2 * (1.0 - lambda * delta_ppm_ms2_ms1 / 1e6)
            }
            _ => precursor_ms2,
        };

        // Stage 4: error_ms2_after = (MS2_corrected - theoretical)
        let error_da_after = precursor_ms2_after - theoretical_mass;
        let error_ppm_after = if theoretical_mass > 0.0 {
            error_da_after * 1e6 / theoretical_mass
        } else {
            0.0
        };

        // Push the complete measurement
        let adduct_str = match point.adduct_family {
            crate::metrics::AdductFamily::Protonated => Some("[M+H]+"),
            crate::metrics::AdductFamily::Deprotonated => Some("[M-H]-"),
            crate::metrics::AdductFamily::AlkaliAmmonium => Some("[M+NH4]+"),
            crate::metrics::AdductFamily::MetalComplex => Some("[M+Metal]"),
            crate::metrics::AdductFamily::Halide => Some("[M-Hal]"),
            crate::metrics::AdductFamily::Other => None,
        };

        diag.push_measurement(
            error_ppm_ms1,
            delta_ppm_ms2_ms1,
            error_ppm_before,
            error_ppm_after,
            error_da_ms1,
            delta_ms2_ms1_da,
            error_da_before,
            error_da_after,
            precursor_ms1,
            precursor_ms2,
            precursor_ms2_after,
            adduct_str,
            5000, // max samples
        );
    }

    diag.compute_statistics();
    diagnostics_signal.set(Some(diag));
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
fn update_recalibration_diagnostics(
    _metrics: &PrecursorMetrics,
    _model: CalibrationModel,
    _diagnostics_signal: &mut Signal<Option<RecalibrationDiagnostics>>,
) {
}

#[cfg(target_arch = "wasm32")]
fn download_svg(svg_markup: &str, filename: &str) {
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

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
const fn download_svg(_svg_markup: &str, _filename: &str) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_recalibrated_mgf_with_tof() {
        let input = r#"BEGIN IONS
TITLE=test
PEPMASS=500.0000
CHARGE=1
100.0 50
200.0 100
500.01 200
250.0 150
END IONS"#;

        let model = CalibrationModel::TOFDa { lambda: 1.0 };
        let output = generate_recalibrated_mgf(input, model, None);

        eprintln!("=== INPUT ===");
        eprintln!("{}", input);
        eprintln!("\n=== OUTPUT ===");
        eprintln!("{}", output);

        // Delta should be 500.01 - 500.0 = 0.01
        // With lambda=1, fragments should shift by -0.01
        // 100.0 -> 99.99, 200.0 -> 199.99, 500.01 -> 500.0, 250.0 -> 249.99

        assert_ne!(input, output, "Output should differ from input!");
        assert!(
            output.contains("99.99") || output.contains("199.99"),
            "Should contain recalibrated values"
        );
    }

    #[test]
    fn test_generate_recalibrated_mgf_no_model() {
        let input = r#"BEGIN IONS
TITLE=test
PEPMASS=500.0000
CHARGE=1
100.0 50
END IONS"#;

        let model = CalibrationModel::None;
        let output = generate_recalibrated_mgf(input, model, None);

        assert_eq!(
            input, output,
            "With None model, output should be identical to input"
        );
    }
}
