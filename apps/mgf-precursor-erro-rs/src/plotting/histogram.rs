// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Histogram renderer for pre/post recalibration precursor error distributions.

use mgf_analysis_core::{floor_to_usize, usize_to_f64};

use super::diagnostics::combined_finite_bounds;
use crate::errors::PlotError;

pub(super) fn fill_histogram(
    bins: &mut [usize],
    values: &[f64],
    min: f64,
    max: f64,
    bin_count: usize,
) {
    if bin_count == 0 {
        return;
    }
    let span = max - min;
    for &error in values {
        if !error.is_finite() {
            continue;
        }
        let bin = if span <= 0.0 {
            0
        } else {
            let normalized = ((error - min) / span).clamp(0.0, 1.0);
            floor_to_usize((normalized * usize_to_f64(bin_count)).floor()).min(bin_count - 1)
        };
        if let Some(count) = bins.get_mut(bin) {
            *count += 1;
        }
    }
}

/// Renders a histogram comparing error distributions before and after recalibration.
///
/// # Errors
///
/// Returns [`PlotError`] if the SVG backend fails during drawing.
pub fn render_recalibration_diagnostic_histogram(
    errors_before: &[f64],
    errors_after: &[f64],
    bin_count: usize,
) -> Result<String, PlotError> {
    use plotters::prelude::*;

    let mut buffer = String::new();
    if bin_count == 0 {
        return Ok(buffer);
    }
    let Some((raw_min, raw_max)) = combined_finite_bounds(errors_before, errors_after) else {
        return Ok(buffer);
    };
    let (error_min, error_max) = if (raw_max - raw_min).abs() <= f64::EPSILON {
        (raw_min - 0.5, raw_max + 0.5)
    } else {
        (raw_min, raw_max)
    };

    let mut hist_before = vec![0usize; bin_count];
    let mut hist_after = vec![0usize; bin_count];
    fill_histogram(
        &mut hist_before,
        errors_before,
        error_min,
        error_max,
        bin_count,
    );
    fill_histogram(
        &mut hist_after,
        errors_after,
        error_min,
        error_max,
        bin_count,
    );

    let max_count = hist_before
        .iter()
        .chain(hist_after.iter())
        .copied()
        .max()
        .unwrap_or(1)
        .max(1);
    let bin_width = (error_max - error_min) / usize_to_f64(bin_count);

    {
        let root = SVGBackend::with_string(&mut buffer, (1200, 400)).into_drawing_area();
        root.fill(&WHITE)?;
        let mut chart = ChartBuilder::on(&root)
            .caption(
                "Error Distribution: Before vs After Recalibration",
                ("sans-serif", 20),
            )
            .margin(15)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d(error_min..error_max, 0usize..max_count)?;

        chart
            .configure_mesh()
            .x_desc("Error (ppm)")
            .y_desc("Count")
            .draw()?;

        let before_style = ShapeStyle::from(&RGBAColor(68, 119, 170, 0.4)).filled();
        chart.draw_series(hist_before.iter().enumerate().map(|(i, &count)| {
            let x = usize_to_f64(i).mul_add(bin_width, error_min);
            Rectangle::new([(x, 0), (bin_width.mul_add(0.9, x), count)], before_style)
        }))?;

        let after_style = ShapeStyle::from(&RGBAColor(34, 136, 51, 0.6)).filled();
        chart.draw_series(hist_after.iter().enumerate().map(|(i, &count)| {
            let x = bin_width.mul_add(0.45, usize_to_f64(i).mul_add(bin_width, error_min));
            Rectangle::new([(x, 0), (bin_width.mul_add(0.45, x), count)], after_style)
        }))?;
        root.present()?;
    }
    Ok(buffer)
}
