// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Memoized diagnostic SVGs and summary table for active recalibration previews.

use dioxus::prelude::*;
use mgf_analysis_core::{ErrorUnit, RecalibrationStats};
use ui::theme::StyleBuilder;

use super::recalibration_panel::RecalibrationSummaryTable;
use crate::plotting::{
    render_cumulative_error_three_curves, render_recalibration_diagnostic_histogram,
    render_recalibration_diagnostic_ppm,
};

/// Renders memoized diagnostic SVGs (cumulative curves, scan-index error, histogram) and summary table.
#[component]
pub(super) fn DiagnosticsPanel(
    diagnostics: Memo<Option<RecalibrationStats>>,
    cumulative_dist_tab: Signal<ErrorUnit>,
) -> Element {
    let cumulative_svg = use_memo(move || {
        let unit = *cumulative_dist_tab.read();
        let guard = diagnostics.read();
        let Some(diag) = guard.as_ref() else {
            return String::new();
        };
        match unit {
            ErrorUnit::Mda => {
                let ms1: Vec<f64> = diag.error_da_ms1.iter().map(|v| v * 1000.0).collect();
                let before: Vec<f64> = diag.error_da_before.iter().map(|v| v * 1000.0).collect();
                let after: Vec<f64> = diag.error_da_after.iter().map(|v| v * 1000.0).collect();
                render_cumulative_error_three_curves(
                    &ms1,
                    &before,
                    &after,
                    unit.as_str(),
                    unit.plot_thresholds(),
                )
                .unwrap_or_default()
            }
            ErrorUnit::Ppm => render_cumulative_error_three_curves(
                &diag.error_ppm_ms1,
                &diag.error_ppm_before,
                &diag.error_ppm_after,
                unit.as_str(),
                unit.plot_thresholds(),
            )
            .unwrap_or_default(),
        }
    });

    let time_series_svg = use_memo(move || {
        let guard = diagnostics.read();
        let Some(diag) = guard.as_ref() else {
            return String::new();
        };
        render_recalibration_diagnostic_ppm(&diag.error_ppm_before, &diag.error_ppm_after)
            .unwrap_or_default()
    });

    let histogram_svg = use_memo(move || {
        let guard = diagnostics.read();
        let Some(diag) = guard.as_ref() else {
            return String::new();
        };
        render_recalibration_diagnostic_histogram(&diag.error_ppm_before, &diag.error_ppm_after, 20)
            .unwrap_or_default()
    });

    let guard = diagnostics.read();
    let Some(diag) = guard.as_ref() else {
        return rsx! {};
    };

    rsx! {
        div {
            style: StyleBuilder::new().property("margin-top", "1.5rem").padding("1rem").border("1px solid #e2e8f0").border_radius("16px").property("background", "linear-gradient(180deg, #ffffff 0%, #f8fafc 100%)").build(),
            h4 { style: StyleBuilder::new().margin("0 0 1rem").font_size("1rem").color("#1e40af").build(), "Recalibration Diagnostics" }

            RecalibrationSummaryTable { diag: diag.clone() }

            div {
                style: StyleBuilder::new().property("margin-bottom", "1.5rem").build(),
                h5 { style: StyleBuilder::new().margin("0 0 0.5rem").font_size("0.95rem").color("#1e40af").build(), "Cumulative Error Distribution" }
                div {
                    style: StyleBuilder::new().display("flex").gap("0.5rem").property("margin-bottom", "0.8rem").border_bottom("1px solid #e2e8f0").property("padding-bottom", "0.5rem").build(),
                    for (tab, label) in [(ErrorUnit::Mda, "mDa (Absolute)"), (ErrorUnit::Ppm, "ppm (Relative)")] {
                        button {
                            style: if *cumulative_dist_tab.read() == tab {
                                StyleBuilder::new().padding("0.5rem 1rem").border("2px solid #1e40af").property("background", "#1e40af").color("white").border_radius("6px").font_weight("600").cursor("pointer").build()
                            } else {
                                StyleBuilder::new().padding("0.5rem 1rem").border("1px solid #e2e8f0").property("background", "white").color("#64748b").border_radius("6px").cursor("pointer").build()
                            },
                            onclick: move |_| cumulative_dist_tab.set(tab),
                            "{label}"
                        }
                    }
                }
                div {
                    style: StyleBuilder::new().property("background", "white").padding("1rem").border("1px solid #e2e8f0").border_radius("12px").property("overflow-x", "auto").build(),
                    p {
                        style: StyleBuilder::new().margin("0 0 0.8rem").font_size("0.9rem").color("#64748b").build(),
                        strong { "Legend: " }
                        "🔵 Blue = MS1 precursor (PEPMASS) vs theoretical | "
                        "🟠 Orange = MS2 precursor before correction vs theoretical | "
                        "🟢 Green = MS2 precursor after recalibration vs theoretical"
                    }
                    div { dangerous_inner_html: cumulative_svg.read().clone() }
                }
            }

            div {
                style: StyleBuilder::new().property("margin-bottom", "1.5rem").build(),
                h5 { style: StyleBuilder::new().margin("0 0 0.5rem").font_size("0.95rem").color("#1e40af").build(), "Precursor error over time" }
                p { style: StyleBuilder::new().margin("0 0 0.8rem").font_size("0.9rem").color("#64748b").build(),
                    strong { "Legend: " }
                    "🔵 Blue = MS2 before correction | 🟢 Green = MS2 after correction"
                }
                div {
                    style: StyleBuilder::new().property("background", "white").padding("1rem").border("1px solid #e2e8f0").border_radius("12px").property("overflow-x", "auto").build(),
                    dangerous_inner_html: time_series_svg.read().clone()
                }
            }

            div {
                h5 { style: StyleBuilder::new().margin("0 0 0.5rem").font_size("0.95rem").color("#1e40af").build(), "Error distribution" }
                p { style: StyleBuilder::new().margin("0 0 0.8rem").font_size("0.9rem").color("#64748b").build(),
                    strong { "Legend: " }
                    "🔵 Blue = MS2 before correction | 🟢 Green = MS2 after correction"
                }
                div {
                    style: StyleBuilder::new().property("background", "white").padding("1rem").border("1px solid #e2e8f0").border_radius("12px").property("overflow-x", "auto").build(),
                    dangerous_inner_html: histogram_svg.read().clone()
                }
            }
        }
    }
}
