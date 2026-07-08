use std::{collections::HashMap, fmt::Write};

use prismatica::crameri::BATLOW;

use crate::metrics::{AdductFamily, PlotPoint, ScatterPlotData};

const fn adduct_family_rank(family: AdductFamily) -> usize {
    match family {
        AdductFamily::Protonated => 0,
        AdductFamily::Deprotonated => 1,
        AdductFamily::AlkaliAmmonium => 2,
        AdductFamily::MetalComplex => 3,
        AdductFamily::Halide => 4,
        AdductFamily::Other => 5,
    }
}

const fn paul_tol_palette(index: usize) -> &'static str {
    [
        "#4477AA", "#66CCEE", "#228833", "#CCBB44", "#EE6677", "#AA3377", "#BBBBBB", "#004488",
    ][index % 8]
}

#[must_use]
pub fn adduct_family_color_hex(family: AdductFamily) -> String {
    let palette_index = adduct_family_rank(family);
    paul_tol_palette(palette_index).to_string()
}

#[must_use]
pub fn adduct_family_color(family: AdductFamily) -> plotters::style::RGBColor {
    let color = adduct_family_color_hex(family);
    let hex = color.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    plotters::style::RGBColor(r, g, b)
}

#[must_use]
pub fn adduct_family_shape_style(family: AdductFamily, alpha: f32) -> plotters::style::ShapeStyle {
    let color = adduct_family_color_hex(family);
    let hex = color.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    let alpha = f64::from(alpha.clamp(0.0, 1.0));
    plotters::style::ShapeStyle::from(&plotters::style::RGBAColor(r, g, b, alpha)).filled()
}

pub fn prepare_scatter_plot_data<F, G>(
    points: &[PlotPoint],
    x_value_fn: F,
    y_value_fn: G,
    fallback_y_limit: f64,
) -> ScatterPlotData
where
    F: Fn(&PlotPoint) -> Option<f64>,
    G: Fn(&PlotPoint) -> Option<f64>,
{
    let mut family_points: HashMap<AdductFamily, Vec<(f64, f64)>> = HashMap::new();
    let mut x_values = Vec::new();
    let mut y_values = Vec::new();

    for point in points {
        let Some(x_value) = x_value_fn(point) else {
            continue;
        };
        let Some(y_value) = y_value_fn(point) else {
            continue;
        };
        if !x_value.is_finite() || !y_value.is_finite() {
            continue;
        }
        x_values.push(x_value);
        y_values.push(y_value);
        family_points
            .entry(point.adduct_family)
            .or_default()
            .push((x_value, y_value));
    }

    let x_min = x_values
        .iter()
        .copied()
        .fold(f64::INFINITY, f64::min)
        .min(1.0);
    let x_max = x_values
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max)
        .max(x_min + 1.0);
    let x_span = (x_max - x_min).max(1.0);
    let x_min = x_min - x_span * 0.05;
    let x_max = x_max + x_span * 0.05;

    let data_max = y_values
        .iter()
        .copied()
        .map(f64::abs)
        .fold(0.0, f64::max);
    let y_limit = if fallback_y_limit > 0.0 {
        data_max.min(fallback_y_limit)
    } else {
        data_max
    };

    let mut families = family_points.keys().copied().collect::<Vec<_>>();
    families.sort_by_key(|family| adduct_family_rank(*family));
    let family_count = families.len().max(1);
    let max_points_per_family = (900usize / family_count).max(120usize);
    let mut series = Vec::with_capacity(families.len());
    for family in families {
        let sampled = sample_scatter_points(
            family_points.remove(&family).unwrap_or_default(),
            max_points_per_family,
        );
        series.push((family, sampled));
    }

    let legend_items = series
        .iter()
        .map(|(family, _)| {
            (
                family.as_str().to_string(),
                adduct_family_color_hex(*family),
            )
        })
        .collect();

    ScatterPlotData {
        legend_items,
        x_min,
        x_max,
        y_limit,
        series,
    }
}

