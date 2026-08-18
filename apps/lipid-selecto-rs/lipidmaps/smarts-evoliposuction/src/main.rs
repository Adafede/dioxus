//! CLI entry point for the `smarts-evoliposuction` binary.
//!
//! Unified tool for downloading LIPID MAPS data, splitting into training sets,
//! and evolving SMARTS patterns.
//!
//! ## Usage examples:
//!
//! # All-in-one pipeline
//! cargo run --release -- all --output-dir ./results --generations 500
//!
//! # Individual steps
//! cargo run --release -- download
//! cargo run --release -- split --input LMSD.tsv
//! cargo run --release -p smarts-evoliposuction -- evolve --manifest `smiles_sets/manifest.csv`

use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write as _};
use std::path::{Path, PathBuf};
use std::time::Instant;

use clap::Parser;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rayon::prelude::*;
use smarts_evoliposuction::{
    Config, EvolutionResult, Level, ManifestRow, SplitConfig, TestedSmartsRecord, evolve_all,
    evolve_all_with_progress, read_manifest, sdf_zip_to_tsv, split_dataset, write_manifest,
};

#[derive(Parser, Debug)]
#[command(about = "LIPID MAPS SMARTS evolution pipeline")]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Parser, Debug)]
enum Command {
    /// Download & convert LIPID MAPS LMSD.sdf.zip
    Download {
        #[arg(long, default_value = "LMSD.sdf.zip")]
        dest: PathBuf,
    },
    /// Split dataset into positive/negative SMILES pairs
    Split {
        #[arg(long, default_value = "LMSD.tsv")]
        input: PathBuf,
        #[arg(long, default_value = "smiles_sets")]
        output_dir: PathBuf,
        #[arg(long, default_value_t = 2000)]
        smiles_cap: usize,
        #[arg(long, default_value_t = 5000)]
        max_negatives: usize,
    },
    /// Evolve SMARTS patterns
    Evolve {
        #[arg(long)]
        manifest: PathBuf,
        #[arg(long, default_value = "smarts_results.csv")]
        output: PathBuf,
        #[arg(long, default_value_t = 100)]
        population: usize,
        #[arg(long, default_value_t = 500)]
        generations: u64,
        #[arg(long, default_value_t = 50)]
        stagnation: u64,
        #[arg(long)]
        seed: Option<u64>,
        #[arg(long)]
        resume: bool,
        #[arg(long)]
        level: Option<String>,
        #[arg(long, default_value_t = 5)]
        match_timeout: u64,
        /// Disable text progress output; run classes in parallel batch mode
        #[arg(long = "no-tui", default_value_t = false)]
        no_tui: bool,
        /// Include subclass-level evolution (default: `category` + `main_class` only)
        #[arg(long = "with-subclasses", default_value_t = false)]
        with_subclasses: bool,
    },
    /// Run full pipeline: download -> split -> evolve
    All {
        #[arg(long, default_value = ".")]
        output_dir: PathBuf,
        #[arg(long, default_value_t = 500)]
        generations: u64,
        #[arg(long, default_value_t = 100)]
        population: usize,
        #[arg(long)]
        seed: Option<u64>,
        #[arg(long, default_value_t = 5)]
        match_timeout: u64,
        /// Disable text progress output; run classes in parallel batch mode
        #[arg(long = "no-tui", default_value_t = false)]
        no_tui: bool,
        /// Include subclass-level evolution (default: `category` + `main_class` only)
        #[arg(long = "with-subclasses", default_value_t = false)]
        with_subclasses: bool,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    match args.command {
        Command::Download { dest } => download_step(&dest),
        Command::Split {
            input,
            output_dir,
            smiles_cap,
            max_negatives,
        } => split_step(&input, &output_dir, smiles_cap, max_negatives),
        Command::Evolve {
            manifest,
            output,
            population,
            generations,
            stagnation,
            seed,
            resume,
            level,
            match_timeout,
            no_tui,
            with_subclasses,
        } => evolve_step(
            &manifest,
            &output,
            population,
            generations,
            stagnation,
            seed,
            resume,
            level.as_deref(),
            match_timeout,
            !no_tui,
            with_subclasses,
        ),
        Command::All {
            output_dir,
            generations,
            population,
            seed,
            match_timeout,
            no_tui,
            with_subclasses,
        } => all_steps(
            &output_dir,
            generations,
            population,
            seed,
            match_timeout,
            !no_tui,
            with_subclasses,
        ),
    }
}

fn download_step(dest: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let tsv_path = dest.with_extension("tsv");

    // Skip download if the zip already exists and is non-empty.
    if dest.exists() {
        let metadata = fs::metadata(dest)?;
        if metadata.len() > 0 {
            eprintln!(
                "[SKIP] {} already exists ({} bytes)",
                dest.display(),
                metadata.len()
            );
            if tsv_path.exists() {
                eprintln!("[OK] TSV already exists: {}", tsv_path.display());
            } else {
                eprintln!(" Converting existing SDF to TSV...");
                let (tsv, count) = sdf_zip_to_tsv(dest)?;
                fs::write(&tsv_path, tsv)?;
                eprintln!("[OK] Converted: {count} records -> {}", tsv_path.display());
            }
            return Ok(());
        }
    }

    eprintln!(" Downloading LIPID MAPS LMSD.sdf.zip from lipidmaps.org...");
    let client = reqwest::blocking::Client::new();
    let url = "https://www.lipidmaps.org/files/?file=LMSD&ext=sdf.zip";
    let resp = client.get(url).send()?;
    let bytes = resp.bytes()?;
    fs::write(dest, &bytes)?;
    eprintln!(
        "[OK] Downloaded {} bytes to {}",
        bytes.len(),
        dest.display()
    );

    eprintln!(" Converting SDF to TSV...");
    let (tsv, count) = sdf_zip_to_tsv(dest)?;
    fs::write(&tsv_path, tsv)?;
    eprintln!("[OK] Converted: {count} records -> {}", tsv_path.display());
    Ok(())
}

#[allow(clippy::too_many_lines)]
fn split_step(
    input: &Path,
    output_dir: &Path,
    smiles_cap: usize,
    max_negatives: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest_path = output_dir.join("manifest.csv");
    if manifest_path.exists() {
        eprintln!(
            "[SKIP] Manifest already exists: {}",
            manifest_path.display()
        );
        return Ok(());
    }

    eprintln!(" Splitting dataset: {}", input.display());
    let tsv_text = fs::read_to_string(input)?;

    // Parse TSV (tab-delimited) manually since parse_csv uses comma delimiter
    let lines: Vec<&str> = tsv_text.lines().collect();
    if lines.is_empty() {
        return Err("Empty input file".into());
    }

    let header: Vec<&str> = lines[0].split('\t').collect();
    let smiles_idx = header
        .iter()
        .position(|&h| h == "SMILES")
        .ok_or("Missing SMILES column")?;
    let cat_idx = header.iter().position(|&h| h == "CATEGORY");
    let main_idx = header
        .iter()
        .position(|&h| h == "MAIN_CLASS")
        .ok_or("Missing MAIN_CLASS column")?;
    let sub_idx = header
        .iter()
        .position(|&h| h == "SUB_CLASS")
        .ok_or("Missing SUB_CLASS column")?;

    let mut rows = Vec::new();
    for line in &lines[1..] {
        let cols: Vec<&str> = line.split('\t').collect();
        let max_idx = smiles_idx.max(main_idx).max(sub_idx);
        if cols.len() > max_idx {
            let smiles = cols[smiles_idx].trim();
            if !smiles.is_empty() {
                rows.push(smarts_evoliposuction::DatasetRow {
                    smiles: smiles.to_string(),
                    category: cat_idx
                        .map_or("", |i| cols.get(i).map_or("", |v| v.trim()))
                        .to_string(),
                    main_class: cols[main_idx].trim().to_string(),
                    subclass: cols[sub_idx].trim().to_string(),
                });
            }
        }
    }

    let dataset = smarts_evoliposuction::Dataset { rows };
    eprintln!("[OK] Loaded {} lipids", dataset.rows.len());

    let config = SplitConfig {
        max_negatives,
        ..SplitConfig::default()
    };
    let result = split_dataset(&dataset, &config);

    let main_count = result
        .classes
        .iter()
        .filter(|s| s.level == Level::MainClass)
        .count();
    let sub_count = result
        .classes
        .iter()
        .filter(|s| s.level == Level::Subclass)
        .count();
    let cat_count = result
        .classes
        .iter()
        .filter(|s| s.level == Level::Category)
        .count();
    eprintln!(
        "[OK] Split into {} classes ({} category, {} main, {} sub)",
        result.classes.len(),
        cat_count,
        main_count,
        sub_count
    );

    fs::create_dir_all(output_dir)?;
    let mut manifest_rows = Vec::new();
    let mut rng = StdRng::seed_from_u64(42);

    for split in &result.classes {
        // Cap and shuffle positives
        let mut positive = split.positive.clone();
        positive.shuffle(&mut rng);
        if positive.len() > smiles_cap {
            positive.truncate(smiles_cap);
        }
        positive.shuffle(&mut rng); // Shuffle again after truncation

        // Cap and shuffle negatives
        let mut negative = split.negative.clone();
        negative.shuffle(&mut rng);
        if negative.len() > smiles_cap {
            negative.truncate(smiles_cap);
        }
        negative.shuffle(&mut rng); // Shuffle again after truncation

        let pos_path = output_dir.join(format!("{}_positive.smiles", split.slug));
        let neg_path = output_dir.join(format!("{}_negative.smiles", split.slug));
        fs::write(&pos_path, positive.join("\n"))?;
        fs::write(&neg_path, negative.join("\n"))?;
        manifest_rows.push(ManifestRow {
            level: split.level.to_string(),
            label: split.label.clone(),
            category: split.category.clone(),
            main_class: split.main_class.clone(),
            subclass: split.subclass.clone(),
            slug: split.slug.clone(),
            positive_path: pos_path.to_string_lossy().to_string(),
            negative_path: neg_path.to_string_lossy().to_string(),
            positive_count: positive.len().to_string(),
            negative_count: negative.len().to_string(),
        });
    }
    write_manifest(&output_dir.join("manifest.csv"), &manifest_rows)?;
    eprintln!(
        "[OK] Manifest & {} SMILES files written to {}",
        manifest_rows.len(),
        output_dir.display()
    );
    Ok(())
}

#[allow(clippy::too_many_arguments, clippy::ptr_arg, clippy::too_many_lines)]
fn evolve_step(
    manifest: &Path,
    output: &Path,
    population: usize,
    generations: u64,
    stagnation: u64,
    seed: Option<u64>,
    resume: bool,
    level: Option<&str>,
    match_timeout_secs: u64,
    show_progress: bool,
    with_subclasses: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut rows = read_manifest(manifest)?;
    if let Some(filter) = level {
        rows.retain(|r| r.level == filter);
    } else if !with_subclasses {
        // By default: only evolve category + main_class, skip subclass.
        rows.retain(|r| r.level == "category" || r.level == "main_class");
    }
    rows = order_rows_for_evolution(rows);

    let done = if resume {
        already_done(output)
    } else {
        HashSet::new()
    };
    let tested_smarts_dir = output.with_file_name(format!(
        "{}_tested_smarts",
        output
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("smarts_results")
    ));
    fs::create_dir_all(&tested_smarts_dir)?;

    let out_exists = output.exists();
    let mut out = OpenOptions::new().create(true).append(true).open(output)?;
    let mut writer = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(&mut out);

    if !out_exists {
        writer.write_record([
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
        ])?;
    }

    let config = Config {
        population_size: population,
        generation_limit: generations,
        stagnation_limit: stagnation,
        seed,
        match_time_limit: Some(std::time::Duration::from_secs(match_timeout_secs)),
    };

    eprintln!(" Evolving SMARTS for {} classes...", rows.len());

    if show_progress {
        // Sequential mode with text progress output — one class at a time.
        for (i, row) in rows.iter().enumerate() {
            if done.contains(&row.slug) {
                eprintln!("[{i}/{}] {} skipped (already done)", rows.len(), row.slug);
                continue;
            }
            let len = rows.len();
            eprintln!("[{i}/{len}] {} ", row.slug);
            std::io::stderr().flush().ok();

            let t0 = Instant::now();
            let result = evolve_all_with_progress(
                &config,
                Path::new(&row.positive_path),
                Path::new(&row.negative_path),
            );
            let elapsed = t0.elapsed().as_secs_f64();

            eprintln!("(done in {elapsed:.1}s)");

            let (p, n, pf, nf, status, smarts, mcc, cov, slen, gens, tested_count, tested_path) =
                process_evolution_result(&result, row, &tested_smarts_dir)?;

            writer.write_record([
                &row.level,
                &row.label,
                &row.category,
                &row.main_class,
                &row.subclass,
                &row.slug,
                &p.to_string(),
                &n.to_string(),
                &pf.to_string(),
                &nf.to_string(),
                &smarts,
                &mcc,
                &cov,
                &slen,
                &gens,
                &format!("{elapsed:.3}"),
                status,
                &tested_count,
                &tested_path,
            ])?;
            writer.flush()?;
        }
    } else {
        // Parallel mode — evolve classes concurrently via rayon.
        // smarts-evolution already parallelises *within* a generation;
        // we also parallelise *across* classes for additional throughput.
        eprintln!(" Running in parallel batch mode (no progress output).");

        let to_evolve: Vec<&ManifestRow> =
            rows.iter().filter(|r| !done.contains(&r.slug)).collect();
        let total = to_evolve.len();

        let results: Vec<EvolResult> = to_evolve
            .par_iter()
            .enumerate()
            .map(|(i, row)| {
                let t0 = Instant::now();
                eprintln!("[{i}/{total}] {} evolving...", row.slug);
                let result = evolve_all(
                    &config,
                    Path::new(&row.positive_path),
                    Path::new(&row.negative_path),
                );
                let elapsed = t0.elapsed().as_secs_f64();
                eprintln!("[{i}/{total}] {} done in {elapsed:.1}s", row.slug);
                EvolResult {
                    row: (*row).clone(),
                    result,
                    elapsed,
                }
            })
            .collect();

        // Write results sequentially (CSV writer is not thread-safe).
        for res in results {
            let row = &res.row;
            let elapsed = res.elapsed;

            let (p, n, pf, nf, status, smarts, mcc, cov, slen, gens, tested_count, tested_path) =
                process_evolution_result(&res.result, row, &tested_smarts_dir)?;

            writer.write_record([
                &row.level,
                &row.label,
                &row.category,
                &row.main_class,
                &row.subclass,
                &row.slug,
                &p.to_string(),
                &n.to_string(),
                &pf.to_string(),
                &nf.to_string(),
                &smarts,
                &mcc,
                &cov,
                &slen,
                &gens,
                &format!("{elapsed:.3}"),
                status,
                &tested_count,
                &tested_path,
            ])?;
            writer.flush()?;
        }
    }
    eprintln!("[OK] Results: {}", output.display());
    Ok(())
}

/// Intermediate result from a parallel evolution task.
struct EvolResult {
    row: ManifestRow,
    result: EvolutionResult,
    elapsed: f64,
}

/// CSV field tuple returned by [`process_evolution_result`].
type CsvFields = (
    usize,
    usize,
    usize,
    usize,
    &'static str,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
);

/// Extract CSV fields from an `EvolutionResult`, writing the tested-SMARTS CSV
/// if evolution succeeded.
fn process_evolution_result(
    result: &EvolutionResult,
    row: &ManifestRow,
    tested_smarts_dir: &Path,
) -> Result<CsvFields, Box<dyn std::error::Error>> {
    match result {
        EvolutionResult::Ok(r, tested_smarts, p, n, pf, nf) => {
            eprintln!("mcc={:.4}", r.best_mcc());
            let path = tested_smarts_dir.join(format!("{}_tested_smarts.csv", row.slug));
            write_tested_smarts_csv(&path, tested_smarts)?;
            Ok((
                *p,
                *n,
                *pf,
                *nf,
                "ok",
                r.best_smarts().to_string(),
                r.best_mcc().to_string(),
                r.best_coverage_score().to_string(),
                r.best_smarts_len().to_string(),
                r.generations().to_string(),
                tested_smarts.len().to_string(),
                path.to_string_lossy().to_string(),
            ))
        }
        EvolutionResult::EmptyAfterParse(p, n, pf, nf) => {
            eprintln!("SKIPPED {}", row.slug);
            Ok((
                *p,
                *n,
                *pf,
                *nf,
                "empty_after_parse",
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                "0".into(),
                String::new(),
            ))
        }
        EvolutionResult::ReadError(p, n, pf, nf) => {
            eprintln!("FAILED {}", row.slug);
            Ok((
                *p,
                *n,
                *pf,
                *nf,
                "read_error",
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                "0".into(),
                String::new(),
            ))
        }
        EvolutionResult::EvolutionError(msg, p, n, pf, nf) => {
            eprintln!("ERROR: {msg}");
            Ok((
                *p,
                *n,
                *pf,
                *nf,
                "evolution_error",
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                "0".into(),
                String::new(),
            ))
        }
    }
}

fn order_rows_for_evolution(rows: Vec<ManifestRow>) -> Vec<ManifestRow> {
    let mut category_rows = Vec::new();
    let mut main_rows = Vec::new();
    let mut subclass_rows = Vec::new();
    for row in rows {
        match row.level.as_str() {
            "category" => category_rows.push(row),
            "subclass" => subclass_rows.push(row),
            _ => main_rows.push(row),
        }
    }

    let sort_by_avg_len = |group: &mut Vec<ManifestRow>| {
        group.sort_by(|a, b| {
            let avg_a = average_smiles_row_len(a);
            let avg_b = average_smiles_row_len(b);
            avg_a.total_cmp(&avg_b).then_with(|| a.slug.cmp(&b.slug))
        });
    };

    sort_by_avg_len(&mut category_rows);
    sort_by_avg_len(&mut main_rows);
    sort_by_avg_len(&mut subclass_rows);
    category_rows.extend(main_rows);
    category_rows.extend(subclass_rows);
    category_rows
}

#[allow(clippy::cast_precision_loss)]
fn average_smiles_row_len(row: &ManifestRow) -> f64 {
    let (sum_pos, cnt_pos) = average_smiles_len(Path::new(&row.positive_path));
    let (sum_neg, cnt_neg) = average_smiles_len(Path::new(&row.negative_path));
    let total_count = cnt_pos + cnt_neg;
    if total_count == 0 {
        return f64::INFINITY;
    }
    (sum_pos + sum_neg) as f64 / total_count as f64
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
        if trimmed.is_empty() {
            continue;
        }
        total_len += trimmed.len();
        count += 1;
    }
    (total_len, count)
}

