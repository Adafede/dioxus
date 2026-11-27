// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smarts-evoliposuction project

//! Core logic for splitting SMILES datasets into positive/negative pairs
//! and running smarts-evolution on each pair.

mod download;
mod error;
mod evolve;
mod manifest;
mod results_io;
mod splitting;

pub use download::{LIPIDMAPS_LMSD_URL, SdfConvertError, sdf_zip_to_tsv};
pub use error::CliError;
pub use evolve::{
    Config, EvolutionResult, ParseCounts, TestedSmartsRecord, evolve_all, evolve_all_with_progress,
};
pub use manifest::{ManifestRow, read_manifest, write_manifest};
pub use results_io::{RESULTS_HEADERS, ResultsOpenMode, already_done, open_results_writer};
pub use splitting::{
    ClassSplit, Dataset, DatasetRow, Level, SkippedClass, SplitConfig, SplitResult,
    SubclassNegatives, parse_lmsd_tsv, split_dataset,
};
