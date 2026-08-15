// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Recalibration diagnostic renderers: before/after error scatter, error
//! histogram, and the m/z comparison plot — plus the summary text stat block.

use super::data::{floor_to_usize, usize_to_f64};
use crate::errors::MgfError;

/// Renders a diagnostic plot comparing precursor errors before and after recalibration.
///
/// Creates a scatter plot with two overlaid series showing precursor errors in ppm,
/// allowing visual assessment of recalibration effectiveness.
///
/// # Errors
///
/// Returns an error if the SVG backend cannot fill, build, or present the drawing area.
pub fn render_recalibration_diagnostic_ppm(
    errors_before: &[f64],
    errors_after: &[f64],
) -> Result<String, MgfError> {
    use plotters::prelude::*;

    let mut buffer = String::new();

    // Compute ranges
    let all_errors: Vec<f64> = errors_before
        .iter()
        .chain(errors_after.iter())
        .copied()
        .filter(|v| v.is_finite())
        .collect();

    if all_errors.is_empty() {
        return Ok(buffer);
    }

    let error_min = all_errors.iter().copied().fold(f64::INFINITY, f64::min);
    let error_max = all_errors.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let error_span = (error_max - error_min).max(1.0);
    let error_min = error_min - error_span * 0.1;
    let error_max = error_max + error_span * 0.1;

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

        // Plot errors before recalibration
        chart
            .draw_series(errors_before.iter().enumerate().map(|(i, &error)| {
                Circle::new(
                    (usize_to_f64(i), error),
                    3,
                    ShapeStyle::from(&RGBAColor(68, 119, 170, 0.4)).filled(),
                )
            }))?
            .label("Before");

        // Plot errors after recalibration
        chart
            .draw_series(errors_after.iter().enumerate().map(|(i, &error)| {
                Circle::new(
                    (usize_to_f64(i), error),
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

/// Renders a histogram comparing error distributions before and after recalibration.
///
/// # Errors
///
/// Returns an error if the SVG backend cannot fill, build, or present the drawing area.
pub fn render_recalibration_diagnostic_histogram(
    errors_before: &[f64],
    errors_after: &[f64],
    bin_count: usize,
) -> Result<String, MgfError> {
    use plotters::prelude::*;

    let mut buffer = String::new();

    // Compute ranges and histograms
    let all_errors: Vec<f64> = errors_before
        .iter()
        .chain(errors_after.iter())
        .copied()
        .filter(|v| v.is_finite())
        .collect();

    if all_errors.is_empty() {
        return Ok(buffer);
    }

    let error_min = all_errors.iter().copied().fold(f64::INFINITY, f64::min);
    let error_max = all_errors.iter().copied().fold(f64::NEG_INFINITY, f64::max);

    // Compute histograms
    let mut hist_before = vec![0usize; bin_count];
    let mut hist_after = vec![0usize; bin_count];

    for &error in errors_before {
        if error.is_finite() {
            let normalized = (error - error_min) / (error_max - error_min + f64::EPSILON);
            let bin = floor_to_usize((normalized * (usize_to_f64(bin_count) - 1.0)).floor());
            if bin < bin_count {
                hist_before[bin] += 1;
            }
        }
    }

    for &error in errors_after {
        if error.is_finite() {
            let normalized = (error - error_min) / (error_max - error_min + f64::EPSILON);
            let bin = floor_to_usize((normalized * (usize_to_f64(bin_count) - 1.0)).floor());
            if bin < bin_count {
                hist_after[bin] += 1;
            }
        }
    }

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

        // Draw histogram bars
        for (i, &count) in hist_before.iter().enumerate() {
            let x = usize_to_f64(i).mul_add(bin_width, error_min);
            chart.draw_series(std::iter::once(Rectangle::new(
                [(x, 0), (bin_width.mul_add(0.9, x), count)],
                ShapeStyle::from(&RGBAColor(68, 119, 170, 0.4)).filled(),
            )))?;
        }

        for (i, &count) in hist_after.iter().enumerate() {
            let x = bin_width.mul_add(0.45, usize_to_f64(i).mul_add(bin_width, error_min));
            chart.draw_series(std::iter::once(Rectangle::new(
                [(x, 0), (bin_width.mul_add(0.45, x), count)],
                ShapeStyle::from(&RGBAColor(34, 136, 51, 0.6)).filled(),
            )))?;
        }

        root.present()?;
    }
    Ok(buffer)
}

/// Renders a plot showing actual precursor m/z values before/after recalibration.
///
/// This plot displays:
/// - X-axis: Precursor m/z (reference/true value from MS1)
/// - Y-axis: Observed precursor m/z (from MS2)
/// - Blue dots: Observed m/z before recalibration
/// - Green dots: Observed m/z after recalibration
/// - Red diagonal line: Perfect calibration (y = x)
///
/// This is more informative than error plots because it shows the actual
/// mass values and how recalibration shifts them toward the diagonal.
///
/// # Errors
///
/// Returns an error if the SVG backend cannot fill, build, or present the drawing area.
pub fn render_recalibration_diagnostic_mz_comparison(
    precursor_ms1_values: &[f64],
    precursor_ms2_before: &[f64],
    precursor_ms2_after: &[f64],
) -> Result<String, MgfError> {
    use plotters::prelude::*;

    let mut buffer = String::new();

    if precursor_ms1_values.is_empty() || precursor_ms2_before.is_empty() {
        return Ok(buffer);
    }

    // Compute range for both axes (should be equal for diagonal line)
    let all_values: Vec<f64> = precursor_ms1_values
        .iter()
        .chain(precursor_ms2_before.iter())
        .chain(precursor_ms2_after.iter())
        .copied()
        .filter(|v| v.is_finite() && *v > 0.0)
        .collect();

    if all_values.is_empty() {
        return Ok(buffer);
    }

    let min_mz = all_values.iter().copied().fold(f64::INFINITY, f64::min);
    let max_mz = all_values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let span = (max_mz - min_mz).max(1.0);
    let min_mz = min_mz - span * 0.05;
    let max_mz = max_mz + span * 0.05;

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

        // Draw perfect calibration line (y = x)
        chart
            .draw_series(std::iter::once(PathElement::new(
                vec![(min_mz, min_mz), (max_mz, max_mz)],
                ShapeStyle::from(&RGBAColor(200, 50, 50, 0.5)),
            )))?
            .label("Perfect calibration");

        // Plot before recalibration (blue)
        chart
            .draw_series(
                precursor_ms1_values
                    .iter()
                    .zip(precursor_ms2_before.iter())
                    .filter(|(m1, m2)| m1.is_finite() && m2.is_finite())
                    .map(|(m1, m2)| {
                        Circle::new(
                            (*m1, *m2),
                            4,
                            ShapeStyle::from(&RGBAColor(68, 119, 170, 0.5)).filled(),
                        )
                    }),
            )?
            .label("Before recalibration");

        // Plot after recalibration (green)
        chart
            .draw_series(
                precursor_ms1_values
                    .iter()
                    .zip(precursor_ms2_after.iter())
                    .filter(|(m1, m2)| m1.is_finite() && m2.is_finite())
                    .map(|(m1, m2)| {
                        Circle::new(
                            (*m1, *m2),
                            3,
                            ShapeStyle::from(&RGBAColor(34, 136, 51, 0.7)).filled(),
                        )
                    }),
            )?
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

/// Renders a diagnostic summary text showing improvement statistics.
///
/// # Errors
///
/// This helper does not currently produce errors; `format!` always succeeds.
/// The `Result` return type is retained for consistency with the other
/// diagnostic SVG renderers, which can fail on the plotters drawing back-end.
pub fn render_recalibration_summary_text(
    mean_before: f64,
    mean_after: f64,
    rms_before: f64,
    rms_after: f64,
    max_before: f64,
    max_after: f64,
) -> Result<String, MgfError> {
    let mean_improvement = (mean_before.abs() - mean_after.abs()).abs();
    let rms_improvement = rms_before - rms_after;
    let max_improvement = max_before - max_after;

    Ok(format!(
        r#"<div style="font-family: monospace; padding: 20px; background-color: #f5f5f5; border-radius: 8px;">
            <h3>Recalibration Summary</h3>
            <table style="border-collapse: collapse; width: 100%;">
                <tr style="border-bottom: 1px solid #ccc;">
                    <th style="text-align: left; padding: 8px;">Metric</th>
                    <th style="text-align: center; padding: 8px;">Before</th>
                    <th style="text-align: center; padding: 8px;">After</th>
                    <th style="text-align: center; padding: 8px;">Improvement</th>
                </tr>
                <tr style="border-bottom: 1px solid #ddd;">
                    <td style="padding: 8px;">Mean Error (ppm)</td>
                    <td style="text-align: center; padding: 8px;">{:.4}</td>
                    <td style="text-align: center; padding: 8px;">{:.4}</td>
                    <td style="text-align: center; padding: 8px; color: {};">{:.4}</td>
                </tr>
                <tr style="border-bottom: 1px solid #ddd;">
                    <td style="padding: 8px;">RMS Error (ppm)</td>
                    <td style="text-align: center; padding: 8px;">{:.4}</td>
                    <td style="text-align: center; padding: 8px;">{:.4}</td>
                    <td style="text-align: center; padding: 8px; color: {};">{:.4}</td>
                </tr>
                <tr>
                    <td style="padding: 8px;">Max Error (ppm)</td>
                    <td style="text-align: center; padding: 8px;">{:.4}</td>
                    <td style="text-align: center; padding: 8px;">{:.4}</td>
                    <td style="text-align: center; padding: 8px; color: {};">{:.4}</td>
                </tr>
            </table>
        </div>"#,
        mean_before,
        mean_after,
        if mean_improvement > 0.0 {
            "green"
        } else {
            "red"
        },
        mean_improvement,
        rms_before,
        rms_after,
        if rms_improvement > 0.0 {
            "green"
        } else {
            "red"
        },
        rms_improvement,
        max_before,
        max_after,
        if max_improvement > 0.0 {
            "green"
        } else {
            "red"
        },
        max_improvement,
    ))
}
