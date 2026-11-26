// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

use super::*;

#[test]
fn render_cumulative_error_three_curves_empty_arrays_returns_ok_empty() {
    let result = render_cumulative_error_three_curves(&[], &[], &[], "ppm", &[]);
    assert!(
        result.is_ok(),
        "empty three-curves input should return Ok, not panic"
    );
    assert_eq!(result.unwrap(), "");
}

#[test]
fn render_cumulative_error_three_curves_all_nan_returns_ok_empty() {
    let result =
        render_cumulative_error_three_curves(&[f64::NAN], &[f64::NAN], &[f64::NAN], "ppm", &[]);
    assert!(
        result.is_ok(),
        "all-NaN three-curves should return Ok empty, not panic"
    );
    assert_eq!(result.unwrap(), "");
}

#[test]
fn render_cumulative_error_three_curves_single_point_returns_ok_with_svg() {
    let svg = render_cumulative_error_three_curves(&[1.0], &[2.0], &[0.5], "ppm", &[5.0]).unwrap();
    assert!(svg.contains("<svg"), "expected SVG root element");
    assert!(
        svg.contains("<polyline") || svg.contains("<path"),
        "expected curve geometry"
    );
}

// ── internal helpers ─────────────────────────────────────────────

#[test]
fn cumulative_points_empty_returns_empty_vec() {
    let result = cumulative_points(&[]);
    assert_eq!(result, Vec::new());
}

#[test]
fn cumulative_points_filters_non_finite_and_coalesces_ties() {
    let result = cumulative_points(&[-2.0, f64::NAN, 1.0, 1.0, f64::INFINITY]);
    assert_eq!(result, vec![(1.0, 2.0 / 3.0 * 100.0), (2.0, 100.0)]);
    let stepped = step_series(&[(1.0, 100.0)], 0.0, 2.0);
    assert_eq!(
        stepped,
        vec![(0.0, 0.0), (1.0, 0.0), (1.0, 100.0), (2.0, 100.0)],
        "single point must produce vertical step from 0 to 100%"
    );
}

#[test]
fn append_cumulative_legend_no_svg_tag_is_noop() {
    let input = "no svg here";
    let result = append_cumulative_legend(input.to_string());
    assert_eq!(result, input);
}

#[test]
fn cumulative_legend_text_starts_inside_1200px_canvas() {
    let svg = render_cumulative_error_three_curves(&[1.0], &[2.0], &[0.5], "ppm", &[5.0]).unwrap();
    assert!(
        svg.contains(r#"x="990""#) || svg.contains(r#"x="980""#),
        "legend rect must leave enough horizontal room for label inside 1200px canvas: {svg}"
    );
}
