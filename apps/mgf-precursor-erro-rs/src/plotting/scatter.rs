// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Scatter/ECDF chart renderers. These compose data helpers (prep + legend +
//! points) with colour helpers and `plotters` to emit SVG strings.

use mgf_analysis_core::{ErrorUnit, PlotPoint};
use plotters::chart::MeshStyle;
use plotters::prelude::*;

use super::color::{
    MESH_AXIS_RGB, MESH_BOLD_RGB, MESH_LIGHT_RGB, adduct_family_shape_style, tolerance_step_color,
    tolerance_step_rgb,
};
use super::data::{
    ScatterPlotData, build_ecdf_points, display_error_value_for_point, fallback_y_limit_for_unit,
    format_threshold_value, prepare_scatter_plot_data,
};
use super::svg::embed_svg_legend;
use crate::errors::PlotError;

fn style_mesh<'a, 'b, X, Y, DB>(
    mut mesh: MeshStyle<'a, 'b, X, Y, DB>,
    x_desc: &'a str,
    y_desc: &'a str,
) -> MeshStyle<'a, 'b, X, Y, DB>
where
    X: Ranged,
    Y: Ranged,
    DB: DrawingBackend,
{
    mesh.axis_style(ShapeStyle::from(&MESH_AXIS_RGB))
        .light_line_style(ShapeStyle::from(&MESH_LIGHT_RGB))
        .bold_line_style(ShapeStyle::from(&MESH_BOLD_RGB))
        .x_desc(x_desc)
        .y_desc(y_desc)
        .x_label_style(("sans-serif", 11).into_font())
        .y_label_style(("sans-serif", 11).into_font());
    mesh
}

/// Renders an ECDF SVG for the observed error values.
///
/// # Errors
///
/// Returns an error if the SVG backend cannot fill or present the drawing area.
pub(crate) fn render_ecdf_svg(
    title: &str,
    values: &[f64],
    thresholds: &[f64],
    unit: ErrorUnit,
) -> Result<String, PlotError> {
    use plotters::series::LineSeries;

    let width = 900u32;
    let height = 520u32;
    let mut buffer = String::new();
    let root = SVGBackend::with_string(&mut buffer, (width, height)).into_drawing_area();
    root.fill(&WHITE)?;

    let unit_str = unit.as_str();
    let legend_items = thresholds
        .iter()
        .enumerate()
        .map(|(index, threshold)| {
            let label = format!("≤ {} {unit_str}", format_threshold_value(*threshold));
            (label, tolerance_step_color(index, 5))
        })
        .collect::<Vec<_>>();

    let (observed_min, observed_max) = values
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .fold((f64::INFINITY, 0.0_f64), |(min, max), v| {
            (min.min(v), max.max(v))
        });
    let x_min = if observed_min.is_finite() {
        observed_min.max(1e-6)
    } else {
        1e-6
    };
    let view_max = thresholds.iter().copied().fold(0.0, f64::max).max(1e-6);
    let plot_max = observed_max.max(view_max);
    let mut x_max = (plot_max * 1.03).max(x_min + 1e-3);
    x_max = x_max.max(fallback_y_limit_for_unit(unit));
    let y_min = 0.0f64;
    let y_max = 1.0f64;

    let x_desc = match unit {
        ErrorUnit::Ppm => format!("Relative error ({unit_str})"),
        ErrorUnit::Mda => format!("Absolute error ({unit_str})"),
    };
    {
        let mut chart = ChartBuilder::on(&root)
            .margin_top(28)
            .margin_right(32)
            .margin_bottom(36)
            .margin_left(48)
            .caption(title, ("sans-serif", 18).into_font())
            .set_label_area_size(LabelAreaPosition::Left, 72)
            .set_label_area_size(LabelAreaPosition::Bottom, 64)
            .build_cartesian_2d((x_min..x_max).log_scale(), y_min..y_max)?;

        style_mesh(chart.configure_mesh(), &x_desc, "cumulative fraction").draw()?;

        chart.draw_series(LineSeries::new(
            [(x_min, y_min), (x_max, y_min)],
            ShapeStyle::from(&RGBColor(203, 213, 225)).stroke_width(1),
        ))?;

        let plot_points = build_ecdf_points(values, x_min, x_max);
        if !plot_points.is_empty() {
            chart.draw_series(LineSeries::new(
                plot_points,
                ShapeStyle::from(&RGBColor(37, 99, 235)).stroke_width(2),
            ))?;
        }

        for (index, &threshold) in thresholds.iter().enumerate() {
            if threshold < x_min || threshold > x_max {
                continue;
            }
            let color = tolerance_step_rgb(index, 5);
            chart.draw_series(LineSeries::new(
                [(threshold, y_min), (threshold, y_max)],
                ShapeStyle::from(&color).stroke_width(1),
            ))?;
        }

        root.present()?;
    }

    drop(root);
    Ok(embed_svg_legend(
        buffer,
        &legend_items,
        "Thresholds",
        900.0,
        520.0,
    ))
}

