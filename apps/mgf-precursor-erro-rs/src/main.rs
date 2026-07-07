#![allow(clippy::all)]
#![allow(warnings)]

use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap, HashMap, HashSet};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::str::FromStr;
use std::sync::{LazyLock, Mutex};

use dioxus::events::{DragData, FormData, WheelData};
use dioxus::html::HasFileData;
use dioxus::prelude::*;
#[cfg(target_arch = "wasm32")]
use gloo_timers::future::TimeoutFuture;
#[cfg(target_arch = "wasm32")]
use js_sys::{Array, Uint8Array};
use mascot_rs::prelude::*;
use molecular_formulas_010::molecular_formula::MolecularFormula;
use prismatica::crameri::BATLOW;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;
#[cfg(target_arch = "wasm32")]
use web_sys::{Blob, console};

#[cfg(any(target_arch = "wasm32", test))]
const CHUNK_SIZE: usize = 1 << 20;
#[cfg(any(target_arch = "wasm32", test))]
const PROGRESS_INTERVAL: usize = 1 << 20;
const MAX_PLOT_POINTS: usize = 10_000;
const PROTON_MASS: f64 = 1.007_276_466_621;
const HYDROGEN_MASS: f64 = PROTON_MASS + ELECTRON_MASS;
const ELECTRON_MASS: f64 = 0.000_548_579_909_065;
const SODIUM_MASS: f64 = 22.989_769_67;
const POTASSIUM_MASS: f64 = 38.963_707;
const AMMONIUM_MASS: f64 = 18.033_823;

static ADDUCT_SPEC_CACHE: LazyLock<Mutex<HashMap<String, Option<(f64, f64)>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[cfg(target_arch = "wasm32")]
type ScanError = JsValue;
#[cfg(not(target_arch = "wasm32"))]
type ScanError = String;

fn main() {
    dioxus::launch(app);
}

#[derive(Clone, Debug, PartialEq)]
struct HistogramData {
    bins: Vec<usize>,
    min: f64,
    max: f64,
}

#[derive(Clone, Debug, PartialEq)]
struct PlotPoint {
    adduct_type: String,
    adduct_family: String,
    observed_precursor_mz: f64,
    signed_error_da: f64,
    signed_error_ppm: f64,
}

#[derive(Clone, Debug, Default)]
struct PlotPointSample {
    seen: usize,
    points: Vec<PlotPoint>,
}

