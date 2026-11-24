// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Precursor error statistics, error buckets, running medians, and warning records.

pub(crate) mod record;
pub(crate) mod stats;

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::adduct::AdductFamily;
use crate::numeric::OrderedF64;

/// Reservoir cap for raw plot point retention.
pub(crate) const MAX_PLOT_POINTS: usize = 10_000;
/// Reservoir cap for empirical CDF values.
pub(crate) const MAX_ECDF_POINTS: usize = 20_000;

/// Display unit for mass-error distributions and compliance thresholds.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ErrorUnit {
    /// Absolute mass error in millidaltons (mDa).
    Mda,
    /// Relative mass error in parts-per-million (ppm).
    Ppm,
}

impl ErrorUnit {
    /// Short display unit suffix (`"mDa"` or `"ppm"`).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Mda => "mDa",
            Self::Ppm => "ppm",
        }
    }

    /// Reference cutoffs displayed on log-scale plots.
    #[must_use]
    pub const fn plot_thresholds(self) -> &'static [f64] {
        match self {
            Self::Mda => &[0.1, 0.5, 1.0, 5.0],
            Self::Ppm => &[0.5, 1.0, 5.0, 10.0],
        }
    }
}

/// Typed tolerance compliance bucket matching the summary grid in the UI.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ErrorBucket {
    /// `<= 0.1 mDa`.
    Within0_1Mda,
    /// `<= 0.5 mDa`.
    Within0_5Mda,
    /// `<= 1.0 mDa`.
    Within1Mda,
    /// `<= 5.0 mDa`.
    Within5Mda,
    /// `> 5.0 mDa`.
    Above5Mda,
    /// `<= 0.5 ppm`.
    Within0_5Ppm,
    /// `<= 1.0 ppm`.
    Within1Ppm,
    /// `<= 5.0 ppm`.
    Within5Ppm,
    /// `<= 10.0 ppm`.
    Within10Ppm,
    /// `> 10.0 ppm`.
    Above10Ppm,
}

impl ErrorBucket {
    /// All ten compliance buckets in display order.
    pub const ALL: [Self; 10] = [
        Self::Within0_1Mda,
        Self::Within0_5Mda,
        Self::Within1Mda,
        Self::Within5Mda,
        Self::Above5Mda,
        Self::Within0_5Ppm,
        Self::Within1Ppm,
        Self::Within5Ppm,
        Self::Within10Ppm,
        Self::Above10Ppm,
    ];

    /// Card label shown in compliance tiles.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Within0_1Mda => "≤ 0.1 mDa",
            Self::Within0_5Mda => "≤ 0.5 mDa",
            Self::Within1Mda => "≤ 1.0 mDa",
            Self::Within5Mda => "≤ 5.0 mDa",
            Self::Above5Mda => "> 5.0 mDa",
            Self::Within0_5Ppm => "≤ 0.5 ppm",
            Self::Within1Ppm => "≤ 1.0 ppm",
            Self::Within5Ppm => "≤ 5.0 ppm",
            Self::Within10Ppm => "≤ 10.0 ppm",
            Self::Above10Ppm => "> 10.0 ppm",
        }
    }

    /// Severity step (`0..=4`) mapped onto the BATLOW palette.
    #[must_use]
    pub const fn severity_level(self) -> usize {
        match self {
            Self::Within0_1Mda | Self::Within0_5Ppm => 0,
            Self::Within0_5Mda | Self::Within1Ppm => 1,
            Self::Within1Mda | Self::Within5Ppm => 2,
            Self::Within5Mda | Self::Within10Ppm => 3,
            Self::Above5Mda | Self::Above10Ppm => 4,
        }
    }
}

/// Dual-heap streaming exact median tracker.
#[derive(Clone, Debug, Default)]
pub(crate) struct MedianTracker {
    lower: BinaryHeap<OrderedF64>,
    upper: BinaryHeap<Reverse<OrderedF64>>,
}

