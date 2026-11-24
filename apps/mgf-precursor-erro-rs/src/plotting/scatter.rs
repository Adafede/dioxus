// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Scatter/ECDF chart renderers. These compose data helpers (prep + legend +
//! points) with colour helpers and `plotters` to emit SVG strings.

use mgf_analysis_core::PlotPoint;

use super::color::{adduct_family_shape_style, tolerance_step_color, tolerance_step_rgb};
use super::data::{
    ScatterPlotData, build_ecdf_points, display_error_value_for_point, fallback_y_limit_for_unit,
    format_threshold_value, prepare_scatter_plot_data,
};
use super::svg::embed_svg_legend;
use crate::errors::PlotError;

/// Renders an ECDF SVG for the observed error values.
///
/// # Errors
///
/// Returns an error if the SVG backend cannot fill or present the drawing area.
pub fn render_ecdf_svg(
    title: &str,
    values: &[f64],
    thresholds: &[f64],
    unit: &str,
) -> Result<String, PlotError> {
    use plotters::prelude::*;
    use plotters::series::LineSeries;

    let width = 900u32;
    let height = 520u32;
    let mut buffer = String::new();
    let root = SVGBackend::with_string(&mut buffer, (width, height)).into_drawing_area();
    root.fill(&WHITE)?;

    let legend_items = thresholds
        .iter()
        .enumerate()
        .map(|(index, threshold)| {
            let label = format!("≤ {} {unit}", format_threshold_value(*threshold));
            (label, tolerance_step_color(index, thresholds.len()))
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
    x_max = if unit == "mDa" {
        x_max.max(5.0)
    } else {
        x_max.max(10.0)
    };
    let y_floor = 1e-6f64;
    let y_min = y_floor;
    let y_max = 1.0f64;

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

        chart
            .configure_mesh()
            .axis_style(ShapeStyle::from(&RGBColor(148, 163, 184)))
            .light_line_style(ShapeStyle::from(&RGBColor(226, 232, 240)))
            .bold_line_style(ShapeStyle::from(&RGBColor(100, 116, 139)))
            .x_desc(if unit == "ppm" {
                format!("Relative error ({unit})")
            } else {
                format!("Absolute error ({unit})")
            })
            .y_desc("cumulative fraction")
            .x_label_style(("sans-serif", 11).into_font())
            .y_label_style(("sans-serif", 11).into_font())
            .draw()?;

        chart.draw_series(LineSeries::new(
            [(x_min, y_min), (x_max, y_min)],
            ShapeStyle::from(&RGBColor(203, 213, 225)).stroke_width(1),
        ))?;

        let plot_points = build_ecdf_points(values, x_min, x_max);
        chart.draw_series(LineSeries::new(
            plot_points,
            ShapeStyle::from(&RGBColor(37, 99, 235)).stroke_width(2),
        ))?;

        for (index, threshold) in thresholds.iter().enumerate() {
            let x = threshold.clamp(x_min, x_max);
            let color = tolerance_step_rgb(index, thresholds.len());
            chart.draw_series(LineSeries::new(
                [(x, y_min), (x, y_max)],
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
    use plotters::prelude::*;
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

        chart
            .configure_mesh()
            .axis_style(ShapeStyle::from(&RGBColor(148, 163, 184)))
            .light_line_style(ShapeStyle::from(&RGBColor(226, 232, 240)))
            .bold_line_style(ShapeStyle::from(&RGBColor(100, 116, 139)))
            .x_desc("Observed precursor 𝑚/𝑧")
            .y_desc(y_desc)
            .x_label_style(("sans-serif", 11).into_font())
            .y_label_style(("sans-serif", 11).into_font())
            .draw()?;

        chart.draw_series(LineSeries::new(
            [(x_min, 0.0f64), (x_max, 0.0f64)],
            ShapeStyle::from(&RGBColor(148, 163, 184)).stroke_width(1),
        ))?;

        for &tick in ticks {
            if tick <= 0.0 {
                continue;
            }
            let positive_tick = tick.min(y_limit);
            if !(y_min..=y_max).contains(&positive_tick) {
                continue;
            }
            for y in [positive_tick, -positive_tick] {
                chart.draw_series(LineSeries::new(
                    [(x_min, y), (x_max, y)],
                    ShapeStyle::from(&RGBColor(226, 232, 240)).stroke_width(1),
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

/// Renders a scatter plot of signed mass-bias errors by adduct family.
///
/// # Errors
///
/// Returns an error if the SVG backend cannot fill or present the drawing area.
pub fn render_mass_bias_svg(title: &str, points: &[PlotPoint]) -> Result<String, PlotError> {
    let plot_data = prepare_scatter_plot_data(
        points,
        |point| point.ms2_precursor_peak.or(Some(point.pepmass_header)),
        |point| {
            point
                .signed_error_da
                .is_finite()
                .then_some(point.signed_error_da)
        },
        fallback_y_limit_for_unit("mDa") / 1000.0,
    );
    render_scatter_svg(title, plot_data, "Signed error (Da)", &[], 0.4, false)
}

/// Renders a scatter plot of signed mass-bias errors using the requested display unit.
///
/// # Errors
///
/// Returns an error if the SVG backend cannot fill or present the drawing area.
pub fn render_absolute_mass_bias_svg(
    title: &str,
    points: &[PlotPoint],
    unit: &str,
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
    let y_desc = format!("Signed error ({unit})");
    render_scatter_svg(title, plot_data, &y_desc, ticks, 0.3, true)
}

#[cfg(test)]
mod tests;
