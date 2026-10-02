// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

// The `tests` tests, extracted from `csv_export.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `csv_export` module, so
// `use super::*` below reaches exactly what it did before the move.
use super::category;

#[test]
fn categorizes_high_quality_pubchem() {
    let verdict = "🌿 PubChem + strong NP evidence — Ertl score +1.50 with 3 NP substituent(s) + 2 NP motif(s).";
    assert_eq!(category(verdict), "likely");
}

#[test]
fn categorizes_moderate_quality_pubchem() {
    let verdict =
        "📚 PubChem hit with NP signals — Ertl score +0.80, 2 substituent(s), 1 NP motif(s).";
    assert_eq!(category(verdict), "neutral");
}

#[test]
fn categorizes_weak_pubchem() {
    let verdict = "📚 PubChem hit — weak NP evidence (Ertl score +0.26).";
    assert_eq!(category(verdict), "caution");
}

#[test]
fn categorizes_lotus_backed() {
    let verdict = "🌿 LOTUS-backed (Ertl score +1.23).";
    assert_eq!(category(verdict), "likely");
}

#[test]
fn categorizes_novel_candidate() {
    let verdict =
        "🌿 Likely hit — strong NP-likeness (+3.43) + NP-like scaffold, not yet in databases.";
    assert_eq!(category(verdict), "likely");
}

#[test]
fn categorizes_fishy() {
    let verdict = "👃 Smells fishy (Ertl score -1.23). Citation needed.";
    assert_eq!(category(verdict), "fishy");
}

#[test]
fn categorizes_synthetic_leaning_as_caution() {
    let verdict = "🟧 Synthetic-leaning structure (Ertl score +2.50).";
    assert_eq!(category(verdict), "caution");
    assert_ne!(category(verdict), "likely");
    assert_ne!(category(verdict), "fishy");
}

#[test]
fn categorizes_lotus_scaffold_hint_as_skeptical() {
    let verdict =
        "👃 Citation needed — LOTUS scaffold hint, insufficient corroboration (Ertl +2.50).";
    assert_eq!(category(verdict), "skeptical");
}

#[cfg(target_arch = "wasm32")]
#[test]
fn escape_csv_wraps_fields_with_commas() {
    let escaped = super::escape_csv("hello, world");
    assert_eq!(escaped, "\"hello, world\"");
}

#[cfg(target_arch = "wasm32")]
#[test]
fn escape_csv_does_not_wrap_simple_fields() {
    let escaped = super::escape_csv("hello");
    assert_eq!(escaped, "hello");
}

#[cfg(target_arch = "wasm32")]
#[test]
fn escape_csv_escapes_internal_quotes() {
    let escaped = super::escape_csv("say \"hi\"");
    assert_eq!(escaped, "\"say \"\"hi\"\"\"");
}
