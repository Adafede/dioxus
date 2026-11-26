// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Recalibration model selector, shrinkage slider, compliance preview, summary table, and MGF download.

use dioxus::prelude::*;
use mgf_analysis_core::{
    CalibrationModel, ErrorBucket, RecalibrationStats, ShrinkageLambda, generate_recalibrated_mgf,
};
use ui::StyleBuilder;

use super::browser::download_recalibrated_mgf;
use super::plots::{format_recalibrated_compliance, format_value, tolerance_card_style};

pub(super) fn improvement_color(improvement: f64) -> &'static str {
    if !improvement.is_finite() || improvement < 0.0 {
        "red"
    } else if improvement > 0.0 {
        "green"
    } else {
        "#475569"
    }
}

#[component]
pub(super) fn RecalibrationSummaryTable(diag: RecalibrationStats) -> Element {
    let mean_imp = diag.mean_error_ppm_before.abs() - diag.mean_error_ppm_after.abs();
    let mae_imp = diag.mean_error_improvement_ppm();
    let rms_imp = diag.rms_error_ppm_before - diag.rms_error_ppm_after;
    let max_imp = diag.max_abs_error_ppm_before - diag.max_abs_error_ppm_after;

    let rows = [
        (
            "Mean Signed Bias (ppm)",
            diag.mean_error_ppm_before,
            diag.mean_error_ppm_after,
            mean_imp,
        ),
        (
            "MAE (ppm)",
            diag.mae_error_ppm_before,
            diag.mae_error_ppm_after,
            mae_imp,
        ),
        (
            "RMS Error (ppm)",
            diag.rms_error_ppm_before,
            diag.rms_error_ppm_after,
            rms_imp,
        ),
        (
            "Max Error (ppm)",
            diag.max_abs_error_ppm_before,
            diag.max_abs_error_ppm_after,
            max_imp,
        ),
    ];

    rsx! {
        div {
            style: StyleBuilder::new().property("font-family", "monospace").padding("20px").property("background-color", "#f5f5f5").border_radius("8px").property("margin-bottom", "1.5rem").build(),
            h3 { "Recalibration Summary" }
            table {
                style: StyleBuilder::new().property("border-collapse", "collapse").width("100%").build(),
                tr {
                    style: StyleBuilder::new().border_bottom("1px solid #ccc").build(),
                    th { style: StyleBuilder::new().text_align("left").padding("8px").build(), "Metric" }
                    th { style: StyleBuilder::new().text_align("center").padding("8px").build(), "Before" }
                    th { style: StyleBuilder::new().text_align("center").padding("8px").build(), "After" }
                    th { style: StyleBuilder::new().text_align("center").padding("8px").build(), "Improvement" }
                }
                for (label, before, after, imp) in rows {
                    tr {
                        style: StyleBuilder::new().border_bottom("1px solid #ddd").build(),
                        td { style: StyleBuilder::new().padding("8px").build(), "{label}" }
                        td { style: StyleBuilder::new().text_align("center").padding("8px").build(), "{format_value(before)}" }
                        td { style: StyleBuilder::new().text_align("center").padding("8px").build(), "{format_value(after)}" }
                        td {
                            style: StyleBuilder::new().text_align("center").padding("8px").color(improvement_color(imp)).build(),
                            "{format_value(imp)}"
                        }
                    }
                }
            }
        }
    }
}

