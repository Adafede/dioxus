// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Adduct-token grammar, polarity classification, and theoretical precursor *m/z*.

mod parse;

use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::str::FromStr;

use crate::error::MgfAnalysisError;

pub use parse::{
    AMMONIUM_MASS, ELECTRON_MASS, HYDROGEN_MASS, NITROGEN_MASS, POTASSIUM_MASS, SODIUM_MASS,
    parse_adduct_term_mass, parse_polarity_tokens, parse_unbracketed_polarity,
};

thread_local! {
    static ADDUCT_SPEC_CACHE: RefCell<HashMap<String, Option<(f64, f64)>>> =
        RefCell::new(HashMap::new());
}

/// Ion polarity mode (`+` / `positive` vs `-` / `negative`).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Polarity {
    /// Positively charged ion mode.
    Positive,
    /// Negatively charged ion mode.
    Negative,
}

impl FromStr for Polarity {
    type Err = MgfAnalysisError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.eq_ignore_ascii_case("positive") || trimmed == "+" {
            Ok(Self::Positive)
        } else if trimmed.eq_ignore_ascii_case("negative") || trimmed == "-" {
            Ok(Self::Negative)
        } else {
            Err(MgfAnalysisError::InvalidPolarity(s.to_string()))
        }
    }
}

/// Parsed adduct formula transformation (`m/z = (M * multiplier + shift ± z*e) / z`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AdductSpec {
    /// Molecule stoichiometry multiplier (e.g., `2.0` for `[2M+H]+`).
    pub multiplier: f64,
    /// Net exact-mass shift from added/lost adduct species (Da, before electron adjustment).
    pub shift: f64,
    /// Inferred ion polarity, when specified on the adduct suffix or form.
    pub polarity: Option<Polarity>,
    /// Inferred charge magnitude, when specified in brackets/suffix.
    pub charge: Option<f64>,
}

impl FromStr for AdductSpec {
    type Err = MgfAnalysisError;

    fn from_str(adduct: &str) -> Result<Self, Self::Err> {
        let (multiplier, shift) = parse::parse_adduct_mass_components(adduct).ok_or_else(|| {
            MgfAnalysisError::InvalidAdduct {
                spec: adduct.to_string(),
            }
        })?;
        let polarity = parse_adduct_polarity(Some(adduct));
        let charge = parse::parse_charge_value(None, Some(adduct));
        Ok(Self {
            multiplier,
            shift,
            polarity,
            charge,
        })
    }
}

/// Coarse chemical grouping of adduct forms for scatter legends and diagnostics.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum AdductFamily {
    /// `[M+H]+` / `[M+2H]2+` protonated species.
    Protonated,
    /// `[M-H]-` / `[M-2H]2-` deprotonated species.
    Deprotonated,
    /// `[M+Na]+` / `[M+K]+` / `[M+NH4]+` alkali/ammonium adducts.
    AlkaliAmmonium,
    /// `Mg` / `Ca` / `Fe` metal complexes.
    MetalComplex,
    /// `Cl` / `Br` halide adducts.
    Halide,
    /// Uncategorized or complex adduct annotations.
    Other,
}

