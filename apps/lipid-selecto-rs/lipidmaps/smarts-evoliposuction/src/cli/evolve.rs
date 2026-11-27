// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smarts-evoliposuction project

//! SMARTS evolution CLI execution step (sequential and rayon parallel modes).

use std::collections::HashSet;
use std::fs;
use std::io::Write as _;
use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use rayon::prelude::*;
use smarts_evoliposuction::{
    CliError, Config, EvolutionResult, ManifestRow, RESULTS_HEADERS, ResultsOpenMode, already_done,
    evolve_all, evolve_all_with_progress, open_results_writer, read_manifest,
};

use super::LevelFilter;
use super::ordering::order_rows_for_evolution;
use super::report::{process_evolution_result, write_evolution_fields};

/// Configuration for [`evolve_step`].
pub(crate) struct EvolveOptions<'a> {
    pub(crate) manifest: &'a Path,
    pub(crate) output: &'a Path,
    pub(crate) population: usize,
    pub(crate) generations: u64,
    pub(crate) stagnation: u64,
    pub(crate) seed: Option<u64>,
    pub(crate) resume: bool,
    pub(crate) level: Option<LevelFilter>,
    pub(crate) match_timeout_secs: u64,
    pub(crate) show_progress: bool,
    pub(crate) with_subclasses: bool,
}

pub(crate) fn evolve_step(opts: &EvolveOptions<'_>) -> Result<(), CliError> {
    let rows = load_filtered_rows(opts.manifest, opts.level, opts.with_subclasses)?;
    let done = if opts.resume {
        already_done(opts.output)?
    } else {
        HashSet::new()
    };

    let stem = opts
        .output
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("smarts_results");
    let tested_smarts_dir = opts.output.with_file_name(format!("{stem}_tested_smarts"));
    fs::create_dir_all(&tested_smarts_dir)?;

    let mode = if opts.resume {
        ResultsOpenMode::Resume
    } else {
        ResultsOpenMode::Truncate
    };
    let (mut writer, need_headers) = open_results_writer(opts.output, mode)?;
    if need_headers {
        writer.write_record(RESULTS_HEADERS)?;
    }

    let config = Config {
        population_size: opts.population,
        generation_limit: opts.generations,
        stagnation_limit: opts.stagnation,
        seed: opts.seed,
        match_time_limit: Some(Duration::from_secs(opts.match_timeout_secs)),
    };

    eprintln!(" Evolving SMARTS for {} classes...", rows.len());
    if opts.show_progress {
        run_sequential(&rows, &done, &config, &mut writer, &tested_smarts_dir)?;
    } else {
        run_parallel(&rows, &done, &config, &mut writer, &tested_smarts_dir)?;
    }
    eprintln!("[OK] Results: {}", opts.output.display());
    Ok(())
}

fn load_filtered_rows(
    manifest: &Path,
    level: Option<LevelFilter>,
    with_subclasses: bool,
) -> Result<Vec<ManifestRow>, CliError> {
    let mut rows = read_manifest(manifest)?;
    if let Some(filter) = level {
        rows.retain(|r| r.level == filter.as_str());
    } else if !with_subclasses {
        rows.retain(|r| r.level == "category" || r.level == "main_class");
    }
    Ok(order_rows_for_evolution(rows))
}

fn evolve_one_row(
    idx: usize,
    total: usize,
    row: &ManifestRow,
    config: &Config,
    show_progress: bool,
) -> (EvolutionResult, f64) {
    let pos_path = Path::new(&row.positive_path);
    let neg_path = Path::new(&row.negative_path);
    let start = Instant::now();
    let result = if show_progress {
        eprintln!("[{idx}/{total}] {} ", row.slug);
        drop(std::io::stderr().flush());
        evolve_all_with_progress(config, pos_path, neg_path)
    } else {
        eprintln!("[{idx}/{total}] {} evolving...", row.slug);
        evolve_all(config, pos_path, neg_path)
    };
    let elapsed = start.elapsed().as_secs_f64();
    if show_progress {
        eprintln!("(done in {elapsed:.1}s)");
    } else {
        eprintln!("[{idx}/{total}] {} done in {elapsed:.1}s", row.slug);
    }
    (result, elapsed)
}

fn run_sequential(
    rows: &[ManifestRow],
    done: &HashSet<String>,
    config: &Config,
    writer: &mut csv::Writer<fs::File>,
    tested_smarts_dir: &Path,
) -> Result<(), CliError> {
    let len = rows.len();
    for (i, row) in rows.iter().enumerate() {
        if done.contains(&row.slug) {
            eprintln!("[{i}/{len}] {} skipped (already done)", row.slug);
            continue;
        }
        let (result, elapsed) = evolve_one_row(i, len, row, config, true);
        let fields = process_evolution_result(&result, row, tested_smarts_dir)?;
        write_evolution_fields(writer, row, elapsed, &fields)?;
    }
    Ok(())
}

fn run_parallel(
    rows: &[ManifestRow],
    done: &HashSet<String>,
    config: &Config,
    writer: &mut csv::Writer<fs::File>,
    tested_smarts_dir: &Path,
) -> Result<(), CliError> {
    eprintln!(" Running in parallel batch mode (no progress output).");
    let to_evolve: Vec<&ManifestRow> = rows.iter().filter(|r| !done.contains(&r.slug)).collect();
    let total = to_evolve.len();
    let writer_mutex = Mutex::new(writer);

    to_evolve
        .par_iter()
        .enumerate()
        .try_for_each(|(i, row)| -> Result<(), CliError> {
            let (result, elapsed) = evolve_one_row(i, total, row, config, false);
            let fields = process_evolution_result(&result, row, tested_smarts_dir)?;
            let mut guard = writer_mutex
                .lock()
                .map_err(|_| CliError::Io(std::io::Error::other("results writer lock poisoned")))?;
            write_evolution_fields(&mut guard, row, elapsed, &fields)
        })
}