fn embed_svg_legend(
    svg_markup: &str,
    legend_items: &[(String, String)],
    title: &str,
    width: f64,
    height: f64,
) -> String {
    if legend_items.is_empty() {
        return svg_markup.to_string();
    }

    let mut legend_entries = String::new();
    let item_height = 13.5;
    let inset = 18.0;
    let title_width = 44.0;
    let marker_radius = 3.2;
    let padding_x = 12.0;
    let label_width = legend_items
        .iter()
        .map(|(label, _)| label.len())
        .max()
        .unwrap_or(0);
    let label_width = f64::from(u32::try_from(label_width).unwrap_or(u32::MAX));
    let content_width = (label_width * 5.6).clamp(72.0, 170.0) + 24.0;
    let entry_width = content_width + 20.0;
    let legend_count = f64::from(u32::try_from(legend_items.len()).unwrap_or(u32::MAX));
    let box_width = (entry_width * legend_count) + (title_width + 12.0) + (padding_x * 2.0);
    let box_height = item_height + 20.0;
    let legend_x = ((width - box_width) / 2.0)
        .max(inset)
        .min(width - box_width - inset);
    let legend_y = (height - box_height - inset).max(inset);

    let title_x = legend_x + 10.0;
    let title_y = legend_y + 12.0;
    let items_start_x = title_x + title_width + 10.0;

    for (index, (family, color)) in legend_items.iter().enumerate() {
        let item_x =
            f64::from(u32::try_from(index).unwrap_or(u32::MAX)).mul_add(entry_width, items_start_x);
        let marker_x = item_x + 8.0;
        let text_x = item_x + 18.0;
        let text_y = legend_y + 13.0;
        let marker_y = legend_y + 10.0;
        let _ = write!(
            legend_entries,
            "<g>\n                <circle cx=\"{marker_x}\" cy=\"{marker_y}\" r=\"{marker_radius}\" fill=\"{color}\" />\n                <text x=\"{text_x}\" y=\"{text_y}\" font-family=\"Inter, sans-serif\" font-size=\"10\" fill=\"#334155\">{family}</text>\n            </g>"
        );
    }

    let rect_x = legend_x;
    let rect_y = legend_y - 2.0;
    let legend_group = format!(
        "<g>\n            <rect x=\"{rect_x}\" y=\"{rect_y}\" width=\"{box_width}\" height=\"{box_height}\" rx=\"8\" ry=\"8\" fill=\"#f8fafc\" fill-opacity=\"0.97\" stroke=\"#cbd5e1\" stroke-width=\"0.8\" />\n            <text x=\"{title_x}\" y=\"{title_y}\" font-family=\"Inter, sans-serif\" font-size=\"10.5\" font-weight=\"600\" fill=\"#0f172a\">{title}</text>\n            {legend_entries}\n        </g>"
    );

    svg_markup.rfind("</svg>").map_or_else(
        || svg_markup.to_string(),
        |position| {
            let mut result = svg_markup[..position].to_string();
            result.push('\n');
            result.push_str(&legend_group);
            result.push('\n');
            result.push_str(&svg_markup[position..]);
            result
        },
    )
}

#[must_use]
pub fn tolerance_step_color(index: usize, total_steps: usize) -> String {
    let total = total_steps.max(2);
    let normalized = index.min(total.saturating_sub(1));
    let lut_index = if total <= 4 {
        let discrete_positions = [200usize, 150, 100, 50];
        discrete_positions[normalized.min(discrete_positions.len().saturating_sub(1))]
    } else {
        let span = u32::try_from(total.saturating_sub(1)).unwrap_or(u32::MAX);
        let normalized = u32::try_from(normalized).unwrap_or(u32::MAX);
        let inverted = 255_u32.saturating_sub((normalized * 255_u32) / span.max(1));
        inverted as usize
    };
    let [r, g, b] = BATLOW.lut[lut_index];
    format!("#{r:02x}{g:02x}{b:02x}")
}

#[must_use]
pub fn tolerance_step_rgb(index: usize, total_steps: usize) -> plotters::style::RGBColor {
    let total = total_steps.max(2);
    let normalized = index.min(total.saturating_sub(1));
    let lut_index = if total <= 4 {
        let discrete_positions = [200usize, 150, 100, 50];
        discrete_positions[normalized.min(discrete_positions.len().saturating_sub(1))]
    } else {
        let span = u32::try_from(total.saturating_sub(1)).unwrap_or(u32::MAX);
        let normalized = u32::try_from(normalized).unwrap_or(u32::MAX);
        let inverted = 255_u32.saturating_sub((normalized * 255_u32) / span.max(1));
        inverted as usize
    };
    let [r, g, b] = BATLOW.lut[lut_index];
    plotters::style::RGBColor(r, g, b)
}

