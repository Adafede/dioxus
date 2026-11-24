// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Data-preparation primitives shared by charts: ECDF binning, scatter sampling,
//! and value/axis formatting helpers.

use std::collections::HashMap;

use mgf_analysis_core::{AdductFamily, PlotPoint};

use super::color::adduct_family_color_hex;

/// Prepared scatter-series data and viewport bounds.
#[derive(Clone, Debug)]
pub(crate) struct ScatterPlotData {
    /// `(family_label, color_hex)` pairs for legend rendering.
    pub(crate) legend_items: Vec<(&'static str, &'static str)>,
    /// Padded minimum X bound.
    pub(crate) x_min: f64,
    /// Padded maximum X bound.
    pub(crate) x_max: f64,
    /// Symmetric positive/negative Y axis limit.
    pub(crate) y_limit: f64,
    /// Sampled `(x, y)` series grouped by adduct family.
    pub(crate) series: Vec<(AdductFamily, Vec<(f64, f64)>)>,
}

/// Groups raw scatter points by [`AdductFamily`] and downsamples large series.
#[must_use]
pub(crate) fn prepare_scatter_plot_data<F, G>(
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
    families.sort_by_key(|family| family.rank());
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

/// Formats numeric tolerance threshold values without trailing zeros.
#[must_use]
pub(crate) fn format_threshold_value(value: f64) -> String {
    let formatted = format!("{value:.6}");
    let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
    if trimmed.is_empty() {
        "0".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Selects signed error in `unit` (`"ppm"` or `"mDa"`) from a [`PlotPoint`].
#[must_use]
pub(crate) fn display_error_value_for_point(point: &PlotPoint, unit: &str) -> f64 {
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

/// Builds step-function `(x, fraction)` points for an ECDF chart.
#[must_use]
pub(crate) fn build_ecdf_points(values: &[f64], x_min: f64, x_max: f64) -> Vec<(f64, f64)> {
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

/// Uniformly downsamples `points` to at most `max_points` (keeping first/last).
#[must_use]
pub(crate) fn sample_scatter_points(points: Vec<(f64, f64)>, max_points: usize) -> Vec<(f64, f64)> {
    if points.len() <= max_points.max(1) {
        return points;
    }
    let target = max_points.max(1);
    let mut sampled = Vec::with_capacity(target);
    for slot in 0..target {
        let point_index = (slot * points.len()) / target;
        if let Some(point) = points.get(point_index) {
            sampled.push(*point);
        }
    }
    if sampled.last() != points.last()
        && let Some(last) = points.last()
    {
        sampled.push(*last);
    }
    sampled
}
