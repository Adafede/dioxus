//! Core logic for splitting SMILES datasets into positive/negative pairs
//! and running smarts-evolution on each pair.
//!
//! This crate is split into two layers:
//! - [`splitting`] — pure Rust, no external dependencies. Parses a CSV into a
//!   [`Dataset`], then produces balanced [`ClassSplit`]s per `CATEGORY`,
//!   `MAIN_CLASS`, and `SUB_CLASS`. This mirrors the `split_for_smarts_evolution.py`
//!   script but in idiomatic Rust with deterministic seeded sampling.
//! - [`evolve`] — wraps `smarts-evolution`'s `EvolutionTask::evolve` for each
//!   class split, catching parse failures and evolution errors per-class
//!   rather than aborting the whole batch.
//! - [`download`] — downloads and converts LMSD.sdf.zip → unique TSV using
//!   [`lipidsdl`]. Also handles the zip → TSV conversion.
//! - [`manifest`] — manifest CSV reading/writing.
//!
//! The `main` binary is a thin CLI over this library.
//!
//! ## External dependencies
//!
//! This crate wraps [`smarts-evolution`](https://github.com/earth-metabolome-initiative/smarts-evolution),
//! [`smarts-rs`](https://github.com/earth-metabolome-initiative/smarts-rs), and
//! [`smiles-parser`](https://github.com/earth-metabolome-initiative/smiles-parser)
//! by **Luca Cappelletti** (`@LucaCappelletti94`), Earth Metabolome Initiative.

pub mod download;
pub mod evolve;
pub mod manifest;
pub mod splitting;

pub use download::{SdfConvertError, download_lmsd, sdf_zip_to_tsv};
pub use evolve::{
    Config, EvolutionResult, TestedSmartsRecord, evolve_all, evolve_all_with_progress,
};
pub use manifest::{ManifestRow, read_manifest, write_manifest};
pub use splitting::{
    ClassSplit, ColumnNames, Dataset, DatasetRow, Level, SkippedClass, SplitConfig, SplitError,
    SplitResult, SubclassNegatives, parse_csv, split_dataset,
};
