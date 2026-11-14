// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Evolution runner wrapping `smarts-evolution` with error isolation.
//!
//! Each class's positive/negative SMILES files are read, parsed, and evolved
//! independently. Parse failures and evolution errors are caught per-class
//! and reported in the output CSV rather than aborting the batch.
//!
//! **Tested-SMARTS capture:** Every evolution path (progress and batch) uses an
//! internal observer that intercepts every genome evaluation via the
//! `EvolutionProgressObserver::on_evaluation` callback. This captures **all**
//! evaluated SMARTS — including duplicates and ties. The observer also
//! records the generation number for each evaluated SMARTS.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use smarts_evolution::{
    EvolutionConfig, EvolutionEvaluationProgress, EvolutionProgress, EvolutionProgressObserver,
    EvolutionTask, FoldData, FoldSample, SeedCorpus, TaskResult,
};
use smarts_rs::PreparedTarget;
use smiles_rs::Smiles;

/// Configuration passed through to `smarts-evolution`'s `EvolutionConfig`.
///
/// The `match_time_limit` controls the per-SMARTS evaluation timeout.
/// The default of 30 seconds (inherited from `smarts-evolution`) is too
/// generous for lipid classification — complex SMARTS patterns can waste 30s
/// each. A 5-second limit is sufficient for accurate MCC measurement while
/// cutting worst-case evaluation time by 6×.
#[derive(Debug, Clone)]
pub struct Config {
    pub population_size: usize,
    pub generation_limit: u64,
    pub stagnation_limit: u64,
    pub seed: Option<u64>,
    /// Per-SMARTS match timeout. `None` = use smarts-evolution's default (30s).
    pub match_time_limit: Option<Duration>,
}

/// One tested SMARTS candidate observed during evolution.
///
/// Includes the `generation` in which the candidate was evaluated, so results
/// can be sorted by `MCC`, `coverage_score`, and `generation`.
#[derive(Debug, Clone)]
pub struct TestedSmartsRecord {
    pub smarts: String,
    pub mcc: f64,
    pub smarts_len: usize,
    pub coverage_score: f64,
    pub limit_exceeded: bool,
    pub generation: u64,
}

impl TestedSmartsRecord {
    /// Total ordering by `mcc` descending, `coverage_score` descending, then `generation` ascending.
    #[must_use]
    pub fn cmp_priority(&self, other: &Self) -> std::cmp::Ordering {
        other
            .mcc
            .total_cmp(&self.mcc)
            .then_with(|| other.coverage_score.total_cmp(&self.coverage_score))
            .then_with(|| self.generation.cmp(&other.generation))
    }
}

/// Result of evolving one class split.
#[derive(Debug)]
pub enum EvolutionResult {
    /// Evolution succeeded. The `TaskResult`, tested SMARTS list, plus parse stats.
    Ok(
        TaskResult,
        Vec<TestedSmartsRecord>,
        usize,
        usize,
        usize,
        usize,
    ),
    /// All SMILES parsed but at least one side is empty after filtering.
    EmptyAfterParse(usize, usize, usize, usize),
    /// File read error.
    ReadError(usize, usize, usize, usize),
    /// `task.evolve()` returned an error.
    EvolutionError(String, usize, usize, usize, usize),
}

impl EvolutionResult {
    /// Returns the `TaskResult` if evolution succeeded, or `None` otherwise.
    #[must_use]
    pub const fn as_task_result(&self) -> Option<&TaskResult> {
        match self {
            Self::Ok(result, ..) => Some(result),
            _ => None,
        }
    }
}

/// Return type from `setup_evolution`: the evolution task, config, seed corpus,
/// and parse counts (`total_positive`, `total_negative`, `positive_failed`,
/// `negative_failed`).
type SetupResult = (
    EvolutionTask,
    EvolutionConfig,
    SeedCorpus,
    usize,
    usize,
    usize,
    usize,
);

