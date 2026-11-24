// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Cumulative-distribution and four-stage error-analysis renderers.

use std::fmt::Write as _;

use mgf_analysis_core::usize_to_f64;

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
    sorted
        .iter()
        .enumerate()
        .map(|(index, &value)| (value, usize_to_f64(index.saturating_add(1)) / total * 100.0))
        .collect()
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

fn append_cumulative_legend(mut svg: String) -> String {
    let Some(pos) = svg.rfind("</svg>") else {
        return svg;
    };
    let tail = svg.split_off(pos);

    let legend_items = [
        ("MS1 Precursor (PEPMASS)", (68u8, 119u8, 170u8)),
        ("MS2 Before Correction", (255u8, 140u8, 0u8)),
        ("MS2 After Correction", (51u8, 153u8, 51u8)),
    ];

    svg.push_str(r#"<g id="legend" font-size="12" font-family="sans-serif">"#);
    for (idx, (label, (r, g, b))) in legend_items.iter().enumerate() {
        let y = 30 + (idx * 20);
        let x = 990;
        let _ = write!(
            svg,
            r#"<rect x="{x}" y="{}" width="15" height="15" fill="rgb({r},{g},{b})" stroke="none"/>"#,
            y - 10
        );
        let _ = write!(
            svg,
            r#"<text x="{}" y="{y}" fill="black" font-size="12">{label}</text>"#,
            x + 20
        );
    }
    svg.push_str("</g>");
    svg.push_str(&tail);
    svg
}

fn render_cumulative_chart(
    title: &str,
    x_desc: &str,
    curves: &[(&[f64], plotters::prelude::RGBColor)],
) -> Result<Option<String>, PlotError> {
    use plotters::prelude::*;

    let mut prepared = Vec::with_capacity(curves.len());
    let mut min_error = f64::INFINITY;
    let mut max_error = f64::NEG_INFINITY;
    for &(values, color) in curves {
        let pts = cumulative_points(values);
        let (Some(&(lo, _)), Some(&(hi, _))) = (pts.first(), pts.last()) else {
            return Ok(None);
        };
        min_error = min_error.min(lo);
        max_error = max_error.max(hi);
        prepared.push((pts, color));
    }

    let margin = (max_error - min_error).abs().max(1.0) * 0.1;
    let mut buffer = String::new();
    {
        let root = SVGBackend::with_string(&mut buffer, (1200, 600)).into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption(title, ("sans-serif", 18))
            .margin(15)
            .x_label_area_size(60)
            .y_label_area_size(70)
            .build_cartesian_2d((min_error - margin)..(max_error + margin), 0f64..100f64)?;

        chart
            .configure_mesh()
            .x_desc(x_desc)
            .y_desc("Cumulative Percentage (%)")
            .draw()?;

        for (pts, color) in prepared {
            draw_cumulative_curve(&mut chart, pts, color, 1)?;
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
    unit: &str,
    _thresholds: &[f64],
) -> Result<String, PlotError> {
    use plotters::style::RGBColor;
    Ok(render_cumulative_chart(
        &format!("Cumulative Error Distribution ({unit})"),
        &format!("Error ({unit})"),
        &[
            (error_ms1, RGBColor(68, 119, 170)),
            (error_before, RGBColor(255, 140, 0)),
            (error_after, RGBColor(51, 153, 51)),
        ],
    )?
    .map_or_else(String::new, append_cumulative_legend))
}

#[cfg(test)]
#[expect(clippy::unwrap_used)]
mod tests;
