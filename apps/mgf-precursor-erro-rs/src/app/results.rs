// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

#![expect(
    clippy::empty_enums,
    reason = "Dioxus component macro generates a Props helper empty enum"
)]

use dioxus::prelude::*;
use ui::prelude::*;

use crate::app::browser::download_recalibrated_mgf;
use crate::app::diagnostics::update_recalibration_diagnostics;
use crate::app::plots::{
    absolute_mass_bias_plot, ecdf_plot, format_cumulative_bucket_count,
    format_recalibrated_compliance_mda, format_recalibrated_compliance_ppm, format_value,
    tolerance_card_style,
};
use crate::diagnostics::RecalibrationStats;
use crate::metrics::PrecursorStats;
use crate::plotting::{
    render_cumulative_error_three_curves, render_recalibration_diagnostic_histogram,
    render_recalibration_diagnostic_ppm, render_recalibration_summary_text,
};
use crate::recalibration::CalibrationModel;

fn format_lambda(lambda: f64) -> String {
    format!("{lambda:.2}")
}

/// Renders the post-upload results panel: summary statistics, recalibration
/// controls, the recalibrated-MGF download, diagnostics, and cumulative-error
/// plots. Extracted from `app` so the route component stays focused on
/// input/upload wiring.
#[allow(clippy::too_many_lines)] // UI composition functions are inherently verbose
#[component]
pub fn ResultsPanel(
    metrics: Signal<Option<PrecursorStats>>,
    calibration_model: Signal<CalibrationModel>,
    lambda_value: Signal<f64>,
    recalibration_diagnostics: Signal<Option<RecalibrationStats>>,
    cumulative_dist_tab: Signal<&'static str>,
    original_mgf_content: Signal<String>,
    file_name: Signal<String>,
    status: Signal<String>,
) -> Element {
    let metrics_signal = metrics;
    use_effect(move || {
        if let Some(m) = metrics.read().as_ref() {
            update_recalibration_diagnostics(
                m,
                *calibration_model.read(),
                &mut recalibration_diagnostics,
            );
        }
    });

    rsx! {
        if let Some(metrics) = metrics.read().as_ref() {
            div {
                style: StyleBuilder::new().property("margin-top", "1rem").padding("1rem").border("1px solid #e2e8f0").border_radius("16px").property("background", "linear-gradient(180deg, #ffffff 0%, #f8fafc 100%)").build(),
                h2 { style: StyleBuilder::new().margin("0 0 0.4rem").font_size("1rem").build(), "Summary" }
                p { style: StyleBuilder::new().margin("0.35rem 0").color("#475569").build(), "Processed {metrics.total_spectra} spectra; compared {metrics.spectra} with usable reference masses." }
                p { style: StyleBuilder::new().margin("0.35rem 0").color("#475569").build(), "{metrics.spectra_with_reference_mass} spectra had a usable reference mass." }

                if metrics.skipped_spectra > 0 || !metrics.unrecognized_adducts.is_empty() {
                    div {
                        style: StyleBuilder::new().property("margin-top", "0.9rem").padding("0.8rem 0.9rem").border("1px solid #fcd34d").border_radius("12px").property("background", "#fffbeb").color("#92400e").build(),
                        p { style: StyleBuilder::new().margin("0 0 0.35rem").font_weight("700").build(), "Warnings" }
                        p { style: StyleBuilder::new().margin("0").font_size("0.9rem").build(), "{metrics.skipped_spectra} spectra were skipped because the adduct or reference mass could not be resolved." }
                        if metrics.unparsed_smiles > 0 {
                            p { style: StyleBuilder::new().margin("0.45rem 0 0").font_size("0.88rem").build(), "{metrics.unparsed_smiles} spectra had SMILES that could not be parsed into a reference mass." }
                        }
                        if !metrics.unparsed_smiles_warnings.is_empty() {
                            div { style: StyleBuilder::new().property("margin-top", "0.6rem").padding("0.7rem 0.8rem").border("1px solid #fde68a").border_radius("10px").property("background", "#fffbeb").color("#92400e").build(),
                                p { style: StyleBuilder::new().margin("0 0 0.35rem").font_weight("700").font_size("0.86rem").build(), "Excluded unparsed SMILES" }
                                ul { style: StyleBuilder::new().margin("0.25rem 0 0 1.05rem").padding("0").font_size("0.84rem").property("max-height", "160px").property("overflow", "auto").build(),
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
                            ul { style: StyleBuilder::new().margin("0.45rem 0 0 1.1rem").padding("0").font_size("0.88rem").build(),
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
                        style: StyleBuilder::new().property("margin-top", "1rem").padding("0.8rem 0.9rem").border("1px solid #fecaca").border_radius("12px").property("background", "#fef2f2").color("#991b1b").build(),
                        p { style: StyleBuilder::new().margin("0 0 0.35rem").font_weight("700").build(), "SMILES for spectra above 0.01 Da" }
                        ul { style: StyleBuilder::new().margin("0.25rem 0 0 1.1rem").padding("0").font_size("0.88rem").property("max-height", "240px").property("overflow", "auto").build(),
                            {
                                let mut sorted_high_error = metrics.high_error_smiles.iter().collect::<Vec<_>>();
                                sorted_high_error.sort_by(|(left_smiles, left_detail), (right_smiles, right_detail)| {
                                    right_detail.max_abs_error_da.unwrap_or_default().total_cmp(&left_detail.max_abs_error_da.unwrap_or_default()).then_with(|| right_detail.count.cmp(&left_detail.count)).then_with(|| left_smiles.cmp(right_smiles))
                                });
                                rsx! {
                                    for (smiles, detail) in sorted_high_error {
                                        {
                                            let suffix = if detail.count > 1 { format!(" (x{})", detail.count) } else { String::new() };
                                            let fmt_opt = |v: Option<f64>| v.map_or_else(|| "n/a".to_string(), format_value);
                                            let calc_value = fmt_opt(detail.calculated_mass);
                                            let expected_value = fmt_opt(detail.expected_mass);
                                            let observed_value = fmt_opt(detail.observed_precursor_mz);
                                            let max_error_da = fmt_opt(detail.max_abs_error_da);
                                            let max_error_ppm = fmt_opt(detail.max_abs_error_ppm);
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
                    style: StyleBuilder::new().display("grid").property("grid-template-columns", "repeat(auto-fit, minmax(220px, 1fr))").gap("1rem").property("margin-top", "1rem").build(),
                    for (heading, subtitle, median, mean, third_label, third_val) in [
                        (
                            "Observed precursor 𝑚/𝑧 distribution",
                            "Range, mean, and median of the observed precursor 𝑚/𝑧 values",
                            metrics.observed_precursor_median,
                            metrics.observed_precursor_mean,
                            "range",
                            metrics.observed_precursor_max - metrics.observed_precursor_min,
                        ),
                        (
                            "Absolute precursor error (Da)",
                            "Median absolute deviation, mean absolute deviation, and RMS in daltons",
                            metrics.abs_error_da_median,
                            metrics.abs_error_da_mean,
                            "RMS",
                            metrics.abs_error_da_rms,
                        ),
                        (
                            "Relative precursor error (ppm)",
                            "Median relative deviation, mean relative deviation, and RMS of the ppm error",
                            metrics.abs_error_ppm_median,
                            metrics.abs_error_ppm_mean,
                            "RMS",
                            metrics.abs_error_ppm_rms,
                        ),
                    ] {
                        div { style: StyleBuilder::new().property("background", "linear-gradient(135deg, #ffffff 0%, #f8fafc 100%)").padding("0.9rem").border_radius("14px").border("1px solid #e2e8f0").box_shadow("0 10px 24px rgba(15, 23, 42, 0.06)").build(),
                            h4 { style: StyleBuilder::new().margin("0 0 0.35rem").font_size("0.95rem").color("#0f172a").build(), "{heading}" }
                            p { style: StyleBuilder::new().margin("0").color("#64748b").font_size("0.8rem").build(), "{subtitle}" }
                            div { style: StyleBuilder::new().display("flex").flex_wrap("wrap").gap("0.45rem").property("margin-top", "0.6rem").build(),
                                span { style: StyleBuilder::new().display("inline-flex").align_items("center").gap("0.3rem").padding("0.35rem 0.6rem").border_radius("999px").property("background", "#eff6ff").color("#1d4ed8").border("1px solid #bfdbfe").font_size("0.78rem").font_weight("700").build(), "median {format_value(median)}" }
                                span { style: StyleBuilder::new().display("inline-flex").align_items("center").gap("0.3rem").padding("0.35rem 0.6rem").border_radius("999px").property("background", "#f8fafc").color("#334155").border("1px solid #e2e8f0").font_size("0.78rem").font_weight("700").build(), "mean {format_value(mean)}" }
                                span { style: StyleBuilder::new().display("inline-flex").align_items("center").gap("0.3rem").padding("0.35rem 0.6rem").border_radius("999px").property("background", "#f8fafc").color("#334155").border("1px solid #e2e8f0").font_size("0.78rem").font_weight("700").build(), "{third_label} {format_value(third_val)}" }
                            }
                        }
                    }
                }

                div {
                    style: StyleBuilder::new().property("margin-top", "1rem").padding("0.95rem 1rem").border("1px solid #e2e8f0").border_radius("16px").property("background", "linear-gradient(180deg, #ffffff 0%, #f8fafc 100%)").box_shadow("0 10px 24px rgba(15, 23, 42, 0.06)").build(),
                    h4 { style: StyleBuilder::new().margin("0 0 0.25rem").font_size("0.95rem").color("#0f172a").build(), "Tolerance-band compliance" }
                    p { style: StyleBuilder::new().margin("0 0 0.7rem").color("#64748b").font_size("0.84rem").build(), "Counts of spectra up to each reported mass-error cutoff (cumulative)" }
                    div { style: StyleBuilder::new().display("grid").property("grid-template-columns", "repeat(auto-fit, minmax(150px, 1fr))").gap("0.6rem").build(),
                        for (level, label, bucket) in [
                            (0usize, "≤ 0.1 mDa", "0.1_da"),
                            (1, "≤ 0.5 mDa", "0.5_da"),
                            (2, "≤ 1.0 mDa", "1.0_da"),
                            (3, "≤ 5.0 mDa", "5.0_da"),
                            (4, "> 5.0 mDa", ">5.0_da"),
                            (0, "≤ 0.5 ppm", "0.5_ppm"),
                            (1, "≤ 1.0 ppm", "1.0_ppm"),
                            (2, "≤ 5.0 ppm", "5.0_ppm"),
                            (3, "≤ 10.0 ppm", "10.0_ppm"),
                            (4, "> 10.0 ppm", ">10.0_ppm"),
                        ] {
                            div { style: tolerance_card_style(level),
                                strong { style: StyleBuilder::new().display("block").font_size("0.8rem").property("margin-bottom", "0.25rem").build(), "{label}" }
                                span { style: StyleBuilder::new().font_size("0.88rem").font_weight("700").build(), "{format_cumulative_bucket_count(metrics, bucket, metrics.spectra)}" }
                            }
                        }
                    }
                }

                // Recalibration control panel
                div {
                    style: StyleBuilder::new().property("margin-top", "1.5rem").padding("1.2rem").border("2px solid #3b82f6").border_radius("16px").property("background", "linear-gradient(135deg, #dbeafe 0%, #eff6ff 100%)").build(),
                    h2 { style: StyleBuilder::new().margin("0 0 0.8rem").font_size("1.1rem").color("#1e40af").build(), "🔬 MS2 Fragment Recalibration" }
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
                                    let value = evt.value();
                                    let lambda = *lambda_value.read();
                                    calibration_model.set(match value.as_str() {
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

                        if !matches!(*calibration_model.read(), CalibrationModel::None) {
                            div {
                                label {
                                    style: StyleBuilder::new().display("block").font_weight("600").color("#1e40af").property("margin-bottom", "0.4rem").build(),
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
                                    style: StyleBuilder::new().width("100%").cursor("pointer").build(),
                                }
                                p { style: StyleBuilder::new().margin("0.4rem 0 0").font_size("0.85rem").color("#0ea5e9").build(), "0 = no correction, 1 = full correction" }
                            }
                        }
                    }
                }

                // Tolerance-band compliance AFTER recalibration
                if let Some(diag) = recalibration_diagnostics.read().as_ref() {
                    div {
                        style: StyleBuilder::new().property("margin-top", "1rem").padding("0.95rem 1rem").border("1px solid #e2e8f0").border_radius("16px").property("background", "linear-gradient(180deg, #ffffff 0%, #f8fafc 100%)").box_shadow("0 10px 24px rgba(15, 23, 42, 0.06)").build(),
                        h4 { style: StyleBuilder::new().margin("0 0 0.25rem").font_size("0.95rem").color("#0f172a").build(), "Tolerance-band compliance (Recalibrated)" }
                        p { style: StyleBuilder::new().margin("0 0 0.7rem").color("#64748b").font_size("0.84rem").build(), "Estimated compliance after MS2 fragment recalibration (preview)" }
                        div { style: StyleBuilder::new().display("grid").property("grid-template-columns", "repeat(auto-fit, minmax(150px, 1fr))").gap("0.6rem").build(),
                            for (level, label, value) in [
                                // mDa after
                                (0usize, "≤ 0.1 mDa", format_recalibrated_compliance_mda(&diag.error_da_after, 0.1, false)),
                                (1, "≤ 0.5 mDa", format_recalibrated_compliance_mda(&diag.error_da_after, 0.5, false)),
                                (2, "≤ 1.0 mDa", format_recalibrated_compliance_mda(&diag.error_da_after, 1.0, false)),
                                (3, "≤ 5.0 mDa", format_recalibrated_compliance_mda(&diag.error_da_after, 5.0, false)),
                                (4, "> 5.0 mDa", format_recalibrated_compliance_mda(&diag.error_da_after, 5.0, true)),
                                // ppm after
                                (0, "≤ 0.5 ppm", format_recalibrated_compliance_ppm(&diag.error_ppm_after, 0.5, false)),
                                (1, "≤ 1.0 ppm", format_recalibrated_compliance_ppm(&diag.error_ppm_after, 1.0, false)),
                                (2, "≤ 5.0 ppm", format_recalibrated_compliance_ppm(&diag.error_ppm_after, 5.0, false)),
                                (3, "≤ 10.0 ppm", format_recalibrated_compliance_ppm(&diag.error_ppm_after, 10.0, false)),
                                (4, "> 10.0 ppm", format_recalibrated_compliance_ppm(&diag.error_ppm_after, 10.0, true)),
                            ] {
                                div { style: tolerance_card_style(level),
                                    strong { style: StyleBuilder::new().display("block").font_size("0.8rem").property("margin-bottom", "0.25rem").build(), "{label}" }
                                    span { style: StyleBuilder::new().font_size("0.88rem").font_weight("700").build(), "{value}" }
                                }
                            }
                        }
                    }
                }

                // Download recalibrated MGF button
                if !original_mgf_content.read().is_empty() && !matches!(*calibration_model.read(), CalibrationModel::None) {
                    button {
                        r#type: "button",
                        style: StyleBuilder::new().property("margin-top", "1rem").width("100%").padding("0.75rem 1rem").border("2px solid #10b981").border_radius("8px").property("background", "#10b981").color("white").font_size("0.95rem").font_weight("700").cursor("pointer").transition("background 0.2s").build(),
                        onclick: move |_| {
                            let file_name = file_name.read();
                            let original = original_mgf_content.read();
                            let model = *calibration_model.read();
                            let diag = recalibration_diagnostics.read();

                            let recalibrated = crate::recalibration::generate_recalibrated_mgf(
                                &original, model, diag.as_ref(),
                            );

                            match download_recalibrated_mgf(&file_name, &recalibrated) {
                                Ok(download_name) => {
                                    status.set(format!("Downloaded: {download_name}"));
                                }
                                Err(e) => {
                                    status.set(format!("Download error: {e}"));
                                }
                            }
                        },
                        "Download Recalibrated MGF"
                    }
                } else if metrics.raw_retention_capped && !matches!(*calibration_model.read(), CalibrationModel::None) {
                    div {
                        style: StyleBuilder::new().property("margin-top", "1rem").padding("0.75rem 0.9rem").border("1px solid #fcd34d").border_radius("10px").property("background", "#fffbeb").color("#92400e").font_size("0.88rem").build(),
                        "Recalibrated MGF export is disabled for files larger than 64 MiB because raw spectrum lines are not retained above that limit; precursor error diagnostics above still cover the entire file."
                    }
                }

                // Recalibration diagnostics display
                if let Some(diag) = recalibration_diagnostics.read().as_ref() {
                    div {
                        style: StyleBuilder::new().property("margin-top", "1.5rem").padding("1rem").border("1px solid #e2e8f0").border_radius("16px").property("background", "linear-gradient(180deg, #ffffff 0%, #f8fafc 100%)").build(),

                        h4 { style: StyleBuilder::new().margin("0 0 1rem").font_size("1rem").color("#1e40af").build(), "Recalibration Diagnostics" }

                        // Summary statistics table
                        div {
                            style: StyleBuilder::new().property("margin-bottom", "1.5rem").build(),
                            dangerous_inner_html: render_recalibration_summary_text(
                                diag.mean_error_ppm_before,
                                diag.mean_error_ppm_after,
                                diag.rms_error_ppm_before,
                                diag.rms_error_ppm_after,
                                diag.max_abs_error_ppm_before,
                                diag.max_abs_error_ppm_after,
                            ).unwrap_or_default(),
                        }

                        // Tabbed cumulative error distribution (ms1, ms2_before, ms2_after)
                        div {
                            style: StyleBuilder::new().property("margin-bottom", "1.5rem").build(),
                            h5 { style: StyleBuilder::new().margin("0 0 0.5rem").font_size("0.95rem").color("#1e40af").build(), "Cumulative Error Distribution" }

                            // Tab buttons
                            div {
                                style: StyleBuilder::new().display("flex").gap("0.5rem").property("margin-bottom", "0.8rem").border_bottom("1px solid #e2e8f0").property("padding-bottom", "0.5rem").build(),
                                for (tab, label) in [("mda", "mDa (Absolute)"), ("ppm", "ppm (Relative)")] {
                                    button {
                                        style: if *cumulative_dist_tab.read() == tab {
                                            StyleBuilder::new().padding("0.5rem 1rem").border("2px solid #1e40af").property("background", "#1e40af").color("white").border_radius("6px").font_weight("600").cursor("pointer").build()
                                        } else {
                                            StyleBuilder::new().padding("0.5rem 1rem").border("1px solid #e2e8f0").property("background", "white").color("#64748b").border_radius("6px").cursor("pointer").build()
                                        },
                                        onclick: move |_| {
                                            cumulative_dist_tab.set(tab);
                                        },
                                        "{label}"
                                    }
                                }
                            }

                            // Tab content
                            div {
                                style: StyleBuilder::new().property("background", "white").padding("1rem").border("1px solid #e2e8f0").border_radius("12px").property("overflow-x", "auto").build(),
                                p {
                                    style: StyleBuilder::new().margin("0 0 0.8rem").font_size("0.9rem").color("#64748b").build(),
                                    strong { "Legend: " }
                                    "🔵 Blue = MS1 precursor (PEPMASS) vs theoretical | "
                                    "🟠 Orange = MS2 precursor before correction vs theoretical | "
                                    "🟢 Green = MS2 precursor after recalibration vs theoretical"
                                }
                                if *cumulative_dist_tab.read() == "mda" {
                                    div {
                                        dangerous_inner_html: {
                                            let ms1_mda: Vec<f64> = diag.error_da_ms1.iter().map(|v| v * 1000.0).collect();
                                            let before_mda: Vec<f64> = diag.error_da_before.iter().map(|v| v * 1000.0).collect();
                                            let after_mda: Vec<f64> = diag.error_da_after.iter().map(|v| v * 1000.0).collect();
                                            render_cumulative_error_three_curves(
                                                &ms1_mda,
                                                &before_mda,
                                                &after_mda,
                                                "mDa",
                                                &[0.1, 0.5, 1.0, 5.0],
                                            ).unwrap_or_default()
                                        },
                                    }
                                } else {
                                    div {
                                        dangerous_inner_html: render_cumulative_error_three_curves(
                                            &diag.error_ppm_ms1,
                                            &diag.error_ppm_before,
                                            &diag.error_ppm_after,
                                            "ppm",
                                            &[0.5, 1.0, 5.0, 10.0],
                                        ).unwrap_or_default(),
                                    }
                                }
                            }
                        }

                        // Error time series (supporting detail)
                        div {
                            style: StyleBuilder::new().property("margin-bottom", "1.5rem").build(),
                            h5 { style: StyleBuilder::new().margin("0 0 0.5rem").font_size("0.95rem").color("#1e40af").build(), "Precursor error over time" }
                            p { style: StyleBuilder::new().margin("0 0 0.8rem").font_size("0.9rem").color("#64748b").build(),
                                strong { "Legend: " }
                                "🔵 Blue = MS2 before correction | "
                                "🟢 Green = MS2 after correction" }
                            div {
                                style: StyleBuilder::new().property("background", "white").padding("1rem").border("1px solid #e2e8f0").border_radius("12px").property("overflow-x", "auto").build(),
                                dangerous_inner_html: render_recalibration_diagnostic_ppm(
                                    &diag.error_ppm_before,
                                    &diag.error_ppm_after,
                                ).unwrap_or_default(),
                            }
                        }

                        // Histogram (supporting detail)
                        div {
                            h5 { style: StyleBuilder::new().margin("0 0 0.5rem").font_size("0.95rem").color("#1e40af").build(), "Error distribution" }
                            p { style: StyleBuilder::new().margin("0 0 0.8rem").font_size("0.9rem").color("#64748b").build(),
                                strong { "Legend: " }
                                "🔵 Blue = MS2 before correction | "
                                "🟢 Green = MS2 after correction" }
                            div {
                                style: StyleBuilder::new().property("background", "white").padding("1rem").border("1px solid #e2e8f0").border_radius("12px").property("overflow-x", "auto").build(),
                                dangerous_inner_html: render_recalibration_diagnostic_histogram(
                                    &diag.error_ppm_before,
                                    &diag.error_ppm_after,
                                    20,
                                ).unwrap_or_default(),
                            }
                        }
                    }
                }

                div {
                    ecdf_plot {
                        title: "Absolute precursor-error cumulative distribution (mDa)",
                        subtitle: "Cumulative fraction below each tolerance cutoff, shown on a log10 scale",
                        metrics: metrics_signal,
                        recalibration_diagnostics,
                        thresholds: &[0.1, 0.5, 1.0, 5.0],
                        unit: "mDa",
                    }
                    ecdf_plot {
                        title: "Relative precursor-error cumulative distribution (ppm)",
                        subtitle: "Cumulative fraction below each ppm tolerance cutoff, shown on a log10 scale",
                        metrics: metrics_signal,
                        recalibration_diagnostics,
                        thresholds: &[0.5, 1.0, 5.0, 10.0],
                        unit: "ppm",
                    }
                    absolute_mass_bias_plot {
                        title: "Signed error vs. precursor 𝑚/𝑧 (mDa)",
                        subtitle: "Signed error at each precursor 𝑚/𝑧, centered on zero and shown with a symmetric y-axis",
                        metrics: metrics_signal,
                        unit: "mDa",
                        ticks: &[0.1, 0.5, 1.0, 5.0],
                    }
                    absolute_mass_bias_plot {
                        title: "Signed relative error vs. precursor 𝑚/𝑧 (ppm)",
                        subtitle: "Signed relative error at each precursor 𝑚/𝑧, centered on zero and shown with a symmetric y-axis",
                        metrics: metrics_signal,
                        unit: "ppm",
                        ticks: &[0.5, 1.0, 5.0, 10.0],
                    }
                }
            }
        }
    }
}
