// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Streaming MGF/SMILES extraction and collection analysis.

mod analysis;
mod block;
mod extractor;

pub use analysis::{
    Analysis, GalleryItem, Summary, analyze, build_analysis, build_analysis_from_classified,
    build_gallery, classify_blocks, gallery_item, summarize,
};
pub use block::SpectrumBlock;
pub use extractor::{BlockExtractor, extract_blocks, extract_blocks_from_lines};
