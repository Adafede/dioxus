// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smarts-evoliposuction project

//! Evolution runner wrapping `smarts-evolution` with per-class error isolation.

use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use smarts_evolution::{
    EvolutionConfig, EvolutionTask, FoldData, FoldSample, SeedCorpus, TaskResult,
};
use smarts_rs::PreparedTarget;
use smiles_rs::Smiles;

mod observer;
#[cfg(test)]
mod tests;

use observer::TestedSmartsObserver;
pub use observer::TestedSmartsRecord;

/// Configuration passed through to `smarts-evolution`'s [`EvolutionConfig`].
#[derive(Debug, Clone)]
pub struct Config {
    pub population_size: usize,
    pub generation_limit: u64,
    pub stagnation_limit: u64,
    pub seed: Option<u64>,
    pub match_time_limit: Option<Duration>,
}

/// Input and parse failure counts collected before running evolution.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ParseCounts {
    pub positive_total: usize,
    pub negative_total: usize,
    pub positive_failed: usize,
    pub negative_failed: usize,
}

/// Outcome of evolving one positive/negative class split.
#[derive(Debug)]
pub enum EvolutionResult {
    /// Evolution succeeded with `TaskResult`, observed candidates, and parse counts.
    Ok(TaskResult, Vec<TestedSmartsRecord>, ParseCounts),
    /// At least one side had zero valid parsed structures.
    EmptyAfterParse(ParseCounts),
    /// Reading either SMILES file failed.
    ReadError(ParseCounts),
    /// `smarts-evolution` configuration or execution failed.
    EvolutionError(String, ParseCounts),
}

impl EvolutionResult {
    /// Returns the `TaskResult` when evolution succeeded.
    #[cfg(test)]
    #[must_use]
    pub(crate) const fn as_task_result(&self) -> Option<&TaskResult> {
        match self {
            Self::Ok(result, ..) => Some(result),
            _ => None,
        }
    }
}

struct EvolutionSetup {
    task: EvolutionTask,
    evolution_config: EvolutionConfig,
    seed_corpus: SeedCorpus,
    counts: ParseCounts,
}

enum SetupError {
    EmptyAfterParse(ParseCounts),
    ReadError(ParseCounts),
    InvalidConfig(String, ParseCounts),
}

impl SetupError {
    fn into_evolution_result(self) -> EvolutionResult {
        match self {
            Self::EmptyAfterParse(counts) => EvolutionResult::EmptyAfterParse(counts),
            Self::ReadError(counts) => EvolutionResult::ReadError(counts),
            Self::InvalidConfig(msg, counts) => EvolutionResult::EvolutionError(msg, counts),
        }
    }
}

fn setup_evolution(
    config: &Config,
    positive_path: &Path,
    negative_path: &Path,
) -> Result<EvolutionSetup, SetupError> {
    let positive_lines = read_smiles_lines(positive_path)
        .map_err(|_| SetupError::ReadError(ParseCounts::default()))?;
    let negative_lines = read_smiles_lines(negative_path).map_err(|_| {
        SetupError::ReadError(ParseCounts {
            positive_total: positive_lines.len(),
            ..ParseCounts::default()
        })
    })?;

    let (positive, positive_failed) = prepare_all(&positive_lines);
    let (negative, negative_failed) = prepare_all(&negative_lines);
    let counts = ParseCounts {
        positive_total: positive_lines.len(),
        negative_total: negative_lines.len(),
        positive_failed,
        negative_failed,
    };

    if positive.is_empty() || negative.is_empty() {
        return Err(SetupError::EmptyAfterParse(counts));
    }

    let mut samples = Vec::with_capacity(positive.len() + negative.len());
    samples.extend(positive.into_iter().map(FoldSample::positive));
    samples.extend(negative.into_iter().map(FoldSample::negative));

    let evolution_config =
        build_evolution_config(config).map_err(|msg| SetupError::InvalidConfig(msg, counts))?;

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

    Ok(EvolutionSetup {
        task: EvolutionTask::new(task_id, vec![FoldData::new(samples)]),
        evolution_config,
        seed_corpus: SeedCorpus::builtin(),
        counts,
    })
}

fn build_evolution_config(config: &Config) -> Result<EvolutionConfig, String> {
    let mut builder = EvolutionConfig::builder()
        .population_size(config.population_size)
        .generation_limit(config.generation_limit)
        .stagnation_limit(config.stagnation_limit);
    if let Some(seed) = config.seed {
        builder = builder.rng_seed(seed);
    }
    if let Some(timeout) = config.match_time_limit {
        builder = builder.match_time_limit(timeout);
    }
    builder.build().map_err(|e| format!("invalid config: {e}"))
}

fn evolve_inner(
    config: &Config,
    positive_path: &Path,
    negative_path: &Path,
    show_progress: bool,
) -> EvolutionResult {
    let setup = match setup_evolution(config, positive_path, negative_path) {
        Ok(v) => v,
        Err(e) => return e.into_evolution_result(),
    };

    let tested = Arc::new(Mutex::new(Vec::new()));
    let observer = TestedSmartsObserver::new(Arc::clone(&tested), show_progress);
    match setup
        .task
        .evolve_with_observer(&setup.evolution_config, &setup.seed_corpus, 1, observer)
    {
        Ok(result) => EvolutionResult::Ok(
            result,
            tested
                .lock()
                .map_or_else(|_| Vec::new(), |mut records| std::mem::take(&mut *records)),
            setup.counts,
        ),
        Err(e) => EvolutionResult::EvolutionError(e.to_string(), setup.counts),
    }
}

/// Run evolution silently in batch mode for one class split.
#[must_use]
pub fn evolve_all(config: &Config, positive_path: &Path, negative_path: &Path) -> EvolutionResult {
    evolve_inner(config, positive_path, negative_path, false)
}

/// Run evolution with stderr progress output for one class split.
#[must_use]
pub fn evolve_all_with_progress(
    config: &Config,
    positive_path: &Path,
    negative_path: &Path,
) -> EvolutionResult {
    evolve_inner(config, positive_path, negative_path, true)
}

fn read_smiles_lines(path: &Path) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut out = Vec::new();
    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            out.push(trimmed.to_string());
        }
    }
    Ok(out)
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
