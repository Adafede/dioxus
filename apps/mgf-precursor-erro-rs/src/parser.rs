#![allow(
    clippy::branches_sharing_code,
    clippy::collapsible_if,
    clippy::float_cmp,
    clippy::if_same_then_else,
    clippy::must_use_candidate,
    clippy::option_if_let_else,
    clippy::redundant_closure_for_method_calls,
    clippy::redundant_clone,
    clippy::single_match_else,
    clippy::suboptimal_flops,
    clippy::type_complexity
)]

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::str::FromStr;

use mascot_rs::prelude::*;
use molecular_formulas_010::molecular_formula::MolecularFormula;

#[cfg(target_arch = "wasm32")]
use gloo_timers::future::TimeoutFuture;
#[cfg(target_arch = "wasm32")]
use js_sys::{Array, Uint8Array};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;
#[cfg(target_arch = "wasm32")]
use web_sys::{Blob, console};

use crate::metrics::merge_metrics;
use crate::metrics::{AdductClass, AdductFamily, PlotPointSample, PrecursorMetrics, WarningDetail};

#[cfg(any(target_arch = "wasm32", test))]
pub const CHUNK_SIZE: usize = 4 << 20;
#[cfg(any(target_arch = "wasm32", test))]
pub const PROGRESS_INTERVAL: usize = 4 << 20;

pub const PROTON_MASS: f64 = 1.007_276_466_621;
pub const HYDROGEN_MASS: f64 = PROTON_MASS + ELECTRON_MASS;
pub const ELECTRON_MASS: f64 = 0.000_548_579_909_065;
pub const SODIUM_MASS: f64 = 22.989_769_67;
pub const POTASSIUM_MASS: f64 = 38.963_707;
pub const AMMONIUM_MASS: f64 = 18.033_823;

