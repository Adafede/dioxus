// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Scatter/ECDF chart renderers. These compose data helpers (prep + legend +
//! points) with colour helpers and `plotters` to emit SVG strings.

use crate::metrics::PlotPoint;

use super::color::{adduct_family_shape_style, tolerance_step_color, tolerance_step_rgb};
use super::data::{
    build_ecdf_points, display_error_value_for_point, embed_svg_legend, fallback_y_limit_for_unit,
    format_threshold_value, prepare_scatter_plot_data,
};

/// Renders an ECDF SVG for the observed error values.
///
/// # Panics
///
/// Panics if the SVG backend cannot fill or present the drawing area.
#[must_use]
pub fn render_ecdf_svg(title: &str, values: &[f64], thresholds: &[f64], unit: &str) -> String {
    use plotters::prelude::*;
    use plotters::series::LineSeries;

    let width = 900u32;
    let height = 520u32;
    let mut buffer = String::new();
    let root = SVGBackend::with_string(&mut buffer, (width, height)).into_drawing_area();
    root.fill(&WHITE).unwrap();

    let legend_items = thresholds
        .iter()
        .enumerate()
        .map(|(index, threshold)| {
            let label = format!("≤ {} {unit}", format_threshold_value(*threshold));
            (label, tolerance_step_color(index, thresholds.len()))
        })
        .collect::<Vec<_>>();

    let observed_min = values
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .fold(f64::INFINITY, f64::min);
    let x_min = if observed_min.is_finite() {
        observed_min.max(1e-6)
    } else {
        1e-6
    };
    let observed_max = values
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .fold(0.0, f64::max);
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
            .build_cartesian_2d((x_min..x_max).log_scale(), y_min..y_max)
            .unwrap();

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
            .draw()
            .unwrap();

        chart
            .draw_series(LineSeries::new(
                vec![(x_min, y_min), (x_max, y_min)],
                ShapeStyle::from(&RGBColor(203, 213, 225)).stroke_width(1),
            ))
            .unwrap();

        let plot_points = build_ecdf_points(values, x_min, x_max);
        chart
            .draw_series(LineSeries::new(
                plot_points,
                ShapeStyle::from(&RGBColor(37, 99, 235)).stroke_width(2),
            ))
            .unwrap();

        for (index, threshold) in thresholds.iter().enumerate() {
            let x = threshold.clamp(x_min, x_max);
            let color = tolerance_step_rgb(index, thresholds.len());
            chart
                .draw_series(LineSeries::new(
                    vec![(x, y_min), (x, y_max)],
                    ShapeStyle::from(&color).stroke_width(1),
                ))
                .unwrap();
        }

        root.present().unwrap();
    }

    drop(root);
    embed_svg_legend(&buffer, &legend_items, "Thresholds", 900.0, 520.0)
}

/// Renders a scatter plot of signed mass-bias errors by adduct family.
///
/// # Panics
///
/// Panics if the SVG backend cannot fill or present the drawing area.
#[must_use]
pub fn render_mass_bias_svg(title: &str, points: &[PlotPoint]) -> String {
    use plotters::prelude::*;
    use plotters::series::{LineSeries, PointSeries};

    let width = 900u32;
    let height = 520u32;
    let mut buffer = String::new();
    let root = SVGBackend::with_string(&mut buffer, (width, height)).into_drawing_area();
    root.fill(&WHITE).unwrap();

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
    let legend_items = plot_data.legend_items;
    let x_min = plot_data.x_min;
    let x_max = plot_data.x_max;
    let y_limit = plot_data.y_limit;
    let y_min = -y_limit;
    let y_max = y_limit;
    let points_by_family = plot_data.series;

    {
        let mut chart = ChartBuilder::on(&root)
            .margin_top(28)
            .margin_right(32)
            .margin_bottom(36)
            .margin_left(48)
            .caption(title, ("sans-serif", 18).into_font())
            .set_label_area_size(LabelAreaPosition::Left, 72)
            .set_label_area_size(LabelAreaPosition::Bottom, 64)
            .build_cartesian_2d(x_min..x_max, y_min..y_max)
            .unwrap();

        chart
            .configure_mesh()
            .axis_style(ShapeStyle::from(&RGBColor(148, 163, 184)))
            .light_line_style(ShapeStyle::from(&RGBColor(226, 232, 240)))
            .bold_line_style(ShapeStyle::from(&RGBColor(100, 116, 139)))
            .x_desc("Observed precursor 𝑚/𝑧")
            .y_desc("Signed error (Da)")
            .x_label_style(("sans-serif", 11).into_font())
            .y_label_style(("sans-serif", 11).into_font())
            .draw()
            .unwrap();

        chart
            .draw_series(LineSeries::new(
                vec![(x_min, 0.0f64), (x_max, 0.0f64)],
                ShapeStyle::from(&RGBColor(148, 163, 184)).stroke_width(1),
            ))
            .unwrap();

        for (family, points) in points_by_family {
            let style = adduct_family_shape_style(family, 0.4);
            chart
                .draw_series(PointSeries::of_element(
                    points.iter().copied(),
                    1.6,
                    style,
                    &|coord, size, style| Circle::new(coord, size, style.filled()),
                ))
                .unwrap();
        }

        root.present().unwrap();
    }

    drop(root);
    embed_svg_legend(&buffer, &legend_items, "Adducts", 900.0, 520.0)
}