fn write_tested_smarts_csv(
    output_path: &Path,
    tested_smarts: &[TestedSmartsRecord],
) -> Result<(), Box<dyn std::error::Error>> {
    // Sort by MCC (desc), then coverage_score (desc), then generation (asc).
    // This groups the most selective patterns first, and within equal metrics
    // shows earlier generations first.
    let mut sorted: Vec<&TestedSmartsRecord> = tested_smarts.iter().collect();
    sorted.sort_by(|a, b| {
        b.mcc
            .partial_cmp(&a.mcc)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                b.coverage_score
                    .partial_cmp(&a.coverage_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| a.generation.cmp(&b.generation))
    });

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

#[allow(clippy::too_many_arguments)]
fn all_steps(
    out_dir: &Path,
    generations: u64,
    population: usize,
    seed: Option<u64>,
    match_timeout: u64,
    show_progress: bool,
    with_subclasses: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(out_dir)?;

    eprintln!("\n=== Step 1/4: Download");
    let zip = out_dir.join("LMSD.sdf.zip");
    download_step(&zip)?;

    eprintln!("\n=== Step 2/4: Split");
    let tsv = out_dir.join("LMSD.sdf.tsv");
    let sets = out_dir.join("smiles_sets");
    split_step(&tsv, &sets, 2000, 5000)?;

    eprintln!("\n=== Step 3/4: Evolve");
    let manifest = sets.join("manifest.csv");
    let results = out_dir.join("smarts_results.csv");
    evolve_step(
        &manifest,
        &results,
        population,
        generations,
        50,
        seed,
        true, // resume: skip classes already in results CSV
        None,
        match_timeout,
        show_progress,
        with_subclasses,
    )?;

    eprintln!("\n=== Step 4/4: Summary");
    eprintln!("[DONE] Pipeline complete! Results: {}", results.display());
    Ok(())
}

fn already_done(output: &Path) -> HashSet<String> {
    let mut done = HashSet::new();
    if let Ok(mut reader) = csv::Reader::from_path(output) {
        for record in reader
            .deserialize::<std::collections::HashMap<String, String>>()
            .flatten()
        {
            if let Some(slug) = record.get("slug") {
                done.insert(slug.clone());
            }
        }
    }
    done
}
