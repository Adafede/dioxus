#![allow(
    clippy::assigning_clones,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::collapsible_if,
    clippy::derive_partial_eq_without_eq,
    clippy::manual_midpoint,
    clippy::map_unwrap_or,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::or_fun_call,
    clippy::redundant_clone,
    clippy::suboptimal_flops,
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::unnecessary_map_or
)]

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
            ((clamped - self.min) / (self.max - self.min) * (self.bins.len() as f64 - 1.0)).floor()
                as usize
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
    pub pepmass_header: f64,  // PEPMASS from MGF header (metadata block)
    pub ms2_precursor_peak: Option<f64>,  // Actual MS2 precursor peak observed in fragment list (near PEPMASS)
    pub signed_error_da: f64,
    pub signed_error_ppm: f64,
    pub expected_mass: Option<f64>,  // Theoretical precursor mass for error calculation
}

#[derive(Clone, Debug, Default)]
pub struct PlotPointSample {
    pub seen: usize,
    pub points: Vec<PlotPoint>,
}

impl PlotPointSample {
    pub fn push(&mut self, point: PlotPoint) {
        self.seen = self.seen.saturating_add(1);
        if self.points.len() < MAX_PLOT_POINTS {
            self.points.push(point);
        } else {
            let stream_index = self.seen as u64;
            let replacement_index = ((stream_index
                .wrapping_mul(0x9e37_79b9_7f4a_7c15)
                .wrapping_add(0xbf58_476d_1ce4_e5b9))
                % stream_index) as usize;
            if replacement_index < MAX_PLOT_POINTS {
                if let Some(existing) = self.points.get_mut(replacement_index) {
                    *existing = point;
                }
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

#[derive(Clone, Debug, PartialEq)]
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
        let should_go_lower = self.lower.is_empty()
            || value
                <= self
                    .lower
                    .peek()
                    .map(|entry| entry.0)
                    .unwrap_or(f64::NEG_INFINITY);
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

    pub fn median(&self) -> f64 {
        if self.lower_len == 0 && self.upper_len == 0 {
            0.0
        } else if self.lower_len > self.upper_len {
            self.lower.peek().map(|entry| entry.0).unwrap_or(0.0)
        } else {
            let lower = self.lower.peek().map(|entry| entry.0).unwrap_or(0.0);
            let upper = self.upper.peek().map(|entry| entry.0.0).unwrap_or(0.0);
            (lower + upper) / 2.0
        }
    }
}

#[derive(Clone, Debug)]
pub struct PrecursorMetrics {
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

impl Default for PrecursorMetrics {
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

impl PrecursorMetrics {
    fn push_sampled_value(value: f64, values: &mut Vec<f64>, seen: &mut usize) {
        *seen = seen.saturating_add(1);
        if values.len() < MAX_ECDF_POINTS {
            values.push(value);
            return;
        }
        let stream_index = *seen as u64;
        let replacement_index = ((stream_index
            .wrapping_mul(0x9e37_79b9_7f4a_7c15)
            .wrapping_add(0xbf58_476d_1ce4_e5b9))
            % stream_index) as usize;
        if replacement_index < MAX_ECDF_POINTS {
            if let Some(existing) = values.get_mut(replacement_index) {
                *existing = value;
            }
        }
    }

    fn push_plot_point(&mut self, point: PlotPoint) {
        self.plot_point_stream_seen = self.plot_point_stream_seen.saturating_add(1);
        if self.plot_points.len() < MAX_PLOT_POINTS {
            self.plot_points.push(point);
        } else {
            let stream_index = self.plot_point_stream_seen as u64;
            let replacement_index = ((stream_index
                .wrapping_mul(0x9e37_79b9_7f4a_7c15)
                .wrapping_add(0xbf58_476d_1ce4_e5b9))
                % stream_index) as usize;
            if replacement_index < MAX_PLOT_POINTS {
                if let Some(existing) = self.plot_points.get_mut(replacement_index) {
                    *existing = point;
                }
            }
        }
    }

    pub fn record_error(
        &mut self,
        abs_error_da: f64,
        abs_ppm: f64,
        adduct_family: AdductFamily,
        ppm_error: f64,
        signed_error_da: f64,
        pepmass_header: f64,
        ms2_precursor_peak: Option<f64>,
        smiles: Option<&str>,
        calculated_mass: Option<f64>,
        expected_mass: Option<f64>,
        formula: Option<&str>,
    ) {
        let mut no_plot_sample = None;
        self.record_error_with_plot_sample(
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
            &mut no_plot_sample,
        );
    }

    pub fn record_error_with_plot_sample(
        &mut self,
        abs_error_da: f64,
        abs_ppm: f64,
        adduct_family: AdductFamily,
        ppm_error: f64,
        signed_error_da: f64,
        pepmass_header: f64,
        ms2_precursor_peak: Option<f64>,
        smiles: Option<&str>,
        calculated_mass: Option<f64>,
        expected_mass: Option<f64>,
        formula: Option<&str>,
        plot_sample: &mut Option<&mut PlotPointSample>,
    ) {
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
        self.observed_precursor_median_tracker
            .push(pepmass_header);
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
        if abs_error_milli_da <= 0.1 {
            self.within_0_1_da = self.within_0_1_da.saturating_add(1);
        } else if abs_error_milli_da <= 0.5 {
            self.within_0_5_da = self.within_0_5_da.saturating_add(1);
        } else if abs_error_milli_da <= 1.0 {
            self.within_1_da = self.within_1_da.saturating_add(1);
        } else if abs_error_milli_da <= 5.0 {
            self.within_5_da = self.within_5_da.saturating_add(1);
        } else {
            self.above_5_da = self.above_5_da.saturating_add(1);
        }

        if abs_ppm <= 0.5 {
            self.within_0_5_ppm = self.within_0_5_ppm.saturating_add(1);
        } else if abs_ppm <= 1.0 {
            self.within_1_ppm = self.within_1_ppm.saturating_add(1);
        } else if abs_ppm <= 5.0 {
            self.within_5_ppm = self.within_5_ppm.saturating_add(1);
        } else if abs_ppm <= 10.0 {
            self.within_10_ppm = self.within_10_ppm.saturating_add(1);
        } else {
            self.above_10_ppm = self.above_10_ppm.saturating_add(1);
        }

        if abs_error_da > 0.01 {
            if let Some(smiles) = smiles.filter(|value| !value.trim().is_empty()) {
                let trimmed = smiles.trim().to_string();
                self.high_error_smiles
                    .entry(trimmed.clone())
                    .and_modify(|entry| {
                        entry.count = entry.count.saturating_add(1);
                        if entry.calculated_mass.is_none() && calculated_mass.is_some() {
                            entry.calculated_mass = calculated_mass;
                        }
                        if entry.expected_mass.is_none() && expected_mass.is_some() {
                            entry.expected_mass = expected_mass;
                        }
                        if entry.formula.is_none() {
                            entry.formula = formula.map(str::to_string);
                        }
                        if entry.observed_precursor_mz.is_none() {
                            entry.observed_precursor_mz = Some(pepmass_header);
                        }
                        let current_error_da = abs_error_da;
                        let current_error_ppm = abs_ppm;
                        let should_replace = entry
                            .max_abs_error_da
                            .map_or(true, |existing| current_error_da > existing);
                        if should_replace {
                            entry.max_abs_error_da = Some(current_error_da);
                            entry.max_abs_error_ppm = Some(current_error_ppm);
                            if calculated_mass.is_some() {
                                entry.calculated_mass = calculated_mass;
                            }
                            if expected_mass.is_some() {
                                entry.expected_mass = expected_mass;
                            }
                            entry.observed_precursor_mz = Some(pepmass_header);
                        }
                    })
                    .or_insert(HighErrorSmilesDetail {
                        count: 1,
                        calculated_mass,
                        expected_mass,
                        formula: formula.map(str::to_string),
                        max_abs_error_da: Some(abs_error_da),
                        max_abs_error_ppm: Some(abs_ppm),
                        observed_precursor_mz: Some(pepmass_header),
                    });
            }
        }

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
}

pub fn merge_metrics(mut current: PrecursorMetrics, next: PrecursorMetrics) -> PrecursorMetrics {
    let current_spectra = current.spectra as f64;
    let next_spectra = next.spectra as f64;
    let total_spectra = current_spectra + next_spectra;

    current.spectra += next.spectra;
    current.total_spectra += next.total_spectra;
    current.skipped_spectra += next.skipped_spectra;
    current.spectra_with_reference_mass += next.spectra_with_reference_mass;
    current.unparsed_smiles += next.unparsed_smiles;
    for (smiles, detail) in next.unparsed_smiles_warnings {
        current
            .unparsed_smiles_warnings
            .entry(smiles)
            .and_modify(|existing| {
                existing.count = existing.count.saturating_add(detail.count);
                if existing.formula.is_none() {
                    existing.formula = detail.formula.clone();
                }
            })
            .or_insert(detail);
    }
    if current.reference_mass_source == "none" {
        current.reference_mass_source = next.reference_mass_source;
    } else if current.reference_mass_source != next.reference_mass_source
        && !next.reference_mass_source.is_empty()
    {
        current.reference_mass_source = "mixed".to_string();
    }

    current.observed_precursor_min = current
        .observed_precursor_min
        .min(next.observed_precursor_min);
    current.observed_precursor_max = current
        .observed_precursor_max
        .max(next.observed_precursor_max);
    current.observed_precursor_mean = ((current.observed_precursor_mean * current_spectra)
        + (next.observed_precursor_mean * next_spectra))
        / total_spectra;
    current
        .observed_precursor_median_tracker
        .merge(next.observed_precursor_median_tracker);
    current.observed_precursor_median = current.observed_precursor_median_tracker.median();

    current.abs_error_da_min = current.abs_error_da_min.min(next.abs_error_da_min);
    current.abs_error_da_max = current.abs_error_da_max.max(next.abs_error_da_max);
    current.abs_error_da_mean = ((current.abs_error_da_mean * current_spectra)
        + (next.abs_error_da_mean * next_spectra))
        / total_spectra;
    current
        .abs_error_da_median_tracker
        .merge(next.abs_error_da_median_tracker);
    current.abs_error_da_median = current.abs_error_da_median_tracker.median();
    let current_da_rms_sq = current.abs_error_da_rms * current.abs_error_da_rms;
    let next_da_rms_sq = next.abs_error_da_rms * next.abs_error_da_rms;
    current.abs_error_da_rms =
        ((current_da_rms_sq * current_spectra) + (next_da_rms_sq * next_spectra)) / total_spectra;
    current.abs_error_da_rms = current.abs_error_da_rms.sqrt();

    current.abs_error_ppm_min = current.abs_error_ppm_min.min(next.abs_error_ppm_min);
    current.abs_error_ppm_max = current.abs_error_ppm_max.max(next.abs_error_ppm_max);
    current.abs_error_ppm_mean = ((current.abs_error_ppm_mean * current_spectra)
        + (next.abs_error_ppm_mean * next_spectra))
        / total_spectra;
    current
        .abs_error_ppm_median_tracker
        .merge(next.abs_error_ppm_median_tracker);
    current.abs_error_ppm_median = current.abs_error_ppm_median_tracker.median();
    let current_ppm_rms_sq = current.abs_error_ppm_rms * current.abs_error_ppm_rms;
    let next_ppm_rms_sq = next.abs_error_ppm_rms * next.abs_error_ppm_rms;
    current.abs_error_ppm_rms =
        ((current_ppm_rms_sq * current_spectra) + (next_ppm_rms_sq * next_spectra)) / total_spectra;
    current.abs_error_ppm_rms = current.abs_error_ppm_rms.sqrt();

    current.signed_error_da_mean = ((current.signed_error_da_mean * current_spectra)
        + (next.signed_error_da_mean * next_spectra))
        / total_spectra;
    current
        .signed_error_da_median_tracker
        .merge(next.signed_error_da_median_tracker);
    current.signed_error_da_median = current.signed_error_da_median_tracker.median();
    current.signed_error_ppm_mean = ((current.signed_error_ppm_mean * current_spectra)
        + (next.signed_error_ppm_mean * next_spectra))
        / total_spectra;
    current
        .signed_error_ppm_median_tracker
        .merge(next.signed_error_ppm_median_tracker);
    current.signed_error_ppm_median = current.signed_error_ppm_median_tracker.median();

    current.within_0_1_da += next.within_0_1_da;
    current.within_0_5_da += next.within_0_5_da;
    current.within_1_da += next.within_1_da;
    current.within_5_da += next.within_5_da;
    current.within_0_5_ppm += next.within_0_5_ppm;
    current.within_1_ppm += next.within_1_ppm;
    current.within_5_ppm += next.within_5_ppm;
    current.above_5_da += next.above_5_da;
    current.within_10_ppm += next.within_10_ppm;
    current.above_10_ppm += next.above_10_ppm;
    for (idx, count) in next.da_error_histogram.bins.iter().enumerate() {
        if let Some(current_count) = current.da_error_histogram.bins.get_mut(idx) {
            *current_count += count;
        }
    }
    for (idx, count) in next.ppm_error_histogram.bins.iter().enumerate() {
        if let Some(current_count) = current.ppm_error_histogram.bins.get_mut(idx) {
            *current_count += count;
        }
    }

    for value in next.absolute_error_da_values {
        PrecursorMetrics::push_sampled_value(
            value,
            &mut current.absolute_error_da_values,
            &mut current.absolute_error_da_sample_seen,
        );
    }

    for value in next.absolute_error_ppm_values {
        PrecursorMetrics::push_sampled_value(
            value,
            &mut current.absolute_error_ppm_values,
            &mut current.absolute_error_ppm_sample_seen,
        );
    }

    for point in next.plot_points {
        current.push_plot_point(point);
    }

    for (adduct, count) in next.unrecognized_adducts {
        current
            .unrecognized_adducts
            .entry(adduct)
            .and_modify(|existing| *existing += count)
            .or_insert(count);
    }
    for (smiles, detail) in next.high_error_smiles {
        current
            .high_error_smiles
            .entry(smiles)
            .and_modify(|existing| {
                existing.count = existing.count.saturating_add(detail.count);
                if existing.calculated_mass.is_none() && detail.calculated_mass.is_some() {
                    existing.calculated_mass = detail.calculated_mass;
                }
                if existing.expected_mass.is_none() && detail.expected_mass.is_some() {
                    existing.expected_mass = detail.expected_mass;
                }
                if existing.formula.is_none() {
                    existing.formula = detail.formula.clone();
                }
                if existing.observed_precursor_mz.is_none()
                    && detail.observed_precursor_mz.is_some()
                {
                    existing.observed_precursor_mz = detail.observed_precursor_mz;
                }
                if let Some(detail_error_da) = detail.max_abs_error_da {
                    let should_replace = existing
                        .max_abs_error_da
                        .map_or(true, |existing_error| detail_error_da > existing_error);
                    if should_replace {
                        existing.max_abs_error_da = Some(detail_error_da);
                        existing.max_abs_error_ppm = detail.max_abs_error_ppm;
                        if detail.calculated_mass.is_some() {
                            existing.calculated_mass = detail.calculated_mass;
                        }
                        if detail.expected_mass.is_some() {
                            existing.expected_mass = detail.expected_mass;
                        }
                        if detail.observed_precursor_mz.is_some() {
                            existing.observed_precursor_mz = detail.observed_precursor_mz;
                        }
                    }
                }
            })
            .or_insert(detail);
    }

    current
}
