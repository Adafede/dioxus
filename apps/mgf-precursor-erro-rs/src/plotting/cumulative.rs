// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Cumulative-distribution and four-stage error-analysis renderers. These
//! produce the "complete recalibration story" SVGs (quartet + cumulative CDF
//! curves) and share a small set of private drawing helpers.

use std::fmt::Write;

use super::data::{mean_and_std_dev, usize_to_f64};

fn cumulative_points(values: &[f64]) -> Vec<(f64, f64)> {
    let mut sorted: Vec<f64> = values
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    if sorted.is_empty() {
        return Vec::new();
    }

    let total = usize_to_f64(sorted.len());
    sorted
        .iter()
        .enumerate()
        .map(|(index, &value)| (value, usize_to_f64(index.saturating_add(1)) / total * 100.0))
        .collect()
}

fn draw_quartet_stage<DB: plotters::prelude::DrawingBackend>(
    chart: &mut plotters::prelude::ChartContext<
        '_,
        DB,
        plotters::prelude::Cartesian2d<
            plotters::coord::types::RangedCoordf64,
            plotters::coord::types::RangedCoordf64,
        >,
    >,
    x: f64,
    values: &[f64],
    color: plotters::style::RGBColor,
) where
    DB::ErrorType: 'static,
{
    if let Some((mean, std_dev)) = mean_and_std_dev(values) {
        let style = plotters::style::ShapeStyle::from(&color);
        let _ = chart.draw_series(std::iter::once(plotters::prelude::PathElement::new(
            vec![(x, mean - std_dev), (x, mean + std_dev)],
            style,
        )));
        let _ = chart.draw_series(std::iter::once(plotters::prelude::Circle::new(
            (x, mean),
            6,
            style.filled(),
        )));
    }
}

fn draw_cumulative_curve<DB: plotters::prelude::DrawingBackend>(
    chart: &mut plotters::prelude::ChartContext<
        '_,
        DB,
        plotters::prelude::Cartesian2d<
            plotters::coord::types::RangedCoordf64,
            plotters::coord::types::RangedCoordf64,
        >,
    >,
    values: &[f64],
    color: plotters::style::RGBColor,
    stroke_width: u32,
) where
    DB::ErrorType: 'static,
{
    let points = cumulative_points(values);
    if points.is_empty() {
        return;
    }

    let style = plotters::style::ShapeStyle::from(&color).stroke_width(stroke_width);
    let _ = chart.draw_series(
        points
            .iter()
            .copied()
            .zip(
                points
                    .iter()
                    .skip(1)
                    .copied()
                    .chain(std::iter::once(points[0])),
            )
            .map(|((x1, y1), (x2, y2))| {
                plotters::prelude::PathElement::new(vec![(x1, y1), (x2, y2)], style)
            }),
    );
}

