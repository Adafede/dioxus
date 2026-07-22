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

    let data_max = y_values.iter().copied().map(f64::abs).fold(0.0, f64::max);
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
    let padding_y = 10.0;
    let label_width = legend_items
        .iter()
        .map(|(label, _)| label.len())
        .max()
        .unwrap_or(0);
    let label_width = f64::from(u32::try_from(label_width).unwrap_or(u32::MAX));
    let content_width = (label_width * 5.6).clamp(72.0, 140.0) + 24.0;
    let entry_width = content_width + 20.0;

    // Wrap legend into multiple rows to fit within plot width
    let max_row_width = (width * 0.9) - (title_width + 50.0);
    let items_per_row = ((max_row_width / entry_width).floor()).max(1.0) as usize;
    let num_rows = (legend_items.len() + items_per_row - 1) / items_per_row;

    let box_width =
        (items_per_row.min(legend_items.len()) as f64 * entry_width) + (padding_x * 2.0);
    let box_height = (num_rows as f64 * item_height) + (padding_y * 2.0) + 12.0;
    let legend_x = ((width - box_width) / 2.0)
        .max(inset)
        .min(width - box_width - inset);
    let legend_y = (height - box_height - inset).max(inset);

    let title_x = legend_x + 10.0;
    let title_y = legend_y + 12.0;
    let items_start_x = title_x + title_width + 10.0;

    for (index, (family, color)) in legend_items.iter().enumerate() {
        let row = index / items_per_row;
        let col = index % items_per_row;
        let item_x =
            f64::from(u32::try_from(col).unwrap_or(u32::MAX)).mul_add(entry_width, items_start_x);
        let item_y = legend_y + 15.0 + (row as f64 * item_height);
        let marker_x = item_x + 8.0;
        let text_x = item_x + 18.0;
        let text_y = item_y + 2.0;
        let marker_y = item_y - 2.0;
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
        .replace("height=\"460\"", "")
        .replace("height=\"520\"", "");
    if !normalized.contains("viewBox=") {
        normalized = normalized.replacen("<svg", "<svg viewBox=\"0 0 900 520\"", 1);
    }
    if !normalized.contains("preserveAspectRatio=") {
        normalized = normalized.replacen("<svg", "<svg preserveAspectRatio=\"xMidYMid meet\"", 1);
    }
    if !normalized.contains("style=") {
        normalized = normalized.replacen(
            "<svg",
            "<svg style=\"max-width:100%; display:block; overflow:visible;\"",
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

/// Renders a diagnostic plot comparing precursor errors before and after recalibration.
///
/// Creates a scatter plot with two overlaid series showing precursor errors in ppm,
/// allowing visual assessment of recalibration effectiveness.
#[must_use]
pub fn render_recalibration_diagnostic_ppm(errors_before: &[f64], errors_after: &[f64]) -> String {
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
        return buffer;
    }

    let error_min = all_errors.iter().copied().fold(f64::INFINITY, f64::min);
    let error_max = all_errors.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let error_span = (error_max - error_min).max(1.0);
    let error_min = error_min - error_span * 0.1;
    let error_max = error_max + error_span * 0.1;

    {
        let root = SVGBackend::with_string(&mut buffer, (1200, 400)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        let mut chart = ChartBuilder::on(&root)
            .caption(
                "Precursor Error (ppm) Before and After Recalibration",
                ("sans-serif", 20),
            )
            .margin(15)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d(
                0f64..(errors_before.len().max(errors_after.len()) as f64),
                error_min..error_max,
            )
            .unwrap();

        chart
            .configure_mesh()
            .x_desc("Scan Index")
            .y_desc("Error (ppm)")
            .draw()
            .unwrap();

        // Plot errors before recalibration
        chart
            .draw_series(errors_before.iter().enumerate().map(|(i, &error)| {
                Circle::new(
                    (i as f64, error),
                    3,
                    ShapeStyle::from(&RGBAColor(68, 119, 170, 0.4)).filled(),
                )
            }))
            .unwrap()
            .label("Before");

        // Plot errors after recalibration
        chart
            .draw_series(errors_after.iter().enumerate().map(|(i, &error)| {
                Circle::new(
                    (i as f64, error),
                    2,
                    ShapeStyle::from(&RGBAColor(34, 136, 51, 0.6)).filled(),
                )
            }))
            .unwrap()
            .label("After");

        chart
            .configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .draw()
            .unwrap();

        root.present().unwrap();
    }
    buffer
}

/// Renders a histogram comparing error distributions before and after recalibration.
#[must_use]
pub fn render_recalibration_diagnostic_histogram(
    errors_before: &[f64],
    errors_after: &[f64],
    bin_count: usize,
) -> String {
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
        return buffer;
    }

    let error_min = all_errors.iter().copied().fold(f64::INFINITY, f64::min);
    let error_max = all_errors.iter().copied().fold(f64::NEG_INFINITY, f64::max);

    // Compute histograms
    let mut hist_before = vec![0usize; bin_count];
    let mut hist_after = vec![0usize; bin_count];

    for &error in errors_before {
        if error.is_finite() {
            let normalized = (error - error_min) / (error_max - error_min + f64::EPSILON);
            let bin = (normalized * (bin_count as f64 - 1.0)).floor() as usize;
            if bin < bin_count {
                hist_before[bin] += 1;
            }
        }
    }

    for &error in errors_after {
        if error.is_finite() {
            let normalized = (error - error_min) / (error_max - error_min + f64::EPSILON);
            let bin = (normalized * (bin_count as f64 - 1.0)).floor() as usize;
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

    let bin_width = (error_max - error_min) / (bin_count as f64);

    {
        let root = SVGBackend::with_string(&mut buffer, (1200, 400)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        let mut chart = ChartBuilder::on(&root)
            .caption(
                "Error Distribution: Before vs After Recalibration",
                ("sans-serif", 20),
            )
            .margin(15)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d(error_min..error_max, 0usize..max_count)
            .unwrap();

        chart
            .configure_mesh()
            .x_desc("Error (ppm)")
            .y_desc("Count")
            .draw()
            .unwrap();

        // Draw histogram bars
        for (i, &count) in hist_before.iter().enumerate() {
            let x = error_min + (i as f64) * bin_width;
            chart
                .draw_series(std::iter::once(Rectangle::new(
                    [(x, 0), (x + bin_width * 0.9, count)],
                    ShapeStyle::from(&RGBAColor(68, 119, 170, 0.4)).filled(),
                )))
                .unwrap();
        }

        for (i, &count) in hist_after.iter().enumerate() {
            let x = error_min + (i as f64) * bin_width + bin_width * 0.45;
            chart
                .draw_series(std::iter::once(Rectangle::new(
                    [(x, 0), (x + bin_width * 0.45, count)],
                    ShapeStyle::from(&RGBAColor(34, 136, 51, 0.6)).filled(),
                )))
                .unwrap();
        }

        root.present().unwrap();
    }
    buffer
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
#[must_use]
pub fn render_recalibration_diagnostic_mz_comparison(
    precursor_ms1_values: &[f64],
    precursor_ms2_before: &[f64],
    precursor_ms2_after: &[f64],
) -> String {
    use plotters::prelude::*;

    let mut buffer = String::new();

    if precursor_ms1_values.is_empty() || precursor_ms2_before.is_empty() {
        return buffer;
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
        return buffer;
    }

    let min_mz = all_values.iter().copied().fold(f64::INFINITY, f64::min);
    let max_mz = all_values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let span = (max_mz - min_mz).max(1.0);
    let min_mz = min_mz - span * 0.05;
    let max_mz = max_mz + span * 0.05;

    {
        let root = SVGBackend::with_string(&mut buffer, (1200, 600)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        let mut chart = ChartBuilder::on(&root)
            .caption(
                "Precursor m/z: Before and After Recalibration",
                ("sans-serif", 20),
            )
            .margin(15)
            .x_label_area_size(45)
            .y_label_area_size(60)
            .right_y_label_area_size(0)
            .build_cartesian_2d(min_mz..max_mz, min_mz..max_mz)
            .unwrap();

        chart
            .configure_mesh()
            .x_desc("Reference Precursor m/z (MS1, Da)")
            .y_desc("Observed Precursor m/z (MS2, Da)")
            .draw()
            .unwrap();

        // Draw perfect calibration line (y = x)
        chart
            .draw_series(std::iter::once(PathElement::new(
                vec![(min_mz, min_mz), (max_mz, max_mz)],
                ShapeStyle::from(&RGBAColor(200, 50, 50, 0.5)),
            )))
            .unwrap()
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
            )
            .unwrap()
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
            )
            .unwrap()
            .label("After recalibration");

        chart
            .configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()
            .unwrap();

        root.present().unwrap();
    }
    buffer
}

/// Renders a diagnostic summary text showing improvement statistics.
#[must_use]
pub fn render_recalibration_summary_text(
    mean_before: f64,
    mean_after: f64,
    rms_before: f64,
    rms_after: f64,
    max_before: f64,
    max_after: f64,
) -> String {
    let mean_improvement = (mean_before.abs() - mean_after.abs()).abs();
    let rms_improvement = rms_before - rms_after;
    let max_improvement = max_before - max_after;

    format!(
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
    )
}

/// Four-stage error analysis showing the complete recalibration story.
///
/// Stage 1 (Blue): MS1 error vs theory
/// Stage 2 (Orange): MS2 error before recalibration vs theory
/// Stage 3 (Red): Calibration delta (MS2 - MS1) - the measurement discrepancy
/// Stage 4 (Green): MS2 error after recalibration vs theory
#[must_use]
pub fn render_error_quartet(
    error_ms1: &[f64],
    delta_ms2_ms1: &[f64],
    error_ms2_before: &[f64],
    error_ms2_after: &[f64],
) -> String {
    use plotters::prelude::*;

    let mut buffer = String::new();

    // Collect all errors to determine range
    let all_values: Vec<f64> = error_ms1
        .iter()
        .chain(delta_ms2_ms1.iter())
        .chain(error_ms2_before.iter())
        .chain(error_ms2_after.iter())
        .copied()
        .filter(|v| v.is_finite())
        .collect();

    if all_values.is_empty() {
        return buffer;
    }

    let min_val = all_values.iter().copied().fold(f64::INFINITY, f64::min);
    let max_val = all_values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let margin = (max_val - min_val).abs().max(0.5) * 0.15;

    {
        let root = SVGBackend::with_string(&mut buffer, (1200, 400)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        let mut chart = ChartBuilder::on(&root)
            .caption(
                "Four-Stage Error Analysis: Complete Recalibration Story",
                ("sans-serif", 18),
            )
            .margin(15)
            .x_label_area_size(60)
            .y_label_area_size(70)
            .build_cartesian_2d(-0.5f64..4.5f64, (min_val - margin)..(max_val + margin))
            .unwrap();

        chart
            .configure_mesh()
            .x_labels(4)
            .x_desc("Analysis Stage")
            .y_desc("Error (ppm)")
            .draw()
            .unwrap();

        // Reference line at y=0 (perfect accuracy)
        let red_style = ShapeStyle::from(&RED).stroke_width(1);
        let _ = chart.draw_series(std::iter::once(PathElement::new(
            vec![(-0.5, 0.0), (4.5, 0.0)],
            red_style,
        )));

        // Plot Stage 1: MS1 error (Blue)
        {
            let filtered: Vec<f64> = error_ms1
                .iter()
                .copied()
                .filter(|v| v.is_finite())
                .collect();
            if !filtered.is_empty() {
                let n = filtered.len() as f64;
                let mean = filtered.iter().sum::<f64>() / n;
                let std_dev = {
                    let var = filtered.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
                    var.sqrt()
                };
                let style = ShapeStyle::from(&RGBColor(68, 119, 170));
                let _ = chart.draw_series(std::iter::once(PathElement::new(
                    vec![(0.8, mean - std_dev), (0.8, mean + std_dev)],
                    style,
                )));
                let _ =
                    chart.draw_series(std::iter::once(Circle::new((0.8, mean), 6, style.filled())));
            }
        }

        // Plot Stage 2: MS2 before error (Orange)
        {
            let filtered: Vec<f64> = error_ms2_before
                .iter()
                .copied()
                .filter(|v| v.is_finite())
                .collect();
            if !filtered.is_empty() {
                let n = filtered.len() as f64;
                let mean = filtered.iter().sum::<f64>() / n;
                let std_dev = {
                    let var = filtered.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
                    var.sqrt()
                };
                let style = ShapeStyle::from(&RGBColor(255, 119, 0));
                let _ = chart.draw_series(std::iter::once(PathElement::new(
                    vec![(1.8, mean - std_dev), (1.8, mean + std_dev)],
                    style,
                )));
                let _ =
                    chart.draw_series(std::iter::once(Circle::new((1.8, mean), 6, style.filled())));
            }
        }

        // Plot Stage 3: MS2-MS1 delta (Red/Maroon)
        {
            let filtered: Vec<f64> = delta_ms2_ms1
                .iter()
                .copied()
                .filter(|v| v.is_finite())
                .collect();
            if !filtered.is_empty() {
                let n = filtered.len() as f64;
                let mean = filtered.iter().sum::<f64>() / n;
                let std_dev = {
                    let var = filtered.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
                    var.sqrt()
                };
                let style = ShapeStyle::from(&RGBColor(204, 51, 51));
                let _ = chart.draw_series(std::iter::once(PathElement::new(
                    vec![(2.8, mean - std_dev), (2.8, mean + std_dev)],
                    style,
                )));
                let _ =
                    chart.draw_series(std::iter::once(Circle::new((2.8, mean), 6, style.filled())));
            }
        }

        // Plot Stage 4: MS2 after error (Green)
        {
            let filtered: Vec<f64> = error_ms2_after
                .iter()
                .copied()
                .filter(|v| v.is_finite())
                .collect();
            if !filtered.is_empty() {
                let n = filtered.len() as f64;
                let mean = filtered.iter().sum::<f64>() / n;
                let std_dev = {
                    let var = filtered.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
                    var.sqrt()
                };
                let style = ShapeStyle::from(&RGBColor(51, 153, 51));
                let _ = chart.draw_series(std::iter::once(PathElement::new(
                    vec![(3.8, mean - std_dev), (3.8, mean + std_dev)],
                    style,
                )));
                let _ =
                    chart.draw_series(std::iter::once(Circle::new((3.8, mean), 6, style.filled())));
            }
        }

        root.present().unwrap();
    }

    buffer
}

/// Cumulative error distribution curves comparing MS1 and recalibrated MS2 errors.
///
/// Shows how many precursors have error magnitude ≤ x ppm, allowing comparison
/// of MS1 baseline with recalibrated MS2 improvement.
#[must_use]
pub fn render_cumulative_error_curves(error_ppm_ms1: &[f64], error_ppm_after: &[f64]) -> String {
    use plotters::prelude::*;

    let mut buffer = String::new();

    // Filter finite values and sort for CDF
    let mut ms1_errors: Vec<f64> = error_ppm_ms1
        .iter()
        .copied()
        .filter(|v| v.is_finite())
        .collect();
    let mut ms2_errors: Vec<f64> = error_ppm_after
        .iter()
        .copied()
        .filter(|v| v.is_finite())
        .collect();

    if ms1_errors.is_empty() || ms2_errors.is_empty() {
        return buffer;
    }

    ms1_errors.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    ms2_errors.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let min_error = ms1_errors
        .iter()
        .chain(ms2_errors.iter())
        .copied()
        .fold(f64::INFINITY, f64::min);
    let max_error = ms1_errors
        .iter()
        .chain(ms2_errors.iter())
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);

    let margin = (max_error - min_error).abs().max(1.0) * 0.1;

    {
        let root = SVGBackend::with_string(&mut buffer, (1200, 600)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        let mut chart = ChartBuilder::on(&root)
            .caption(
                "Cumulative Error Distribution: MS1 vs Recalibrated MS2",
                ("sans-serif", 18),
            )
            .margin(15)
            .x_label_area_size(60)
            .y_label_area_size(70)
            .build_cartesian_2d((min_error - margin)..(max_error + margin), 0f64..100f64)
            .unwrap();

        chart
            .configure_mesh()
            .x_desc("Absolute Error (ppm)")
            .y_desc("Cumulative Percentage (%)")
            .draw()
            .unwrap();

        // MS1 cumulative curve (blue)
        {
            let n = ms1_errors.len() as f64;
            let points: Vec<(f64, f64)> = ms1_errors
                .iter()
                .enumerate()
                .map(|(i, &err)| (err, (i as f64 + 1.0) / n * 100.0))
                .collect();

            let style = ShapeStyle::from(&RGBColor(68, 119, 170)).stroke_width(2);
            let _ = chart.draw_series(
                points
                    .iter()
                    .copied()
                    .zip(
                        points
                            .iter()
                            .skip(1)
                            .copied()
                            .chain(std::iter::once(points[0])),
                    )
                    .map(|((x1, y1), (x2, y2))| PathElement::new(vec![(x1, y1), (x2, y2)], style)),
            );
        }

        // MS2 (recalibrated) cumulative curve (green)
        {
            let n = ms2_errors.len() as f64;
            let points: Vec<(f64, f64)> = ms2_errors
                .iter()
                .enumerate()
                .map(|(i, &err)| (err, (i as f64 + 1.0) / n * 100.0))
                .collect();

            let style = ShapeStyle::from(&RGBColor(51, 153, 51)).stroke_width(2);
            let _ = chart.draw_series(
                points
                    .iter()
                    .copied()
                    .zip(
                        points
                            .iter()
                            .skip(1)
                            .copied()
                            .chain(std::iter::once(points[0])),
                    )
                    .map(|((x1, y1), (x2, y2))| PathElement::new(vec![(x1, y1), (x2, y2)], style)),
            );
        }

        root.present().unwrap();
    }

    buffer
}

/// Render cumulative error distribution with three curves: ms1, ms2_before, ms2_after
pub fn render_cumulative_error_three_curves(
    error_ms1: &[f64],
    error_before: &[f64],
    error_after: &[f64],
    unit: &str,
    _thresholds: Vec<f64>,
) -> String {
    use plotters::prelude::*;

    let mut buffer = String::new();

    // Filter finite values and sort for CDF
    let mut ms1_errors: Vec<f64> = error_ms1
        .iter()
        .copied()
        .filter(|v| v.is_finite())
        .collect();
    let mut before_errors: Vec<f64> = error_before
        .iter()
        .copied()
        .filter(|v| v.is_finite())
        .collect();
    let mut after_errors: Vec<f64> = error_after
        .iter()
        .copied()
        .filter(|v| v.is_finite())
        .collect();

    if ms1_errors.is_empty() || before_errors.is_empty() || after_errors.is_empty() {
        return buffer;
    }

    ms1_errors.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    before_errors.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    after_errors.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let min_error = ms1_errors
        .iter()
        .chain(before_errors.iter())
        .chain(after_errors.iter())
        .copied()
        .fold(f64::INFINITY, f64::min);
    let max_error = ms1_errors
        .iter()
        .chain(before_errors.iter())
        .chain(after_errors.iter())
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);

    let margin = (max_error - min_error).abs().max(1.0) * 0.1;

    {
        let root = SVGBackend::with_string(&mut buffer, (1200, 600)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        let title = format!("Cumulative Error Distribution ({})", unit);
        let mut chart = ChartBuilder::on(&root)
            .caption(&title, ("sans-serif", 18))
            .margin(15)
            .x_label_area_size(60)
            .y_label_area_size(70)
            .build_cartesian_2d((min_error - margin)..(max_error + margin), 0f64..100f64)
            .unwrap();

        chart
            .configure_mesh()
            .x_desc(format!("Error ({})", unit))
            .y_desc("Cumulative Percentage (%)")
            .draw()
            .unwrap();

        // MS1 cumulative curve (blue)
        {
            let n = ms1_errors.len() as f64;
            let points: Vec<(f64, f64)> = ms1_errors
                .iter()
                .enumerate()
                .map(|(i, &err)| (err, (i as f64 + 1.0) / n * 100.0))
                .collect();

            let style = ShapeStyle::from(&RGBColor(68, 119, 170)).stroke_width(1);
            let _ = chart.draw_series(
                points
                    .iter()
                    .copied()
                    .zip(
                        points
                            .iter()
                            .skip(1)
                            .copied()
                            .chain(std::iter::once(points[0])),
                    )
                    .map(|((x1, y1), (x2, y2))| PathElement::new(vec![(x1, y1), (x2, y2)], style)),
            );
        }

        // MS2 before cumulative curve (orange)
        {
            let n = before_errors.len() as f64;
            let points: Vec<(f64, f64)> = before_errors
                .iter()
                .enumerate()
                .map(|(i, &err)| (err, (i as f64 + 1.0) / n * 100.0))
                .collect();

            let style = ShapeStyle::from(&RGBColor(255, 140, 0)).stroke_width(1);
            let _ = chart.draw_series(
                points
                    .iter()
                    .copied()
                    .zip(
                        points
                            .iter()
                            .skip(1)
                            .copied()
                            .chain(std::iter::once(points[0])),
                    )
                    .map(|((x1, y1), (x2, y2))| PathElement::new(vec![(x1, y1), (x2, y2)], style)),
            );
        }

        // MS2 after cumulative curve (green)
        {
            let n = after_errors.len() as f64;
            let points: Vec<(f64, f64)> = after_errors
                .iter()
                .enumerate()
                .map(|(i, &err)| (err, (i as f64 + 1.0) / n * 100.0))
                .collect();

            let style = ShapeStyle::from(&RGBColor(51, 153, 51)).stroke_width(1);
            let _ = chart.draw_series(
                points
                    .iter()
                    .copied()
                    .zip(
                        points
                            .iter()
                            .skip(1)
                            .copied()
                            .chain(std::iter::once(points[0])),
                    )
                    .map(|((x1, y1), (x2, y2))| PathElement::new(vec![(x1, y1), (x2, y2)], style)),
            );
        }

        root.present().unwrap();
    }

    // Add legend to the SVG
    let legend_items = vec![
        ("MS1 Precursor (PEPMASS)", (68u8, 119u8, 170u8)),
        ("MS2 Before Correction", (255u8, 140u8, 0u8)),
        ("MS2 After Correction", (51u8, 153u8, 51u8)),
    ];

    let mut legend_svg = String::new();
    legend_svg.push_str(r#"<g id="legend" font-size="12" font-family="sans-serif">"#);

    for (idx, (label, (r, g, b))) in legend_items.iter().enumerate() {
        let y = 30 + (idx * 20);
        let x = 1050;

        // Color box
        legend_svg.push_str(&format!(
            r#"<rect x="{}" y="{}" width="15" height="15" fill="rgb({},{},{})" stroke="none"/>"#,
            x,
            y - 10,
            r,
            g,
            b
        ));

        // Label
        legend_svg.push_str(&format!(
            r#"<text x="{}" y="{}" fill="black" font-size="12">{}</text>"#,
            x + 20,
            y,
            label
        ));
    }
    legend_svg.push_str("</g>");

    // Inject legend into SVG
    if let Some(pos) = buffer.rfind("</svg>") {
        let (before, after) = buffer.split_at(pos);
        buffer = format!("{}{}{}", before, legend_svg, after);
    }

    buffer
}
