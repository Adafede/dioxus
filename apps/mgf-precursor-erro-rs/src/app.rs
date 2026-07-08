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

use crate::metrics::{PlotPoint, PrecursorMetrics};
#[cfg(target_arch = "wasm32")]
use crate::parser::{ScanError, scan_blob_with_progress};
use crate::plotting::{
    make_svg_responsive, render_absolute_mass_bias_svg, render_ecdf_svg, render_mass_bias_svg,
};

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
) {
    let mut status_for_progress = status;
    let mut metrics_for_results = metrics;
    let mut busy_for_results = busy;

    spawn(async move {
        let total_bytes = blob.size() as u64;
        status_for_progress.set(format!("Scanning {total_bytes} bytes..."));
        let result = match scan_blob_with_progress(&blob, move |processed, total| {
            status_for_progress.set(format_progress_message(processed, total));
        })
        .await
        {
            Ok(metrics) => metrics,
            Err(error) => {
                status_for_progress.set(format!("Error reading file: {error:?}"));
                PrecursorMetrics::default()
            }
        };
        metrics_for_results.set(Some(result));
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

    start_analysis(blob, status_for_state, metrics_for_state, busy_for_state);
}

#[cfg(target_arch = "wasm32")]
fn load_example_mgf(
    status: Signal<String>,
    metrics: Signal<Option<PrecursorMetrics>>,
    busy: Signal<bool>,
    file_name: Signal<String>,
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
                );
            }
            Err(error) => {
                status_for_progress.set(error);
                busy_for_results.set(false);
            }
        }
    });
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
                        h2 { style: "margin: 0; font-size: 1.7rem; letter-spacing: -0.02em;", "MGF Precursor Error Metrics" }
                        p {
                            style: "margin: 0.2rem 0 0; color: #475569; font-size: 0.95rem;",
                            "Upload an MGF file and summarize precursor mass errors in Da and ppm."
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
                                load_example_mgf(status, metrics, busy, file_name);
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

                            div {
                                style: "margin-top: 1rem; display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 1rem;",
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