thread_local! {
    static ADDUCT_SPEC_CACHE: RefCell<HashMap<String, Option<(f64, f64)>>> =
        RefCell::new(HashMap::new());
    static ADDUCT_CLASS_CACHE: RefCell<HashMap<String, Option<AdductClass>>> =
        RefCell::new(HashMap::new());
    static ADDUCT_FAMILY_CACHE: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

#[cfg(target_arch = "wasm32")]
pub type ScanError = JsValue;
#[cfg(not(target_arch = "wasm32"))]
pub type ScanError = String;

#[derive(Clone, Debug, Default)]
pub struct BlockParseState {
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
    fragment_peaks: Vec<f64>,
}

impl BlockParseState {
    pub fn consume_line(&mut self, line: &str) {
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
            return;
        }

        // Parse fragment line: "m/z intensity" (not a header)
        if !trimmed.contains('=') && trimmed.contains(char::is_whitespace) {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 1 {
                if let Ok(mz) = parts[0].parse::<f64>() {
                    if mz > 0.0 {
                        self.fragment_peaks.push(mz);
                    }
                }
            }
        }
    }

    pub fn consume_block_lines(&mut self, block_lines: &[String]) {
        for line in block_lines {
            self.consume_line(line);
        }
    }

    /// Extract MS2 precursor peak from fragment list.
    /// Returns the closest fragment to PEPMASS if within ~0.02 Da (~100 ppm), otherwise None.
    pub fn get_ms2_precursor_peak(&self, pepmass_header: f64) -> Option<f64> {
        const TOLERANCE_DA: f64 = 0.02;
        
        self.fragment_peaks
            .iter()
            .copied()
            .min_by(|a, b| {
                let dist_a = (a - pepmass_header).abs();
                let dist_b = (b - pepmass_header).abs();
                dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .and_then(|closest| {
                let delta_da = (closest - pepmass_header).abs();
                let delta_ppm = if pepmass_header.abs() > f64::EPSILON {
                    delta_da * 1e6 / pepmass_header
                } else {
                    f64::INFINITY
                };
                
                // Return only if within both Da and ppm thresholds
                if delta_da <= TOLERANCE_DA && delta_ppm <= 100.0 {
                    Some(closest)
                } else {
                    None
                }
            })
    }
}

pub fn smiles_is_supported(smiles: &str) -> bool {
    !smiles.trim().is_empty()
}

pub fn exact_mass_from_smiles(smiles: &str) -> Option<f64> {
    let mut cache = HashMap::new();
    let mut logged_failures = HashSet::new();
    exact_mass_from_smiles_cached(smiles, &mut cache, &mut logged_failures)
}

fn exact_mass_from_smiles_cached<S: ::std::hash::BuildHasher>(
    smiles: &str,
    cache: &mut HashMap<String, Option<f64>, S>,
    logged_failures: &mut HashSet<String, std::collections::hash_map::RandomState>,
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
    logged_failures: &mut HashSet<String, std::collections::hash_map::RandomState>,
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
            let _ = &panic;
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

pub fn exact_mass_from_formula(formula: &str) -> Option<f64> {
    let mut cache = HashMap::new();
    let mut logged_failures = HashSet::new();
    exact_mass_from_formula_cached(formula, &mut cache, &mut logged_failures)
}

fn exact_mass_from_formula_cached<S: ::std::hash::BuildHasher>(
    formula: &str,
    cache: &mut HashMap<String, Option<f64>, S>,
    logged_failures: &mut HashSet<String, std::collections::hash_map::RandomState>,
) -> Option<f64> {
    let trimmed = formula.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(mass) = cache.get(trimmed) {
        return *mass;
    }

    let parsed: std::result::Result<molecular_formulas_010::ChemicalFormula<u32, i32>, _> =
        molecular_formulas_010::ChemicalFormula::<u32, i32>::from_str(trimmed);
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

pub fn expected_precursor_mz(
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

    let base_mz = neutral_mass.mul_add(multiplier, shift) + electron_adjustment;
    Some(base_mz / charge_value.max(1.0))
}

pub fn parse_adduct_charge_sign(adduct: Option<&str>) -> Option<bool> {
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

    ADDUCT_SPEC_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some(mass_spec) = cache.get(&normalized) {
            return *mass_spec;
        }

        let mass_spec = parse_adduct_mass_spec(&normalized);
        cache.insert(normalized, mass_spec);
        mass_spec
    })
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
        "FA" | "FORMAT" | "HCOO" => "CHO2",
        "HCOONA" | "NACHO2" | "NAHCOO" | "NAHCO2" | "CHNAO2" => "CHNaO2",
        "HCOOH" | "FORMICACID" | "HFA" => "CH2O2",
        "HAC" | "HACETIC" | "ACETICACID" | "CH3COOH" => "C2H4O2",
        "MEOH" | "CH3OH" => "CH4O",
        "H2O" => "H2O",
        "NH3" => "NH3",
        "CO" => "CO",
        "CO2" => "CO2",
        "O" => "O",
        "C2H4" => return Some(28.031_300_128 * multiplier),
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

pub fn decimal_precision(value: &str) -> usize {
    let trimmed = value.trim();
    let Some((_, fractional)) = trimmed.split_once('.') else {
        return 0;
    };
    fractional
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .count()
}

#[must_use]
pub fn round_to_precision(value: f64, precision: usize) -> f64 {
    if precision == 0 {
        return value.round();
    }

    let precision = i32::try_from(precision).unwrap_or(i32::MAX);
    let factor = 10_f64.powi(precision);
    (value * factor).round() / factor
}

#[must_use]
pub fn normalize_adduct_label(adduct: &str) -> String {
    let trimmed = adduct.trim();
    if trimmed.is_empty() {
        return "unknown".to_string();
    }

    let normalized = trimmed.replace(' ', "").to_ascii_uppercase();
    let (body, suffix) = normalized
        .find(']')
        .map_or((normalized.as_str(), ""), |idx| {
            let body = &normalized[1..idx];
            let suffix = &normalized[idx + 1..];
            (body, suffix)
        });
    let body = body.trim_matches(|ch| ch == '[' || ch == ']');
    let suffix = suffix.trim();

    match (body, suffix) {
        ("M", "" | "+") => "[M]+".to_string(),
        ("M", "++" | "2+") => "[M]2+".to_string(),
        ("M+2NA", "" | "2+" | "++") => "[M+2Na]2+".to_string(),
        ("M+H", "" | "+") => "[M+H]+".to_string(),
        ("M+K", "" | "+") => "[M+K]+".to_string(),
        ("M+NH4", "" | "+") => "[M+NH4]+".to_string(),
        ("M+NA", "" | "+") => "[M+Na]+".to_string(),
        ("M+2H", "" | "+" | "++" | "2+") => "[M+2H]2+".to_string(),
        ("M-H", "" | "-" | "1-" | "--") => "[M-H]-".to_string(),
        ("M-2H", "" | "2-" | "--") => "[M-2H]2-".to_string(),
        ("4M-H", "" | "-" | "1-" | "--") => "[4M-H]-".to_string(),
        _ => trimmed.to_string(),
    }
}

#[must_use]
pub fn normalize_adduct_key(adduct: &str) -> String {
    adduct.trim().replace(' ', "").to_ascii_uppercase()
}

#[must_use]
pub const fn is_excluded_adduct(adduct: &str) -> bool {
    let _ = adduct;
    false
}

#[must_use]
pub fn is_supported_adduct(adduct: &str) -> bool {
    parse_adduct_mass_spec(adduct).is_some()
}

/// # Panics
/// Panics if the adduct-class cache mutex is poisoned.
pub fn adduct_family(adduct: &str) -> String {
    let normalized_key = normalize_adduct_key(adduct);
    let cached = ADDUCT_FAMILY_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some(cached) = cache.get(&normalized_key) {
            return Some(cached.clone());
        }

        let family =
            adduct_class(adduct).map_or_else(|| "Other".to_string(), |adduct| adduct.family);
        cache.insert(normalized_key.clone(), family.clone());
        Some(family)
    });
    cached.unwrap_or_else(|| "Other".to_string())
}

/// # Panics
/// Panics if the adduct-class cache mutex is poisoned.
pub fn adduct_class(adduct: &str) -> Option<AdductClass> {
    let normalized_key = adduct.trim().replace(' ', "").to_ascii_uppercase();
    ADDUCT_CLASS_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some(cached) = cache.get(&normalized_key).cloned() {
            return cached;
        }

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
        } else if normalized.contains("MG")
            || normalized.contains("CA")
            || normalized.contains("FE")
        {
            "Metal / complex".to_string()
        } else if normalized.contains("CL") || normalized.contains("BR") {
            "Halide".to_string()
        } else {
            "Other".to_string()
        };

        let result = Some(AdductClass {
            label: normalized.clone(),
            display: normalized,
            family,
            charge,
        });

        cache.insert(normalized_key, result.clone());
        result
    })
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
            buffer: Vec::with_capacity(CHUNK_SIZE),
            buffer_start: 0,
            processed: 0,
            progress: ProgressReporter::new(on_progress),
        }
    }

    fn total_bytes(&self) -> u64 {
        self.blob.size() as u64
    }

    async fn next_line(&mut self) -> std::result::Result<Option<String>, ScanError> {
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

    async fn load_next_chunk(&mut self) -> std::result::Result<(), ScanError> {
        let start = self.offset;
        let end = (self.offset + CHUNK_SIZE as u64).min(self.total_bytes());
        let chunk = self
            .blob
            .slice_with_f64_and_f64(start as f64, end as f64)
            .map_err(JsValue::from)?;
        let promise = chunk.array_buffer();
        let bytes = JsFuture::from(promise).await?;
        let array = Uint8Array::new(&bytes);
        let chunk_len = array.byte_length() as usize;
        let mut chunk_bytes = Vec::with_capacity(chunk_len);
        chunk_bytes.resize(chunk_len, 0);
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
pub async fn scan_blob_with_progress(
    blob: &Blob,
    mut on_progress: impl FnMut(u64, u64),
) -> std::result::Result<PrecursorMetrics, ScanError> {
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

/// Process a single MGF block into precursor metrics.
///
/// # Errors
/// Returns an error when the parser cannot produce a valid scan result.
pub fn process_block<S: ::std::hash::BuildHasher>(
    block_lines: &[String],
    smiles_cache: &mut HashMap<String, Option<f64>, S>,
    formula_cache: &mut HashMap<String, Option<f64>, S>,
    logged_failures: &mut HashSet<String, std::collections::hash_map::RandomState>,
    plot_sample: Option<&mut PlotPointSample>,
) -> std::result::Result<Option<PrecursorMetrics>, ScanError> {
    let mut state = BlockParseState::default();
    state.consume_block_lines(block_lines);
    let use_external_sample = plot_sample.is_some();
    let mut local_plot_sample = PlotPointSample::default();
    let mut sample_ref = if use_external_sample {
        plot_sample
    } else {
        Some(&mut local_plot_sample)
    };
    let result = process_block_state(
        &state,
        smiles_cache,
        formula_cache,
        logged_failures,
        &mut sample_ref,
    )?;
    Ok(result.map(|mut metrics| {
        if let Some(plot_sample) = sample_ref.as_ref() {
            metrics.plot_points.clone_from(&plot_sample.points);
            metrics.plot_point_stream_seen = plot_sample.seen;
        }
        metrics
    }))
}

#[allow(
    clippy::field_reassign_with_default,
    clippy::too_many_lines,
    clippy::unnecessary_wraps
)]
fn process_block_state<S: ::std::hash::BuildHasher>(
    state: &BlockParseState,
    smiles_cache: &mut HashMap<String, Option<f64>, S>,
    formula_cache: &mut HashMap<String, Option<f64>, S>,
    logged_failures: &mut HashSet<String, std::collections::hash_map::RandomState>,
    plot_sample: &mut Option<&mut PlotPointSample>,
) -> std::result::Result<Option<PrecursorMetrics>, ScanError> {
    let Some(observed_precursor) = state.observed_precursor else {
        return Ok(None);
    };

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
                .or_insert_with(|| WarningDetail {
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
                console::warn_1(&format!("Unable to derive reference mass from SMILES/formula for: {trimmed_smiles} (formula: {})", state.formula.as_deref().unwrap_or("n/a")).into());
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
    let error_da = observed_precursor - expected_precursor_mz;
    let abs_error_da = error_da.abs();
    let error_milli_da = abs_error_da * 1000.0;
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
        AdductFamily::from_label(&adduct_label),
        ppm,
        error_da,
        observed_precursor,  // PEPMASS from header (metadata block)
        state.get_ms2_precursor_peak(observed_precursor),  // MS2 precursor peak, closest to PEPMASS within tolerance
        state.smiles.as_deref(),
        Some(reference_mass),
        Some(expected_precursor_mz),
        state.formula.as_deref(),
        plot_sample,
    );
    if error_milli_da <= 0.1 {
        metrics.within_0_1_da = 1;
    } else if error_milli_da <= 0.5 {
        metrics.within_0_5_da = 1;
    } else if error_milli_da <= 1.0 {
        metrics.within_1_da = 1;
    } else if error_milli_da <= 5.0 {
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

/// Parse MGF from a string (for non-async contexts, e.g., browser file reading).
pub fn parse_mgf_from_string(content: &str) -> std::result::Result<PrecursorMetrics, String> {
    let mut current_state = BlockParseState::default();
    let mut current_is_in_block = false;
    let mut metrics = PrecursorMetrics::default();
    let mut plot_sample = PlotPointSample::default();
    let mut smiles_cache = HashMap::new();
    let mut formula_cache = HashMap::new();
    let mut logged_failures = HashSet::new();

    for line in content.lines() {
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
            ).map_err(|e| format!("{:?}", e))? {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_full_precision_when_computing_error() {
        let state = BlockParseState {
            observed_precursor_raw: Some("100.1234".to_string()),
            observed_precursor: Some(100.1234),
            reference_mass: Some(100.123_456_789),
            reference_mass_source: Some("EXACTMASS".to_string()),
            charge: Some("1".to_string()),
            ..Default::default()
        };

        let mut smiles_cache = HashMap::new();
        let mut formula_cache = HashMap::new();
        let mut logged_failures = HashSet::new();
        let mut plot_sample = None;

        let metrics = process_block_state(
            &state,
            &mut smiles_cache,
            &mut formula_cache,
            &mut logged_failures,
            &mut plot_sample,
        )
        .expect("parser should succeed")
        .expect("metrics should be produced");

        let expected_error = 100.1234_f64 - 100.123_456_789_f64;
        assert!((metrics.sample_abs_error_da - expected_error.abs()).abs() < 1e-12);
    }
}
