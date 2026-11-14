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
const fn paul_tol_entry(index: usize) -> (&'static str, (u8, u8, u8)) {
    // `index % 8` is below the eight-entry table above, so the slot exists.
    #[allow(clippy::indexing_slicing)] // see above
    [
        ("#4477AA", (0x44, 0x77, 0xAA)),
        ("#66CCEE", (0x66, 0xCC, 0xEE)),
        ("#228833", (0x22, 0x88, 0x33)),
        ("#CCBB44", (0xCC, 0xBB, 0x44)),
        ("#EE6677", (0xEE, 0x66, 0x77)),
        ("#AA3377", (0xAA, 0x33, 0x77)),
        ("#BBBBBB", (0xBB, 0xBB, 0xBB)),
        ("#004488", (0x00, 0x44, 0x88)),
    ][index % 8]
}

#[must_use]
pub const fn adduct_family_color_hex(family: AdductFamily) -> &'static str {
    let palette_index = adduct_family_rank(family);
    paul_tol_entry(palette_index).0
}

#[must_use]
pub const fn adduct_family_color(family: AdductFamily) -> plotters::style::RGBColor {
    let (_, (r, g, b)) = paul_tol_entry(adduct_family_rank(family));
    plotters::style::RGBColor(r, g, b)
}

#[must_use]
pub fn adduct_family_shape_style(family: AdductFamily, alpha: f32) -> plotters::style::ShapeStyle {
    let (_, (r, g, b)) = paul_tol_entry(adduct_family_rank(family));
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
        let span = u64::try_from(total.saturating_sub(1)).unwrap_or(u64::MAX);
        let normalized = u64::try_from(normalized).unwrap_or(u64::MAX);
        let scaled = normalized.saturating_mul(255) / span.max(1);
        let inverted = 255_u64.saturating_sub(scaled);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tolerance_step_color_handles_endpoints_and_large_step_counts_without_overflow() {
        for total in [2usize, 4, 10, 100_000_000] {
            let first = tolerance_step_color(0, total);
            let mid = tolerance_step_color(total / 2, total);
            let last = tolerance_step_color(total - 1, total);
            assert!(first.starts_with('#') && first.len() == 7);
            assert!(mid.starts_with('#') && mid.len() == 7);
            assert!(last.starts_with('#') && last.len() == 7);
            assert_ne!(
                first, last,
                "endpoints must map to distinct palette colors for total={total}"
            );
        }
    }
}
