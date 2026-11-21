// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Adduct-token parsing and normalization.
//!
//! `expected_precursor_mz` lives here (rather than in `mass`) because it needs
//! the adduct parsers (`parse_charge_sign`, `parse_adduct_mass_spec_cached`,
//! `parse_charge_value`, ...) — keeping it with its dependencies avoids a
//! mass↔adduct module cycle, while `mass` stays a pure leaf that depends on
//! nothing but `chematic` / `molecular_formulas`.

use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::str::FromStr;

use molecular_formulas::prelude::*;

use super::{AMMONIUM_MASS, ELECTRON_MASS, HYDROGEN_MASS, POTASSIUM_MASS, SODIUM_MASS};

thread_local! {
    static ADDUCT_SPEC_CACHE: RefCell<HashMap<String, Option<(f64, f64)>>> =
        RefCell::new(HashMap::new());
}

fn fallback_proton_spec(charge_sign: Option<bool>, charge_value: f64) -> (f64, f64) {
    let shift = match charge_sign {
        Some(true) => -HYDROGEN_MASS * charge_value,
        Some(false) => HYDROGEN_MASS * charge_value,
        None => 0.0,
    };
    (1.0, shift)
}

#[must_use]
pub(crate) fn expected_precursor_mz(
    neutral_mass: f64,
    adduct: Option<&str>,
    charge: Option<&str>,
    ion_mode: Option<&str>,
) -> f64 {
    let normalized_adduct = adduct.unwrap_or("").trim();
    let charge_value = parse_charge_value(charge, adduct).unwrap_or(1.0);
    let charge_sign =
        parse_charge_sign(charge, ion_mode).or_else(|| parse_adduct_charge_sign(adduct));

    let (multiplier, shift) = if normalized_adduct.is_empty() {
        fallback_proton_spec(charge_sign, charge_value)
    } else {
        parse_adduct_mass_spec_cached(normalized_adduct)
            .unwrap_or_else(|| fallback_proton_spec(charge_sign, charge_value))
    };
    let electron_adjustment = match charge_sign {
        Some(false) => -ELECTRON_MASS * charge_value,
        Some(true) => ELECTRON_MASS * charge_value,
        None => 0.0,
    };

    let base_mz = neutral_mass.mul_add(multiplier, shift) + electron_adjustment;
    base_mz / charge_value.max(1.0)
}

fn infer_unbracketed_charge_sign(adduct: &str) -> Option<bool> {
    let compact = adduct.replace(' ', "").to_ascii_uppercase();
    if compact.contains('[') || compact.contains(']') || compact.is_empty() {
        return None;
    }
    if compact.ends_with("-H")
        || compact.ends_with("-2H")
        || compact.ends_with("+CL")
        || compact.ends_with("+BR")
        || compact.ends_with("+HCOO")
        || compact.ends_with("+CH3COO")
        || compact == "M+FA"
        || compact == "M+HAC"
    {
        return Some(true);
    }
    if compact.contains("+H")
        || compact.contains("+NA")
        || compact.contains("+2NA")
        || compact.contains("+K")
        || compact.contains("+NH4")
        || compact.contains("+MG")
        || compact.contains("+CA")
        || compact.contains("+FE")
    {
        return Some(false);
    }
    None
}

#[must_use]
pub(crate) fn parse_adduct_charge_sign(adduct: Option<&str>) -> Option<bool> {
    let adduct = adduct?.trim();
    if let Some((_, suffix)) = adduct.split_once(']') {
        let suffix = suffix.trim();
        if suffix.contains('-') {
            return Some(true);
        }
        if suffix.contains('+') {
            return Some(false);
        }
        return None;
    }
    infer_unbracketed_charge_sign(adduct)
}

fn parse_charge_sign(charge: Option<&str>, ion_mode: Option<&str>) -> Option<bool> {
    let charge_text = charge.unwrap_or("").trim();
    let first_byte = charge_text.bytes().next();
    let last_byte = charge_text.bytes().next_back();

    if first_byte == Some(b'-') || last_byte == Some(b'-') {
        return Some(true);
    }
    if first_byte == Some(b'+') || last_byte == Some(b'+') {
        return Some(false);
    }

    let mode = ion_mode.unwrap_or("").trim();
    if mode.eq_ignore_ascii_case("negative") {
        Some(true)
    } else if mode.eq_ignore_ascii_case("positive") {
        Some(false)
    } else {
        None
    }
}

