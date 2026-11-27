// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Adduct-token grammar, polarity classification, and theoretical precursor *m/z*.

pub(crate) mod parse;

use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::BuildHasher;

const CANONICAL_ADDUCTS: &[(&str, &[&str], &str)] = &[
    ("M", &["", "+"], "[M]+"),
    ("M", &["++", "2+"], "[M]2+"),
    ("M+2NA", &["", "2+", "++"], "[M+2Na]2+"),
    ("M+H", &["", "+"], "[M+H]+"),
    ("M+K", &["", "+"], "[M+K]+"),
    ("M+NH4", &["", "+"], "[M+NH4]+"),
    ("M+NA", &["", "+"], "[M+Na]+"),
    ("M+2H", &["", "++", "2+"], "[M+2H]2+"),
    ("M-H", &["", "-", "1-"], "[M-H]-"),
    ("M-2H", &["", "2-", "--"], "[M-2H]2-"),
    ("4M-H", &["", "-", "1-"], "[4M-H]-"),
];

/// Ion polarity mode (`+` / `positive` vs `-` / `negative`).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum Polarity {
    /// Positively charged ion mode.
    Positive,
    /// Negatively charged ion mode.
    Negative,
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
    pub(crate) fn from_label(label: &str) -> Self {
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
        Some(Polarity::Negative) => -parse::HYDROGEN_MASS * charge_value,
        Some(Polarity::Positive) => parse::HYDROGEN_MASS * charge_value,
        None => 0.0,
    };
    (1.0, shift)
}

/// Computes expected precursor *m/z* using an already-resolved `(multiplier, shift)` spec.
#[must_use]
pub(crate) fn expected_precursor_mz_with_spec(
    neutral_mass: f64,
    adduct_spec: Option<(f64, f64)>,
    adduct: Option<&str>,
    charge: Option<&str>,
    ion_mode: Option<&str>,
) -> f64 {
    let charge_value = parse::parse_charge_value(charge, adduct).unwrap_or(1.0);
    let polarity =
        parse::parse_polarity_tokens(charge, ion_mode).or_else(|| parse_adduct_polarity(adduct));

    let (multiplier, shift) =
        adduct_spec.unwrap_or_else(|| fallback_proton_spec(polarity, charge_value));
    let electron_adjustment = match polarity {
        Some(Polarity::Positive) => -parse::ELECTRON_MASS * charge_value,
        Some(Polarity::Negative) => parse::ELECTRON_MASS * charge_value,
        None => 0.0,
    };

    let base_mz = neutral_mass.mul_add(multiplier, shift) + electron_adjustment;
    base_mz / charge_value.max(1.0)
}

/// Computes expected precursor *m/z* from neutral mass and optional adduct, charge, and ion mode.
#[cfg(test)]
#[must_use]
pub(crate) fn expected_precursor_mz(
    neutral_mass: f64,
    adduct: Option<&str>,
    charge: Option<&str>,
    ion_mode: Option<&str>,
) -> f64 {
    let adduct_spec = adduct
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .and_then(parse::parse_adduct_mass_components);
    expected_precursor_mz_with_spec(neutral_mass, adduct_spec, adduct, charge, ion_mode)
}

/// Extracts [`Polarity`] from an adduct string (suffix after `]` or known unbracketed suffix).
#[must_use]
pub(crate) fn parse_adduct_polarity(adduct: Option<&str>) -> Option<Polarity> {
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
    parse::parse_unbracketed_polarity(adduct)
}

pub(crate) fn resolve_adduct_mass_spec_cached(
    adduct: &str,
    cache: &mut HashMap<String, Option<(f64, f64)>, impl BuildHasher>,
) -> Option<(f64, f64)> {
    let trimmed = adduct.trim();
    if trimmed.is_empty() {
        return None;
    }
    let key: Cow<'_, str> = if trimmed.contains(' ') {
        Cow::Owned(trimmed.replace(' ', ""))
    } else {
        Cow::Borrowed(trimmed)
    };
    if let Some(&mass_spec) = cache.get(key.as_ref()) {
        return mass_spec;
    }
    let mass_spec = parse::parse_adduct_mass_components(&key);
    cache.insert(key.into_owned(), mass_spec);
    mass_spec
}

/// Normalizes common adduct spellings into canonical notation (`[M+H]+`, `[M+Na]+`, etc.).
#[must_use]
pub(crate) fn normalize_adduct_label(adduct: &str) -> &str {
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

    for &(canonical_body, valid_suffixes, normalized) in CANONICAL_ADDUCTS {
        if body.eq_ignore_ascii_case(canonical_body) && valid_suffixes.contains(&suffix) {
            return normalized;
        }
    }
    trimmed
}

/// Returns `true` when `adduct` parses into a valid mass transformation.
#[cfg(test)]
#[must_use]
pub(crate) fn is_supported_adduct(adduct: &str) -> bool {
    parse::parse_adduct_mass_components(adduct).is_some()
}
