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
//! use lipid_core::{analyze, lmsd_classes};
//!
//! let input = "CCCCCCCCCCCCCCCC(=O)O\tpalmitic_acid\nCCO\tethanol\n";
//! let (blocks, summary) = analyze(input);
//! assert_eq!(summary.total_items, 2);
//! assert_eq!(summary.lipid_items, 1);
//! assert!(blocks[0].is_lipid());
//! assert_eq!(lmsd_classes().len(), 86);
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
pub use chemical_class::{ChemicalClass, lmsd_classes};
pub use examples::example_smiles;
pub use export::{
    ClassTags, block_class_tags, build_filtered_mgf_content, build_filtered_smiles_content,
    class_tags_from_block, class_tags_from_gallery, insert_class_comment,
};
pub use family::LipidFamily;
pub use filter::{
    FilterCriteria, collect_adduct_options, group_classes_by_family, lmsd_classes_by_family,
    passes_item_filters,
};
pub use format::LipidFormat;
pub use lipids::{
    ElementCounts, LipidClassification, classify_acyclic_molecule, classify_formula, is_acyclic,
};
pub use parser::{
    Analysis, BlockExtractor, GalleryItem, SpectrumBlock, Summary, analyze, build_analysis,
    build_analysis_from_classified, build_gallery, classify_blocks, extract_blocks,
    extract_blocks_from_lines, gallery_item, summarize,
};

#[cfg(test)]
#[expect(clippy::unwrap_used)]
#[expect(clippy::expect_used)]
#[allow(clippy::indexing_slicing)]
mod tests;
