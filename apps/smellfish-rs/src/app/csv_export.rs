// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! CSV export: build a downloadable CSV string from molecule rows and
//! trigger a browser download via a data-URI `eval` call.
//!
//! This is the output-side counterpart to the crate-level `csv` module
//! (which parses CSV *input*).  Kept separate so the parsing and
//! serialisation concerns never co-mingle in one file.

use crate::model::MoleculeRow;
#[cfg(target_arch = "wasm32")]
use std::fmt::Write;

/// Machine-readable category for CSV export — strips emojis and
/// normalises to "likely", "neutral", "caution", "skeptical", or "fishy".
///
/// Order matters: `"citation needed"` is matched *before* `"lotus"` so that a
/// LOTUS-backed molecule that still reads "citation needed" is not misfiled
/// as "likely"; `"synthetic-leaning"` is matched before both so a structural
/// warning is an orange `caution`, not green.
#[cfg(any(test, target_arch = "wasm32"))]
#[must_use]
pub fn category(verdict: &str) -> &'static str {
    let l = verdict.to_ascii_lowercase();

    // RED — Highly synthetic / fishy (check first!).
    if l.contains("highly synthetic") || l.contains("smells fishy") {
        return "fishy";
    }

    // ORANGE — Synthetic-leaning structural warning.
    if l.contains("synthetic-leaning") {
        return "caution";
    }

    // YELLOW — Skeptical (needs citation). Before `lotus` so a LOTUS-backed
    // molecule that still reads "citation needed" is not filed as "likely".
    if l.contains("citation needed") {
        return "skeptical";
    }

    // GREEN — High NP confidence (LOTUS or strong structural + Ertl score).
    if l.contains("lotus") {
        return "likely";
    }
    if l.contains("likely hit") || l.contains("likely novel") {
        return "likely";
    }
    if l.contains("pubchem + strong") {
        return "likely";
    }
    if l.contains("strong np score") && !l.contains("weak") {
        return "likely";
    }

    // BLUE — Moderate NP confidence (PubChem with some NP features).
    if l.contains("pubchem with") || l.contains("pubchem + np") {
        return "neutral";
    }

    // RED — Low/weak NP confidence.
    if l.contains("weak np signals")
        || (l.contains("pubchem") && l.contains("weak"))
        || (l.contains("ertl") && l.contains("-1"))
    {
        return "caution";
    }

    "neutral"
}

/// Escape a field for CSV output: if it contains a comma or double-quote,
/// wrap in quotes and double any inner quotes.
#[cfg(target_arch = "wasm32")]
pub fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') {
        let escaped = s.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        s.to_string()
    }
}

/// Build a CSV string from molecule rows.
#[cfg(target_arch = "wasm32")]
pub fn build_csv(rows: &[MoleculeRow]) -> String {
    let mut csv = String::from(
        "label,smiles,np_score,np_label,np_confidence,ring_family,substituents,locus,verdict_category,chemist_checks\n",
    );
    for r in rows {
        let checks = r
            .chemist_checks
            .iter()
            .map(|c| format!("{}:{}", c.name, c.status))
            .collect::<Vec<_>>()
            .join(";");
        let substituents: String = r
            .substituents_counts
            .iter()
            .map(|(label, count)| format!("{label}({count})"))
            .collect::<Vec<_>>()
            .join(";");
        let locus = r
            .lotus_compounds
            .iter()
            .chain(r.pubchem_cids.iter())
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(";");
        let _ = writeln!(
            csv,
            "{},{},{:.3},{},{}%,{},{},{},{},{}",
            escape_csv(&r.label),
            escape_csv(&r.smiles),
            r.np_likeness,
            r.np_label,
            (r.np_confidence * 100.0).round(),
            escape_csv(&r.ring_family),
            escape_csv(&substituents),
            escape_csv(&locus),
            category(&r.verdict),
            escape_csv(&checks),
        );
    }
    csv
}

/// Build a CSV string from molecule rows and trigger a browser download
/// via a data-URI injected through `eval`.
#[cfg(target_arch = "wasm32")]
pub fn download_csv(rows: &[MoleculeRow]) {
    let csv = build_csv(rows);
    let url = format!("data:text/csv;charset=utf-8,{}", urlencoding::encode(&csv));
    let script = format!(
        r"(function(){{var a=document.createElement('a');a.href='{url}';a.download='smellfish-results.csv';a.click();}})()"
    );
    let _ = js_sys::eval(&script);
}

#[cfg(not(target_arch = "wasm32"))]
pub const fn download_csv(_rows: &[MoleculeRow]) {}

#[cfg(test)]
mod tests {
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
}