impl AdductFamily {
    /// Classify a raw or normalized adduct label into its [`AdductFamily`].
    #[must_use]
    pub fn from_label(label: &str) -> Self {
        fn has_ascii_ci(haystack: &str, needle: &str) -> bool {
            haystack
                .as_bytes()
                .windows(needle.len())
                .any(|window| window.eq_ignore_ascii_case(needle.as_bytes()))
        }

        let trimmed = label.trim();
        let compact: Cow<'_, str> = if trimmed.contains(' ') {
            Cow::Owned(trimmed.replace(' ', ""))
        } else {
            Cow::Borrowed(trimmed)
        };
        let s = compact.as_ref();
        if has_ascii_ci(s, "[M+H]") || has_ascii_ci(s, "[M+2H]") {
            Self::Protonated
        } else if has_ascii_ci(s, "[M-H]") || has_ascii_ci(s, "[M-2H]") {
            Self::Deprotonated
        } else if has_ascii_ci(s, "[M+NA]")
            || has_ascii_ci(s, "[M+K]")
            || has_ascii_ci(s, "[M+NH4]")
        {
            Self::AlkaliAmmonium
        } else if has_ascii_ci(s, "MG") || has_ascii_ci(s, "CA") || has_ascii_ci(s, "FE") {
            Self::MetalComplex
        } else if has_ascii_ci(s, "CL") || has_ascii_ci(s, "BR") {
            Self::Halide
        } else {
            Self::Other
        }
    }

    /// Canonical display name for legends and diagnostic tables.
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

    /// Stable legend ordering rank (`0..=5`).
    #[must_use]
    pub const fn rank(self) -> usize {
        match self {
            Self::Protonated => 0,
            Self::Deprotonated => 1,
            Self::AlkaliAmmonium => 2,
            Self::MetalComplex => 3,
            Self::Halide => 4,
            Self::Other => 5,
        }
    }
}

fn fallback_proton_spec(polarity: Option<Polarity>, charge_value: f64) -> (f64, f64) {
    let shift = match polarity {
        Some(Polarity::Negative) => -HYDROGEN_MASS * charge_value,
        Some(Polarity::Positive) => HYDROGEN_MASS * charge_value,
        None => 0.0,
    };
    (1.0, shift)
}

/// Computes expected precursor *m/z* from neutral mass and optional adduct, charge, and ion mode.
#[must_use]
pub fn expected_precursor_mz(
    neutral_mass: f64,
    adduct: Option<&str>,
    charge: Option<&str>,
    ion_mode: Option<&str>,
) -> f64 {
    let normalized_adduct = adduct.unwrap_or("").trim();
    let charge_value = parse::parse_charge_value(charge, adduct).unwrap_or(1.0);
    let polarity =
        parse_polarity_tokens(charge, ion_mode).or_else(|| parse_adduct_polarity(adduct));

    let (multiplier, shift) = if normalized_adduct.is_empty() {
        fallback_proton_spec(polarity, charge_value)
    } else {
        parse_adduct_mass_spec_cached(normalized_adduct)
            .unwrap_or_else(|| fallback_proton_spec(polarity, charge_value))
    };
    let electron_adjustment = match polarity {
        Some(Polarity::Positive) => -ELECTRON_MASS * charge_value,
        Some(Polarity::Negative) => ELECTRON_MASS * charge_value,
        None => 0.0,
    };

    let base_mz = neutral_mass.mul_add(multiplier, shift) + electron_adjustment;
    base_mz / charge_value.max(1.0)
}

/// Extracts [`Polarity`] from an adduct string (suffix after `]` or known unbracketed suffix).
#[must_use]
pub fn parse_adduct_polarity(adduct: Option<&str>) -> Option<Polarity> {
    let adduct = adduct?.trim();
    if let Some((_, suffix)) = adduct.split_once(']') {
        let suffix = suffix.trim();
        if suffix.contains('-') {
            return Some(Polarity::Negative);
        }
        if suffix.contains('+') {
            return Some(Polarity::Positive);
        }
        return None;
    }
    parse_unbracketed_polarity(adduct)
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
        let mass_spec = parse::parse_adduct_mass_components(&key);
        cache.insert(key.into_owned(), mass_spec);
        mass_spec
    })
}

/// Normalizes common adduct spellings into canonical notation (`[M+H]+`, `[M+Na]+`, etc.).
#[must_use]
pub fn normalize_adduct_label(adduct: &str) -> &str {
    let trimmed = adduct.trim();
    if trimmed.is_empty() {
        return "unknown";
    }
    let (body, suffix) = trimmed.find(']').map_or((trimmed, ""), |idx| {
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

/// Returns `true` when `adduct` parses into a valid mass transformation.
#[must_use]
pub fn is_supported_adduct(adduct: &str) -> bool {
    parse_adduct_mass_spec_cached(adduct).is_some()
}
