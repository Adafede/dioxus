// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! CSV input parsing and sanitized CSV export.

use crate::error::SmellfishError;
use crate::model::{MoleculeRow, RawRow};
use std::borrow::Cow;
use std::fmt::Write;

/// Parse CSV text or newline-delimited SMILES into [`RawRow`] values, auto-detecting columns.
///
/// # Errors
///
/// Returns [`SmellfishError::EmptyInput`], [`SmellfishError::UnterminatedCsvQuote`],
/// or [`SmellfishError::NoDataRows`].
pub fn parse_csv_rows(text: &str) -> Result<Vec<RawRow>, SmellfishError> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(SmellfishError::EmptyInput);
    }
    if looks_like_smiles_list(trimmed) {
        return Ok(parse_smiles_lines(trimmed));
    }

    let mut records = parse_csv_records(trimmed)?.into_iter();
    let Some(headers) = records.next() else {
        return Err(SmellfishError::EmptyInput);
    };
    let smiles_idx = detect_column(
        &headers,
        &["smiles", "smile", "structure", "canonical_smiles"],
    )
    .unwrap_or(0);
    let label_idx = detect_column(&headers, &["name", "label", "id", "compound"]);

    let rows: Vec<RawRow> = records
        .enumerate()
        .map(|(line, record)| RawRow {
            index: line + 1,
            label: label_for_record(&record, label_idx, line + 1),
            smiles: record
                .get(smiles_idx)
                .map_or("", String::as_str)
                .trim()
                .to_string(),
        })
        .collect();

    if rows.is_empty() {
        return Err(SmellfishError::NoDataRows);
    }
    Ok(rows)
}

/// Parses RFC 4180 CSV text into raw string records without indexing panics.
///
/// # Errors
///
/// Returns [`SmellfishError::UnterminatedCsvQuote`] when a quoted cell runs past EOF.
pub(crate) fn parse_csv_records(text: &str) -> Result<Vec<Vec<String>>, SmellfishError> {
    let mut records = Vec::new();
    let mut current_record = Vec::new();
    let mut current_field = String::new();
    let mut in_quotes = false;
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if in_quotes {
            if ch == '"' && chars.peek() == Some(&'"') {
                let _ = chars.next();
                current_field.push('"');
            } else if ch == '"' {
                in_quotes = false;
            } else {
                current_field.push(ch);
            }
            continue;
        }
        match ch {
            '"' if current_field.trim().is_empty() => {
                current_field.clear();
                in_quotes = true;
            }
            ',' => current_record.push(std::mem::take(&mut current_field)),
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    let _ = chars.next();
                }
                finish_csv_row(&mut records, &mut current_record, &mut current_field);
            }
            '\n' => finish_csv_row(&mut records, &mut current_record, &mut current_field),
            _ => current_field.push(ch),
        }
    }

    if in_quotes {
        return Err(SmellfishError::UnterminatedCsvQuote);
    }
    if !current_field.is_empty() || !current_record.is_empty() {
        finish_csv_row(&mut records, &mut current_record, &mut current_field);
    }
    Ok(records)
}

fn finish_csv_row(
    records: &mut Vec<Vec<String>>,
    current_record: &mut Vec<String>,
    current_field: &mut String,
) {
    current_record.push(std::mem::take(current_field));
    if current_record.len() > 1 || current_record.first().is_some_and(|f| !f.is_empty()) {
        records.push(std::mem::take(current_record));
    } else {
        current_record.clear();
    }
}

fn looks_like_smiles_list(text: &str) -> bool {
    let mut lines = text.lines().map(str::trim).filter(|line| !line.is_empty());
    let Some(first) = lines.next() else {
        return false;
    };
    is_plain_smiles_line(first) && lines.all(is_plain_smiles_line)
}

fn is_plain_smiles_line(line: &str) -> bool {
    !line.contains(',')
        && !line.contains('\t')
        && !line.contains(';')
        && !strip_header_prefix(line).eq_ignore_ascii_case("smiles")
}

fn strip_header_prefix(header: &str) -> &str {
    header
        .trim()
        .trim_start_matches('\u{feff}')
        .trim_start_matches(|c: char| ['>', '#', '"', '\'', ':', ';'].contains(&c))
        .trim()
}

fn parse_smiles_lines(text: &str) -> Vec<RawRow> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .enumerate()
        .map(|(idx, smiles)| RawRow {
            index: idx + 1,
            label: format!("Molecule {}", idx + 1),
            smiles: smiles.to_string(),
        })
        .collect()
}

fn detect_column(headers: &[String], names: &[&str]) -> Option<usize> {
    let normalized: Vec<String> = headers
        .iter()
        .map(|h| strip_header_prefix(h).to_ascii_lowercase())
        .collect();
    if let Some(idx) = normalized
        .iter()
        .position(|h| names.iter().any(|needle| h == needle))
    {
        return Some(idx);
    }
    if let Some(idx) = normalized.iter().position(|h| {
        h.split(|c: char| !c.is_ascii_alphanumeric())
            .any(|token| names.contains(&token))
    }) {
        return Some(idx);
    }
    normalized
        .iter()
        .position(|h| names.iter().any(|needle| h.contains(needle)))
}

fn label_for_record(record: &[String], label_idx: Option<usize>, line_no: usize) -> String {
    label_idx
        .and_then(|idx| record.get(idx))
        .map(|s| s.trim())
        .filter(|value| !value.is_empty())
        .map_or_else(|| format!("Molecule {line_no}"), str::to_string)
}

/// Escapes a CSV field according to RFC 4180 and prefixes spreadsheet formula triggers.
#[cfg_attr(not(test), expect(dead_code, reason = "tested directly in unit tests"))]
#[must_use]
pub(crate) fn escape_csv(value: &str) -> String {
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
pub fn format_csv_rows(rows: &[MoleculeRow]) -> String {
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
