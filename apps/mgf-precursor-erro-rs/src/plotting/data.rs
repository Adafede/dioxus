// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Data-preparation primitives shared by every chart: numeric conversions,
//! scatter-plot point assembly, legend embedding, ECDF sampling and axis/value
//! formatting. Colour helpers are imported from [`super::color`].

use std::{collections::HashMap, fmt::Write};

use crate::metrics::{AdductFamily, PlotPoint, ScatterPlotData};

use super::color::{adduct_family_color_hex, adduct_family_rank};

// `pub(crate)`, not `pub(super)`: `metrics::plot` reaches these too, and a
// reader editing one has no way to know a second copy exists if the check
// cannot see both.
pub(crate) fn usize_to_f64(value: usize) -> f64 {
    f64::from(u32::try_from(value).unwrap_or(u32::MAX))
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub(crate) fn floor_to_usize(value: f64) -> usize {
    if !value.is_finite() || value <= 0.0 {
        return 0;
    }
    value.floor() as usize
}

pub(super) fn mean_and_std_dev(values: &[f64]) -> Option<(f64, f64)> {
    let mut count = 0usize;
    let mut sum = 0.0f64;
    for value in values.iter().copied().filter(|v| v.is_finite()) {
        count += 1;
        sum += value;
    }
    if count == 0 {
        return None;
    }

    let n = usize_to_f64(count);
    let mean = sum / n;
    let variance = values
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / n;
    Some((mean, variance.sqrt()))
}

#[must_use]
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
    let mut raw_x_min = f64::INFINITY;
    let mut raw_x_max = f64::NEG_INFINITY;
    let mut data_max = 0.0f64;

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
        raw_x_min = raw_x_min.min(x_value);
        raw_x_max = raw_x_max.max(x_value);
        data_max = data_max.max(y_value.abs());
        family_points
            .entry(point.adduct_family)
            .or_default()
            .push((x_value, y_value));
    }

    let x_min = if raw_x_min.is_finite() {
        raw_x_min
    } else {
        1.0
    };
    let x_max = raw_x_max.max(x_min + 1.0);
    let x_span = (x_max - x_min).max(1.0);
    let x_min = x_min - x_span * 0.05;
    let x_max = x_max + x_span * 0.05;
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
        .map(|(family, _)| (family.as_str(), adduct_family_color_hex(*family)))
        .collect();

    ScatterPlotData {
        legend_items,
        x_min,
        x_max,
        y_limit,
        series,
    }
}

