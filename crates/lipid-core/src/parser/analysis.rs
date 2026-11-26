// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Collection-level classification, summarization, and gallery construction.

use super::block::SpectrumBlock;
#[cfg(test)]
use super::extractor::extract_blocks;
use crate::bitset::ClassMatchSet;
use crate::chemical_class::ChemicalClass;
#[cfg(test)]
use crate::chemical_class::lmsd::lmsd_classes;
use crate::family::LipidFamily;
use crate::filter::{FilterCriteria, passes_item_filters};
use crate::lipids::types::LipidClassification;

/// Aggregate counts produced by collection summarization.
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
pub(crate) fn summarize(blocks: &[SpectrumBlock]) -> Summary {
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

/// Specificity priority when multiple LMSD classes match: prefer complex
/// lipid families (e.g. `GP`, `SP`, `GL`, `ST`) over `FattyAcyls` (`FA`).
const fn specificity_priority(family: LipidFamily) -> u8 {
    match family {
        LipidFamily::Glycerophospholipids => 7,
        LipidFamily::Sphingolipids => 6,
        LipidFamily::Saccharolipids => 5,
        LipidFamily::SterolLipids => 4,
        LipidFamily::Glycerolipids => 3,
        LipidFamily::Polyketides => 2,
        LipidFamily::PrenolLipids => 1,
        LipidFamily::FattyAcyls => 0,
    }
}

/// Select the highest-specificity matched LMSD class for gallery/export display.
#[must_use]
pub(crate) fn primary_class(
    classes: &[ChemicalClass],
    class_matches: ClassMatchSet,
    broad_family: Option<LipidFamily>,
) -> Option<&ChemicalClass> {
    classes
        .iter()
        .filter(|c| class_matches.contains(c.index))
        .max_by_key(|c| {
            let matches_broad = u8::from(broad_family == Some(c.family));
            (
                matches_broad,
                specificity_priority(c.family),
                std::cmp::Reverse(c.index),
            )
        })
}

/// Project one classified [`SpectrumBlock`] into a [`GalleryItem`] without re-parsing SMILES.
#[must_use]
pub(crate) fn gallery_item(block: &SpectrumBlock, classes: &[ChemicalClass]) -> GalleryItem {
    let class_matches = block.class_matches();
    let broad = block.classification.as_ref().map(|c| c.family);
    let first_class = primary_class(classes, class_matches, broad);
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
pub(crate) fn build_gallery(
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

/// Build up to `limit` [`GalleryItem`] cards from `blocks` matching `criteria`.
///
/// Filters all blocks before applying the `limit` cap so lipids outside the
/// unfiltered top `limit` remain visible when narrowing classes, mass, or adducts.
/// Reuses pre-rendered items from `cached_gallery` (ordered ascending by
/// `block_index`) before rendering additional blocks on demand.
#[must_use]
pub fn build_filtered_gallery(
    blocks: &[SpectrumBlock],
    cached_gallery: &[GalleryItem],
    criteria: &FilterCriteria<'_>,
    limit: usize,
    classes: &[ChemicalClass],
) -> Vec<GalleryItem> {
    blocks
        .iter()
        .filter(|block| {
            block.is_lipid()
                && passes_item_filters(
                    block.class_matches(),
                    block.exact_mass,
                    block.precursor_mz,
                    block.adduct.as_deref(),
                    criteria,
                )
        })
        .take(limit)
        .map(|block| {
            cached_gallery
                .binary_search_by_key(&block.index, |g| g.block_index)
                .ok()
                .and_then(|pos| cached_gallery.get(pos))
                .map_or_else(|| gallery_item(block, classes), Clone::clone)
        })
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
#[cfg(test)]
#[must_use]
pub(crate) fn build_analysis(mut blocks: Vec<SpectrumBlock>, gallery_limit: usize) -> Analysis {
    let all_classes = lmsd_classes();
    classify_blocks(&mut blocks, all_classes);
    build_analysis_from_classified(blocks, gallery_limit, all_classes)
}

/// Parse, classify, and summarize in-memory MGF or SMILES text in one call.
#[cfg(test)]
#[must_use]
pub(crate) fn analyze(content: &str) -> (Vec<SpectrumBlock>, Summary) {
    let mut blocks = extract_blocks(content);
    classify_blocks(&mut blocks, lmsd_classes());
    let summary = summarize(&blocks);
    (blocks, summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitset::ClassMatchSet;
    use crate::example_smiles;

    #[test]
    fn filtered_gallery_includes_classes_beyond_initial_cache_cap() {
        let input = example_smiles().join("\n");
        let mut blocks = extract_blocks(&input);
        let classes = lmsd_classes();
        classify_blocks(&mut blocks, classes);
        let analysis = build_analysis_from_classified(blocks, 16, classes);
        let first_st_block = analysis.blocks.iter().find(|b| {
            b.classification
                .as_ref()
                .is_some_and(|c| c.family == LipidFamily::SterolLipids)
        });
        assert!(
            first_st_block.is_some_and(|b| b.index > 16),
            "example dataset must contain a sterol lipid beyond cap 16"
        );
        let st_only = classes
            .iter()
            .filter(|c| c.family == LipidFamily::SterolLipids)
            .fold(ClassMatchSet::empty(), |mut set, c| {
                set.insert(c.index);
                set
            });
        let criteria = FilterCriteria {
            selected_classes: st_only,
            mz_min: 0.0,
            mz_max: 5000.0,
            precursor_min: 0.0,
            precursor_max: 5000.0,
            adduct: "",
        };
        let filtered = build_filtered_gallery(
            &analysis.blocks,
            &analysis.gallery,
            &criteria,
            16,
            analysis.all_classes,
        );
        assert!(
            !filtered.is_empty(),
            "sterols beyond cap 16 must appear when filtering by ST"
        );
    }
}
