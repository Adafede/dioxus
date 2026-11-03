// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! CSV export: build a downloadable CSV string from molecule rows and
//! trigger a browser download via [`upload::download_text`].
//!
//! This is the output-side counterpart to the crate-level `csv` module
//! (which parses CSV *input*).  Kept separate so the parsing and
//! serialisation concerns never co-mingle in one file.

use crate::model::MoleculeRow;
use std::fmt::Write;

/// Machine-readable category for CSV export — strips emojis and
/// normalises to "likely", "neutral", "caution", "skeptical", or "fishy".
///
/// Order matters: `"citation needed"` is matched *before* `"lotus"` so that a
/// LOTUS-backed molecule that still reads "citation needed" is not misfiled
/// as "likely"; `"synthetic-leaning"` is matched before both so a structural
/// warning is an orange `caution`, not green.
#[must_use]
pub(super) fn category(verdict: &str) -> &'static str {
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

/// Escape a field for CSV output: if it contains a comma, double-quote, or
/// line break, wrap in quotes and double any inner quotes.
///
/// The line break matters even though `build_csv` writes `writeln!`: a field
/// whose *value* holds `\n` or `\r` — a SMILES comment, a label pasted from a
/// spreadsheet — silently starts a new row otherwise, corrupting every row
/// after it. Matching it here is what RFC 4180 requires a writer to do.
#[cfg(test)]
pub(crate) fn escape_csv(s: &str) -> String {
    let mut out = String::new();
    push_csv_escaped(&mut out, s);
    out
}

fn push_csv_escaped(out: &mut String, s: &str) {
    if s.bytes().any(|b| matches!(b, b',' | b'"' | b'\n' | b'\r')) {
        out.reserve(s.len() + 2);
        out.push('"');
        for ch in s.chars() {
            if ch == '"' {
                out.push_str("\"\"");
            } else {
                out.push(ch);
            }
        }
        out.push('"');
    } else {
        out.push_str(s);
    }
}

/// Build a CSV string from molecule rows.
#[must_use]
pub(crate) fn build_csv(rows: &[MoleculeRow]) -> String {
    let mut csv = String::with_capacity(128 + rows.len() * 128);
    csv.push_str(
        "label,smiles,np_score,np_label,np_confidence,ring_family,substituents,locus,verdict_category,chemist_checks\n",
    );
    let mut scratch = String::new();
    for r in rows {
        push_csv_escaped(&mut csv, &r.label);
        csv.push(',');
        push_csv_escaped(&mut csv, &r.smiles);
        let _ = write!(
            csv,
            ",{:.3},{},{}%,",
            r.np_likeness,
            r.np_label,
            (r.np_confidence * 100.0).round()
        );
        push_csv_escaped(&mut csv, &r.ring_family);
        csv.push(',');

        scratch.clear();
        for (idx, (label, count)) in r.substituents_counts.iter().enumerate() {
            if idx > 0 {
                scratch.push(';');
            }
            let _ = write!(scratch, "{label}({count})");
        }
        push_csv_escaped(&mut csv, &scratch);
        csv.push(',');

        scratch.clear();
        for (idx, item) in r
            .lotus_compounds
            .iter()
            .chain(r.pubchem_cids.iter())
            .enumerate()
        {
            if idx > 0 {
                scratch.push(';');
            }
            scratch.push_str(item);
        }
        push_csv_escaped(&mut csv, &scratch);
        let _ = write!(csv, ",{},", category(&r.verdict));

        scratch.clear();
        for (idx, c) in r.chemist_checks.iter().enumerate() {
            if idx > 0 {
                scratch.push(';');
            }
            scratch.push_str(c.name);
            scratch.push(':');
            scratch.push_str(c.status);
        }
        push_csv_escaped(&mut csv, &scratch);
        csv.push('\n');
    }
    csv
}

/// Build a CSV string from molecule rows and trigger a browser download.
pub(super) fn download_csv(rows: &[MoleculeRow]) {
    let csv = build_csv(rows);
    drop(upload::download_text(&csv, "smellfish-results.csv"));
}

#[cfg(test)]
mod tests;