pub(super) fn embed_svg_legend<L: AsRef<str>, C: AsRef<str>>(
    mut svg_markup: String,
    legend_items: &[(L, C)],
    title: &str,
    width: f64,
    height: f64,
) -> String {
    if legend_items.is_empty() {
        return svg_markup;
    }
    let Some(position) = svg_markup.rfind("</svg>") else {
        return svg_markup;
    };

    let item_height = 13.5;
    let inset = 18.0;
    let title_width = 44.0;
    let marker_radius = 3.2;
    let padding_x = 12.0;
    let padding_y = 10.0;
    let label_width = legend_items
        .iter()
        .map(|(label, _)| label.as_ref().len())
        .max()
        .unwrap_or(0);
    let label_width = f64::from(u32::try_from(label_width).unwrap_or(u32::MAX));
    let content_width = (label_width * 5.6).clamp(72.0, 140.0) + 24.0;
    let entry_width = content_width + 20.0;

    // Wrap legend into multiple rows to fit within plot width
    let max_row_width = width.mul_add(0.9, -(title_width + 50.0));
    let items_per_row = floor_to_usize((max_row_width / entry_width).floor().max(1.0));
    let num_rows = legend_items.len().div_ceil(items_per_row);

    let box_width =
        usize_to_f64(items_per_row.min(legend_items.len())).mul_add(entry_width, padding_x * 2.0);
    let box_height = usize_to_f64(num_rows).mul_add(item_height, padding_y * 2.0) + 12.0;
    let legend_x = ((width - box_width) / 2.0)
        .max(inset)
        .min(width - box_width - inset);
    let legend_y = (height - box_height - inset).max(inset);

    let title_x = legend_x + 10.0;
    let title_y = legend_y + 12.0;
    let items_start_x = title_x + title_width + 10.0;
    let rect_x = legend_x;
    let rect_y = legend_y - 2.0;

    let tail = svg_markup.split_off(position);
    let _ = write!(
        svg_markup,
        "\n<g>\n            <rect x=\"{rect_x}\" y=\"{rect_y}\" width=\"{box_width}\" height=\"{box_height}\" rx=\"8\" ry=\"8\" fill=\"#f8fafc\" fill-opacity=\"0.97\" stroke=\"#cbd5e1\" stroke-width=\"0.8\" />\n            <text x=\"{title_x}\" y=\"{title_y}\" font-family=\"Inter, sans-serif\" font-size=\"10.5\" font-weight=\"600\" fill=\"#0f172a\">{title}</text>\n            "
    );

    for (index, (family, color)) in legend_items.iter().enumerate() {
        let family = family.as_ref();
        let color = color.as_ref();
        let row = index / items_per_row;
        let col = index % items_per_row;
        let item_x = usize_to_f64(col).mul_add(entry_width, items_start_x);
        let item_y = usize_to_f64(row).mul_add(item_height, legend_y + 15.0);
        let marker_x = item_x + 8.0;
        let text_x = item_x + 18.0;
        let text_y = item_y + 2.0;
        let marker_y = item_y - 2.0;
        let _ = write!(
            svg_markup,
            "<g>\n                <circle cx=\"{marker_x}\" cy=\"{marker_y}\" r=\"{marker_radius}\" fill=\"{color}\" />\n                <text x=\"{text_x}\" y=\"{text_y}\" font-family=\"Inter, sans-serif\" font-size=\"10\" fill=\"#334155\">{family}</text>\n            </g>"
        );
    }

    svg_markup.push_str("\n        </g>\n");
    svg_markup.push_str(&tail);
    svg_markup
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
pub(super) fn fallback_y_limit_for_unit(unit: &str) -> f64 {
    match unit {
        "ppm" => 10.0,
        _ => 5.0,
    }
}

#[must_use]
pub fn make_svg_responsive(mut svg_markup: String) -> String {
    let Some(svg_start) = svg_markup.find("<svg") else {
        return svg_markup;
    };
    let tag_end = svg_markup
        .get(svg_start..)
        .and_then(|rest| rest.find('>').map(|rel| svg_start + rel + 1))
        .unwrap_or(svg_markup.len());
    let Some(tag) = svg_markup.get(svg_start..tag_end) else {
        return svg_markup;
    };
    let mut normalized = tag
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
    svg_markup.replace_range(svg_start..tag_end, &normalized);
    svg_markup
}

#[must_use]
pub fn build_ecdf_points(values: &[f64], x_min: f64, x_max: f64) -> Vec<(f64, f64)> {
    let mut sorted: Vec<f64> = values
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .take(50_000)
        .collect();
    if sorted.is_empty() {
        return vec![(x_min, 0.0), (x_max, 1.0)];
    }

    sorted.sort_by(f64::total_cmp);
    let total = sorted.len();
    let mut points = Vec::with_capacity(sorted.len().saturating_mul(2) + 2);
    points.push((x_min, 0.0));

    let mut index = 0usize;
    let mut previous_y = 0.0f64;
    while let Some(&value) = sorted.get(index) {
        let mut next_index = index + 1;
        while sorted
            .get(next_index)
            .is_some_and(|&next| (next - value).abs() <= f64::EPSILON)
        {
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
        // `slot < target` and `target >= 1`, so `slot * len / target` is below
        // `len` for every slot: the sampled point always exists.
        let point_index = (slot * points.len()) / target;
        if let Some(point) = points.get(point_index) {
            sampled.push(*point);
        }
    }
    // `points.len() > target >= 1` past the early return above, so there is
    // always a last point to close the series with.
    if sampled.last() != points.last()
        && let Some(last) = points.last()
    {
        sampled.push(*last);
    }
    sampled
}
