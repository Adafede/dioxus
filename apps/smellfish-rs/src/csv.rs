// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

use crate::model::RawRow;

/// Parse CSV text into rows, auto-detecting SMILES and label columns.
///
/// # Errors
///
/// Returns an error if the input is empty, the CSV is malformed,
/// or no data rows are found.
pub(crate) fn parse_csv_rows(text: &str) -> Result<Vec<RawRow>, String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err("Input is empty".to_string());
    }

    if looks_like_smiles_list(trimmed) {
        return parse_smiles_lines(trimmed);
    }

    let mut records = parse_csv_records(trimmed)?.into_iter();
    let Some(headers) = records.next() else {
        return Err("Input is empty".to_string());
    };
    let smiles_idx = detect_column(
        &headers,
        &["smiles", "smile", "structure", "canonical_smiles"],
    )
    .unwrap_or(0);
    let label_idx = detect_column(&headers, &["name", "label", "id", "compound"]);

    let mut rows = Vec::new();
    for (line, record) in records.enumerate() {
        let smiles = record
            .get(smiles_idx)
            .map_or("", String::as_str)
            .trim()
            .to_string();
        rows.push(RawRow {
            index: line + 1,
            label: label_for_record(&record, label_idx, line + 1),
            smiles,
        });
    }

    if rows.is_empty() {
        return Err("CSV does not contain any data rows".to_string());
    }
    Ok(rows)
}

fn parse_csv_records(text: &str) -> Result<Vec<Vec<String>>, String> {
    let mut records = Vec::new();
    let mut current_record = Vec::new();
    let mut current_field = String::new();
    let mut in_quotes = false;
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if in_quotes {
            if ch == '"' {
                if chars.peek() == Some(&'"') {
                    let _ = chars.next();
                    current_field.push('"');
                } else {
                    in_quotes = false;
                }
            } else {
                current_field.push(ch);
            }
        } else {
            match ch {
                '"' if current_field.trim().is_empty() => {
                    current_field.clear();
                    in_quotes = true;
                }
                ',' => {
                    current_record.push(std::mem::take(&mut current_field));
                }
                '\r' => {
                    if chars.peek() == Some(&'\n') {
                        let _ = chars.next();
                    }
                    finish_csv_row(&mut records, &mut current_record, &mut current_field);
                }
                '\n' => {
                    finish_csv_row(&mut records, &mut current_record, &mut current_field);
                }
                _ => current_field.push(ch),
            }
        }
    }

    if in_quotes {
        return Err("Unterminated quoted CSV field".to_string());
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
    let lines = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    !lines.is_empty()
        && lines.iter().all(|line| {
            !line.contains(',')
                && !line.contains('\t')
                && !line.contains(';')
                && normalize_column_header(line) != "smiles"
        })
}

fn parse_smiles_lines(text: &str) -> Result<Vec<RawRow>, String> {
    let rows = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .enumerate()
        .map(|(idx, smiles)| RawRow {
            index: idx + 1,
            label: format!("Molecule {}", idx + 1),
            smiles: smiles.to_string(),
        })
        .collect::<Vec<_>>();

    if rows.is_empty() {
        return Err("Input does not contain any SMILES".to_string());
    }

    Ok(rows)
}

fn detect_column(headers: &[String], names: &[&str]) -> Option<usize> {
    headers.iter().enumerate().find_map(|(idx, header)| {
        let normalized = normalize_column_header(header);
        names
            .iter()
            .any(|needle| normalized == *needle || normalized.contains(needle))
            .then_some(idx)
    })
}

fn normalize_column_header(header: &str) -> String {
    header
        .trim()
        .trim_start_matches('\u{feff}')
        .trim_start_matches(|c: char| ['>', '#', '"', '\'', ':', ';'].contains(&c))
        .trim()
        .to_ascii_lowercase()
}

fn label_for_record(record: &[String], label_idx: Option<usize>, line_no: usize) -> String {
    let label = label_idx
        .and_then(|idx| record.get(idx))
        .map(|s| s.trim())
        .filter(|value| !value.is_empty())
        .unwrap_or("");
    if label.is_empty() {
        format!("Molecule {line_no}")
    } else {
        label.to_string()
    }
}

#[cfg(test)]
#[expect(clippy::expect_used)]
// tests expect valid CSV fixtures to parse without error
// Every assertion below reads a row out of the literal fixture the same test
// just parsed, so an out-of-range read is itself the failing assertion.
#[allow(clippy::indexing_slicing)]
mod tests;
