// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Reactive ECDF and scatter-plot Dioxus components plus numeric formatting helpers.

use dioxus::prelude::*;
use mgf_analysis_core::{ErrorBucket, ErrorUnit, PrecursorStats, RecalibrationStats};
use ui::StyleBuilder;

use crate::plotting::{
    make_svg_responsive, render_absolute_mass_bias_svg, render_ecdf_svg, tolerance_step_color,
};

#[cfg(target_arch = "wasm32")]
use super::browser::download_svg;

pub(super) fn format_value(value: f64) -> String {
    if value.is_finite() {
        format!("{value:.4}")
    } else {
        "n/a".to_string()
    }
}

pub(super) fn format_count_with_percentage(count: usize, total: usize) -> String {
    if total == 0 {
        format!("{count} (0.0%)")
    } else {
        let count = u32::try_from(count).unwrap_or(u32::MAX);
        let total = u32::try_from(total).unwrap_or(u32::MAX);
        let pct = (f64::from(count) / f64::from(total)) * 100.0;
        format!("{count} ({pct:.1}%)")
    }
}

pub(super) fn format_cumulative_bucket_count(
    metrics: &PrecursorStats,
    bucket: ErrorBucket,
    total: usize,
) -> String {
    format_count_with_percentage(metrics.cumulative_bucket_count(bucket), total)
}

pub(super) fn tolerance_card_style(index: usize) -> String {
    let color = tolerance_step_color(index, 5);
    StyleBuilder::new()
        .padding("0.6rem 0.7rem")
        .border_radius("12px")
        .border(&format!("1px solid {color}"))
        .property("background", "#f8fafc")
        .color(&color)
        .build()
}

fn format_threshold_compliance(errors: &[f64], threshold: f64, above: bool) -> String {
    let count = errors
        .iter()
        .filter(|e| {
            let abs = e.abs();
            if above {
                abs > threshold
            } else {
                abs <= threshold
            }
        })
        .count();
    format_count_with_percentage(count, errors.len())
}

pub(super) fn format_recalibrated_compliance(
    diag: &RecalibrationStats,
    bucket: ErrorBucket,
) -> String {
    match bucket {
        ErrorBucket::Within0_1Mda => {
            format_threshold_compliance(&diag.error_da_after, 0.1 / 1000.0, false)
        }
        ErrorBucket::Within0_5Mda => {
            format_threshold_compliance(&diag.error_da_after, 0.5 / 1000.0, false)
        }
        ErrorBucket::Within1Mda => {
            format_threshold_compliance(&diag.error_da_after, 1.0 / 1000.0, false)
        }
        ErrorBucket::Within5Mda => {
            format_threshold_compliance(&diag.error_da_after, 5.0 / 1000.0, false)
        }
        ErrorBucket::Above5Mda => {
            format_threshold_compliance(&diag.error_da_after, 5.0 / 1000.0, true)
        }
        ErrorBucket::Within0_5Ppm => format_threshold_compliance(&diag.error_ppm_after, 0.5, false),
        ErrorBucket::Within1Ppm => format_threshold_compliance(&diag.error_ppm_after, 1.0, false),
        ErrorBucket::Within5Ppm => format_threshold_compliance(&diag.error_ppm_after, 5.0, false),
        ErrorBucket::Within10Ppm => format_threshold_compliance(&diag.error_ppm_after, 10.0, false),
        ErrorBucket::Above10Ppm => format_threshold_compliance(&diag.error_ppm_after, 10.0, true),
    }
}

fn plot_shell(title: &'static str, subtitle: &'static str, svg_markup: Memo<String>) -> Element {
    rsx! {
        div {
            style: StyleBuilder::new().padding("0.95rem").border("1px solid #e2e8f0").border_radius("18px").property("background", "linear-gradient(180deg, #ffffff 0%, #f8fafc 100%)").box_shadow("0 12px 24px rgba(15, 23, 42, 0.04)").build(),
            div { style: StyleBuilder::new().display("flex").align_items("center").justify_content("space-between").gap("0.6rem").property("margin-bottom", "0.65rem").build(),
                div { style: StyleBuilder::new().flex("1").build(),
                    h4 { style: StyleBuilder::new().margin("0 0 0.2rem").font_size("0.95rem").color("#0f172a").build(), "{title}" }
                    p { style: StyleBuilder::new().margin("0").color("#64748b").font_size("0.84rem").build(), "{subtitle}" }
                }
                button {
                    r#type: "button",
                    style: StyleBuilder::new().border("1px solid #cbd5e1").border_radius("999px").property("background", "white").color("#334155").font_size("0.76rem").font_weight("700").padding("0.35rem 0.65rem").cursor("pointer").build(),
                    onclick: move |_| {
                        #[cfg(target_arch = "wasm32")]
                        download_svg(&svg_markup.read(), title);
                        #[cfg(not(target_arch = "wasm32"))]
                        drop(svg_markup.read());
                    },
                    "Download"
                }
            }
            div { style: StyleBuilder::new().border_radius("16px").property("overflow", "visible").border("1px solid #e2e8f0").property("background", "#fcfdff").build(),
                dangerous_inner_html: svg_markup.read().clone()
            }
        }
    }
}