impl MedianTracker {
    /// Pushes one value into the tracker, ignoring non-finite inputs.
    pub(crate) fn push(&mut self, value: f64) {
        if !value.is_finite() {
            return;
        }
        if self.lower.peek().is_none_or(|entry| value <= entry.0) {
            self.lower.push(OrderedF64(value));
        } else {
            self.upper.push(Reverse(OrderedF64(value)));
        }

        if self.lower.len() > self.upper.len() + 1 {
            if let Some(OrderedF64(top)) = self.lower.pop() {
                self.upper.push(Reverse(OrderedF64(top)));
            }
        } else if self.upper.len() > self.lower.len()
            && let Some(Reverse(OrderedF64(top))) = self.upper.pop()
        {
            self.lower.push(OrderedF64(top));
        }
    }

    /// Returns the median of all pushed finite values, or `0.0` when empty.
    #[must_use]
    pub(crate) fn median(&self) -> f64 {
        if self.lower.is_empty() && self.upper.is_empty() {
            0.0
        } else if self.lower.len() > self.upper.len() {
            self.lower.peek().map_or(0.0, |entry| entry.0)
        } else {
            let lower = self.lower.peek().map_or(0.0, |entry| entry.0);
            let upper = self.upper.peek().map_or(0.0, |entry| entry.0.0);
            lower.midpoint(upper)
        }
    }
}

/// Single observed spectrum point retained for scatter and recalibration diagnostics.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlotPoint {
    /// Grouped adduct family for legend colorization.
    pub adduct_family: AdductFamily,
    /// Observed precursor `m/z` from `PEPMASS=`/`PRECURSOR_MZ=`.
    pub pepmass_header: f64,
    /// Matched MS2 precursor peak near `PEPMASS`, if detected.
    pub ms2_precursor_peak: Option<f64>,
    /// Signed mass error in Da (`observed - expected`).
    pub signed_error_da: f64,
    /// Signed mass error in ppm.
    pub signed_error_ppm: f64,
    /// Theoretical precursor `m/z` used as reference.
    pub expected_mass: Option<f64>,
}

/// Single observation recorded into [`PrecursorStats`](stats::PrecursorStats).
#[derive(Clone, Copy, Debug)]
pub(crate) struct ErrorMeasurement<'a> {
    /// Absolute mass error (Da).
    pub(crate) abs_error_da: f64,
    /// Absolute mass error (ppm).
    pub(crate) abs_ppm: f64,
    /// Adduct family label.
    pub(crate) adduct_family: AdductFamily,
    /// Signed error (ppm).
    pub(crate) ppm_error: f64,
    /// Signed error (Da).
    pub(crate) signed_error_da: f64,
    /// Header precursor `m/z`.
    pub(crate) pepmass_header: f64,
    /// Detected MS2 precursor peak, if any.
    pub(crate) ms2_precursor_peak: Option<f64>,
    /// Optional SMILES annotation.
    pub(crate) smiles: Option<&'a str>,
    /// Reference neutral exact mass (Da).
    pub(crate) calculated_mass: Option<f64>,
    /// Expected ionized precursor `m/z`.
    pub(crate) expected_mass: Option<f64>,
    /// Optional molecular formula annotation.
    pub(crate) formula: Option<&'a str>,
}

/// Warning metadata aggregated per unparsable SMILES string.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WarningDetail {
    /// Occurrence count across spectra.
    pub count: usize,
    /// First non-empty `FORMULA=` observed alongside this SMILES.
    pub formula: Option<String>,
}

/// Worst-error metadata aggregated per SMILES exceeding `0.01 Da`.
#[derive(Clone, Debug, PartialEq)]
pub struct HighErrorSmilesDetail {
    /// Occurrence count above threshold.
    pub count: usize,
    /// Reference neutral exact mass.
    pub calculated_mass: Option<f64>,
    /// Derived expected precursor `m/z`.
    pub expected_mass: Option<f64>,
    /// Optional formula string.
    pub formula: Option<String>,
    /// Worst absolute error (Da).
    pub max_abs_error_da: Option<f64>,
    /// Associated absolute error (ppm).
    pub max_abs_error_ppm: Option<f64>,
    /// Observed precursor `m/z` for the worst occurrence.
    pub observed_precursor_mz: Option<f64>,
}