#[must_use]
pub fn format_threshold_value(value: f64) -> String {
    let formatted = format!("{value:.6}");
    let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
    if trimmed.is_empty() {
        "0".to_string()
    } else {
        trimmed.to_string()
    }
}

#[must_use]
pub fn display_error_value(value: f64, unit: &str) -> f64 {
    if unit == "mDa" { value * 1000.0 } else { value }
}

#[must_use]
pub fn display_error_value_for_point(point: &PlotPoint, unit: &str) -> f64 {
    match unit {
        "ppm" => point.signed_error_ppm,
        _ => point.signed_error_da * 1000.0,
    }
}

#[must_use]
fn fallback_y_limit_for_unit(unit: &str) -> f64 {
    match unit {
        "ppm" => 10.0,
        _ => 5.0,
    }
}

#[must_use]
pub fn make_svg_responsive(svg_markup: String) -> String {
    let mut normalized = svg_markup;
    normalized = normalized
        .replace("width=\"800\"", "width=\"100%\"")
        .replace("width=\"900\"", "width=\"100%\"")
        .replace("height=\"460\"", "height=\"auto\"")
        .replace("height=\"520\"", "height=\"auto\"");
    if !normalized.contains("viewBox=") {
        normalized = normalized.replacen("<svg", "<svg viewBox=\"0 0 900 520\"", 1);
    }
    if !normalized.contains("preserveAspectRatio=") {
        normalized = normalized.replacen("<svg", "<svg preserveAspectRatio=\"xMidYMid meet\"", 1);
    }
    if !normalized.contains("style=") {
        normalized = normalized.replacen(
            "<svg",
            "<svg style=\"max-width:100%; height:auto; display:block; overflow:visible;\"",
            1,
        );
    }
    normalized
}

#[must_use]
pub fn build_ecdf_points(values: &[f64], x_min: f64, x_max: f64) -> Vec<(f64, f64)> {
    if values.is_empty() {
        return vec![(x_min, 0.0), (x_max, 1.0)];
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    if sorted.len() > 50_000 {
        sorted.truncate(50_000);
    }
    let total = sorted.len();
    let mut points = Vec::with_capacity(sorted.len().saturating_mul(2) + 2);
    points.push((x_min, 0.0));

    let mut index = 0usize;
    let mut previous_y = 0.0f64;
    while index < sorted.len() {
        let value = sorted[index];
        let mut next_index = index + 1;
        while next_index < sorted.len() && (sorted[next_index] - value).abs() <= f64::EPSILON {
            next_index += 1;
        }
        let next_index_u32 = u32::try_from(next_index).unwrap_or(u32::MAX);
        let total_u32 = u32::try_from(total).unwrap_or(u32::MAX);
        let y = f64::from(next_index_u32) / f64::from(total_u32);
        let x = value.max(x_min);
        points.push((x, previous_y));
        points.push((x, y));
        previous_y = y;
        index = next_index;
    }

    points.push((x_max, 1.0));
    points
}

#[must_use]
pub fn sample_scatter_points(points: Vec<(f64, f64)>, max_points: usize) -> Vec<(f64, f64)> {
    if points.len() <= max_points.max(1) {
        return points;
    }

    let target = max_points.max(1);
    let mut sampled = Vec::with_capacity(target);
    for slot in 0..target {
        let point_index = (slot * points.len()) / target;
        sampled.push(points[point_index]);
    }
    if sampled.last() != points.last() {
        sampled.push(points[points.len() - 1]);
    }
    sampled
}

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
                ShapeStyle::from(&RGBColor(37, 99, 235)).stroke_width(3),
            ))
            .unwrap();

        for (index, threshold) in thresholds.iter().enumerate() {
            let x = threshold.clamp(x_min, x_max);
            let color = tolerance_step_rgb(index, thresholds.len());
            chart
                .draw_series(LineSeries::new(
                    vec![(x, y_min), (x, y_max)],
                    ShapeStyle::from(&color).stroke_width(2),
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
        |point| Some(point.observed_precursor_mz),
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
                .observed_precursor_mz
                .is_finite()
                .then_some(point.observed_precursor_mz)
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
