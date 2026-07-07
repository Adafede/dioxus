#![allow(clippy::all)]
#![allow(warnings)]

use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap, HashMap, HashSet};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::str::FromStr;

use dioxus::events::{DragData, FormData, WheelData};
use dioxus::html::HasFileData;
use dioxus::prelude::*;
#[cfg(target_arch = "wasm32")]
use gloo_timers::future::TimeoutFuture;
#[cfg(target_arch = "wasm32")]
use js_sys::Uint8Array;
use mascot_rs::prelude::*;
use molecular_formulas::{MolecularFormula, prelude::ChemicalFormula};
use smiles_parser::chain;
use smiles_parser::graph::{Atom, MoleculeGraph};
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
const PROTON_MASS: f64 = 1.007_276_466_621;
const HYDROGEN_MASS: f64 = PROTON_MASS + ELECTRON_MASS;
const ELECTRON_MASS: f64 = 0.000_548_579_909_065;
const SODIUM_MASS: f64 = 22.989_769_67;
const POTASSIUM_MASS: f64 = 38.963_707;
const AMMONIUM_MASS: f64 = 18.033_823;

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
    signed_error_da: f64,
    signed_error_ppm: f64,
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
    within_0_0001_da: usize,
    within_0_0005_da: usize,
    within_0_001_da: usize,
    within_0_005_da: usize,
    within_0_5_ppm: usize,
    within_1_ppm: usize,
    within_5_ppm: usize,
    within_10_ppm: usize,
    da_error_histogram: HistogramData,
    ppm_error_histogram: HistogramData,
    plot_points: Vec<PlotPoint>,
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
            within_0_0001_da: 0,
            within_0_0005_da: 0,
            within_0_001_da: 0,
            within_0_005_da: 0,
            within_0_5_ppm: 0,
            within_1_ppm: 0,
            within_5_ppm: 0,
            within_10_ppm: 0,
            da_error_histogram: HistogramData::new(48, 0.0, 0.5),
            ppm_error_histogram: HistogramData::new(48, 0.0, 50.0),
            plot_points: Vec::new(),
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
        self.da_error_histogram.add_value(abs_error_da);
        self.ppm_error_histogram.add_value(abs_ppm);
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

        if abs_ppm > 10.0 {
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

        if self.plot_points.len() < 260 {
            self.plot_points.push(PlotPoint {
                adduct_type: adduct_type.to_string(),
                signed_error_da,
                signed_error_ppm: ppm_error,
            });
        } else {
            let len = self.plot_points.len();
            let stride = (self.spectra / 260).max(1);
            if self.spectra % stride == 0 {
                if let Some(point) = self.plot_points.get_mut(len / 2) {
                    *point = PlotPoint {
                        adduct_type: adduct_type.to_string(),
                        signed_error_da,
                        signed_error_ppm: ppm_error,
                    };
                }
            }
        }
    }
}