/// Interactive model/lambda controls, recalibrated tolerance preview, and MGF download action.
#[component]
pub(super) fn RecalibrationPanel(
    calibration_model: Signal<CalibrationModel>,
    preferred_lambda: Signal<ShrinkageLambda>,
    diagnostics: Memo<Option<RecalibrationStats>>,
    original_mgf_content: Signal<String>,
    file_name: Signal<String>,
    status: Signal<String>,
    raw_retention_capped: bool,
) -> Element {
    let active_lambda = calibration_model
        .read()
        .shrinkage()
        .unwrap_or_else(|| *preferred_lambda.read());

    rsx! {
        div {
            style: StyleBuilder::new().property("margin-top", "1.5rem").padding("1.2rem").border("2px solid #3b82f6").border_radius("16px").property("background", "linear-gradient(135deg, #dbeafe 0%, #eff6ff 100%)").build(),
            h2 { style: StyleBuilder::new().margin("0 0 0.8rem").font_size("1.1rem").color("#1e40af").build(), "MS2 Fragment Recalibration" }
            p { style: StyleBuilder::new().margin("0 0 1rem").color("#1e40af").font_size("0.95rem").build(), "Apply precursor-driven recalibration to MS2 fragments using the discrepancy between MS1 and MS2 precursor m/z." }

            div {
                style: StyleBuilder::new().display("grid").property("grid-template-columns", "1fr 1fr").gap("1rem").property("margin-bottom", "1rem").build(),
                div {
                    label { style: StyleBuilder::new().display("block").font_weight("600").color("#1e40af").property("margin-bottom", "0.4rem").build(), "Calibration Model" }
                    select {
                        value: match *calibration_model.read() {
                            CalibrationModel::None => "none",
                            CalibrationModel::TOFDa { .. } => "tof",
                            CalibrationModel::OrbitrapPPM { .. } => "orbitrap",
                        },
                        onchange: move |evt| {
                            let lambda = *preferred_lambda.read();
                            calibration_model.set(match evt.value().as_str() {
                                "tof" => CalibrationModel::TOFDa { lambda },
                                "orbitrap" => CalibrationModel::OrbitrapPPM { lambda },
                                _ => CalibrationModel::None,
                            });
                        },
                        style: StyleBuilder::new().width("100%").padding("0.5rem").border("1px solid #0ea5e9").border_radius("8px").property("background", "white").color("#1e40af").font_weight("500").cursor("pointer").build(),
                        option { value: "none", "None (No Correction)" }
                        option { value: "tof", "TOF (Absolute Da)" }
                        option { value: "orbitrap", "Orbitrap (ppm)" }
                    }
                }

                if calibration_model.read().is_active() {
                    div {
                        label {
                            style: StyleBuilder::new().display("block").font_weight("600").color("#1e40af").property("margin-bottom", "0.4rem").build(),
                            "Lambda (λ): {active_lambda.get():.2}"
                        }
                        input {
                            r#type: "range",
                            min: "0.0",
                            max: "1.0",
                            step: "0.05",
                            value: "{active_lambda.get()}",
                            oninput: move |evt| {
                                let raw = evt.value().parse::<f64>().unwrap_or(0.5);
                                let clamped = ShrinkageLambda::clamped(raw);
                                preferred_lambda.set(clamped);
                                let next = calibration_model.read().with_lambda(clamped);
                                calibration_model.set(next);
                            },
                            style: StyleBuilder::new().width("100%").cursor("pointer").build(),
                        }
                        p { style: StyleBuilder::new().margin("0.4rem 0 0").font_size("0.85rem").color("#0ea5e9").build(), "0 = no correction, 1 = full correction" }
                    }
                }
            }
        }

        if let Some(diag) = diagnostics.read().as_ref() {
            div {
                style: StyleBuilder::new().property("margin-top", "1rem").padding("0.95rem 1rem").border("1px solid #e2e8f0").border_radius("16px").property("background", "linear-gradient(180deg, #ffffff 0%, #f8fafc 100%)").box_shadow("0 10px 24px rgba(15, 23, 42, 0.06)").build(),
                h4 { style: StyleBuilder::new().margin("0 0 0.25rem").font_size("0.95rem").color("#0f172a").build(), "Tolerance-band compliance (Recalibrated)" }
                p { style: StyleBuilder::new().margin("0 0 0.7rem").color("#64748b").font_size("0.84rem").build(), "Estimated compliance after MS2 fragment recalibration (preview over n = {diag.sample_count} sampled spectra)" }
                div { style: StyleBuilder::new().display("grid").property("grid-template-columns", "repeat(auto-fit, minmax(150px, 1fr))").gap("0.6rem").build(),
                    for bucket in ErrorBucket::ALL {
                        div { style: tolerance_card_style(bucket.severity_level()),
                            strong { style: StyleBuilder::new().display("block").font_size("0.8rem").property("margin-bottom", "0.25rem").build(), "{bucket.label()}" }
                            span { style: StyleBuilder::new().font_size("0.88rem").font_weight("700").build(), "{format_recalibrated_compliance(diag, bucket)}" }
                        }
                    }
                }
            }
        }

        if !original_mgf_content.read().is_empty() && calibration_model.read().is_active() {
            button {
                r#type: "button",
                style: StyleBuilder::new().property("margin-top", "1rem").width("100%").padding("0.75rem 1rem").border("2px solid #10b981").border_radius("8px").property("background", "#10b981").color("white").font_size("0.95rem").font_weight("700").cursor("pointer").transition("background 0.2s").build(),
                onclick: move |_| {
                    let name = file_name.read();
                    let original = original_mgf_content.read();
                    let model = *calibration_model.read();
                    let recalibrated = generate_recalibrated_mgf(&original, model);
                    match download_recalibrated_mgf(&name, &recalibrated) {
                        Ok(download_name) => status.set(format!("Downloaded: {download_name}")),
                        Err(err) => status.set(format!("Download error: {err}")),
                    }
                },
                "Download Recalibrated MGF"
            }
        } else if raw_retention_capped && calibration_model.read().is_active() {
            div {
                style: StyleBuilder::new().property("margin-top", "1rem").padding("0.75rem 0.9rem").border("1px solid #fcd34d").border_radius("10px").property("background", "#fffbeb").color("#92400e").font_size("0.88rem").build(),
                "Recalibrated MGF export is disabled for files larger than 64 MiB because raw spectrum lines are not retained above that limit; precursor error diagnostics above still cover the entire file."
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::improvement_color;

    #[test]
    fn improvement_color_distinguishes_positive_neutral_regression_and_nan() {
        assert_eq!(improvement_color(3.0), "green");
        assert_eq!(improvement_color(0.0), "#475569");
        assert_eq!(improvement_color(-3.0), "red");
        assert_eq!(improvement_color(f64::NAN), "red");
    }
}
