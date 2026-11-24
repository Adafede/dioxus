// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Collection-level classification, summarization, and gallery construction.

use super::block::SpectrumBlock;
use super::extractor::extract_blocks;
use crate::bitset::ClassMatchSet;
use crate::chemical_class::{ChemicalClass, lmsd_classes};
use crate::lipids::LipidClassification;

/// Aggregate counts produced by [`summarize`] / [`analyze`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Summary {
    /// Total number of blocks parsed from the input.
    pub total_items: usize,
    /// Blocks matching at least one LMSD chemical class.
    pub lipid_items: usize,
    /// Annotated blocks that matched zero lipid classes.
    pub unclassified: usize,
    /// Blocks missing both `SMILES` and `FORMULA` metadata.
    pub skipped: usize,
}

/// Summarize classification counts across `blocks`.
#[must_use]
pub fn summarize(blocks: &[SpectrumBlock]) -> Summary {
    let mut summary = Summary {
        total_items: blocks.len(),
        ..Summary::default()
    };
    for block in blocks {
        let annotated = block
            .psm_smiles
            .as_deref()
            .is_some_and(|s| !s.trim().is_empty())
            || block
                .formula
                .as_deref()
                .is_some_and(|s| !s.trim().is_empty());
        if !annotated {
            summary.skipped += 1;
        } else if block.is_lipid() {
            summary.lipid_items += 1;
        } else {
            summary.unclassified += 1;
        }
    }
    summary
}

/// Lightweight view of one classified spectrum for gallery cards.
#[derive(Clone, Debug, PartialEq)]
pub struct GalleryItem {
    /// 1-based block index linking back to [`SpectrumBlock::index`].
    pub block_index: usize,
    /// Spectrum or compound title.
    pub title: Option<String>,
    /// Annotated SMILES string.
    pub smiles: Option<String>,
    /// Precomputed monoisotopic mass.
    pub exact_mass: f64,
    /// Observed precursor m/z, if available.
    pub precursor_mz: Option<f64>,
    /// Adduct annotation, if present.
    pub adduct: Option<String>,
    /// Pre-rendered compact 2D SVG diagram.
    pub svg: String,
    /// Compact bitset of matched LMSD classes.
    pub class_matches: ClassMatchSet,
    /// Microshades color of the highest-priority matched class.
    pub primary_class_color: String,
    /// Display name of the highest-priority matched class.
    pub primary_class_name: Option<String>,
    /// Broad lipid classification assignment.
    pub classification: Option<LipidClassification>,
}

/// Project one classified [`SpectrumBlock`] into a [`GalleryItem`] without re-parsing SMILES.
#[must_use]
pub fn gallery_item(block: &SpectrumBlock, classes: &[ChemicalClass]) -> GalleryItem {
    let class_matches = block.class_matches();
    let first_class = classes.iter().find(|c| class_matches.contains(c.index));
    let primary_class_color =
        first_class.map_or_else(|| "#f1f5f9".to_string(), |c| c.color.to_string());
    let primary_class_name = first_class.map(|c| c.name.clone());

    let svg = block.parsed_molecule.as_ref().map_or_else(
        || {
            block.psm_smiles.as_deref().map_or_else(
                chem_depict::empty_compact_svg,
                chem_depict::render_smiles_svg_compact,
            )
        },
        chem_depict::render_molecule_svg_compact,
    );

    GalleryItem {
        block_index: block.index,
        title: block.title.clone(),
        smiles: block.psm_smiles.clone(),
        exact_mass: block.exact_mass,
        precursor_mz: block.precursor_mz,
        adduct: block.adduct.clone(),
        svg,
        class_matches,
        primary_class_color,
        primary_class_name,
        classification: block.classification.clone(),
    }
}

/// Build up to `limit` gallery items from lipid-positive blocks.
#[must_use]
pub fn build_gallery(
    blocks: &[SpectrumBlock],
    limit: usize,
    classes: &[ChemicalClass],
) -> Vec<GalleryItem> {
    blocks
        .iter()
        .filter(|block| block.is_lipid())
        .take(limit)
        .map(|block| gallery_item(block, classes))
        .collect()
}

/// Complete analysis bundle produced from uploaded spectra.
#[derive(Clone, Debug)]
pub struct Analysis {
    /// Aggregate summary counts.
    pub summary: Summary,
    /// Capped gallery cards with pre-rendered 2D SVGs.
    pub gallery: Vec<GalleryItem>,
    /// All parsed blocks for interactive re-filtering and export.
    pub blocks: Vec<SpectrumBlock>,
    /// Shared static slice of precompiled LMSD classes (zero cloning).
    pub all_classes: &'static [ChemicalClass],
}

/// Classify `blocks` against `classes` in place.
pub fn classify_blocks(blocks: &mut [SpectrumBlock], classes: &[ChemicalClass]) {
    for block in blocks {
        block.classify_and_match(classes);
    }
}

/// Assemble an [`Analysis`] from already-classified blocks.
#[must_use]
pub fn build_analysis_from_classified(
    blocks: Vec<SpectrumBlock>,
    gallery_limit: usize,
    all_classes: &'static [ChemicalClass],
) -> Analysis {
    let summary = summarize(&blocks);
    let gallery = build_gallery(&blocks, gallery_limit, all_classes);
    Analysis {
        summary,
        gallery,
        blocks,
        all_classes,
    }
}

/// Classify `blocks` against LMSD classes and assemble an [`Analysis`].
#[must_use]
pub fn build_analysis(mut blocks: Vec<SpectrumBlock>, gallery_limit: usize) -> Analysis {
    let all_classes = lmsd_classes();
    classify_blocks(&mut blocks, all_classes);
    build_analysis_from_classified(blocks, gallery_limit, all_classes)
}

/// Parse, classify, and summarize in-memory MGF or SMILES text in one call.
#[must_use]
pub fn analyze(content: &str) -> (Vec<SpectrumBlock>, Summary) {
    let mut blocks = extract_blocks(content);
    classify_blocks(&mut blocks, lmsd_classes());
    let summary = summarize(&blocks);
    (blocks, summary)
}
