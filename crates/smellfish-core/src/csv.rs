// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! CSV input parsing and sanitized CSV export.

use crate::error::SmellfishError;
use crate::model::{MoleculeRow, RawRow};
use std::fmt::Write;

const SMILES_HEADERS: &[&str] = &["smiles", "smile", "structure", "canonical_smiles"];
const LABEL_HEADERS: &[&str] = &["name", "label", "id", "compound"];

/// Parse delimited text (`CSV`, `TSV`, `;`-separated) or newline-delimited SMILES into [`RawRow`] values.
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

    let delimiter = detect_delimiter(trimmed);
    let records = parse_delimited_records(trimmed, delimiter)?;
    if records.is_empty() {
        return Err(SmellfishError::EmptyInput);
    }
    let has_header = records
        .first()
        .is_some_and(|first| delimiter == ',' || row_looks_like_header(first));

    let (smiles_idx, label_idx, skip) = if has_header {
        let headers = records.first().ok_or(SmellfishError::EmptyInput)?;
        let smiles_idx = detect_column(headers, SMILES_HEADERS).unwrap_or(0);
        let label_idx = detect_column(headers, LABEL_HEADERS);
        (smiles_idx, label_idx, 1usize)
    } else {
        // Headerless TSV / semicolon SMILES+label rows: column 0 is SMILES, column 1 is label.
        let label_idx = records.first().and_then(|r| (r.len() > 1).then_some(1));
        (0, label_idx, 0usize)
    };

    let rows: Vec<RawRow> = records
        .iter()
        .skip(skip)
        .filter_map(|record| {
            let smiles = record
                .get(smiles_idx)
                .map_or("", String::as_str)
                .trim()
                .to_string();
            (!smiles.is_empty()).then_some((smiles, record))
        })
        .enumerate()
        .map(|(line, (smiles, record))| RawRow {
            index: line + 1,
            label: label_for_record(record, label_idx, line + 1),
            smiles,
        })
        .collect();

    if rows.is_empty() {
        return Err(SmellfishError::NoDataRows);
    }
    Ok(rows)
}

fn parse_delimited_records(
    text: &str,
    delimiter: char,
) -> Result<Vec<Vec<String>>, SmellfishError> {
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
            c if c == delimiter => current_record.push(std::mem::take(&mut current_field)),
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

fn detect_delimiter(text: &str) -> char {
    let first_line = text
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("");
    let (mut commas, mut tabs, mut semis) = (0usize, 0usize, 0usize);
    let mut in_quotes = false;
    for ch in first_line.chars() {
        match ch {
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => commas += 1,
            '\t' if !in_quotes => tabs += 1,
            ';' if !in_quotes => semis += 1,
            _ => {}
        }
    }
    if commas > 0 {
        ','
    } else if tabs > 0 {
        '\t'
    } else if semis > 0 {
        ';'
    } else {
        ','
    }
}

fn row_looks_like_header(record: &[String]) -> bool {
    record.iter().any(|cell| {
        let norm = strip_header_prefix(cell).to_ascii_lowercase();
        SMILES_HEADERS.iter().any(|k| norm == *k) || LABEL_HEADERS.iter().any(|k| norm == *k)
    })
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
#[cfg(test)]
#[must_use]
pub(crate) fn escape_csv(value: &str) -> String {
    let mut out = String::new();
    push_csv_escaped(&mut out, value);
    out
}

fn push_csv_escaped(out: &mut String, s: &str) {
    let formula_prefix = matches!(
        s.as_bytes().first(),
        Some(b'=' | b'+' | b'-' | b'@' | b'\t' | b'\r')
    );
    let needs_quotes = s.bytes().any(|b| matches!(b, b',' | b'"' | b'\n' | b'\r'));
    if needs_quotes {
        out.reserve(s.len() + 2 + usize::from(formula_prefix));
        out.push('"');
        if formula_prefix {
            out.push('\'');
        }
        for ch in s.chars() {
            if ch == '"' {
                out.push_str("\"\"");
            } else {
                out.push(ch);
            }
        }
        out.push('"');
    } else {
        if formula_prefix {
            out.push('\'');
        }
        out.push_str(s);
    }
}

fn push_csv_field(out: &mut String, value: &str) {
    push_csv_escaped(out, value);
    out.push(',');
}

/// Formats assessed molecule rows into a downloadable CSV report.
#[must_use]
pub fn format_csv_rows(rows: &[MoleculeRow]) -> String {
    let mut csv = String::with_capacity(128 + rows.len() * 128);
    csv.push_str(
        "label,smiles,inchikey,np_score,np_label,np_confidence,ring_family,substituents,lotus_qids,lotus_taxa,pubchem_cids,verdict_category,chemist_checks\n",
    );
    let mut scratch = String::new();
    for r in rows {
        push_csv_field(&mut csv, &r.label);
        push_csv_field(&mut csv, &r.smiles);
        push_csv_field(&mut csv, &r.inchikey);
        if let Some(np) = r.np_score.filter(|np| np.score.is_finite()) {
            let _ = write!(csv, "{:.3},", np.score);
            push_csv_escaped(&mut csv, &r.np_label);
            if np.confidence.is_finite() {
                let conf_pct = (np.confidence * 100.0).round();
                let _ = write!(csv, ",{conf_pct}%,");
            } else {
                csv.push_str(",,");
            }
        } else {
            csv.push(',');
            push_csv_escaped(&mut csv, &r.np_label);
            csv.push_str(",,");
        }
        push_csv_field(&mut csv, r.ring_family.as_str());

        scratch.clear();
        for (idx, (label, count)) in r.substituents_counts.iter().enumerate() {
            if idx > 0 {
                scratch.push(';');
            }
            let _ = write!(scratch, "{label}({count})");
        }
        push_csv_escaped(&mut csv, &scratch);
        csv.push(',');

        push_joined_csv_list(
            &mut csv,
            &mut scratch,
            r.lotus_compounds.iter().map(|c| c.qid.as_str()),
        );
        csv.push(',');
        push_joined_csv_list(
            &mut csv,
            &mut scratch,
            r.lotus_taxa.iter().map(String::as_str),
        );
        csv.push(',');
        push_joined_csv_list(
            &mut csv,
            &mut scratch,
            r.pubchem_cids.iter().map(String::as_str),
        );
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

fn push_joined_csv_list<'a>(
    out: &mut String,
    scratch: &mut String,
    items: impl Iterator<Item = &'a str>,
) {
    scratch.clear();
    for (idx, item) in items.enumerate() {
        if idx > 0 {
            scratch.push(';');
        }
        scratch.push_str(item);
    }
    push_csv_escaped(out, scratch);
}
