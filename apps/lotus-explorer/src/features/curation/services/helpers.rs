// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use super::*;

// -- SPARQL / QS helpers -------------------------------------------------------

pub(super) fn extract_first_qid(
    raw_json: &str,
    var_name: &str,
) -> Result<Option<String>, CurationError> {
    let json =
        serde_json::from_str::<Value>(raw_json).map_err(|e| CurationError::Parse(e.to_string()))?;
    let qid = json
        .get("results")
        .and_then(|v| v.get("bindings"))
        .and_then(Value::as_array)
        .and_then(|arr| arr.first())
        .and_then(|b| b.get(var_name))
        .and_then(|v| v.get("value"))
        .and_then(Value::as_str)
        .and_then(extract_qid_from_uri)
        .map(ToOwned::to_owned);
    Ok(qid)
}

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

pub(super) fn escape_sparql_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

pub(super) fn escape_qs_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Format a Wikidata QuickStatements mass statement using the dalton unit (Q483261).
/// Unit syntax is `U<QID>` - there is NO leading `Q` after the `U`.
pub(crate) fn qs_mass_statement(subject: &str, mass: f64) -> String {
    format!("{subject}|P2067|+{mass:.6}U483261")
}

// -- Text / chemistry normalization --------------------------------------------

pub(super) fn normalize_doi(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let lowered = trimmed.to_ascii_lowercase();
    let canonical = if let Some(idx) = lowered.find("doi.org/") {
        &trimmed[(idx + 8)..]
    } else {
        trimmed
    };
    if canonical.is_empty() {
        return None;
    }
    Some(canonical.to_ascii_uppercase())
}

pub(super) fn normalize_taxon_lookup_key(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_ascii_lowercase())
}

pub(super) fn has_stereo_marks(smiles: &str) -> bool {
    smiles.contains('@') || smiles.contains('/') || smiles.contains('\\')
}

pub(super) fn has_isomeric_smiles(smiles: &str) -> bool {
    has_stereo_marks(smiles)
}

pub(crate) fn extract_formula_from_inchi(inchi: &str) -> Option<String> {
    let cleaned = inchi.trim();
    if cleaned.is_empty() || !cleaned.starts_with("InChI=") {
        return None;
    }
    let right = cleaned.split('/').nth(1)?;
    if right.is_empty() {
        return None;
    }
    Some(right.to_string())
}

pub(crate) fn normalize_formula_for_wikidata(value: &str) -> String {
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
