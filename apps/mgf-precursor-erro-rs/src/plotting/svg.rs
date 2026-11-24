// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! SVG legend injection and responsive `viewBox` normalization helpers.

use std::fmt::Write as _;

use mgf_analysis_core::{floor_to_usize, usize_to_f64};

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

fn extract_svg_attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let needle = format!("{name}=\"");
    let start = tag.find(&needle)? + needle.len();
    let rest = tag.get(start..)?;
    let end = rest.find('"')?;
    rest.get(..end)
}

/// Normalizes root `<svg>` attributes for fluid width with explicit `viewBox`.
#[must_use]
pub(crate) fn make_svg_responsive(mut svg_markup: String) -> String {
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
    let raw_width = extract_svg_attr(tag, "width").map(str::to_owned);
    let raw_height = extract_svg_attr(tag, "height").map(str::to_owned);
    let vw = raw_width
        .as_deref()
        .filter(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
        .unwrap_or("900");
    let vh = raw_height
        .as_deref()
        .filter(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
        .unwrap_or("520");

    let mut normalized = tag.to_string();
    if let Some(w) = raw_width.as_deref() {
        normalized = normalized.replace(&format!("width=\"{w}\""), "width=\"100%\"");
    }
    if let Some(h) = raw_height.as_deref() {
        normalized = normalized.replace(&format!("height=\"{h}\""), "");
    }
    if !normalized.contains("viewBox=") {
        normalized = normalized.replacen("<svg", &format!("<svg viewBox=\"0 0 {vw} {vh}\""), 1);
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

#[cfg(test)]
mod tests {
    use super::make_svg_responsive;

    #[test]
    fn make_svg_responsive_preserves_1200px_dimensions_in_viewbox() {
        let raw = r#"<svg width="1200" height="400" xmlns="http://www.w3.org/2000/svg"></svg>"#;
        let out = make_svg_responsive(raw.to_string());
        assert!(out.contains(r#"width="100%""#), "got: {out}");
        assert!(out.contains(r#"viewBox="0 0 1200 400""#), "got: {out}");
        assert!(!out.contains(r#"height="400""#), "got: {out}");
    }
}