#[component]
pub fn EcdfPlot(
    title: &'static str,
    subtitle: &'static str,
    metrics: Signal<Option<PrecursorStats>>,
    diagnostics: Memo<Option<RecalibrationStats>>,
    unit: ErrorUnit,
) -> Element {
    let unit_str = unit.as_str();
    let thresholds = unit.plot_thresholds();
    let svg_markup = use_memo(move || {
        if let Some(diag) = diagnostics.read().as_ref() {
            let values: Vec<f64> = match unit {
                ErrorUnit::Mda => diag
                    .error_da_after
                    .iter()
                    .map(|v| v.abs() * 1000.0)
                    .collect(),
                ErrorUnit::Ppm => diag.error_ppm_after.iter().map(|v| v.abs()).collect(),
            };
            return make_svg_responsive(
                render_ecdf_svg(title, &values, thresholds, unit_str).unwrap_or_default(),
            );
        }
        let metrics_guard = metrics.read();
        let Some(m) = metrics_guard.as_ref() else {
            return String::new();
        };
        match unit {
            ErrorUnit::Mda => {
                let values: Vec<f64> = m
                    .absolute_error_da_values
                    .iter()
                    .map(|v| v * 1000.0)
                    .collect();
                make_svg_responsive(
                    render_ecdf_svg(title, &values, thresholds, unit_str).unwrap_or_default(),
                )
            }
            ErrorUnit::Ppm => make_svg_responsive(
                render_ecdf_svg(title, &m.absolute_error_ppm_values, thresholds, unit_str)
                    .unwrap_or_default(),
            ),
        }
    });
    plot_shell(title, subtitle, svg_markup)
}

#[component]
pub fn AbsoluteMassBiasPlot(
    title: &'static str,
    subtitle: &'static str,
    metrics: Signal<Option<PrecursorStats>>,
    unit: ErrorUnit,
) -> Element {
    let unit_str = unit.as_str();
    let ticks = unit.plot_thresholds();
    let svg_markup = use_memo(move || {
        let metrics_guard = metrics.read();
        let Some(m) = metrics_guard.as_ref() else {
            return String::new();
        };
        make_svg_responsive(
            render_absolute_mass_bias_svg(title, &m.plot_points, unit_str, ticks)
                .unwrap_or_default(),
        )
    });
    plot_shell(title, subtitle, svg_markup)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formatting_and_compliance_helpers_match_expected_labels() {
        assert_eq!(format_value(1.23456), "1.2346");
        assert_eq!(format_value(f64::NAN), "n/a");
        assert_eq!(format_count_with_percentage(0, 0), "0 (0.0%)");
        assert_eq!(format_count_with_percentage(1, 4), "1 (25.0%)");

        let mut stats = PrecursorStats::default();
        stats.within_0_1_mda = 2;
        stats.within_0_5_mda = 3;
        stats.within_1_mda = 1;
        stats.within_5_mda = 2;
        stats.above_5_mda = 2;
        stats.within_0_5_ppm = 1;
        stats.within_1_ppm = 2;
        stats.within_5_ppm = 3;
        stats.within_10_ppm = 2;
        stats.above_10_ppm = 2;
        assert_eq!(
            format_cumulative_bucket_count(&stats, ErrorBucket::Within0_5Mda, 10),
            "5 (50.0%)"
        );
        assert_eq!(
            format_cumulative_bucket_count(&stats, ErrorBucket::Within10Ppm, 10),
            "8 (80.0%)"
        );
        assert_eq!(
            format_cumulative_bucket_count(&stats, ErrorBucket::Above10Ppm, 10),
            "2 (20.0%)"
        );

        let mut diag = RecalibrationStats::default();
        diag.error_da_after = vec![0.0005, 0.002];
        diag.error_ppm_after = vec![4.0, 12.0];
        assert_eq!(
            format_recalibrated_compliance(&diag, ErrorBucket::Within1Mda),
            "1 (50.0%)"
        );
        assert_eq!(
            format_recalibrated_compliance(&diag, ErrorBucket::Above10Ppm),
            "1 (50.0%)"
        );
        assert!(tolerance_card_style(0).contains("border: 1px solid #"));
    }
}
