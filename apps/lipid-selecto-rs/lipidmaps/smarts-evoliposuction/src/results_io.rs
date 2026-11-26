// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Helpers for opening, resuming, and deduplicating `smarts_results.csv`.

use std::collections::{HashMap, HashSet};
use std::fs::{self, OpenOptions};
use std::path::Path;

/// Column headers for the evolved SMARTS results CSV.
pub const RESULTS_HEADERS: [&str; 19] = [
    "level",
    "label",
    "category",
    "main_class",
    "subclass",
    "slug",
    "positive_count",
    "negative_count",
    "positive_parse_failures",
    "negative_parse_failures",
    "best_smarts",
    "best_mcc",
    "best_coverage_score",
    "best_smarts_len",
    "generations",
    "elapsed_secs",
    "status",
    "tested_smarts_count",
    "tested_smarts_path",
];

/// Open the results CSV writer, truncating unless `resume` is true.
///
/// Returns the writer and whether the header row still needs to be written.
///
/// # Errors
///
/// Returns an I/O error if opening or truncating `output` fails.
pub fn open_results_writer(
    output: &Path,
    resume: bool,
) -> std::io::Result<(csv::Writer<fs::File>, bool)> {
    let has_content = resume && output.metadata().is_ok_and(|meta| meta.len() > 0);
    let mut options = OpenOptions::new();
    options.create(true);
    if resume {
        options.append(true);
    } else {
        options.write(true).truncate(true);
    }
    let file = options.open(output)?;
    let writer = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(file);
    Ok((writer, !has_content))
}

/// Collect slugs that already completed with `status == "ok"` in an existing results CSV.
///
/// # Errors
///
/// Returns an error if `output` exists and cannot be read or contains malformed CSV rows.
pub fn already_done(output: &Path) -> Result<HashSet<String>, crate::CliError> {
    if !output.exists() {
        return Ok(HashSet::new());
    }
    let mut done = HashSet::new();
    let mut reader = csv::Reader::from_path(output)?;
    for result in reader.deserialize::<HashMap<String, String>>() {
        let record = result?;
        if record.get("status").is_some_and(|s| s == "ok")
            && let Some(slug) = record.get("slug")
        {
            done.insert(slug.clone());
        }
    }
    Ok(done)
}
