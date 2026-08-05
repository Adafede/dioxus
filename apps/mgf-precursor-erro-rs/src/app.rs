use dioxus::events::{DragData, FormData};
use dioxus::html::HasFileData;
use dioxus::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::Blob;

mod browser;
mod diagnostics;
mod plots;

use self::browser::download_recalibrated_mgf;
use self::diagnostics::update_recalibration_diagnostics;
use self::plots::{
    absolute_mass_bias_plot, ecdf_plot, estimate_compliance_mda, estimate_compliance_ppm,
    format_bucket_text as format_cumulative_bucket_count, format_value_text as format_value,
    tolerance_style as tolerance_card_style,
};

use crate::diagnostics::RecalibrationStats;
use crate::metrics::PrecursorStats;
use crate::plotting::{
    render_cumulative_error_three_curves, render_recalibration_diagnostic_histogram,
    render_recalibration_diagnostic_ppm, render_recalibration_summary_text,
};
use crate::recalibration::CalibrationModel;

#[component]
fn skip_link() -> Element {
    rsx! {
        a {
            href: "#main",
            class: "skip-link",
            style: "position:absolute;top:-100%;left:0.5rem;z-index:9999;padding:0.5rem 1rem;background:#dbeafe;color:#0b1f33;font-size:0.875rem;font-weight:600;border-radius:0 0 4px 4px;text-decoration:none;",
            "Skip to main content"
        }
    }
}

fn format_lambda(lambda: f64) -> String {
    format!("{lambda:.2}")
}

/// Renders the MGF precursor-error analysis UI.
///
/// # Errors
///
/// Returns an error if the component tree fails to build or render.
#[allow(clippy::too_many_lines)]
pub fn app() -> Element {
    #[cfg(target_arch = "wasm32")]
    let file_name = use_signal(String::new);
    #[cfg(not(target_arch = "wasm32"))]
    let mut file_name = use_signal(String::new);
    #[cfg(target_arch = "wasm32")]
    let metrics = use_signal(|| None::<PrecursorStats>);
    #[cfg(not(target_arch = "wasm32"))]
    let mut metrics = use_signal(|| None::<PrecursorStats>);
    let mut status = use_signal(|| "Drop an MGF file to begin.".to_string());
    #[cfg(target_arch = "wasm32")]
    let busy = use_signal(|| false);
    #[cfg(not(target_arch = "wasm32"))]
    let mut busy = use_signal(|| false);
    let mut drag_active = use_signal(|| false);
    let original_mgf_content = use_signal(String::new);

    // Recalibration control signals
    let mut calibration_model = use_signal(|| CalibrationModel::None);
    let mut lambda_value = use_signal(|| 0.5);
    let mut recalibration_diagnostics = use_signal(|| None::<RecalibrationStats>);
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
        browser::begin_analysis_from_blob(
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
        browser::begin_analysis_from_blob(
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
            style { ".skip-link:focus {{ top: 0 !important; outline: 3px solid #0b5cab; outline-offset: 2px; }}" }
            skip_link {}

            main { id: "main",
                style: "max-width: 960px; margin: 0 auto;",
                h1 { style: "margin: 0 0 0.35rem; font-size: 1.7rem; letter-spacing: -0.02em;", "MGF Precursor Error" }
                p {
                    style: "margin: 0 0 1.25rem; color: #475569; font-size: 0.95rem;",
                    "Upload an MGF file and explore precursor mass errors in Da and ppm."
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
                            aria_describedby: "mgf-upload-help mgf-upload-status",
                            style: "position: absolute; inset: 0; width: 100%; height: 100%; opacity: 0; cursor: pointer;",
                        }
                    }
                    p { id: "mgf-upload-help", style: "margin: 0.7rem 0 0; color: #475569; font-size: 0.9rem;", "Accepts .mgf files. Use drag and drop or browse." }

                    if !file_name.read().is_empty() {
                        p {
                            style: "margin: 0.35rem 0 0; color: #475569; font-size: 0.9rem;",
                            "Selected file: {file_name}"
                        }
                    }

                    p {
                        id: "mgf-upload-status",
                        role: "status",
                        aria_live: "polite",
                        aria_atomic: "true",
                        style: "margin: 0.7rem 0 0; font-weight: 600; color: #334155;",
                        "{status}"
                    }

                    if file_name.read().is_empty() && metrics.read().is_none() && !(*busy.read()) {
                        button {
                            r#type: "button",
                            style: "margin-top: 0.8rem; border: 1px solid #2563eb; border-radius: 999px; background: #eff6ff; color: #1d4ed8; font-size: 0.84rem; font-weight: 700; padding: 0.45rem 0.8rem; cursor: pointer;",
                            onclick: move |_| {
                                #[cfg(target_arch = "wasm32")]
                                browser::load_example_mgf(status, metrics, busy, file_name, original_mgf_content);
                                #[cfg(not(target_arch = "wasm32"))]
                                {
                                    status.set("This app needs to run in a browser.".to_string());
                                }
                            },
                            "Load example MGF"
                        }
                    }

                    if let Some(metrics) = metrics.read().as_ref() {
                        div {
                            style: "margin-top: 1rem; padding: 1rem; border: 1px solid #e2e8f0; border-radius: 16px; background: linear-gradient(180deg, #ffffff 0%, #f8fafc 100%);",
                            h2 { style: "margin: 0 0 0.4rem; font-size: 1rem;", "Summary" }
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
                                h2 { style: "margin: 0 0 0.8rem; font-size: 1.1rem; color: #1e40af;", "🔬 MS2 Fragment Recalibration" }
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
                                                        CalibrationModel::None => CalibrationModel::None,
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
                                        let file_name = file_name.read();
                                        let content = original_mgf_content.read();
                                        let model = *calibration_model.read();
                                        let diag = recalibration_diagnostics.read().clone();

                                        if let Err(e) =
                                            download_recalibrated_mgf(&file_name, &content, model, diag.as_ref())
                                        {
                                            status.set(format!("Download error: {e}"));
                                        } else {
                                            status.set(format!(
                                                "Downloaded: {}_recalibrated.mgf",
                                                if file_name.ends_with(".mgf") {
                                                    &file_name[..file_name.len() - 4]
                                                } else {
                                                    &file_name
                                                }
                                            ));
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
