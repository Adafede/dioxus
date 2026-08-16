use dioxus::prelude::*;
use ui::prelude::*;

use crate::app::browser::download_recalibrated_mgf;
use crate::app::diagnostics::update_recalibration_diagnostics;
use crate::app::plots::{
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

fn format_lambda(lambda: f64) -> String {
    format!("{lambda:.2}")
}

/// Renders the post-upload results panel: summary statistics, recalibration
/// controls, the recalibrated-MGF download, diagnostics, and cumulative-error
/// plots. Extracted from `app` so the route component stays focused on
/// input/upload wiring.
#[allow(clippy::too_many_lines)]
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
                    style: StyleBuilder::new().display("grid").property("grid-template-columns", "repeat(auto-fit, minmax(220px, 1fr))").gap("1rem").property("margin-top", "1rem").build(),
                    div { style: StyleBuilder::new().property("background", "linear-gradient(135deg, #ffffff 0%, #f8fafc 100%)").padding("0.9rem").border_radius("14px").border("1px solid #e2e8f0").box_shadow("0 10px 24px rgba(15, 23, 42, 0.06)").build(),
                        h4 { style: StyleBuilder::new().margin("0 0 0.35rem").font_size("0.95rem").color("#0f172a").build(), "Observed precursor 𝑚/𝑧 distribution" }
                        p { style: StyleBuilder::new().margin("0").color("#64748b").font_size("0.8rem").build(), "Range, mean, and median of the observed precursor 𝑚/𝑧 values" }
                        div { style: StyleBuilder::new().display("flex").flex_wrap("wrap").gap("0.45rem").property("margin-top", "0.6rem").build(),
                            span { style: StyleBuilder::new().display("inline-flex").align_items("center").gap("0.3rem").padding("0.35rem 0.6rem").border_radius("999px").property("background", "#eff6ff").color("#1d4ed8").border("1px solid #bfdbfe").font_size("0.78rem").font_weight("700").build(), "median {format_value(metrics.observed_precursor_median)}" }
                            span { style: StyleBuilder::new().display("inline-flex").align_items("center").gap("0.3rem").padding("0.35rem 0.6rem").border_radius("999px").property("background", "#f8fafc").color("#334155").border("1px solid #e2e8f0").font_size("0.78rem").font_weight("700").build(), "mean {format_value(metrics.observed_precursor_mean)}" }
                            span { style: StyleBuilder::new().display("inline-flex").align_items("center").gap("0.3rem").padding("0.35rem 0.6rem").border_radius("999px").property("background", "#f8fafc").color("#334155").border("1px solid #e2e8f0").font_size("0.78rem").font_weight("700").build(), "range {format_value(metrics.observed_precursor_max - metrics.observed_precursor_min)}" }
                        }
                    }
                    div { style: StyleBuilder::new().property("background", "linear-gradient(135deg, #ffffff 0%, #f8fafc 100%)").padding("0.9rem").border_radius("14px").border("1px solid #e2e8f0").box_shadow("0 10px 24px rgba(15, 23, 42, 0.06)").build(),
                        h4 { style: StyleBuilder::new().margin("0 0 0.35rem").font_size("0.95rem").color("#0f172a").build(), "Absolute precursor error (Da)" }
                        p { style: StyleBuilder::new().margin("0").color("#64748b").font_size("0.8rem").build(), "Median absolute deviation, mean absolute deviation, and RMS in daltons" }
                        div { style: StyleBuilder::new().display("flex").flex_wrap("wrap").gap("0.45rem").property("margin-top", "0.6rem").build(),
                            span { style: StyleBuilder::new().display("inline-flex").align_items("center").gap("0.3rem").padding("0.35rem 0.6rem").border_radius("999px").property("background", "#eff6ff").color("#1d4ed8").border("1px solid #bfdbfe").font_size("0.78rem").font_weight("700").build(), "median {format_value(metrics.abs_error_da_median)}" }
                            span { style: StyleBuilder::new().display("inline-flex").align_items("center").gap("0.3rem").padding("0.35rem 0.6rem").border_radius("999px").property("background", "#f8fafc").color("#334155").border("1px solid #e2e8f0").font_size("0.78rem").font_weight("700").build(), "mean {format_value(metrics.abs_error_da_mean)}" }
                            span { style: StyleBuilder::new().display("inline-flex").align_items("center").gap("0.3rem").padding("0.35rem 0.6rem").border_radius("999px").property("background", "#f8fafc").color("#334155").border("1px solid #e2e8f0").font_size("0.78rem").font_weight("700").build(), "RMS {format_value(metrics.abs_error_da_rms)}" }
                        }
                    }
                    div { style: StyleBuilder::new().property("background", "linear-gradient(135deg, #ffffff 0%, #f8fafc 100%)").padding("0.9rem").border_radius("14px").border("1px solid #e2e8f0").box_shadow("0 10px 24px rgba(15, 23, 42, 0.06)").build(),
                        h4 { style: StyleBuilder::new().margin("0 0 0.35rem").font_size("0.95rem").color("#0f172a").build(), "Relative precursor error (ppm)" }
                        p { style: StyleBuilder::new().margin("0").color("#64748b").font_size("0.8rem").build(), "Median relative deviation, mean relative deviation, and RMS of the ppm error" }
                        div { style: StyleBuilder::new().display("flex").flex_wrap("wrap").gap("0.45rem").property("margin-top", "0.6rem").build(),
                            span { style: StyleBuilder::new().display("inline-flex").align_items("center").gap("0.3rem").padding("0.35rem 0.6rem").border_radius("999px").property("background", "#eff6ff").color("#1d4ed8").border("1px solid #bfdbfe").font_size("0.78rem").font_weight("700").build(), "median {format_value(metrics.abs_error_ppm_median)}" }
                            span { style: StyleBuilder::new().display("inline-flex").align_items("center").gap("0.3rem").padding("0.35rem 0.6rem").border_radius("999px").property("background", "#f8fafc").color("#334155").border("1px solid #e2e8f0").font_size("0.78rem").font_weight("700").build(), "mean {format_value(metrics.abs_error_ppm_mean)}" }
                            span { style: StyleBuilder::new().display("inline-flex").align_items("center").gap("0.3rem").padding("0.35rem 0.6rem").border_radius("999px").property("background", "#f8fafc").color("#334155").border("1px solid #e2e8f0").font_size("0.78rem").font_weight("700").build(), "RMS {format_value(metrics.abs_error_ppm_rms)}" }
                        }
                    }
                }

                div {
                    style: StyleBuilder::new().property("margin-top", "1rem").padding("0.95rem 1rem").border("1px solid #e2e8f0").border_radius("16px").property("background", "linear-gradient(180deg, #ffffff 0%, #f8fafc 100%)").box_shadow("0 10px 24px rgba(15, 23, 42, 0.06)").build(),
                    h4 { style: StyleBuilder::new().margin("0 0 0.25rem").font_size("0.95rem").color("#0f172a").build(), "Tolerance-band compliance" }
                    p { style: StyleBuilder::new().margin("0 0 0.7rem").color("#64748b").font_size("0.84rem").build(), "Counts of spectra up to each reported mass-error cutoff (cumulative)" }
                    div { style: StyleBuilder::new().display("grid").property("grid-template-columns", "repeat(auto-fit, minmax(150px, 1fr))").gap("0.6rem").build(),
                        div { style: tolerance_card_style(0),
                            strong { style: StyleBuilder::new().display("block").font_size("0.8rem").property("margin-bottom", "0.25rem").build(), "≤ 0.1 mDa" }
                            span { style: StyleBuilder::new().font_size("0.88rem").font_weight("700").build(), "{format_cumulative_bucket_count(metrics, \"0.1_da\", metrics.spectra)}" }
                        }
                        div { style: tolerance_card_style(1),
                            strong { style: StyleBuilder::new().display("block").font_size("0.8rem").property("margin-bottom", "0.25rem").build(), "≤ 0.5 mDa" }
                            span { style: StyleBuilder::new().font_size("0.88rem").font_weight("700").build(), "{format_cumulative_bucket_count(metrics, \"0.5_da\", metrics.spectra)}" }
                        }
                        div { style: tolerance_card_style(2),
                            strong { style: StyleBuilder::new().display("block").font_size("0.8rem").property("margin-bottom", "0.25rem").build(), "≤ 1.0 mDa" }
                            span { style: StyleBuilder::new().font_size("0.88rem").font_weight("700").build(), "{format_cumulative_bucket_count(metrics, \"1.0_da\", metrics.spectra)}" }
                        }
                        div { style: tolerance_card_style(3),
                            strong { style: StyleBuilder::new().display("block").font_size("0.8rem").property("margin-bottom", "0.25rem").build(), "≤ 5.0 mDa" }
                            span { style: StyleBuilder::new().font_size("0.88rem").font_weight("700").build(), "{format_cumulative_bucket_count(metrics, \"5.0_da\", metrics.spectra)}" }
                        }
                        div { style: tolerance_card_style(4),
                            strong { style: StyleBuilder::new().display("block").font_size("0.8rem").property("margin-bottom", "0.25rem").build(), "> 5.0 mDa" }
                            span { style: StyleBuilder::new().font_size("0.88rem").font_weight("700").build(), "{format_cumulative_bucket_count(metrics, \">5.0_da\", metrics.spectra)}" }
                        }
                        div { style: tolerance_card_style(0),
                            strong { style: StyleBuilder::new().display("block").font_size("0.8rem").property("margin-bottom", "0.25rem").build(), "≤ 0.5 ppm" }
                            span { style: StyleBuilder::new().font_size("0.88rem").font_weight("700").build(), "{format_cumulative_bucket_count(metrics, \"0.5_ppm\", metrics.spectra)}" }
                        }
                        div { style: tolerance_card_style(1),
                            strong { style: StyleBuilder::new().display("block").font_size("0.8rem").property("margin-bottom", "0.25rem").build(), "≤ 1.0 ppm" }
                            span { style: StyleBuilder::new().font_size("0.88rem").font_weight("700").build(), "{format_cumulative_bucket_count(metrics, \"1.0_ppm\", metrics.spectra)}" }
                        }
                        div { style: tolerance_card_style(2),
                            strong { style: StyleBuilder::new().display("block").font_size("0.8rem").property("margin-bottom", "0.25rem").build(), "≤ 5.0 ppm" }
                            span { style: StyleBuilder::new().font_size("0.88rem").font_weight("700").build(), "{format_cumulative_bucket_count(metrics, \"5.0_ppm\", metrics.spectra)}" }
                        }
                        div { style: tolerance_card_style(3),
                            strong { style: StyleBuilder::new().display("block").font_size("0.8rem").property("margin-bottom", "0.25rem").build(), "≤ 10.0 ppm" }
                            span { style: StyleBuilder::new().font_size("0.88rem").font_weight("700").build(), "{format_cumulative_bucket_count(metrics, \"10.0_ppm\", metrics.spectra)}" }
                        }
                        div { style: tolerance_card_style(4),
                            strong { style: StyleBuilder::new().display("block").font_size("0.8rem").property("margin-bottom", "0.25rem").build(), "> 10.0 ppm" }
                            span { style: StyleBuilder::new().font_size("0.88rem").font_weight("700").build(), "{format_cumulative_bucket_count(metrics, \">10.0_ppm\", metrics.spectra)}" }
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

                // Tolerance-band compliance AFTER recalibration (duplicated)
                div {
                    style: StyleBuilder::new().property("margin-top", "1rem").padding("0.95rem 1rem").border("1px solid #e2e8f0").border_radius("16px").property("background", "linear-gradient(180deg, #ffffff 0%, #f8fafc 100%)").box_shadow("0 10px 24px rgba(15, 23, 42, 0.06)").build(),
                    h4 { style: StyleBuilder::new().margin("0 0 0.25rem").font_size("0.95rem").color("#0f172a").build(), "Tolerance-band compliance (Recalibrated)" }
                    p { style: StyleBuilder::new().margin("0 0 0.7rem").color("#64748b").font_size("0.84rem").build(), "Estimated compliance after MS2 fragment recalibration (preview)" }
                    div { style: StyleBuilder::new().display("grid").property("grid-template-columns", "repeat(auto-fit, minmax(150px, 1fr))").gap("0.6rem").build(),
                        if let Some(diag) = recalibration_diagnostics.read().as_ref() {
                            // mDa after
                            div { style: tolerance_card_style(0),
                                strong { style: StyleBuilder::new().display("block").font_size("0.8rem").property("margin-bottom", "0.25rem").build(), "≤ 0.1 mDa" }
                                span { style: StyleBuilder::new().font_size("0.88rem").font_weight("700").build(), "{format_value(estimate_compliance_mda(&diag.error_da_after, 0.1))}" }
                            }
                            div { style: tolerance_card_style(1),
                                strong { style: StyleBuilder::new().display("block").font_size("0.8rem").property("margin-bottom", "0.25rem").build(), "≤ 0.5 mDa" }
                                span { style: StyleBuilder::new().font_size("0.88rem").font_weight("700").build(), "{format_value(estimate_compliance_mda(&diag.error_da_after, 0.5))}" }
                            }
                            div { style: tolerance_card_style(2),
                                strong { style: StyleBuilder::new().display("block").font_size("0.8rem").property("margin-bottom", "0.25rem").build(), "≤ 1.0 mDa" }
                                span { style: StyleBuilder::new().font_size("0.88rem").font_weight("700").build(), "{format_value(estimate_compliance_mda(&diag.error_da_after, 1.0))}" }
                            }
                            div { style: tolerance_card_style(3),
                                strong { style: StyleBuilder::new().display("block").font_size("0.8rem").property("margin-bottom", "0.25rem").build(), "≤ 5.0 mDa" }
                                span { style: StyleBuilder::new().font_size("0.88rem").font_weight("700").build(), "{format_value(estimate_compliance_mda(&diag.error_da_after, 5.0))}" }
                            }
                            div { style: tolerance_card_style(4),
                                strong { style: StyleBuilder::new().display("block").font_size("0.8rem").property("margin-bottom", "0.25rem").build(), "> 5.0 mDa" }
                                span { style: StyleBuilder::new().font_size("0.88rem").font_weight("700").build(), "{format_value(100.0 - estimate_compliance_mda(&diag.error_da_after, 5.0))}" }
                            }
                            // ppm after
                            div { style: tolerance_card_style(0),
                                strong { style: StyleBuilder::new().display("block").font_size("0.8rem").property("margin-bottom", "0.25rem").build(), "≤ 0.5 ppm" }
                                span { style: StyleBuilder::new().font_size("0.88rem").font_weight("700").build(), "{format_value(estimate_compliance_ppm(&diag.error_ppm_after, 0.5))}" }
                            }
                            div { style: tolerance_card_style(1),
                                strong { style: StyleBuilder::new().display("block").font_size("0.8rem").property("margin-bottom", "0.25rem").build(), "≤ 1.0 ppm" }
                                span { style: StyleBuilder::new().font_size("0.88rem").font_weight("700").build(), "{format_value(estimate_compliance_ppm(&diag.error_ppm_after, 1.0))}" }
                            }
                            div { style: tolerance_card_style(2),
                                strong { style: StyleBuilder::new().display("block").font_size("0.8rem").property("margin-bottom", "0.25rem").build(), "≤ 5.0 ppm" }
                                span { style: StyleBuilder::new().font_size("0.88rem").font_weight("700").build(), "{format_value(estimate_compliance_ppm(&diag.error_ppm_after, 5.0))}" }
                            }
                            div { style: tolerance_card_style(3),
                                strong { style: StyleBuilder::new().display("block").font_size("0.8rem").property("margin-bottom", "0.25rem").build(), "≤ 10.0 ppm" }
                                span { style: StyleBuilder::new().font_size("0.88rem").font_weight("700").build(), "{format_value(estimate_compliance_ppm(&diag.error_ppm_after, 10.0))}" }
                            }
                            div { style: tolerance_card_style(4),
                                strong { style: StyleBuilder::new().display("block").font_size("0.8rem").property("margin-bottom", "0.25rem").build(), "> 10.0 ppm" }
                                span { style: StyleBuilder::new().font_size("0.88rem").font_weight("700").build(), "{format_value(100.0 - estimate_compliance_ppm(&diag.error_ppm_after, 10.0))}" }
                            }
                        }
                    }
                }

                // Download recalibrated MGF button
                if !original_mgf_content.read().is_empty() && !matches!(*calibration_model.read(), CalibrationModel::None) {
                    button {
                        r#type: "button",
                        style: StyleBuilder::new().property("margin-top", "1rem").width("100%").padding("0.75rem 1rem").border("2px solid #10b981").border_radius("8px").property("background", "#10b981").color("white").font_size("0.95rem").font_weight("700").cursor("pointer").transition("background 0.2s").property("hover", "background #059669").build(),
                        onclick: move |_| {
                            let file_name = file_name.read();
                            let original = original_mgf_content.read();
                            let model = *calibration_model.read();
                            let diag = recalibration_diagnostics.read().clone();

                            let recalibrated = crate::recalibration::generate_recalibrated_mgf(
                                &original, model, diag.as_ref(),
                            );

                            if let Err(e) =
                                download_recalibrated_mgf(&file_name, &recalibrated)
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
                                        dangerous_inner_html: render_cumulative_error_three_curves(
                                            &diag.error_da_ms1,
                                            &diag.error_da_before,
                                            &diag.error_da_after,
                                            "mDa",
                                            vec![0.1, 0.5, 1.0, 5.0],
                                        ).unwrap_or_default(),
                                    }
                                } else {
                                    div {
                                        dangerous_inner_html: render_cumulative_error_three_curves(
                                            &diag.error_ppm_ms1,
                                            &diag.error_ppm_before,
                                            &diag.error_ppm_after,
                                            "ppm",
                                            vec![0.5, 1.0, 5.0, 10.0],
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
