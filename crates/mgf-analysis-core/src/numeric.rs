// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Shared numeric conversions, totally-ordered floats, and reservoir sampling.

use std::cmp::Ordering;

/// Converts `usize` counts cleanly to `f64`, saturating at `u32::MAX`.
#[must_use]
pub fn usize_to_f64(value: usize) -> f64 {
    f64::from(u32::try_from(value).unwrap_or(u32::MAX))
}

/// Floors non-negative finite floats into `usize`, returning `0` otherwise.
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "value is checked finite and clamped to [0.0, u32::MAX] after floor()"
)]
pub fn floor_to_usize(value: f64) -> usize {
    if !value.is_finite() || value <= 0.0 {
        return 0;
    }
    value.floor().min(f64::from(u32::MAX)) as usize
}

/// Newtype over `f64` implementing `Eq` and `Ord` via `f64::total_cmp`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct OrderedF64(pub(crate) f64);

impl PartialEq for OrderedF64 {
    fn eq(&self, other: &Self) -> bool {
        self.0.total_cmp(&other.0).is_eq()
    }
}

impl Eq for OrderedF64 {}

impl Ord for OrderedF64 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl PartialOrd for OrderedF64 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Deterministic `SplitMix64` reservoir index in `0..seen.max(1)`.
#[must_use]
pub(crate) fn reservoir_index(seen: usize) -> usize {
    let stream_index = u64::try_from(seen).unwrap_or(u64::MAX).max(1);
    let mut z = stream_index.wrapping_mul(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^= z >> 31;
    usize::try_from(z % stream_index).unwrap_or(0)
}

/// Writes `value` into `series[index]` when `index < series.len()`.
pub(crate) fn write_reservoir_slot(series: &mut [f64], index: usize, value: f64) {
    if let Some(slot) = series.get_mut(index) {
        *slot = value;
    }
}
