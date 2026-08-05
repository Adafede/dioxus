use crate::model::RawRow;

#[allow(dead_code)]
pub fn parse_csv_rows(text: &str) -> Result<Vec<RawRow>, String> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(text.as_bytes());

    let headers = rdr.headers().map_err(|e| e.to_string())?.clone();
    let smiles_idx = detect_column(
        &headers,
        &["smiles", "smile", "structure", "canonical_smiles"],
    )
    .unwrap_or(0);
    let label_idx = detect_column(&headers, &["name", "label", "id", "compound"]);

    let mut rows = Vec::new();
    for (line, record) in rdr.records().enumerate() {
        let record = record.map_err(|e| e.to_string())?;
        let smiles = record.get(smiles_idx).unwrap_or("").trim().to_string();
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

#[allow(dead_code)]
fn detect_column(headers: &csv::StringRecord, names: &[&str]) -> Option<usize> {
    headers.iter().enumerate().find_map(|(idx, header)| {
        let normalized = normalize_column_header(header);
        names
            .iter()
            .any(|needle| normalized == *needle || normalized.contains(needle))
            .then_some(idx)
    })
}

#[allow(dead_code)]
fn normalize_column_header(header: &str) -> String {
    header
        .trim()
        .trim_start_matches('\u{feff}')
        .trim_start_matches(|c: char| ['>', '#', '"', '\'', ':', ';'].contains(&c))
        .trim()
        .to_ascii_lowercase()
}

#[allow(dead_code)]
fn label_for_record(
    record: &csv::StringRecord,
    label_idx: Option<usize>,
    line_no: usize,
) -> String {
    let label = label_idx
        .and_then(|idx| record.get(idx))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("");
    if label.is_empty() {
        format!("Molecule {line_no}")
    } else {
        label.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_smiles_and_label_columns() {
        let rows = parse_csv_rows("name,smiles\nalpha,C1CCCCC1\n").expect("rows");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].label, "alpha");
        assert_eq!(rows[0].smiles, "C1CCCCC1");
    }

    #[test]
    fn falls_back_to_generated_labels() {
        let rows = parse_csv_rows("smiles\nCCO\n").expect("rows");
        assert_eq!(rows[0].label, "Molecule 1");
    }
}
