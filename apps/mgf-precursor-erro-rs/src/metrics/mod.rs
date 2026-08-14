use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap};

pub const MAX_PLOT_POINTS: usize = 10_000;
pub const MAX_ECDF_POINTS: usize = 20_000;

#[derive(Clone, Debug, PartialEq)]
pub struct HistogramData {
    pub bins: Vec<usize>,
    pub min: f64,
    pub max: f64,
}

fn usize_to_f64(value: usize) -> f64 {
    f64::from(u32::try_from(value).unwrap_or(u32::MAX))
}

fn floor_to_usize(value: f64) -> usize {
    if !value.is_finite() || value <= 0.0 {
        return 0;
    }

    let mut result = 0usize;
    let mut remaining = value.floor();
    loop {
        if remaining < 1.0 {
            break result;
        }
        result = result.saturating_add(1);
        remaining -= 1.0;
    }
}

impl HistogramData {
    #[must_use]
    pub fn new(bin_count: usize, min: f64, max: f64) -> Self {
        Self {
            bins: vec![0; bin_count],
            min,
            max,
        }
    }

    pub fn add_value(&mut self, value: f64) {
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
        self.bins[idx] = self.bins[idx].saturating_add(1);
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
    pub fn from_label(label: &str) -> Self {
        let normalized = label.trim().replace(' ', "").to_ascii_uppercase();
        if normalized.contains("[M+H]")
            || normalized.contains("[M+2H]")
            || normalized.contains("[M+NH4]")
        {
            Self::Protonated
        } else if normalized.contains("[M-H]") || normalized.contains("[M-2H]") {
            Self::Deprotonated
        } else if normalized.contains("[M+NA]")
            || normalized.contains("[M+K]")
            || normalized.contains("[M+NH4]")
        {
            Self::AlkaliAmmonium
        } else if normalized.contains("MG")
            || normalized.contains("CA")
            || normalized.contains("FE")
        {
            Self::MetalComplex
        } else if normalized.contains("CL") || normalized.contains("BR") {
            Self::Halide
        } else {
            Self::Other
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
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
pub struct PlotPointSample {
    pub seen: usize,
    pub points: Vec<PlotPoint>,
}

#[derive(Clone, Copy, Debug)]
pub struct ErrorMeasurement<'a> {
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
struct HighErrorSmilesUpdate<'a> {
    abs_error_da: f64,
    abs_ppm: f64,
    pepmass_header: f64,
    smiles: Option<&'a str>,
    calculated_mass: Option<f64>,
    expected_mass: Option<f64>,
    formula: Option<&'a str>,
}

impl PlotPointSample {
    pub fn push(&mut self, point: PlotPoint) {
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
pub struct WarningDetail {
    pub count: usize,
    pub formula: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HighErrorSmilesDetail {
    pub count: usize,
    pub calculated_mass: Option<f64>,
    pub expected_mass: Option<f64>,
    pub formula: Option<String>,
    pub max_abs_error_da: Option<f64>,
    pub max_abs_error_ppm: Option<f64>,
    pub observed_precursor_mz: Option<f64>,
}

#[derive(Clone, Debug)]
pub struct AdductClass {
    pub label: String,
    pub display: String,
    pub family: String,
    pub charge: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct OrderedF64(pub f64);

impl PartialEq for OrderedF64 {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for OrderedF64 {}

impl Ord for OrderedF64 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl PartialOrd for OrderedF64 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug, Default)]
pub struct MedianTracker {
    lower: BinaryHeap<OrderedF64>,
    upper: BinaryHeap<Reverse<OrderedF64>>,
    lower_len: usize,
    upper_len: usize,
}

impl MedianTracker {
    pub fn push(&mut self, value: f64) {
        let should_go_lower =
            self.lower.is_empty() || self.lower.peek().is_none_or(|entry| value <= entry.0);
        if should_go_lower {
            self.lower.push(OrderedF64(value));
            self.lower_len += 1;
        } else {
            self.upper.push(Reverse(OrderedF64(value)));
            self.upper_len += 1;
        }

        if self.lower_len > self.upper_len + 1 {
            if let Some(OrderedF64(value)) = self.lower.pop() {
                self.upper.push(Reverse(OrderedF64(value)));
            }
            self.lower_len -= 1;
            self.upper_len += 1;
        } else if self.upper_len > self.lower_len {
            if let Some(Reverse(OrderedF64(value))) = self.upper.pop() {
                self.lower.push(OrderedF64(value));
            }
            self.upper_len -= 1;
            self.lower_len += 1;
        }
    }

    pub fn merge(&mut self, mut other: Self) {
        let mut values = Vec::with_capacity(other.lower_len + other.upper_len);
        while let Some(OrderedF64(value)) = other.lower.pop() {
            values.push(value);
        }
        while let Some(Reverse(OrderedF64(value))) = other.upper.pop() {
            values.push(value);
        }
        for value in values {
            self.push(value);
        }
    }

    #[must_use]
    pub fn median(&self) -> f64 {
        if self.lower_len == 0 && self.upper_len == 0 {
            0.0
        } else if self.lower_len > self.upper_len {
            self.lower.peek().map_or(0.0, |entry| entry.0)
        } else {
            let lower = self.lower.peek().map_or(0.0, |entry| entry.0);
            let upper = self.upper.peek().map_or(0.0, |entry| entry.0.0);
            lower.midpoint(upper)
        }
    }
}

#[derive(Clone, Debug)]
pub struct PrecursorStats {
    pub spectra: usize,
    pub total_spectra: usize,
    pub skipped_spectra: usize,
    pub spectra_with_reference_mass: usize,
    pub reference_mass_source: String,
    pub unparsed_smiles: usize,
    pub unparsed_smiles_warnings: BTreeMap<String, WarningDetail>,
    pub observed_precursor_min: f64,
    pub observed_precursor_max: f64,
    pub observed_precursor_mean: f64,
    pub observed_precursor_median: f64,
    pub observed_precursor_median_tracker: MedianTracker,
    pub sample_observed_precursor: f64,
    pub abs_error_da_min: f64,
    pub abs_error_da_max: f64,
    pub abs_error_da_mean: f64,
    pub abs_error_da_median: f64,
    pub abs_error_da_median_tracker: MedianTracker,
    pub sample_abs_error_da: f64,
    pub abs_error_da_rms: f64,
    pub abs_error_ppm_min: f64,
    pub abs_error_ppm_max: f64,
    pub abs_error_ppm_mean: f64,
    pub abs_error_ppm_median: f64,
    pub abs_error_ppm_median_tracker: MedianTracker,
    pub sample_abs_error_ppm: f64,
    pub abs_error_ppm_rms: f64,
    pub signed_error_da_mean: f64,
    pub signed_error_da_median: f64,
    pub signed_error_da_median_tracker: MedianTracker,
    pub sample_signed_error_da: f64,
    pub signed_error_ppm_mean: f64,
    pub signed_error_ppm_median: f64,
    pub signed_error_ppm_median_tracker: MedianTracker,
    pub sample_signed_error_ppm: f64,
    pub within_0_1_da: usize,
    pub within_0_5_da: usize,
    pub within_1_da: usize,
    pub within_5_da: usize,
    pub above_5_da: usize,
    pub within_0_5_ppm: usize,
    pub within_1_ppm: usize,
    pub within_5_ppm: usize,
    pub within_10_ppm: usize,
    pub above_10_ppm: usize,
    pub da_error_histogram: HistogramData,
    pub ppm_error_histogram: HistogramData,
    pub absolute_error_da_values: Vec<f64>,
    pub absolute_error_ppm_values: Vec<f64>,
    pub absolute_error_da_sample_seen: usize,
    pub absolute_error_ppm_sample_seen: usize,
    pub plot_points: Vec<PlotPoint>,
    pub plot_point_stream_seen: usize,
    pub unrecognized_adducts: BTreeMap<String, usize>,
    pub high_error_smiles: BTreeMap<String, HighErrorSmilesDetail>,
}

impl Default for PrecursorStats {
    fn default() -> Self {
        Self {
            spectra: 0,
            total_spectra: 0,
            skipped_spectra: 0,
            spectra_with_reference_mass: 0,
            reference_mass_source: "none".to_string(),
            unparsed_smiles: 0,
            unparsed_smiles_warnings: BTreeMap::new(),
            observed_precursor_min: 0.0,
            observed_precursor_max: 0.0,
            observed_precursor_mean: 0.0,
            observed_precursor_median: 0.0,
            observed_precursor_median_tracker: MedianTracker::default(),
            sample_observed_precursor: 0.0,
            abs_error_da_min: 0.0,
            abs_error_da_max: 0.0,
            abs_error_da_mean: 0.0,
            abs_error_da_median: 0.0,
            abs_error_da_median_tracker: MedianTracker::default(),
            sample_abs_error_da: 0.0,
            abs_error_da_rms: 0.0,
            abs_error_ppm_min: 0.0,
            abs_error_ppm_max: 0.0,
            abs_error_ppm_mean: 0.0,
            abs_error_ppm_median: 0.0,
            abs_error_ppm_median_tracker: MedianTracker::default(),
            sample_abs_error_ppm: 0.0,
            abs_error_ppm_rms: 0.0,
            signed_error_da_mean: 0.0,
            signed_error_da_median: 0.0,
            signed_error_da_median_tracker: MedianTracker::default(),
            sample_signed_error_da: 0.0,
            signed_error_ppm_mean: 0.0,
            signed_error_ppm_median: 0.0,
            signed_error_ppm_median_tracker: MedianTracker::default(),
            sample_signed_error_ppm: 0.0,
            within_0_1_da: 0,
            within_0_5_da: 0,
            within_1_da: 0,
            within_5_da: 0,
            above_5_da: 0,
            within_0_5_ppm: 0,
            within_1_ppm: 0,
            within_5_ppm: 0,
            within_10_ppm: 0,
            above_10_ppm: 0,
            da_error_histogram: HistogramData::new(48, 0.0, 0.5),
            ppm_error_histogram: HistogramData::new(48, 0.0, 50.0),
            absolute_error_da_values: Vec::new(),
            absolute_error_ppm_values: Vec::new(),
            absolute_error_da_sample_seen: 0,
            absolute_error_ppm_sample_seen: 0,
            plot_points: Vec::new(),
            plot_point_stream_seen: 0,
            unrecognized_adducts: BTreeMap::new(),
            high_error_smiles: BTreeMap::new(),
        }
    }
}

impl PrecursorStats {
    fn push_sampled_value(value: f64, values: &mut Vec<f64>, seen: &mut usize) {
        *seen = seen.saturating_add(1);
        if values.len() < MAX_ECDF_POINTS {
            values.push(value);
            return;
        }
        let stream_index = u64::try_from(*seen).unwrap_or(u64::MAX);
        let replacement_index = usize::try_from(
            (stream_index
                .wrapping_mul(0x9e37_79b9_7f4a_7c15)
                .wrapping_add(0xbf58_476d_1ce4_e5b9))
                % stream_index,
        )
        .unwrap_or(0);
        if replacement_index < MAX_ECDF_POINTS
            && let Some(existing) = values.get_mut(replacement_index)
        {
            *existing = value;
        }
    }

    fn push_plot_point(&mut self, point: PlotPoint) {
        self.plot_point_stream_seen = self.plot_point_stream_seen.saturating_add(1);
        if self.plot_points.len() < MAX_PLOT_POINTS {
            self.plot_points.push(point);
        } else {
            let stream_index = u64::try_from(self.plot_point_stream_seen).unwrap_or(u64::MAX);
            let replacement_index = usize::try_from(
                (stream_index
                    .wrapping_mul(0x9e37_79b9_7f4a_7c15)
                    .wrapping_add(0xbf58_476d_1ce4_e5b9))
                    % stream_index,
            )
            .unwrap_or(0);
            if replacement_index < MAX_PLOT_POINTS
                && let Some(existing) = self.plot_points.get_mut(replacement_index)
            {
                *existing = point;
            }
        }
    }

    pub fn record_error(&mut self, measurement: ErrorMeasurement<'_>) {
        let mut no_plot_sample = None;
        self.record_error_with_plot_sample(measurement, &mut no_plot_sample);
    }

    pub fn record_error_with_plot_sample(
        &mut self,
        measurement: ErrorMeasurement<'_>,
        plot_sample: &mut Option<&mut PlotPointSample>,
    ) {
        let ErrorMeasurement {
            abs_error_da,
            abs_ppm,
            adduct_family,
            ppm_error,
            signed_error_da,
            pepmass_header,
            ms2_precursor_peak,
            smiles,
            calculated_mass,
            expected_mass,
            formula,
        } = measurement;

        self.da_error_histogram.add_value(abs_error_da);
        self.ppm_error_histogram.add_value(abs_ppm);
        Self::push_sampled_value(
            abs_error_da,
            &mut self.absolute_error_da_values,
            &mut self.absolute_error_da_sample_seen,
        );
        Self::push_sampled_value(
            abs_ppm,
            &mut self.absolute_error_ppm_values,
            &mut self.absolute_error_ppm_sample_seen,
        );
        self.sample_observed_precursor = pepmass_header;
        self.sample_abs_error_da = abs_error_da;
        self.sample_abs_error_ppm = abs_ppm;
        self.sample_signed_error_da = signed_error_da;
        self.sample_signed_error_ppm = ppm_error;
        self.observed_precursor_median_tracker.push(pepmass_header);
        self.abs_error_da_median_tracker.push(abs_error_da);
        self.abs_error_ppm_median_tracker.push(abs_ppm);
        self.signed_error_da_median_tracker.push(signed_error_da);
        self.signed_error_ppm_median_tracker.push(ppm_error);
        self.observed_precursor_median = self.observed_precursor_median_tracker.median();
        self.abs_error_da_median = self.abs_error_da_median_tracker.median();
        self.abs_error_ppm_median = self.abs_error_ppm_median_tracker.median();
        self.signed_error_da_median = self.signed_error_da_median_tracker.median();
        self.signed_error_ppm_median = self.signed_error_ppm_median_tracker.median();

        let abs_error_milli_da = abs_error_da * 1000.0;
        self.update_error_buckets(abs_error_milli_da, abs_ppm);
        self.update_high_error_smiles(HighErrorSmilesUpdate {
            abs_error_da,
            abs_ppm,
            pepmass_header,
            smiles,
            calculated_mass,
            expected_mass,
            formula,
        });

        let point = PlotPoint {
            adduct_family,
            pepmass_header,
            ms2_precursor_peak,
            signed_error_da,
            signed_error_ppm: ppm_error,
            expected_mass,
        };
        if let Some(sample) = plot_sample.as_mut() {
            sample.push(point);
        } else {
            self.push_plot_point(point);
        }
    }

    fn update_error_buckets(&mut self, abs_error_milli_da: f64, abs_ppm: f64) {
        match abs_error_milli_da {
            value if value <= 0.1 => self.within_0_1_da = self.within_0_1_da.saturating_add(1),
            value if value <= 0.5 => self.within_0_5_da = self.within_0_5_da.saturating_add(1),
            value if value <= 1.0 => self.within_1_da = self.within_1_da.saturating_add(1),
            value if value <= 5.0 => self.within_5_da = self.within_5_da.saturating_add(1),
            _ => self.above_5_da = self.above_5_da.saturating_add(1),
        }

        match abs_ppm {
            value if value <= 0.5 => self.within_0_5_ppm = self.within_0_5_ppm.saturating_add(1),
            value if value <= 1.0 => self.within_1_ppm = self.within_1_ppm.saturating_add(1),
            value if value <= 5.0 => self.within_5_ppm = self.within_5_ppm.saturating_add(1),
            value if value <= 10.0 => self.within_10_ppm = self.within_10_ppm.saturating_add(1),
            _ => self.above_10_ppm = self.above_10_ppm.saturating_add(1),
        }
    }

    fn update_high_error_smiles(&mut self, update: HighErrorSmilesUpdate<'_>) {
        let Some(smiles) = update.smiles.filter(|value| !value.trim().is_empty()) else {
            return;
        };
        if update.abs_error_da <= 0.01 {
            return;
        }

        let key = smiles.trim().to_owned();
        self.high_error_smiles
            .entry(key)
            .and_modify(|entry| {
                entry.count = entry.count.saturating_add(1);
                if entry.calculated_mass.is_none() && update.calculated_mass.is_some() {
                    entry.calculated_mass = update.calculated_mass;
                }
                if entry.expected_mass.is_none() && update.expected_mass.is_some() {
                    entry.expected_mass = update.expected_mass;
                }
                if entry.formula.is_none() {
                    entry.formula = update.formula.map(str::to_string);
                }
                if entry.observed_precursor_mz.is_none() {
                    entry.observed_precursor_mz = Some(update.pepmass_header);
                }
                if entry
                    .max_abs_error_da
                    .is_none_or(|existing| update.abs_error_da > existing)
                {
                    entry.max_abs_error_da = Some(update.abs_error_da);
                    entry.max_abs_error_ppm = Some(update.abs_ppm);
                    if update.calculated_mass.is_some() {
                        entry.calculated_mass = update.calculated_mass;
                    }
                    if update.expected_mass.is_some() {
                        entry.expected_mass = update.expected_mass;
                    }
                    entry.observed_precursor_mz = Some(update.pepmass_header);
                }
            })
            .or_insert_with(|| HighErrorSmilesDetail {
                count: 1,
                calculated_mass: update.calculated_mass,
                expected_mass: update.expected_mass,
                formula: update.formula.map(str::to_string),
                max_abs_error_da: Some(update.abs_error_da),
                max_abs_error_ppm: Some(update.abs_ppm),
                observed_precursor_mz: Some(update.pepmass_header),
            });
    }
}

mod merge;

pub use merge::*;
