// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Colour palettes for adduct families and BATLOW tolerance steps.

use mgf_analysis_core::AdductFamily;
use plotters::style::{RGBAColor, RGBColor, ShapeStyle};
use prismatica::crameri::BATLOW;

pub(super) const MESH_AXIS_RGB: RGBColor = RGBColor(148, 163, 184);
pub(super) const MESH_LIGHT_RGB: RGBColor = RGBColor(226, 232, 240);
pub(super) const MESH_BOLD_RGB: RGBColor = RGBColor(100, 116, 139);
pub(super) const MS1_BEFORE_RGB: RGBColor = RGBColor(68, 119, 170);
pub(super) const MS2_BEFORE_RGB: RGBColor = RGBColor(255, 140, 0);
pub(super) const AFTER_RGB: RGBColor = RGBColor(34, 136, 51);

pub(super) fn filled_rgba(rgb: RGBColor, alpha: f64) -> ShapeStyle {
    ShapeStyle::from(&RGBAColor(rgb.0, rgb.1, rgb.2, alpha)).filled()
}

const PAUL_TOL_QUALITATIVE: [(&str, (u8, u8, u8)); 8] = [
    ("#4477AA", (0x44, 0x77, 0xAA)),
    ("#66CCEE", (0x66, 0xCC, 0xEE)),
    ("#228833", (0x22, 0x88, 0x33)),
    ("#CCBB44", (0xCC, 0xBB, 0x44)),
    ("#EE6677", (0xEE, 0x66, 0x77)),
    ("#AA3377", (0xAA, 0x33, 0x77)),
    ("#BBBBBB", (0xBB, 0xBB, 0xBB)),
    ("#004488", (0x00, 0x44, 0x88)),
];

const fn paul_tol_entry(index: usize) -> (&'static str, (u8, u8, u8)) {
    match index % 8 {
        0 => PAUL_TOL_QUALITATIVE[0],
        1 => PAUL_TOL_QUALITATIVE[1],
        2 => PAUL_TOL_QUALITATIVE[2],
        3 => PAUL_TOL_QUALITATIVE[3],
        4 => PAUL_TOL_QUALITATIVE[4],
        5 => PAUL_TOL_QUALITATIVE[5],
        6 => PAUL_TOL_QUALITATIVE[6],
        _ => PAUL_TOL_QUALITATIVE[7],
    }
}

/// Returns the Paul Tol qualitative hex color assigned to `family`.
#[must_use]
pub(crate) const fn adduct_family_color_hex(family: AdductFamily) -> &'static str {
    paul_tol_entry(family.rank()).0
}

/// Returns a filled `ShapeStyle` for scatter points belonging to `family`.
#[must_use]
pub(crate) fn adduct_family_shape_style(
    family: AdductFamily,
    alpha: f32,
) -> plotters::style::ShapeStyle {
    let (_, (r, g, b)) = paul_tol_entry(family.rank());
    let alpha = f64::from(alpha.clamp(0.0, 1.0));
    plotters::style::ShapeStyle::from(&plotters::style::RGBAColor(r, g, b, alpha)).filled()
}

fn batlow_entry(index: usize, total_steps: usize) -> [u8; 3] {
    let total = total_steps.max(2);
    let normalized = index.min(total.saturating_sub(1));
    let lut_index = if total <= 4 {
        match normalized {
            0 => 200,
            1 => 150,
            2 => 100,
            _ => 50,
        }
    } else {
        let span = u64::try_from(total.saturating_sub(1)).unwrap_or(u64::MAX);
        let normalized = u64::try_from(normalized).unwrap_or(u64::MAX);
        let scaled = normalized.saturating_mul(255) / span.max(1);
        let inverted = 255_u64.saturating_sub(scaled);
        usize::try_from(inverted).unwrap_or(usize::MAX)
    };
    BATLOW.lut.get(lut_index).copied().unwrap_or([0, 0, 0])
}

/// Maps tolerance bucket `(index, total_steps)` to a CSS hex color string.
#[must_use]
pub(crate) fn tolerance_step_color(index: usize, total_steps: usize) -> String {
    let [r, g, b] = batlow_entry(index, total_steps);
    format!("#{r:02x}{g:02x}{b:02x}")
}

/// Maps tolerance bucket `(index, total_steps)` to a plotters `RGBColor`.
#[must_use]
pub(crate) fn tolerance_step_rgb(index: usize, total_steps: usize) -> plotters::style::RGBColor {
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
