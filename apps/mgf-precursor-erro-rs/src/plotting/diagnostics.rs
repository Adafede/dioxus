// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Recalibration diagnostic SVG renderers: downsampled error scatter, histogram, and `m/z` comparison.

use mgf_analysis_core::usize_to_f64;

use super::data::sample_scatter_points;
#[cfg(test)]
use super::histogram::{fill_histogram, render_recalibration_diagnostic_histogram};
use crate::errors::PlotError;

const MAX_DIAGNOSTIC_POINTS: usize = 1_500;

pub(super) fn combined_finite_bounds(a: &[f64], b: &[f64]) -> Option<(f64, f64)> {
    let mut min_val = f64::INFINITY;
    let mut max_val = f64::NEG_INFINITY;
    for value in a.iter().chain(b.iter()).copied().filter(|v| v.is_finite()) {
        min_val = min_val.min(value);
        max_val = max_val.max(value);
    }
    min_val.is_finite().then_some((min_val, max_val))
}

fn sampled_index_series(values: &[f64]) -> Vec<(f64, f64)> {
    let pts: Vec<(f64, f64)> = values
        .iter()
        .enumerate()
        .filter(|(_, error)| error.is_finite())
        .map(|(i, &error)| (usize_to_f64(i), error))
        .collect();
    sample_scatter_points(pts, MAX_DIAGNOSTIC_POINTS)
}

fn sampled_pair_series(xs: &[f64], ys: &[f64]) -> Vec<(f64, f64)> {
    let pts: Vec<(f64, f64)> = xs
        .iter()
        .zip(ys.iter())
        .filter(|(x, y)| x.is_finite() && y.is_finite())
        .map(|(&x, &y)| (x, y))
        .collect();
    sample_scatter_points(pts, MAX_DIAGNOSTIC_POINTS)
}

/// Renders a downsampled scatter plot comparing precursor errors (ppm) before and after recalibration.
///
/// # Errors
///
/// Returns [`PlotError`] if the SVG backend fails during drawing.
pub fn render_recalibration_diagnostic_ppm(
    errors_before: &[f64],
    errors_after: &[f64],
) -> Result<String, PlotError> {
    use plotters::prelude::*;

    let mut buffer = String::new();
    let Some((error_min, error_max)) = combined_finite_bounds(errors_before, errors_after) else {
        return Ok(buffer);
    };
    let error_span = (error_max - error_min).max(1.0);
    let error_min = error_min - error_span * 0.1;
    let error_max = error_max + error_span * 0.1;

    let before_pts = sampled_index_series(errors_before);
    let after_pts = sampled_index_series(errors_after);

    {
        let root = SVGBackend::with_string(&mut buffer, (1200, 400)).into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption(
                "Precursor Error (ppm) Before and After Recalibration",
                ("sans-serif", 20),
            )
            .margin(15)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d(
                0f64..usize_to_f64(errors_before.len().max(errors_after.len())),
                error_min..error_max,
            )?;

        chart
            .configure_mesh()
            .x_desc("Scan Index")
            .y_desc("Error (ppm)")
            .draw()?;

        chart
            .draw_series(before_pts.into_iter().map(|pt| {
                Circle::new(
                    pt,
                    3,
                    ShapeStyle::from(&RGBAColor(68, 119, 170, 0.4)).filled(),
                )
            }))?
            .label("Before");
        chart
            .draw_series(after_pts.into_iter().map(|pt| {
                Circle::new(
                    pt,
                    2,
                    ShapeStyle::from(&RGBAColor(34, 136, 51, 0.6)).filled(),
                )
            }))?
            .label("After");

        chart
            .configure_series_labels()
            .background_style(WHITE.mix(0.8))
            .draw()?;
        root.present()?;
    }
    Ok(buffer)
}

/// Renders a downsampled scatter plot comparing observed vs reference precursor *m/z*.
///
/// # Errors
///
/// Returns [`PlotError`] if the SVG backend fails during drawing.
pub fn render_recalibration_diagnostic_mz_comparison(
    precursor_ms1_values: &[f64],
    precursor_ms2_before: &[f64],
    precursor_ms2_after: &[f64],
) -> Result<String, PlotError> {
    use plotters::prelude::*;

    let mut buffer = String::new();
    if precursor_ms1_values.is_empty() || precursor_ms2_before.is_empty() {
        return Ok(buffer);
    }

    let mut min_mz = f64::INFINITY;
    let mut max_mz = f64::NEG_INFINITY;
    for value in precursor_ms1_values
        .iter()
        .chain(precursor_ms2_before.iter())
        .chain(precursor_ms2_after.iter())
        .copied()
        .filter(|v| v.is_finite() && *v > 0.0)
    {
        min_mz = min_mz.min(value);
        max_mz = max_mz.max(value);
    }
    if !min_mz.is_finite() {
        return Ok(buffer);
    }
    let span = (max_mz - min_mz).max(1.0);
    let min_mz = min_mz - span * 0.05;
    let max_mz = max_mz + span * 0.05;

    let before_pts = sampled_pair_series(precursor_ms1_values, precursor_ms2_before);
    let after_pts = sampled_pair_series(precursor_ms1_values, precursor_ms2_after);

    {
        let root = SVGBackend::with_string(&mut buffer, (1200, 600)).into_drawing_area();
        root.fill(&WHITE)?;
        let mut chart = ChartBuilder::on(&root)
            .caption(
                "Precursor m/z: Before and After Recalibration",
                ("sans-serif", 20),
            )
            .margin(15)
            .x_label_area_size(45)
            .y_label_area_size(60)
            .right_y_label_area_size(0)
            .build_cartesian_2d(min_mz..max_mz, min_mz..max_mz)?;

        chart
            .configure_mesh()
            .x_desc("Reference Precursor m/z (MS1, Da)")
            .y_desc("Observed Precursor m/z (MS2, Da)")
            .draw()?;
        chart
            .draw_series(std::iter::once(PathElement::new(
                [(min_mz, min_mz), (max_mz, max_mz)],
                ShapeStyle::from(&RGBAColor(200, 50, 50, 0.5)),
            )))?
            .label("Perfect calibration");
        chart
            .draw_series(before_pts.into_iter().map(|pt| {
                Circle::new(
                    pt,
                    4,
                    ShapeStyle::from(&RGBAColor(68, 119, 170, 0.5)).filled(),
                )
            }))?
            .label("Before recalibration");
        chart
            .draw_series(after_pts.into_iter().map(|pt| {
                Circle::new(
                    pt,
                    3,
                    ShapeStyle::from(&RGBAColor(34, 136, 51, 0.7)).filled(),
                )
            }))?
            .label("After recalibration");
        chart
            .configure_series_labels()
            .background_style(WHITE.mix(0.8))
            .border_style(BLACK)
            .draw()?;
        root.present()?;
    }
    Ok(buffer)
}

#[cfg(test)]
#[expect(clippy::unwrap_used)]
mod tests;