fn apply_adduct_token(
    current: &str,
    sign: f64,
    multiplier: &mut f64,
    shift: &mut f64,
    saw_molecule_term: &mut bool,
    saw_unsupported_token: &mut bool,
) {
    if let Some(token_mass) = parse_adduct_term_mass(current) {
        *shift = sign.mul_add(token_mass, *shift);
    } else if let Some(token_multiplier) = parse_multiplicity_token(current)
        && sign > 0.0
    {
        *multiplier = token_multiplier;
        *saw_molecule_term = true;
    } else if !current.is_empty() {
        *saw_unsupported_token = true;
    }
}

fn is_valid_adduct_suffix(suffix: &str) -> bool {
    suffix
        .chars()
        .all(|c| c.is_ascii_digit() || c == '+' || c == '-')
        && !(suffix.contains('+') && suffix.contains('-'))
}

fn parse_adduct_mass_spec(adduct: &str) -> Option<(f64, f64)> {
    let normalized = adduct.trim().replace(' ', "").to_ascii_uppercase();
    if normalized.is_empty() {
        return None;
    }

    let body = if normalized.contains('[') || normalized.contains(']') {
        let close_idx = normalized.find(']')?;
        if !normalized.starts_with('[') || close_idx <= 1 {
            return None;
        }
        let body = &normalized[1..close_idx];
        let suffix = &normalized[close_idx + 1..];
        if body.contains('[')
            || suffix.contains('[')
            || suffix.contains(']')
            || !is_valid_adduct_suffix(suffix)
        {
            return None;
        }
        body
    } else {
        normalized.as_str()
    };

    let charge_sign = parse_adduct_charge_sign(Some(adduct));
    if charge_sign == Some(true) {
        if body == "M+FA" {
            return parse_adduct_term_mass("HCOO").map(|m| (1.0, m));
        }
        if body == "M+HAC" {
            return parse_adduct_term_mass("CH3COO").map(|m| (1.0, m));
        }
    }

    let mut multiplier = 1.0f64;
    let mut shift = 0.0f64;
    let mut current = String::new();
    let mut sign = 1.0f64;
    let mut saw_molecule_term = false;
    let mut saw_unsupported_token = false;

    for ch in body.chars() {
        match ch {
            '+' | '-' => {
                apply_adduct_token(
                    &current,
                    sign,
                    &mut multiplier,
                    &mut shift,
                    &mut saw_molecule_term,
                    &mut saw_unsupported_token,
                );
                current.clear();
                sign = if ch == '+' { 1.0 } else { -1.0 };
            }
            _ => current.push(ch),
        }
    }

    apply_adduct_token(
        &current,
        sign,
        &mut multiplier,
        &mut shift,
        &mut saw_molecule_term,
        &mut saw_unsupported_token,
    );

    if saw_unsupported_token || !saw_molecule_term || body.is_empty() {
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
    let trimmed = adduct.trim();
    if trimmed.is_empty() {
        return None;
    }

    let key: Cow<'_, str> = if trimmed.contains(' ') {
        Cow::Owned(trimmed.replace(' ', ""))
    } else {
        Cow::Borrowed(trimmed)
    };
    ADDUCT_SPEC_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some(&mass_spec) = cache.get(key.as_ref()) {
            return mass_spec;
        }
        let mass_spec = parse_adduct_mass_spec(&key);
        cache.insert(key.into_owned(), mass_spec);
        mass_spec
    })
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
    let formula = match trimmed[digits_end..].trim() {
        "FA" | "HCOOH" | "FORMICACID" | "HFA" => "CH2O2",
        "HCOO" | "FORMAT" | "FORMATE" => "CHO2",
        "HCOONA" | "NACHO2" | "NAHCOO" | "NAHCO2" | "CHNAO2" => "CHNaO2",
        "HAC" | "HACETIC" | "ACETICACID" | "CH3COOH" => "C2H4O2",
        "CH3COO" | "ACETATE" | "OAC" => "C2H3O2",
        "ACN" | "CH3CN" => "C2H3N",
        "DMSO" => "C2H6OS",
        "ISOPROP" | "IPA" | "C3H8O" => "C3H8O",
        "TFA" | "CF3COOH" => "C2HF3O2",
        "CF3COO" => "C2F3O2",
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
        "MG" => return Some(23.985_041_7 * multiplier),
        "CA" => return Some(39.962_590_98 * multiplier),
        "FE" => return Some(55.934_937_5 * multiplier),
        "CL" => return Some(34.968_852_68 * multiplier),
        "BR" => return Some(78.918_337_1 * multiplier),
        // Neutral NH4 radical mass; expected_precursor_mz subtracts electrons once per charge.
        "NH4" => return Some(AMMONIUM_MASS * multiplier),
        "OH" => return Some(17.002_739_65 * multiplier),
        _ => return None,
    };
    let formula: ChemicalFormula<u32, i32> = ChemicalFormula::from_str(formula).ok()?;
    Some(formula.isotopologue_mass() * multiplier)
}