impl PlotPointSample {
    fn push(&mut self, point: PlotPoint) {
        self.seen = self.seen.saturating_add(1);
        if self.points.len() < MAX_PLOT_POINTS {
            self.points.push(point);
        } else {
            let stream_index = self.seen as u64;
            let replacement_index = ((stream_index
                .wrapping_mul(0x9e3779b97f4a7c15)
                .wrapping_add(0xbf58476d1ce4e5b9))
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
struct ScatterPlotData {
    legend_items: Vec<(String, String)>,
    x_min: f64,
    x_max: f64,
    y_limit: f64,
    series: Vec<(String, Vec<(f64, f64)>)>,
}

#[derive(Clone, Debug, Default)]
struct BlockParseState {
    observed_precursor_raw: Option<String>,
    observed_precursor: Option<f64>,
    reference_mass: Option<f64>,
    reference_mass_source: Option<String>,
    charge: Option<String>,
    adduct: Option<String>,
    ion_mode: Option<String>,
    smiles: Option<String>,
    formula: Option<String>,
    feature_id: Option<String>,
    scans: Option<String>,
}

impl BlockParseState {
    fn consume_line(&mut self, line: &str) {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed == "BEGIN IONS" || trimmed == "END IONS" {
            return;
        }

        if let Some(stripped) = trimmed.strip_prefix("PRECURSOR_MZ=") {
            self.observed_precursor_raw = Some(stripped.to_string());
            if let Ok(value) = stripped.parse::<f64>() {
                self.observed_precursor = Some(value);
            }
            return;
        }

        if let Some(stripped) = trimmed.strip_prefix("PEPMASS=") {
            self.observed_precursor_raw = Some(stripped.to_string());
            if let Ok(value) = stripped.parse::<f64>() {
                self.observed_precursor = Some(value);
            }
            return;
        }

        if let Some(stripped) = trimmed.strip_prefix("EXACTMASS=") {
            if let Ok(value) = stripped.parse::<f64>() {
                self.reference_mass = Some(value);
                self.reference_mass_source = Some("EXACTMASS".to_string());
            }
            return;
        }

        if let Some(stripped) = trimmed.strip_prefix("MOLECULEMASS=") {
            if let Ok(value) = stripped.parse::<f64>() {
                if self.reference_mass.is_none() {
                    self.reference_mass = Some(value);
                    self.reference_mass_source = Some("MOLECULEMASS".to_string());
                }
            }
            return;
        }

        if let Some(stripped) = trimmed.strip_prefix("CHARGE=") {
            self.charge = Some(stripped.to_string());
            return;
        }

        if let Some(stripped) = trimmed.strip_prefix("SMILES=") {
            self.smiles = Some(stripped.trim().to_string());
            return;
        }

        if let Some(stripped) = trimmed.strip_prefix("FORMULA=") {
            self.formula = Some(stripped.trim().to_string());
            return;
        }

        if let Some(stripped) = trimmed.strip_prefix("ADDUCT=") {
            self.adduct = Some(stripped.to_string());
            return;
        }

        if let Some(stripped) = trimmed.strip_prefix("IONMODE=") {
            self.ion_mode = Some(stripped.to_string());
            return;
        }

        if let Some(stripped) = trimmed.strip_prefix("FEATURE_ID=") {
            self.feature_id = Some(stripped.to_string());
            return;
        }

        if let Some(stripped) = trimmed.strip_prefix("SCANS=") {
            self.scans = Some(stripped.to_string());
            return;
        }

        if let Some(stripped) = trimmed.strip_prefix("EXTRACTSCAN=") {
            if self.feature_id.is_none() {
                self.feature_id = Some(stripped.to_string());
            }
            if self.scans.is_none() {
                self.scans = Some(stripped.to_string());
            }
        }
    }

    fn consume_block_lines(&mut self, block_lines: &[String]) {
        for line in block_lines {
            self.consume_line(line);
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct WarningDetail {
    count: usize,
    formula: Option<String>,
}

#[derive(Clone, Copy, Debug)]
struct OrderedF64(f64);

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
struct MedianTracker {
    lower: BinaryHeap<OrderedF64>,
    upper: BinaryHeap<Reverse<OrderedF64>>,
    lower_len: usize,
    upper_len: usize,
}

impl MedianTracker {
    fn push(&mut self, value: f64) {
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

    fn merge(&mut self, mut other: Self) {
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

    fn median(&self) -> f64 {
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

#[derive(Clone, Debug, PartialEq)]
struct HighErrorSmilesDetail {
    count: usize,
    calculated_mass: Option<f64>,
    expected_mass: Option<f64>,
    formula: Option<String>,
    max_abs_error_da: Option<f64>,
    max_abs_error_ppm: Option<f64>,
    observed_precursor_mz: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
struct AdductClass {
    label: String,
    display: String,
    family: String,
    charge: i32,
}

#[derive(Clone, Debug)]
struct PrecursorMetrics {
    spectra: usize,
    total_spectra: usize,
    skipped_spectra: usize,
    spectra_with_reference_mass: usize,
    reference_mass_source: String,
    unparsed_smiles: usize,
    unparsed_smiles_warnings: BTreeMap<String, WarningDetail>,
    observed_precursor_min: f64,
    observed_precursor_max: f64,
    observed_precursor_mean: f64,
    observed_precursor_median: f64,
    observed_precursor_median_tracker: MedianTracker,
    sample_observed_precursor: f64,
    abs_error_da_min: f64,
    abs_error_da_max: f64,
    abs_error_da_mean: f64,
    abs_error_da_median: f64,
    abs_error_da_median_tracker: MedianTracker,
    sample_abs_error_da: f64,
    abs_error_da_rms: f64,
    abs_error_ppm_min: f64,
    abs_error_ppm_max: f64,
    abs_error_ppm_mean: f64,
    abs_error_ppm_median: f64,
    abs_error_ppm_median_tracker: MedianTracker,
    sample_abs_error_ppm: f64,
    abs_error_ppm_rms: f64,
    signed_error_da_mean: f64,
    signed_error_da_median: f64,
    signed_error_da_median_tracker: MedianTracker,
    sample_signed_error_da: f64,
    signed_error_ppm_mean: f64,
    signed_error_ppm_median: f64,
    signed_error_ppm_median_tracker: MedianTracker,
    sample_signed_error_ppm: f64,
    within_0_1_da: usize,
    within_0_5_da: usize,
    within_1_da: usize,
    within_5_da: usize,
    above_5_da: usize,
    within_0_5_ppm: usize,
    within_1_ppm: usize,
    within_5_ppm: usize,
    within_10_ppm: usize,
    above_10_ppm: usize,
    da_error_histogram: HistogramData,
    ppm_error_histogram: HistogramData,
    absolute_error_da_values: Vec<f64>,
    absolute_error_ppm_values: Vec<f64>,
    plot_points: Vec<PlotPoint>,
    plot_point_stream_seen: usize,
    unrecognized_adducts: BTreeMap<String, usize>,
    high_error_smiles: BTreeMap<String, HighErrorSmilesDetail>,
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
            plot_points: Vec::new(),
            plot_point_stream_seen: 0,
            unrecognized_adducts: BTreeMap::new(),
            high_error_smiles: BTreeMap::new(),
        }
    }
}

impl HistogramData {
    fn new(bin_count: usize, min: f64, max: f64) -> Self {
        Self {
            bins: vec![0; bin_count],
            min,
            max,
        }
    }

    fn add_value(&mut self, value: f64) {
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

impl PrecursorMetrics {
    fn record_error(
        &mut self,
        abs_error_da: f64,
        abs_ppm: f64,
        adduct_type: &str,
        ppm_error: f64,
        signed_error_da: f64,
        observed_precursor_mz: f64,
        smiles: Option<&str>,
        calculated_mass: Option<f64>,
        expected_mass: Option<f64>,
        formula: Option<&str>,
    ) {
        let mut no_plot_sample = None;
        self.record_error_with_plot_sample(
            abs_error_da,
            abs_ppm,
            adduct_type,
            ppm_error,
            signed_error_da,
            observed_precursor_mz,
            smiles,
            calculated_mass,
            expected_mass,
            formula,
            &mut no_plot_sample,
        );
    }

    fn record_error_with_plot_sample(
        &mut self,
        abs_error_da: f64,
        abs_ppm: f64,
        adduct_type: &str,
        ppm_error: f64,
        signed_error_da: f64,
        observed_precursor_mz: f64,
        smiles: Option<&str>,
        calculated_mass: Option<f64>,
        expected_mass: Option<f64>,
        formula: Option<&str>,
        plot_sample: &mut Option<&mut PlotPointSample>,
    ) {
        self.da_error_histogram.add_value(abs_error_da);
        self.ppm_error_histogram.add_value(abs_ppm);
        self.absolute_error_da_values.push(abs_error_da);
        self.absolute_error_ppm_values.push(abs_ppm);
        self.sample_observed_precursor = observed_precursor_mz;
        self.sample_abs_error_da = abs_error_da;
        self.sample_abs_error_ppm = abs_ppm;
        self.sample_signed_error_da = signed_error_da;
        self.sample_signed_error_ppm = ppm_error;
        self.observed_precursor_median_tracker
            .push(observed_precursor_mz);
        self.abs_error_da_median_tracker.push(abs_error_da);
        self.abs_error_ppm_median_tracker.push(abs_ppm);
        self.signed_error_da_median_tracker.push(signed_error_da);
        self.signed_error_ppm_median_tracker.push(ppm_error);
        self.observed_precursor_median = self.observed_precursor_median_tracker.median();
        self.abs_error_da_median = self.abs_error_da_median_tracker.median();
        self.abs_error_ppm_median = self.abs_error_ppm_median_tracker.median();
        self.signed_error_da_median = self.signed_error_da_median_tracker.median();
        self.signed_error_ppm_median = self.signed_error_ppm_median_tracker.median();

        let abs_error_mda = abs_error_da * 1000.0;
        if abs_error_mda <= 0.1 {
            self.within_0_1_da = self.within_0_1_da.saturating_add(1);
        } else if abs_error_mda <= 0.5 {
            self.within_0_5_da = self.within_0_5_da.saturating_add(1);
        } else if abs_error_mda <= 1.0 {
            self.within_1_da = self.within_1_da.saturating_add(1);
        } else if abs_error_mda <= 5.0 {
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
                            entry.observed_precursor_mz = Some(observed_precursor_mz);
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
                            entry.observed_precursor_mz = Some(observed_precursor_mz);
                        }
                    })
                    .or_insert(HighErrorSmilesDetail {
                        count: 1,
                        calculated_mass,
                        expected_mass,
                        formula: formula.map(str::to_string),
                        max_abs_error_da: Some(abs_error_da),
                        max_abs_error_ppm: Some(abs_ppm),
                        observed_precursor_mz: Some(observed_precursor_mz),
                    });
            }
        }

        self.plot_point_stream_seen = self.plot_point_stream_seen.saturating_add(1);
        let point = PlotPoint {
            adduct_type: adduct_type.to_string(),
            adduct_family: adduct_class(adduct_type)
                .map(|adduct| adduct.family)
                .unwrap_or_else(|| "Other".to_string()),
            observed_precursor_mz,
            signed_error_da,
            signed_error_ppm: ppm_error,
        };
        if let Some(sample) = plot_sample.as_mut() {
            sample.push(point.clone());
        } else {
            if self.plot_points.len() < MAX_PLOT_POINTS {
                self.plot_points.push(point.clone());
            } else {
                let stream_index = self.plot_point_stream_seen as u64;
                let replacement_index = ((stream_index
                    .wrapping_mul(0x9e3779b97f4a7c15)
                    .wrapping_add(0xbf58476d1ce4e5b9))
                    % stream_index) as usize;
                if replacement_index < MAX_PLOT_POINTS {
                    if let Some(existing) = self.plot_points.get_mut(replacement_index) {
                        *existing = point;
                    }
                }
            }
        }
    }
}

fn smiles_is_supported(smiles: &str) -> bool {
    !smiles.trim().is_empty()
}

fn exact_mass_from_smiles(smiles: &str) -> Option<f64> {
    let mut cache = HashMap::new();
    let mut logged_failures = HashSet::new();
    exact_mass_from_smiles_cached(smiles, &mut cache, &mut logged_failures)
}

fn exact_mass_from_smiles_cached(
    smiles: &str,
    cache: &mut HashMap<String, Option<f64>>,
    logged_failures: &mut HashSet<String>,
) -> Option<f64> {
    let trimmed = smiles.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(mass) = cache.get(trimmed) {
        return *mass;
    }

    let mass = exact_mass_from_smiles_uncached(trimmed, logged_failures);
    cache.insert(trimmed.to_string(), mass);
    mass
}

fn exact_mass_from_smiles_uncached(
    smiles: &str,
    logged_failures: &mut HashSet<String>,
) -> Option<f64> {
    if !smiles_is_supported(smiles) {
        let warning_key = format!("unsupported-smiles:{smiles}");
        if logged_failures.insert(warning_key) {
            #[cfg(target_arch = "wasm32")]
            console::warn_1(
                &format!("Skipping unsupported SMILES for mass parsing: {smiles}").into(),
            );
        }
        return None;
    }

    let parsed = catch_unwind(AssertUnwindSafe(|| {
        let parsed = Smiles::from_str(smiles).ok()?;
        let formula: ChemicalFormula<u32, i32> = ChemicalFormula::from(&parsed);
        Some(formula.isotopologue_mass())
    }));

    match parsed {
        Ok(mass) => {
            if mass.is_none() {
                let warning_key = format!("parse-failed-smiles:{smiles}");
                if logged_failures.insert(warning_key) {
                    #[cfg(target_arch = "wasm32")]
                    console::warn_1(&format!("SMILES parse failed for: {smiles}").into());
                }
            }
            mass
        }
        Err(panic) => {
            let warning_key = format!("panic-smiles:{smiles}");
            if logged_failures.insert(warning_key) {
                #[cfg(target_arch = "wasm32")]
                {
                    let panic_message = panic
                        .downcast_ref::<&str>()
                        .copied()
                        .or_else(|| panic.downcast_ref::<String>().map(String::as_str))
                        .unwrap_or("unknown panic");
                    console::warn_1(
                        &format!("SMILES parser panicked for: {smiles} ({panic_message})").into(),
                    );
                }
            }
            None
        }
    }
}

fn exact_mass_from_formula(formula: &str) -> Option<f64> {
    let mut cache = HashMap::new();
    let mut logged_failures = HashSet::new();
    exact_mass_from_formula_cached(formula, &mut cache, &mut logged_failures)
}

fn exact_mass_from_formula_cached(
    formula: &str,
    cache: &mut HashMap<String, Option<f64>>,
    logged_failures: &mut HashSet<String>,
) -> Option<f64> {
    let trimmed = formula.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(mass) = cache.get(trimmed) {
        return *mass;
    }

    let parsed: Result<ChemicalFormula<u32, i32>, _> = ChemicalFormula::from_str(trimmed);
    let mass = match parsed {
        Ok(parsed_formula) => Some(parsed_formula.isotopologue_mass()),
        Err(_) => {
            let warning_key = format!("formula-parse-failed:{trimmed}");
            if logged_failures.insert(warning_key) {
                #[cfg(target_arch = "wasm32")]
                console::warn_1(&format!("Formula parse failed for: {trimmed}").into());
            }
            None
        }
    };
    cache.insert(trimmed.to_string(), mass);
    mass
}

fn hill_order(left: &str, right: &str) -> std::cmp::Ordering {
    match (left, right) {
        ("C", "C") => std::cmp::Ordering::Equal,
        ("C", _) => std::cmp::Ordering::Less,
        (_, "C") => std::cmp::Ordering::Greater,
        ("H", "H") => std::cmp::Ordering::Equal,
        ("H", _) => std::cmp::Ordering::Less,
        (_, "H") => std::cmp::Ordering::Greater,
        (left, right) => left.cmp(right),
    }
}

fn expected_precursor_mz(
    neutral_mass: f64,
    adduct: Option<&str>,
    charge: Option<&str>,
    ion_mode: Option<&str>,
) -> Option<f64> {
    let normalized_adduct = adduct.unwrap_or("").trim();
    let normalized_ion_mode = ion_mode.unwrap_or("").trim().to_ascii_lowercase();
    let charge_sign =
        parse_charge_sign(charge, ion_mode).or_else(|| parse_adduct_charge_sign(adduct));

    let charge_value = parse_charge_value(charge, adduct).unwrap_or_else(|| {
        if charge_sign == Some(true) || normalized_ion_mode == "negative" {
            1.0
        } else if charge_sign == Some(false) || normalized_ion_mode == "positive" {
            1.0
        } else {
            1.0
        }
    });

    let (multiplier, shift, electron_adjustment) = if normalized_adduct.is_empty() {
        (
            1.0,
            if charge_sign == Some(true) || normalized_ion_mode == "negative" {
                -PROTON_MASS
            } else if charge_sign == Some(false) || normalized_ion_mode == "positive" {
                PROTON_MASS
            } else {
                0.0
            },
            0.0,
        )
    } else {
        let (multiplier, shift) =
            parse_adduct_mass_spec_cached(normalized_adduct).unwrap_or_else(|| {
                (
                    1.0,
                    if charge_sign == Some(true) || normalized_ion_mode == "negative" {
                        -PROTON_MASS
                    } else if charge_sign == Some(false) || normalized_ion_mode == "positive" {
                        PROTON_MASS
                    } else {
                        0.0
                    },
                )
            });
        let uses_mg_charge_correction =
            normalized_adduct.to_ascii_uppercase().contains("MG") && charge_sign == Some(false);
        let electron_adjustment = match charge_sign {
            Some(false) if uses_mg_charge_correction => {
                -ELECTRON_MASS * (charge_value - 1.0).max(0.0)
            }
            Some(false) => -ELECTRON_MASS * charge_value,
            Some(true) => ELECTRON_MASS * charge_value,
            None => 0.0,
        };
        (multiplier, shift, electron_adjustment)
    };

    Some((neutral_mass * multiplier + shift + electron_adjustment) / charge_value.max(1.0))
}

fn parse_adduct_charge_sign(adduct: Option<&str>) -> Option<bool> {
    let adduct = adduct?.trim();
    let cleaned = adduct.replace(' ', "");
    let suffix = cleaned.split(']').nth(1).unwrap_or("").trim();
    if suffix.contains('-') {
        Some(true)
    } else if suffix.contains('+') {
        Some(false)
    } else {
        None
    }
}

fn parse_charge_sign(charge: Option<&str>, ion_mode: Option<&str>) -> Option<bool> {
    let charge_text = charge.unwrap_or("").trim();
    if charge_text.is_empty() {
        return match ion_mode.unwrap_or("").trim().to_ascii_lowercase().as_str() {
            "negative" => Some(true),
            "positive" => Some(false),
            _ => None,
        };
    }

    let normalized = charge_text.replace(' ', "");
    let first_char = normalized.chars().next();
    let last_char = normalized.chars().last();

    if first_char == Some('-') || last_char == Some('-') {
        Some(true)
    } else if first_char == Some('+') || last_char == Some('+') {
        Some(false)
    } else {
        None
    }
}

fn apply_adduct_token(
    current: &str,
    sign: f64,
    uses_double_mg_mass: bool,
    multiplier: &mut f64,
    shift: &mut f64,
    saw_unsupported_token: &mut bool,
) {
    if let Some(token_mass) =
        parse_adduct_term_mass_with_context(current, sign, uses_double_mg_mass)
    {
        *shift += sign * token_mass;
    } else if let Some(token_multiplier) = parse_multiplicity_token(current) {
        *multiplier = token_multiplier;
    } else if current.eq_ignore_ascii_case("H") {
        *shift += sign * HYDROGEN_MASS;
    } else if !current.is_empty() {
        *saw_unsupported_token = true;
    }
}

fn parse_adduct_mass_spec(adduct: &str) -> Option<(f64, f64)> {
    let normalized = adduct.trim().replace(' ', "").to_ascii_uppercase();
    let body = if let Some(index) = normalized.find(']') {
        normalized[1..index].to_string()
    } else {
        normalized.clone()
    };
    let charge_sign = parse_adduct_charge_sign(Some(adduct));
    let uses_double_mg_mass = charge_sign == Some(false);
    let mut multiplier = 1.0f64;
    let mut shift = 0.0f64;
    let mut current = String::new();
    let mut sign = 1.0f64;
    let mut saw_unsupported_token = false;

    for ch in body.chars() {
        match ch {
            '+' => {
                apply_adduct_token(
                    &current,
                    sign,
                    uses_double_mg_mass,
                    &mut multiplier,
                    &mut shift,
                    &mut saw_unsupported_token,
                );
                current.clear();
                sign = 1.0;
            }
            '-' => {
                apply_adduct_token(
                    &current,
                    sign,
                    uses_double_mg_mass,
                    &mut multiplier,
                    &mut shift,
                    &mut saw_unsupported_token,
                );
                current.clear();
                sign = -1.0;
            }
            _ => current.push(ch),
        }
    }

    apply_adduct_token(
        &current,
        sign,
        uses_double_mg_mass,
        &mut multiplier,
        &mut shift,
        &mut saw_unsupported_token,
    );

    if saw_unsupported_token {
        None
    } else if shift == 0.0 && multiplier == 1.0 && body == "M" {
        Some((1.0, 0.0))
    } else if body.is_empty() || body == "M" {
        None
    } else {
        Some((multiplier, shift))
    }
}

fn parse_multiplicity_token(token: &str) -> Option<f64> {
    let trimmed = token.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.eq_ignore_ascii_case("M") {
        return Some(1.0);
    }

    let digits_end = trimmed
        .chars()
        .position(|ch| !ch.is_ascii_digit())
        .unwrap_or(trimmed.len());
    if digits_end == 0 {
        return None;
    }

    let multiplier_str = &trimmed[..digits_end];
    let remainder = &trimmed[digits_end..];
    if remainder.eq_ignore_ascii_case("M") {
        multiplier_str.parse::<f64>().ok()
    } else {
        None
    }
}

fn parse_adduct_mass_spec_cached(adduct: &str) -> Option<(f64, f64)> {
    let normalized = adduct.trim().replace(' ', "");
    if normalized.is_empty() {
        return None;
    }

    let mut cache = ADDUCT_SPEC_CACHE.lock().unwrap();
    if let Some(mass_spec) = cache.get(&normalized) {
        return *mass_spec;
    }

    let mass_spec = parse_adduct_mass_spec(&normalized);
    cache.insert(normalized, mass_spec);
    mass_spec
}

fn parse_adduct_shift(adduct: &str) -> Option<f64> {
    parse_adduct_mass_spec(adduct).map(|(_, shift)| shift)
}

fn parse_adduct_term_mass(token: &str) -> Option<f64> {
    parse_adduct_term_mass_with_context(token, 1.0, false)
}

fn parse_adduct_term_mass_with_context(
    token: &str,
    sign: f64,
    uses_double_mg_mass: bool,
) -> Option<f64> {
    let trimmed = token.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("M") {
        return None;
    }

    let digits_end = trimmed
        .chars()
        .position(|ch| !ch.is_ascii_digit())
        .unwrap_or(trimmed.len());
    let multiplier_str = &trimmed[..digits_end];
    let multiplier = multiplier_str.parse::<f64>().unwrap_or(1.0);
    let formula = trimmed[digits_end..].trim().to_ascii_uppercase();
    let formula = match formula.as_str() {
        "FA" | "FORMATE" | "HCOO" => "CHO2",
        "HCOONA" | "NACHO2" | "NAHCOO" | "NAHCO2" | "CHNAO2" => "CHNaO2",
        "HCOOH" | "FORMICACID" | "HFA" => "CH2O2",
        "MEOH" | "CH3OH" => "CH4O",
        "H2O" => "H2O",
        "NH3" => "NH3",
        "CO" => "CO",
        "CO2" => "CO2",
        "O" => "O",
        "C2H4" => return Some(28.031_300_128 * multiplier),
        "CHNAO2" | "HCOONA" => {
            return Some(exact_mass_from_formula("CHNaO2").unwrap_or(67.987_423_942) * multiplier);
        }
        "H" => return Some(HYDROGEN_MASS * multiplier),
        "NA" => return Some(SODIUM_MASS * multiplier),
        "K" => return Some(POTASSIUM_MASS * multiplier),
        "MG" => {
            let base = 23.985_041_7 * multiplier;
            if sign > 0.0 && uses_double_mg_mass {
                return Some(base + 23.985_041_7 * multiplier);
            }
            return Some(base);
        }
        "CA" => return Some(39.962_590_98 * multiplier),
        "FE" => return Some(55.934_937_5 * multiplier),
        "CL" => return Some(34.968_852_68 * multiplier),
        "BR" => return Some(78.918_337_1 * multiplier),
        "NH4" => return Some(AMMONIUM_MASS * multiplier),
        "OH" => return Some(17.002_739_65 * multiplier),
        _ => return None,
    };
    let formula: ChemicalFormula<u32, i32> = ChemicalFormula::from_str(formula).ok()?;
    Some(formula.isotopologue_mass() * multiplier)
}

fn parse_charge_value(charge: Option<&str>, adduct: Option<&str>) -> Option<f64> {
    if let Some(value) = charge {
        let cleaned = value.trim();
        let digits = cleaned
            .chars()
            .filter(|ch| ch.is_ascii_digit())
            .collect::<String>();
        if let Ok(parsed) = digits.parse::<f64>() {
            return Some(parsed.max(1.0));
        }
    }

    if let Some(adduct) = adduct {
        let cleaned = adduct.trim();
        let suffix = cleaned.split(']').nth(1).unwrap_or("").trim();
        let digits = suffix
            .chars()
            .filter(|ch| ch.is_ascii_digit())
            .collect::<String>();
        if let Ok(parsed) = digits.parse::<f64>() {
            return Some(parsed.max(1.0));
        }
    }

    None
}

fn decimal_precision(value: &str) -> usize {
    let trimmed = value.trim();
    let Some((_, fractional)) = trimmed.split_once('.') else {
        return 0;
    };
    fractional
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .count()
}

fn round_to_precision(value: f64, precision: usize) -> f64 {
    if precision == 0 {
        return value.round();
    }

    let factor = 10_f64.powi(precision as i32);
    (value * factor).round() / factor
}

#[cfg(target_arch = "wasm32")]
fn format_progress_message(processed: u64, total: u64) -> String {
    let safe_total = total.max(1);
    let displayed_processed = processed.min(safe_total);
    let percent = (displayed_processed * 100 / safe_total).min(100);
    format!("Scanning {displayed_processed}/{safe_total} bytes ({percent}%)...")
}

#[cfg(target_arch = "wasm32")]
const EXAMPLE_MGF_URL: &str =
    "https://raw.githubusercontent.com/zamboni-lab/MultiMS2/main/data/multims2_spectra.mgf";

#[cfg(target_arch = "wasm32")]
async fn fetch_remote_blob(url: &str) -> Result<Blob, String> {
    let window = web_sys::window().ok_or_else(|| "Browser window unavailable.".to_string())?;
    let response_value = JsFuture::from(window.fetch_with_str(url))
        .await
        .map_err(|error| format!("Unable to fetch the example MGF: {error:?}"))?;
    let response: web_sys::Response = response_value
        .dyn_into()
        .map_err(|error| format!("Expected a fetch response: {error:?}"))?;
    if !response.ok() {
        return Err(format!(
            "The example MGF could not be loaded (HTTP {}).",
            response.status()
        ));
    }

    let text_value = JsFuture::from(
        response
            .text()
            .map_err(|error| format!("Unable to read the example MGF response body: {error:?}"))?,
    )
    .await
    .map_err(|error| format!("Unable to read the example MGF text: {error:?}"))?;
    let text = js_sys::JsString::from(text_value)
        .as_string()
        .ok_or_else(|| "The example MGF response was not valid text.".to_string())?;

    let array = Array::new();
    array.push(&JsValue::from(text));
    Blob::new_with_str_sequence(&array)
        .map_err(|error| format!("Unable to create a blob from the example MGF: {error:?}"))
}

#[cfg(target_arch = "wasm32")]
fn start_analysis(
    blob: Blob,
    status: Signal<String>,
    metrics: Signal<Option<PrecursorMetrics>>,
    busy: Signal<bool>,
) {
    let mut status_for_progress = status;
    let mut metrics_for_results = metrics;
    let mut busy_for_results = busy;

    spawn(async move {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let total_bytes = blob.size() as u64;
        status_for_progress.set(format!("Scanning {total_bytes} bytes..."));
        let result = match scan_blob_with_progress(&blob, move |processed, total| {
            status_for_progress.set(format_progress_message(processed, total));
        })
        .await
        {
            Ok(metrics) => metrics,
            Err(error) => {
                status_for_progress.set(format!("Error reading file: {error:?}"));
                PrecursorMetrics::default()
            }
        };
        metrics_for_results.set(Some(result));
        busy_for_results.set(false);
    });
}

#[cfg(target_arch = "wasm32")]
fn load_example_mgf(
    status: Signal<String>,
    metrics: Signal<Option<PrecursorMetrics>>,
    busy: Signal<bool>,
    file_name: Signal<String>,
) {
    let mut status_for_progress = status;
    let mut metrics_for_results = metrics;
    let mut busy_for_results = busy;
    let mut file_name_for_results = file_name;

    spawn(async move {
        status_for_progress.set("Loading example MGF...".to_string());
        busy_for_results.set(true);
        metrics_for_results.set(None);

        match fetch_remote_blob(EXAMPLE_MGF_URL).await {
            Ok(blob) => {
                file_name_for_results.set("multims2_spectra.mgf".to_string());
                start_analysis(
                    blob,
                    status_for_progress,
                    metrics_for_results,
                    busy_for_results,
                );
            }
            Err(error) => {
                status_for_progress.set(error);
                busy_for_results.set(false);
            }
        }
    });
}

#[component]
fn app() -> Element {
    let mut file_name = use_signal(String::new);
    let mut metrics = use_signal(|| None::<PrecursorMetrics>);
    let mut status = use_signal(|| "Drop an MGF file to begin.".to_string());
    let mut busy = use_signal(|| false);
    let mut drag_active = use_signal(|| false);

    let on_file_change = move |evt: Event<FormData>| {
        let Some(file) = evt.data().files().into_iter().next() else {
            status.set("No file selected.".to_string());
            return;
        };

        #[cfg(target_arch = "wasm32")]
        let Some(web_file) = file.inner().downcast_ref::<web_sys::File>() else {
            status.set("This file type is not supported in the browser.".to_string());
            return;
        };

        #[cfg(target_arch = "wasm32")]
        let Ok(blob) = web_file.clone().dyn_into::<Blob>() else {
            status.set("Unable to read the selected file as a blob.".to_string());
            return;
        };

        file_name.set(file.name());
        busy.set(true);
        drag_active.set(false);
        status.set("Reading MGF...".to_string());
        metrics.set(None);

        #[cfg(target_arch = "wasm32")]
        start_analysis(blob, status, metrics, busy);

        #[cfg(not(target_arch = "wasm32"))]
        {
            status.set("This app needs to run in a browser.".to_string());
            busy.set(false);
        }
    };

    let on_drag_enter = move |evt: Event<DragData>| {
        evt.prevent_default();
        drag_active.set(true);
    };

    let on_drag_over = move |evt: Event<DragData>| {
        evt.prevent_default();
        drag_active.set(true);
    };

    let on_drag_leave = move |evt: Event<DragData>| {
        evt.prevent_default();
        drag_active.set(false);
    };

    let on_drop = move |evt: Event<DragData>| {
        evt.prevent_default();
        drag_active.set(false);
        let Some(file) = evt.data().files().into_iter().next() else {
            status.set("No file selected.".to_string());
            return;
        };

        #[cfg(target_arch = "wasm32")]
        let Some(web_file) = file.inner().downcast_ref::<web_sys::File>() else {
            status.set("This file type is not supported in the browser.".to_string());
            return;
        };

        #[cfg(target_arch = "wasm32")]
        let Ok(blob) = web_file.clone().dyn_into::<Blob>() else {
            status.set("Unable to read the selected file as a blob.".to_string());
            return;
        };

        file_name.set(file.name());
        busy.set(true);
        status.set("Reading MGF...".to_string());
        metrics.set(None);

        #[cfg(target_arch = "wasm32")]
        start_analysis(blob, status, metrics, busy);

        #[cfg(not(target_arch = "wasm32"))]
        {
            status.set("This app needs to run in a browser.".to_string());
            busy.set(false);
        }
    };

    rsx! {
        div {
            style: "min-height: 100vh; padding: 2rem 1rem 3rem; background: linear-gradient(135deg, #f8fafc 0%, #eef2ff 100%); color: #0f172a;",
            div {
                style: "max-width: 960px; margin: 0 auto;",
                div {
                    style: "display: flex; align-items: center; gap: 1rem; margin-bottom: 1.25rem;",
                    img {
                        src: "assets/favicon.svg",
                        alt: "MGF precursor error icon",
                        style: "width: 56px; height: 56px; border-radius: 16px; box-shadow: 0 8px 24px rgba(15, 23, 42, 0.12);",
                    }
                    div {
                        h2 { style: "margin: 0; font-size: 1.7rem; letter-spacing: -0.02em;", "MGF Precursor Error Metrics" }
                        p {
                            style: "margin: 0.2rem 0 0; color: #475569; font-size: 0.95rem;",
                            "Upload an MGF file and summarize precursor mass errors in Da and ppm."
                        }
                    }
                }

                div {
                    style: "background: rgba(255,255,255,0.9); border: 1px solid rgba(148,163,184,0.22); border-radius: 20px; box-shadow: 0 12px 40px rgba(15, 23, 42, 0.08); padding: 1.25rem; backdrop-filter: blur(12px);",
                    label {
                        r#for: "mgf-upload",
                        style: format!(
                            "display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 0.6rem; min-height: 140px; width: 100%; box-sizing: border-box; position: relative; isolation: isolate; border: 2px dashed {}; border-radius: 18px; padding: 1.1rem; cursor: pointer; background: {}; color: #334155; font-weight: 600; text-align: center; transition: border-color 160ms ease, background 160ms ease, transform 160ms ease;",
                            if *drag_active.read() { "#2563eb" } else { "#94a3b8" },
                            if *drag_active.read() { "linear-gradient(135deg, rgba(219,234,254,0.96), rgba(239,246,255,0.94))" } else { "linear-gradient(135deg, rgba(248,250,252,0.95), rgba(239,246,255,0.95))" }
                        ),
                        ondragenter: on_drag_enter,
                        ondragover: on_drag_over,
                        ondragleave: on_drag_leave,
                        ondrop: on_drop,
                        span { style: "font-size: 1rem;", "Drop an MGF file here or click to browse" }
                        span { style: "font-size: 0.85rem; font-weight: 500; color: #64748b;", ".mgf files only" }
                        input {
                            id: "mgf-upload",
                            r#type: "file",
                            accept: ".mgf",
                            disabled: *busy.read(),
                            onchange: on_file_change,
                            style: "position: absolute; inset: 0; width: 100%; height: 100%; opacity: 0; cursor: pointer;",
                        }
                    }

                    if file_name.read().is_empty() && metrics.read().is_none() && !(*busy.read()) {
                        button {
                            r#type: "button",
                            style: "margin-top: 0.8rem; border: 1px solid #2563eb; border-radius: 999px; background: #eff6ff; color: #1d4ed8; font-size: 0.84rem; font-weight: 700; padding: 0.45rem 0.8rem; cursor: pointer;",
                            onclick: move |_| {
                                #[cfg(target_arch = "wasm32")]
                                load_example_mgf(status, metrics, busy, file_name);
                                #[cfg(not(target_arch = "wasm32"))]
                                {
                                    status.set("This app needs to run in a browser.".to_string());
                                }
                            },
                            "Load example MGF"
                        }
                    }

                    p {
                        style: "margin: 0.7rem 0 0; color: #475569; font-size: 0.9rem;",
                        if !file_name.read().is_empty() {
                            "Selected file: {file_name}"
                        }
                    }

                    p { style: "margin: 0.7rem 0 0; font-weight: 600; color: #334155;", "{status}" }

                    if let Some(metrics) = metrics.read().as_ref() {
                        div {
                            style: "margin-top: 1rem; padding: 1rem; border: 1px solid #e2e8f0; border-radius: 16px; background: linear-gradient(180deg, #ffffff 0%, #f8fafc 100%);",
                            h3 { style: "margin: 0 0 0.4rem; font-size: 1rem;", "Summary" }
                            p { style: "margin: 0.35rem 0; color: #475569;", "Processed {metrics.total_spectra} spectra; compared {metrics.spectra} with usable reference masses." }
                            p { style: "margin: 0.35rem 0; color: #475569;", "{metrics.spectra_with_reference_mass} spectra had a usable reference mass." }

                            if metrics.skipped_spectra > 0 || !metrics.unrecognized_adducts.is_empty() {
                                div {
                                    style: "margin-top: 0.9rem; padding: 0.8rem 0.9rem; border: 1px solid #fcd34d; border-radius: 12px; background: #fffbeb; color: #92400e;",
                                    p { style: "margin: 0 0 0.35rem; font-weight: 700;", "Warnings" }
                                    p { style: "margin: 0; font-size: 0.9rem;", "{metrics.skipped_spectra} spectra were skipped because the adduct or reference mass could not be resolved." }
                                    if metrics.unparsed_smiles > 0 {
                                        p { style: "margin: 0.45rem 0 0; font-size: 0.88rem;", "{metrics.unparsed_smiles} spectra had SMILES that could not be parsed into a reference mass." }
                                    }
                                    if !metrics.unparsed_smiles_warnings.is_empty() {
                                        div { style: "margin-top: 0.6rem; padding: 0.7rem 0.8rem; border: 1px solid #fde68a; border-radius: 10px; background: #fffbeb; color: #92400e;",
                                            p { style: "margin: 0 0 0.35rem; font-weight: 700; font-size: 0.86rem;", "Excluded unparsed SMILES" }
                                            ul { style: "margin: 0.25rem 0 0 1.05rem; padding: 0; font-size: 0.84rem; max-height: 160px; overflow: auto;",
                                                {
                                                    let mut sorted_unparsed = metrics.unparsed_smiles_warnings.iter().collect::<Vec<_>>();
                                                    sorted_unparsed.sort_by(|(left_smiles, left_detail), (right_smiles, right_detail)| {
                                                        right_detail.count.cmp(&left_detail.count).then_with(|| left_smiles.cmp(right_smiles))
                                                    });
                                                    rsx! {
                                                        for (smiles, detail) in sorted_unparsed {
                                                            {
                                                                let formula_display = detail.formula.as_deref().filter(|value| !value.trim().is_empty()).map_or_else(|| String::new(), |formula| format!(" [formula: {formula}]"));
                                                                let item_label = format!("{smiles} ({}){formula_display}", detail.count);
                                                                rsx! {
                                                                    li { "{item_label}" }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    if !metrics.unrecognized_adducts.is_empty() {
                                        ul { style: "margin: 0.45rem 0 0 1.1rem; padding: 0; font-size: 0.88rem;",
                                            {
                                                let mut sorted_adducts = metrics.unrecognized_adducts.iter().collect::<Vec<_>>();
                                                sorted_adducts.sort_by(|(left_adduct, left_count), (right_adduct, right_count)| {
                                                    right_count.cmp(left_count).then_with(|| left_adduct.cmp(right_adduct))
                                                });
                                                rsx! {
                                                    for (adduct, count) in sorted_adducts {
                                                        li { "{adduct}: {count}" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            if !metrics.high_error_smiles.is_empty() {
                                div {
                                    style: "margin-top: 1rem; padding: 0.8rem 0.9rem; border: 1px solid #fecaca; border-radius: 12px; background: #fef2f2; color: #991b1b;",
                                    p { style: "margin: 0 0 0.35rem; font-weight: 700;", "SMILES for spectra above 0.01 Da" }
                                    ul { style: "margin: 0.25rem 0 0 1.1rem; padding: 0; font-size: 0.88rem; max-height: 240px; overflow: auto;",
                                       {
                                           let mut sorted_high_error = metrics.high_error_smiles.iter().collect::<Vec<_>>();
                                           sorted_high_error.sort_by(|(left_smiles, left_detail), (right_smiles, right_detail)| {
                                               right_detail
                                                   .max_abs_error_da
                                                   .unwrap_or_default()
                                                   .total_cmp(&left_detail.max_abs_error_da.unwrap_or_default())
                                                   .then_with(|| right_detail.count.cmp(&left_detail.count))
                                                   .then_with(|| left_smiles.cmp(right_smiles))
                                           });
                                           rsx! {
                                               for (smiles, detail) in sorted_high_error {
                                                   {
                                                       let suffix = if detail.count > 1 { format!(" (x{})", detail.count) } else { String::new() };
                                                       let calc_value = detail.calculated_mass.map_or_else(|| "n/a".to_string(), format_value);
                                                       let expected_value = detail.expected_mass.map_or_else(|| "n/a".to_string(), format_value);
                                                       let observed_value = detail.observed_precursor_mz.map_or_else(|| "n/a".to_string(), format_value);
                                                       let max_error_da = detail.max_abs_error_da.map_or_else(|| "n/a".to_string(), format_value);
                                                       let max_error_ppm = detail.max_abs_error_ppm.map_or_else(|| "n/a".to_string(), format_value);
                                                       let formula_suffix = detail.formula.as_deref().filter(|value| !value.trim().is_empty()).map_or_else(|| String::new(), |formula| format!("; formula {formula}"));
                                                       let item_label = format!("{smiles}{suffix} — worst error {max_error_da} Da / {max_error_ppm} ppm (derived expected precursor {expected_value}; observed precursor {observed_value}; reference mass {calc_value}){formula_suffix}");
                                                       rsx! {
                                                           li { "{item_label}" }
                                                       }
                                                   }
                                               }
                                           }
                                       }
                                    }
                                }
                            }

                            div {
                                style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 1rem; margin-top: 1rem;",
                                div { style: "background: linear-gradient(135deg, #ffffff 0%, #f8fafc 100%); padding: 0.9rem; border-radius: 14px; border: 1px solid #e2e8f0; box-shadow: 0 10px 24px rgba(15, 23, 42, 0.06);",
                                    h4 { style: "margin: 0 0 0.35rem; font-size: 0.95rem; color: #0f172a;", "Observed precursor 𝑚/𝑧 distribution" }
                                    p { style: "margin: 0; color: #64748b; font-size: 0.8rem;", "Range, mean, and median of the observed precursor 𝑚/𝑧 values" }
                                    div { style: "display: flex; flex-wrap: wrap; gap: 0.45rem; margin-top: 0.6rem;",
                                        span { style: "display: inline-flex; align-items: center; gap: 0.3rem; padding: 0.35rem 0.6rem; border-radius: 999px; background: #eff6ff; color: #1d4ed8; border: 1px solid #bfdbfe; font-size: 0.78rem; font-weight: 700;", "median {format_value(metrics.observed_precursor_median)}" }
                                        span { style: "display: inline-flex; align-items: center; gap: 0.3rem; padding: 0.35rem 0.6rem; border-radius: 999px; background: #f8fafc; color: #334155; border: 1px solid #e2e8f0; font-size: 0.78rem; font-weight: 700;", "mean {format_value(metrics.observed_precursor_mean)}" }
                                        span { style: "display: inline-flex; align-items: center; gap: 0.3rem; padding: 0.35rem 0.6rem; border-radius: 999px; background: #f8fafc; color: #334155; border: 1px solid #e2e8f0; font-size: 0.78rem; font-weight: 700;", "range {format_value(metrics.observed_precursor_max - metrics.observed_precursor_min)}" }
                                    }
                                }
                                div { style: "background: linear-gradient(135deg, #ffffff 0%, #f8fafc 100%); padding: 0.9rem; border-radius: 14px; border: 1px solid #e2e8f0; box-shadow: 0 10px 24px rgba(15, 23, 42, 0.06);",
                                    h4 { style: "margin: 0 0 0.35rem; font-size: 0.95rem; color: #0f172a;", "Absolute precursor error (Da)" }
                                    p { style: "margin: 0; color: #64748b; font-size: 0.8rem;", "Median absolute deviation, mean absolute deviation, and RMS in daltons" }
                                    div { style: "display: flex; flex-wrap: wrap; gap: 0.45rem; margin-top: 0.6rem;",
                                        span { style: "display: inline-flex; align-items: center; gap: 0.3rem; padding: 0.35rem 0.6rem; border-radius: 999px; background: #eff6ff; color: #1d4ed8; border: 1px solid #bfdbfe; font-size: 0.78rem; font-weight: 700;", "median {format_value(metrics.abs_error_da_median)}" }
                                        span { style: "display: inline-flex; align-items: center; gap: 0.3rem; padding: 0.35rem 0.6rem; border-radius: 999px; background: #f8fafc; color: #334155; border: 1px solid #e2e8f0; font-size: 0.78rem; font-weight: 700;", "mean {format_value(metrics.abs_error_da_mean)}" }
                                        span { style: "display: inline-flex; align-items: center; gap: 0.3rem; padding: 0.35rem 0.6rem; border-radius: 999px; background: #f8fafc; color: #334155; border: 1px solid #e2e8f0; font-size: 0.78rem; font-weight: 700;", "RMS {format_value(metrics.abs_error_da_rms)}" }
                                    }
                                }
                                div { style: "background: linear-gradient(135deg, #ffffff 0%, #f8fafc 100%); padding: 0.9rem; border-radius: 14px; border: 1px solid #e2e8f0; box-shadow: 0 10px 24px rgba(15, 23, 42, 0.06);",
                                    h4 { style: "margin: 0 0 0.35rem; font-size: 0.95rem; color: #0f172a;", "Relative precursor error (ppm)" }
                                    p { style: "margin: 0; color: #64748b; font-size: 0.8rem;", "Median relative deviation, mean relative deviation, and RMS of the ppm error" }
                                    div { style: "display: flex; flex-wrap: wrap; gap: 0.45rem; margin-top: 0.6rem;",
                                        span { style: "display: inline-flex; align-items: center; gap: 0.3rem; padding: 0.35rem 0.6rem; border-radius: 999px; background: #eff6ff; color: #1d4ed8; border: 1px solid #bfdbfe; font-size: 0.78rem; font-weight: 700;", "median {format_value(metrics.abs_error_ppm_median)}" }
                                        span { style: "display: inline-flex; align-items: center; gap: 0.3rem; padding: 0.35rem 0.6rem; border-radius: 999px; background: #f8fafc; color: #334155; border: 1px solid #e2e8f0; font-size: 0.78rem; font-weight: 700;", "mean {format_value(metrics.abs_error_ppm_mean)}" }
                                        span { style: "display: inline-flex; align-items: center; gap: 0.3rem; padding: 0.35rem 0.6rem; border-radius: 999px; background: #f8fafc; color: #334155; border: 1px solid #e2e8f0; font-size: 0.78rem; font-weight: 700;", "RMS {format_value(metrics.abs_error_ppm_rms)}" }
                                    }
                                }
                            }

                            div {
                                style: "margin-top: 1rem; padding: 0.95rem 1rem; border: 1px solid #e2e8f0; border-radius: 16px; background: linear-gradient(180deg, #ffffff 0%, #f8fafc 100%); box-shadow: 0 10px 24px rgba(15, 23, 42, 0.06);",
                                h4 { style: "margin: 0 0 0.25rem; font-size: 0.95rem; color: #0f172a;", "Tolerance-band compliance" }
                                p { style: "margin: 0 0 0.7rem; color: #64748b; font-size: 0.84rem;", "Counts of spectra up to each reported mass-error cutoff (cumulative)" }
                                div { style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 0.6rem;",
                                    div { style: tolerance_card_style(0),
                                        strong { style: "display:block; font-size: 0.8rem; margin-bottom: 0.25rem;", "≤ 0.1 mDa" }
                                        span { style: "font-size: 0.88rem; font-weight: 700;", "{format_cumulative_bucket_count(metrics, \"0.1_da\", metrics.spectra)}" }
                                    }
                                    div { style: tolerance_card_style(1),
                                        strong { style: "display:block; font-size: 0.8rem; margin-bottom: 0.25rem;", "≤ 0.5 mDa" }
                                        span { style: "font-size: 0.88rem; font-weight: 700;", "{format_cumulative_bucket_count(metrics, \"0.5_da\", metrics.spectra)}" }
                                    }
                                    div { style: tolerance_card_style(2),
                                        strong { style: "display:block; font-size: 0.8rem; margin-bottom: 0.25rem;", "≤ 1.0 mDa" }
                                        span { style: "font-size: 0.88rem; font-weight: 700;", "{format_cumulative_bucket_count(metrics, \"1.0_da\", metrics.spectra)}" }
                                    }
                                    div { style: tolerance_card_style(3),
                                        strong { style: "display:block; font-size: 0.8rem; margin-bottom: 0.25rem;", "≤ 5.0 mDa" }
                                        span { style: "font-size: 0.88rem; font-weight: 700;", "{format_cumulative_bucket_count(metrics, \"5.0_da\", metrics.spectra)}" }
                                    }
                                    div { style: tolerance_card_style(4),
                                        strong { style: "display:block; font-size: 0.8rem; margin-bottom: 0.25rem;", "> 5.0 mDa" }
                                        span { style: "font-size: 0.88rem; font-weight: 700;", "{format_cumulative_bucket_count(metrics, \">5.0_da\", metrics.spectra)}" }
                                    }
                                    div { style: tolerance_card_style(0),
                                        strong { style: "display:block; font-size: 0.8rem; margin-bottom: 0.25rem;", "≤ 0.5 ppm" }
                                        span { style: "font-size: 0.88rem; font-weight: 700;", "{format_cumulative_bucket_count(metrics, \"0.5_ppm\", metrics.spectra)}" }
                                    }
                                    div { style: tolerance_card_style(1),
                                        strong { style: "display:block; font-size: 0.8rem; margin-bottom: 0.25rem;", "≤ 1.0 ppm" }
                                        span { style: "font-size: 0.88rem; font-weight: 700;", "{format_cumulative_bucket_count(metrics, \"1.0_ppm\", metrics.spectra)}" }
                                    }
                                    div { style: tolerance_card_style(2),
                                        strong { style: "display:block; font-size: 0.8rem; margin-bottom: 0.25rem;", "≤ 5.0 ppm" }
                                        span { style: "font-size: 0.88rem; font-weight: 700;", "{format_cumulative_bucket_count(metrics, \"5.0_ppm\", metrics.spectra)}" }
                                    }
                                    div { style: tolerance_card_style(3),
                                        strong { style: "display:block; font-size: 0.8rem; margin-bottom: 0.25rem;", "≤ 10.0 ppm" }
                                        span { style: "font-size: 0.88rem; font-weight: 700;", "{format_cumulative_bucket_count(metrics, \"10.0_ppm\", metrics.spectra)}" }
                                    }
                                    div { style: tolerance_card_style(4),
                                        strong { style: "display:block; font-size: 0.8rem; margin-bottom: 0.25rem;", "> 10.0 ppm" }
                                        span { style: "font-size: 0.88rem; font-weight: 700;", "{format_cumulative_bucket_count(metrics, \">10.0_ppm\", metrics.spectra)}" }
                                    }
                                }
                            }

                            div {
                                style: "margin-top: 1rem; display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 1rem;",
                                ecdf_plot {
                                    title: "Absolute precursor-error cumulative distribution (mDa)".to_string(),
                                    subtitle: "Cumulative fraction below each tolerance cutoff, shown on a log10 scale".to_string(),
                                    values: metrics.absolute_error_da_values.iter().map(|value| value * 1000.0).collect::<Vec<_>>(),
                                    thresholds: vec![0.1, 0.5, 1.0, 5.0],
                                    unit: "mDa".to_string(),
                                }
                                ecdf_plot {
                                    title: "Relative precursor-error cumulative distribution (ppm)".to_string(),
                                    subtitle: "Cumulative fraction below each ppm tolerance cutoff, shown on a log10 scale".to_string(),
                                    values: metrics.absolute_error_ppm_values.clone(),
                                    thresholds: vec![0.5, 1.0, 5.0, 10.0],
                                    unit: "ppm".to_string(),
                                }
                                absolute_mass_bias_plot {
                                    title: "Signed error vs. precursor 𝑚/𝑧 (mDa)".to_string(),
                                    subtitle: "Signed error at each precursor 𝑚/𝑧, centered on zero and shown with a symmetric y-axis".to_string(),
                                    points: metrics.plot_points.clone(),
                                    unit: "mDa".to_string(),
                                    ticks: vec![0.1, 0.5, 1.0, 5.0],
                                }
                                absolute_mass_bias_plot {
                                    title: "Signed relative error vs. precursor 𝑚/𝑧 (ppm)".to_string(),
                                    subtitle: "Signed relative error at each precursor 𝑚/𝑧, centered on zero and shown with a symmetric y-axis".to_string(),
                                    points: metrics.plot_points.clone(),
                                    unit: "ppm".to_string(),
                                    ticks: vec![0.5, 1.0, 5.0, 10.0],
                                }
                            }

                        }
                    }
                }
            }
        }
    }
}

fn median(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(|left, right| left.total_cmp(right));
    let midpoint = sorted.len() / 2;
    if sorted.len() % 2 == 0 {
        (sorted[midpoint - 1] + sorted[midpoint]) / 2.0
    } else {
        sorted[midpoint]
    }
}

fn format_value(value: f64) -> String {
    if value.is_finite() {
        format!("{value:.4}")
    } else {
        "n/a".to_string()
    }
}

fn format_count_with_percentage(count: usize, total: usize) -> String {
    if total == 0 {
        format!("{count} (0.0%)")
    } else {
        let pct = (count as f64 / total as f64) * 100.0;
        format!("{count} ({pct:.1}%)")
    }
}

fn format_cumulative_bucket_count(
    metrics: &PrecursorMetrics,
    bucket: &str,
    total: usize,
) -> String {
    let count = match bucket {
        "0.1_da" => metrics.within_0_1_da,
        "0.5_da" => metrics.within_0_1_da + metrics.within_0_5_da,
        "1.0_da" => metrics.within_0_1_da + metrics.within_0_5_da + metrics.within_1_da,
        "5.0_da" => {
            metrics.within_0_1_da
                + metrics.within_0_5_da
                + metrics.within_1_da
                + metrics.within_5_da
        }
        ">5.0_da" => metrics.above_5_da,
        "0.5_ppm" => metrics.within_0_5_ppm,
        "1.0_ppm" => metrics.within_0_5_ppm + metrics.within_1_ppm,
        "5.0_ppm" => metrics.within_0_5_ppm + metrics.within_1_ppm + metrics.within_5_ppm,
        "10.0_ppm" => {
            metrics.within_0_5_ppm
                + metrics.within_1_ppm
                + metrics.within_5_ppm
                + metrics.within_10_ppm
        }
        ">10.0_ppm" => metrics.above_10_ppm,
        _ => 0,
    };
    format_count_with_percentage(count, total)
}

fn normalize_adduct_label(adduct: &str) -> String {
    let trimmed = adduct.trim();
    if trimmed.is_empty() {
        return "unknown".to_string();
    }

    let normalized = trimmed.replace(' ', "").to_ascii_uppercase();
    let (body, suffix) = if let Some(idx) = normalized.find(']') {
        let body = &normalized[1..idx];
        let suffix = &normalized[idx + 1..];
        (body, suffix)
    } else {
        (normalized.as_str(), "")
    };
    let body = body.trim_matches(|ch| ch == '[' || ch == ']');
    let suffix = suffix.trim();

    match (body, suffix) {
        ("M", "") | ("M", "+") => "[M]+".to_string(),
        ("M", "++") | ("M", "2+") => "[M]2+".to_string(),
        ("M+2NA", "") | ("M+2NA", "2+") | ("M+2NA", "++") => "[M+2Na]2+".to_string(),
        ("M+H", "") | ("M+H", "+") => "[M+H]+".to_string(),
        ("M+K", "") | ("M+K", "+") => "[M+K]+".to_string(),
        ("M+NH4", "") | ("M+NH4", "+") => "[M+NH4]+".to_string(),
        ("M+NA", "") | ("M+NA", "+") => "[M+Na]+".to_string(),
        ("M+2H", "") | ("M+2H", "+") | ("M+2H", "++") | ("M+2H", "2+") => "[M+2H]2+".to_string(),
        ("M-H", "") | ("M-H", "-") | ("M-H", "1-") | ("M-H", "--") => "[M-H]-".to_string(),
        ("M-2H", "") | ("M-2H", "2-") | ("M-2H", "--") => "[M-2H]2-".to_string(),
        ("4M-H", "") | ("4M-H", "-") | ("4M-H", "1-") | ("4M-H", "--") => "[4M-H]-".to_string(),
        _ => trimmed.to_string(),
    }
}

fn normalize_adduct_key(adduct: &str) -> String {
    adduct.trim().replace(' ', "").to_ascii_uppercase()
}

fn is_excluded_adduct(adduct: &str) -> bool {
    let _ = adduct;
    false
}

fn is_supported_adduct(adduct: &str) -> bool {
    parse_adduct_mass_spec(adduct).is_some()
}

fn adduct_class(adduct: &str) -> Option<AdductClass> {
    let normalized = normalize_adduct_label(adduct);
    let charge = parse_adduct_charge_sign(Some(adduct)).map_or_else(
        || {
            if normalized.contains("]-") {
                -1
            } else if normalized.contains("]+") || normalized.contains("]2+") {
                1
            } else if normalized.contains("]2-") {
                -2
            } else {
                0
            }
        },
        |sign| if sign { -1 } else { 1 },
    );
    let family = if normalized.contains("[M+H]")
        || normalized.contains("[M+2H]")
        || normalized.contains("[M+NH4]")
    {
        "Protonated".to_string()
    } else if normalized.contains("[M-H]") || normalized.contains("[M-2H]") {
        "Deprotonated".to_string()
    } else if normalized.contains("[M+NA]")
        || normalized.contains("[M+K]")
        || normalized.contains("[M+NH4]")
    {
        "Alkali / ammonium".to_string()
    } else if normalized.contains("MG") || normalized.contains("CA") || normalized.contains("FE") {
        "Metal / complex".to_string()
    } else if normalized.contains("CL") || normalized.contains("BR") {
        "Halide".to_string()
    } else {
        "Other".to_string()
    };

    Some(AdductClass {
        label: normalized.clone(),
        display: normalized,
        family,
        charge,
    })
}

fn paul_tol_palette(index: usize) -> &'static str {
    [
        "#4477AA", "#66CCEE", "#228833", "#CCBB44", "#EE6677", "#AA3377", "#BBBBBB", "#004488",
    ][index % 8]
}

fn adduct_family_rank(family: &str) -> usize {
    match family {
        "Protonated" => 0,
        "Deprotonated" => 1,
        "Alkali / ammonium" => 2,
        "Metal / complex" => 3,
        "Halide" => 4,
        "Other" => 5,
        _ => 6,
    }
}

fn adduct_family_color_hex(family: &str) -> String {
    let palette_index = adduct_family_rank(family);
    paul_tol_palette(palette_index).to_string()
}

fn adduct_family_color(family: &str) -> plotters::style::RGBColor {
    let color = adduct_family_color_hex(family);
    let hex = color.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    plotters::style::RGBColor(r, g, b)
}

fn adduct_family_shape_style(family: &str, alpha: f32) -> plotters::style::ShapeStyle {
    let color = adduct_family_color_hex(family);
    let hex = color.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    let alpha = alpha.clamp(0.0, 1.0) as f64;
    plotters::style::ShapeStyle::from(&plotters::style::RGBAColor(r, g, b, alpha)).filled()
}

fn prepare_scatter_plot_data<F, G>(
    points: &[PlotPoint],
    x_value_fn: F,
    y_value_fn: G,
    fallback_y_limit: f64,
) -> ScatterPlotData
where
    F: Fn(&PlotPoint) -> Option<f64>,
    G: Fn(&PlotPoint) -> Option<f64>,
{
    let mut family_points: HashMap<String, Vec<(f64, f64)>> = HashMap::new();
    let mut x_values = Vec::new();
    let mut y_values = Vec::new();

    for point in points {
        let Some(x_value) = x_value_fn(point) else {
            continue;
        };
        let Some(y_value) = y_value_fn(point) else {
            continue;
        };
        if !x_value.is_finite() || !y_value.is_finite() {
            continue;
        }
        x_values.push(x_value);
        y_values.push(y_value);
        family_points
            .entry(point.adduct_family.clone())
            .or_default()
            .push((x_value, y_value));
    }

    let x_min = x_values
        .iter()
        .copied()
        .fold(f64::INFINITY, f64::min)
        .min(1.0);
    let x_max = x_values
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max)
        .max(x_min + 1.0);
    let x_span = (x_max - x_min).max(1.0);
    let x_min = x_min - x_span * 0.05;
    let x_max = x_max + x_span * 0.05;

    let y_limit = y_values
        .iter()
        .copied()
        .map(|value| value.abs())
        .fold(0.0, f64::max)
        .max(fallback_y_limit);

    let mut families = family_points.keys().cloned().collect::<Vec<_>>();
    families.sort_by_key(|family| adduct_family_rank(family));
    let family_count = families.len().max(1);
    let max_points_per_family = (1500usize / family_count).max(180usize);
    let mut series = Vec::with_capacity(families.len());
    for family in families {
        let sampled = sample_scatter_points(
            family_points.remove(&family).unwrap_or_default(),
            max_points_per_family,
        );
        series.push((family.clone(), sampled));
    }

    let legend_items = series
        .iter()
        .map(|(family, _)| (family.clone(), adduct_family_color_hex(family)))
        .collect();

    ScatterPlotData {
        legend_items,
        x_min,
        x_max,
        y_limit,
        series,
    }
}

fn embed_svg_legend(
    svg_markup: &str,
    legend_items: &[(String, String)],
    title: &str,
    width: f64,
    height: f64,
) -> String {
    if legend_items.is_empty() {
        return svg_markup.to_string();
    }

    let mut legend_entries = String::new();
    let item_height = 13.5;
    let entry_gap = 10.0;
    let inset = 18.0;
    let title_width = 44.0;
    let marker_radius = 3.2;
    let text_height = 11.0;
    let padding_x = 12.0;
    let padding_y = 10.0;
    let label_width = legend_items
        .iter()
        .map(|(label, _)| label.len())
        .max()
        .unwrap_or(0);
    let content_width = (label_width as f64 * 5.6).max(72.0).min(170.0) + 24.0;
    let entry_width = content_width + 20.0;
    let box_width =
        (entry_width * legend_items.len() as f64) + (title_width + 12.0) + (padding_x * 2.0);
    let box_height = item_height + 20.0;
    let legend_x = ((width - box_width) / 2.0)
        .max(inset)
        .min(width - box_width - inset);
    let legend_y = (height - box_height - inset).max(inset);

    let title_x = legend_x + 10.0;
    let title_y = legend_y + 12.0;
    let items_start_x = title_x + title_width + 10.0;

    for (index, (family, color)) in legend_items.iter().enumerate() {
        let item_x = items_start_x + (index as f64 * entry_width);
        let marker_x = item_x + 8.0;
        let text_x = item_x + 18.0;
        let text_y = legend_y + 13.0;
        let marker_y = legend_y + 10.0;
        legend_entries.push_str(&format!(
            "<g>\n                <circle cx=\"{marker_x}\" cy=\"{marker_y}\" r=\"{marker_radius}\" fill=\"{color}\" />\n                <text x=\"{text_x}\" y=\"{text_y}\" font-family=\"Inter, sans-serif\" font-size=\"10\" fill=\"#334155\">{family}</text>\n            </g>"
        ));
    }

    let rect_x = legend_x;
    let rect_y = legend_y - 2.0;
    let legend_group = format!(
        "<g>\n            <rect x=\"{rect_x}\" y=\"{rect_y}\" width=\"{box_width}\" height=\"{box_height}\" rx=\"8\" ry=\"8\" fill=\"#f8fafc\" fill-opacity=\"0.97\" stroke=\"#cbd5e1\" stroke-width=\"0.8\" />\n            <text x=\"{title_x}\" y=\"{title_y}\" font-family=\"Inter, sans-serif\" font-size=\"10.5\" font-weight=\"600\" fill=\"#0f172a\">{title}</text>\n            {legend_entries}\n        </g>"
    );

    if let Some(position) = svg_markup.rfind("</svg>") {
        let mut result = svg_markup[..position].to_string();
        result.push('\n');
        result.push_str(&legend_group);
        result.push('\n');
        result.push_str(&svg_markup[position..]);
        result
    } else {
        svg_markup.to_string()
    }
}

fn tolerance_step_color(index: usize, total_steps: usize) -> String {
    let total = total_steps.max(2);
    let normalized = index.min(total.saturating_sub(1));
    let lut_index = if total <= 4 {
        let discrete_positions = [200usize, 150, 100, 50];
        discrete_positions[normalized.min(discrete_positions.len().saturating_sub(1))]
    } else {
        let fraction = normalized as f32 / (total - 1) as f32;
        ((255.0 * (1.0 - fraction)).round() as usize).clamp(0, 255)
    };
    let [r, g, b] = BATLOW.lut[lut_index];
    format!("#{r:02x}{g:02x}{b:02x}")
}

fn tolerance_step_rgb(index: usize, total_steps: usize) -> plotters::style::RGBColor {
    let total = total_steps.max(2);
    let normalized = index.min(total.saturating_sub(1));
    let lut_index = if total <= 4 {
        let discrete_positions = [200usize, 150, 100, 50];
        discrete_positions[normalized.min(discrete_positions.len().saturating_sub(1))]
    } else {
        let fraction = normalized as f32 / (total - 1) as f32;
        ((255.0 * (1.0 - fraction)).round() as usize).clamp(0, 255)
    };
    let [r, g, b] = BATLOW.lut[lut_index];
    plotters::style::RGBColor(r, g, b)
}

fn tolerance_card_style(index: usize) -> String {
    let color = tolerance_step_color(index, 5);
    format!(
        "padding: 0.6rem 0.7rem; border-radius: 12px; border: 1px solid {color}; background: #f8fafc; color: {color};"
    )
}

fn format_threshold_value(value: f64) -> String {
    let formatted = format!("{value:.6}");
    let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
    if trimmed.is_empty() {
        "0".to_string()
    } else {
        trimmed.to_string()
    }
}

fn format_error_tick(value: f64, unit: &str) -> String {
    if unit == "ppm" {
        format!("{value:.2}")
    } else {
        format!("{value:.2}")
    }
}

fn display_error_value(value: f64, unit: &str) -> f64 {
    if unit == "mDa" { value * 1000.0 } else { value }
}

fn display_error_value_for_point(point: &PlotPoint, unit: &str) -> f64 {
    match unit {
        "mDa" => point.signed_error_da * 1000.0,
        "ppm" => point.signed_error_ppm,
        _ => point.signed_error_da * 1000.0,
    }
}

fn transform_signed_error(value: f64, floor: f64) -> f64 {
    if value == 0.0 {
        0.0
    } else if value > 0.0 {
        value.abs().max(floor / 10.0).log10()
    } else {
        -value.abs().max(floor / 10.0).log10()
    }
}

fn make_svg_responsive(svg_markup: String) -> String {
    let mut normalized = svg_markup;
    normalized = normalized
        .replace("width=\"800\"", "width=\"100%\"")
        .replace("width=\"900\"", "width=\"100%\"")
        .replace("height=\"460\"", "height=\"auto\"")
        .replace("height=\"520\"", "height=\"auto\"");
    if !normalized.contains("viewBox=") {
        normalized = normalized.replacen("<svg", "<svg viewBox=\"0 0 900 520\"", 1);
    }
    if !normalized.contains("preserveAspectRatio=") {
        normalized = normalized.replacen("<svg", "<svg preserveAspectRatio=\"xMidYMid meet\"", 1);
    }
    if !normalized.contains("style=") {
        normalized = normalized.replacen(
            "<svg",
            "<svg style=\"max-width:100%; height:auto; display:block; overflow:visible;\"",
            1,
        );
    }
    normalized
}

#[cfg(target_arch = "wasm32")]
fn download_svg(svg_markup: &str, filename: &str) {
    let safe_name = if filename.ends_with(".svg") {
        filename.to_string()
    } else {
        format!("{filename}.svg")
    };

    let array = Array::new();
    array.push(&JsValue::from(svg_markup));
    let blob = Blob::new_with_str_sequence(&array).unwrap();
    let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let anchor: web_sys::HtmlAnchorElement = document
        .create_element("a")
        .unwrap()
        .dyn_into::<web_sys::HtmlAnchorElement>()
        .unwrap();
    anchor.set_attribute("href", &url).unwrap();
    anchor.set_attribute("download", &safe_name).unwrap();
    anchor.set_attribute("style", "display:none").unwrap();
    document.body().unwrap().append_child(&anchor).unwrap();
    anchor.click();
    document.body().unwrap().remove_child(&anchor).unwrap();
    web_sys::Url::revoke_object_url(&url).unwrap();
}

#[cfg(not(target_arch = "wasm32"))]
fn download_svg(_svg_markup: &str, _filename: &str) {}

fn build_ecdf_points(values: &[f64], x_min: f64, x_max: f64) -> Vec<(f64, f64)> {
    if values.is_empty() {
        return vec![(x_min, 0.0), (x_max, 1.0)];
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(|left, right| left.total_cmp(right));
    let total = sorted.len();
    let mut points = Vec::with_capacity(sorted.len().saturating_mul(2) + 2);
    points.push((x_min, 0.0));

    let mut index = 0usize;
    let mut previous_y = 0.0f64;
    while index < sorted.len() {
        let value = sorted[index];
        let mut next_index = index + 1;
        while next_index < sorted.len() && sorted[next_index] == value {
            next_index += 1;
        }
        let y = next_index as f64 / total as f64;
        let x = value.max(x_min);
        points.push((x, previous_y));
        points.push((x, y));
        previous_y = y;
        index = next_index;
    }

    points.push((x_max, 1.0));
    points
}

fn render_ecdf_svg(title: &str, values: &[f64], thresholds: &[f64], unit: &str) -> String {
    use plotters::prelude::*;
    use plotters::series::LineSeries;

    let width = 900u32;
    let height = 520u32;
    let mut buffer = String::new();
    let root = SVGBackend::with_string(&mut buffer, (width, height)).into_drawing_area();
    root.fill(&WHITE).unwrap();

    let legend_items = thresholds
        .iter()
        .enumerate()
        .map(|(index, threshold)| {
            let label = format!("≤ {} {unit}", format_threshold_value(*threshold));
            (label, tolerance_step_color(index, thresholds.len()))
        })
        .collect::<Vec<_>>();

    let observed_min = values
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .fold(f64::INFINITY, f64::min);
    let x_min = if observed_min.is_finite() {
        observed_min.max(1e-6)
    } else {
        1e-6
    };
    let observed_max = values
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .fold(0.0, f64::max);
    let view_max = thresholds.iter().copied().fold(0.0, f64::max).max(1e-6);
    let plot_max = observed_max.max(view_max);
    let x_max = (plot_max * 1.03).max(x_min + 1e-3);
    let x_max = if unit == "mDa" {
        x_max.max(5.0)
    } else {
        x_max.max(10.0)
    };
    let y_floor = 1e-6f64;
    let y_min = y_floor;
    let y_max = 1.0f64;

    {
        let mut chart = ChartBuilder::on(&root)
            .margin_top(28)
            .margin_right(32)
            .margin_bottom(36)
            .margin_left(48)
            .caption(title, ("sans-serif", 18).into_font())
            .set_label_area_size(LabelAreaPosition::Left, 72)
            .set_label_area_size(LabelAreaPosition::Bottom, 64)
            .build_cartesian_2d((x_min..x_max).log_scale(), y_min..y_max)
            .unwrap();

        chart
            .configure_mesh()
            .axis_style(ShapeStyle::from(&RGBColor(148, 163, 184)))
            .light_line_style(ShapeStyle::from(&RGBColor(226, 232, 240)))
            .bold_line_style(ShapeStyle::from(&RGBColor(100, 116, 139)))
            .x_desc(if unit == "ppm" {
                format!("Relative error ({unit})")
            } else {
                format!("Absolute error ({unit})")
            })
            .y_desc("cumulative fraction")
            .x_label_style(("sans-serif", 11).into_font())
            .y_label_style(("sans-serif", 11).into_font())
            .draw()
            .unwrap();

        chart
            .draw_series(LineSeries::new(
                vec![(x_min, y_min), (x_max, y_min)],
                ShapeStyle::from(&RGBColor(203, 213, 225)).stroke_width(1),
            ))
            .unwrap();

        let plot_points = build_ecdf_points(values, x_min, x_max);

        chart
            .draw_series(LineSeries::new(
                plot_points.clone(),
                ShapeStyle::from(&RGBColor(37, 99, 235)).stroke_width(3),
            ))
            .unwrap();

        for (index, threshold) in thresholds.iter().enumerate() {
            let x = threshold.clamp(x_min, x_max);
            let color = tolerance_step_rgb(index, thresholds.len());
            chart
                .draw_series(LineSeries::new(
                    vec![(x, y_min), (x, y_max)],
                    ShapeStyle::from(&color).stroke_width(2),
                ))
                .unwrap();
        }

        root.present().unwrap();
    }

    drop(root);
    embed_svg_legend(&buffer, &legend_items, "Thresholds", 900.0, 520.0)
}

fn sample_scatter_points(points: Vec<(f64, f64)>, max_points: usize) -> Vec<(f64, f64)> {
    if points.len() <= max_points.max(1) {
        return points;
    }

    let target = max_points.max(1);
    let step = points.len() as f64 / target as f64;
    let mut sampled = Vec::with_capacity(target);
    let mut index = 0.0f64;
    while index < points.len() as f64 {
        let point_index = index.round() as usize;
        if point_index < points.len() {
            sampled.push(points[point_index]);
        }
        index += step;
    }
    if sampled.last() != points.last() {
        sampled.push(points[points.len() - 1]);
    }
    sampled
}

fn render_mass_bias_svg(title: &str, points: &[PlotPoint]) -> String {
    use plotters::prelude::*;
    use plotters::series::{LineSeries, PointSeries};

    let width = 900u32;
    let height = 520u32;
    let mut buffer = String::new();
    let root = SVGBackend::with_string(&mut buffer, (width, height)).into_drawing_area();
    root.fill(&WHITE).unwrap();

    let plot_data = prepare_scatter_plot_data(
        points,
        |point| Some(point.observed_precursor_mz),
        |point| {
            point
                .signed_error_da
                .is_finite()
                .then_some(point.signed_error_da)
        },
        1e-4,
    );
    let legend_items = plot_data.legend_items;
    let x_min = plot_data.x_min;
    let x_max = plot_data.x_max;
    let y_limit = plot_data.y_limit;
    let y_min = -y_limit;
    let y_max = y_limit;
    let points_by_family = plot_data.series;

    {
        let mut chart = ChartBuilder::on(&root)
            .margin_top(28)
            .margin_right(32)
            .margin_bottom(36)
            .margin_left(48)
            .caption(title, ("sans-serif", 18).into_font())
            .set_label_area_size(LabelAreaPosition::Left, 72)
            .set_label_area_size(LabelAreaPosition::Bottom, 64)
            .build_cartesian_2d(x_min..x_max, y_min..y_max)
            .unwrap();

        chart
            .configure_mesh()
            .axis_style(ShapeStyle::from(&RGBColor(148, 163, 184)))
            .light_line_style(ShapeStyle::from(&RGBColor(226, 232, 240)))
            .bold_line_style(ShapeStyle::from(&RGBColor(100, 116, 139)))
            .x_desc("Observed precursor 𝑚/𝑧")
            .y_desc("Signed error (Da)")
            .x_label_style(("sans-serif", 11).into_font())
            .y_label_style(("sans-serif", 11).into_font())
            .draw()
            .unwrap();

        chart
            .draw_series(LineSeries::new(
                vec![(x_min, 0.0f64), (x_max, 0.0f64)],
                ShapeStyle::from(&RGBColor(148, 163, 184)).stroke_width(1),
            ))
            .unwrap();

        for (family, points) in points_by_family {
            let style = adduct_family_shape_style(&family, 0.4);
            chart
                .draw_series(PointSeries::of_element(
                    points.iter().copied().collect::<Vec<_>>(),
                    1.6,
                    style,
                    &|coord, size, style| Circle::new(coord, size, style.filled()),
                ))
                .unwrap();
        }

        root.present().unwrap();
    }

    drop(root);
    embed_svg_legend(&buffer, &legend_items, "Adducts", 900.0, 520.0)
}

fn render_absolute_mass_bias_svg(
    title: &str,
    points: &[PlotPoint],
    unit: &str,
    ticks: &[f64],
) -> String {
    use plotters::prelude::*;
    use plotters::series::{LineSeries, PointSeries};

    let width = 900u32;
    let height = 520u32;
    let mut buffer = String::new();
    let root = SVGBackend::with_string(&mut buffer, (width, height)).into_drawing_area();
    root.fill(&WHITE).unwrap();

    let plot_data = prepare_scatter_plot_data(
        points,
        |point| {
            point
                .observed_precursor_mz
                .is_finite()
                .then_some(point.observed_precursor_mz)
        },
        |point| {
            let error_value = display_error_value_for_point(point, unit);
            error_value.is_finite().then_some(error_value)
        },
        1e-3,
    );
    let legend_items = plot_data.legend_items;
    let x_min = plot_data.x_min;
    let x_max = plot_data.x_max;
    let y_limit = plot_data.y_limit;
    let y_min = -y_limit;
    let y_max = y_limit;
    let points_by_family = plot_data.series;

    {
        let mut chart = ChartBuilder::on(&root)
            .margin_top(28)
            .margin_right(32)
            .margin_bottom(36)
            .margin_left(48)
            .caption(title, ("sans-serif", 18).into_font())
            .set_label_area_size(LabelAreaPosition::Left, 72)
            .set_label_area_size(LabelAreaPosition::Bottom, 64)
            .build_cartesian_2d(x_min..x_max, y_min..y_max)
            .unwrap();

        chart
            .configure_mesh()
            .axis_style(ShapeStyle::from(&RGBColor(148, 163, 184)))
            .light_line_style(ShapeStyle::from(&RGBColor(226, 232, 240)))
            .bold_line_style(ShapeStyle::from(&RGBColor(100, 116, 139)))
            .x_desc("Observed precursor 𝑚/𝑧")
            .y_desc(if unit == "ppm" {
                format!("Signed error ({unit})")
            } else {
                format!("Signed error ({unit})")
            })
            .x_label_style(("sans-serif", 11).into_font())
            .y_label_style(("sans-serif", 11).into_font())
            .draw()
            .unwrap();

        chart
            .draw_series(LineSeries::new(
                vec![(x_min, 0.0), (x_max, 0.0)],
                ShapeStyle::from(&RGBColor(148, 163, 184)).stroke_width(1),
            ))
            .unwrap();

        for tick in ticks {
            if *tick <= 0.0 {
                continue;
            }
            let positive_tick = (*tick).min(y_limit);
            let negative_tick = -positive_tick;
            if !(y_min..=y_max).contains(&positive_tick) {
                continue;
            }
            chart
                .draw_series(LineSeries::new(
                    vec![(x_min, positive_tick), (x_max, positive_tick)],
                    ShapeStyle::from(&RGBColor(226, 232, 240)).stroke_width(1),
                ))
                .unwrap();
            chart
                .draw_series(LineSeries::new(
                    vec![(x_min, negative_tick), (x_max, negative_tick)],
                    ShapeStyle::from(&RGBColor(226, 232, 240)).stroke_width(1),
                ))
                .unwrap();
        }

        for (family, points) in points_by_family {
            let style = adduct_family_shape_style(&family, 0.3);
            chart
                .draw_series(PointSeries::of_element(
                    points
                        .iter()
                        .map(|(x, value)| (*x, value.clamp(-y_limit, y_limit)))
                        .collect::<Vec<_>>(),
                    1.6,
                    style,
                    &|coord, size, style| Circle::new(coord, size, style.filled()),
                ))
                .unwrap();
        }

        root.present().unwrap();
    }

    drop(root);
    embed_svg_legend(&buffer, &legend_items, "Adducts", 900.0, 520.0)
}

#[component]
fn ecdf_plot(
    title: String,
    subtitle: String,
    values: Vec<f64>,
    thresholds: Vec<f64>,
    unit: String,
) -> Element {
    let svg_markup = make_svg_responsive(render_ecdf_svg(&title, &values, &thresholds, &unit));
    let download_markup = svg_markup.clone();

    rsx! {
        div {
            style: "padding: 0.95rem; border: 1px solid #e2e8f0; border-radius: 18px; background: linear-gradient(180deg, #ffffff 0%, #f8fafc 100%); box-shadow: 0 12px 24px rgba(15, 23, 42, 0.04);",
            div { style: "display: flex; align-items: center; justify-content: space-between; gap: 0.6rem; margin-bottom: 0.65rem;",
               div { style: "flex: 1;",
                   h4 { style: "margin: 0 0 0.2rem; font-size: 0.95rem; color: #0f172a;", "{title}" }
                   p { style: "margin: 0; color: #64748b; font-size: 0.84rem;", "{subtitle}" }
               }
               button {
                   r#type: "button",
                   style: "border: 1px solid #cbd5e1; border-radius: 999px; background: white; color: #334155; font-size: 0.76rem; font-weight: 700; padding: 0.35rem 0.65rem; cursor: pointer;",
                   onclick: move |_| {
                       #[cfg(target_arch = "wasm32")]
                       download_svg(&download_markup, &title);
                   },
                   "Download"
               }
            }
            div { style: "border-radius: 16px; overflow: visible; border: 1px solid #e2e8f0; background: #fcfdff;",
               dangerous_inner_html: svg_markup
            }
        }
    }
}

#[component]
fn mass_bias_plot(
    title: String,
    subtitle: String,
    points: Vec<PlotPoint>,
    other_label: Option<String>,
) -> Element {
    let svg_markup = make_svg_responsive(render_mass_bias_svg(&title, &points));
    let download_markup = svg_markup.clone();

    rsx! {
        div {
            style: "padding: 0.95rem; border: 1px solid #e2e8f0; border-radius: 18px; background: linear-gradient(180deg, #ffffff 0%, #f8fafc 100%); box-shadow: 0 12px 24px rgba(15, 23, 42, 0.04);",
            div { style: "display: flex; align-items: center; justify-content: space-between; gap: 0.6rem; margin-bottom: 0.65rem;",
               div { style: "flex: 1;",
                   h4 { style: "margin: 0 0 0.2rem; font-size: 0.95rem; color: #0f172a;", "{title}" }
                   p { style: "margin: 0; color: #64748b; font-size: 0.84rem;", "{subtitle}" }
               }
               button {
                   r#type: "button",
                   style: "border: 1px solid #cbd5e1; border-radius: 999px; background: white; color: #334155; font-size: 0.76rem; font-weight: 700; padding: 0.35rem 0.65rem; cursor: pointer;",
                   onclick: move |_| {
                       #[cfg(target_arch = "wasm32")]
                       download_svg(&download_markup, &title);
                   },
                   "Download"
               }
            }
            div { style: "border-radius: 16px; overflow: visible; border: 1px solid #e2e8f0; background: #fcfdff;",
               dangerous_inner_html: svg_markup
            }
        }
    }
}

#[component]
fn absolute_mass_bias_plot(
    title: String,
    subtitle: String,
    points: Vec<PlotPoint>,
    unit: String,
    ticks: Vec<f64>,
) -> Element {
    let svg_markup = make_svg_responsive(render_absolute_mass_bias_svg(
        &title, &points, &unit, &ticks,
    ));
    let download_markup = svg_markup.clone();

    rsx! {
        div {
            style: "padding: 0.95rem; border: 1px solid #e2e8f0; border-radius: 18px; background: linear-gradient(180deg, #ffffff 0%, #f8fafc 100%); box-shadow: 0 12px 24px rgba(15, 23, 42, 0.04);",
            div { style: "display: flex; align-items: center; justify-content: space-between; gap: 0.6rem; margin-bottom: 0.65rem;",
               div { style: "flex: 1;",
                   h4 { style: "margin: 0 0 0.2rem; font-size: 0.95rem; color: #0f172a;", "{title}" }
                   p { style: "margin: 0; color: #64748b; font-size: 0.84rem;", "{subtitle}" }
               }
               button {
                   r#type: "button",
                   style: "border: 1px solid #cbd5e1; border-radius: 999px; background: white; color: #334155; font-size: 0.76rem; font-weight: 700; padding: 0.35rem 0.65rem; cursor: pointer;",
                   onclick: move |_| {
                       #[cfg(target_arch = "wasm32")]
                       download_svg(&download_markup, &title);
                   },
                   "Download"
               }
            }
            div { style: "border-radius: 16px; overflow: visible; border: 1px solid #e2e8f0; background: #fcfdff;",
               dangerous_inner_html: svg_markup
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
struct ProgressReporter<'a> {
    last_reported: u64,
    callback: Box<dyn FnMut(u64, u64) + 'a>,
}

#[cfg(target_arch = "wasm32")]
impl<'a> ProgressReporter<'a> {
    fn new<F>(callback: F) -> Self
    where
        F: FnMut(u64, u64) + 'a,
    {
        Self {
            last_reported: 0,
            callback: Box::new(callback),
        }
    }

    fn report_now(&mut self, processed: u64, total: u64) {
        (self.callback)(processed, total);
        self.last_reported = processed;
    }

    fn maybe_report(&mut self, processed: u64, total: u64) -> bool {
        if processed.saturating_sub(self.last_reported) >= PROGRESS_INTERVAL as u64 {
            self.report_now(processed, total);
            true
        } else {
            false
        }
    }
}

#[cfg(target_arch = "wasm32")]
struct BlobLineReader<'a> {
    blob: Blob,
    offset: u64,
    buffer: Vec<u8>,
    buffer_start: usize,
    processed: u64,
    progress: ProgressReporter<'a>,
}

#[cfg(target_arch = "wasm32")]
impl<'a> BlobLineReader<'a> {
    fn new<F>(blob: &Blob, on_progress: F) -> Self
    where
        F: FnMut(u64, u64) + 'a,
    {
        Self {
            blob: blob.clone(),
            offset: 0,
            buffer: Vec::new(),
            buffer_start: 0,
            processed: 0,
            progress: ProgressReporter::new(on_progress),
        }
    }

    fn total_bytes(&self) -> u64 {
        self.blob.size() as u64
    }

    async fn next_line(&mut self) -> Result<Option<String>, ScanError> {
        loop {
            if let Some(line) = self.take_line_from_buffer() {
                return Ok(Some(line));
            }

            if self.offset >= self.total_bytes() {
                if self.buffer_start >= self.buffer.len() {
                    return Ok(None);
                }
                let remaining = String::from_utf8_lossy(&self.buffer[self.buffer_start..]);
                self.buffer_start = self.buffer.len();
                return Ok(Some(remaining.into_owned()));
            }

            self.load_next_chunk().await?;
        }
    }

    fn take_line_from_buffer(&mut self) -> Option<String> {
        let available = &self.buffer[self.buffer_start..];
        if let Some(pos) = available.iter().position(|byte| *byte == b'\n') {
            let line_bytes = &available[..pos];
            let mut line = String::from_utf8_lossy(line_bytes).into_owned();
            self.buffer_start += pos + 1;
            if line.ends_with('\r') {
                line.pop();
            }
            if self.buffer_start > self.buffer.len() / 2 {
                self.buffer.drain(..self.buffer_start);
                self.buffer_start = 0;
            }
            Some(line)
        } else {
            None
        }
    }

    async fn load_next_chunk(&mut self) -> Result<(), ScanError> {
        let start = self.offset;
        let end = (self.offset + CHUNK_SIZE as u64).min(self.total_bytes());
        let start_f64 = start as f64;
        let end_f64 = end as f64;
        let chunk = self
            .blob
            .slice_with_f64_and_f64(start_f64, end_f64)
            .map_err(JsValue::from)?;
        let promise = chunk.array_buffer();
        let bytes = JsFuture::from(promise).await?;
        let array = Uint8Array::new(&bytes);
        let chunk_len = array.byte_length() as usize;
        let mut chunk_bytes = vec![0u8; chunk_len];
        array.copy_to(&mut chunk_bytes);
        self.buffer.extend_from_slice(&chunk_bytes);
        self.offset = end;
        self.processed = self.processed.saturating_add((end - start).max(1));
        if self
            .progress
            .maybe_report(self.processed, self.total_bytes())
        {
            TimeoutFuture::new(0).await;
        }
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
async fn scan_blob_with_progress(
    blob: &Blob,
    mut on_progress: impl FnMut(u64, u64),
) -> Result<PrecursorMetrics, ScanError> {
    let mut reader = BlobLineReader::new(blob, move |processed, total| {
        on_progress(processed, total);
    });

    let mut current_state = BlockParseState::default();
    let mut current_is_in_block = false;
    let mut metrics = PrecursorMetrics::default();
    let mut plot_sample = PlotPointSample::default();
    let mut smiles_cache = HashMap::new();
    let mut formula_cache = HashMap::new();
    let mut logged_failures = HashSet::new();

    while let Some(line) = reader.next_line().await? {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed == "BEGIN IONS" {
            current_state = BlockParseState::default();
            current_is_in_block = true;
            continue;
        }

        if !current_is_in_block {
            continue;
        }

        current_state.consume_line(trimmed);

        if trimmed == "END IONS" {
            let mut block_plot_sample = Some(&mut plot_sample);
            if let Some(result) = process_block_state(
                &current_state,
                &mut smiles_cache,
                &mut formula_cache,
                &mut logged_failures,
                &mut block_plot_sample,
            )? {
                metrics = merge_metrics(metrics, result);
            }
            current_state = BlockParseState::default();
            current_is_in_block = false;
        }
    }

    metrics.plot_points = plot_sample.points;
    metrics.plot_point_stream_seen = plot_sample.seen;
    Ok(metrics)
}

fn process_block(
    block_lines: &[String],
    smiles_cache: &mut HashMap<String, Option<f64>>,
    formula_cache: &mut HashMap<String, Option<f64>>,
    logged_failures: &mut HashSet<String>,
    plot_sample: Option<&mut PlotPointSample>,
) -> Result<Option<PrecursorMetrics>, ScanError> {
    let mut state = BlockParseState::default();
    state.consume_block_lines(block_lines);
    let use_external_sample = plot_sample.is_some();
    let mut local_plot_sample = PlotPointSample::default();
    let mut sample_ref = plot_sample;
    if !use_external_sample {
        sample_ref = Some(&mut local_plot_sample);
    }
    let result = process_block_state(
        &state,
        smiles_cache,
        formula_cache,
        logged_failures,
        &mut sample_ref,
    )?;
    Ok(result.map(|mut metrics| {
        if let Some(plot_sample) = sample_ref.as_ref() {
            metrics.plot_points = plot_sample.points.clone();
            metrics.plot_point_stream_seen = plot_sample.seen;
        }
        metrics
    }))
}

fn process_block_state(
    state: &BlockParseState,
    smiles_cache: &mut HashMap<String, Option<f64>>,
    formula_cache: &mut HashMap<String, Option<f64>>,
    logged_failures: &mut HashSet<String>,
    plot_sample: &mut Option<&mut PlotPointSample>,
) -> Result<Option<PrecursorMetrics>, ScanError> {
    let Some(observed_precursor) = state.observed_precursor else {
        return Ok(None);
    };

    let observed_precision = state
        .observed_precursor_raw
        .as_deref()
        .map(decimal_precision)
        .unwrap_or(5);

    let reference_mass = state
        .reference_mass
        .map(|mass| {
            (
                mass,
                state
                    .reference_mass_source
                    .clone()
                    .or_else(|| Some("unknown".to_string())),
            )
        })
        .or_else(|| {
            state
                .formula
                .as_deref()
                .and_then(|value| {
                    exact_mass_from_formula_cached(value, formula_cache, logged_failures)
                })
                .map(|mass| (mass, Some("FORMULA".to_string())))
        })
        .or_else(|| {
            let parsed_smiles = state.smiles.as_deref().and_then(|value| {
                exact_mass_from_smiles_cached(value, smiles_cache, logged_failures)
            });
            if state.smiles.is_some() && parsed_smiles.is_none() {
                return None;
            }
            parsed_smiles.map(|mass| (mass, Some("SMILES".to_string())))
        });

    let Some((reference_mass, reference_mass_source)) = reference_mass else {
        let mut metrics = PrecursorMetrics::default();
        metrics.total_spectra = 1;
        metrics.skipped_spectra = 1;
        if let Some(smiles_text) = state
            .smiles
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            let trimmed_smiles = smiles_text.trim();
            metrics.unparsed_smiles = 1;
            metrics
                .unparsed_smiles_warnings
                .entry(trimmed_smiles.to_string())
                .and_modify(|detail| detail.count = detail.count.saturating_add(1))
                .or_insert(WarningDetail {
                    count: 1,
                    formula: state.formula.as_deref().map(str::to_string),
                });
            let warning_key = format!(
                "missing-reference-mass:{}|{}",
                trimmed_smiles,
                state.formula.as_deref().unwrap_or("n/a")
            );
            if logged_failures.insert(warning_key) {
                #[cfg(target_arch = "wasm32")]
                console::warn_1(
                    &format!(
                        "Unable to derive reference mass from SMILES/formula for: {trimmed_smiles} (formula: {})",
                        state.formula.as_deref().unwrap_or("n/a")
                    )
                    .into(),
                );
            }
        }
        return Ok(Some(metrics));
    };

    let reference_mass_source = reference_mass_source.unwrap_or_else(|| "unknown".to_string());
    let reference_mass_label = state.adduct.as_deref().map_or_else(
        || reference_mass_source.clone(),
        |adduct| format!("{reference_mass_source} + {adduct}"),
    );
    let adduct_label = normalize_adduct_label(state.adduct.as_deref().unwrap_or("unknown"));
    let adduct_text = state.adduct.as_deref().unwrap_or("").trim();
    let adduct_is_excluded = is_excluded_adduct(adduct_text);
    let adduct_is_supported = adduct_text.is_empty() || is_supported_adduct(adduct_text);
    if adduct_is_excluded || !adduct_is_supported && !adduct_text.is_empty() {
        let mut metrics = PrecursorMetrics::default();
        metrics.total_spectra = 1;
        metrics.skipped_spectra = 1;
        if !adduct_is_excluded && !adduct_text.is_empty() {
            metrics
                .unrecognized_adducts
                .entry(adduct_text.to_string())
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
        return Ok(Some(metrics));
    }

    let expected_precursor_mz = expected_precursor_mz(
        reference_mass,
        state.adduct.as_deref(),
        state.charge.as_deref(),
        state.ion_mode.as_deref(),
    )
    .unwrap_or(reference_mass);
    let expected_precursor_mz = round_to_precision(expected_precursor_mz, observed_precision);
    let error_da = observed_precursor - expected_precursor_mz;
    let abs_error_da = error_da.abs();
    let error_mda = abs_error_da * 1000.0;
    let ppm = if expected_precursor_mz.abs() > f64::EPSILON {
        error_da / expected_precursor_mz * 1_000_000.0
    } else {
        f64::NAN
    };
    let abs_ppm = ppm.abs();

    let mut metrics = PrecursorMetrics::default();
    metrics.total_spectra = 1;
    metrics.spectra = 1;
    metrics.spectra_with_reference_mass = 1;
    metrics.reference_mass_source = reference_mass_label;
    metrics.observed_precursor_min = observed_precursor;
    metrics.observed_precursor_max = observed_precursor;
    metrics.observed_precursor_mean = observed_precursor;
    metrics.abs_error_da_min = abs_error_da;
    metrics.abs_error_da_max = abs_error_da;
    metrics.abs_error_da_mean = abs_error_da;
    metrics.abs_error_da_rms = abs_error_da;
    metrics.abs_error_ppm_min = abs_ppm;
    metrics.abs_error_ppm_max = abs_ppm;
    metrics.abs_error_ppm_mean = abs_ppm;
    metrics.abs_error_ppm_rms = abs_ppm;
    metrics.signed_error_da_mean = error_da;
    metrics.signed_error_ppm_mean = ppm;
    metrics.record_error_with_plot_sample(
        abs_error_da,
        abs_ppm,
        &adduct_label,
        ppm,
        error_da,
        observed_precursor,
        state.smiles.as_deref(),
        Some(reference_mass),
        Some(expected_precursor_mz),
        state.formula.as_deref(),
        plot_sample,
    );
    if error_mda <= 0.1 {
        metrics.within_0_1_da = 1;
    } else if error_mda <= 0.5 {
        metrics.within_0_5_da = 1;
    } else if error_mda <= 1.0 {
        metrics.within_1_da = 1;
    } else if error_mda <= 5.0 {
        metrics.within_5_da = 1;
    } else {
        metrics.above_5_da = 1;
    }
    if abs_ppm <= 0.5 {
        metrics.within_0_5_ppm = 1;
    } else if abs_ppm <= 1.0 {
        metrics.within_1_ppm = 1;
    } else if abs_ppm <= 5.0 {
        metrics.within_5_ppm = 1;
    } else if abs_ppm <= 10.0 {
        metrics.within_10_ppm = 1;
    } else {
        metrics.above_10_ppm = 1;
    }

    Ok(Some(metrics))
}

#[cfg(test)]
mod tests {
    use super::{
        ELECTRON_MASS, PROTON_MASS, SODIUM_MASS, exact_mass_from_smiles, expected_precursor_mz,
    };

    #[test]
    fn computes_exact_mass_from_smiles() {
        let mass = exact_mass_from_smiles("CCO").expect("valid SMILES should parse");
        assert!((mass - 46.041_864_814).abs() < 1e-4);
    }

    #[test]
    fn parses_chiral_smiles_when_the_upstream_parser_supports_them() {
        let mass = exact_mass_from_smiles(
            "C#C[C@]1(O)C=C[C@H]2[C@@H]3CCC4=CC(=O)CC[C@@H]4[C@H]3CC[C@@]21CC",
        );
        assert!(mass.is_some());
        assert!((mass.unwrap() - 310.193_280_077_12).abs() < 1e-6);
    }

    #[test]
    fn buckets_absolute_errors_in_mda() {
        let mut metrics = super::PrecursorMetrics::default();
        metrics.record_error(
            0.00005, 0.2, "[M+H]+", 0.2, 0.00005, 100.0, None, None, None, None,
        );
        assert_eq!(metrics.within_0_1_da, 1);
        assert_eq!(metrics.within_0_5_da, 0);
        assert_eq!(metrics.within_1_da, 0);
        assert_eq!(metrics.within_5_da, 0);
        assert_eq!(metrics.above_5_da, 0);
    }

    #[test]
    fn records_plot_points_when_an_external_sample_is_provided() {
        let block = vec![
            "BEGIN IONS".to_string(),
            "FILENAME=test.mzML".to_string(),
            "FORMULA=C10H12N2O".to_string(),
            "SMILES=CC(=O)N1CCCCC1".to_string(),
            "CHARGE=1+".to_string(),
            "IONMODE=positive".to_string(),
            "ADDUCT=[M+H]+".to_string(),
            "EXACTMASS=164.095".to_string(),
            "PRECURSOR_MZ=165.102".to_string(),
            "END IONS".to_string(),
        ];

        let mut smiles_cache = std::collections::HashMap::new();
        let mut formula_cache = std::collections::HashMap::new();
        let mut logged_failures = std::collections::HashSet::new();
        let mut sample = super::PlotPointSample::default();
        let metrics = super::process_block(
            &block,
            &mut smiles_cache,
            &mut formula_cache,
            &mut logged_failures,
            Some(&mut sample),
        )
        .expect("block should be processed")
        .expect("block should produce metrics");

        assert_eq!(metrics.plot_points.len(), 1);
        assert!(metrics.plot_points[0].observed_precursor_mz > 0.0);
    }

    #[test]
    fn converts_display_errors_to_the_requested_units() {
        assert!((super::display_error_value(0.001, "mDa") - 1.0).abs() < 1e-9);
        assert!((super::display_error_value(2.5, "ppm") - 2.5).abs() < 1e-9);
    }

    #[test]
    fn builds_ecdf_points_from_sorted_values() {
        let points = super::build_ecdf_points(&[0.1, 0.2, 0.2, 0.4], 0.001, 1.0);
        assert!((points[0].0 - 0.001).abs() < 1e-9);
        assert!((points[0].1 - 0.0).abs() < 1e-9);
        assert!((points[1].0 - 0.1).abs() < 1e-9);
        assert!((points[1].1 - 0.0).abs() < 1e-9);
        assert!((points[2].0 - 0.1).abs() < 1e-9);
        assert!((points[2].1 - 0.25).abs() < 1e-9);
        assert!((points.last().unwrap().0 - 1.0).abs() < 1e-9);
        assert!((points.last().unwrap().1 - 1.0).abs() < 1e-9);
    }

    #[test]
    fn handles_double_protonated_adducts() {
        let mass = expected_precursor_mz(1000.0, Some("[M+2H]2+"), Some("2+"), Some("positive"))
            .expect("double protonated adduct should be supported");
        assert!((mass - (500.0 + PROTON_MASS)).abs() < 1e-9);
    }

    #[test]
    fn respects_negative_charge_signs() {
        let mass = expected_precursor_mz(1000.0, None, Some("-1"), None)
            .expect("negative charge should be supported");
        assert!((mass - (1000.0 - PROTON_MASS)).abs() < 1e-9);
    }

    #[test]
    fn supports_quaternary_multiplicity_adducts() {
        let spec =
            super::parse_adduct_mass_spec("[4M-H]-").expect("4M-H adduct should be supported");
        assert_eq!(spec, (4.0, -super::HYDROGEN_MASS));
    }

    #[test]
    fn handles_complex_mg_and_meoh_adducts_from_multims2_blocks() {
        let neutral = 343.141_97;
        let mass = expected_precursor_mz(
            neutral,
            Some("[M-C2H4-H+Mg]+"),
            Some("1+"),
            Some("positive"),
        )
        .expect("complex magnesium adduct should be supported");
        assert!((mass - 362.072_38).abs() < 1e-3);

        let neutral = 228.085_85;
        let mass = expected_precursor_mz(
            neutral,
            Some("[2M+MeOH-H+Mg]+"),
            Some("1+"),
            Some("positive"),
        )
        .expect("dimer methanol magnesium adduct should be supported");
        assert!((mass - 535.159_62).abs() < 1e-3);

        let neutral = 292.103_42;
        let mass =
            expected_precursor_mz(neutral, Some("[2M+O+Mg]+2"), Some("2+"), Some("positive"))
                .expect("dimer oxygen magnesium adduct should be supported");
        assert!((mass - 324.085_370_430_090_96).abs() < 5e-4);

        let neutral = 380.165_54;
        let mass = expected_precursor_mz(neutral, Some("[M+H]+"), Some("1+"), Some("positive"))
            .expect("protonated adduct should be supported");
        assert!((mass - 381.172_8).abs() < 5e-4);

        let neutral = 333.939_62;
        let mass = expected_precursor_mz(neutral, Some("[M+O+Mg]+2"), Some("2+"), Some("positive"))
            .expect("oxygen magnesium dication adduct should be supported");
        assert!((mass - 198.952_03).abs() < 5e-4);
    }

    #[test]
    fn uses_ion_mass_for_sodium_adducts() {
        let mass = expected_precursor_mz(1000.0, Some("[M+Na]+"), Some("1+"), Some("positive"))
            .expect("sodium adduct should be supported");
        assert!((mass - (1000.0 + SODIUM_MASS - ELECTRON_MASS)).abs() < 1e-9);
    }

    #[test]
    fn tracks_largest_absolute_error_for_high_error_smiles() {
        let mut metrics = super::PrecursorMetrics::default();
        metrics.record_error(
            5.0,
            12.0,
            "[M+H]+",
            12.0,
            0.0,
            25.0,
            Some("CCO"),
            Some(1.0),
            Some(20.0),
            Some("C2H6O"),
        );
        metrics.record_error(
            10.0,
            12.0,
            "[M+H]+",
            12.0,
            0.0,
            40.0,
            Some("CCO"),
            Some(1.0),
            Some(30.0),
            Some("C2H6O"),
        );

        let detail = metrics
            .high_error_smiles
            .get("CCO")
            .expect("entry should exist");
        assert_eq!(detail.count, 2);
        assert_eq!(detail.max_abs_error_da, Some(10.0));
        assert_eq!(detail.max_abs_error_ppm, Some(12.0));
        assert_eq!(detail.expected_mass, Some(30.0));
    }

    #[test]
    fn tracks_above_threshold_counts_and_da_filtered_high_error_smiles() {
        let mut metrics = super::PrecursorMetrics::default();
        metrics.record_error(
            0.009,
            11.0,
            "[M+H]+",
            11.0,
            0.0,
            10.0,
            Some("CCO"),
            Some(1.0),
            Some(20.0),
            Some("C2H6O"),
        );
        assert_eq!(metrics.above_5_da, 1);
        assert_eq!(metrics.above_10_ppm, 1);
        assert!(metrics.high_error_smiles.is_empty());

        metrics.record_error(
            0.011,
            11.0,
            "[M+H]+",
            11.0,
            0.0,
            10.0,
            Some("CCO"),
            Some(1.0),
            Some(20.0),
            Some("C2H6O"),
        );
        assert!(metrics.high_error_smiles.contains_key("CCO"));
    }

    #[test]
    fn merges_median_trackers_across_chunks() {
        let mut left = super::MedianTracker::default();
        left.push(1.0);
        left.push(10.0);

        let mut right = super::MedianTracker::default();
        right.push(3.0);
        right.push(4.0);
        right.push(100.0);

        left.merge(right);
        assert!((left.median() - 4.0).abs() < 1e-9);
    }

    #[test]
    fn computes_median_values_from_recorded_errors() {
        let mut metrics = super::PrecursorMetrics::default();
        metrics.record_error(
            5.0,
            6.0,
            "[M+H]+",
            6.0,
            -2.0,
            10.0,
            Some("CCO"),
            Some(1.0),
            Some(2.0),
            Some("C2H6O"),
        );
        metrics.record_error(
            15.0,
            24.0,
            "[M+H]+",
            24.0,
            2.0,
            30.0,
            Some("CCO"),
            Some(1.0),
            Some(2.0),
            Some("C2H6O"),
        );
        metrics.record_error(
            25.0,
            42.0,
            "[M+H]+",
            42.0,
            4.0,
            50.0,
            Some("CCO"),
            Some(1.0),
            Some(2.0),
            Some("C2H6O"),
        );

        assert!((metrics.abs_error_da_median - 15.0).abs() < 1e-9);
        assert!((metrics.abs_error_ppm_median - 24.0).abs() < 1e-9);
        assert!((metrics.observed_precursor_median - 30.0).abs() < 1e-9);
    }

    #[test]
    fn supports_mg_oxygen_dication_adducts() {
        let mass = expected_precursor_mz(
            292.103_42,
            Some("[2M+O+Mg]+2"),
            Some("2+"),
            Some("positive"),
        )
        .expect("dimer oxygen magnesium dication adduct should be supported");
        assert!((mass - 324.085_370_430_090_96).abs() < 5e-4);

        let mass =
            expected_precursor_mz(333.939_62, Some("[M+O+Mg]+2"), Some("2+"), Some("positive"))
                .expect("oxygen magnesium dication adduct should be supported");
        assert!((mass - 198.952_03).abs() < 5e-4);
    }

    #[test]
    fn processes_mgf_blocks_with_mg_oxygen_adducts() {
        let block = vec![
            "BEGIN IONS".to_string(),
            "FILENAME=test.mzML".to_string(),
            "FORMULA=C18H16N2S".to_string(),
            "SMILES=c1ccc(-c2csc(N3CCc4ccccc4C3)n2)cc1".to_string(),
            "CHARGE=2+".to_string(),
            "IONMODE=positive".to_string(),
            "ADDUCT=[2M+O+Mg]+2".to_string(),
            "EXACTMASS=292.10342".to_string(),
            "PRECURSOR_MZ=324.08564".to_string(),
            "END IONS".to_string(),
        ];

        let mut smiles_cache = std::collections::HashMap::new();
        let mut formula_cache = std::collections::HashMap::new();
        let mut logged_failures = std::collections::HashSet::new();
        let metrics = super::process_block(
            &block,
            &mut smiles_cache,
            &mut formula_cache,
            &mut logged_failures,
            None,
        )
        .expect("block should be processed")
        .expect("block should produce metrics");
        assert_eq!(metrics.spectra, 1);
        assert!(metrics.spectra_with_reference_mass > 0);
        assert!(metrics.abs_error_da_max < 5e-6);
        assert!(metrics.abs_error_ppm_max < 0.02);

        let block = vec![
            "BEGIN IONS".to_string(),
            "FILENAME=test.mzML".to_string(),
            "FORMULA=C15H8BrClO2".to_string(),
            "SMILES=O=c1cc(-c2ccc(Br)cc2)oc2ccc(Cl)cc12".to_string(),
            "CHARGE=2+".to_string(),
            "IONMODE=positive".to_string(),
            "ADDUCT=[M+O+Mg]+2".to_string(),
            "EXACTMASS=333.93962".to_string(),
            "PRECURSOR_MZ=198.95203".to_string(),
            "END IONS".to_string(),
        ];

        let mut smiles_cache = std::collections::HashMap::new();
        let mut formula_cache = std::collections::HashMap::new();
        let mut logged_failures = std::collections::HashSet::new();
        let metrics = super::process_block(
            &block,
            &mut smiles_cache,
            &mut formula_cache,
            &mut logged_failures,
            None,
        )
        .expect("block should be processed")
        .expect("block should produce metrics");
        assert_eq!(metrics.spectra, 1);
        assert!(metrics.spectra_with_reference_mass > 0);
        assert!(metrics.abs_error_da_max < 5e-6);
        assert!(metrics.abs_error_ppm_max < 0.02);
    }

    #[test]
    fn normalizes_bracketed_adducts() {
        assert_eq!(super::normalize_adduct_label("[M]+"), "[M]+");
        assert_eq!(super::normalize_adduct_label("[M]++"), "[M]2+");
        assert_eq!(super::normalize_adduct_label("[M+H]+"), "[M+H]+");
        assert_eq!(super::normalize_adduct_label("[M+2H]+"), "[M+2H]2+");
        assert_eq!(super::normalize_adduct_label("[M-H]1-"), "[M-H]-");
        assert_eq!(super::normalize_adduct_label("[M-2H]--"), "[M-2H]2-");
    }

    #[test]
    fn supports_common_adduct_families() {
        let mass = expected_precursor_mz(1000.0, Some("[M+Cl]-"), Some("1-"), Some("negative"))
            .expect("chloride adduct should be supported");
        assert!(mass > 1000.0);

        let methanol_mass =
            expected_precursor_mz(1000.0, Some("[M+MeOH+H]+"), Some("1+"), Some("positive"))
                .expect("methanol adduct should be supported");
        assert!(methanol_mass > 1000.0);

        let water_mass =
            expected_precursor_mz(1000.0, Some("[M+H2O+H]+"), Some("1+"), Some("positive"))
                .expect("water adduct should be supported");
        assert!(water_mass > 1000.0);

        let dimer_mass =
            expected_precursor_mz(1000.0, Some("[2M+H]+"), Some("1+"), Some("positive"))
                .expect("dimer adduct should be supported");
        assert!((dimer_mass - (2000.0 + PROTON_MASS)).abs() < 1e-9);
    }

    #[test]
    fn rejects_uncommon_formula_adducts() {
        assert!(super::parse_adduct_mass_spec("[M+ZZZ]+").is_none());
    }

    #[test]
    fn supports_multi_part_formula_adducts() {
        let mass = super::expected_precursor_mz(
            1000.0,
            Some("[M+H2O+CH3OH+H]+"),
            Some("1+"),
            Some("positive"),
        )
        .expect("multi-part formula adduct should be supported");
        assert!(mass > 1000.0);
    }

    #[test]
    fn supports_iron_hydride_dimer_adducts() {
        assert!(super::is_supported_adduct("[2M+HFA+Fe-H]+"));
        let mass = super::expected_precursor_mz(
            1000.0,
            Some("[2M+HFA+Fe-H]+"),
            Some("1+"),
            Some("positive"),
        )
        .expect("iron hydride dimer adduct should be supported");
        assert!(mass > 2000.0);
    }

    #[test]
    fn uses_formate_like_mass_for_hfa_adducts() {
        let mass = super::expected_precursor_mz(
            202.131_74,
            Some("[M+HFA+Ca-H]+"),
            Some("1+"),
            Some("positive"),
        )
        .expect("HFA calcium-hydride adduct should be supported");
        assert!((mass - 287.091_44).abs() < 1e-5);
    }

    #[test]
    fn handles_hydrogen_minus_terms_for_metal_adducts() {
        let protonated_mass =
            super::expected_precursor_mz(1000.0, Some("[M+H]+"), Some("1+"), Some("positive"))
                .expect("protonated adduct should be supported");
        assert!((protonated_mass - (1000.0 + PROTON_MASS)).abs() < 1e-9);

        let calcium_hydride_mass = super::expected_precursor_mz(
            1000.0,
            Some("[M+HFA+Ca-H]+"),
            Some("1+"),
            Some("positive"),
        )
        .expect("calcium-hydride adduct should be supported");
        assert!(calcium_hydride_mass > 1000.0);

        let magnesium_hydride_mass = super::expected_precursor_mz(
            344.119_46,
            Some("[M+HFA-H+Mg]+"),
            Some("1+"),
            Some("positive"),
        )
        .expect("magnesium hydride adduct should be supported");
        assert!((magnesium_hydride_mass - 437.086_65).abs() < 1e-3);
    }

    #[test]
    fn supports_c2h4_and_sodium_formate_terms() {
        let c2h4_loss =
            super::expected_precursor_mz(1000.0, Some("[M-C2H4+H]+"), Some("1+"), Some("positive"))
                .expect("C2H4 loss should be supported");
        assert!((c2h4_loss - (1000.0 - 28.031_300_128 + PROTON_MASS)).abs() < 1e-9);

        let sodium_formate = super::expected_precursor_mz(
            1000.0,
            Some("[M+CHNaO2+H]+"),
            Some("1+"),
            Some("positive"),
        )
        .expect("sodium formate should be supported");
        let sodium_formate_mass =
            super::exact_mass_from_formula("CHNaO2").expect("CHNaO2 mass should be available");
        assert!((sodium_formate - (1000.0 + sodium_formate_mass + PROTON_MASS)).abs() < 1e-9);

        let formate_adduct =
            super::expected_precursor_mz(1000.0, Some("[M+FA]-"), Some("1-"), Some("negative"))
                .expect("formate adduct should be supported");
        let expected_formate_mass = 1000.0
            + super::exact_mass_from_formula("CHO2").expect("CHO2 mass should be available")
            + super::ELECTRON_MASS;
        assert!((formate_adduct - expected_formate_mass).abs() < 1e-9);

        let sodium_formate_alias =
            super::expected_precursor_mz(1000.0, Some("[M+NaHCOO]+"), Some("1+"), Some("positive"))
                .expect("sodium formate alias should be supported");
        let sodium_formate_alias_mass =
            super::exact_mass_from_formula("CHNaO2").expect("CHNaO2 mass should be available");
        assert!(
            (sodium_formate_alias - (1000.0 + sodium_formate_alias_mass - ELECTRON_MASS)).abs()
                < 1e-9
        );
    }
}

#[cfg(target_arch = "wasm32")]
fn merge_metrics(mut current: PrecursorMetrics, next: PrecursorMetrics) -> PrecursorMetrics {
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
    current
        .absolute_error_da_values
        .reserve(next.absolute_error_da_values.len());
    current
        .absolute_error_ppm_values
        .reserve(next.absolute_error_ppm_values.len());
    current
        .absolute_error_da_values
        .extend(next.absolute_error_da_values);
    current
        .absolute_error_ppm_values
        .extend(next.absolute_error_ppm_values);

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
