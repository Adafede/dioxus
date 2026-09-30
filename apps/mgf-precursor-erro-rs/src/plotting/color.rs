// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Colour helpers: adduct-family palettes, Paul Tol colours, and the
//! tolerance-step LUT (driven by the `prismatic` BATLOW palette).
//!
//! These are pure functions shared by the data-preparation and chart-rendering
//! modules — no `plotters` rendering logic lives here.

use prismatica::crameri::BATLOW;

use crate::metrics::AdductFamily;

pub(super) const fn adduct_family_rank(family: AdductFamily) -> usize {
    match family {
        AdductFamily::Protonated => 0,
        AdductFamily::Deprotonated => 1,
        AdductFamily::AlkaliAmmonium => 2,
        AdductFamily::MetalComplex => 3,
        AdductFamily::Halide => 4,
        AdductFamily::Other => 5,
    }
}

/// Paul Tol's qualitative palette, wrapped cyclically by family rank.
const fn paul_tol_palette(index: usize) -> &'static str {
    // `index % 8` is below the eight-entry table above, so the slot exists.
    #[allow(clippy::indexing_slicing)] // see above
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

/// Resolves a tolerance step to its `BATLOW` palette entry.
///
/// Step `index` of `total_steps` is first clamped to the existing range, then
/// either snapped to one of four discrete positions (few steps) or mapped
/// linearly onto the palette (many steps). Both arms yield an index below the
/// table's length:
///
/// * the `total <= 4` arm picks one of `200`, `150`, `100`, `50`, all below
///   `200`, and the table has 256 entries;
/// * the other arm returns `255 - normalized * 255 / (total - 1)` with
///   `normalized <= total - 1`, so the quotient is at most `255` and the
///   result lies in `0..=255`.
#[allow(clippy::indexing_slicing)] // see above
fn batlow_entry(index: usize, total_steps: usize) -> [u8; 3] {
    let total = total_steps.max(2);
    let normalized = index.min(total.saturating_sub(1));
    let lut_index = if total <= 4 {
        let discrete_positions = [200usize, 150, 100, 50];
        discrete_positions[normalized.min(discrete_positions.len().saturating_sub(1))]
    } else {
        let span = u32::try_from(total.saturating_sub(1)).unwrap_or(u32::MAX);
        let normalized = u32::try_from(normalized).unwrap_or(u32::MAX);
        let inverted = 255_u32.saturating_sub((normalized * 255_u32) / span.max(1));
        usize::try_from(inverted).unwrap_or(usize::MAX)
    };
    BATLOW.lut[lut_index]
}

#[must_use]
pub fn tolerance_step_color(index: usize, total_steps: usize) -> String {
    let [r, g, b] = batlow_entry(index, total_steps);
    format!("#{r:02x}{g:02x}{b:02x}")
}

#[must_use]
pub fn tolerance_step_rgb(index: usize, total_steps: usize) -> plotters::style::RGBColor {
    let [r, g, b] = batlow_entry(index, total_steps);
    plotters::style::RGBColor(r, g, b)
}