/// Observer that captures **every** evaluated SMARTS during evolution.
///
/// Unlike the ratatui TUI dashboard (which only returned the final leaderboard),
/// this observer intercepts each `on_evaluation` callback and records the
/// evaluated genome with its generation number. No deduplication is performed —
/// duplicate SMARTS strings and ties are preserved.
///
/// When `show_progress` is enabled, a concise text progress line is printed to
/// stderr after each generation, providing TUI-like feedback without taking
/// over the terminal.
#[derive(Debug)]
struct TestedSmartsObserver {
    tested: Arc<Mutex<Vec<TestedSmartsRecord>>>,
    show_progress: bool,
}

impl TestedSmartsObserver {
    const fn new(tested: Arc<Mutex<Vec<TestedSmartsRecord>>>, show_progress: bool) -> Self {
        Self {
            tested,
            show_progress,
        }
    }

    fn push_last(&self, progress: &EvolutionEvaluationProgress) {
        let Some(last) = progress.last() else {
            return;
        };
        let generation = progress.generation();
        if let Ok(mut tested) = self.tested.lock() {
            tested.push(TestedSmartsRecord {
                smarts: last.smarts().to_string(),
                mcc: last.mcc(),
                smarts_len: last.smarts_len(),
                coverage_score: last.coverage_score(),
                limit_exceeded: last.limit_exceeded(),
                generation,
            });
        }
    }
}

impl EvolutionProgressObserver for TestedSmartsObserver {
    fn on_evaluation(&mut self, progress: &EvolutionEvaluationProgress) {
        self.push_last(progress);
    }

    fn on_generation(&mut self, progress: &EvolutionProgress) {
        if !self.show_progress {
            return;
        }
        let generation = progress.generation();
        let total = progress.generation_limit();
        let best = progress.best_so_far();
        let best_mcc = best.mcc();
        let best_smarts = best.smarts();
        let display = truncate_str(best_smarts, 60);
        eprintln!("  gen {generation:>4}/{total:<4}  best_mcc={best_mcc:.4}  best= {display}");
    }
}

fn truncate_str(s: &str, max_bytes: usize) -> &str {
    &s[..s.floor_char_boundary(max_bytes)]
}

/// Shared setup: read SMILES files, parse, build evolution config and task.
///
/// Returns `Err` early (with the appropriate `EvolutionResult` error variant)
/// if the files can't be read or the parsed sets are empty.
#[allow(clippy::result_large_err)]
fn setup_evolution(
    config: &Config,
    positive_path: &Path,
    negative_path: &Path,
) -> Result<SetupResult, EvolutionResult> {
    let (positive_lines, read_err) = read_smiles_lines(positive_path);
    if read_err {
        return Err(EvolutionResult::ReadError(0, 0, 0, 0));
    }
    let (negative_lines, read_err) = read_smiles_lines(negative_path);
    if read_err {
        return Err(EvolutionResult::ReadError(positive_lines.len(), 0, 0, 0));
    }

    let (positive, positive_failed) = prepare_all(&positive_lines);
    let (negative, negative_failed) = prepare_all(&negative_lines);

    let total_positive = positive_lines.len();
    let total_negative = negative_lines.len();

    if positive.is_empty() || negative.is_empty() {
        return Err(EvolutionResult::EmptyAfterParse(
            total_positive,
            total_negative,
            positive_failed,
            negative_failed,
        ));
    }

    let mut samples = Vec::with_capacity(positive.len() + negative.len());
    for target in positive {
        samples.push(FoldSample::positive(target));
    }
    for target in negative {
        samples.push(FoldSample::negative(target));
    }

    let mut config_builder = EvolutionConfig::builder()
        .population_size(config.population_size)
        .generation_limit(config.generation_limit)
        .stagnation_limit(config.stagnation_limit);
    if let Some(seed) = config.seed {
        config_builder = config_builder.rng_seed(seed);
    }
    if let Some(timeout) = config.match_time_limit {
        config_builder = config_builder.match_time_limit(timeout);
    }
    let evolution_config = match config_builder.build() {
        Ok(c) => c,
        Err(e) => {
            return Err(EvolutionResult::EvolutionError(
                format!("invalid config: {e}"),
                total_positive,
                total_negative,
                positive_failed,
                negative_failed,
            ));
        }
    };

    let seed_corpus = SeedCorpus::builtin();
    let task_id = format!(
        "{}__{}",
        positive_path
            .file_stem()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("unknown"),
        negative_path
            .file_stem()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("unknown"),
    );

    let task = EvolutionTask::new(task_id, vec![FoldData::new(samples)]);
    Ok((
        task,
        evolution_config,
        seed_corpus,
        total_positive,
        total_negative,
        positive_failed,
        negative_failed,
    ))
}