fn parse_ascii_digits_f64(text: &str) -> Option<f64> {
    let mut acc = 0u64;
    let mut saw_digit = false;
    for byte in text.bytes() {
        if byte.is_ascii_digit() {
            saw_digit = true;
            acc = acc
                .saturating_mul(10)
                .saturating_add(u64::from(byte - b'0'));
        }
    }
    saw_digit.then(|| f64::from(u32::try_from(acc).unwrap_or(u32::MAX)).max(1.0))
}

fn parse_charge_magnitude(text: &str) -> Option<f64> {
    let trimmed = text.trim();
    if let Some(parsed) = parse_ascii_digits_f64(trimmed) {
        return Some(parsed);
    }
    let plus_count = trimmed.bytes().filter(|&b| b == b'+').count();
    let minus_count = trimmed.bytes().filter(|&b| b == b'-').count();
    if plus_count > 0 && minus_count == 0 {
        return Some(f64::from(u32::try_from(plus_count).unwrap_or(u32::MAX)));
    }
    if minus_count > 0 && plus_count == 0 {
        return Some(f64::from(u32::try_from(minus_count).unwrap_or(u32::MAX)));
    }
    None
}

fn parse_charge_value(charge: Option<&str>, adduct: Option<&str>) -> Option<f64> {
    charge.and_then(parse_charge_magnitude).or_else(|| {
        adduct?
            .trim()
            .split_once(']')
            .and_then(|(_, suffix)| parse_charge_magnitude(suffix))
    })
}

#[must_use]
pub(crate) fn normalize_adduct_label(adduct: &str) -> &str {
    let trimmed = adduct.trim();
    if trimmed.is_empty() {
        return "unknown";
    }

    let (body, suffix) = trimmed.find(']').map_or((trimmed, ""), |idx| {
        // `find` gives a byte offset that need not be a char boundary, and
        // can be `0` for something like `"]+H]"`. Slicing either way is a
        // panic; `.get` degrades to an empty body, which the caller already
        // tolerates.
        let body = trimmed.get(1..idx).unwrap_or("");
        let suffix = trimmed.get(idx + 1..).unwrap_or("");
        (body, suffix)
    });
    let body = body.trim().trim_matches(|ch| ch == '[' || ch == ']').trim();
    let suffix = suffix.trim();

    if body.eq_ignore_ascii_case("M") {
        if matches!(suffix, "" | "+") {
            return "[M]+";
        }
        if matches!(suffix, "++" | "2+") {
            return "[M]2+";
        }
    } else if body.eq_ignore_ascii_case("M+2NA") && matches!(suffix, "" | "2+" | "++") {
        return "[M+2Na]2+";
    } else if body.eq_ignore_ascii_case("M+H") && matches!(suffix, "" | "+") {
        return "[M+H]+";
    } else if body.eq_ignore_ascii_case("M+K") && matches!(suffix, "" | "+") {
        return "[M+K]+";
    } else if body.eq_ignore_ascii_case("M+NH4") && matches!(suffix, "" | "+") {
        return "[M+NH4]+";
    } else if body.eq_ignore_ascii_case("M+NA") && matches!(suffix, "" | "+") {
        return "[M+Na]+";
    } else if body.eq_ignore_ascii_case("M+2H") && matches!(suffix, "" | "+" | "++" | "2+") {
        return "[M+2H]2+";
    } else if body.eq_ignore_ascii_case("M-H") && matches!(suffix, "" | "-" | "1-" | "--") {
        return "[M-H]-";
    } else if body.eq_ignore_ascii_case("M-2H") && matches!(suffix, "" | "2-" | "--") {
        return "[M-2H]2-";
    } else if body.eq_ignore_ascii_case("4M-H") && matches!(suffix, "" | "-" | "1-" | "--") {
        return "[4M-H]-";
    }

    trimmed
}

#[must_use]
pub(crate) fn is_supported_adduct(adduct: &str) -> bool {
    parse_adduct_mass_spec_cached(adduct).is_some()
}

#[cfg(test)]
mod tests;
