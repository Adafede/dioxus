// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smarts-evoliposuction project

//! Core logic for splitting SMILES datasets into positive/negative pairs
//! and running smarts-evolution on each pair.

pub mod download;
pub mod error;
pub mod evolve;
pub mod lipidsdl;
pub mod manifest;
pub mod results_io;
pub mod splitting;

pub use download::{SdfConvertError, download_lmsd, sdf_zip_to_tsv};
pub use error::CliError;
pub use evolve::{
    Config, EvolutionResult, ParseCounts, TestedSmartsRecord, evolve_all, evolve_all_with_progress,
};
pub use manifest::{ManifestRow, read_manifest, write_manifest};
pub use results_io::{RESULTS_HEADERS, already_done, open_results_writer};
pub use splitting::{
    ClassSplit, ColumnNames, Dataset, DatasetRow, Level, SkippedClass, SplitConfig, SplitError,
    SplitResult, SubclassNegatives, parse_csv, split_dataset,
};