/// Run evolution for one positive/negative pair of `.smiles` files.
///
/// Uses an internal observer to capture **all** evaluated SMARTS
/// (including duplicates and ties) with their generation numbers.
/// No progress output is printed — use [`evolve_all_with_progress`] for
/// text-based progress feedback to stderr.
#[allow(clippy::module_name_repetitions, clippy::too_many_lines)]
#[must_use]
pub fn evolve_all(config: &Config, positive_path: &Path, negative_path: &Path) -> EvolutionResult {
    let parsed = match setup_evolution(config, positive_path, negative_path) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let (
        task,
        evolution_config,
        seed_corpus,
        total_positive,
        total_negative,
        positive_failed,
        negative_failed,
    ) = parsed;

    let tested = Arc::new(Mutex::new(Vec::new()));
    let observer = TestedSmartsObserver::new(Arc::clone(&tested), false);
    match task.evolve_with_observer(&evolution_config, &seed_corpus, 1, observer) {
        Ok(result) => EvolutionResult::Ok(
            result,
            tested
                .lock()
                .map_or_else(|_| Vec::new(), |records| records.clone()),
            total_positive,
            total_negative,
            positive_failed,
            negative_failed,
        ),
        Err(e) => EvolutionResult::EvolutionError(
            e.to_string(),
            total_positive,
            total_negative,
            positive_failed,
            negative_failed,
        ),
    }
}

/// Run evolution for one positive/negative pair of `.smiles` files, with
/// text-based progress output to stderr.
///
/// This function always captures **all** evaluated SMARTS via an internal
/// observer. The progress output provides feedback
/// (generation, best MCC) without taking over the terminal, so it is safe
/// for use in both interactive and batch contexts.
///
/// Unlike the original ratatui TUI dashboard, this approach captures every
/// genome evaluated — not just the final leaderboard — so the per-class CSV
/// contains the complete set of tested SMARTS.
#[allow(clippy::module_name_repetitions, clippy::too_many_lines)]
#[must_use]
pub fn evolve_all_with_progress(
    config: &Config,
    positive_path: &Path,
    negative_path: &Path,
) -> EvolutionResult {
    let parsed = match setup_evolution(config, positive_path, negative_path) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let (
        task,
        evolution_config,
        seed_corpus,
        total_positive,
        total_negative,
        positive_failed,
        negative_failed,
    ) = parsed;

    let tested = Arc::new(Mutex::new(Vec::new()));
    let observer = TestedSmartsObserver::new(Arc::clone(&tested), true);
    match task.evolve_with_observer(&evolution_config, &seed_corpus, 1, observer) {
        Ok(result) => EvolutionResult::Ok(
            result,
            tested
                .lock()
                .map_or_else(|_| Vec::new(), |records| records.clone()),
            total_positive,
            total_negative,
            positive_failed,
            negative_failed,
        ),
        Err(e) => EvolutionResult::EvolutionError(
            e.to_string(),
            total_positive,
            total_negative,
            positive_failed,
            negative_failed,
        ),
    }
}

fn read_smiles_lines(path: &Path) -> (Vec<String>, bool) {
    let Ok(file) = File::open(path) else {
        return (Vec::new(), true);
    };
    let reader = BufReader::new(file);
    let mut out = Vec::new();
    for trimmed in reader.lines().map_while(Result::ok) {
        let trimmed = trimmed.trim();
        if !trimmed.is_empty() {
            out.push(trimmed.to_string());
        }
    }
    (out, false)
}

fn prepare_all(lines: &[String]) -> (Vec<PreparedTarget>, usize) {
    let mut prepared = Vec::with_capacity(lines.len());
    let mut failed = 0usize;
    for smi in lines {
        match Smiles::from_str(smi) {
            Ok(parsed) => prepared.push(PreparedTarget::new(parsed)),
            Err(_) => failed += 1,
        }
    }
    (prepared, failed)
}

