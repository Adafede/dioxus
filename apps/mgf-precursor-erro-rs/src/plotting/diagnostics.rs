// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Recalibration diagnostic scatter plot over sampled precursor errors.

use mgf_analysis_core::usize_to_f64;

use super::color::{AFTER_RGB, MS1_BEFORE_RGB, filled_rgba};
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
    let half_span = error_max / 2.0 - error_min / 2.0;
    let error_span = (half_span * 2.0).max(1.0);
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
            .x_desc("Sample Index")
            .y_desc("Error (ppm)")
            .draw()?;

        let before_style = filled_rgba(MS1_BEFORE_RGB, 0.4);
        chart
            .draw_series(
                before_pts
                    .into_iter()
                    .map(|pt| Circle::new(pt, 3, before_style)),
            )?
            .label("Before")
            .legend(move |(x, y)| Circle::new((x + 8, y), 4, before_style));
        let after_style = filled_rgba(AFTER_RGB, 0.6);
        chart
            .draw_series(
                after_pts
                    .into_iter()
                    .map(|pt| Circle::new(pt, 2, after_style)),
            )?
            .label("After")
            .legend(move |(x, y)| Circle::new((x + 8, y), 4, after_style));

        chart
            .configure_series_labels()
            .background_style(WHITE.mix(0.8))
            .draw()?;
        root.present()?;
    }
    Ok(buffer)
}

#[cfg(test)]
mod tests;