fn render_scatter_svg(
    title: &str,
    plot_data: ScatterPlotData,
    y_desc: &str,
    ticks: &[f64],
    alpha: f32,
    clamp_y: bool,
) -> Result<String, PlotError> {
    use plotters::series::{LineSeries, PointSeries};

    let width = 900u32;
    let height = 520u32;
    let mut buffer = String::new();
    let root = SVGBackend::with_string(&mut buffer, (width, height)).into_drawing_area();
    root.fill(&WHITE)?;

    let legend_items = plot_data.legend_items;
    let x_min = plot_data.x_min;
    let x_max = plot_data.x_max;
    let y_limit = plot_data.y_limit;
    let y_min = -y_limit;
    let y_max = y_limit;

    {
        let mut chart = ChartBuilder::on(&root)
            .margin_top(28)
            .margin_right(32)
            .margin_bottom(36)
            .margin_left(48)
            .caption(title, ("sans-serif", 18).into_font())
            .set_label_area_size(LabelAreaPosition::Left, 72)
            .set_label_area_size(LabelAreaPosition::Bottom, 64)
            .build_cartesian_2d(x_min..x_max, y_min..y_max)?;

        style_mesh(chart.configure_mesh(), "Observed precursor 𝑚/𝑧", y_desc).draw()?;

        chart.draw_series(LineSeries::new(
            [(x_min, 0.0f64), (x_max, 0.0f64)],
            ShapeStyle::from(&MESH_AXIS_RGB).stroke_width(1),
        ))?;

        for &tick in ticks {
            if tick <= 0.0 || tick > y_limit {
                continue;
            }
            for y in [tick, -tick] {
                chart.draw_series(LineSeries::new(
                    [(x_min, y), (x_max, y)],
                    ShapeStyle::from(&MESH_LIGHT_RGB).stroke_width(1),
                ))?;
            }
        }

        for (family, series_points) in plot_data.series {
            let style = adduct_family_shape_style(family, alpha);
            chart.draw_series(PointSeries::of_element(
                series_points
                    .iter()
                    .map(|&(x, y)| (x, if clamp_y { y.clamp(y_min, y_max) } else { y })),
                1.6,
                style,
                &|coord, size, style| Circle::new(coord, size, style.filled()),
            ))?;
        }

        root.present()?;
    }

    drop(root);
    Ok(embed_svg_legend(
        buffer,
        &legend_items,
        "Adducts",
        900.0,
        520.0,
    ))
}

/// Renders a scatter plot of signed mass-bias errors using the requested display unit.
///
/// # Errors
///
/// Returns an error if the SVG backend cannot fill or present the drawing area.
pub(crate) fn render_absolute_mass_bias_svg(
    title: &str,
    points: &[PlotPoint],
    unit: ErrorUnit,
    ticks: &[f64],
) -> Result<String, PlotError> {
    let plot_data = prepare_scatter_plot_data(
        points,
        |point| {
            point
                .ms2_precursor_peak
                .or(Some(point.pepmass_header))
                .and_then(|mz| mz.is_finite().then_some(mz))
        },
        |point| {
            let error_value = display_error_value_for_point(point, unit);
            error_value.is_finite().then_some(error_value)
        },
        fallback_y_limit_for_unit(unit),
    );
    let y_desc = format!("Signed error ({})", unit.as_str());
    render_scatter_svg(title, plot_data, &y_desc, ticks, 0.3, true)
}

#[cfg(test)]
mod tests;