fn append_cumulative_legend(svg: &str) -> String {
    let legend_items = [
        ("MS1 Precursor (PEPMASS)", (68u8, 119u8, 170u8)),
        ("MS2 Before Correction", (255u8, 140u8, 0u8)),
        ("MS2 After Correction", (51u8, 153u8, 51u8)),
    ];

    let mut legend_svg = String::new();
    legend_svg.push_str(r#"<g id="legend" font-size="12" font-family="sans-serif">"#);

    for (idx, (label, (r, g, b))) in legend_items.iter().enumerate() {
        let y = 30 + (idx * 20);
        let x = 1050;
        let _ = write!(
            legend_svg,
            r#"<rect x="{x}" y="{}" width="15" height="15" fill="rgb({r},{g},{b})" stroke="none"/>"#,
            y - 10
        );
        let _ = write!(
            legend_svg,
            r#"<text x="{}" y="{y}" fill="black" font-size="12">{label}</text>"#,
            x + 20
        );
    }
    legend_svg.push_str("</g>");

    if let Some(pos) = svg.rfind("</svg>") {
        let (before, after) = svg.split_at(pos);
        return format!("{before}{legend_svg}{after}");
    }

    svg.to_string()
}

/// Four-stage error analysis showing the complete recalibration story.
///
/// Stage 1 (Blue): MS1 error vs theory
/// Stage 2 (Orange): MS2 error before recalibration vs theory
/// Stage 3 (Red): Calibration delta (MS2 - MS1) - the measurement discrepancy
/// Stage 4 (Green): MS2 error after recalibration vs theory
///
/// # Panics
///
/// Panics if the SVG backend cannot fill, build, or present the drawing area.
#[must_use]
pub fn render_error_quartet(
    error_ms1: &[f64],
    delta_ms2_ms1: &[f64],
    error_ms2_before: &[f64],
    error_ms2_after: &[f64],
) -> String {
    use plotters::prelude::*;

    let mut buffer = String::new();

    let all_values: Vec<f64> = error_ms1
        .iter()
        .chain(delta_ms2_ms1.iter())
        .chain(error_ms2_before.iter())
        .chain(error_ms2_after.iter())
        .copied()
        .filter(|v| v.is_finite())
        .collect();

    if all_values.is_empty() {
        return buffer;
    }

    let min_val = all_values.iter().copied().fold(f64::INFINITY, f64::min);
    let max_val = all_values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let margin = (max_val - min_val).abs().max(0.5) * 0.15;

    {
        let root = SVGBackend::with_string(&mut buffer, (1200, 400)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        let mut chart = ChartBuilder::on(&root)
            .caption(
                "Four-Stage Error Analysis: Complete Recalibration Story",
                ("sans-serif", 18),
            )
            .margin(15)
            .x_label_area_size(60)
            .y_label_area_size(70)
            .build_cartesian_2d(-0.5f64..4.5f64, (min_val - margin)..(max_val + margin))
            .unwrap();

        chart
            .configure_mesh()
            .x_labels(4)
            .x_desc("Analysis Stage")
            .y_desc("Error (ppm)")
            .draw()
            .unwrap();

        let red_style = ShapeStyle::from(&RED).stroke_width(1);
        let _ = chart.draw_series(std::iter::once(PathElement::new(
            vec![(-0.5, 0.0), (4.5, 0.0)],
            red_style,
        )));
        draw_quartet_stage(&mut chart, 0.8, error_ms1, RGBColor(68, 119, 170));
        draw_quartet_stage(&mut chart, 1.8, error_ms2_before, RGBColor(255, 119, 0));
        draw_quartet_stage(&mut chart, 2.8, delta_ms2_ms1, RGBColor(204, 51, 51));
        draw_quartet_stage(&mut chart, 3.8, error_ms2_after, RGBColor(51, 153, 51));

        root.present().unwrap();
    }

    buffer
}

/// Cumulative error distribution curves comparing MS1 and recalibrated MS2 errors.
///
/// Shows how many precursors have error magnitude ≤ x ppm, allowing comparison
/// of MS1 baseline with recalibrated MS2 improvement.
///
/// # Panics
///
/// Panics if the SVG backend cannot fill, build, or present the drawing area.
#[must_use]
pub fn render_cumulative_error_curves(error_ppm_ms1: &[f64], error_ppm_after: &[f64]) -> String {
    use plotters::prelude::*;

    let mut buffer = String::new();

    // Filter finite values and sort for CDF
    let mut ms1_errors: Vec<f64> = error_ppm_ms1
        .iter()
        .copied()
        .filter(|v| v.is_finite())
        .collect();
    let mut ms2_errors: Vec<f64> = error_ppm_after
        .iter()
        .copied()
        .filter(|v| v.is_finite())
        .collect();

    if ms1_errors.is_empty() || ms2_errors.is_empty() {
        return buffer;
    }

    ms1_errors.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    ms2_errors.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let min_error = ms1_errors
        .iter()
        .chain(ms2_errors.iter())
        .copied()
        .fold(f64::INFINITY, f64::min);
    let max_error = ms1_errors
        .iter()
        .chain(ms2_errors.iter())
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);

    let margin = (max_error - min_error).abs().max(1.0) * 0.1;

    {
        let root = SVGBackend::with_string(&mut buffer, (1200, 600)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        let mut chart = ChartBuilder::on(&root)
            .caption(
                "Cumulative Error Distribution: MS1 vs Recalibrated MS2",
                ("sans-serif", 18),
            )
            .margin(15)
            .x_label_area_size(60)
            .y_label_area_size(70)
            .build_cartesian_2d((min_error - margin)..(max_error + margin), 0f64..100f64)
            .unwrap();

        chart
            .configure_mesh()
            .x_desc("Absolute Error (ppm)")
            .y_desc("Cumulative Percentage (%)")
            .draw()
            .unwrap();

        draw_cumulative_curve(&mut chart, &ms1_errors, RGBColor(68, 119, 170), 1);
        draw_cumulative_curve(&mut chart, &ms2_errors, RGBColor(51, 153, 51), 1);

        root.present().unwrap();
    }

    buffer
}

/// Render cumulative error distribution with three curves: ms1, `ms2_before`, `ms2_after`
///
/// # Panics
///
/// Panics if the SVG backend cannot fill, build, or present the drawing area.
#[must_use]
pub fn render_cumulative_error_three_curves(
    error_ms1: &[f64],
    error_before: &[f64],
    error_after: &[f64],
    unit: &str,
    _thresholds: Vec<f64>,
) -> String {
    use plotters::prelude::*;

    let mut buffer = String::new();

    // Filter finite values and sort for CDF
    let mut ms1_errors: Vec<f64> = error_ms1
        .iter()
        .copied()
        .filter(|v| v.is_finite())
        .collect();
    let mut before_errors: Vec<f64> = error_before
        .iter()
        .copied()
        .filter(|v| v.is_finite())
        .collect();
    let mut after_errors: Vec<f64> = error_after
        .iter()
        .copied()
        .filter(|v| v.is_finite())
        .collect();

    if ms1_errors.is_empty() || before_errors.is_empty() || after_errors.is_empty() {
        return buffer;
    }

    ms1_errors.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    before_errors.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    after_errors.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let min_error = ms1_errors
        .iter()
        .chain(before_errors.iter())
        .chain(after_errors.iter())
        .copied()
        .fold(f64::INFINITY, f64::min);
    let max_error = ms1_errors
        .iter()
        .chain(before_errors.iter())
        .chain(after_errors.iter())
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);

    let margin = (max_error - min_error).abs().max(1.0) * 0.1;

    {
        let root = SVGBackend::with_string(&mut buffer, (1200, 600)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        let title = format!("Cumulative Error Distribution ({unit})");
        let mut chart = ChartBuilder::on(&root)
            .caption(&title, ("sans-serif", 18))
            .margin(15)
            .x_label_area_size(60)
            .y_label_area_size(70)
            .build_cartesian_2d((min_error - margin)..(max_error + margin), 0f64..100f64)
            .unwrap();

        chart
            .configure_mesh()
            .x_desc(format!("Error ({unit})"))
            .y_desc("Cumulative Percentage (%)")
            .draw()
            .unwrap();

        draw_cumulative_curve(&mut chart, &ms1_errors, RGBColor(68, 119, 170), 1);
        draw_cumulative_curve(&mut chart, &before_errors, RGBColor(255, 140, 0), 1);
        draw_cumulative_curve(&mut chart, &after_errors, RGBColor(51, 153, 51), 1);

        root.present().unwrap();
    }

    append_cumulative_legend(&buffer)
}