#[cfg(test)]
#[expect(clippy::unwrap_used)] // tests unwrap fixtures to fail-fast rather than assert on the error
#[expect(clippy::assert_is_empty)] // the empty case is the thing under test here
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn read_smiles_lines_skips_empty() {
        let mut tmp = NamedTempFile::new().unwrap();
        writeln!(tmp, "CCO").unwrap();
        writeln!(tmp).unwrap();
        writeln!(tmp, "C(C)C").unwrap();
        tmp.flush().unwrap();

        let (lines, err) = read_smiles_lines(tmp.path());
        assert!(!err);
        assert_eq!(lines, vec!["CCO".to_string(), "C(C)C".to_string()]);
    }

    #[test]
    fn read_smiles_lines_missing_file() {
        let (lines, err) = read_smiles_lines(std::path::Path::new("/nonexistent/file.smiles"));
        assert!(err);
        assert!(lines.is_empty());
    }

    #[test]
    fn prepare_all_counts_failures() {
        let lines = vec![
            "CCO".to_string(),
            "NOT_A_SMILES".to_string(),
            "C(C)C".to_string(),
        ];
        let (prepared, failed) = prepare_all(&lines);
        assert_eq!(prepared.len(), 2);
        assert_eq!(failed, 1);
    }

    #[test]
    fn truncate_str_respects_utf8_boundaries() {
        assert_eq!(truncate_str("αβγ", 0), "");
        assert_eq!(truncate_str("αβγ", 1), "");
        assert_eq!(truncate_str("αβγ", 2), "α");
        assert_eq!(truncate_str("αβγ", 3), "α");
        assert_eq!(truncate_str("αβγ", 4), "αβ");
        assert_eq!(truncate_str("αβγ", 100), "αβγ");
    }

    #[test]
    fn error_variants_yield_no_task_result() {
        let empty = EvolutionResult::EmptyAfterParse(3, 2, 1, 0);
        let read_err = EvolutionResult::ReadError(0, 0, 0, 0);
        let evo_err = EvolutionResult::EvolutionError("boom".to_owned(), 1, 1, 0, 0);
        assert!(
            empty.as_task_result().is_none(),
            "EmptyAfterParse has no TaskResult"
        );
        assert!(
            read_err.as_task_result().is_none(),
            "ReadError has no TaskResult"
        );
        assert!(
            evo_err.as_task_result().is_none(),
            "EvolutionError has no TaskResult"
        );
    }

    #[test]
    fn cmp_priority_sorts_records_with_nan_deterministically() {
        let make = |smarts: &str, mcc: f64, cov: f64, generation: u64| TestedSmartsRecord {
            smarts: smarts.to_owned(),
            mcc,
            smarts_len: smarts.len(),
            coverage_score: cov,
            limit_exceeded: false,
            generation,
        };

        let mut records = vec![
            make("g_late", 0.7, 0.5, 5),
            make("nan_cov", 0.7, f64::NAN, 1),
            make("nan_mcc", f64::NAN, 0.9, 0),
            make("best", 0.9, 0.2, 9),
            make("g_early", 0.7, 0.5, 1),
            make("cov_high", 0.7, 0.8, 4),
        ];
        for i in 0..18_u32 {
            let mcc = if i % 5 == 0 {
                f64::NAN
            } else {
                f64::from(i).mul_add(0.01, 0.1)
            };
            records.push(make(&format!("filler_{i}"), mcc, 0.1, u64::from(i)));
        }

        records.sort_by(TestedSmartsRecord::cmp_priority);

        let top_six: Vec<&str> = records.iter().take(6).map(|r| r.smarts.as_str()).collect();
        // f64::total_cmp places +NaN above finite numbers; descending sort puts NaN first without panicking.
        assert_eq!(
            top_six,
            vec![
                "nan_mcc",
                "filler_0",
                "filler_5",
                "filler_10",
                "filler_15",
                "best"
            ]
        );

        let finite: Vec<&str> = records
            .iter()
            .filter(|r| !r.mcc.is_nan() && !r.coverage_score.is_nan())
            .take(4)
            .map(|r| r.smarts.as_str())
            .collect();
        assert_eq!(finite, vec!["best", "cov_high", "g_early", "g_late"]);
    }
}
