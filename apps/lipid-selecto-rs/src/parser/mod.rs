//! MGF parsing and chemical class matching.
//!
//! The parser walks an MGF file block-by-block (`BEGIN IONS ... END IONS`),
//! preserving each block's verbatim text so that the filtered MGF is a faithful
//! subset of the input (no re-serialization drift). For every block it extracts
//! SMILES and FORMULA metadata, then matches against user-defined chemical classes.
//!
//! Split by responsibility:
//! - `parsing`: `SpectrumBlock` (struct + classification methods) and block
//!   extraction.
//! - `analysis`: aggregation, summary, gallery, class matching across the
//!   collection, and filtered-MGF generation.

#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]

mod analysis;
mod parsing;

pub use analysis::{
    Analysis, GalleryItem, Summary, analyze, build_analysis, build_analysis_from_classified,
    build_filtered_mgf, build_filtered_mgf_with_classes, build_gallery, classify_blocks,
    gallery_item, summarize,
};
pub use parsing::{SpectrumBlock, extract_blocks, extract_blocks_from_lines};
