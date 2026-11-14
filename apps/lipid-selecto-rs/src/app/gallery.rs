// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Gallery / download data preparation.
//!
//! These functions transform `GalleryItem` / `SpectrumBlock` collections
//! into formats suitable for browser download (filtered SMILES, filtered MGF
//! with class tags).

use crate::chemical_class::ChemicalClass;
use crate::parser::{GalleryItem, SpectrumBlock};
use std::collections::HashMap;
use std::fmt::Write;

use super::class_tags::{class_tags_from_block, class_tags_from_gallery};

/// Shared filter predicate used across the gallery view and MGF/SMILES downloads.
#[allow(clippy::too_many_arguments)]
pub(super) fn passes_item_filters(
    class_matches: Option<&HashMap<String, bool>>,
    exact_mass: f64,
    precursor_mz: Option<f64>,
    adduct: Option<&str>,
    selected_classes: &[String],
    mz_min: f64,
    mz_max: f64,
    prec_min: f64,
    prec_max: f64,
    adduct_filter: &str,
) -> bool {
    if selected_classes.is_empty() {
        return false;
    }
    let Some(matches) = class_matches else {
        return false;
    };
    if !selected_classes
        .iter()
        .any(|class_name| matches.get(class_name).copied().unwrap_or(false))
    {
        return false;
    }
    if exact_mass < mz_min || exact_mass > mz_max {
        return false;
    }
    if let Some(pmz) = precursor_mz
        && (pmz < prec_min || pmz > prec_max)
    {
        return false;
    }
    adduct_filter.is_empty() || adduct == Some(adduct_filter)
}

/// Compute class tags for a single block (for MGF download tagging).
///
/// Tries to find the corresponding `GalleryItem` by `block_index` to get
/// the same class as the UI. Falls back to direct block classification.
pub(super) fn block_class_tags(
    block: &SpectrumBlock,
    gallery: &[GalleryItem],
    all_classes: &[ChemicalClass],
) -> Vec<(String, String)> {
    let gallery_item = gallery
        .binary_search_by_key(&block.index, |item| item.block_index)
        .ok()
        .and_then(|idx| gallery.get(idx));
    gallery_item.map_or_else(
        || class_tags_from_block(block, all_classes),
        |item| class_tags_from_gallery(item, all_classes),
    )
}

/// Build a SMILES file content string from filtered spectrum blocks for download.
pub(super) fn build_smiles_from_blocks(
    blocks: &[&SpectrumBlock],
    gallery: &[GalleryItem],
    all_classes: &[ChemicalClass],
) -> String {
    let mut content = String::new();
    for block in blocks {
        if let Some(smiles) = block.psm_smiles.as_deref() {
            let class_tags = block_class_tags(block, gallery, all_classes);
            let cat_str = class_tags
                .iter()
                .find_map(|(k, v)| (k == "CATEGORY" && !v.is_empty()).then_some(v.as_str()))
                .unwrap_or("-");
            let mc_str = class_tags
                .iter()
                .find_map(|(k, v)| (k == "MAIN_CLASS" && !v.is_empty()).then_some(v.as_str()))
                .unwrap_or("-");
            let title_str = block
                .title
                .as_deref()
                .filter(|s| !s.is_empty())
                .unwrap_or("-");
            let _ = writeln!(content, "{smiles}\t{title_str}\t{cat_str}\t{mc_str}\t-");
        }
    }
    content
}
