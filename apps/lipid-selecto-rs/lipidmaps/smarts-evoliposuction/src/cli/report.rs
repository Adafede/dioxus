// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smarts-evoliposuction project

//! Evolution result CSV reporting and tested-SMARTS export.

use std::fmt;
use std::fs;
use std::path::Path;

use serde::{Serialize, Serializer};
use smarts_evoliposuction::{
    CliError, EvolutionResult, ManifestRow, ParseCounts, TestedSmartsRecord,
};

struct OptDisplay<T>(Option<T>);

impl<T: fmt::Display> Serialize for OptDisplay<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match &self.0 {
            Some(val) => serializer.collect_str(val),
            None => serializer.serialize_none(),
        }
    }
}

struct DisplayFloat3(f64);

impl Serialize for DisplayFloat3 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&format_args!("{:.3}", self.0))
    }
}

/// CSV field values for one evolution summary row.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ResultRowFields {
    pub(crate) positives_total: usize,
    pub(crate) negatives_total: usize,
    pub(crate) positives_filtered: usize,
    pub(crate) negatives_filtered: usize,
    pub(crate) status: &'static str,
    pub(crate) smarts: String,
    pub(crate) mcc: Option<f64>,
    pub(crate) coverage_score: Option<f64>,
    pub(crate) smarts_len: Option<usize>,
    pub(crate) generations: Option<u64>,
    pub(crate) tested_smarts_count: usize,
    pub(crate) tested_smarts_path: String,
}

#[derive(Serialize)]
struct EvolutionCsvRow<'a> {
    level: &'a str,
    label: &'a str,
    category: &'a str,
    main_class: &'a str,
    subclass: &'a str,
    slug: &'a str,
    positive_count: usize,
    negative_count: usize,
    positive_parse_failures: usize,
    negative_parse_failures: usize,
    best_smarts: &'a str,
    best_mcc: OptDisplay<f64>,
    best_coverage_score: OptDisplay<f64>,
    best_smarts_len: OptDisplay<usize>,
    generations: OptDisplay<u64>,
    elapsed_secs: DisplayFloat3,
    status: &'a str,
    tested_smarts_count: usize,
    tested_smarts_path: &'a str,
}

const fn empty_row_fields(counts: ParseCounts, status: &'static str) -> ResultRowFields {
    ResultRowFields {
        positives_total: counts.positive_total,
        negatives_total: counts.negative_total,
        positives_filtered: counts.positive_failed,
        negatives_filtered: counts.negative_failed,
        status,
        smarts: String::new(),
        mcc: None,
        coverage_score: None,
        smarts_len: None,
        generations: None,
        tested_smarts_count: 0,
        tested_smarts_path: String::new(),
    }
}

pub(crate) fn write_evolution_fields(
    writer: &mut csv::Writer<fs::File>,
    row: &ManifestRow,
    elapsed: f64,
    fields: &ResultRowFields,
) -> Result<(), CliError> {
    let csv_row = EvolutionCsvRow {
        level: &row.level,
        label: &row.label,
        category: &row.category,
        main_class: &row.main_class,
        subclass: &row.subclass,
        slug: &row.slug,
        positive_count: fields.positives_total,
        negative_count: fields.negatives_total,
        positive_parse_failures: fields.positives_filtered,
        negative_parse_failures: fields.negatives_filtered,
        best_smarts: &fields.smarts,
        best_mcc: OptDisplay(fields.mcc),
        best_coverage_score: OptDisplay(fields.coverage_score),
        best_smarts_len: OptDisplay(fields.smarts_len),
        generations: OptDisplay(fields.generations),
        elapsed_secs: DisplayFloat3(elapsed),
        status: fields.status,
        tested_smarts_count: fields.tested_smarts_count,
        tested_smarts_path: &fields.tested_smarts_path,
    };
    writer.serialize(&csv_row)?;
    writer.flush()?;
    Ok(())
}

/// Extract CSV fields from an [`EvolutionResult`], writing the tested-SMARTS CSV
/// when evolution succeeded.
pub(crate) fn process_evolution_result(
    result: &EvolutionResult,
    row: &ManifestRow,
    tested_smarts_dir: &Path,
) -> Result<ResultRowFields, CliError> {
    match result {
        EvolutionResult::Ok(task_result, tested_smarts, counts) => {
            eprintln!("mcc={:.4}", task_result.best_mcc());
            let path = tested_smarts_dir.join(format!("{}_tested_smarts.csv", row.slug));
            write_tested_smarts_csv(&path, tested_smarts)?;
            Ok(ResultRowFields {
                positives_total: counts.positive_total,
                negatives_total: counts.negative_total,
                positives_filtered: counts.positive_failed,
                negatives_filtered: counts.negative_failed,
                status: "ok",
                smarts: task_result.best_smarts().to_string(),
                mcc: Some(task_result.best_mcc()),
                coverage_score: Some(task_result.best_coverage_score()),
                smarts_len: Some(task_result.best_smarts_len()),
                generations: Some(task_result.generations()),
                tested_smarts_count: tested_smarts.len(),
                tested_smarts_path: path.to_string_lossy().into_owned(),
            })
        }
        EvolutionResult::EmptyAfterParse(counts) => {
            eprintln!("SKIPPED {}", row.slug);
            Ok(empty_row_fields(*counts, "empty_after_parse"))
        }
        EvolutionResult::ReadError(counts) => {
            eprintln!("FAILED {}", row.slug);
            Ok(empty_row_fields(*counts, "read_error"))
        }
        EvolutionResult::EvolutionError(msg, counts) => {
            eprintln!("ERROR: {msg}");
            Ok(empty_row_fields(*counts, "evolution_error"))
        }
    }
}

#[derive(Serialize)]
struct TestedSmartsCsvRow<'a> {
    smarts: &'a str,
    mcc: f64,
    smarts_len: usize,
    coverage_score: f64,
    limit_exceeded: bool,
    generation: u64,
}

fn write_tested_smarts_csv(
    output_path: &Path,
    tested_smarts: &[TestedSmartsRecord],
) -> Result<(), CliError> {
    // Sort by MCC (desc), then coverage_score (desc), then generation (asc).
    // This groups the most selective patterns first, and within equal metrics
    // shows earlier generations first.
    let mut sorted: Vec<&TestedSmartsRecord> = tested_smarts.iter().collect();
    sorted.sort_by(|a, b| a.cmp_priority(b));

    let mut writer = csv::WriterBuilder::new()
        .has_headers(false)
        .from_path(output_path)?;
    writer.write_record([
        "smarts",
        "mcc",
        "smarts_len",
        "coverage_score",
        "limit_exceeded",
        "generation",
    ])?;
    for tested in &sorted {
        writer.serialize(TestedSmartsCsvRow {
            smarts: &tested.smarts,
            mcc: tested.mcc,
            smarts_len: tested.smarts_len,
            coverage_score: tested.coverage_score,
            limit_exceeded: tested.limit_exceeded,
            generation: tested.generation,
        })?;
    }
    writer.flush()?;
    Ok(())
}
