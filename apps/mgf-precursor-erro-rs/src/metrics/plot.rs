// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

// The shapes the plotters draw: histograms, ECDF points, scatter series, and
// the per-family summaries that go into them. Extracted from `mod.rs`, which
// held these alongside the statistics that produce them -- 200 lines of plot
// vocabulary in the middle of 450 lines of median arithmetic.
//
// [`PrecursorStats`](super::PrecursorStats) is the only thing that builds
// these, so the dependency runs one way: `stats` knows about `plot`, and
// `plot` knows nothing about `stats`.
pub(crate) const MAX_PLOT_POINTS: usize = 10_000;
pub(crate) const MAX_ECDF_POINTS: usize = 20_000;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HistogramData {
    pub bins: Vec<usize>,
    pub min: f64,
    pub max: f64,
}

// `usize_to_f64` and `floor_to_usize` live in [`crate::plotting::data`], which
// is where the other two callers already reach them from. Kept here as
// re-exports so `merge` and this module's other users do not have to know.
pub(super) use crate::plotting::data::{floor_to_usize, usize_to_f64};

impl HistogramData {
    #[must_use]
    pub(crate) fn new(bin_count: usize, min: f64, max: f64) -> Self {
        Self {
            bins: vec![0; bin_count],
            min,
            max,
        }
    }

    pub(crate) fn add_value(&mut self, value: f64) {
        if self.bins.is_empty() || !value.is_finite() {
            return;
        }
        let clamped = value.clamp(self.min, self.max);
        let idx = if (self.max - self.min).abs() < f64::EPSILON {
            0
        } else {
            let bins_last = usize_to_f64(self.bins.len().saturating_sub(1));
            let index = (((clamped - self.min) / (self.max - self.min)) * bins_last).floor();
            floor_to_usize(index)
        };
        let idx = idx.min(self.bins.len().saturating_sub(1));
        // An empty histogram has nowhere to count into, which is a degenerate
        // configuration rather than a value that fell out of the bin index.
        if let Some(bin) = self.bins.get_mut(idx) {
            *bin = bin.saturating_add(1);
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AdductFamily {
    Protonated,
    Deprotonated,
    AlkaliAmmonium,
    MetalComplex,
    Halide,
    Other,
}

impl AdductFamily {
    #[must_use]
    pub(crate) fn from_label(label: &str) -> Self {
        fn contains_ignore_ascii_case(haystack: &str, needle: &str) -> bool {
            haystack
                .as_bytes()
                .windows(needle.len())
                .any(|window| window.eq_ignore_ascii_case(needle.as_bytes()))
        }

        let trimmed = label.trim();
        let compact: std::borrow::Cow<'_, str> = if trimmed.contains(' ') {
            std::borrow::Cow::Owned(trimmed.replace(' ', ""))
        } else {
            std::borrow::Cow::Borrowed(trimmed)
        };
        let s = compact.as_ref();
        if contains_ignore_ascii_case(s, "[M+H]") || contains_ignore_ascii_case(s, "[M+2H]") {
            Self::Protonated
        } else if contains_ignore_ascii_case(s, "[M-H]") || contains_ignore_ascii_case(s, "[M-2H]")
        {
            Self::Deprotonated
        } else if contains_ignore_ascii_case(s, "[M+NA]")
            || contains_ignore_ascii_case(s, "[M+K]")
            || contains_ignore_ascii_case(s, "[M+NH4]")
        {
            Self::AlkaliAmmonium
        } else if contains_ignore_ascii_case(s, "MG")
            || contains_ignore_ascii_case(s, "CA")
            || contains_ignore_ascii_case(s, "FE")
        {
            Self::MetalComplex
        } else if contains_ignore_ascii_case(s, "CL") || contains_ignore_ascii_case(s, "BR") {
            Self::Halide
        } else {
            Self::Other
        }
    }

    #[must_use]
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Protonated => "Protonated",
            Self::Deprotonated => "Deprotonated",
            Self::AlkaliAmmonium => "Alkali / ammonium",
            Self::MetalComplex => "Metal / complex",
            Self::Halide => "Halide",
            Self::Other => "Other",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlotPoint {
    pub adduct_family: AdductFamily,
    pub pepmass_header: f64, // PEPMASS from MGF header (metadata block)
    pub ms2_precursor_peak: Option<f64>, // Actual MS2 precursor peak observed in fragment list (near PEPMASS)
    pub signed_error_da: f64,
    pub signed_error_ppm: f64,
    pub expected_mass: Option<f64>, // Theoretical precursor mass for error calculation
}

#[derive(Clone, Debug, Default)]
pub(crate) struct PlotPointSample {
    pub seen: usize,
    pub points: Vec<PlotPoint>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ErrorMeasurement<'a> {
    pub abs_error_da: f64,
    pub abs_ppm: f64,
    pub adduct_family: AdductFamily,
    pub ppm_error: f64,
    pub signed_error_da: f64,
    pub pepmass_header: f64,
    pub ms2_precursor_peak: Option<f64>,
    pub smiles: Option<&'a str>,
    pub calculated_mass: Option<f64>,
    pub expected_mass: Option<f64>,
    pub formula: Option<&'a str>,
}

#[derive(Clone, Copy, Debug)]
// `pub(super)`, fields included, because `stats` folds these in. They were
// module-private when all three files were one, and `merge` reaches the two
// `push_*` helpers on `PrecursorStats` in `stats` the same way.
pub(super) struct HighErrorSmilesUpdate<'a> {
    pub(super) abs_error_da: f64,
    pub(super) abs_ppm: f64,
    pub(super) pepmass_header: f64,
    pub(super) smiles: Option<&'a str>,
    pub(super) calculated_mass: Option<f64>,
    pub(super) expected_mass: Option<f64>,
    pub(super) formula: Option<&'a str>,
}

impl PlotPointSample {
    pub(crate) fn push(&mut self, point: PlotPoint) {
        self.seen = self.seen.saturating_add(1);
        if self.points.len() < MAX_PLOT_POINTS {
            self.points.push(point);
        } else {
            let stream_index = u64::try_from(self.seen).unwrap_or(u64::MAX);
            let replacement_index = usize::try_from(
                (stream_index
                    .wrapping_mul(0x9e37_79b9_7f4a_7c15)
                    .wrapping_add(0xbf58_476d_1ce4_e5b9))
                    % stream_index,
            )
            .unwrap_or(0);
            if replacement_index < MAX_PLOT_POINTS
                && let Some(existing) = self.points.get_mut(replacement_index)
            {
                *existing = point;
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct ScatterPlotData {
    pub legend_items: Vec<(String, String)>,
    pub x_min: f64,
    pub x_max: f64,
    pub y_limit: f64,
    pub series: Vec<(AdductFamily, Vec<(f64, f64)>)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WarningDetail {
    pub count: usize,
    pub formula: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HighErrorSmilesDetail {
    pub count: usize,
    pub calculated_mass: Option<f64>,
    pub expected_mass: Option<f64>,
    pub formula: Option<String>,
    pub max_abs_error_da: Option<f64>,
    pub max_abs_error_ppm: Option<f64>,
    pub observed_precursor_mz: Option<f64>,
}
