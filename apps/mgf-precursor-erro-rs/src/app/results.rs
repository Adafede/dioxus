// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Top-level post-upload results view composing summary, warnings, recalibration, and charts.

use dioxus::prelude::*;
use mgf_analysis_core::{
    CalibrationModel, ErrorBucket, ErrorUnit, PrecursorStats, ShrinkageLambda,
    compute_recalibration_diagnostics,
};
use ui::StyleBuilder;

use super::diagnostics_panel::DiagnosticsPanel;
use super::plots::{
    AbsoluteMassBiasPlot, EcdfPlot, format_cumulative_bucket_count, format_value,
    tolerance_card_style,
};
use super::recalibration_panel::RecalibrationPanel;
use super::warnings_panel::WarningsPanel;

/// Renders summary metrics, warnings, recalibration controls/diagnostics, and ECDF/scatter plots.
#[component]
pub fn ResultsPanel(
    metrics: Signal<Option<PrecursorStats>>,
    calibration_model: Signal<CalibrationModel>,
    preferred_lambda: Signal<ShrinkageLambda>,
    cumulative_dist_tab: Signal<ErrorUnit>,
    original_mgf_content: Signal<String>,
    file_name: Signal<String>,
    status: Signal<String>,
) -> Element {
    let diagnostics = use_memo(move || {
        let guard = metrics.read();
        let m = guard.as_ref()?;
        compute_recalibration_diagnostics(m, *calibration_model.read())
    });

    let metrics_guard = metrics.read();
    let Some(m) = metrics_guard.as_ref() else {
        return rsx! {};
    };

    let has_data = m.spectra > 0;
    let or_na = |val: f64| if has_data { val } else { f64::NAN };
    let dist_cards = [
        (
            "Observed precursor 𝑚/𝑧 distribution",
            "Range, mean, and median of the observed precursor 𝑚/𝑧 values",
            or_na(m.observed_precursor_median),
            or_na(m.observed_precursor_mean),
            "range",
            or_na(m.observed_precursor_max - m.observed_precursor_min),
        ),
        (
            "Absolute precursor error (Da)",
            "Median absolute error, mean absolute error, and RMS in daltons",
            or_na(m.abs_error_da_median),
            or_na(m.abs_error_da_mean),
            "RMS",
            or_na(m.abs_error_da_rms),
        ),
        (
            "Relative precursor error (ppm)",
            "Median absolute error, mean absolute error, and RMS in ppm",
            or_na(m.abs_error_ppm_median),
            or_na(m.abs_error_ppm_mean),
            "RMS",
            or_na(m.abs_error_ppm_rms),
        ),
    ];

    rsx! {
        div {
            style: StyleBuilder::new().property("margin-top", "1rem").padding("1rem").border("1px solid #e2e8f0").border_radius("16px").property("background", "linear-gradient(180deg, #ffffff 0%, #f8fafc 100%)").build(),
            h2 { style: StyleBuilder::new().margin("0 0 0.4rem").font_size("1rem").build(), "Summary" }
            p { style: StyleBuilder::new().margin("0.35rem 0").color("#475569").build(), "Processed {m.total_spectra} spectra; compared {m.spectra} with usable reference masses." }

            WarningsPanel { metrics }

            div {
                style: StyleBuilder::new().display("grid").property("grid-template-columns", "repeat(auto-fit, minmax(220px, 1fr))").gap("1rem").property("margin-top", "1rem").build(),
                for (heading, subtitle, median, mean, third_label, third_val) in dist_cards {
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
                    for bucket in ErrorBucket::ALL {
                        div { style: tolerance_card_style(bucket.severity_level()),
                            strong { style: StyleBuilder::new().display("block").font_size("0.8rem").property("margin-bottom", "0.25rem").build(), "{bucket.label()}" }
                            span { style: StyleBuilder::new().font_size("0.88rem").font_weight("700").build(), "{format_cumulative_bucket_count(m, bucket, m.spectra)}" }
                        }
                    }
                }
            }

            RecalibrationPanel {
                calibration_model,
                preferred_lambda,
                diagnostics,
                original_mgf_content,
                file_name,
                status,
                raw_retention_capped: m.raw_retention_capped,
            }

            DiagnosticsPanel { diagnostics, cumulative_dist_tab }

            div {
                EcdfPlot {
                    title: "Absolute precursor-error cumulative distribution (mDa)",
                    subtitle: "Cumulative fraction below each tolerance cutoff, shown on a log10 scale",
                    metrics,
                    diagnostics,
                    unit: ErrorUnit::Mda,
                }
                EcdfPlot {
                    title: "Relative precursor-error cumulative distribution (ppm)",
                    subtitle: "Cumulative fraction below each ppm tolerance cutoff, shown on a log10 scale",
                    metrics,
                    diagnostics,
                    unit: ErrorUnit::Ppm,
                }
                AbsoluteMassBiasPlot {
                    title: "Signed error vs. precursor 𝑚/𝑧 (mDa)",
                    subtitle: "Signed error at each precursor 𝑚/𝑧, centered on zero and shown with a symmetric y-axis",
                    metrics,
                    unit: ErrorUnit::Mda,
                }
                AbsoluteMassBiasPlot {
                    title: "Signed relative error vs. precursor 𝑚/𝑧 (ppm)",
                    subtitle: "Signed relative error at each precursor 𝑚/𝑧, centered on zero and shown with a symmetric y-axis",
                    metrics,
                    unit: ErrorUnit::Ppm,
                }
            }
        }
    }
}
