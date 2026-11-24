// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! CSV export and legacy verdict-text category parsing.

use crate::model::{MoleculeRow, VerdictCategory, contains_ignore_ascii_case, matches_any};
use std::borrow::Cow;
use std::fmt::Write;

/// Maps an arbitrary verdict string to a [`VerdictCategory`] for compatibility with
/// pre-refactor verdict strings.
#[must_use]
pub fn verdict_category_from_text(verdict: &str) -> VerdictCategory {
    if matches_any(verdict, &["highly synthetic", "smells fishy"]) {
        VerdictCategory::Fishy
    } else if contains_ignore_ascii_case(verdict, "synthetic-leaning") {
        VerdictCategory::Caution
    } else if contains_ignore_ascii_case(verdict, "citation needed") {
        VerdictCategory::Skeptical
    } else if matches_any(
        verdict,
        &["lotus", "likely", "strong natural", "pubchem + strong"],
    ) || (contains_ignore_ascii_case(verdict, "strong np score")
        && !contains_ignore_ascii_case(verdict, "weak"))
    {
        VerdictCategory::Likely
    } else if matches_any(verdict, &["pubchem with", "pubchem + np"]) {
        VerdictCategory::Neutral
    } else if contains_ignore_ascii_case(verdict, "weak np signals")
        || (contains_ignore_ascii_case(verdict, "pubchem")
            && contains_ignore_ascii_case(verdict, "weak"))
        || (contains_ignore_ascii_case(verdict, "ertl")
            && (verdict.contains("-1") || verdict.contains('\u{2212}')))
    {
        VerdictCategory::Caution
    } else {
        VerdictCategory::Neutral
    }
}

/// Escapes a CSV field according to RFC 4180 and prefixes spreadsheet formula triggers.
#[must_use]
pub fn escape_csv(value: &str) -> String {
    let mut out = String::new();
    push_csv_escaped(&mut out, value);
    out
}

fn push_csv_escaped(out: &mut String, s: &str) {
    let safe = if matches!(
        s.as_bytes().first(),
        Some(b'=' | b'+' | b'-' | b'@' | b'\t' | b'\r')
    ) {
        Cow::Owned(format!("'{s}"))
    } else {
        Cow::Borrowed(s)
    };
    let text = safe.as_ref();
    if text
        .bytes()
        .any(|b| matches!(b, b',' | b'"' | b'\n' | b'\r'))
    {
        out.reserve(text.len() + 2);
        out.push('"');
        for ch in text.chars() {
            if ch == '"' {
                out.push_str("\"\"");
            } else {
                out.push(ch);
            }
        }
        out.push('"');
    } else {
        out.push_str(text);
    }
}

/// Formats assessed molecule rows into a downloadable CSV report.
#[must_use]
pub fn build_csv(rows: &[MoleculeRow]) -> String {
    let mut csv = String::with_capacity(128 + rows.len() * 128);
    csv.push_str(
        "label,smiles,np_score,np_label,np_confidence,ring_family,substituents,locus,verdict_category,chemist_checks\n",
    );
    let mut scratch = String::new();
    for r in rows {
        push_csv_escaped(&mut csv, &r.label);
        csv.push(',');
        push_csv_escaped(&mut csv, &r.smiles);
        let (score, confidence_pct) = r
            .np_score
            .map_or((0.0, 0.0), |np| (np.score, (np.confidence * 100.0).round()));
        let _ = write!(csv, ",{score:.3},{},{confidence_pct}%,", r.np_label);
        push_csv_escaped(&mut csv, r.ring_family.as_str());
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
            .map(|c| c.qid.as_str())
            .chain(r.pubchem_cids.iter().map(String::as_str))
            .enumerate()
        {
            if idx > 0 {
                scratch.push(';');
            }
            scratch.push_str(item);
        }
        push_csv_escaped(&mut csv, &scratch);
        let _ = write!(csv, ",{},", r.verdict.category.csv_label());

        scratch.clear();
        for (idx, c) in r.chemist_checks.iter().enumerate() {
            if idx > 0 {
                scratch.push(';');
            }
            scratch.push_str(c.name);
            scratch.push(':');
            scratch.push_str(c.status.as_str());
        }
        push_csv_escaped(&mut csv, &scratch);
        csv.push('\n');
    }
    csv
}
