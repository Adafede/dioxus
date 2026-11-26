// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Const atomic/adduct masses and low-level adduct token parsing.

use super::{Polarity, parse_adduct_polarity};

/// 2022 CODATA electron rest mass (Da).
pub(crate) const ELECTRON_MASS: f64 = 0.000_548_579_909_044_1;
/// Monoisotopic hydrogen mass (CIAAW, Da).
pub(crate) const HYDROGEN_MASS: f64 = 1.007_825_032_2;
const NITROGEN_MASS: f64 = 14.003_074_004;
/// Neutral NH4 radical mass (electron charge adjusted separately per z).
pub(crate) const AMMONIUM_MASS: f64 = NITROGEN_MASS + (4.0 * HYDROGEN_MASS);
/// Monoisotopic sodium-23 mass (CIAAW, Da).
pub(crate) const SODIUM_MASS: f64 = 22.989_769_28;
/// Monoisotopic potassium-39 mass (CIAAW, Da).
pub(crate) const POTASSIUM_MASS: f64 = 38.963_706_49;

const CARBON_MASS: f64 = 12.0;
const OXYGEN_MASS: f64 = 15.994_914_619_57;
const FLUORINE_MASS: f64 = 18.998_403_162_73;
const SULFUR_MASS: f64 = 31.972_071_174_4;
/// Monoisotopic magnesium-24 mass (CIAAW, Da).
const MAGNESIUM_MASS: f64 = 23.985_041_7;
/// Monoisotopic chlorine-35 mass (CIAAW, Da).
const CHLORINE_MASS: f64 = 34.968_852_68;
/// Monoisotopic calcium-40 mass (CIAAW, Da).
const CALCIUM_MASS: f64 = 39.962_590_98;
/// Monoisotopic iron-56 mass (CIAAW, Da).
const IRON_MASS: f64 = 55.934_937_5;
/// Monoisotopic bromine-79 mass (CIAAW, Da).
const BROMINE_MASS: f64 = 78.918_337_1;
const ETHYLENE_MASS: f64 = 2.0 * CARBON_MASS + 4.0 * HYDROGEN_MASS;
const HYDROXYL_MASS: f64 = OXYGEN_MASS + HYDROGEN_MASS;

const FORMIC_ACID_MASS: f64 = CARBON_MASS + 2.0 * HYDROGEN_MASS + 2.0 * OXYGEN_MASS;
const FORMATE_MASS: f64 = CARBON_MASS + HYDROGEN_MASS + 2.0 * OXYGEN_MASS;
const SODIUM_FORMATE_MASS: f64 = CARBON_MASS + HYDROGEN_MASS + SODIUM_MASS + 2.0 * OXYGEN_MASS;
const ACETIC_ACID_MASS: f64 = 2.0 * CARBON_MASS + 4.0 * HYDROGEN_MASS + 2.0 * OXYGEN_MASS;
const ACETATE_MASS: f64 = 2.0 * CARBON_MASS + 3.0 * HYDROGEN_MASS + 2.0 * OXYGEN_MASS;
const ACETONITRILE_MASS: f64 = 2.0 * CARBON_MASS + 3.0 * HYDROGEN_MASS + NITROGEN_MASS;
const DMSO_MASS: f64 = 2.0 * CARBON_MASS + 6.0 * HYDROGEN_MASS + OXYGEN_MASS + SULFUR_MASS;
const ISOPROPANOL_MASS: f64 = 3.0 * CARBON_MASS + 8.0 * HYDROGEN_MASS + OXYGEN_MASS;
const TFA_MASS: f64 = 2.0 * CARBON_MASS + HYDROGEN_MASS + 3.0 * FLUORINE_MASS + 2.0 * OXYGEN_MASS;
const TRIFLUOROACETATE_MASS: f64 = 2.0 * CARBON_MASS + 3.0 * FLUORINE_MASS + 2.0 * OXYGEN_MASS;
const METHANOL_MASS: f64 = CARBON_MASS + 4.0 * HYDROGEN_MASS + OXYGEN_MASS;
const WATER_MASS: f64 = 2.0 * HYDROGEN_MASS + OXYGEN_MASS;
const AMMONIA_MASS: f64 = NITROGEN_MASS + 3.0 * HYDROGEN_MASS;
const CO_MASS: f64 = CARBON_MASS + OXYGEN_MASS;
const CO2_MASS: f64 = CARBON_MASS + 2.0 * OXYGEN_MASS;

/// Infers polarity for unbracketed adduct forms like `"M+H"` or `"M-H"`.
#[must_use]
pub(crate) fn parse_unbracketed_polarity(adduct: &str) -> Option<Polarity> {
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
        return Some(Polarity::Negative);
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
        return Some(Polarity::Positive);
    }
    None
}

/// Resolves ion [`Polarity`] from `CHARGE=` signs or fallback `IONMODE=`.
#[must_use]
pub(crate) fn parse_polarity_tokens(
    charge: Option<&str>,
    ion_mode: Option<&str>,
) -> Option<Polarity> {
    let charge_text = charge.unwrap_or("").trim();
    let first = charge_text.bytes().next();
    let last = charge_text.bytes().next_back();
    if first == Some(b'-') || last == Some(b'-') {
        return Some(Polarity::Negative);
    }
    if first == Some(b'+') || last == Some(b'+') {
        return Some(Polarity::Positive);
    }
    let mode = ion_mode.unwrap_or("").trim();
    if mode.eq_ignore_ascii_case("negative") {
        Some(Polarity::Negative)
    } else if mode.eq_ignore_ascii_case("positive") {
        Some(Polarity::Positive)
    } else {
        None
    }
}