/// Renders a scatter plot of signed mass-bias errors using the requested display unit.
///
/// # Panics
///
/// Panics if the SVG backend cannot fill or present the drawing area.
#[must_use]
pub fn render_absolute_mass_bias_svg(
    title: &str,
    points: &[PlotPoint],
    unit: &str,
    ticks: &[f64],
) -> String {
    use plotters::prelude::*;
    use plotters::series::{LineSeries, PointSeries};

    let width = 900u32;
    let height = 520u32;
    let mut buffer = String::new();
    let root = SVGBackend::with_string(&mut buffer, (width, height)).into_drawing_area();
    root.fill(&WHITE).unwrap();

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
    let legend_items = plot_data.legend_items;
    let x_min = plot_data.x_min;
    let x_max = plot_data.x_max;
    let y_limit = plot_data.y_limit;
    let y_min = -y_limit;
    let y_max = y_limit;
    let points_by_family = plot_data.series;
    let signed_error_label = format!("Signed error ({unit})");

    {
        let mut chart = ChartBuilder::on(&root)
            .margin_top(28)
            .margin_right(32)
            .margin_bottom(36)
            .margin_left(48)
            .caption(title, ("sans-serif", 18).into_font())
            .set_label_area_size(LabelAreaPosition::Left, 72)
            .set_label_area_size(LabelAreaPosition::Bottom, 64)
            .build_cartesian_2d(x_min..x_max, y_min..y_max)
            .unwrap();

        chart
            .configure_mesh()
            .axis_style(ShapeStyle::from(&RGBColor(148, 163, 184)))
            .light_line_style(ShapeStyle::from(&RGBColor(226, 232, 240)))
            .bold_line_style(ShapeStyle::from(&RGBColor(100, 116, 139)))
            .x_desc("Observed precursor 𝑚/𝑧")
            .y_desc(signed_error_label)
            .x_label_style(("sans-serif", 11).into_font())
            .y_label_style(("sans-serif", 11).into_font())
            .draw()
            .unwrap();

        chart
            .draw_series(LineSeries::new(
                vec![(x_min, 0.0), (x_max, 0.0)],
                ShapeStyle::from(&RGBColor(148, 163, 184)).stroke_width(1),
            ))
            .unwrap();

        for tick in ticks {
            if *tick <= 0.0 {
                continue;
            }
            let positive_tick = (*tick).min(y_limit);
            let negative_tick = -positive_tick;
            if !(y_min..=y_max).contains(&positive_tick) {
                continue;
            }
            chart
                .draw_series(LineSeries::new(
                    vec![(x_min, positive_tick), (x_max, positive_tick)],
                    ShapeStyle::from(&RGBColor(226, 232, 240)).stroke_width(1),
                ))
                .unwrap();
            chart
                .draw_series(LineSeries::new(
                    vec![(x_min, negative_tick), (x_max, negative_tick)],
                    ShapeStyle::from(&RGBColor(226, 232, 240)).stroke_width(1),
                ))
                .unwrap();
        }

        for (family, points) in points_by_family {
            let style = adduct_family_shape_style(family, 0.3);
            chart
                .draw_series(PointSeries::of_element(
                    points
                        .iter()
                        .map(|(x, value)| (*x, value.clamp(-y_limit, y_limit))),
                    1.6,
                    style,
                    &|coord, size, style| Circle::new(coord, size, style.filled()),
                ))
                .unwrap();
        }

        root.present().unwrap();
    }

    drop(root);
    embed_svg_legend(&buffer, &legend_items, "Adducts", 900.0, 520.0)
}
