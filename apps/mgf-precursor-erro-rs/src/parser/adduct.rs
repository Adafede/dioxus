// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Adduct-token parsing and normalization.
//!
//! `expected_precursor_mz` lives here (rather than in `mass`) because it needs
//! the adduct parsers (`parse_charge_sign`, `parse_adduct_mass_spec_cached`,
//! `parse_charge_value`, ...) — keeping it with its dependencies avoids a
//! mass↔adduct module cycle, while `mass` stays a pure leaf that depends on
//! nothing but `mascot_rs` / `molecular_formulas`.

use std::cell::RefCell;
use std::collections::HashMap;
use std::str::FromStr;

use mascot_rs::prelude::*;
use molecular_formulas::molecular_formula::MolecularFormula;

use crate::metrics::AdductClass;

use super::{
    AMMONIUM_MASS, ELECTRON_MASS, HYDROGEN_MASS, POTASSIUM_MASS, PROTON_MASS, SODIUM_MASS,
};

thread_local! {
    static ADDUCT_SPEC_CACHE: RefCell<HashMap<String, Option<(f64, f64)>>> =
        RefCell::new(HashMap::new());
    static ADDUCT_CLASS_CACHE: RefCell<HashMap<String, Option<AdductClass>>> =
        RefCell::new(HashMap::new());
    static ADDUCT_FAMILY_CACHE: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

#[must_use]
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

    let charge_value = parse_charge_value(charge, adduct).unwrap_or(1.0);

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

#[must_use]
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
        *shift = sign.mul_add(token_mass, *shift);
    } else if let Some(token_multiplier) = parse_multiplicity_token(current) {
        *multiplier = token_multiplier;
    } else if current.eq_ignore_ascii_case("H") {
        *shift = sign.mul_add(HYDROGEN_MASS, *shift);
    } else if !current.is_empty() {
        *saw_unsupported_token = true;
    }
}

fn parse_adduct_mass_spec(adduct: &str) -> Option<(f64, f64)> {
    let normalized = adduct.trim().replace(' ', "").to_ascii_uppercase();
    let body = normalized.find(']').map_or_else(
        || normalized.clone(),
        |index| normalized[1..index].to_string(),
    );
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
    } else if shift.abs() < f64::EPSILON && (multiplier - 1.0).abs() < f64::EPSILON && body == "M" {
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
                return Some(23.985_041_7f64.mul_add(multiplier, base));
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
            .filter(char::is_ascii_digit)
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
            .filter(char::is_ascii_digit)
            .collect::<String>();
        if let Ok(parsed) = digits.parse::<f64>() {
            return Some(parsed.max(1.0));
        }
    }

    None
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
#[must_use]
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
#[must_use]
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