/// Computes the exact neutral/ionic mass contribution of an adduct token (e.g. `"2H"`, `"ACN"`).
#[must_use]
pub(crate) fn parse_adduct_term_mass(token: &str) -> Option<f64> {
    let trimmed = token.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("M") {
        return None;
    }
    let digits_end = trimmed
        .chars()
        .position(|ch| !ch.is_ascii_digit())
        .unwrap_or(trimmed.len());
    let multiplier = if digits_end == 0 {
        1.0
    } else {
        let count = trimmed.get(..digits_end)?.parse::<u32>().ok()?;
        if count == 0 {
            return None;
        }
        f64::from(count)
    };
    let symbol = trimmed.get(digits_end..).unwrap_or("").trim();
    let term_mass = match symbol {
        "FA" | "HCOOH" | "FORMICACID" | "HFA" => FORMIC_ACID_MASS,
        "HCOO" | "FORMAT" | "FORMATE" => FORMATE_MASS,
        "HCOONA" | "NACHO2" | "NAHCOO" | "NAHCO2" | "CHNAO2" => SODIUM_FORMATE_MASS,
        "HAC" | "HACETIC" | "ACETICACID" | "CH3COOH" => ACETIC_ACID_MASS,
        "CH3COO" | "ACETATE" | "OAC" => ACETATE_MASS,
        "ACN" | "CH3CN" => ACETONITRILE_MASS,
        "DMSO" => DMSO_MASS,
        "ISOPROP" | "IPA" | "C3H8O" => ISOPROPANOL_MASS,
        "TFA" | "CF3COOH" => TFA_MASS,
        "CF3COO" => TRIFLUOROACETATE_MASS,
        "MEOH" | "CH3OH" => METHANOL_MASS,
        "H2O" => WATER_MASS,
        "NH3" => AMMONIA_MASS,
        "CO" => CO_MASS,
        "CO2" => CO2_MASS,
        "O" => OXYGEN_MASS,
        "C2H4" => ETHYLENE_MASS,
        "H" => HYDROGEN_MASS,
        "NA" => SODIUM_MASS,
        "K" => POTASSIUM_MASS,
        "MG" => MAGNESIUM_MASS,
        "CA" => CALCIUM_MASS,
        "FE" => IRON_MASS,
        "CL" => CHLORINE_MASS,
        "BR" => BROMINE_MASS,
        "NH4" => AMMONIUM_MASS,
        "OH" => HYDROXYL_MASS,
        _ => return None,
    };
    Some(term_mass * multiplier)
}

pub(crate) fn parse_adduct_mass_components(adduct: &str) -> Option<(f64, f64)> {
    let normalized = adduct.trim().replace(' ', "").to_ascii_uppercase();
    if normalized.is_empty() {
        return None;
    }
    let body = if normalized.contains('[') || normalized.contains(']') {
        let close_idx = normalized.find(']')?;
        if !normalized.starts_with('[') || close_idx <= 1 {
            return None;
        }
        let body = normalized.get(1..close_idx)?;
        let suffix = normalized.get(close_idx + 1..)?;
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

    if parse_adduct_polarity(Some(adduct)) == Some(Polarity::Negative) {
        if body == "M+FA" {
            return Some((1.0, FORMATE_MASS));
        }
        if body == "M+HAC" {
            return Some((1.0, ACETATE_MASS));
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

    (!saw_unsupported_token && saw_molecule_term && !body.is_empty()).then_some((multiplier, shift))
}

fn apply_adduct_token(
    current: &str,
    sign: f64,
    multiplier: &mut f64,
    shift: &mut f64,
    saw_molecule_term: &mut bool,
    saw_unsupported_token: &mut bool,
) {
    if current.is_empty() {
        *saw_unsupported_token = true;
        return;
    }
    if let Some(token_mass) = parse_adduct_term_mass(current) {
        *shift = sign.mul_add(token_mass, *shift);
    } else if let Some(token_multiplier) = parse_multiplicity_token(current)
        && sign > 0.0
        && !*saw_molecule_term
    {
        *multiplier = token_multiplier;
        *saw_molecule_term = true;
    } else {
        *saw_unsupported_token = true;
    }
}

fn is_valid_adduct_suffix(suffix: &str) -> bool {
    suffix
        .chars()
        .all(|c| c.is_ascii_digit() || c == '+' || c == '-')
        && !(suffix.contains('+') && suffix.contains('-'))
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
    let mult = trimmed.get(..digits_end)?;
    let rem = trimmed.get(digits_end..)?;
    if !rem.eq_ignore_ascii_case("M") {
        return None;
    }
    let count = mult.parse::<u32>().ok()?;
    (count > 0).then(|| f64::from(count))
}

pub(crate) fn parse_charge_value(charge: Option<&str>, adduct: Option<&str>) -> Option<f64> {
    charge.and_then(parse_charge_magnitude).or_else(|| {
        adduct?
            .trim()
            .split_once(']')
            .and_then(|(_, suffix)| parse_charge_magnitude(suffix))
    })
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

fn parse_ascii_digits_f64(text: &str) -> Option<f64> {
    let mut acc = 0u64;
    let mut saw_digit = false;
    for byte in text.bytes() {
        if byte.is_ascii_digit() {
            saw_digit = true;
            acc = acc
                .saturating_mul(10)
                .saturating_add(u64::from(byte - b'0'));
        } else if saw_digit {
            break;
        }
    }
    (saw_digit && acc > 0).then(|| f64::from(u32::try_from(acc).unwrap_or(u32::MAX)))
}
