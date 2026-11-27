// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Three-curve cumulative error-distribution renderer.

use std::fmt::Write as _;

use mgf_analysis_core::{ErrorUnit, usize_to_f64};

use super::color::{AFTER_RGB, MS1_BEFORE_RGB, MS2_BEFORE_RGB, tolerance_step_rgb};
use super::svg::inject_before_closing_svg;
use crate::errors::PlotError;

fn cumulative_points(values: &[f64]) -> Vec<(f64, f64)> {
    let mut sorted: Vec<f64> = values
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .map(f64::abs)
        .collect();
    sorted.sort_unstable_by(f64::total_cmp);
    if sorted.is_empty() {
        return Vec::new();
    }

    let total = usize_to_f64(sorted.len());
    let mut out = Vec::with_capacity(sorted.len());
    let mut idx = 0usize;
    while let Some(&value) = sorted.get(idx) {
        let mut next = idx + 1;
        while sorted
            .get(next)
            .is_some_and(|&v| v.total_cmp(&value).is_eq())
        {
            next += 1;
        }
        out.push((value, usize_to_f64(next) / total * 100.0));
        idx = next;
    }
    out
}

fn step_series(points: &[(f64, f64)], x_min: f64, x_max: f64) -> Vec<(f64, f64)> {
    if points.is_empty() {
        return Vec::new();
    }
    let mut stepped = Vec::with_capacity(points.len().saturating_mul(2) + 2);
    stepped.push((x_min, 0.0));
    let mut prev_y = 0.0;
    for &(x, y) in points {
        stepped.push((x, prev_y));
        stepped.push((x, y));
        prev_y = y;
    }
    stepped.push((x_max, prev_y));
    stepped
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
    points: Vec<(f64, f64)>,
    color: plotters::style::RGBColor,
    stroke_width: u32,
) -> Result<(), PlotError>
where
    DB::ErrorType: Send + Sync + 'static,
{
    if points.is_empty() {
        return Ok(());
    }
    let style = plotters::style::ShapeStyle::from(&color).stroke_width(stroke_width);
    chart.draw_series(plotters::series::LineSeries::new(points, style))?;
    Ok(())
}

fn append_cumulative_legend(svg: String) -> String {
    inject_before_closing_svg(svg, |out| {
        let legend_items = [
            ("MS1 Precursor (PEPMASS)", MS1_BEFORE_RGB),
            ("MS2 Before Correction", MS2_BEFORE_RGB),
            ("MS2 After Correction", AFTER_RGB),
        ];

        out.push_str(r#"<g id="legend" font-size="12" font-family="sans-serif">"#);
        for (idx, (label, rgb)) in legend_items.iter().enumerate() {
            let y = 30 + (idx * 20);
            let x = 990;
            let _ = write!(
                out,
                r#"<rect x="{x}" y="{}" width="15" height="15" fill="rgb({},{},{})" stroke="none"/>"#,
                y - 10,
                rgb.0,
                rgb.1,
                rgb.2
            );
            let _ = write!(
                out,
                r#"<text x="{}" y="{y}" fill="black" font-size="12">{label}</text>"#,
                x + 20
            );
        }
        out.push_str("</g>");
    })
}

fn render_cumulative_chart(
    title: &str,
    x_desc: &str,
    curves: &[(&[f64], plotters::prelude::RGBColor)],
    thresholds: &[f64],
) -> Result<Option<String>, PlotError> {
    use plotters::prelude::*;

    let mut prepared = Vec::with_capacity(curves.len());
    let mut min_error = f64::INFINITY;
    let mut max_error = f64::NEG_INFINITY;
    for &(values, color) in curves {
        let pts = cumulative_points(values);
        if let (Some(&(lo, _)), Some(&(hi, _))) = (pts.first(), pts.last()) {
            min_error = min_error.min(lo);
            max_error = max_error.max(hi);
            prepared.push((pts, color));
        }
    }
    if prepared.is_empty() {
        return Ok(None);
    }

    let half_span = max_error / 2.0 - min_error / 2.0;
    let margin = (half_span * 2.0).abs().max(1.0) * 0.1;
    let x_min = (min_error - margin).max(0.0);
    let x_max = max_error + margin;
    let mut buffer = String::new();
    {
        let root = SVGBackend::with_string(&mut buffer, (1200, 600)).into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption(title, ("sans-serif", 18))
            .margin(15)
            .x_label_area_size(60)
            .y_label_area_size(70)
            .build_cartesian_2d(x_min..x_max, 0f64..100f64)?;

        chart
            .configure_mesh()
            .x_desc(x_desc)
            .y_desc("Cumulative Percentage (%)")
            .draw()?;

        for (idx, &threshold) in thresholds.iter().enumerate() {
            if threshold >= x_min && threshold <= x_max {
                let color = tolerance_step_rgb(idx, 5);
                chart.draw_series(LineSeries::new(
                    [(threshold, 0.0), (threshold, 100.0)],
                    ShapeStyle::from(&color).stroke_width(1),
                ))?;
            }
        }

        for (pts, color) in prepared {
            draw_cumulative_curve(&mut chart, step_series(&pts, x_min, x_max), color, 1)?;
        }

        root.present()?;
    }

    Ok(Some(buffer))
}

/// Render cumulative error distribution with three curves: ms1, `ms2_before`, `ms2_after`.
///
/// # Errors
///
/// Returns [`PlotError`] if the SVG backend fails during drawing.
pub(crate) fn render_cumulative_error_three_curves(
    error_ms1: &[f64],
    error_before: &[f64],
    error_after: &[f64],
    unit: ErrorUnit,
    thresholds: &[f64],
) -> Result<String, PlotError> {
    let unit_str = unit.as_str();
    Ok(render_cumulative_chart(
        &format!("Cumulative Error Distribution ({unit_str})"),
        &format!("Absolute Error ({unit_str})"),
        &[
            (error_ms1, MS1_BEFORE_RGB),
            (error_before, MS2_BEFORE_RGB),
            (error_after, AFTER_RGB),
        ],
        thresholds,
    )?
    .map_or_else(String::new, append_cumulative_legend))
}

#[cfg(test)]
mod tests;
