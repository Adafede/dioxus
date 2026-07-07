#![allow(clippy::all)]
#![allow(warnings)]

use std::collections::BTreeMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::str::FromStr;

use dioxus::events::{DragData, FormData, WheelData};
use dioxus::html::HasFileData;
use dioxus::prelude::*;
#[cfg(target_arch = "wasm32")]
use gloo_timers::future::TimeoutFuture;
#[cfg(target_arch = "wasm32")]
use js_sys::Uint8Array;
use molecular_formulas::{MolecularFormula, prelude::ChemicalFormula};
use smiles_parser::chain;
use smiles_parser::graph::{Atom, MoleculeGraph};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;
#[cfg(target_arch = "wasm32")]
use web_sys::Blob;

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
struct AdductClass {
    label: String,
    display: String,
    charge: i32,
}

#[derive(Clone, Debug, PartialEq)]
struct PrecursorMetrics {
    spectra: usize,
    total_spectra: usize,
    skipped_spectra: usize,
    spectra_with_reference_mass: usize,
    reference_mass_source: String,
    unparsed_smiles: usize,
    observed_precursor_min: f64,
    observed_precursor_max: f64,
    observed_precursor_mean: f64,
    abs_error_da_min: f64,
    abs_error_da_max: f64,
    abs_error_da_mean: f64,
    abs_error_da_rms: f64,
    abs_error_ppm_min: f64,
    abs_error_ppm_max: f64,
    abs_error_ppm_mean: f64,
    abs_error_ppm_rms: f64,
    signed_error_da_mean: f64,
    signed_error_ppm_mean: f64,
    within_0_0005_da: usize,
    within_0_001_da: usize,
    within_0_005_da: usize,
    within_1_ppm: usize,
    within_5_ppm: usize,
    within_10_ppm: usize,
    da_error_histogram: HistogramData,
    ppm_error_histogram: HistogramData,
    plot_points: Vec<PlotPoint>,
    unrecognized_adducts: BTreeMap<String, usize>,
    high_error_smiles: BTreeMap<String, usize>,
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
            observed_precursor_min: 0.0,
            observed_precursor_max: 0.0,
            observed_precursor_mean: 0.0,
            abs_error_da_min: 0.0,
            abs_error_da_max: 0.0,
            abs_error_da_mean: 0.0,
            abs_error_da_rms: 0.0,
            abs_error_ppm_min: 0.0,
            abs_error_ppm_max: 0.0,
            abs_error_ppm_mean: 0.0,
            abs_error_ppm_rms: 0.0,
            signed_error_da_mean: 0.0,
            signed_error_ppm_mean: 0.0,
            within_0_0005_da: 0,
            within_0_001_da: 0,
            within_0_005_da: 0,
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
        smiles: Option<&str>,
    ) {
        self.da_error_histogram.add_value(abs_error_da);
        self.ppm_error_histogram.add_value(abs_ppm);

        if abs_ppm > 10.0 {
            if let Some(smiles) = smiles.filter(|value| !value.trim().is_empty()) {
                self.high_error_smiles
                    .entry(smiles.trim().to_string())
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
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

fn exact_mass_from_smiles(smiles: &str) -> Option<f64> {
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
        Ok(mass) => mass,
        Err(_) => None,
    }
}

fn exact_mass_from_formula(formula: &str) -> Option<f64> {
    let formula: ChemicalFormula<u32, i32> = ChemicalFormula::from_str(formula).ok()?;
    Some(formula.isotopologue_mass())
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

    let (multiplier, shift) = if normalized_adduct.is_empty() {
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
    } else {
        parse_adduct_mass_spec(normalized_adduct).unwrap_or_else(|| {
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
        })
    };

    let charge_value = parse_charge_value(charge, adduct).unwrap_or_else(|| {
        if charge_sign == Some(true) || normalized_ion_mode == "negative" {
            1.0
        } else if charge_sign == Some(false) || normalized_ion_mode == "positive" {
            1.0
        } else {
            1.0
        }
    });
    let electron_adjustment = match charge_sign {
        Some(false) => -ELECTRON_MASS * charge_value,
        Some(true) => 0.0,
        None => 0.0,
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
    let mut multiplier = 1.0f64;
    let mut shift = 0.0f64;
    let mut current = String::new();
    let mut sign = 1.0f64;
    let mut saw_unsupported_token = false;

    for ch in body.chars() {
        match ch {
            '+' => {
                if let Some(token_mass) = parse_adduct_term_mass(&current) {
                    shift += sign * token_mass;
                } else if current.eq_ignore_ascii_case("M") {
                    multiplier = 1.0;
                } else if current.eq_ignore_ascii_case("2M") {
                    multiplier = 2.0;
                } else if current.eq_ignore_ascii_case("3M") {
                    multiplier = 3.0;
                } else if !current.is_empty() {
                    saw_unsupported_token = true;
                }
                current.clear();
                sign = 1.0;
            }
            '-' => {
                if let Some(token_mass) = parse_adduct_term_mass(&current) {
                    shift += sign * token_mass;
                } else if current.eq_ignore_ascii_case("M") {
                    multiplier = 1.0;
                } else if current.eq_ignore_ascii_case("2M") {
                    multiplier = 2.0;
                } else if current.eq_ignore_ascii_case("3M") {
                    multiplier = 3.0;
                } else if !current.is_empty() {
                    saw_unsupported_token = true;
                }
                current.clear();
                sign = -1.0;
            }
            _ => current.push(ch),
        }
    }

    if let Some(token_mass) = parse_adduct_term_mass(&current) {
        shift += sign * token_mass;
    } else if current.eq_ignore_ascii_case("M") {
        multiplier = 1.0;
    } else if current.eq_ignore_ascii_case("2M") {
        multiplier = 2.0;
    } else if current.eq_ignore_ascii_case("3M") {
        multiplier = 3.0;
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
        "HCOOH" | "FORMICACID" => "CH2O2",
        "MEOH" | "CH3OH" => "CH4O",
        "HFA" => "C2HF3O2",
        "H2O" => "H2O",
        "NH3" => "NH3",
        "CO" => "CO",
        "CO2" => "CO2",
        "O" => "O",
        "C2H4" => return Some(28.031_300_128 * multiplier),
        "CHNAO2" | "HCOONA" => return Some(67.987_423_942 * multiplier),
        "H" => return Some(HYDROGEN_MASS * multiplier),
        "NA" => return Some(SODIUM_MASS * multiplier),
        "K" => return Some(POTASSIUM_MASS * multiplier),
        "MG" => return Some(23.985_041_7 * multiplier),
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
                                        for (smiles, count) in metrics.high_error_smiles.iter() {
                                           {
                                               let suffix = if *count > 1 { format!(" (x{count})") } else { String::new() };
                                               rsx! {
                                                   li { "{smiles}{suffix}" }
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
                                        li { "mean: {format_value(metrics.observed_precursor_mean)}" }
                                    }
                                }
                                div { style: "background: white; padding: 0.8rem; border-radius: 12px; border: 1px solid #e2e8f0; box-shadow: 0 4px 12px rgba(15, 23, 42, 0.04);",
                                    h4 { style: "margin: 0 0 0.45rem; font-size: 0.95rem; color: #0f172a;", "Absolute error (Da)" }
                                    ul { style: "padding-left: 1.1rem; margin: 0.25rem 0 0; color: #475569;",
                                        li { "min: {format_value(metrics.abs_error_da_min)}" }
                                        li { "mean: {format_value(metrics.abs_error_da_mean)}" }
                                        li { "RMS: {format_value(metrics.abs_error_da_rms)}" }
                                        li { "max: {format_value(metrics.abs_error_da_max)}" }
                                    }
                                }
                                div { style: "background: white; padding: 0.8rem; border-radius: 12px; border: 1px solid #e2e8f0; box-shadow: 0 4px 12px rgba(15, 23, 42, 0.04);",
                                    h4 { style: "margin: 0 0 0.45rem; font-size: 0.95rem; color: #0f172a;", "Absolute error (ppm)" }
                                    ul { style: "padding-left: 1.1rem; margin: 0.25rem 0 0; color: #475569;",
                                        li { "min: {format_value(metrics.abs_error_ppm_min)}" }
                                        li { "mean: {format_value(metrics.abs_error_ppm_mean)}" }
                                        li { "RMS: {format_value(metrics.abs_error_ppm_rms)}" }
                                        li { "max: {format_value(metrics.abs_error_ppm_max)}" }
                                    }
                                }
                                div { style: "background: white; padding: 0.8rem; border-radius: 12px; border: 1px solid #e2e8f0; box-shadow: 0 4px 12px rgba(15, 23, 42, 0.04);",
                                    h4 { style: "margin: 0 0 0.45rem; font-size: 0.95rem; color: #0f172a;", "Signed mean" }
                                    ul { style: "padding-left: 1.1rem; margin: 0.25rem 0 0; color: #475569;",
                                        li { "Da: {format_value(metrics.signed_error_da_mean)}" }
                                        li { "ppm: {format_value(metrics.signed_error_ppm_mean)}" }
                                    }
                                }
                            }

                            div {
                                style: "margin-top: 1rem; display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 1rem;",
                                histogram_plot {
                                    title: "Absolute precursor error (Da)".to_string(),
                                    subtitle: "Distribution across spectra, with tolerance bands highlighted".to_string(),
                                    histogram: metrics.da_error_histogram.clone(),
                                    thresholds: vec![0.0005, 0.005],
                                    unit: "Da".to_string(),
                                }
                                histogram_plot {
                                    title: "Absolute precursor error (ppm)".to_string(),
                                    subtitle: "Distribution across spectra, with common tolerance bands".to_string(),
                                    histogram: metrics.ppm_error_histogram.clone(),
                                    thresholds: vec![1.0, 5.0],
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
                                span { style: "display: inline-block; padding: 0.4rem 0.7rem; background: #ecfdf3; color: #166534; border: 1px solid #86efac; border-radius: 999px; font-weight: 700; box-shadow: 0 1px 2px rgba(22, 101, 52, 0.12);",
                                    "≤ 0.0005 Da: {format_count_with_percentage(metrics.within_0_0005_da, metrics.spectra)}"
                                }
                                span { style: "display: inline-block; padding: 0.4rem 0.7rem; background: #f0fdf4; color: #15803d; border: 1px solid #4ade80; border-radius: 999px; font-weight: 700; box-shadow: 0 1px 2px rgba(21, 128, 61, 0.12);",
                                    "≤ 0.001 Da: {format_count_with_percentage(metrics.within_0_001_da, metrics.spectra)}"
                                }
                                span { style: "display: inline-block; padding: 0.4rem 0.7rem; background: #eff6ff; color: #1d4ed8; border: 1px solid #93c5fd; border-radius: 999px; font-weight: 700; box-shadow: 0 1px 2px rgba(29, 78, 216, 0.12);",
                                    "≤ 0.005 Da: {format_count_with_percentage(metrics.within_0_005_da, metrics.spectra)}"
                                }
                                span { style: "display: inline-block; padding: 0.4rem 0.7rem; background: #ecfeff; color: #0f766e; border: 1px solid #5eead4; border-radius: 999px; font-weight: 700; box-shadow: 0 1px 2px rgba(15, 118, 110, 0.12);",
                                    "≤ 1 ppm: {format_count_with_percentage(metrics.within_1_ppm, metrics.spectra)}"
                                }
                                span { style: "display: inline-block; padding: 0.4rem 0.7rem; background: #f5f3ff; color: #6d28d9; border: 1px solid #c4b5fd; border-radius: 999px; font-weight: 700; box-shadow: 0 1px 2px rgba(109, 40, 217, 0.12);",
                                    "≤ 5 ppm: {format_count_with_percentage(metrics.within_5_ppm, metrics.spectra)}"
                                }
                                span { style: "display: inline-block; padding: 0.4rem 0.7rem; background: #fef2f2; color: #b91c1c; border: 1px solid #fda4af; border-radius: 999px; font-weight: 700; box-shadow: 0 1px 2px rgba(185, 28, 28, 0.12);",
                                    "≤ 10 ppm: {format_count_with_percentage(metrics.within_10_ppm, metrics.spectra)}"
                                }
                            }
                        }
                    }
                }
            }
        }
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

fn scientific_palette(index: usize) -> &'static str {
    [
        "#4477AA", "#66CCEE", "#228833", "#CCBB44", "#EE6677", "#AA3377", "#BBBBBB", "#004488",
    ][index % 8]
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
            let bar_height = if *count == 0 {
                0.0
            } else {
                (*count as f64 / max_count as f64) * plot_height
            };
            let y = height - padding - bar_height;
            rsx! {
                rect {
                    x: x as i32,
                    y: y as i32,
                    width: bar_width as i32,
                    height: bar_height as i32,
                    fill: scientific_palette(index),
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
                    stroke: scientific_palette(index + 2),
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
            let label = format!("≤ {threshold:.3} {unit}");
            rsx! {
                div { style: "display: flex; align-items: center; gap: 0.35rem; font-size: 0.75rem; color: #475569;",
                    span { style: format!("display:inline-block; width:10px; height:10px; border-radius:999px; background:{};", scientific_palette(index + 2)) }
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
        .map(|(index, category)| (category.clone(), scientific_palette(index)))
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
                .unwrap_or(scientific_palette(7));
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
            let color = category_colors.get(label).copied().unwrap_or(scientific_palette(index));
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
                span { style: "display:inline-block; width:10px; height:10px; border-radius:999px; background:#94a3b8;" }
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
                if self.buffer.is_empty() {
                    return Ok(None);
                }
                let line = std::mem::take(&mut self.buffer);
                return Ok(Some(String::from_utf8_lossy(&line).into_owned()));
            }

            self.load_next_chunk().await?;
        }
    }

    fn take_line_from_buffer(&mut self) -> Option<String> {
        if let Some(pos) = self.buffer.iter().position(|byte| *byte == b'\n') {
            let mut line_bytes = self.buffer[..pos].to_vec();
            self.buffer.drain(..=pos);
            if line_bytes.last() == Some(&b'\r') {
                line_bytes.pop();
            }
            Some(String::from_utf8_lossy(&line_bytes).into_owned())
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

    while let Some(line) = reader.next_line().await? {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed == "BEGIN IONS" {
            current_block.clear();
            current_is_in_block = true;
            current_block.push(trimmed.to_string());
            continue;
        }

        if !current_is_in_block {
            continue;
        }

        if trimmed == "END IONS" {
            current_block.push(trimmed.to_string());
            if let Some(result) = process_block(&mut current_block).await? {
                metrics = merge_metrics(metrics, result);
            }
            current_block.clear();
            current_is_in_block = false;
            continue;
        }

        current_block.push(trimmed.to_string());
    }

    Ok(metrics)
}

#[cfg(target_arch = "wasm32")]
async fn process_block(block_lines: &mut [String]) -> Result<Option<PrecursorMetrics>, ScanError> {
    let mut headers = std::collections::BTreeMap::new();
    let mut observed_precursor = None;
    let mut reference_mass = None;
    let mut reference_mass_source = None;
    let mut charge = None;
    let mut adduct = None;
    let mut ion_mode = None;
    let mut smiles = None;
    let mut formula = None;
    let mut feature_id = None;
    let mut scans = None;
    let mut spectra_lines = Vec::new();

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

        if let Some(stripped) = trimmed.strip_prefix("FILENAME=") {
            headers.insert("FILENAME".to_string(), stripped.to_string());
            continue;
        }

        spectra_lines.push(trimmed.to_string());
    }

    let Some(observed_precursor) = observed_precursor else {
        return Ok(None);
    };

    let reference_mass = reference_mass.or_else(|| {
        formula
            .as_deref()
            .and_then(exact_mass_from_formula)
            .map(|mass| {
                reference_mass_source = Some("FORMULA");
                mass
            })
            .or_else(|| {
                let parsed_smiles = smiles.as_deref().and_then(exact_mass_from_smiles);
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
        if smiles.is_some() {
            metrics.unparsed_smiles = 1;
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

    let mut normalized = Vec::new();
    normalized.push("BEGIN IONS".to_string());

    if let Some(filename) = headers.get("FILENAME") {
        normalized.push(format!("FILENAME={filename}"));
    }

    if let Some(feature_id) = feature_id.as_deref() {
        normalized.push(format!("FEATURE_ID={feature_id}"));
    }

    normalized.push(format!("PEPMASS={observed_precursor}"));
    if let Some(charge) = charge.as_deref() {
        normalized.push(format!("CHARGE={charge}"));
    }
    if let Some(scans) = scans.as_deref() {
        normalized.push(format!("SCANS={scans}"));
    }
    normalized.push("RTINSECONDS=1.0".to_string());
    normalized.extend(spectra_lines.clone());
    normalized.push("END IONS".to_string());

    let block_text = normalized.join("\n");
    let mut parsed_lines = Vec::new();
    for raw_line in block_text.lines().filter(|line| !line.is_empty()) {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed == "BEGIN IONS" || trimmed == "END IONS" {
            continue;
        }

        if trimmed.starts_with("SCANS=") {
            parsed_lines.push(trimmed.to_string());
            continue;
        }

        if trimmed.starts_with("FEATURE_ID=")
            || trimmed.starts_with("PEPMASS=")
            || trimmed.starts_with("CHARGE=")
            || trimmed.starts_with("RTINSECONDS=")
            || trimmed.starts_with("FILENAME=")
        {
            parsed_lines.push(trimmed.to_string());
        }
    }

    if parsed_lines.is_empty() {
        return Ok(None);
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
        smiles.as_deref(),
    );
    if abs_error_da <= 0.0005 {
        metrics.within_0_0005_da = 1;
    }
    if abs_error_da <= 0.001 {
        metrics.within_0_001_da = 1;
    }
    if abs_error_da < 0.005 {
        metrics.within_0_005_da = 1;
    }
    if abs_ppm <= 1.0 {
        metrics.within_1_ppm = 1;
    }
    if abs_ppm <= 5.0 {
        metrics.within_5_ppm = 1;
    }
    if abs_ppm <= 10.0 {
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
    fn uses_ion_mass_for_sodium_adducts() {
        let mass = expected_precursor_mz(1000.0, Some("[M+Na]+"), Some("1+"), Some("positive"))
            .expect("sodium adduct should be supported");
        assert!((mass - (1000.0 + SODIUM_MASS - ELECTRON_MASS)).abs() < 1e-9);
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
    fn handles_hydrogen_minus_terms_for_metal_adducts() {
        let protonated_mass = super::expected_precursor_mz(1000.0, Some("[M+H]+"), Some("1+"), Some("positive"))
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
        assert!((sodium_formate - (1000.0 + 67.987_423_942 + PROTON_MASS)).abs() < 1e-9);

        let formate_adduct = super::expected_precursor_mz(
            1000.0,
            Some("[M+FA]-"),
            Some("1-"),
            Some("negative"),
        )
        .expect("formate adduct should be supported");
        let expected_formate_mass = 1000.0 + super::exact_mass_from_formula("CHO2")
            .expect("CHO2 mass should be available");
        assert!((formate_adduct - expected_formate_mass).abs() < 1e-9);
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

    current.abs_error_da_min = current.abs_error_da_min.min(next.abs_error_da_min);
    current.abs_error_da_max = current.abs_error_da_max.max(next.abs_error_da_max);
    current.abs_error_da_mean = ((current.abs_error_da_mean * current_spectra)
        + (next.abs_error_da_mean * next_spectra))
        / total_spectra;
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
    let current_ppm_rms_sq = current.abs_error_ppm_rms * current.abs_error_ppm_rms;
    let next_ppm_rms_sq = next.abs_error_ppm_rms * next.abs_error_ppm_rms;
    current.abs_error_ppm_rms =
        ((current_ppm_rms_sq * current_spectra) + (next_ppm_rms_sq * next_spectra)) / total_spectra;
    current.abs_error_ppm_rms = current.abs_error_ppm_rms.sqrt();

    current.signed_error_da_mean = ((current.signed_error_da_mean * current_spectra)
        + (next.signed_error_da_mean * next_spectra))
        / total_spectra;
    current.signed_error_ppm_mean = ((current.signed_error_ppm_mean * current_spectra)
        + (next.signed_error_ppm_mean * next_spectra))
        / total_spectra;

    current.within_0_0005_da += next.within_0_0005_da;
    current.within_0_001_da += next.within_0_001_da;
    current.within_0_005_da += next.within_0_005_da;
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
    for (smiles, count) in next.high_error_smiles {
        current
            .high_error_smiles
            .entry(smiles)
            .and_modify(|existing| *existing += count)
            .or_insert(count);
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
