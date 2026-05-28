// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use super::*;

// -- SPARQL / QS helpers -------------------------------------------------------

pub(super) fn extract_qid_from_uri(uri: &str) -> Option<&str> {
    uri.rsplit('/').next().filter(|segment| {
        segment.starts_with('Q') && segment[1..].bytes().all(|b| b.is_ascii_digit())
    })
}

pub(super) fn binding_value(binding: &Value, key: &str) -> Option<String> {
    binding
        .get(key)
        .and_then(|v| v.get("value"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

/// Escape a string literal for use inside a SPARQL double-quoted string.
///
/// Backslashes are doubled and double-quotes are backslash-escaped.
/// A single forward pass avoids two intermediate heap allocations.
pub(super) fn escape_sparql_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 4);
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            other => out.push(other),
        }
    }
    out
}

/// Escape a string literal for use inside a QuickStatements statement value.
///
/// Uses the same escaping rules as SPARQL double-quoted strings.
pub(super) fn escape_qs_string(value: &str) -> String {
    escape_sparql_string(value)
}

/// Format a Wikidata QuickStatements mass statement using the dalton unit (Q483261).
/// Unit syntax is `U<QID>` - there is NO leading `Q` after the `U`.
pub fn qs_mass_statement(subject: &str, mass: f64) -> String {
    format!("{subject}|P2067|+{mass:.6}U483261")
}

// -- Text / chemistry normalization --------------------------------------------

pub(super) fn normalize_doi(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let canonical = find_ascii_ci(trimmed, b"doi.org/").map_or(trimmed, |idx| &trimmed[(idx + 8)..]);
    if canonical.is_empty() {
        return None;
    }
    Some(canonical.to_ascii_uppercase())
}

pub(super) fn find_ascii_ci(haystack: &str, needle: &[u8]) -> Option<usize> {
    let hb = haystack.as_bytes();
    if needle.is_empty() || hb.len() < needle.len() {
        return None;
    }
    hb.windows(needle.len())
        .position(|w| w.iter().zip(needle).all(|(a, b)| a.eq_ignore_ascii_case(b)))
}

pub(super) fn has_stereo_marks(smiles: &str) -> bool {
    smiles.contains('@') || smiles.contains('/') || smiles.contains('\\')
}

pub(super) fn has_isomeric_smiles(smiles: &str) -> bool {
    has_stereo_marks(smiles)
}

pub fn extract_formula_from_inchi(inchi: &str) -> Option<String> {
    let cleaned = inchi.trim();
    if cleaned.is_empty() || !cleaned.starts_with("InChI=") {
        return None;
    }
    let right = cleaned.split('/').nth(1)?;
    if right.is_empty() {
        return None;
    }
    Some(right.into())
}

pub fn normalize_formula_for_wikidata(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            '0' => '₀',
            '1' => '₁',
            '2' => '₂',
            '3' => '₃',
            '4' => '₄',
            '5' => '₅',
            '6' => '₆',
            '7' => '₇',
            '8' => '₈',
            '9' => '₉',
            _ => ch,
        })
        .collect()
}
