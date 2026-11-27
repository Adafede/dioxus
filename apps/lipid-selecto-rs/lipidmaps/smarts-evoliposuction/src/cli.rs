// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smarts-evoliposuction project

//! Command-line argument definition and subcommand dispatch.

use std::fs;
use std::path::{Path, PathBuf};

use clap::{Args as ClapArgs, Parser, ValueEnum};
use smarts_evoliposuction::CliError;

pub(crate) mod download;
pub(crate) mod evolve;
pub(crate) mod ordering;
pub(crate) mod report;
pub(crate) mod split;

use download::download_step;
use evolve::{EvolveOptions, evolve_step};
use split::split_step;

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LevelFilter {
    #[value(name = "category")]
    Category,
    #[value(name = "main_class")]
    MainClass,
    #[value(name = "subclass")]
    Subclass,
}

impl LevelFilter {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Category => "category",
            Self::MainClass => "main_class",
            Self::Subclass => "subclass",
        }
    }
}

#[derive(ClapArgs, Debug)]
pub(crate) struct CommonEvolveArgs {
    #[arg(long, default_value_t = 100)]
    pub(crate) population: usize,
    #[arg(long, default_value_t = 500)]
    pub(crate) generations: u64,
    #[arg(long)]
    pub(crate) seed: Option<u64>,
    #[arg(long, default_value_t = 5)]
    pub(crate) match_timeout: u64,
    /// Disable text progress output; run classes in parallel batch mode
    #[arg(long = "no-tui", default_value_t = false)]
    pub(crate) no_tui: bool,
    /// Include subclass-level evolution (default: `category` + `main_class` only)
    #[arg(long = "with-subclasses", default_value_t = false)]
    pub(crate) with_subclasses: bool,
}

#[derive(Parser, Debug)]
#[command(about = "LIPID MAPS SMARTS evolution pipeline")]
pub(crate) struct Args {
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Parser, Debug)]
pub(crate) enum Command {
    /// Download & convert LIPID MAPS LMSD.sdf.zip
    Download {
        #[arg(long, default_value = "LMSD.sdf.zip")]
        dest: PathBuf,
    },
    /// Split dataset into positive/negative SMILES pairs
    Split {
        #[arg(long, default_value = "LMSD.sdf.tsv")]
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
        #[arg(long, default_value_t = 50)]
        stagnation: u64,
        #[arg(long)]
        resume: bool,
        #[arg(long, value_enum)]
        level: Option<LevelFilter>,
        #[command(flatten)]
        common: CommonEvolveArgs,
    },
    /// Run full pipeline: download -> split -> evolve
    All {
        #[arg(long, default_value = ".")]
        output_dir: PathBuf,
        #[command(flatten)]
        common: CommonEvolveArgs,
    },
}

struct PipelineOptions<'a> {
    out_dir: &'a Path,
    common: &'a CommonEvolveArgs,
}

pub(crate) fn run(args: Args) -> Result<(), CliError> {
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
            stagnation,
            resume,
            level,
            common,
        } => evolve_step(&EvolveOptions {
            manifest: &manifest,
            output: &output,
            population: common.population,
            generations: common.generations,
            stagnation,
            seed: common.seed,
            resume,
            level,
            match_timeout_secs: common.match_timeout,
            show_progress: !common.no_tui,
            with_subclasses: common.with_subclasses,
        }),
        Command::All { output_dir, common } => all_steps(&PipelineOptions {
            out_dir: &output_dir,
            common: &common,
        }),
    }
}

fn all_steps(opts: &PipelineOptions<'_>) -> Result<(), CliError> {
    fs::create_dir_all(opts.out_dir)?;

    eprintln!("\n=== Step 1/4: Download");
    download_step(&opts.out_dir.join("LMSD.sdf.zip"))?;

    eprintln!("\n=== Step 2/4: Split");
    let tsv = opts.out_dir.join("LMSD.sdf.tsv");
    let sets = opts.out_dir.join("smiles_sets");
    split_step(&tsv, &sets, 2000, 5000)?;

    eprintln!("\n=== Step 3/4: Evolve");
    let manifest = sets.join("manifest.csv");
    let results = opts.out_dir.join("smarts_results.csv");
    evolve_step(&EvolveOptions {
        manifest: &manifest,
        output: &results,
        population: opts.common.population,
        generations: opts.common.generations,
        stagnation: 50,
        seed: opts.common.seed,
        resume: true,
        level: None,
        match_timeout_secs: opts.common.match_timeout,
        show_progress: !opts.common.no_tui,
        with_subclasses: opts.common.with_subclasses,
    })?;

    eprintln!("\n=== Step 4/4: Summary");
    eprintln!("[DONE] Pipeline complete! Results: {}", results.display());
    Ok(())
}
