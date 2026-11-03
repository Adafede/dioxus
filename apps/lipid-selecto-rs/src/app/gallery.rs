// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Gallery / download data preparation.
//!
//! These functions transform `GalleryItem` / `SpectrumBlock` collections
//! into formats suitable for browser download (filtered SMILES, filtered MGF
//! with class tags).

use crate::chemical_class::ChemicalClass;
use crate::parser::{GalleryItem, SpectrumBlock};
#[cfg(any(test, target_arch = "wasm32"))]
use std::fmt::Write;

use super::class_tags::{class_tags_from_block, class_tags_from_gallery};
#[cfg(any(test, target_arch = "wasm32"))]
use super::types::GallerySmilesEntry;

/// Pre-compute class tags for each block (for MGF download tagging).
///
/// Tries to find the corresponding `GalleryItem` by `block_index` to get
/// the same class as the UI. Falls back to direct block classification.
pub(super) fn prepare_block_class_tags(
    blocks: &[SpectrumBlock],
    gallery: &[GalleryItem],
    all_classes: &[ChemicalClass],
) -> Vec<Vec<(String, String)>> {
    blocks
        .iter()
        .map(|block| {
            let gallery_item = gallery
                .binary_search_by_key(&block.index, |item| item.block_index)
                .ok()
                .and_then(|idx| gallery.get(idx));
            gallery_item.map_or_else(
                || class_tags_from_block(block, all_classes),
                |item| class_tags_from_gallery(item, all_classes),
            )
        })
        .collect()
}

/// Build a SMILES file content string from gallery-smiles entries for download.
#[cfg(any(test, target_arch = "wasm32"))]
pub(super) fn build_smiles_from_gallery(gallery: &[&GallerySmilesEntry]) -> String {
    let mut content = String::new();
    for (title, smiles, category, main_class, _sub_class, _, _, _, _) in gallery {
        if let Some(smiles) = smiles {
            let cat_str = category.as_deref().filter(|s| !s.is_empty()).unwrap_or("-");
            let mc_str = main_class
                .as_deref()
                .filter(|s| !s.is_empty())
                .unwrap_or("-");
            let title_str = title.as_deref().filter(|s| !s.is_empty()).unwrap_or("-");
            let _ = writeln!(content, "{smiles}\t{title_str}\t{cat_str}\t{mc_str}\t-");
        }
    }
    content
}

/// Prepare gallery SMILES entries for WASM download closures.
#[cfg(any(test, target_arch = "wasm32"))]
pub(super) fn prepare_gallery_smiles(
    gallery: &[GalleryItem],
    all_classes: &[ChemicalClass],
) -> Vec<GallerySmilesEntry> {
    gallery
        .iter()
        .map(|item| {
            let class_tags = class_tags_from_gallery(item, all_classes);
            let category = class_tags
                .iter()
                .find_map(|(k, v)| (k == "CATEGORY").then(|| v.clone()))
                .unwrap_or_else(|| "-".to_string());
            let main_class = class_tags
                .iter()
                .find_map(|(k, v)| (k == "MAIN_CLASS").then(|| v.clone()))
                .unwrap_or_else(|| "-".to_string());
            (
                item.title.clone(),
                item.smiles.clone(),
                Some(category),
                Some(main_class),
                Some("-".to_string()),
                Some(item.exact_mass),
                item.precursor_mz,
                item.adduct.clone(),
                item.class_matches.clone(),
            )
        })
        .collect()
}