fn smiles_is_supported(smiles: &str) -> bool {
    let trimmed = smiles.trim();
    if trimmed.is_empty() {
        return false;
    }

    !trimmed.contains(['[', ']', '@', '/', '\\', ':', ';'])
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
        let (rest, parsed_chain) = chain(smiles.as_bytes()).ok()?;
        if !rest.is_empty() {
            return None;
        }

        let graph = MoleculeGraph::from_chain(parsed_chain);
        let mut counts: BTreeMap<&str, u32> = BTreeMap::new();
        for node_index in graph.node_indices() {
            let atom = graph.node_weight(node_index)?;
            let symbol = match atom {
                Atom::AliphaticOrganic(atom) => atom.element.get_symbol(),
                Atom::Element(element) => element.get_symbol(),
            };
            *counts.entry(symbol).or_default() += 1;
        }

        let mut formula_parts = counts.into_iter().collect::<Vec<_>>();
        formula_parts.sort_by(|(left, _), (right, _)| hill_order(left, right));

        let mut formula = String::new();
        for (symbol, count) in formula_parts {
            formula.push_str(symbol);
            if count > 1 {
                formula.push_str(&count.to_string());
            }
        }

        let formula: ChemicalFormula<u32, i32> = ChemicalFormula::from_str(&formula).ok()?;
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
        let (multiplier, shift) = parse_adduct_mass_spec(normalized_adduct).unwrap_or_else(|| {
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
        let electron_adjustment = match charge_sign {
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

fn parse_adduct_mass_spec(adduct: &str) -> Option<(f64, f64)> {
    let normalized = adduct.trim().replace(' ', "").to_ascii_uppercase();
    let body = if let Some(index) = normalized.find(']') {
        normalized[1..index].to_string()
    } else {
        normalized.clone()
    };
    let charge_sign = parse_adduct_charge_sign(Some(adduct));
    let charge_value = parse_charge_value(None, Some(adduct)).unwrap_or(1.0);
    let uses_single_positive_mg = charge_sign == Some(false) && charge_value <= 1.0;
    let mut multiplier = 1.0f64;
    let mut shift = 0.0f64;
    let mut current = String::new();
    let mut sign = 1.0f64;
    let mut saw_unsupported_token = false;

    for ch in body.chars() {
        match ch {
            '+' => {
                if let Some(token_mass) =
                    parse_adduct_term_mass_with_context(&current, sign, uses_single_positive_mg)
                {
                    shift += sign * token_mass;
                } else if current.eq_ignore_ascii_case("M") {
                    multiplier = 1.0;
                } else if current.eq_ignore_ascii_case("2M") {
                    multiplier = 2.0;
                } else if current.eq_ignore_ascii_case("3M") {
                    multiplier = 3.0;
                } else if current.eq_ignore_ascii_case("H") {
                    shift += sign * HYDROGEN_MASS;
                } else if !current.is_empty() {
                    saw_unsupported_token = true;
                }
                current.clear();
                sign = 1.0;
            }
            '-' => {
                if let Some(token_mass) =
                    parse_adduct_term_mass_with_context(&current, sign, uses_single_positive_mg)
                {
                    shift += sign * token_mass;
                } else if current.eq_ignore_ascii_case("M") {
                    multiplier = 1.0;
                } else if current.eq_ignore_ascii_case("2M") {
                    multiplier = 2.0;
                } else if current.eq_ignore_ascii_case("3M") {
                    multiplier = 3.0;
                } else if current.eq_ignore_ascii_case("H") {
                    shift += sign * HYDROGEN_MASS;
                } else if !current.is_empty() {
                    saw_unsupported_token = true;
                }
                current.clear();
                sign = -1.0;
            }
            _ => current.push(ch),
        }
    }

    if let Some(token_mass) =
        parse_adduct_term_mass_with_context(&current, sign, uses_single_positive_mg)
    {
        shift += sign * token_mass;
    } else if current.eq_ignore_ascii_case("M") {
        multiplier = 1.0;
    } else if current.eq_ignore_ascii_case("2M") {
        multiplier = 2.0;
    } else if current.eq_ignore_ascii_case("3M") {
        multiplier = 3.0;
    } else if current.eq_ignore_ascii_case("H") {
        shift += sign * HYDROGEN_MASS;
    } else if !current.is_empty() {
        saw_unsupported_token = true;
    }

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

fn parse_adduct_shift(adduct: &str) -> Option<f64> {
    parse_adduct_mass_spec(adduct).map(|(_, shift)| shift)
}

fn parse_adduct_term_mass(token: &str) -> Option<f64> {
    parse_adduct_term_mass_with_context(token, 1.0, false)
}

fn parse_adduct_term_mass_with_context(
    token: &str,
    sign: f64,
    uses_single_positive_mg: bool,
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
            if sign > 0.0 && uses_single_positive_mg {
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

        let mut status_for_progress = status;
        let mut metrics_for_results = metrics;

        spawn(async move {
            #[cfg(target_arch = "wasm32")]
            {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let total_bytes = blob.size() as u64;
                status_for_progress.set(format!("Scanning {total_bytes} bytes..."));
                let result = match scan_blob_with_progress(&blob, move |processed, total| {
                    let safe_total = total.max(1);
                    let displayed_processed = processed.min(safe_total);
                    let percent = (displayed_processed * 100 / safe_total).min(100);
                    status_for_progress.set(format!(
                        "Scanning {displayed_processed}/{safe_total} bytes ({percent}%)..."
                    ));
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
                busy.set(false);
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                status_for_progress.set("This app needs to run in a browser.".to_string());
                busy.set(false);
            }
        });
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

        let mut status_for_progress = status;
        let mut metrics_for_results = metrics;

        spawn(async move {
            #[cfg(target_arch = "wasm32")]
            {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let total_bytes = blob.size() as u64;
                status_for_progress.set(format!("Scanning {total_bytes} bytes..."));
                let result = match scan_blob_with_progress(&blob, move |processed, total| {
                    let safe_total = total.max(1);
                    let displayed_processed = processed.min(safe_total);
                    let percent = (displayed_processed * 100 / safe_total).min(100);
                    status_for_progress.set(format!(
                        "Scanning {displayed_processed}/{safe_total} bytes ({percent}%)..."
                    ));
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
                busy.set(false);
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                status_for_progress.set("This app needs to run in a browser.".to_string());
                busy.set(false);
            }
        });
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
                                    p { style: "margin: 0 0 0.35rem; font-weight: 700;", "SMILES for spectra above 10 ppm" }
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
                                div { style: "background: white; padding: 0.8rem; border-radius: 12px; border: 1px solid #e2e8f0; box-shadow: 0 4px 12px rgba(15, 23, 42, 0.04);",
                                    h4 { style: "margin: 0 0 0.45rem; font-size: 0.95rem; color: #0f172a;", "Observed precursor m/z" }
                                    ul { style: "padding-left: 1.1rem; margin: 0.25rem 0 0; color: #475569;",
                                        li { "min: {format_value(metrics.observed_precursor_min)}" }
                                        li { "max: {format_value(metrics.observed_precursor_max)}" }
                                        li { "median: {format_value(metrics.observed_precursor_median)}" }
                                        li { "mean: {format_value(metrics.observed_precursor_mean)}" }
                                    }
                                }
                                div { style: "background: white; padding: 0.8rem; border-radius: 12px; border: 1px solid #e2e8f0; box-shadow: 0 4px 12px rgba(15, 23, 42, 0.04);",
                                    h4 { style: "margin: 0 0 0.45rem; font-size: 0.95rem; color: #0f172a;", "Absolute error (Da)" }
                                    ul { style: "padding-left: 1.1rem; margin: 0.25rem 0 0; color: #475569;",
                                        li { "min: {format_value(metrics.abs_error_da_min)}" }
                                        li { "median: {format_value(metrics.abs_error_da_median)}" }
                                        li { "mean: {format_value(metrics.abs_error_da_mean)}" }
                                        li { "RMS: {format_value(metrics.abs_error_da_rms)}" }
                                        li { "max: {format_value(metrics.abs_error_da_max)}" }
                                    }
                                }
                                div { style: "background: white; padding: 0.8rem; border-radius: 12px; border: 1px solid #e2e8f0; box-shadow: 0 4px 12px rgba(15, 23, 42, 0.04);",
                                    h4 { style: "margin: 0 0 0.45rem; font-size: 0.95rem; color: #0f172a;", "Absolute error (ppm)" }
                                    ul { style: "padding-left: 1.1rem; margin: 0.25rem 0 0; color: #475569;",
                                        li { "min: {format_value(metrics.abs_error_ppm_min)}" }
                                        li { "median: {format_value(metrics.abs_error_ppm_median)}" }
                                        li { "mean: {format_value(metrics.abs_error_ppm_mean)}" }
                                        li { "RMS: {format_value(metrics.abs_error_ppm_rms)}" }
                                        li { "max: {format_value(metrics.abs_error_ppm_max)}" }
                                    }
                                }
                                div { style: "background: white; padding: 0.8rem; border-radius: 12px; border: 1px solid #e2e8f0; box-shadow: 0 4px 12px rgba(15, 23, 42, 0.04);",
                                    h4 { style: "margin: 0 0 0.45rem; font-size: 0.95rem; color: #0f172a;", "Signed mean" }
                                    ul { style: "padding-left: 1.1rem; margin: 0.25rem 0 0; color: #475569;",
                                        li { "Da median: {format_value(metrics.signed_error_da_median)}" }
                                        li { "Da mean: {format_value(metrics.signed_error_da_mean)}" }
                                        li { "ppm median: {format_value(metrics.signed_error_ppm_median)}" }
                                        li { "ppm mean: {format_value(metrics.signed_error_ppm_mean)}" }
                                    }
                                }
                            }

                            div {
                                style: "margin-top: 1rem; display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 1rem;",
                                histogram_plot {
                                    title: "Absolute precursor error (Da)".to_string(),
                                    subtitle: "Distribution across spectra, with tighter tolerance bands highlighted".to_string(),
                                    histogram: metrics.da_error_histogram.clone(),
                                    thresholds: vec![0.0001, 0.0005, 0.005],
                                    unit: "Da".to_string(),
                                }
                                histogram_plot {
                                    title: "Absolute precursor error (ppm)".to_string(),
                                    subtitle: "Distribution across spectra, with tighter ppm tolerance bands".to_string(),
                                    histogram: metrics.ppm_error_histogram.clone(),
                                    thresholds: vec![0.5, 1.0, 5.0],
                                    unit: "ppm".to_string(),
                                }
                                scatter_plot {
                                    title: "Error versus adduct type".to_string(),
                                    subtitle: "Signed Da error by adduct family".to_string(),
                                    points: metrics.plot_points.clone(),
                                    other_label: if metrics.skipped_spectra > 0 {
                                        Some(format!("Other ({})", metrics.skipped_spectra))
                                    } else {
                                        None
                                    },
                                }
                            }

                            div {
                                style: "margin-top: 1rem; display: flex; flex-wrap: wrap; gap: 0.55rem;",
                                span { style: "display: inline-block; padding: 0.4rem 0.7rem; background: #f0fdf4; color: #166534; border: 1px solid #86efac; border-radius: 999px; font-weight: 700; box-shadow: 0 1px 2px rgba(22, 101, 52, 0.12);",
                                    "≤ 0.0001 Da: {format_count_with_percentage(metrics.within_0_0001_da, metrics.spectra)}"
                                }
                                span { style: "display: inline-block; padding: 0.4rem 0.7rem; background: #ecfdf3; color: #15803d; border: 1px solid #4ade80; border-radius: 999px; font-weight: 700; box-shadow: 0 1px 2px rgba(21, 128, 61, 0.12);",
                                    "0.0001–0.0005 Da: {format_count_with_percentage(metrics.within_0_0005_da, metrics.spectra)}"
                                }
                                span { style: "display: inline-block; padding: 0.4rem 0.7rem; background: #fef3c7; color: #92400e; border: 1px solid #fde68a; border-radius: 999px; font-weight: 700; box-shadow: 0 1px 2px rgba(146, 64, 14, 0.12);",
                                    "0.0005–0.001 Da: {format_count_with_percentage(metrics.within_0_001_da, metrics.spectra)}"
                                }
                                span { style: "display: inline-block; padding: 0.4rem 0.7rem; background: #ffedd5; color: #9a2c00; border: 1px solid #fdba74; border-radius: 999px; font-weight: 700; box-shadow: 0 1px 2px rgba(154, 44, 0, 0.12);",
                                    "0.001–0.005 Da: {format_count_with_percentage(metrics.within_0_005_da, metrics.spectra)}"
                                }
                                span { style: "display: inline-block; padding: 0.4rem 0.7rem; background: #ecfdf3; color: #166534; border: 1px solid #86efac; border-radius: 999px; font-weight: 700; box-shadow: 0 1px 2px rgba(22, 101, 52, 0.12);",
                                    "≤ 0.5 ppm: {format_count_with_percentage(metrics.within_0_5_ppm, metrics.spectra)}"
                                }
                                span { style: "display: inline-block; padding: 0.4rem 0.7rem; background: #fef3c7; color: #92400e; border: 1px solid #fde68a; border-radius: 999px; font-weight: 700; box-shadow: 0 1px 2px rgba(146, 64, 14, 0.12);",
                                    "0.5–1 ppm: {format_count_with_percentage(metrics.within_1_ppm, metrics.spectra)}"
                                }
                                span { style: "display: inline-block; padding: 0.4rem 0.7rem; background: #ffedd5; color: #9a2c00; border: 1px solid #fdba74; border-radius: 999px; font-weight: 700; box-shadow: 0 1px 2px rgba(154, 44, 0, 0.12);",
                                    "1–5 ppm: {format_count_with_percentage(metrics.within_5_ppm, metrics.spectra)}"
                                }
                                span { style: "display: inline-block; padding: 0.4rem 0.7rem; background: #fee2e2; color: #b91c1c; border: 1px solid #fda4af; border-radius: 999px; font-weight: 700; box-shadow: 0 1px 2px rgba(185, 28, 28, 0.12);",
                                    "5–10 ppm: {format_count_with_percentage(metrics.within_10_ppm, metrics.spectra)}"
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
    Some(AdductClass {
        label: normalized.clone(),
        display: normalized,
        charge,
    })
}

fn paul_tol_palette(index: usize) -> &'static str {
    [
        "#4477AA", "#66CCEE", "#228833", "#CCBB44", "#EE6677", "#AA3377", "#BBBBBB", "#004488",
    ][index % 8]
}

fn tolerance_step_color(index: usize, total_steps: usize) -> &'static str {
    let palette = ["#16A34A", "#4ADE80", "#F59E0B", "#EA580C"];
    if total_steps <= 1 {
        return palette[0];
    }
    let normalized = index.min(total_steps.saturating_sub(1));
    let slot = ((normalized as f64 / (total_steps - 1) as f64) * (palette.len() - 1) as f64).round()
        as usize;
    palette[slot.min(palette.len() - 1)]
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

#[component]
fn histogram_plot(
    title: String,
    subtitle: String,
    histogram: HistogramData,
    thresholds: Vec<f64>,
    unit: String,
) -> Element {
    let mut zoom = use_signal(|| 1.0f64);
    let max_count = histogram.bins.iter().copied().max().unwrap_or(1).max(1);
    let width = 360.0f64;
    let height = 220.0f64;
    let padding = 24.0f64;
    let plot_width = width - padding * 2.0;
    let plot_height = height - padding * 2.0;
    let zoom_in = move |_| {
        let current_zoom = *zoom.read();
        zoom.set((current_zoom * 1.18).min(8.0));
    };
    let zoom_out = move |_| {
        let current_zoom = *zoom.read();
        zoom.set((current_zoom / 1.18).max(0.8));
    };
    let reset_zoom = move |_| zoom.set(1.0);
    let zoom_transform = format!(
        "translate({} {}) scale({}) translate({} {})",
        width / 2.0,
        height / 2.0,
        *zoom.read(),
        -(width / 2.0),
        -(height / 2.0)
    );
    let bars = histogram
        .bins
        .iter()
        .enumerate()
        .map(|(index, count)| {
            let bar_width = plot_width / histogram.bins.len() as f64 - 2.0;
            let x = padding + (index as f64 * (plot_width / histogram.bins.len() as f64));
            let step = (histogram.max - histogram.min).max(1e-9) / histogram.bins.len() as f64;
            let bin_center = histogram.min + (index as f64 + 0.5) * step;
            let bar_height = if *count == 0 {
                0.0
            } else {
                (*count as f64 / max_count as f64) * plot_height
            };
            let y = height - padding - bar_height;
            let color = paul_tol_palette(index);
            rsx! {
                rect {
                    x: x as i32,
                    y: y as i32,
                    width: bar_width as i32,
                    height: bar_height as i32,
                    fill: color,
                    opacity: "0.95"
                }
            }
        })
        .collect::<Vec<_>>();

    let threshold_lines = thresholds
        .iter()
        .enumerate()
        .map(|(index, threshold)| {
            let normalized = ((threshold - histogram.min)
                / (histogram.max - histogram.min).max(1e-9))
            .clamp(0.0, 1.0);
            let x = padding + normalized * plot_width;
            rsx! {
                line {
                    x1: x as i32,
                    y1: padding as i32,
                    x2: x as i32,
                    y2: (height - padding) as i32,
                    stroke: tolerance_step_color(index, thresholds.len()),
                    stroke_width: "1.25",
                    stroke_dasharray: "4 3"
                }
            }
        })
        .collect::<Vec<_>>();

    let legend_items = thresholds
        .iter()
        .enumerate()
        .map(|(index, threshold)| {
            let label = format!("≤ {} {unit}", format_threshold_value(*threshold));
            rsx! {
                div { style: "display: flex; align-items: center; gap: 0.35rem; font-size: 0.75rem; color: #475569;",
                    span { style: format!("display:inline-block; width:10px; height:10px; border-radius:999px; background:{};", tolerance_step_color(index, thresholds.len())) }
                    span { "{label}" }
                }
            }
        })
        .collect::<Vec<_>>();

    rsx! {
        div {
            style: "padding: 0.9rem; border: 1px solid #e2e8f0; border-radius: 16px; background: linear-gradient(180deg, #ffffff 0%, #f8fafc 100%);",
            h4 { style: "margin: 0 0 0.2rem; font-size: 0.95rem; color: #0f172a;", "{title}" }
            p { style: "margin: 0 0 0.65rem; color: #64748b; font-size: 0.84rem;", "{subtitle}" }
            div { style: "display: flex; justify-content: flex-end; gap: 0.3rem; margin-bottom: 0.45rem;",
               button { type: "button", style: "border: 1px solid #cbd5e1; background: white; border-radius: 999px; width: 28px; height: 28px; cursor: pointer; font-size: 0.95rem;", onclick: zoom_in, "−" }
               button { type: "button", style: "border: 1px solid #cbd5e1; background: white; border-radius: 999px; width: 28px; height: 28px; cursor: pointer; font-size: 0.95rem;", onclick: zoom_out, "＋" }
               button { type: "button", style: "border: 1px solid #cbd5e1; background: white; border-radius: 999px; padding: 0 0.55rem; height: 28px; cursor: pointer; font-size: 0.8rem; color: #475569;", onclick: reset_zoom, "Reset" }
            }
            div { style: "display: flex; flex-wrap: wrap; gap: 0.65rem; margin-bottom: 0.6rem;",
               for item in legend_items {
                   {item}
               }
            }
            svg {
               width: "100%",
               height: "220px",
               view_box: "0 0 360 220",
               role: "img",
               style: "display: block; overflow: visible;",
               onwheel: move |evt: Event<WheelData>| {
                   evt.prevent_default();
                   let delta = evt.data().delta().strip_units();
                   if delta.y < 0.0 {
                       let current_zoom = *zoom.read();
                       zoom.set((current_zoom * 1.12).min(8.0));
                   } else {
                       let current_zoom = *zoom.read();
                       zoom.set((current_zoom / 1.12).max(0.8));
                   }
               },
               title { "{title}" }
               rect { x: 0, y: 0, width: 360, height: 220, fill: "#f8fafc" }
               g { transform: zoom_transform,
                   line { x1: padding as i32, y1: (height - padding) as i32, x2: (width - padding) as i32, y2: (height - padding) as i32, stroke: "#64748b", stroke_width: "1" }
                   line { x1: padding as i32, y1: padding as i32, x2: padding as i32, y2: (height - padding) as i32, stroke: "#64748b", stroke_width: "1" }
                   for bar in bars {
                       {bar}
                   }
                   for line in threshold_lines {
                       {line}
                   }
                   text { x: (padding + 4.0) as i32, y: (padding + 12.0) as i32, fill: "#64748b", font_size: "11", "0" }
                   text { x: (width - padding - 20.0) as i32, y: (height - padding + 16.0) as i32, fill: "#64748b", font_size: "11", "{unit}" }
               }
            }
        }
    }
}

#[component]
fn scatter_plot(
    title: String,
    subtitle: String,
    points: Vec<PlotPoint>,
    other_label: Option<String>,
) -> Element {
    let mut zoom = use_signal(|| 1.0f64);
    let width = 360.0f64;
    let height = 220.0f64;
    let padding = 24.0f64;
    let plot_width = width - padding * 2.0;
    let plot_height = height - padding * 2.0;
    let zoom_in = move |_| {
        let current_zoom = *zoom.read();
        zoom.set((current_zoom * 1.18).min(8.0));
    };
    let zoom_out = move |_| {
        let current_zoom = *zoom.read();
        zoom.set((current_zoom / 1.18).max(0.8));
    };
    let reset_zoom = move |_| zoom.set(1.0);
    let zoom_transform = format!(
        "translate({} {}) scale({}) translate({} {})",
        width / 2.0,
        height / 2.0,
        *zoom.read(),
        -(width / 2.0),
        -(height / 2.0)
    );

    let categories = points
        .iter()
        .filter_map(|point| adduct_class(&point.adduct_type).map(|adduct| adduct.display))
        .collect::<Vec<_>>();
    let unique_categories = categories.iter().fold(Vec::new(), |mut acc, category| {
        if !acc.contains(category) {
            acc.push(category.clone());
        }
        acc
    });

    let x_positions = unique_categories
        .iter()
        .enumerate()
        .map(|(index, category)| (category.clone(), index as f64))
        .collect::<Vec<_>>();
    let x_lookup = x_positions.iter().cloned().collect::<BTreeMap<_, _>>();
    let x_min = 0.0f64;
    let x_max = (unique_categories.len().max(1) - 1) as f64;
    let x_span = (x_max - x_min).max(1e-9);

    let (y_min, y_max) = if points.is_empty() {
        (-0.02, 0.02)
    } else {
        let min = points
            .iter()
            .map(|point| point.signed_error_da)
            .fold(f64::INFINITY, f64::min);
        let max = points
            .iter()
            .map(|point| point.signed_error_da)
            .fold(f64::NEG_INFINITY, f64::max);
        let span = (max - min).max(0.02);
        (min - span * 0.05, max + span * 0.05)
    };
    let y_span = (y_max - y_min).max(1e-9);

    let category_colors = unique_categories
        .iter()
        .enumerate()
        .map(|(index, category)| (category.clone(), paul_tol_palette(index)))
        .collect::<BTreeMap<_, _>>();

    let circles = points
        .iter()
        .map(|point| {
            let adduct_class = adduct_class(&point.adduct_type).unwrap_or_else(|| AdductClass {
                label: point.adduct_type.clone(),
                display: point.adduct_type.clone(),
                charge: 0,
            });
            let x_index = x_lookup
                .get(&adduct_class.display)
                .copied()
                .unwrap_or_default();
            let x = padding + ((x_index - x_min) / x_span) * plot_width;
            let y = height - padding - ((point.signed_error_da - y_min) / y_span) * plot_height;
            let color = category_colors
                .get(&adduct_class.display)
                .copied()
                .unwrap_or("#64748B");
            rsx! {
                circle {
                    cx: x as i32,
                    cy: y as i32,
                    r: "2.2",
                    fill: color,
                    opacity: "0.95"
                }
            }
        })
        .collect::<Vec<_>>();

    let category_labels = unique_categories
        .iter()
        .enumerate()
        .map(|(index, label)| {
            let x = padding + ((index as f64 - x_min) / x_span) * plot_width;
            rsx! {
                text { x: x as i32, y: (height - padding + 16.0) as i32, fill: "#64748b", font_size: "10", "{label}" }
            }
        })
        .collect::<Vec<_>>();

    let tick_count = 5;
    let y_ticks = (0..=tick_count)
        .map(|tick| {
            let ratio = tick as f64 / tick_count as f64;
            let value = y_min + ratio * y_span;
            let y = height - padding - ratio * plot_height;
            rsx! {
                line { x1: padding as i32, y1: y as i32, x2: (width - padding) as i32, y2: y as i32, stroke: "#e2e8f0", stroke_width: "0.8" }
                text { x: (padding - 8.0) as i32, y: (y - 3.0) as i32, fill: "#64748b", font_size: "10", text_anchor: "end", {format_value(value)} }
            }
        })
        .collect::<Vec<_>>();

    let category_counts = unique_categories
        .iter()
        .map(|label| {
            let count = points
                .iter()
                .filter(|point| {
                    adduct_class(&point.adduct_type)
                        .map(|adduct| adduct.display == *label)
                        .unwrap_or(false)
                })
                .count();
            (label.clone(), count)
        })
        .collect::<Vec<_>>();

    let category_count_labels = category_counts
        .iter()
        .enumerate()
        .map(|(index, (_label, count))| {
            let x = padding + ((index as f64 - x_min) / x_span) * plot_width;
            rsx! {
                text { x: x as i32, y: (height - padding + 30.0) as i32, fill: "#64748b", font_size: "9", text_anchor: "middle", "{count}" }
            }
        })
        .collect::<Vec<_>>();

    let mut legend_items = unique_categories
        .iter()
        .enumerate()
        .map(|(index, label)| {
            let color = category_colors.get(label).copied().unwrap_or(paul_tol_palette(index));
            rsx! {
                div { style: "display: flex; align-items: center; gap: 0.35rem; font-size: 0.75rem; color: #475569;",
                    span { style: format!("display:inline-block; width:10px; height:10px; border-radius:999px; background:{};", color) }
                    span { "{label}" }
                }
            }
        })
        .collect::<Vec<_>>();
    if let Some(label) = other_label.as_ref() {
        legend_items.push(rsx! {
            div { style: "display: flex; align-items: center; gap: 0.35rem; font-size: 0.75rem; color: #475569;",
                span { style: "display:inline-block; width:10px; height:10px; border-radius:999px; background:#BBBBBB;" }
                span { "{label}" }
            }
        });
    }

    rsx! {
        div {
            style: "padding: 0.9rem; border: 1px solid #e2e8f0; border-radius: 16px; background: linear-gradient(180deg, #ffffff 0%, #f8fafc 100%);",
            h4 { style: "margin: 0 0 0.2rem; font-size: 0.95rem; color: #0f172a;", "{title}" }
            p { style: "margin: 0 0 0.65rem; color: #64748b; font-size: 0.84rem;", "{subtitle}" }
            div { style: "display: flex; justify-content: flex-end; gap: 0.3rem; margin-bottom: 0.45rem;",
                button { type: "button", style: "border: 1px solid #cbd5e1; background: white; border-radius: 999px; width: 28px; height: 28px; cursor: pointer; font-size: 0.95rem;", onclick: zoom_in, "−" }
                button { type: "button", style: "border: 1px solid #cbd5e1; background: white; border-radius: 999px; width: 28px; height: 28px; cursor: pointer; font-size: 0.95rem;", onclick: zoom_out, "＋" }
                button { type: "button", style: "border: 1px solid #cbd5e1; background: white; border-radius: 999px; padding: 0 0.55rem; height: 28px; cursor: pointer; font-size: 0.8rem; color: #475569;", onclick: reset_zoom, "Reset" }
            }
            div { style: "display: flex; flex-wrap: wrap; gap: 0.65rem; margin-bottom: 0.6rem;",
                for item in legend_items {
                    {item}
                }
            }
            svg {
                width: "100%",
                height: "220px",
                view_box: "0 0 360 220",
                role: "img",
                style: "display: block; overflow: visible;",
                onwheel: move |evt: Event<WheelData>| {
                    evt.prevent_default();
                    let delta = evt.data().delta().strip_units();
                    if delta.y > 0.0 {
                        let current_zoom = *zoom.read();
                        zoom.set((current_zoom * 1.12).min(8.0));
                    } else {
                        let current_zoom = *zoom.read();
                        zoom.set((current_zoom / 1.12).max(0.8));
                    }
                },
                title { "{title}" }
                rect { x: 0, y: 0, width: 360, height: 220, fill: "#f8fafc" }
                g { transform: zoom_transform,
                    line { x1: padding as i32, y1: (height - padding) as i32, x2: (width - padding) as i32, y2: (height - padding) as i32, stroke: "#64748b", stroke_width: "1" }
                    line { x1: padding as i32, y1: padding as i32, x2: padding as i32, y2: (height - padding) as i32, stroke: "#64748b", stroke_width: "1" }
                    line { x1: padding as i32, y1: (height - padding - ((0.0 - y_min) / y_span) * plot_height) as i32, x2: (width - padding) as i32, y2: (height - padding - ((0.0 - y_min) / y_span) * plot_height) as i32, stroke: "#94a3b8", stroke_width: "1", stroke_dasharray: "4 3" }
                    for circle in circles {
                        {circle}
                    }
                    for label in category_labels {
                        {label}
                    }
                    for tick in y_ticks {
                        {tick}
                    }
                    for count_label in category_count_labels {
                        {count_label}
                    }
                    text { x: 12, y: (height / 2.0) as i32, fill: "#64748b", font_size: "11", transform: "rotate(-90 12 110)", "Signed error (Da)" }
                    text { x: (width / 2.0) as i32, y: (height - 6.0) as i32, fill: "#64748b", font_size: "11", text_anchor: "middle", "Adduct type" }
                }
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
                let remaining = self.buffer[self.buffer_start..].to_vec();
                self.buffer_start = self.buffer.len();
                return Ok(Some(String::from_utf8_lossy(&remaining).into_owned()));
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
        self.buffer.extend_from_slice(&array.to_vec());
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

    let mut current_block = Vec::new();
    let mut current_is_in_block = false;
    let mut metrics = PrecursorMetrics::default();
    let mut smiles_cache = HashMap::new();
    let mut formula_cache = HashMap::new();
    let mut logged_failures = HashSet::new();
    let mut mascot_builder = MascotGenericFormatBuilder::<usize, f64>::default();

    while let Some(line) = reader.next_line().await? {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed == "BEGIN IONS" {
            current_block.clear();
            current_is_in_block = true;
            current_block.push(trimmed.to_string());
            let _ = mascot_builder.digest_line(trimmed);
            continue;
        }

        if !current_is_in_block {
            continue;
        }

        current_block.push(trimmed.to_string());
        if MascotGenericFormatBuilder::<usize, f64>::can_parse_line(trimmed) {
            let _ = mascot_builder.digest_line(trimmed);
        }

        if trimmed == "END IONS" {
            let parsed_mascot = std::mem::take(&mut mascot_builder).build().ok();
            if let Some(result) = process_block(
                &current_block,
                parsed_mascot.as_ref(),
                &mut smiles_cache,
                &mut formula_cache,
                &mut logged_failures,
            )? {
                metrics = merge_metrics(metrics, result);
            }
            mascot_builder = MascotGenericFormatBuilder::<usize, f64>::default();
            current_block.clear();
            current_is_in_block = false;
        }
    }

    Ok(metrics)
}

fn process_block(
    block_lines: &[String],
    parsed_mascot: Option<&MascotGenericFormat<usize, f64>>,
    smiles_cache: &mut HashMap<String, Option<f64>>,
    formula_cache: &mut HashMap<String, Option<f64>>,
    logged_failures: &mut HashSet<String>,
) -> Result<Option<PrecursorMetrics>, ScanError> {
    let mut observed_precursor = parsed_mascot.map(|block| block.parent_ion_mass());
    let mut reference_mass = None;
    let mut reference_mass_source = None;
    let mut charge = parsed_mascot.map(|block| block.charge().to_string());
    let mut adduct = None;
    let mut ion_mode = None;
    let mut smiles = None;
    let mut formula = None;
    let mut feature_id = None;
    let mut scans = None;

    for line in block_lines.iter() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed == "BEGIN IONS" || trimmed == "END IONS" {
            continue;
        }

        if let Some(stripped) = trimmed.strip_prefix("PRECURSOR_MZ=") {
            if let Ok(value) = stripped.parse::<f64>() {
                observed_precursor = Some(value);
            }
            continue;
        }

        if let Some(stripped) = trimmed.strip_prefix("PEPMASS=") {
            if let Ok(value) = stripped.parse::<f64>() {
                observed_precursor = Some(value);
            }
            continue;
        }

        if let Some(stripped) = trimmed.strip_prefix("EXACTMASS=") {
            if let Ok(value) = stripped.parse::<f64>() {
                reference_mass = Some(value);
                reference_mass_source = Some("EXACTMASS");
            }
            continue;
        }

        if let Some(stripped) = trimmed.strip_prefix("MOLECULEMASS=") {
            if let Ok(value) = stripped.parse::<f64>() {
                if reference_mass.is_none() {
                    reference_mass = Some(value);
                    reference_mass_source = Some("MOLECULEMASS");
                }
            }
            continue;
        }

        if let Some(stripped) = trimmed.strip_prefix("CHARGE=") {
            charge = Some(stripped.to_string());
            continue;
        }

        if let Some(stripped) = trimmed.strip_prefix("SMILES=") {
            smiles = Some(stripped.trim().to_string());
            continue;
        }

        if let Some(stripped) = trimmed.strip_prefix("FORMULA=") {
            formula = Some(stripped.trim().to_string());
            continue;
        }

        if let Some(stripped) = trimmed.strip_prefix("ADDUCT=") {
            adduct = Some(stripped.to_string());
            continue;
        }

        if let Some(stripped) = trimmed.strip_prefix("IONMODE=") {
            ion_mode = Some(stripped.to_string());
            continue;
        }

        if let Some(stripped) = trimmed.strip_prefix("FEATURE_ID=") {
            feature_id = Some(stripped.to_string());
            continue;
        }

        if let Some(stripped) = trimmed.strip_prefix("SCANS=") {
            scans = Some(stripped.to_string());
            continue;
        }

        if let Some(stripped) = trimmed.strip_prefix("EXTRACTSCAN=") {
            if feature_id.is_none() {
                feature_id = Some(stripped.to_string());
            }
            if scans.is_none() {
                scans = Some(stripped.to_string());
            }
            continue;
        }
    }

    let Some(observed_precursor) = observed_precursor else {
        return Ok(None);
    };

    let reference_mass = reference_mass.or_else(|| {
        formula
            .as_deref()
            .and_then(|value| exact_mass_from_formula_cached(value, formula_cache, logged_failures))
            .map(|mass| {
                reference_mass_source = Some("FORMULA");
                mass
            })
            .or_else(|| {
                let parsed_smiles = smiles.as_deref().and_then(|value| {
                    exact_mass_from_smiles_cached(value, smiles_cache, logged_failures)
                });
                if smiles.is_some() && parsed_smiles.is_none() {
                    return None;
                }
                parsed_smiles.map(|mass| {
                    reference_mass_source = Some("SMILES");
                    mass
                })
            })
    });

    let Some(reference_mass) = reference_mass else {
        let mut metrics = PrecursorMetrics::default();
        metrics.total_spectra = 1;
        metrics.skipped_spectra = 1;
        if let Some(smiles_text) = smiles.as_deref().filter(|value| !value.trim().is_empty()) {
            let trimmed_smiles = smiles_text.trim();
            metrics.unparsed_smiles = 1;
            metrics
                .unparsed_smiles_warnings
                .entry(trimmed_smiles.to_string())
                .and_modify(|detail| detail.count = detail.count.saturating_add(1))
                .or_insert(WarningDetail {
                    count: 1,
                    formula: formula.as_deref().map(str::to_string),
                });
            let warning_key = format!(
                "missing-reference-mass:{}|{}",
                trimmed_smiles,
                formula.as_deref().unwrap_or("n/a")
            );
            if logged_failures.insert(warning_key) {
                #[cfg(target_arch = "wasm32")]
                console::warn_1(
                    &format!(
                        "Unable to derive reference mass from SMILES/formula for: {trimmed_smiles} (formula: {})",
                        formula.as_deref().unwrap_or("n/a")
                    )
                    .into(),
                );
            }
        }
        return Ok(Some(metrics));
    };

    let reference_mass_source = reference_mass_source.unwrap_or("unknown");
    let reference_mass_label = adduct.as_deref().map_or_else(
        || reference_mass_source.to_string(),
        |adduct| format!("{reference_mass_source} + {adduct}"),
    );
    let adduct_label = normalize_adduct_label(adduct.as_deref().unwrap_or("unknown"));
    let adduct_text = adduct.as_deref().unwrap_or("").trim();
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
        adduct.as_deref(),
        charge.as_deref(),
        ion_mode.as_deref(),
    )
    .unwrap_or(reference_mass);
    let error_da = observed_precursor - expected_precursor_mz;
    let abs_error_da = error_da.abs();
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
    metrics.record_error(
        abs_error_da,
        abs_ppm,
        &adduct_label,
        ppm,
        error_da,
        observed_precursor,
        smiles.as_deref(),
        Some(reference_mass),
        Some(expected_precursor_mz),
        formula.as_deref(),
    );
    if abs_error_da <= 0.0001 {
        metrics.within_0_0001_da = 1;
    } else if abs_error_da <= 0.0005 {
        metrics.within_0_0005_da = 1;
    } else if abs_error_da <= 0.001 {
        metrics.within_0_001_da = 1;
    } else if abs_error_da <= 0.005 {
        metrics.within_0_005_da = 1;
    }
    if abs_ppm <= 0.5 {
        metrics.within_0_5_ppm = 1;
    } else if abs_ppm <= 1.0 {
        metrics.within_1_ppm = 1;
    } else if abs_ppm <= 5.0 {
        metrics.within_5_ppm = 1;
    } else if abs_ppm <= 10.0 {
        metrics.within_10_ppm = 1;
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
    fn treats_chiral_smiles_as_unavailable_when_parser_cannot_handle_them() {
        let mass = exact_mass_from_smiles(
            "C#C[C@]1(O)C=C[C@H]2[C@@H]3CCC4=CC(=O)CC[C@@H]4[C@H]3CC[C@@]21CC",
        );
        assert!(mass.is_none());
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
        assert!((mass - 312.092_8).abs() < 5e-4);

        let neutral = 380.165_54;
        let mass = expected_precursor_mz(neutral, Some("[M+H]+"), Some("1+"), Some("positive"))
            .expect("protonated adduct should be supported");
        assert!((mass - 381.172_8).abs() < 5e-4);

        let neutral = 333.939_62;
        let mass = expected_precursor_mz(neutral, Some("[M+O+Mg]+2"), Some("2+"), Some("positive"))
            .expect("oxygen magnesium dication adduct should be supported");
        assert!((mass - 186.959_2).abs() < 5e-4);
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
        assert!((mass - 312.092_8).abs() < 5e-4);

        let mass =
            expected_precursor_mz(333.939_62, Some("[M+O+Mg]+2"), Some("2+"), Some("positive"))
                .expect("oxygen magnesium dication adduct should be supported");
        assert!((mass - 186.959_2).abs() < 5e-4);
    }

    #[test]
    fn processes_mgf_blocks_with_mg_oxygen_adducts() {
        let block = vec![
            "BEGIN IONS".to_string(),
            "FILENAME=20230914_nexus_plate_1_Q4_pos_B7_CID_60ev.mzML".to_string(),
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
            None,
            &mut smiles_cache,
            &mut formula_cache,
            &mut logged_failures,
        )
        .expect("block should be processed")
        .expect("block should produce metrics");
        assert_eq!(metrics.spectra, 1);
        assert!(metrics.spectra_with_reference_mass > 0);
        assert!((metrics.abs_error_da_max - 11.992_84).abs() < 1e-4);
        assert!((metrics.abs_error_ppm_max - 38_427.160_1).abs() < 1.0);

        let block = vec![
            "BEGIN IONS".to_string(),
            "FILENAME=20230913_nexus_plate_1_Q2_pos_B12_CID_60ev.mzML".to_string(),
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
            None,
            &mut smiles_cache,
            &mut formula_cache,
            &mut logged_failures,
        )
        .expect("block should be processed")
        .expect("block should produce metrics");
        assert_eq!(metrics.spectra, 1);
        assert!(metrics.spectra_with_reference_mass > 0);
        assert!((metrics.abs_error_da_max - 11.992_83).abs() < 1e-4);
        assert!((metrics.abs_error_ppm_max - 64_146.776_4).abs() < 1.0);
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
        .push(next.sample_observed_precursor);
    current.observed_precursor_median = current.observed_precursor_median_tracker.median();

    current.abs_error_da_min = current.abs_error_da_min.min(next.abs_error_da_min);
    current.abs_error_da_max = current.abs_error_da_max.max(next.abs_error_da_max);
    current.abs_error_da_mean = ((current.abs_error_da_mean * current_spectra)
        + (next.abs_error_da_mean * next_spectra))
        / total_spectra;
    current
        .abs_error_da_median_tracker
        .push(next.sample_abs_error_da);
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
        .push(next.sample_abs_error_ppm);
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
        .push(next.sample_signed_error_da);
    current.signed_error_da_median = current.signed_error_da_median_tracker.median();
    current.signed_error_ppm_mean = ((current.signed_error_ppm_mean * current_spectra)
        + (next.signed_error_ppm_mean * next_spectra))
        / total_spectra;
    current
        .signed_error_ppm_median_tracker
        .push(next.sample_signed_error_ppm);
    current.signed_error_ppm_median = current.signed_error_ppm_median_tracker.median();

    current.within_0_0001_da += next.within_0_0001_da;
    current.within_0_0005_da += next.within_0_0005_da;
    current.within_0_001_da += next.within_0_001_da;
    current.within_0_005_da += next.within_0_005_da;
    current.within_0_5_ppm += next.within_0_5_ppm;
    current.within_1_ppm += next.within_1_ppm;
    current.within_5_ppm += next.within_5_ppm;
    current.within_10_ppm += next.within_10_ppm;

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

    current.plot_points.extend(next.plot_points);
    if current.plot_points.len() > 260 {
        current.plot_points = current
            .plot_points
            .iter()
            .enumerate()
            .filter(|(idx, _)| idx % (current.plot_points.len() / 260 + 1) == 0)
            .map(|(_, point)| point.clone())
            .collect();
    }

    current
}
