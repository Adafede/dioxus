// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Pure-Rust LIPID MAPS lipid classification, MGF/SMILES extraction, and export filtering.
//!
//! Moves the core chemistry, SMARTS matching, MGF block state machine,
//! metadata tagging, and export builders out of the Dioxus browser shell.
//!
//! # Example
//!
//! ```rust
//! use lipid_core::{
//!     BlockExtractor, build_analysis_from_classified, classify_blocks, lmsd_classes,
//! };
//!
//! let input = "CCCCCCCCCCCCCCCC(=O)O\tpalmitic_acid\nCCO\tethanol\n";
//! let mut extractor = BlockExtractor::default();
//! for line in input.lines() {
//!     extractor.push_line(line);
//! }
//! let mut blocks = extractor.finish();
//! let classes = lmsd_classes();
//! classify_blocks(&mut blocks, classes);
//! let analysis = build_analysis_from_classified(blocks, 16, classes);
//! assert_eq!(analysis.summary.total_items, 2);
//! assert_eq!(analysis.summary.lipid_items, 1);
//! assert!(analysis.blocks[0].is_lipid());
//! assert_eq!(classes.len(), 86);
//! ```

#![warn(missing_docs)]

mod bitset;
mod chemical_class;
mod examples;
mod export;
mod family;
mod filter;
mod format;
mod lipids;
mod parser;

pub use bitset::ClassMatchSet;
pub use chemical_class::ChemicalClass;
pub use chemical_class::lmsd::lmsd_classes;
pub use examples::example_smiles;
pub use export::{build_filtered_mgf_content, build_filtered_smiles_content};
pub use family::LipidFamily;
pub use filter::{
    FilterCriteria, collect_adduct_options, lmsd_classes_by_family, passes_item_filters,
};
pub use format::LipidFormat;
pub use lipids::types::LipidClassification;
pub use parser::analysis::{
    Analysis, GalleryItem, Summary, build_analysis_from_classified, classify_blocks,
};
pub use parser::block::SpectrumBlock;
pub use parser::extractor::BlockExtractor;

#[cfg(test)]
#[expect(clippy::unwrap_used)]
#[expect(clippy::expect_used)]
#[expect(
    clippy::indexing_slicing,
    reason = "tests index fixtures of asserted lengths"
)]
mod tests;
