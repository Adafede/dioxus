// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smarts-evoliposuction project

//! Manifest row ordering by hierarchy level and average SMILES length.

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

use smarts_evoliposuction::ManifestRow;

pub(crate) fn order_rows_for_evolution(rows: Vec<ManifestRow>) -> Vec<ManifestRow> {
    // Precompute average SMILES length once per row to avoid O(N log N) disk reads.
    let mut keyed: Vec<(u8, f64, ManifestRow)> = rows
        .into_iter()
        .map(|row| (level_rank(&row.level), average_smiles_row_len(&row), row))
        .collect();

    keyed.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then_with(|| a.1.total_cmp(&b.1))
            .then_with(|| a.2.slug.cmp(&b.2.slug))
    });

    keyed.into_iter().map(|(_, _, row)| row).collect()
}

fn level_rank(level: &str) -> u8 {
    match level {
        "category" => 0,
        "subclass" => 2,
        _ => 1,
    }
}

fn average_smiles_row_len(row: &ManifestRow) -> f64 {
    let (sum_pos, cnt_pos) = average_smiles_len(Path::new(&row.positive_path));
    let (sum_neg, cnt_neg) = average_smiles_len(Path::new(&row.negative_path));
    let total_count = cnt_pos + cnt_neg;
    if total_count == 0 {
        return f64::INFINITY;
    }
    let sum = u32::try_from(sum_pos + sum_neg).unwrap_or(u32::MAX);
    let count = u32::try_from(total_count).unwrap_or(u32::MAX);
    f64::from(sum) / f64::from(count)
}

fn average_smiles_len(path: &Path) -> (usize, usize) {
    let Ok(file) = fs::File::open(path) else {
        return (0, 0);
    };
    let reader = BufReader::new(file);
    let mut total_len = 0usize;
    let mut count = 0usize;
    for line in reader.lines().map_while(Result::ok) {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            total_len += trimmed.len();
            count += 1;
        }
    }
    (total_len, count)
}
