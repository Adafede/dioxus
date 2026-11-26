// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smarts-evoliposuction project

//! Evolution result CSV reporting and tested-SMARTS export.

use std::fs;
use std::path::Path;

use smarts_evoliposuction::{
    CliError, EvolutionResult, ManifestRow, ParseCounts, TestedSmartsRecord,
};

/// Bundle of a manifest class row, evolution outcome, and elapsed wall seconds.
pub(crate) struct EvolutionOutcome<'a> {
    pub(crate) row: &'a ManifestRow,
    pub(crate) result: &'a EvolutionResult,
    pub(crate) elapsed: f64,
}

/// Formatted CSV field values for one evolution summary row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResultRowFields {
    pub(crate) positives_total: usize,
    pub(crate) negatives_total: usize,
    pub(crate) positives_filtered: usize,
    pub(crate) negatives_filtered: usize,
    pub(crate) status: &'static str,
    pub(crate) smarts: String,
    pub(crate) mcc: String,
    pub(crate) coverage_score: String,
    pub(crate) smarts_len: String,
    pub(crate) generations: String,
    pub(crate) tested_smarts_count: String,
    pub(crate) tested_smarts_path: String,
}

fn empty_row_fields(counts: ParseCounts, status: &'static str) -> ResultRowFields {
    ResultRowFields {
        positives_total: counts.positive_total,
        negatives_total: counts.negative_total,
        positives_filtered: counts.positive_failed,
        negatives_filtered: counts.negative_failed,
        status,
        smarts: String::new(),
        mcc: String::new(),
        coverage_score: String::new(),
        smarts_len: String::new(),
        generations: String::new(),
        tested_smarts_count: "0".into(),
        tested_smarts_path: String::new(),
    }
}

pub(crate) fn write_evolution_fields(
    writer: &mut csv::Writer<fs::File>,
    row: &ManifestRow,
    elapsed: f64,
    fields: &ResultRowFields,
) -> Result<(), CliError> {
    writer.write_record([
        &row.level,
        &row.label,
        &row.category,
        &row.main_class,
        &row.subclass,
        &row.slug,
        &fields.positives_total.to_string(),
        &fields.negatives_total.to_string(),
        &fields.positives_filtered.to_string(),
        &fields.negatives_filtered.to_string(),
        &fields.smarts,
        &fields.mcc,
        &fields.coverage_score,
        &fields.smarts_len,
        &fields.generations,
        &format!("{elapsed:.3}"),
        fields.status,
        &fields.tested_smarts_count,
        &fields.tested_smarts_path,
    ])?;
    writer.flush()?;
    Ok(())
}

pub(crate) fn write_evolution_row(
    writer: &mut csv::Writer<fs::File>,
    outcome: &EvolutionOutcome<'_>,
    tested_smarts_dir: &Path,
) -> Result<(), CliError> {
    let fields = process_evolution_result(outcome.result, outcome.row, tested_smarts_dir)?;
    write_evolution_fields(writer, outcome.row, outcome.elapsed, &fields)
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
                mcc: task_result.best_mcc().to_string(),
                coverage_score: task_result.best_coverage_score().to_string(),
                smarts_len: task_result.best_smarts_len().to_string(),
                generations: task_result.generations().to_string(),
                tested_smarts_count: tested_smarts.len().to_string(),
                tested_smarts_path: path.to_string_lossy().to_string(),
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

fn write_tested_smarts_csv(
    output_path: &Path,
    tested_smarts: &[TestedSmartsRecord],
) -> Result<(), CliError> {
    // Sort by MCC (desc), then coverage_score (desc), then generation (asc).
    // This groups the most selective patterns first, and within equal metrics
    // shows earlier generations first.
    let mut sorted: Vec<&TestedSmartsRecord> = tested_smarts.iter().collect();
    sorted.sort_by(|a, b| a.cmp_priority(b));

    let mut writer = csv::Writer::from_path(output_path)?;
    writer.write_record([
        "smarts",
        "mcc",
        "smarts_len",
        "coverage_score",
        "limit_exceeded",
        "generation",
    ])?;
    for tested in &sorted {
        writer.write_record([
            tested.smarts.as_str(),
            &tested.mcc.to_string(),
            &tested.smarts_len.to_string(),
            &tested.coverage_score.to_string(),
            &tested.limit_exceeded.to_string(),
            &tested.generation.to_string(),
        ])?;
    }
    writer.flush()?;
    Ok(())
}
